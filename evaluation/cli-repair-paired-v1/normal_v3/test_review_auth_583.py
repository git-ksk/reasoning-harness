"""Real temporary Ed25519 signatures over synthetic v3 data; no external calls."""
import copy
import json
import os
import shutil
import subprocess
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import annotate
import collect
import review_auth as auth
import score
from authority import AuthorityBinding, ScoringContext
from contracts import canonical, digest


class SyntheticSigner:
    """Ephemeral test keys only. Never enrolls a production identity."""
    def __init__(self, root):
        self.root = Path(root)
        self.root.mkdir(mode=0o700)
        binary = shutil.which("openssl")
        if binary is None:
            raise AssertionError("OpenSSL with Ed25519 support required for #583 controls")
        self.binary = str(Path(binary).resolve())
        self.binary_sha = digest(Path(self.binary).read_bytes())
        keys = []
        for name in ("reviewer", "other"):
            private = self.root / (name + ".pem")
            self.run("genpkey", "-algorithm", "ED25519", "-out", str(private))
            private.chmod(0o600)
            der = self.run("pkey", "-in", str(private), "-pubout", "-outform", "DER")
            assert der.startswith(auth.ED25519_SPKI) and len(der) == 44
            keys.append({"key_id": name + "-key", "subject_id": name + "-subject",
                         "public_key_hex": der[12:].hex(), "revoked": False})
        self.policy = {
            "contract": auth.POLICY, "policy_id": "synthetic-policy-583",
            "principals": [
                {"subject_id": "executor-subject", "role": "agent",
                 "aliases": ["fixture-executor", "executor-alias"],
                 "conflict_groups": ["execution-team"]},
                {"subject_id": "reviewer-subject", "role": "agent",
                 "aliases": ["fixture-reviewer", "independent-synthetic-reviewer"],
                 "conflict_groups": ["independent-review-team"]},
                {"subject_id": "other-subject", "role": "human",
                 "aliases": ["other-reviewer"], "conflict_groups": ["other-team"]},
            ], "keys": keys,
        }

    def run(self, *args):
        return subprocess.run([self.binary, *args], check=True, stdout=subprocess.PIPE,
                              stderr=subprocess.PIPE, stdin=subprocess.DEVNULL,
                              env={"OPENSSL_CONF": os.devnull}, timeout=10).stdout

    def sign(self, bindings, *, evaluation_id="fixture-run", executor_id="fixture-executor",
             evaluator_id="fixture-reviewer", policy=None, updates=None, signer="reviewer"):
        policy = self.policy if policy is None else policy
        payload = {
            "contract": auth.ATTESTATION, "evaluation_id": evaluation_id,
            "executor_id": executor_id, "evaluator_id": evaluator_id,
            "policy_sha256": digest(canonical(policy)),
            "complete_display_reviewed": True, "full_sources_reviewed": True,
            **bindings, **(updates or {}),
        }
        message = self.root / "message"
        message.write_bytes(auth.DOMAIN + canonical(payload))
        signature = self.root / "signature"
        self.run("pkeyutl", "-sign", "-rawin", "-inkey",
                 str(self.root / (signer + ".pem")), "-in", str(message), "-out", str(signature))
        return auth.ReviewAuthentication(
            canonical(policy), canonical({
                "contract": auth.ATTESTATION, "algorithm": "Ed25519",
                "key_id": signer + "-key", "payload": payload,
                "signature_hex": signature.read_bytes().hex(),
            }), self.binary, self.binary_sha,
        )


class ReviewerAuthenticationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.signer = SyntheticSigner(self.root / "signer")
        self.bindings = {name: digest(name.encode()) for name in (
            "receipt_sha256", "observation_index_sha256", "packet_index_sha256",
            "gold_sha256", "review_manifest_sha256",
            "execution_binding_sha256", "plan_sha256",
        )}
        self.reviewers = [{"id": "independent-synthetic-reviewer", "role": "agent"}]

    def verify(self, authentication, **changes):
        return authentication.verify(**{
            "evaluation_id": "fixture-run", "executor_id": "fixture-executor",
            "evaluator_id": "fixture-reviewer", "bindings": self.bindings,
            "reviewers": self.reviewers, **changes,
        })

    def test_real_signature_and_enrolled_alias_authenticate_without_truth_credit(self):
        signed = self.signer.sign(self.bindings)
        receipt = self.verify(signed)
        self.assertEqual(receipt["attestation_sha256"], digest(signed.attestation))
        self.assertEqual(receipt["subject_id"], "reviewer-subject")
        self.assertFalse(receipt["semantic_truth_certified"])

    def test_tampered_signature_rejected(self):
        signed = self.signer.sign(self.bindings)
        envelope = json.loads(signed.attestation)
        envelope["signature_hex"] = "00" * 64
        with self.assertRaisesRegex(ValueError, "invalid_review_signature"):
            self.verify(replace(signed, attestation=canonical(envelope)))

    def test_foreign_evaluation_and_each_changed_frozen_input_rejected(self):
        signed = self.signer.sign(self.bindings)
        with self.assertRaisesRegex(ValueError, "stale_or_foreign"):
            self.verify(signed, evaluation_id="another-run")
        for name in self.bindings:
            with self.subTest(binding=name), self.assertRaisesRegex(ValueError, "stale_or_foreign"):
                self.verify(signed, bindings={**self.bindings, name: "e" * 64})

    def test_same_executor_under_another_alias_cannot_be_reviewer(self):
        signed = self.signer.sign(self.bindings, evaluator_id="executor-alias")
        with self.assertRaisesRegex(ValueError, "reviewer_execution_conflict"):
            self.verify(signed, evaluator_id="executor-alias")

    def test_distinct_identity_with_execution_conflict_rejected(self):
        policy = copy.deepcopy(self.signer.policy)
        policy["principals"][1]["conflict_groups"].append("execution-team")
        with self.assertRaisesRegex(ValueError, "reviewer_execution_conflict"):
            self.verify(self.signer.sign(self.bindings, policy=policy))

    def test_revoked_and_wrong_enrolled_key_rejected(self):
        policy = copy.deepcopy(self.signer.policy)
        policy["keys"][0]["revoked"] = True
        with self.assertRaisesRegex(ValueError, "unknown_or_revoked"):
            self.verify(self.signer.sign(self.bindings, policy=policy))
        with self.assertRaisesRegex(ValueError, "wrong_enrolled_review_signer"):
            self.verify(self.signer.sign(self.bindings, signer="other"))

    def test_unknown_key_cannot_self_enroll_via_envelope(self):
        signed = self.signer.sign(self.bindings)
        envelope = json.loads(signed.attestation)
        envelope["key_id"] = "untrusted-key"
        with self.assertRaisesRegex(ValueError, "unknown_or_revoked"):
            self.verify(replace(signed, attestation=canonical(envelope)))
        envelope["public_key_hex"] = self.signer.policy["keys"][0]["public_key_hex"]
        with self.assertRaisesRegex(ValueError, "review_attestation_unexpected_keys"):
            self.verify(replace(signed, attestation=canonical(envelope)))

    def test_declared_reviewer_identity_and_human_role_must_match_enrollment(self):
        signed = self.signer.sign(self.bindings)
        for reviewer in ({"id": "invented-reviewer", "role": "agent"},
                         {"id": "executor-alias", "role": "agent"},
                         {"id": "fixture-reviewer", "role": "human"}):
            with self.subTest(reviewer=reviewer), self.assertRaisesRegex(ValueError, "forged_reviewer"):
                self.verify(signed, reviewers=[reviewer])

    def test_incomplete_display_or_source_review_cannot_be_signed_into_acceptance(self):
        for field in ("complete_display_reviewed", "full_sources_reviewed"):
            for value in (False, 1):
                with self.subTest(field=field, value=value), self.assertRaisesRegex(ValueError, "incomplete_authenticated"):
                    self.verify(self.signer.sign(self.bindings, updates={field: value}))

    def test_changed_trust_policy_invalidates_previously_signed_review(self):
        signed = self.signer.sign(self.bindings)
        policy = copy.deepcopy(self.signer.policy)
        policy["policy_id"] = "new-enrollment"
        with self.assertRaisesRegex(ValueError, "stale_or_foreign"):
            self.verify(replace(signed, policy_bytes=canonical(policy)))

    def test_noncanonical_and_duplicate_json_or_malformed_signature_rejected(self):
        signed = self.signer.sign(self.bindings)
        for raw in (signed.attestation + b"\n",
                    b'{"contract":"ignored",' + signed.attestation[1:]):
            with self.assertRaisesRegex(ValueError, "noncanonical"):
                self.verify(replace(signed, attestation=raw))
        for signature in ("00", "GG" * 64, "00" * 65):
            envelope = json.loads(signed.attestation)
            envelope["signature_hex"] = signature
            with self.assertRaisesRegex(ValueError, "invalid_ed25519_signature"):
                self.verify(replace(signed, attestation=canonical(envelope)))

    def test_ambiguous_alias_and_shared_key_policy_rejected(self):
        policy = copy.deepcopy(self.signer.policy)
        policy["principals"][1]["aliases"].append("executor-alias")
        with self.assertRaisesRegex(ValueError, "ambiguous_principal_alias"):
            self.verify(self.signer.sign(self.bindings, policy=policy))
        policy = copy.deepcopy(self.signer.policy)
        policy["keys"][1]["public_key_hex"] = policy["keys"][0]["public_key_hex"]
        with self.assertRaisesRegex(ValueError, "reused_review_public_key"):
            self.verify(self.signer.sign(self.bindings, policy=policy))

    def test_verifier_missing_modified_or_timed_out_fails_closed(self):
        signed = self.signer.sign(self.bindings)
        for changed, reason in ((replace(signed, openssl_binary="/missing/openssl"), "unavailable"),
                                (replace(signed, openssl_sha256="e" * 64), "pin_mismatch")):
            with self.assertRaisesRegex(ValueError, reason):
                self.verify(changed)
        with patch.object(auth.subprocess, "run", side_effect=subprocess.TimeoutExpired("openssl", 10)):
            with self.assertRaisesRegex(ValueError, "verification_unavailable"):
                self.verify(signed)

    def test_verifier_ignores_inherited_openssl_config(self):
        signed = self.signer.sign(self.bindings)
        config = self.root / "untrusted.conf"
        config.write_text("this file is not valid openssl configuration !!!")
        with patch.dict(os.environ, {"OPENSSL_CONF": str(config), "OPENSSL_MODULES": "/missing/modules"}):
            self.verify(signed)

    def prepared_score(self):
        from test_measurement import demo_fixture, review_of
        manifest, lane, gold, receipt, _ = demo_fixture(self.root)
        observations, packets, reviews = (self.root / name for name in ("obs", "packets", "reviews"))
        collect.collect(manifest, lane, observations)
        annotate.make_packets(observations, gold, packets)
        index, rows = annotate.read_observations(observations)
        by_case = annotate.validate_gold(annotate.read_json(gold))
        reviews.mkdir()
        hashes = {}
        for jid, (observation, obs_sha) in rows.items():
            review = review_of({**observation, "_sha": obs_sha}, by_case[observation["case_id"]])
            raw = canonical(review)
            (reviews / (jid + ".review.json")).write_bytes(raw)
            hashes[jid] = digest(raw)
        freeze = {
            "contract": "reason-cli-normal-v3-review-index-v1",
            "receipt_sha256": index["receipt_sha256"], "executor_id": index["executor_id"],
            "observation_index_sha256": digest((observations / "index.json").read_bytes()),
            "packet_index_sha256": digest((packets / "packet-index.json").read_bytes()),
            "reviews": hashes,
        }
        (reviews / "review-index.json").write_bytes(canonical(freeze))
        class Spy:
            calls = []
            def claim_score(self, *args, **kwargs):
                self.calls.append(kwargs)
        spy = Spy()
        spy.calls = []
        context = ScoringContext(spy, "fixture-run", {}, AuthorityBinding(
            "a" * 64, "b" * 64, "c" * 64, "d" * 64,
            index["executor_id"], "fixture-reviewer"))
        policy = copy.deepcopy(self.signer.policy)
        if index["executor_id"] not in policy["principals"][0]["aliases"]:
            policy["principals"][0]["aliases"].append(index["executor_id"])
        bindings = {"receipt_sha256": index["receipt_sha256"],
                    "execution_binding_sha256": digest(canonical(vars(context.binding))),
                    "plan_sha256": digest(canonical(context.plan)),
                    "gold_sha256": digest(gold.read_bytes()),
                    "review_manifest_sha256": digest((reviews / "review-index.json").read_bytes()),
                    **{k: freeze[k] for k in ("observation_index_sha256", "packet_index_sha256")}}
        signed = self.signer.sign(bindings, executor_id=index["executor_id"], policy=policy)
        return (observations, packets, gold, reviews, receipt, self.root / "score"), context, signed

    def test_missing_signature_rejected_before_any_first_score_claim(self):
        args, context, _ = self.prepared_score()
        with self.assertRaisesRegex(ValueError, "authenticated_reviewer_required"):
            score.score(*args, authoritative_score=context)
        self.assertFalse(context.authority.calls)
        self.assertFalse(args[-1].exists())
        self.assertFalse(list((self.root / "score-ledger").glob("*.claim.json")))

    def test_signed_snapshot_flows_into_first_score_and_keeps_quality_no_go(self):
        args, context, signed = self.prepared_score()
        result = score.score(*args, authoritative_score=context, review_authentication=signed)
        self.assertFalse(result["quality_go"])
        self.assertFalse(result["human_attestation"])
        self.assertEqual(context.authority.calls[0]["review_authentication_sha256"],
                         result["review_authentication_sha256"])

    def test_score_authenticates_the_exact_bytes_it_parses_once(self):
        args, context, signed = self.prepared_score()
        paths = {args[0] / "index.json", args[1] / "packet-index.json",
                 args[2], args[3] / "review-index.json", args[4],
                 *args[3].glob("*.review.json")}
        original = Path.read_bytes
        reads = {path: 0 for path in paths}
        def read_once(path):
            raw = original(path)
            if path in reads:
                reads[path] += 1
                if reads[path] > 1:
                    return b'{"tampered_after_first_read":true}'
            return raw
        with patch.object(Path, "read_bytes", read_once):
            result = score.score(*args, authoritative_score=context, review_authentication=signed)
        self.assertEqual(set(reads.values()), {1})
        self.assertEqual(result["review_authentication"]["attestation_sha256"], digest(signed.attestation))

    def test_review_rewrite_with_recomputed_manifest_still_rejected_before_claim(self):
        args, context, signed = self.prepared_score()
        reviews = args[3]
        path = next(reviews.glob("*.review.json"))
        review = json.loads(path.read_bytes())
        review["notes"] = "Altered but structurally valid annotation."
        path.write_bytes(canonical(review))
        manifest_path = reviews / "review-index.json"
        manifest = json.loads(manifest_path.read_bytes())
        manifest["reviews"][review["job_id"]] = digest(path.read_bytes())
        manifest_path.write_bytes(canonical(manifest))
        with self.assertRaisesRegex(ValueError, "stale_or_foreign_review_attestation"):
            score.score(*args, authoritative_score=context, review_authentication=signed)
        self.assertFalse(context.authority.calls)
        self.assertFalse(args[-1].exists())


if __name__ == "__main__":
    unittest.main()
