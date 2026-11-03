"""Native transport admission controls; loopback only, no provider credentials."""
import copy
import http.server
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import threading
import unittest

import authority
from contracts import canonical, digest, frozen_request_digest
from local_store import initialize_local_store, LocalAtomicStore
from native_http import JOURNAL, NativeHttpCapture, normalize_journal
import test_authority_580 as authority_controls

QUESTION = "What is the answer?"
MODEL = "fixture-model"


def first_binding(job_id, request, sha, receipt):
    case_id, arm = job_id.rsplit("-", 1)
    return frozen_request_digest(job_id=job_id, case_id=case_id, arm=arm, question=QUESTION,
                                 request=request, request_sha256=sha, provider="mistral",
                                 model=MODEL, receipt_sha256=receipt)


def encode(events, binding):
    previous = "0" * 64
    rows = []
    for sequence, event in enumerate(events):
        row = canonical({"contract": JOURNAL, "sequence": sequence, "previous_sha256": previous,
                         "binding_sha256": digest(canonical(binding)), "event": event})
        rows.append(row + b"\n")
        previous = digest(row)
    return b"".join(rows)


class JournalValidationTests(unittest.TestCase):
    def setUp(self):
        request = {"task": QUESTION}
        self.trace = {"contract": "reason-natural-diagnostic-trace-v1", "provider": "mistral",
                      "model": MODEL, "completion_status": "partial", "events": [
            {"kind": "model_request", "sequence": 0, "correlation_id": "control", "attempt": 1,
             "provider": "mistral", "model": MODEL, "request": request, "request_sha256": "a" * 64},
            {"kind": "model_response", "sequence": 1, "correlation_id": "control", "attempt": 1,
             "provider": "mistral", "model": MODEL, "response": {"provider_attempts": 1,
              "usage": {"input_tokens": 2, "output_tokens": 3, "total_tokens": 5}}}]}
        self.binding = {"contract": "reason-cli-normal-v3-native-http-binding-v1",
                        "question": QUESTION, "provider": "mistral", "model": MODEL,
                        "legacy_receipt_sha256": "b" * 64, "job": {
                            "job_id": "case-01-baseline", "max_provider_attempts": 2, "reserve_tokens": 20,
                            "frozen_request_binding_sha256": first_binding("case-01-baseline", request, "a" * 64, "b" * 64)}}
        self.events = [
            {"kind": "binding", "max_attempts": 2, "token_reserve": 20},
            {"kind": "request", "request_id": "http-control-1", "provider": "mistral", "model": MODEL,
             "context": {"correlation_id": "control", "logical_attempt": 1, "request_sha256": "a" * 64},
             "wire_request_sha256": "c" * 64},
            {"kind": "response", "request_id": "http-control-1", "http_status": 200,
             "response_sha256": "d" * 64, "usage": {"input_tokens": 2, "output_tokens": 3, "total_tokens": 5},
             "failure_class": None}, {"kind": "completed", "request_id": "http-control-1"}]

    def normalized(self, events=None, binding=None, trace=None):
        return normalize_journal(encode(events or self.events, self.binding), binding or self.binding,
                                 trace or self.trace)

    def test_exact_native_requests_produce_existing_collector_schema(self):
        result = self.normalized()
        self.assertEqual(result, [{"request_id": "http-control-1", "correlation_id": "control",
                                  "logical_attempt": 1, "provider": "mistral", "model": MODEL,
                                  "http_status": 200, "input_tokens": 2, "output_tokens": 3, "total_tokens": 5}])

    def test_partial_changed_chain_wrong_sequence_and_duplicate_json_rejected(self):
        raw = encode(self.events, self.binding)
        for bad in (raw[:-1], b"\n".join(raw.splitlines()[1:]) + b"\n",
                    raw.replace(b'"sequence":1', b'"sequence":9'),
                    raw.replace(b'"http_status":200', b'"http_status":201'),
                    raw.replace(b'"sequence":0', b'"sequence":0,"sequence":0')):
            with self.subTest(raw=bad[:80]), self.assertRaises(ValueError):
                normalize_journal(bad, self.binding, self.trace)

    def test_foreign_binding_stale_request_and_relabelled_plan_rejected(self):
        for mutate in (lambda x: x.update(model="foreign"),
                       lambda x: x["job"].update(frozen_request_binding_sha256="e" * 64),
                       lambda x: x.update(legacy_receipt_sha256="e" * 64)):
            changed = copy.deepcopy(self.binding); mutate(changed)
            with self.assertRaises(ValueError): self.normalized(binding=changed)
        changed = copy.deepcopy(self.events)
        changed[1]["context"]["request_sha256"] = "e" * 64
        with self.assertRaisesRegex(ValueError, "trace_request_mismatch"): self.normalized(events=changed)

    def test_unknown_usage_non2xx_and_aborted_never_normalize_for_scoring(self):
        for mutate in (lambda x: x[2]["usage"].update(total_tokens=None),
                       lambda x: x[2]["usage"].update(total_tokens=999),
                       lambda x: x[2].update(http_status=429),
                       lambda x: x[2].update(failure_class="provider_error"),
                       lambda x: x.__setitem__(2, {"kind": "aborted", "request_id": "http-control-1"})):
            changed = copy.deepcopy(self.events); mutate(changed)
            with self.assertRaises(ValueError): self.normalized(events=changed)

    def test_missing_reordered_duplicated_and_internal_retry_records_rejected(self):
        for events in (self.events[:2], self.events + self.events[1:],
                       [self.events[0], self.events[2], self.events[1]]):
            with self.assertRaises(ValueError): self.normalized(events=events)
        trace = copy.deepcopy(self.trace)
        trace["events"][1]["response"]["provider_attempts"] = 2
        with self.assertRaisesRegex(ValueError, "attempt_count_mismatch"): self.normalized(trace=trace)


class LoopbackServer:
    def __init__(self, *, status=200, usage=True, malformed=False, block=False):
        self.requests, self.responses = [], []
        self.started, self.release = threading.Event(), threading.Event()
        owner = self
        class Handler(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                owner.requests.append(self.rfile.read(int(self.headers["Content-Length"])))
                owner.started.set()
                if block: owner.release.wait(10)
                envelope = {"model": MODEL, "choices": [{"finish_reason": "stop", "message": {
                    "content": '{"claims":[],"inferences":[]}'}}]}
                if usage: envelope["usage"] = {"prompt_tokens": 2, "completion_tokens": 3, "total_tokens": 5}
                body = b"{malformed" if malformed else canonical(envelope)
                owner.responses.append(body)
                try:
                    self.send_response(status); self.send_header("Content-Length", str(len(body)))
                    self.end_headers(); self.wfile.write(body)
                except (BrokenPipeError, ConnectionResetError): pass
            def log_message(self, *args): pass
        self.server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
        self.url = f"http://127.0.0.1:{self.server.server_port}/v1/"
        self.worker = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.worker.start()

    def close(self):
        self.release.set(); self.server.shutdown(); self.server.server_close(); self.worker.join(10)


@unittest.skipUnless(os.environ.get("REASON_V3_NATIVE_CLI"), "set REASON_V3_NATIVE_CLI for actual native subprocess controls")
class ActualNativeCliTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.binary = str(Path(os.environ["REASON_V3_NATIVE_CLI"]).resolve())
        authority_controls.AuthorityTests.setUpClass()
        cls.template = authority_controls.AuthorityTests()
        # A provider-free component control supplies this synthetic request
        # template BEFORE freezing the separate local-authority test cohort.
        # This is not a production admission or a provider request-plan proof.
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp).resolve(); server = LoopbackServer()
            fd = os.open(path / "preflight.jsonl", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            try:
                r = cls.run_native(server, fd, path / "trace.json", "a" * 64)
                trace = json.loads((path / "trace.json").read_bytes())
                cls.expected_first = next(e for e in trace["events"] if e["kind"] == "model_request")
                assert server.requests and r.returncode in (0, 1)
            finally:
                os.close(fd); server.close()

    @classmethod
    def tearDownClass(cls): authority_controls.AuthorityTests.tearDownClass()

    @classmethod
    def argv(cls, server, fd, trace, binding):
        return [cls.binary, "--no-config", "--provider", "mistral", "--model", MODEL,
                "--max-model-calls", "1", "--max-tokens", "32", "--max-total-tokens", "100",
                "--format", "json", "--diagnostic-trace", str(trace),
                "--evaluation-http-journal-fd", str(fd), "--evaluation-http-binding-sha256", binding,
                "--evaluation-http-max-attempts", "2", "--evaluation-http-token-reserve", "40",
                "--evaluation-http-loopback-url", server.url, QUESTION]

    @classmethod
    def run_native(cls, server, fd, trace, binding):
        return subprocess.run(cls.argv(server, fd, trace, binding), env={"PATH": "/usr/bin:/bin",
                              "MISTRAL_API_KEY": "fixture-only-key", "NO_PROXY": "127.0.0.1"},
                              pass_fds=(fd,), umask=0o077, capture_output=True, timeout=20)

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.plan = copy.deepcopy(self.template.plan)
        first = self.expected_first
        for job in self.plan["jobs"]:
            self.plan["frozen_request_bindings"][job["id"]] = first_binding(
                job["id"], first["request"], first["request_sha256"], digest(self.template.local_receipt))
        profile = initialize_local_store(self.root / "authority")
        self.store = LocalAtomicStore(profile, role="executor")
        issuer = LocalAtomicStore(profile, role="issuer")
        binding = authority.AuthorityBinding(**{**vars(self.template.binding), "cli_build_sha256": digest(Path(self.binary).read_bytes())})
        _, receipt = self.template.fixture("native-http", plan=self.plan, binding=binding)
        issuer.create(f"{authority.PREFIX}/receipts/native-http.json", canonical(receipt))
        self.session = authority.TrustedAuthority(self.store).start("native-http", self.plan, self.template.local_receipt, binding)
        self.job_id = self.plan["jobs"][0]["id"]

    def capture(self):
        self.session.reserve(self.job_id)
        capture = NativeHttpCapture(self.session, self.job_id, QUESTION, self.root / "capture")
        self.session.before_spawn(self.job_id)
        self.addCleanup(capture.close)
        return capture

    def test_actual_cli_journal_binds_real_http_trace_and_private_local_authority(self):
        capture = self.capture(); server = LoopbackServer(); self.addCleanup(server.close)
        r = self.run_native(server, capture.fd, capture.directory / "trace.json", digest(canonical(capture.binding)))
        capture.close()
        events = capture.verify()
        self.assertEqual(len(events), 1)
        self.assertEqual(len(server.requests), 1)
        rows = [json.loads(line) for line in (capture.directory / "http-journal.jsonl").read_bytes().splitlines()]
        self.assertEqual(rows[1]["event"]["wire_request_sha256"], digest(server.requests[0]))
        self.assertEqual(rows[2]["event"]["response_sha256"], digest(server.responses[0]))
        self.assertIsNotNone(self.store.read(authority._key("native-http", "http/" + self.job_id + "/capture")))
        self.assertNotIn(b"fixture-only-key", (capture.directory / "http-journal.jsonl").read_bytes())
        # The one-call control intentionally exhausts the CLI budget before
        # final rendering. Valid HTTP evidence must not erase that CLI stop.
        self.assertEqual(r.returncode, 1)
        self.assertFalse(self.session.observe(self.job_id, tokens=5, attempts=1, operational_failure="cli_budget_control"))
        with self.assertRaisesRegex(ValueError, "halted"): self.session.reserve(self.plan["jobs"][1]["id"])

    def test_unknown_usage_records_real_response_and_blocks_authority_advance(self):
        capture = self.capture(); server = LoopbackServer(usage=False); self.addCleanup(server.close)
        self.run_native(server, capture.fd, capture.directory / "trace.json", digest(canonical(capture.binding)))
        capture.close()
        self.assertEqual(len(server.requests), 1)
        with self.assertRaises(ValueError): capture.verify()
        self.assertFalse(self.session.observe(self.job_id, tokens=0, attempts=1))
        observed = json.loads(self.store.read(authority._key("native-http", "observed/" + self.job_id)))
        self.assertIsNone(observed["tokens"])
        self.assertEqual(observed["accounting_charge_tokens"], capture.binding["job"]["reserve_tokens"])

    def test_cli_totals_cannot_override_verified_transport_usage(self):
        capture = self.capture(); server = LoopbackServer(); self.addCleanup(server.close)
        self.run_native(server, capture.fd, capture.directory / "trace.json", digest(canonical(capture.binding)))
        capture.close(); capture.verify()
        self.assertFalse(self.session.observe(self.job_id, tokens=0, attempts=1))
        observed = json.loads(self.store.read(authority._key("native-http", "observed/" + self.job_id)))
        self.assertIsNone(observed["tokens"])
        self.assertEqual(observed["operational_failure"], "native_http_usage_mismatch")

    def test_sigkill_leaves_durable_request_and_burned_reservation(self):
        capture = self.capture(); server = LoopbackServer(block=True); self.addCleanup(server.close)
        process = subprocess.Popen(self.argv(server, capture.fd, capture.directory / "trace.json", digest(canonical(capture.binding))),
                                   env={"PATH": "/usr/bin:/bin", "MISTRAL_API_KEY": "fixture-only-key", "NO_PROXY": "127.0.0.1"},
                                   **capture.subprocess_options(), stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            self.assertTrue(server.started.wait(10))
            process.send_signal(signal.SIGKILL); process.communicate(timeout=10)
        finally:
            if process.poll() is None: process.kill(); process.communicate(timeout=10)
            capture.close(); server.release.set()
        self.assertEqual(process.returncode, -signal.SIGKILL)
        rows = (capture.directory / "http-journal.jsonl").read_bytes().splitlines()
        self.assertEqual(len(rows), 2)
        self.assertEqual(json.loads(rows[-1])["event"]["kind"], "request")
        self.assertFalse(self.session.observe(self.job_id, tokens=None, attempts=None, operational_failure="killed"))
        with self.assertRaisesRegex(ValueError, "halted"): self.session.reserve(self.plan["jobs"][1]["id"])

    def test_existing_or_readonly_journal_fd_and_external_control_url_send_nothing(self):
        server = LoopbackServer(); self.addCleanup(server.close)
        path = self.root / "already-used.jsonl"; path.write_bytes(b"used"); path.chmod(0o600)
        for flags in (os.O_WRONLY, os.O_RDONLY):
            fd = os.open(path, flags)
            try:
                r = self.run_native(server, fd, self.root / "rejected-trace.json", "a" * 64)
                self.assertNotEqual(r.returncode, 0)
            finally: os.close(fd)
        fd = os.open(self.root / "invalid-url.jsonl", os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        try:
            server.url = "https://example.invalid/v1/"
            r = self.run_native(server, fd, self.root / "url-trace.json", "a" * 64)
            self.assertNotEqual(r.returncode, 0)
        finally: os.close(fd)
        self.assertEqual(server.requests, [])

    def test_loopback_control_bypasses_ambient_proxy(self):
        capture = self.capture()
        server, proxy = LoopbackServer(), LoopbackServer()
        self.addCleanup(server.close); self.addCleanup(proxy.close)
        subprocess.run(self.argv(server, capture.fd, capture.directory / "trace.json", digest(canonical(capture.binding))),
                       env={"PATH": "/usr/bin:/bin", "MISTRAL_API_KEY": "fixture-only-key",
                            "HTTP_PROXY": proxy.url, "ALL_PROXY": proxy.url},
                       **capture.subprocess_options(), capture_output=True, timeout=20)
        capture.close(); capture.verify()
        self.assertEqual(len(server.requests), 1)
        self.assertEqual(proxy.requests, [])


if __name__ == "__main__": unittest.main()
