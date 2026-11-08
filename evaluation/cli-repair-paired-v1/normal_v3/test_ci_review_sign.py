"""No live model, no GitHub writes: consented independent review signer tests."""
import base64
import json
import os
from pathlib import Path
import tempfile
import unittest

from nacl.signing import SigningKey

from ci_exchange import APPROVAL, authenticate
from ci_review_sign import envelope, final, job, signer, start
from annotate import build_packet
from contracts import canonical, digest, keyed_digest, OBS_ID, PACKET_ID, REVIEW_ID
from native_admission import ADMISSION, verify_pre_advance
from native_lane import private_bytes


class ReviewerSignTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        os.chmod(self.root, 0o700)
        self.source_signer = SigningKey.generate()
        self.seed = self.root / "reviewer-seed.bin"
        private_bytes(self.seed, bytes(self.source_signer))
        policy = {
            "contract": "reason-cli-normal-v3-review-policy-v1",
            "policy_id": "offline-reviewer-tests",
            "keys": [{"key_id": "test-reviewer", "subject_id": "reviewer",
                      "public_key_hex": self.source_signer.verify_key.encode().hex(), "revoked": False}],
            "principals": [
                {"subject_id": "executor", "aliases": ["exec"], "role": "agent", "conflict_groups": ["execution"]},
                {"subject_id": "reviewer", "aliases": ["review"], "role": "agent", "conflict_groups": ["review"]},
            ],
        }
        self.operator = {"evaluator_id": "review", "executor_id": "exec", "review_policy": policy,
                         "fee_mode": "free-only", "additional_charges_enabled": False}
        self.key, self.key_id, self.policy_bytes = signer(self.operator, self.seed)

    def test_registered_key_and_signature_validity(self):
        payload = {"contract": APPROVAL, "policy_sha256": digest(self.policy_bytes), "decision": "deny"}
        item = envelope(APPROVAL, payload, self.key, self.key_id)
        self.assertEqual(authenticate(item, self.policy_bytes, "review", "exec", APPROVAL,
                                      {"policy_sha256": digest(self.policy_bytes)}), payload)
        item["payload"]["decision"] = "approve"
        with self.assertRaises(ValueError):
            authenticate(item, self.policy_bytes, "review", "exec", APPROVAL,
                         {"policy_sha256": digest(self.policy_bytes)})

    def test_wrong_registered_key_cannot_sign(self):
        private_bytes(self.root / "wrong.bin", bytes(SigningKey.generate()))
        with self.assertRaisesRegex(ValueError, "mismatch"):
            signer(self.operator, self.root / "wrong.bin")

    def test_revoked_reviewer_cannot_sign(self):
        self.operator["review_policy"]["keys"][0]["revoked"] = True
        with self.assertRaisesRegex(ValueError, "one_independent_signer_required"):
            signer(self.operator, self.seed)

    def make_start(self):
        path = self.root / "bootstrap"
        path.mkdir(mode=0o700)
        session = {"mode": "live", "execution_tag": "cli-normal-real24-development-v3",
                   "repository": "git-ksk/reasoning-harness", "run_id": 101, "run_attempt": 1,
                   "source_commit": "a" * 40}
        receipt = {"contract": "fixture-receipt"}
        plan = {"jobs": [{"id": i} for i in range(48)], "receipt_sha256": digest(canonical(receipt))}
        request = {"contract": APPROVAL, "policy_sha256": digest(self.policy_bytes),
                   "run_id": 101, "stage": "start", "fee_mode": "free-only",
                   "session_sha256": digest(canonical(session)),
                   "plan_sha256": digest(canonical(plan)),
                   "native_bundle_sha256": digest(canonical({"bundle": "fixture"})),
                   "authority_receipt_sha256": digest(canonical({"authority": "fixture"}))}

        values = {
            "session.json": session,
            "start-request.json": request,
            "readiness.json": {"authentication_and_model_available": True, "model_generations": 0,
                               "metadata_requests_attempted": 1},
            "plan.json": plan,
            "receipt.json": receipt,
            "native-bundle.json": {"bundle": "fixture"},
            "authority-receipt.json": {"authority": "fixture"},
            "gold.json": {"contract": "reason-cli-normal-v3-gold-v1", "receipt_sha256": digest(canonical(receipt)),
                          "cases": [{"case_id": f"case-{i:02}", "question": "q", "label": "positive",
                                     "required_groups": ["g1"]} for i in range(24)]},
        }
        for filename, value in values.items():
            private_bytes(path / filename, canonical(value))
        decision = {"contract": "reason-cli-v3-independent-reviewed-start-v1",
                    "decision": "approve", "start_request_sha256": digest(canonical(request)),
                    "full_sources_reviewed": True,
                    "notes": "Separately inspected the entire 24 case / 48 job frozen bundle and readiness."}
        return path, decision, request

    def test_start_requires_live_and_prior_independent_review(self):
        path, decision, request = self.make_start()
        result = start(path, decision, self.operator, self.key, self.key_id, self.policy_bytes)
        self.assertEqual(authenticate(result, self.policy_bytes, "review", "exec", APPROVAL, request)["decision"], "approve")
        decision["full_sources_reviewed"] = False
        with self.assertRaisesRegex(ValueError, "explicit_independent"):
            start(path, decision, self.operator, self.key, self.key_id, self.policy_bytes)

    def test_start_rejects_prepare_mode_even_with_signature(self):
        path, decision, _ = self.make_start()
        (path / "session.json").write_bytes(canonical({"mode": "prepare", "execution_tag": None}))
        with self.assertRaisesRegex(ValueError, "not_a_live"):
            start(path, decision, self.operator, self.key, self.key_id, self.policy_bytes)

    def make_job(self):
        bootstrap, _, _ = self.make_start()
        stdout = '{"fact":"hello"}'
        sources = {"s1": {"text": "hello", "source_version": "v1"}}
        packet = {"job_id": "rw-01-baseline", "stdout": stdout, "stderr": "",
                  "full_sources": sources, "observation": {"job_id": "rw-01-baseline"}}
        expected = {"contract": ADMISSION, "evaluation_id": "ci-v3-101", "job_id": packet["job_id"],
                    "executor_id": "exec", "evaluator_id": "review",
                    "authority_receipt_sha256": digest(canonical({"authority": "fixture"})),
                    "native_bundle_sha256": digest(canonical({"bundle": "fixture"})),
                    "http_binding_sha256": "c" * 64, "packet_sha256": keyed_digest(packet),
                    "observation_sha256": keyed_digest(packet["observation"]), "sources_sha256": "f" * 64,
                    "stdout_sha256": digest(stdout.encode()), "stderr_sha256": digest(b"")}
        path = self.root / "packet.json"
        private_bytes(path, canonical({"packet": packet, "expected": expected,
                                      "ci_session_sha256": digest((bootstrap / "session.json").read_bytes())}))
        decision = {"contract": "reason-cli-v3-independent-reviewed-job-v1",
                    "packet_sha256": keyed_digest(packet), "decision": "safe", "full_sources_reviewed": True,
                    "notes": "Independent review of every byte and the full source shows a supported assertion.",
                    "segments": {
                        "stdout": [{"start_byte": 0, "end_byte": len(stdout), "label": "fact", "source_ids": ["s1"]}],
                        "stderr": [],
                    }}
        return bootstrap, path, packet, expected, decision

    def test_source_bound_safe_job_signs_and_validates(self):
        bootstrap, path, packet, expected, decision = self.make_job()
        result = job(bootstrap, path, decision, self.operator, self.key, self.key_id, self.policy_bytes)
        self.assertEqual(result["payload"]["decision"], "safe")
        self.assertTrue(verify_pre_advance(policy_bytes=self.policy_bytes, attestation=canonical(result),
            expected=expected, stdout=packet["stdout"].encode(), stderr=b"",
            sources=packet["full_sources"]))

    def test_unbound_or_gapped_stream_fails_closed(self):
        bootstrap, path, _, _, decision = self.make_job()
        decision["segments"]["stdout"][0]["end_byte"] -= 1
        with self.assertRaisesRegex(ValueError, "incomplete"):
            job(bootstrap, path, decision, self.operator, self.key, self.key_id, self.policy_bytes)
        decision["segments"]["stdout"][0]["end_byte"] += 1
        decision["segments"]["stdout"][0]["source_ids"] = ["unknown"]
        with self.assertRaisesRegex(ValueError, "unbound"):
            job(bootstrap, path, decision, self.operator, self.key, self.key_id, self.policy_bytes)

    def test_explicit_unsafe_review_is_signed_as_unsafe(self):
        bootstrap, path, _, expected, decision = self.make_job()
        decision.update({"decision": "unsafe", "full_sources_reviewed": False, "segments": {}})
        result = job(bootstrap, path, decision, self.operator, self.key, self.key_id, self.policy_bytes)
        self.assertEqual(authenticate(result, self.policy_bytes, "review", "exec", ADMISSION, expected)["decision"], "unsafe")

    def test_missing_verdict_or_packet_match_cannot_sign(self):
        bootstrap, path, _, _, decision = self.make_job()
        decision["packet_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "explicit_independent"):
            job(bootstrap, path, decision, self.operator, self.key, self.key_id, self.policy_bytes)
        decision["packet_sha256"] = "f" * 64
        decision.pop("notes")
        with self.assertRaisesRegex(ValueError, "explicit_independent"):
            job(bootstrap, path, decision, self.operator, self.key, self.key_id, self.policy_bytes)

    def test_full48_synthetic_final_review_authenticates_before_scoring(self):
        bootstrap, _, _ = self.make_start()
        receipt_sha = digest((bootstrap / "receipt.json").read_bytes())
        cases, reviews, obs_rows, packet_rows, files = [], {}, [], [], {}
        for n in range(24):
            cid = f"task-{n:02}"
            gold_case = {"case_id": cid, "question": f"Q{n}", "label": "positive",
                         "required_groups": ["g1"]}
            cases.append(gold_case)
            for arm in ("baseline", "candidate"):
                jid = f"{cid}-{arm}"
                obs = {"contract": OBS_ID, "receipt_sha256": receipt_sha, "executor_id": "exec",
                       "job_id": jid, "case_id": cid, "arm": arm, "question": gold_case["question"],
                       "display_artifacts": [{"id": "answer", "kind": "final_answer", "text": ""}],
                       "display_citations": [], "terminal_status": "failed",
                       "measurement_state": "normal_observed", "diagnostics": [],
                       "admitted_source_ids": [], "sources": {}}
                obs_raw = canonical(obs)
                obs_sha = digest(obs_raw)
                files[f"observations/{jid}.json"] = obs_raw
                obs_rows.append({"job_id": jid, "case_id": cid, "arm": arm, "sha256": obs_sha})
                packet_raw = canonical(build_packet(obs, obs_sha, gold_case))
                files[f"packets/{jid}.review-packet.json"] = packet_raw
                packet_rows.append({"job_id": jid, "sha256": digest(packet_raw)})
                reviews[jid] = {"contract": REVIEW_ID, "job_id": jid,
                    "observation_sha256": obs_sha, "gold_sha256": keyed_digest(gold_case),
                    "reviewer": {"role": "agent", "id": "review",
                                 "independent_of_execution": True, "full_source_context_reviewed": True},
                    "decision": "invalid", "correct_groups": [], "incorrect_assertions": 0,
                    "hard_violations": [], "citation_checks": [],
                    "readable_and_actionable": False, "safety_audit_complete": True,
                    "notes": "Reviewing a synthetic failed output: no model answer exists to credit.",
                    "segments": []}
        gold = canonical({"contract": "reason-cli-normal-v3-gold-v1",
                          "receipt_sha256": receipt_sha, "cases": cases})
        index = canonical({"contract": OBS_ID, "continuation_authorized": False,
                           "receipt_sha256": receipt_sha, "executor_id": "exec", "rows": obs_rows})
        pindex = canonical({"contract": PACKET_ID, "rows": packet_rows})
        files.update({"gold.json": gold, "observations/index.json": index,
                      "packets/packet-index.json": pindex})
        bindings = {"receipt_sha256": receipt_sha, "gold_sha256": digest(gold),
                    "observation_index_sha256": digest(index), "packet_index_sha256": digest(pindex),
                    "execution_binding_sha256": "b" * 64, "plan_sha256": "a" * 64}
        packet = {"contract": "reason-cli-normal-v3-ci-final-request-v1",
                  "ci_session_sha256": digest((bootstrap / "session.json").read_bytes()),
                  "evaluation_id": "ci-v3-101", "bindings": bindings,
                  "files": {name: base64.b64encode(raw).decode() for name, raw in files.items()}}
        path = self.root / "final-full.json"
        private_bytes(path, canonical(packet))
        decision = {"contract": "reason-cli-v3-independent-reviewed-final-v1",
                    "final_request_sha256": keyed_digest(packet), "full_sources_reviewed": True,
                    "complete_display_reviewed": True, "reviews": reviews}
        signed = final(bootstrap, path, decision, self.operator, self.key, self.key_id, self.policy_bytes)
        self.assertEqual(len(signed["reviews"]), 48)
        self.assertEqual(signed["attestation"]["payload"]["evaluation_id"], "ci-v3-101")
        self.assertEqual(signed["attestation"]["contract"], "reason-cli-normal-v3-review-attestation-v1")
        decisions = {k: dict(v) for k, v in reviews.items()}
        first = next(iter(decisions))
        decisions[first] = {**decisions[first], "observation_sha256": "0" * 64}
        with self.assertRaisesRegex(ValueError, "stale_or_foreign"):
            final(bootstrap, path, {**decision, "reviews": decisions},
                  self.operator, self.key, self.key_id, self.policy_bytes)

    def test_final_will_not_sign_without_evidence(self):
        bootstrap, _, _ = self.make_start()
        path = self.root / "final.json"
        private_bytes(path, canonical({"contract": "reason-cli-normal-v3-ci-final-request-v1",
                                       "evaluation_id": "ci-v3-101", "bindings": {}, "files": {},
                                       "ci_session_sha256": digest((bootstrap / "session.json").read_bytes())}))
        with self.assertRaisesRegex(ValueError, "explicit_independent_final"):
            final(bootstrap, path, {"contract": "wrong", "reviews": {}},
                  self.operator, self.key, self.key_id, self.policy_bytes)


if __name__ == "__main__":
    unittest.main()
