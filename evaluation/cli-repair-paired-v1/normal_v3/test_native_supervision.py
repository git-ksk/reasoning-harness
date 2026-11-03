"""#540 real process controls; loopback only, never provider credentials."""
import json
import os
from pathlib import Path
import platform
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
import shutil
from unittest.mock import patch
from contracts import canonical, digest

from native_lane import capture_process, NativeCancelled, validate_argv
from native_bundle import MAC_PROCESS_POLICY
import authority
import test_native_admission as controls


def wait_for(predicate, timeout=5):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        if predicate(): return
        time.sleep(.02)
    raise AssertionError("process control did not reach its expected state")


def not_running(pid):
    result = subprocess.run(["/bin/ps", "-p", str(pid), "-o", "stat="], capture_output=True, text=True)
    return result.returncode == 1 or result.stdout.lstrip().startswith("Z")


class CancellationControls(unittest.TestCase):
    def test_timeout_first_requests_sigint_and_preserves_grace_output(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            code = "import signal,sys,time;signal.signal(signal.SIGINT,lambda *a:(print('cancelled',file=sys.stderr,flush=True),sys.exit(0)));print('partial',flush=True);time.sleep(30)"
            rc, complete = capture_process([sys.executable, "-I", "-S", "-c", code], directory=root,
                environment={}, pass_fds=(), timeout=.3, cancel_grace=.5)
            execution = json.loads((root / "execution.json").read_bytes())
            self.assertEqual(rc, 0); self.assertFalse(complete)
            self.assertTrue(execution["sigint_sent"]); self.assertFalse(execution["sigkill_sent"])
            self.assertTrue(execution["process_reaped"])
            self.assertIn(b"partial", (root / "cli.json").read_bytes())
            self.assertIn(b"cancelled", (root / "stderr.txt").read_bytes())

    def test_uncooperative_child_is_killed_after_bounded_grace(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve(); started = time.monotonic()
            rc, complete = capture_process([sys.executable, "-I", "-S", "-c",
                "import signal,time;signal.signal(signal.SIGINT,signal.SIG_IGN);print('partial',flush=True);time.sleep(30)"],
                directory=root, environment={}, pass_fds=(), timeout=.3, cancel_grace=.2)
            self.assertEqual(rc, -signal.SIGKILL); self.assertFalse(complete)
            self.assertTrue(json.loads((root / "execution.json").read_bytes())["sigkill_sent"])
            self.assertLess(time.monotonic() - started, 3)

    def test_event_cancellation_reaps_child_and_cannot_look_successful(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve(); stop = threading.Event(); failures = []
            def run():
                try: capture_process([sys.executable, "-I", "-S", "-c", "import time;print('partial',flush=True);time.sleep(30)"],
                    directory=root, environment={}, pass_fds=(), timeout=20, cancel_grace=.2, stop_event=stop)
                except NativeCancelled as e: failures.append(e)
            worker = threading.Thread(target=run); worker.start()
            wait_for(lambda: (root / "supervision.json").exists()); stop.set(); worker.join(4)
            self.assertFalse(worker.is_alive()); self.assertEqual(len(failures), 1)
            result = json.loads((root / "execution.json").read_bytes())
            self.assertFalse(result["streams_complete"]); self.assertTrue(result["process_reaped"])

    def test_pre_spawn_cancel_does_not_start_command(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve(); stop = threading.Event(); stop.set()
            with self.assertRaises(NativeCancelled):
                capture_process(["/not/an/executable"], directory=root, environment={}, pass_fds=(),
                    timeout=2, stop_event=stop)
            record = json.loads((root / "execution.json").read_bytes())
            self.assertIsNone(record["exit_code"]); self.assertFalse(record["process_reaped"])

    def test_no_fork_profile_rejects_external_program_before_planning(self):
        class Bundle:
            data = {"process_policy": MAC_PROCESS_POLICY, "files": {}}
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "config.json"; path.write_text('{"resolver":{"program":"/untrusted"}}')
            for option in (["--resolver-command", "/untrusted"], ["--config", str(path)]):
                argv = ["/reason", "--no-config", "--plain", "--format", "json", *option, "question"]
                with self.assertRaisesRegex(ValueError, "external_program_rejected"):
                    validate_argv(argv, "/reason", "question", Bundle())


@unittest.skipUnless(os.environ.get("REASON_V3_NATIVE_CLI"), "set REASON_V3_NATIVE_CLI for native supervision")
class ActualLifelineControls(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve(); self.binary = os.environ["REASON_V3_NATIVE_CLI"]

    def launch(self):
        parent, child = socket.socketpair()
        argv = [self.binary, "--no-config", "--plain", "--provider", "mistral", "--model", "fixture-model",
                "--evaluation-request-plan", str(self.root / "plan.json"), "question",
                "--evaluation-lifeline-fd", str(child.fileno())]
        if platform.system() == "Darwin": argv += ["--evaluation-no-fork"]
        process = subprocess.Popen(argv, env={"PATH": "/usr/bin:/bin"}, pass_fds=(child.fileno(),),
            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        child.close(); parent.settimeout(5)
        self.addCleanup(parent.close)
        self.addCleanup(lambda: (process.kill(), process.wait()) if process.poll() is None else None)
        return process, parent

    def test_dead_controller_before_go_permits_no_request_plan(self):
        process, parent = self.launch()
        self.assertEqual(parent.recv(100), b"reason-evaluation-lifeline-v1\n")
        self.assertFalse((self.root / "plan.json").exists())
        parent.close(); out, _ = process.communicate(timeout=3)
        self.assertNotEqual(process.returncode, 0); self.assertFalse(out)
        self.assertFalse((self.root / "plan.json").exists())

    def test_invalid_controller_ack_is_rejected(self):
        process, parent = self.launch(); parent.recv(100); parent.sendall(b"X")
        process.communicate(timeout=3)
        self.assertNotEqual(process.returncode, 0); self.assertFalse((self.root / "plan.json").exists())

    def test_regular_file_cannot_be_a_lifeline(self):
        path = self.root / "regular"; path.write_bytes(b"G")
        with path.open("rb") as stream:
            result = subprocess.run([self.binary, "--no-config", "question", "--evaluation-lifeline-fd", str(stream.fileno())],
                env={}, pass_fds=(stream.fileno(),), capture_output=True, timeout=5)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"private socket pair", result.stderr)

    def controller_signal(self, sig):
        server = controls.ScriptedServer("hold"); self.addCleanup(server.close)
        marker = self.root / "controller.json"
        code = '''
import json,sys
from pathlib import Path
import test_native_admission as controls
from native_lane import NativeLane
c=controls.ActualNativeAdmissionTests
c.setUpClass(); instance=c(); instance.setUp()
# The production authority is operator-owned persistent storage. Prevent the
# fixture's TemporaryDirectory finalizer from deleting it on graceful exit.
instance.tmp._finalizer.detach()
lane=NativeLane(session=instance.session,plan=instance.plan,bundle=instance.bundle,
 policy_bytes=instance.reviewer.policy_bytes,catalog_bytes=instance.catalog_bytes,
 output=instance.root/'lane',control_url=sys.argv[2])
Path(sys.argv[1]).write_text(json.dumps({'root':str(instance.root),'extra_roots':[str(c.root),str(c.template.path)],'profile':instance.store.profile.to_json(),
 'first_job':instance.plan['jobs'][0]['id']}))
lane.run_next()
'''
        # Only the known control module path is supplied to this isolated
        # test driver; the native child itself receives the minimal fixed env.
        env = {**os.environ, "PYTHONDONTWRITEBYTECODE": "1", "PYTHONPATH": str(Path(__file__).resolve().parent)}
        driver = subprocess.Popen([sys.executable, "-B", "-c", code, str(marker), server.url],
            env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        info = None; pid = None
        try:
            wait_for(lambda: marker.exists(), 15); info = json.loads(marker.read_bytes())
            root = Path(info["root"])
            self.addCleanup(shutil.rmtree, root, True)
            for extra in info["extra_roots"]: self.addCleanup(shutil.rmtree, extra, True)
            record = root / "lane" / info["first_job"] / "supervision.json"
            wait_for(lambda: record.exists(), 10); pid = json.loads(record.read_bytes())["pid"]
            wait_for(lambda: len(server.requests) == 1, 5)
            os.kill(driver.pid, sig); driver.communicate(timeout=5)
            wait_for(lambda: not_running(pid), 5)
            self.assertEqual(len(server.requests), 1)
            from local_store import LocalAtomicStore, LocalStoreProfile
            store = LocalAtomicStore(LocalStoreProfile.from_json(info["profile"]), role="executor")
            self.assertIsNotNone(store.read(authority._key("native-582", "reserved/" + info["first_job"])))
            self.assertIsNotNone(store.read(authority._key("native-582", "inflight/" + info["first_job"])))
            self.assertIsNone(store.read(authority._key("native-582", "completed")))
            if sig == signal.SIGTERM:
                result = json.loads((root / "lane" / info["first_job"] / "execution.json").read_bytes())
                self.assertFalse(result["streams_complete"]); self.assertTrue(result["process_reaped"])
        finally:
            if driver.poll() is None: driver.kill(); driver.communicate(timeout=5)
            if pid is not None and not not_running(pid):
                try: os.killpg(pid, signal.SIGKILL)
                except ProcessLookupError: pass

    def test_sigkill_of_actual_controller_stops_native_http_child_and_keeps_reserve(self):
        self.controller_signal(signal.SIGKILL)

    def test_sigterm_of_actual_controller_cancels_and_preserves_partial_records(self):
        self.controller_signal(signal.SIGTERM)


@unittest.skipUnless(os.environ.get("REASON_V3_NATIVE_CLI"), "set REASON_V3_NATIVE_CLI for native cancellation")
class NativeLaneStopControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        controls.ActualNativeAdmissionTests.setUpClass()
    @classmethod
    def tearDownClass(cls):
        controls.ActualNativeAdmissionTests.tearDownClass()
    def setUp(self):
        self.control = controls.ActualNativeAdmissionTests()
        self.control.setUp(); self.addCleanup(self.control.doCleanups)

    def test_cancel_pending_review_is_durable_and_no_second_http_starts(self):
        c = self.control; lane, server = c.lane(); packet, expected = lane.run_next()
        count = len(server.requests); lane.cancel()
        record = c.store.read(authority._key("native-582", "halted"))
        self.assertEqual(json.loads(record)["inflight_job_id"], c.plan["jobs"][0]["id"])
        self.assertGreater(json.loads(record)["charged_reservation_tokens"], 0)
        with self.assertRaisesRegex(ValueError, "cancelled_or_halted"): lane.run_next()
        with self.assertRaisesRegex(ValueError, "cancelled_or_halted"): lane.accept_review(c.sign(packet, expected))
        self.assertEqual(len(server.requests), count)
        with self.assertRaises(authority.AuthorityFailure): c.session.complete()

    def test_cancel_before_first_job_has_no_http_or_reservation(self):
        c = self.control; lane, server = c.lane(); lane.cancel()
        with self.assertRaisesRegex(ValueError, "cancelled_or_halted"): lane.run_next()
        self.assertEqual(server.requests, [])
        self.assertEqual(c.session._reserved, 0)

    def test_changed_supervision_record_cannot_authorize_advance(self):
        c = self.control; lane, _ = c.lane(); packet, expected = lane.run_next()
        path = lane.output / expected["job_id"] / "supervision.json"
        path.write_bytes(b"{}")
        with self.assertRaisesRegex(ValueError, "supervision_changed"):
            lane.accept_review(c.sign(packet, expected))
        self.assertTrue(c.session._halted)

    def test_supervision_write_failure_permits_no_http_before_go(self):
        c = self.control; lane, server = c.lane()
        import native_lane
        original = native_lane.exclusive_json
        def fail(path, value):
            if Path(path).name == "supervision.json": raise OSError("control disk failure")
            return original(path, value)
        with patch.object(native_lane, "exclusive_json", side_effect=fail):
            with self.assertRaises(OSError): lane.run_next()
        self.assertEqual(server.requests, []); self.assertTrue(c.session._halted)
        self.assertGreater(c.session._reserved, 0)
        result = json.loads(next(lane.output.glob("*/execution.json")).read_bytes())
        self.assertFalse(result["streams_complete"]); self.assertTrue(result["process_reaped"])

    def test_cancel_during_actual_http_stops_child_and_blocks_next_job(self):
        c = self.control; lane, server = c.lane("hold"); failures = []
        def run():
            try: lane.run_next()
            except NativeCancelled as e: failures.append(e)
        worker = threading.Thread(target=run); worker.start()
        try:
            wait_for(lambda: len(server.requests) == 1, 8); lane.cancel(); worker.join(5)
            self.assertFalse(worker.is_alive()); self.assertEqual(len(failures), 1)
            self.assertEqual(len(server.requests), 1)
            result = json.loads(next(lane.output.glob("*/execution.json")).read_bytes())
            self.assertFalse(result["streams_complete"]); self.assertTrue(result["process_reaped"])
            with self.assertRaisesRegex(ValueError, "cancelled_or_halted"): lane.run_next()
            self.assertGreater(c.session._reserved, 0)
        finally:
            lane.stop_event.set(); server.release_event.set(); worker.join(5)


class AuthorityCancellationControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        controls.legacy_controls.AuthorityTests.setUpClass()
        cls.template = controls.legacy_controls.AuthorityTests()
    @classmethod
    def tearDownClass(cls): controls.legacy_controls.AuthorityTests.tearDownClass()

    def test_cancellation_racing_reservation_retains_exact_charge(self):
        t = self.template; store, _ = t.fixture(); session = t.start(store)
        entered, release, stop_started = threading.Event(), threading.Event(), threading.Event()
        create = store.create; failures = []
        def slow(key, value):
            if "/reserved/" in key: entered.set(); self.assertTrue(release.wait(3))
            return create(key, value)
        store.create = slow
        def reserve():
            try: session.reserve(t.plan["jobs"][0]["id"])
            except BaseException as e: failures.append(e)
        def stop():
            stop_started.set()
            try: session.stop("control cancellation")
            except BaseException as e: failures.append(e)
        worker = threading.Thread(target=reserve); worker.start(); self.assertTrue(entered.wait(3))
        stopper = threading.Thread(target=stop); stopper.start(); self.assertTrue(stop_started.wait(3)); release.set()
        worker.join(3); stopper.join(3)
        self.assertFalse(worker.is_alive()); self.assertFalse(stopper.is_alive()); self.assertFalse(failures)
        record = json.loads(store.read(authority._key("synthetic-run-580", "halted")))
        self.assertGreater(record["charged_reservation_tokens"], 0)
        self.assertEqual(record["charged_reservation_tokens"], session._reserved)
        self.assertEqual(record["inflight_job_id"], t.plan["jobs"][0]["id"])
        with self.assertRaises(authority.AuthorityFailure): session.before_spawn(record["inflight_job_id"])

    def test_failed_stop_write_never_restores_reservation_or_permission(self):
        t = self.template; store, _ = t.fixture(); session = t.start(store)
        session.reserve(t.plan["jobs"][0]["id"]); amount = session._reserved
        store.fail_before = "/halted.json"
        with self.assertRaises(OSError): session.stop("control disk failure")
        self.assertTrue(session._halted); self.assertEqual(session._reserved, amount)
        with self.assertRaises(authority.AuthorityFailure): session.before_spawn(t.plan["jobs"][0]["id"])

    def test_completed_run_is_terminal_and_cannot_race_cancellation(self):
        t = self.template; store, _ = t.fixture(); session = t.start(store); t.all_48(session)
        with self.assertRaisesRegex(authority.AuthorityFailure, "already_completed"): session.stop("too late")
        self.assertIsNone(store.read(authority._key("synthetic-run-580", "halted")))

    def test_stop_record_blocks_first_score_even_if_completion_is_present(self):
        t = self.template; store, _ = t.fixture(); session = t.start(store); t.all_48(session)
        store.objects[authority._key("synthetic-run-580", "halted")] = canonical({"reason": "control stop"})
        with self.assertRaisesRegex(authority.AuthorityFailure, "explicitly_stopped"):
            authority.TrustedAuthority(store).claim_score("synthetic-run-580", t.plan, t.local_receipt, t.binding,
                observation_index_sha256="1"*64, review_manifest_sha256="2"*64,
                observation_hashes={j["id"]: digest(canonical({"id": j["id"]})) for j in t.plan["jobs"]})
