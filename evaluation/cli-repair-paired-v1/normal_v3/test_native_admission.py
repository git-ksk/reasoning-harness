"""#582 owner-trusted native controls, full streams and signed pre-advance gate."""
import copy
import http.server
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest

from nacl.signing import SigningKey
import authority
from contracts import canonical, digest, frozen_request_digest
from local_store import initialize_local_store, LocalAtomicStore
from native_admission import ADMISSION, CATALOG, DOMAIN, capture_display, source_catalog, verify_pre_advance
from native_bundle import NativeBundle
from native_lane import CONTROL, NativeLane, plan_first_request, private_bytes, capture_process
import review_auth
import test_authority_580 as legacy_controls

QUESTION = "What can be verified from this source?"


class ControlReviewer:
    def __init__(self):
        self.key = SigningKey.generate()
        self.policy = {"contract": review_auth.POLICY, "policy_id": "native-control-only",
            "principals": [
                {"subject_id": "executor", "aliases": ["trusted-executor"], "role": "agent", "conflict_groups": ["execution"]},
                {"subject_id": "reviewer", "aliases": ["trusted-evaluator"], "role": "agent", "conflict_groups": ["review"]}],
            "keys": [{"key_id": "control-key", "subject_id": "reviewer", "public_key_hex": self.key.verify_key.encode().hex(), "revoked": False}]}
        self.policy_bytes = canonical(self.policy)

    def sign(self, expected, stdout, stderr, *, decision="safe", segments=None):
        if segments is None:
            segments = {name: ([{"start_byte": 0, "end_byte": len(raw), "label": "nonfact", "source_ids": []}] if raw else [])
                        for name, raw in (("stdout", stdout), ("stderr", stderr))}
        payload = {**expected, "policy_sha256": digest(self.policy_bytes), "decision": decision,
                   "full_sources_reviewed": True, "segments": segments,
                   "notes": "Synthetic known-content loopback control; not real semantic acceptance."}
        return canonical({"contract": ADMISSION, "algorithm": "Ed25519", "key_id": "control-key",
                          "payload": payload, "signature_hex": self.key.sign(DOMAIN + canonical(payload)).signature.hex()})


class PreAdvanceTests(unittest.TestCase):
    def setUp(self):
        self.reviewer = ControlReviewer()
        self.stdout, self.stderr = '{"answer":"資料は未確認"}\n'.encode(), b"a warning\n"
        self.expected = {"contract": ADMISSION, "evaluation_id": "control", "job_id": "case-baseline",
                         "executor_id": "trusted-executor", "evaluator_id": "trusted-evaluator",
                         "stdout_sha256": digest(self.stdout), "stderr_sha256": digest(self.stderr), "packet_sha256": "a" * 64}
        self.expected.update({k: "c" * 64 for k in ("authority_receipt_sha256", "native_bundle_sha256",
            "http_binding_sha256", "observation_sha256", "sources_sha256")})

    def verify(self, signed=None, **overrides):
        args = dict(policy_bytes=self.reviewer.policy_bytes, attestation=signed or self.reviewer.sign(self.expected, self.stdout, self.stderr),
                    expected=self.expected, stdout=self.stdout, stderr=self.stderr, sources={})
        args.update(overrides)
        return verify_pre_advance(**args)

    def test_complete_actual_streams_and_registered_signature_required(self):
        self.assertFalse(self.verify().authentication["semantic_truth_certified"])
        changed = json.loads(self.reviewer.sign(self.expected, self.stdout, self.stderr))
        changed["signature_hex"] = "00" * 64
        with self.assertRaisesRegex(ValueError, "signature"): self.verify(canonical(changed))

    def test_unsafe_unknown_missing_stderr_gap_and_utf8_split_rejected(self):
        for decision in ("unsafe", "unknown"):
            with self.assertRaisesRegex(ValueError, "unsafe_or_unknown"):
                self.verify(self.reviewer.sign(self.expected, self.stdout, self.stderr, decision=decision))
        segments = {"stdout": [{"start_byte": 0, "end_byte": len(self.stdout), "label": "nonfact", "source_ids": []}], "stderr": []}
        with self.assertRaisesRegex(ValueError, "incomplete"):
            self.verify(self.reviewer.sign(self.expected, self.stdout, self.stderr, segments=segments))
        segments["stderr"] = [{"start_byte": 0, "end_byte": len(self.stderr), "label": "nonfact", "source_ids": []}]
        for end in (len(self.stdout) - 1, self.stdout.index("資".encode()) + 1):
            segments["stdout"][0]["end_byte"] = end
            with self.assertRaises((ValueError, UnicodeDecodeError)):
                self.verify(self.reviewer.sign(self.expected, self.stdout, self.stderr, segments=segments))

    def test_stale_foreign_and_revoked_pre_advance_review_rejected(self):
        signed = self.reviewer.sign(self.expected, self.stdout, self.stderr)
        for name in ("job_id", "packet_sha256", "stdout_sha256"):
            changed = {**self.expected, name: "foreign" if name == "job_id" else "e" * 64}
            with self.assertRaisesRegex(ValueError, "stale_or_foreign|stream_mismatch"): self.verify(signed, expected=changed)
        policy = copy.deepcopy(self.reviewer.policy); policy["keys"][0]["revoked"] = True
        with self.assertRaisesRegex(ValueError, "revoked"): self.verify(signed, policy_bytes=canonical(policy))

    def test_signed_unbound_fact_and_executor_alias_cannot_authorize_advance(self):
        segments = {"stdout": [{"start_byte": 0, "end_byte": len(self.stdout), "label": "fact", "source_ids": []}],
                    "stderr": [{"start_byte": 0, "end_byte": len(self.stderr), "label": "nonfact", "source_ids": []}]}
        with self.assertRaisesRegex(ValueError, "unbound_fact"):
            self.verify(self.reviewer.sign(self.expected, self.stdout, self.stderr, segments=segments))
        policy = copy.deepcopy(self.reviewer.policy)
        policy["principals"][1]["conflict_groups"] = ["execution"]
        with self.assertRaisesRegex(ValueError, "conflict"): self.verify(policy_bytes=canonical(policy))

    def test_equal_length_changed_actual_stream_does_not_match_signed_hash(self):
        changed = self.stdout.replace(b"answer", b"ANSWER")
        with self.assertRaisesRegex(ValueError, "actual_stream_mismatch"): self.verify(stdout=changed)


class SourceCaptureTests(unittest.TestCase):
    def setUp(self):
        evidence = {"id": "ev", "source": "source", "observation": "Full original source.", "facts": {"f": "present"}, "metadata": {}}
        self.catalog = {"contract": CATALOG, "sources": {"source": {"text": "Full original source. More context.", "source_version": "v1"}},
            "evidence_bindings": {"ev": {"evidence_sha256": digest(json.dumps(evidence, ensure_ascii=False, separators=(",", ":")).encode()),
                                         "source_id": "source", "source_version": "v1"}}}
        self.cli = {"result": {"finalization": {"status": "grounded_answer", "text": 'Fact. "Full original source."'},
            "final_outcome": {"artifact": {"evidence": [evidence]}}, "answer_context": {"citations": [{
                "evidence_id": "ev", **self.catalog["evidence_bindings"]["ev"], "quote": "Full original source.",
                "fact_start_byte": 0, "fact_end_byte": 5, "quote_start_byte": 6, "quote_end_byte": 29,
                "quote_display_encoding": "json_string", "quote_sha256": digest(b"Full original source.")}]}}}

    def test_full_source_and_actual_native_evidence_citation_binding(self):
        catalog = source_catalog(canonical(self.catalog))
        capture = capture_display(self.cli, catalog)
        self.assertEqual(capture["admitted_source_ids"], ["source"])
        self.assertEqual(len(capture["display_citations"]), 2)
        self.assertIn("More context.", catalog["sources"]["source"]["text"])

    def test_changed_evidence_old_source_version_missing_context_or_quote_rejected(self):
        for mutate in (lambda x: x["result"]["final_outcome"]["artifact"]["evidence"][0].update(observation="changed"),
                       lambda x: x["result"]["answer_context"]["citations"][0].update(source_version="v0"),
                       lambda x: x["result"].pop("answer_context"),
                       lambda x: x["result"]["answer_context"]["citations"][0].update(quote="not in full source")):
            changed = copy.deepcopy(self.cli); mutate(changed)
            with self.assertRaises(ValueError): capture_display(changed, self.catalog)
        changed = copy.deepcopy(self.catalog); changed["sources"]["source"]["text"] = ""
        with self.assertRaises(ValueError): source_catalog(canonical(changed))


class NativeBundleTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.source = self.root / "source"; self.source.mkdir(mode=0o700)
        (self.source / "Cargo.toml").write_text("source-control")
        (self.source / "Cargo.lock").write_text("lock-control")
        (self.source / "crates").mkdir(); (self.source / "crates/main.rs").write_text("source")
        self.binary = self.root / "reason"; self.binary.write_bytes(b"control-binary"); self.binary.chmod(0o700)
        lock = self.root / "lock"; lock.write_bytes(b"frozen-runtime")
        self.bundle = NativeBundle.enroll(source_roots={a: self.source for a in ("baseline", "candidate")},
            binaries={a: self.binary for a in ("baseline", "candidate")}, configurations={},
            runtime_files={"python": Path(sys.executable).resolve(), "verifier-lock": lock})

    def test_explicit_bundle_freezes_real_file_hashes_and_native_binding(self):
        self.assertEqual(self.bundle.verify(), self.bundle.sha256)
        self.assertEqual(self.bundle.binding("e", "r").image_sha256, self.bundle.sha256)

    def test_source_binary_config_replacement_and_new_file_are_rejected(self):
        file = self.source / "crates/main.rs"
        file.write_text("changed")
        with self.assertRaisesRegex(ValueError, "artifact_changed"): self.bundle.verify()
        file.write_text("source"); self.bundle.verify()
        extra = self.source / "crates/extra.rs"; extra.write_text("new")
        with self.assertRaisesRegex(ValueError, "inventory_changed"): self.bundle.verify()
        extra.unlink(); self.binary.unlink(); self.binary.write_bytes(b"control-binary"); self.binary.chmod(0o700)
        with self.assertRaisesRegex(ValueError, "artifact_changed"): self.bundle.verify()

    def test_symlinked_unknown_native_executable_is_rejected(self):
        self.binary.unlink(); self.binary.symlink_to(Path(sys.executable).resolve())
        with self.assertRaises(OSError): self.bundle.verify()

    def test_runtime_dependency_change_is_rejected(self):
        (self.root / "lock").write_bytes(b"changed runtime dependency")
        with self.assertRaisesRegex(ValueError, "artifact_changed"): self.bundle.verify()


class PipeCaptureTests(unittest.TestCase):
    def test_overflow_is_explicitly_incomplete_and_cannot_look_like_full_capture(self):
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder).resolve()
            _, complete = capture_process([sys.executable, "-I", "-S", "-c", "import os;os.write(1,b'x'*(9*1024*1024))"],
                directory=directory, environment={}, pass_fds=(), timeout=10)
            self.assertFalse(complete)
            self.assertLessEqual((directory / "cli.json").stat().st_size, 8 * 1024 * 1024)
            self.assertEqual(json.loads((directory / "execution.json").read_bytes())["operational_failure"], "native_stream_overflow")

    def test_closed_streams_do_not_hide_a_lingering_child(self):
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder).resolve()
            _, complete = capture_process([sys.executable, "-I", "-S", "-c", "import os,time;os.close(1);os.close(2);time.sleep(5)"],
                directory=directory, environment={}, pass_fds=(), timeout=0.15)
            self.assertFalse(complete)
            self.assertEqual(json.loads((directory / "execution.json").read_bytes())["operational_failure"], "native_process_timeout")


class ScriptedServer:
    def __init__(self, mode="normal"):
        self.requests = []
        owner = self
        class Handler(http.server.BaseHTTPRequestHandler):
            def do_POST(self):
                request = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
                owner.requests.append(request)
                schema = request.get("response_format", {}).get("json_schema", {}).get("name", "")
                if mode == "decode": content = "not JSON"
                elif schema == "reasoning_candidate" or (request.get("response_format", {}).get("type") == "json_object" and "candidate" in request["messages"][-1]["content"].lower()):
                    content = '{"claims":[],"inferences":[]}' + (" EXTRA NOTE" if mode == "trailing" else "")
                else: content = '{"text":"No verified answer is available.","factual_claims":[]}'
                body = canonical({"model": "fixture-model", "choices": [{"message": {"content": content}, "finish_reason": "stop"}],
                                  "usage": {"prompt_tokens": 2, "completion_tokens": 3, "total_tokens": 5}})
                self.send_response(200); self.send_header("Content-Length", str(len(body))); self.end_headers(); self.wfile.write(body)
            def log_message(self, *args): pass
        self.server = http.server.HTTPServer(("127.0.0.1", 0), Handler)
        self.url = f"http://127.0.0.1:{self.server.server_port}/v1/"
        self.worker = threading.Thread(target=self.server.serve_forever, daemon=True); self.worker.start()
    def close(self): self.server.shutdown(); self.server.server_close(); self.worker.join(10)


@unittest.skipUnless(os.environ.get("REASON_V3_NATIVE_CLI"), "set REASON_V3_NATIVE_CLI for actual 48-child controls")
class ActualNativeAdmissionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory(); cls.root = Path(cls.temp.name).resolve()
        cls.reviewer = ControlReviewer()
        cls.catalog_bytes = canonical({"contract": CATALOG, "sources": {}, "evidence_bindings": {}})
        cls.policy_path, cls.catalog_path = cls.root / "policy.json", cls.root / "sources.json"
        private_bytes(cls.policy_path, cls.reviewer.policy_bytes); private_bytes(cls.catalog_path, cls.catalog_bytes)
        binary = Path(os.environ["REASON_V3_NATIVE_CLI"]).resolve()
        source = Path(__file__).resolve().parents[3]
        cls.bundle = NativeBundle.enroll(source_roots={a: source for a in ("baseline", "candidate")},
            binaries={a: binary for a in ("baseline", "candidate")}, configurations={"policy": cls.policy_path, "sources": cls.catalog_path})
        cls.argv = [str(binary), "--no-config", "--plain", "--provider", "mistral", "--model", "fixture-model",
                    "--max-model-calls", "4", "--max-tokens", "32", "--max-total-tokens", "128", "--format", "json", QUESTION]
        cls.planned = {a: plan_first_request(cls.bundle, a, cls.argv, cls.root / (a + "-request.json")) for a in ("baseline", "candidate")}
        legacy_controls.AuthorityTests.setUpClass(); cls.template = legacy_controls.AuthorityTests()

    @classmethod
    def tearDownClass(cls): legacy_controls.AuthorityTests.tearDownClass(); cls.temp.cleanup()

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.plan = copy.deepcopy(self.template.plan); self.plan["purpose"] = CONTROL
        self.plan["questions"] = {case: QUESTION for case in self.plan["case_ids"]}
        reserve = 0
        for job in self.plan["jobs"]:
            jid, arm = job["id"], job["arm"]
            job["argv"] = list(self.argv); job["native_request_plan"] = self.planned[arm]
            job["timeout_seconds"] = 15
            job["budget"] = {"max_generations": 4, "max_input_tokens_per_generation": 20000,
                             "max_output_tokens_per_generation": 32, "max_attempts_per_generation": 1}
            reserve += authority._job_reserve(job["budget"])
            self.plan["frozen_request_bindings"][jid] = frozen_request_digest(job_id=jid, case_id=jid.rsplit("-", 1)[0],
                arm=arm, question=QUESTION, request=self.planned[arm]["request"], request_sha256=self.planned[arm]["request_sha256"],
                provider="mistral", model="fixture-model", receipt_sha256=digest(self.template.local_receipt))
        self.plan["token_budget"] = reserve
        profile = initialize_local_store(self.root / "authority")
        self.store = LocalAtomicStore(profile, role="executor")
        self.evaluator_store = LocalAtomicStore(profile, role="evaluator")
        binding = self.bundle.binding("trusted-executor", "trusted-evaluator")
        _, receipt = self.template.fixture("native-582", plan=self.plan, binding=binding)
        receipt["contract"] = authority.NATIVE_RECEIPT
        receipt["native_admission"] = {"bundle_sha256": self.bundle.sha256,
            "policy_sha256": digest(self.reviewer.policy_bytes), "sources_sha256": digest(self.catalog_bytes)}
        LocalAtomicStore(profile, role="issuer").create(f"{authority.PREFIX}/receipts/native-582.json", canonical(receipt))
        self.session = authority.TrustedAuthority(self.store).start("native-582", self.plan, self.template.local_receipt, binding)
        self.binding = binding

    def lane(self, mode="normal"):
        server = ScriptedServer(mode); self.addCleanup(server.close)
        lane = NativeLane(session=self.session, plan=self.plan, bundle=self.bundle,
            policy_bytes=self.reviewer.policy_bytes, catalog_bytes=self.catalog_bytes, output=self.root / "lane", control_url=server.url)
        return lane, server

    def sign(self, packet, expected, **kwargs):
        return self.reviewer.sign(expected, packet["stdout"].encode(), packet["stderr"].encode(), **kwargs)

    def test_prospective_plan_matches_real_http_and_review_precedes_second_child(self):
        lane, server = self.lane()
        packet, expected = lane.run_next()
        self.assertTrue(packet["stdout"])
        calls = len(server.requests)
        with self.assertRaisesRegex(ValueError, "review_pending"): lane.run_next()
        self.assertEqual(len(server.requests), calls)
        observation = lane.accept_review(self.sign(packet, expected))
        self.assertIn(observation["measurement_state"], ("normal_observed", "local_decode_observed"))
        self.assertIsNotNone(self.store.read(authority._key("native-582", "admission/" + expected["job_id"])))
        lane.run_next()
        self.assertGreater(len(server.requests), calls)

    def test_unsafe_or_tampered_display_stops_without_second_child(self):
        lane, server = self.lane()
        packet, expected = lane.run_next(); calls = len(server.requests)
        with self.assertRaisesRegex(ValueError, "unsafe_or_unknown"):
            lane.accept_review(self.sign(packet, expected, decision="unsafe"))
        with self.assertRaisesRegex(ValueError, "halted"): lane.run_next()
        self.assertEqual(len(server.requests), calls)

    def test_actual_stderr_cannot_be_omitted_or_changed(self):
        lane, server = self.lane("trailing")
        packet, expected = lane.run_next(); self.assertTrue(packet["stderr"])
        signed = self.sign(packet, expected)
        stderr = lane.output / expected["job_id"] / "stderr.txt"
        stderr.write_bytes(stderr.read_bytes() + b"unsafe hidden side output")
        with self.assertRaisesRegex(ValueError, "display_changed"): lane.accept_review(signed)
        with self.assertRaisesRegex(ValueError, "halted"): lane.run_next()

    def test_accounted_native_decode_invalid_can_advance_only_after_signed_review(self):
        lane, server = self.lane("decode")
        packet, expected = lane.run_next()
        self.assertEqual(packet["observation"]["measurement_state"], "local_decode_observed")
        self.assertEqual(packet["exit_code"], 1)
        lane.accept_review(self.sign(packet, expected))
        self.assertEqual(lane.next, 1)

    def test_full_48_actual_cli_children_keep_all_pre_advance_records_and_score_order(self):
        lane, server = self.lane()
        hashes = {}
        for _ in range(48):
            packet, expected = lane.run_next()
            lane.accept_review(self.sign(packet, expected))
            hashes[expected["job_id"]] = expected["observation_sha256"]
        self.assertEqual(lane.next, 48)
        self.assertGreaterEqual(len(server.requests), 48)
        self.assertIsNotNone(self.store.read(authority._key("native-582", "completed")))
        evaluator = authority.TrustedAuthority(self.evaluator_store)
        args = dict(observation_index_sha256="1" * 64, review_manifest_sha256="2" * 64, observation_hashes=hashes)
        hidden = authority._key("native-582", "admission/" + next(iter(hashes)))
        store = self.evaluator_store
        class MissingAdmission:
            def read(self, key): return None if key == hidden else store.read(key)
            def create(self, key, value): return store.create(key, value)
        with self.assertRaisesRegex(ValueError, "pre_advance_record_missing"):
            authority.TrustedAuthority(MissingAdmission()).claim_score("native-582", self.plan, self.template.local_receipt, self.binding, **args)
        evaluator.claim_score("native-582", self.plan, self.template.local_receipt, self.binding, **args)
        with self.assertRaises(FileExistsError): evaluator.claim_score("native-582", self.plan, self.template.local_receipt, self.binding, **args)

    def test_native_receipt_cannot_use_the_legacy_path_without_safety_profile(self):
        with self.assertRaisesRegex(ValueError, "profile_required_before_reservation"):
            self.session.reserve(self.plan["jobs"][0]["id"])
        with self.assertRaisesRegex(ValueError, "capability_required"):
            self.session.acknowledge_native_admission(self.plan["jobs"][0]["id"], packet_sha256="a" * 64,
                observation_sha256="b" * 64, authentication={"contract": ADMISSION, "safe": True})

    def test_source_change_after_child_blocks_signed_advance(self):
        lane, server = self.lane()
        packet, expected = lane.run_next(); signed = self.sign(packet, expected)
        original = self.catalog_path.read_bytes()
        try:
            self.catalog_path.write_bytes(b"changed source inventory")
            with self.assertRaisesRegex(ValueError, "artifact_changed"): lane.accept_review(signed)
        finally: self.catalog_path.write_bytes(original)
        with self.assertRaisesRegex(ValueError, "halted"): lane.run_next()

    def test_mutating_returned_packet_cannot_change_frozen_pending_review(self):
        lane, server = self.lane()
        packet, expected = lane.run_next()
        signed = self.sign(packet, expected)
        expected["packet_sha256"] = "f" * 64
        packet["observation"]["files"]["cli.json"] = "f" * 64
        packet["full_sources"]["foreign"] = {"text": "unbound", "source_version": "v1"}
        lane.accept_review(signed)
        self.assertEqual(lane.next, 1)

    def test_request_planning_refuses_resume_route_before_any_model_access(self):
        destination = self.root / "invalid-plan.json"
        r = subprocess.run([self.argv[0], "--no-config", "--format", "json", "--continue", "--evaluation-request-plan", str(destination)],
                           env={"PATH": "/usr/bin:/bin"}, capture_output=True, timeout=10)
        self.assertNotEqual(r.returncode, 0)
        self.assertFalse(destination.exists())


if __name__ == "__main__": unittest.main()
