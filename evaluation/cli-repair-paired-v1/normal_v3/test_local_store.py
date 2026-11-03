"""Native local authority acceptance: private files and actual OS processes."""
import json
import os
import signal
import socket
import subprocess
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import authority
import local_store
import test_authority_580 as controls
from contracts import canonical, digest


class LocalAuthorityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        controls.AuthorityTests.setUpClass()

    @classmethod
    def tearDownClass(cls):
        controls.AuthorityTests.tearDownClass()

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.profile = local_store.initialize_local_store(self.root / "authority")
        self.issuer = local_store.LocalAtomicStore(self.profile, role="issuer")
        self.store = local_store.LocalAtomicStore(self.profile, role="executor")
        self.fixture = controls.AuthorityTests()
        self.plan, self.binding, self.receipt_bytes = (
            self.fixture.plan, self.fixture.binding, self.fixture.local_receipt)
        self.service = authority.TrustedAuthority(self.store)

    def publish(self, evaluation="local-test"):
        _, receipt = self.fixture.fixture(evaluation)
        self.issuer.create(f"{authority.PREFIX}/receipts/{evaluation}.json", canonical(receipt))
        return receipt

    def start(self, evaluation="local-test"):
        return self.service.start(evaluation, self.plan, self.receipt_bytes, self.binding)

    def score_args(self):
        return {"observation_index_sha256": "1" * 64, "review_manifest_sha256": "2" * 64,
                "observation_hashes": {j["id"]: digest(canonical({"id": j["id"]})) for j in self.plan["jobs"]}}

    def child_data(self):
        for name, value in (("profile", self.profile.to_json()), ("plan", self.plan),
                            ("binding", vars(self.binding)), ("score", self.score_args())):
            (self.root / (name + ".json")).write_bytes(canonical(value))
        (self.root / "receipt.bin").write_bytes(self.receipt_bytes)

    def child(self, mode, evaluation="local-test"):
        code = r'''
import json,sys,time
from pathlib import Path
sys.path.insert(0,sys.argv[1])
import authority,local_store
root=Path(sys.argv[2]);mode=sys.argv[3];evaluation=sys.argv[4]
profile=local_store.LocalStoreProfile.from_json(json.loads((root/'profile.json').read_bytes()))
role='evaluator' if mode=='score' else 'executor'
service=authority.TrustedAuthority(local_store.LocalAtomicStore(profile,role=role))
plan=json.loads((root/'plan.json').read_bytes())
binding=authority.AuthorityBinding(**json.loads((root/'binding.json').read_bytes()))
raw=(root/'receipt.bin').read_bytes()
try:
 if mode=='score':
  args=json.loads((root/'score.json').read_bytes())
  hashes=args['observation_hashes']
  args['observation_hashes']={job['id']:hashes[job['id']] for job in plan['jobs']}
  service.claim_score(evaluation,plan,raw,binding,**args)
 else:
  session=service.start(evaluation,plan,raw,binding)
  if mode=='crash':
   job=plan['jobs'][0]['id'];session.reserve(job);session.before_spawn(job)
   print('inflight',flush=True);time.sleep(60)
 print('won',flush=True)
except FileExistsError: print('lost',flush=True)
'''
        return subprocess.Popen([sys.executable, "-I", "-B", "-c", code,
                                 str(Path(authority.__file__).parent), str(self.root), mode, evaluation],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

    def test_private_profile_and_no_reset_or_implicit_enrollment(self):
        self.assertEqual(Path(self.profile.root).stat().st_mode & 0o777, 0o700)
        self.assertEqual((Path(self.profile.root) / local_store.IDENTITY_FILE).stat().st_mode & 0o777, 0o600)
        self.assertEqual(local_store.LocalStoreProfile.from_json(self.profile.to_json()), self.profile)
        with self.assertRaises(FileExistsError):
            local_store.initialize_local_store(Path(self.profile.root))
        with self.assertRaises(FileNotFoundError):
            local_store.LocalAtomicStore(replace(self.profile, root=str(self.root / "absent")), role="executor")
        self.assertFalse((self.root / "absent").exists())

    def test_role_checks_and_private_create_only_objects(self):
        receipt = self.publish()
        self.assertEqual(self.store.read(f"{authority.PREFIX}/receipts/local-test.json"), canonical(receipt))
        with self.assertRaisesRegex(ValueError, "role_denied"):
            self.store.create(f"{authority.PREFIX}/receipts/forged.json", b"{}")
        with self.assertRaisesRegex(ValueError, "role_denied"):
            self.store.create(authority._key("local-test", "first-score"), b"{}")
        self.start()
        path = Path(self.profile.root) / authority._key("local-test", "run")
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        with self.assertRaises(FileExistsError):
            self.store.create(authority._key("local-test", "run"), b"{}")

    def test_maximum_evaluation_id_is_supported(self):
        evaluation = "x" * 120
        self.publish(evaluation)
        self.start(evaluation)
        self.assertIsNotNone(self.store.read(authority._key(evaluation, "run")))

    def test_two_actual_processes_one_run_claim(self):
        self.publish(); self.child_data()
        children = [self.child("run") for _ in range(2)]
        results = [p.communicate(timeout=20) for p in children]
        self.assertEqual([p.returncode for p in children], [0, 0], results)
        self.assertCountEqual([out.strip() for out, _ in results], ["won", "lost"])

    def test_two_actual_processes_one_first_score_claim(self):
        self.publish(); self.fixture.all_48(self.start()); self.child_data()
        children = [self.child("score") for _ in range(2)]
        results = [p.communicate(timeout=20) for p in children]
        self.assertEqual([p.returncode for p in children], [0, 0], results)
        self.assertCountEqual([out.strip() for out, _ in results], ["won", "lost"])

    def test_relabelled_evaluation_id_still_conflicts_on_same_cohort(self):
        self.publish("first"); self.start("first"); self.publish("second")
        with self.assertRaises(FileExistsError): self.start("second")
        self.assertIsNone(self.store.read(authority._key("second", "run")))

    def test_native_authority_needs_no_cloud_sdk_or_network(self):
        with patch.dict(sys.modules, {"google": None, "google.cloud": None}), \
                patch.object(socket, "socket", side_effect=AssertionError("network forbidden")):
            self.publish()
            self.fixture.all_48(self.start())
            evaluator = authority.TrustedAuthority(local_store.LocalAtomicStore(self.profile, role="evaluator"))
            evaluator.claim_score("local-test", self.plan, self.receipt_bytes, self.binding, **self.score_args())
            self.assertIsNotNone(self.store.read(authority._key("local-test", "first-score")))

    def test_sigkill_retains_inflight_and_reservation_and_prevents_restart(self):
        self.publish(); self.child_data()
        child = self.child("crash")
        try:
            self.assertEqual(child.stdout.readline().strip(), "inflight")
            os.kill(child.pid, signal.SIGKILL)
            child.communicate(timeout=10)
            self.assertEqual(child.returncode, -signal.SIGKILL)
            first = self.plan["jobs"][0]["id"]
            reserved = json.loads(self.store.read(authority._key("local-test", "reserved/" + first)))
            self.assertGreater(reserved["reserve_tokens"], 0)
            self.assertIsNotNone(self.store.read(authority._key("local-test", "inflight/" + first)))
            with self.assertRaises(FileExistsError): self.start()
            evaluator = authority.TrustedAuthority(local_store.LocalAtomicStore(self.profile, role="evaluator"))
            with self.assertRaisesRegex(ValueError, "not_completed"):
                evaluator.claim_score("local-test", self.plan, self.receipt_bytes, self.binding, **self.score_args())
        finally:
            if child.poll() is None: child.kill()
            child.communicate(timeout=10)

    def test_directory_replacement_alternate_root_or_changed_identity_rejected(self):
        original = Path(self.profile.root)
        original.rename(self.root / "old-authority")
        new_profile = local_store.initialize_local_store(original)
        with self.assertRaisesRegex(ValueError, "root_replaced"):
            self.store.read(authority._key("local-test", "run"))
        with self.assertRaisesRegex(ValueError, "root_replaced"):
            local_store.LocalAtomicStore(self.profile, role="executor")
        alternate = local_store.initialize_local_store(self.root / "alternate")
        with self.assertRaisesRegex(ValueError, "root_replaced"):
            local_store.LocalAtomicStore(replace(alternate, identity_bytes=new_profile.identity_bytes), role="executor")

    def test_identity_file_change_and_unsafe_root_mode_rejected(self):
        identity = Path(self.profile.root) / local_store.IDENTITY_FILE
        original = identity.read_bytes()
        identity.write_bytes(original + b"\n")
        with self.assertRaisesRegex(ValueError, "identity_changed"):
            self.store.read(authority._key("local-test", "run"))
        identity.write_bytes(original)
        Path(self.profile.root).chmod(0o755)
        with self.assertRaisesRegex(ValueError, "mode_unsafe"):
            self.store.read(authority._key("local-test", "run"))

    def test_symlink_hardlink_and_traversal_never_followed(self):
        self.publish(); self.start()
        key = authority._key("local-test", "run")
        path = Path(self.profile.root) / key
        link = self.root / "link"
        os.link(path, link)
        with self.assertRaisesRegex(ValueError, "file_unsafe"): self.store.read(key)
        link.unlink(); path.unlink(); path.symlink_to(self.root / "outside")
        with self.assertRaises(OSError): self.store.read(key)
        with self.assertRaises((OSError, FileExistsError)): self.store.create(key, b"{}")
        for bad in ("../outside", authority.PREFIX + "/../outside.json", authority.PREFIX + "//bad.json"):
            with self.assertRaises(ValueError): self.store.create(bad, b"{}")
        self.assertFalse((self.root / "outside").exists())

    def test_ambiguous_fsync_keeps_claim_burned(self):
        self.publish()
        self.store.create(authority.PREFIX + "/cohorts/warmup.json", b"{}")
        with patch.object(local_store.os, "fsync", side_effect=OSError("injected storage failure")):
            with self.assertRaises(OSError): self.start()
        with self.assertRaises(FileExistsError): self.start()
        self.assertIsNone(self.store.read(authority._key("local-test", "run")))

    def test_fifo_is_rejected_without_waiting_for_a_writer(self):
        self.publish(); self.start()
        key = authority._key("local-test", "run")
        path = Path(self.profile.root) / key
        path.unlink()
        os.mkfifo(path, mode=0o600)
        with self.assertRaisesRegex(ValueError, "file_unsafe"):
            self.store.read(key)

    def test_symlinked_intermediate_directory_never_escapes_profile(self):
        self.publish(); self.start()
        parent = Path(self.profile.root) / authority.PREFIX / "claims" / "local-test"
        original = parent.with_name("original-claim-directory")
        parent.rename(original)
        outside = self.root / "outside-directory"
        outside.mkdir(mode=0o700)
        parent.symlink_to(outside, target_is_directory=True)
        with self.assertRaises(OSError):
            self.store.create(authority._key("local-test", "inflight/new"), b"{}")
        with self.assertRaises(OSError):
            self.store.read(authority._key("local-test", "run"))
        self.assertFalse(any(outside.iterdir()))

    def test_real_48_child_scheduler_persists_to_local_authority(self):
        import offline_lane
        self.publish()
        session = self.start()
        output = self.root / "local-native-lane"
        result = offline_lane.OfflineV3Runner(
            self.plan, output, self.fixture.receipt_file, str(self.fixture.script),
            authority_session=session).run()
        self.assertTrue(result["complete"])
        self.assertEqual(len(list(output.glob("*/spawn.json"))), 48)
        self.assertIsNotNone(self.store.read(authority._key("local-test", "completed")))
        with self.assertRaises(FileExistsError): self.start()

    def test_real_unknown_usage_stops_local_runner_and_preserves_charge(self):
        import offline_lane
        from test_offline_lane import make_plan
        folder = self.root / "unknown-case"
        folder.mkdir()
        plan, receipt_file, script, _ = make_plan(folder, {"synthetic-00-baseline": "missing_usage"})
        _, published = self.fixture.fixture("unknown-local", plan=plan, local_receipt=receipt_file.read_bytes())
        self.issuer.create(f"{authority.PREFIX}/receipts/unknown-local.json", canonical(published))
        session = self.service.start("unknown-local", plan, receipt_file.read_bytes(), self.binding)
        output = self.root / "unknown-lane"
        result = offline_lane.OfflineV3Runner(plan, output, receipt_file, str(script), authority_session=session).run()
        self.assertFalse(result["complete"])
        self.assertEqual(len(list(output.glob("*/spawn.json"))), 1)
        first = plan["jobs"][0]["id"]
        self.assertIsNotNone(self.store.read(authority._key("unknown-local", "reserved/" + first)))
        self.assertIsNone(self.store.read(authority._key("unknown-local", "completed")))


if __name__ == "__main__":
    unittest.main()
