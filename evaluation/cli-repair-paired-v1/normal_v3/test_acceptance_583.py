"""Adversarial #583 acceptance cases authored after candidate 8832c74.

These are local acceptance controls, not an independent reviewer credential.
Only ephemeral synthetic identities and signatures are used.
"""
import copy
import json
import subprocess
import tempfile
import unittest
import sys
from dataclasses import replace
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from contracts import canonical, digest
import review_auth as auth
from test_review_auth_583 import SyntheticSigner

RFC_PUBLIC = bytes.fromhex("3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c")
RFC_SIGNATURE = bytes.fromhex(
    "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da"
    "085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00")


class ReviewAcceptance(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.signer = SyntheticSigner(Path(self.tmp.name) / "signer")
        self.bindings = {name: digest(name.encode()) for name in (
            "receipt_sha256", "observation_index_sha256", "packet_index_sha256",
            "gold_sha256", "review_manifest_sha256", "execution_binding_sha256", "plan_sha256",
        )}

    def verify(self, signed):
        return signed.verify(
            evaluation_id="fixture-run", executor_id="fixture-executor",
            evaluator_id="fixture-reviewer", bindings=self.bindings,
            reviewers=[{"id": "fixture-reviewer", "role": "agent"}],
        )

    def test_identity_public_key_cannot_authenticate_unsigned_attestation(self):
        policy = copy.deepcopy(self.signer.policy)
        policy["keys"][0]["public_key_hex"] = "01" + "00" * 31
        signed = self.signer.sign(self.bindings, policy=policy)
        envelope = json.loads(signed.attestation)
        # No private key signs this payload: R=identity, S=0 is a constant.
        envelope["signature_hex"] = "01" + "00" * 63
        with self.assertRaisesRegex(ValueError, "invalid_ed25519_public_key"):
            self.verify(replace(signed, attestation=canonical(envelope)))

    def test_fake_success_from_external_program_cannot_validate_signature(self):
        signed = self.signer.sign(self.bindings)
        envelope = json.loads(signed.attestation)
        envelope["signature_hex"] = "00" * 64
        signed = replace(signed, attestation=canonical(envelope))
        # Models the old check/exec path swap: the executed program reports 0
        # despite a forged signature. Verification must never invoke it.
        with patch.object(subprocess, "run", return_value=SimpleNamespace(returncode=0)) as invoked:
            with self.assertRaisesRegex(ValueError, "invalid_review_signature"):
                self.verify(signed)
            self.assertFalse(invoked.called)

    def test_rfc8032_vector_and_changed_message_and_noncanonical_scalar(self):
        # Public TEST 2, RFC 8032 section 7.1; no private key is needed.
        verifier = auth.ReviewAuthentication(b"", b"")
        verifier._verify_signature(RFC_PUBLIC, RFC_SIGNATURE, b"\x72")
        with self.assertRaisesRegex(ValueError, "invalid_review_signature"):
            verifier._verify_signature(RFC_PUBLIC, RFC_SIGNATURE, b"\x73")
        order = 2 ** 252 + 27742317777372353535851937790883648493
        scalar = int.from_bytes(RFC_SIGNATURE[32:], "little") + order
        noncanonical = RFC_SIGNATURE[:32] + scalar.to_bytes(32, "little")
        with self.assertRaisesRegex(ValueError, "invalid_review_signature"):
            verifier._verify_signature(RFC_PUBLIC, noncanonical, b"\x72")

    def test_zero_identity_negative_identity_noncanonical_and_mixed_order_keys(self):
        from nacl.bindings import crypto_core_ed25519_add
        order_two = bytes.fromhex("ec" + "ff" * 30 + "7f")
        mixed = crypto_core_ed25519_add(RFC_PUBLIC, order_two)
        for public in (bytes(32), auth.IDENTITY, order_two,
                       bytes.fromhex("ed" + "ff" * 30 + "7f"), mixed):
            with self.subTest(key=public.hex()), self.assertRaisesRegex(ValueError, "invalid_ed25519_public_key"):
                auth._validate_public_point(public)

    def test_main_subgroup_guard_does_not_rely_on_is_valid_point_alone(self):
        from nacl import bindings
        order_two = bytes.fromhex("ec" + "ff" * 30 + "7f")
        mixed = bindings.crypto_core_ed25519_add(RFC_PUBLIC, order_two)
        # Simulate the older predicate's documented mixed-subgroup false positive.
        with patch.object(bindings, "crypto_core_ed25519_is_valid_point", return_value=True):
            with self.assertRaisesRegex(ValueError, "invalid_ed25519_public_key"):
                auth._validate_public_point(mixed)

    def test_verification_in_another_process_and_foreign_run_replay(self):
        signed = self.signer.sign(self.bindings)
        root = Path(self.tmp.name)
        (root / "policy.json").write_bytes(signed.policy_bytes)
        (root / "attestation.json").write_bytes(signed.attestation)
        (root / "bindings.json").write_bytes(canonical(self.bindings))
        module_root = str(Path(auth.__file__).resolve().parent)
        code = """
import json,sys
from pathlib import Path
sys.path.insert(0,sys.argv[1])
from review_auth import ReviewAuthentication
root=Path(sys.argv[2])
authentication=ReviewAuthentication((root/'policy.json').read_bytes(), (root/'attestation.json').read_bytes())
try:
    result=authentication.verify(evaluation_id=sys.argv[3],executor_id='fixture-executor',
        evaluator_id='fixture-reviewer',bindings=json.loads((root/'bindings.json').read_bytes()),
        reviewers=[{'id':'fixture-reviewer','role':'agent'}])
    print(json.dumps({'authenticated':result['semantic_truth_certified'] is False}))
except ValueError as error:
    print(json.dumps({'rejected':str(error)}))
"""
        for evaluation, expected in (("fixture-run", {"authenticated": True}),
                                     ("foreign-run", {"rejected": "stale_or_foreign_review_attestation"})):
            result = subprocess.run([sys.executable, "-I", "-B", "-c", code,
                                     module_root, str(root), evaluation],
                                    capture_output=True, timeout=10, check=True)
            self.assertEqual(json.loads(result.stdout), expected)

    def test_valid_signature_cannot_cover_missing_display_segments(self):
        from test_review_auth_583 import ReviewerAuthenticationTests
        import score
        fixture = ReviewerAuthenticationTests()
        fixture.setUp()
        self.addCleanup(fixture.doCleanups)
        # A nonempty exposed diagnostic must be inventoried even for invalid output.
        args, context, signed = fixture.prepared_score(text="Unverified diagnostic.")
        review_path = next(args[3].glob("*.review.json"))
        review = json.loads(review_path.read_bytes())
        review["segments"] = []
        review_path.write_bytes(canonical(review))
        index_path = args[3] / "review-index.json"
        manifest = json.loads(index_path.read_bytes())
        manifest["reviews"][review["job_id"]] = digest(review_path.read_bytes())
        index_path.write_bytes(canonical(manifest))
        payload = json.loads(signed.attestation)["payload"]
        bindings = {key: payload[key] for key in self.bindings}
        bindings["review_manifest_sha256"] = digest(index_path.read_bytes())
        resigned = fixture.signer.sign(bindings, policy=json.loads(signed.policy_bytes))
        with self.assertRaisesRegex(ValueError, "display_segments_incomplete"):
            score.score(*args, authoritative_score=context, review_authentication=resigned)
        self.assertFalse(context.authority.calls)
        self.assertFalse(args[-1].exists())


if __name__ == "__main__":
    unittest.main()
