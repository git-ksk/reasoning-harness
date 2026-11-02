"""#583: operator-enrolled Ed25519 reviewer authentication, prospective v3 only.

Verification authenticates an attestation, never its semantic truth. Enrollment,
key custody, conflict declarations and real runtime admission remain external.
No signing keys, enrollment CLI, provider calls or self-enrollment are supplied.
"""
import json
import re
from dataclasses import dataclass
from pathlib import Path

from contracts import canonical, digest, require, safe_id, sha_field, strict_keys

POLICY = "reason-cli-normal-v3-review-policy-v1"
ATTESTATION = "reason-cli-normal-v3-review-attestation-v1"
DOMAIN = (ATTESTATION + "\0").encode("ascii")
PINNED_PYNACL = "1.6.2"
IDENTITY = bytes.fromhex("01" + "00" * 31)
# Libsodium's documented main-subgroup check: (L - 1)P + P == identity.
L_MINUS_ONE = bytes.fromhex("ecd3f55c1a631258d69cf7a2def9de14" + "00" * 15 + "10")
PAYLOAD_FIELDS = {
    "contract", "evaluation_id", "policy_sha256", "receipt_sha256",
    "observation_index_sha256", "packet_index_sha256", "gold_sha256",
    "review_manifest_sha256", "executor_id", "evaluator_id",
    "execution_binding_sha256", "plan_sha256",
    "complete_display_reviewed", "full_sources_reviewed",
}


def _hex(value, size, reason):
    require(type(value) is str and re.fullmatch("[0-9a-f]{" + str(size * 2) + "}", value)
            is not None, reason)
    return bytes.fromhex(value)


def _backend():
    try:
        import nacl
        from nacl import bindings, exceptions, signing
    except ImportError as error:
        raise ValueError("pynacl_backend_required") from error
    require(nacl.__version__ == PINNED_PYNACL, "unadmitted_pynacl_version")
    return bindings, exceptions, signing


def _validate_public_point(public_key):
    """Reject noncanonical, small-order and mixed-order enrolled public keys.

    Uses only libsodium arithmetic. Its documented subgroup workaround also
    covers older 1.0.20 builds where is_valid_point alone was insufficient.
    """
    require(type(public_key) is bytes and len(public_key) == 32, "invalid_ed25519_public_key")
    bindings, exceptions, _ = _backend()
    require(bindings.crypto_core_ed25519_is_valid_point(public_key), "invalid_ed25519_public_key")
    try:
        multiple = bindings.crypto_scalarmult_ed25519_noclamp(L_MINUS_ONE, public_key)
        subgroup = bindings.crypto_core_ed25519_add(multiple, public_key)
    except (ValueError, exceptions.CryptoError) as error:
        raise ValueError("invalid_ed25519_public_key") from error
    require(subgroup == IDENTITY, "invalid_ed25519_public_key")


def validate_policy(raw):
    """Policy bytes come from the operator, never from review/receipt inputs."""
    require(type(raw) is bytes and len(raw) <= 65536, "invalid_review_policy")
    policy = json.loads(raw)
    require(canonical(policy) == raw, "noncanonical_review_policy")
    fields = {"contract", "policy_id", "principals", "keys"}
    strict_keys(policy, fields, fields, "review_policy")
    require(policy["contract"] == POLICY, "unknown_review_policy")
    safe_id(policy["policy_id"], "invalid_review_policy_id")
    require(type(policy["principals"]) is list and policy["principals"],
            "missing_review_principals")
    aliases = {}
    subjects = set()
    for principal in policy["principals"]:
        fields = {"subject_id", "aliases", "role", "conflict_groups"}
        strict_keys(principal, fields, fields, "review_principal")
        subject = safe_id(principal["subject_id"], "invalid_review_subject")
        require(subject not in subjects, "duplicate_review_subject")
        subjects.add(subject)
        require(principal["role"] in ("human", "agent"), "invalid_enrolled_role")
        names, conflicts = principal["aliases"], principal["conflict_groups"]
        require(type(names) is list and names and type(conflicts) is list and conflicts,
                "missing_principal_aliases_or_conflicts")
        require(len(set(conflicts)) == len(conflicts), "duplicate_conflict_group")
        for group in conflicts:
            safe_id(group, "invalid_conflict_group")
        for name in names:
            safe_id(name, "invalid_principal_alias")
            require(name not in aliases, "ambiguous_principal_alias")
            aliases[name] = principal
    require(type(policy["keys"]) is list and policy["keys"], "missing_reviewer_keys")
    keys, public_keys = {}, set()
    for key in policy["keys"]:
        fields = {"key_id", "subject_id", "public_key_hex", "revoked"}
        strict_keys(key, fields, fields, "review_key")
        kid = safe_id(key["key_id"], "invalid_review_key_id")
        require(kid not in keys and key["subject_id"] in subjects,
                "unknown_or_duplicate_review_key")
        public = _hex(key["public_key_hex"], 32, "invalid_ed25519_public_key")
        _validate_public_point(public)
        require(public not in public_keys, "reused_review_public_key")
        require(type(key["revoked"]) is bool, "invalid_key_revocation")
        keys[kid] = key
        public_keys.add(public)
    return aliases, keys


@dataclass(frozen=True)
class ReviewAuthentication:
    """Admitted host injects policy bytes, never a selected verification program.

    attestation is untrusted. policy_bytes and the admitted runtime are trust roots.
    Keeping bytes immutable also prevents a policy-file swap during scoring.
    """
    policy_bytes: bytes
    attestation: bytes

    def _verify_signature(self, public_key, signature, message):
        _validate_public_point(public_key)
        require(type(signature) is bytes and len(signature) == 64,
                "invalid_ed25519_signature")
        _, exceptions, signing = _backend()
        try:
            signing.VerifyKey(public_key).verify(message, signature)
        except exceptions.BadSignatureError as error:
            raise ValueError("invalid_review_signature") from error

    def verify(self, *, evaluation_id, executor_id, evaluator_id, bindings, reviewers):
        aliases, keys = validate_policy(self.policy_bytes)
        require(executor_id in aliases and evaluator_id in aliases,
                "unenrolled_execution_or_evaluation_principal")
        executor, evaluator = aliases[executor_id], aliases[evaluator_id]
        require(executor["subject_id"] != evaluator["subject_id"]
                and not set(executor["conflict_groups"]) & set(evaluator["conflict_groups"]),
                "reviewer_execution_conflict")
        require(type(self.attestation) is bytes and len(self.attestation) <= 65536,
                "missing_signed_review_attestation")
        envelope = json.loads(self.attestation)
        require(canonical(envelope) == self.attestation, "noncanonical_review_attestation")
        fields = {"contract", "algorithm", "key_id", "payload", "signature_hex"}
        strict_keys(envelope, fields, fields, "review_attestation")
        require(envelope["contract"] == ATTESTATION and envelope["algorithm"] == "Ed25519",
                "unknown_review_signature_contract")
        safe_id(envelope["key_id"], "invalid_review_signer")
        key = keys.get(envelope["key_id"])
        require(key is not None and not key["revoked"], "unknown_or_revoked_review_key")
        require(key["subject_id"] == evaluator["subject_id"], "wrong_enrolled_review_signer")
        safe_id(evaluation_id, "invalid_review_evaluation_id")
        require(set(bindings) == {
            "receipt_sha256", "observation_index_sha256", "packet_index_sha256",
            "gold_sha256", "review_manifest_sha256",
            "execution_binding_sha256", "plan_sha256",
        }, "missing_review_authentication_bindings")
        for name, value in bindings.items():
            sha_field(value, "invalid_review_binding_" + name)
        expected = {
            "contract": ATTESTATION, "evaluation_id": evaluation_id,
            "executor_id": executor_id, "evaluator_id": evaluator_id,
            "policy_sha256": digest(self.policy_bytes),
            "complete_display_reviewed": True, "full_sources_reviewed": True,
            **bindings,
        }
        strict_keys(envelope["payload"], PAYLOAD_FIELDS, PAYLOAD_FIELDS, "review_attestation_payload")
        require(envelope["payload"]["complete_display_reviewed"] is True
                and envelope["payload"]["full_sources_reviewed"] is True,
                "incomplete_authenticated_review")
        require(envelope["payload"] == expected, "stale_or_foreign_review_attestation")
        require(type(reviewers) is list and reviewers, "missing_authenticated_reviews")
        for reviewer in reviewers:
            require(type(reviewer) is dict and type(reviewer.get("id")) is str,
                    "invalid_authenticated_reviewer")
            enrolled = aliases.get(reviewer["id"])
            require(enrolled is not None and enrolled["subject_id"] == evaluator["subject_id"]
                    and reviewer.get("role") == enrolled["role"],
                    "unenrolled_or_forged_reviewer_identity")
        self._verify_signature(
            _hex(key["public_key_hex"], 32, "invalid_ed25519_public_key"),
            _hex(envelope["signature_hex"], 64, "invalid_ed25519_signature"),
            DOMAIN + canonical(expected),
        )
        return {
            "contract": ATTESTATION,
            "attestation_sha256": digest(self.attestation),
            "policy_sha256": digest(self.policy_bytes),
            "verifier_backend": "PyNaCl/libsodium", "pynacl_version": PINNED_PYNACL,
            "requirements_sha256": digest(Path(__file__).with_name("review-requirements.txt").read_bytes()),
            "key_id": key["key_id"], "subject_id": evaluator["subject_id"],
            "role": evaluator["role"], "semantic_truth_certified": False,
        }
