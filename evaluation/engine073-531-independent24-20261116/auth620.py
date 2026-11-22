"""Issue 620 restricted, offline-only RH-AUTH/1 verification primitives.

The operator pins registry bytes and evaluation context outside artifacts.
A verified signature is not authorization or proof of human independence.
No production enrollment, network, model invocation, or formal score path.
"""
import base64
import hashlib
import json
import re
from datetime import datetime, timezone
from pathlib import Path
import sys

# This evaluation uses the existing verifier and its pinned PyNaCl backend.
_V3 = Path(__file__).resolve().parents[1] / "cli-repair-paired-v1" / "normal_v3"
if str(_V3) not in sys.path:
    sys.path.insert(0, str(_V3))
from review_auth import ReviewAuthentication, _validate_public_point
from contracts import canonical, require

VERSION = "RH-AUTH/1"
DOMAIN = b"RH-AUTH/1\x00Ed25519\x00"
MAX_BYTES = 4 * 1024 * 1024
MAX_DEPTH = 32
MAX_INT = (1 << 53) - 1
KEY = re.compile(r"[A-Za-z0-9_][A-Za-z0-9_.:-]{0,127}\Z")
ID = re.compile(r"[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}\Z")
SHA = re.compile(r"[0-9a-f]{64}\Z")
B64 = re.compile(r"[A-Za-z0-9_-]+\Z")
TIME = re.compile(r"[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z\Z")
ARTIFACT_ROLES = {
    "inventory": {"author"}, "execution_conditions": {"coordinator"},
    "admission": {"safety_reviewer"}, "journal_event": {"executor"},
    "terminal": {"capture"}, "raw_evidence": {"capture"},
    "atomic_judgment": {"judge_a", "judge_b"},
    "adjudication": {"adjudicator"},
    "adjudication_ratification": {"adjudication_ratifier"},
    "blind_lock": {"blind_lock_admin"}, "unblind": {"unblinder"},
    "normative_ratification": {"normative_ratifier"},
    "implementation_acceptance": {"implementation_acceptor"},
    "run_permission": {"run_approver"}, "score_permission": {"score_approver"},
    "quality_result": {"formal_scorer_service"},
    "final_acceptance": {"final_approver"}, "release_permission": {"release_approver"},
    "identity_attestation": {"identity_authority"},
    "isolation_attestation": {"isolation_witness"},
    "checkpoint": {"journal_witness"},
    "billing_attestation": {"billing_witness"},
}
ROLES = frozenset().union(*ARTIFACT_ROLES.values())
ENVELOPE = frozenset(("schema_version", "artifact_type", "artifact_id", "evaluation_id",
    "protocol_version", "signer_id", "key_id", "signature_algorithm", "issued_at",
    "expires_at", "payload_digest", "parent_receipt_digests", "registry_digest",
    "scope", "signature"))


def sha(raw):
    require(type(raw) is bytes, "bytes_required")
    return hashlib.sha256(raw).hexdigest()


def match(value, pattern, reason):
    require(type(value) is str and pattern.fullmatch(value) is not None, reason)
    return value


def fields(value, expected):
    require(type(value) is dict and set(value) == set(expected), "schema_fields")
    return value


def _walk(value, depth=0):
    require(depth <= MAX_DEPTH, "canonical_depth")
    t = type(value)
    if t is dict:
        for k, v in value.items():
            match(k, KEY, "canonical_ascii_key")
            _walk(v, depth + 1)
    elif t is list:
        for v in value:
            _walk(v, depth + 1)
    elif t is str:
        value.encode("utf-8", errors="strict")
    elif t is int:
        require(0 <= value <= MAX_INT, "canonical_integer_range")
    else:
        require(t is bool or value is None, "canonical_type")


def encode(value):
    """Restricted profile, not general RFC 8785; no Unicode normalization."""
    _walk(value)
    raw = canonical(value)
    require(len(raw) <= MAX_BYTES, "canonical_size")
    return raw


def _pairs(pairs):
    out = {}
    for k, v in pairs:
        require(k not in out, "canonical_duplicate_key")
        out[k] = v
    return out


def _int(value):
    require(len(value) <= 16 and re.fullmatch(r"0|[1-9][0-9]*", value) is not None,
            "canonical_integer_syntax")
    n = int(value)
    require(n <= MAX_INT, "canonical_integer_range")
    return n


def _reject_number(_):
    raise ValueError("canonical_noninteger")


def decode(raw):
    require(type(raw) is bytes and 0 < len(raw) <= MAX_BYTES, "canonical_size")
    try:
        value = json.loads(raw.decode("utf-8", errors="strict"), object_pairs_hook=_pairs,
                           parse_int=_int, parse_float=_reject_number, parse_constant=_reject_number)
        require(encode(value) == raw, "canonical_bytes")
    except (UnicodeError, RecursionError) as exc:
        raise ValueError("canonical_invalid_unicode_or_depth") from exc
    return value


def b64encode(raw):
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii")


def b64decode(value, size):
    match(value, B64, "base64url_syntax")
    require(len(value) == (size * 8 + 5) // 6, "base64url_length")
    raw = base64.b64decode(value + "=" * (-len(value) % 4), altchars=b"-_", validate=True)
    require(len(raw) == size and b64encode(raw) == value, "base64url_noncanonical")
    return raw


def instant(value):
    match(value, TIME, "utc_time_syntax")
    out = datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").replace(tzinfo=timezone.utc)
    require(out.isoformat(timespec="seconds").replace("+00:00", "Z") == value, "utc_time_noncanonical")
    return out


def _unique_strings(values, pattern):
    require(type(values) is list, "string_list_required")
    for value in values:
        match(value, pattern, "invalid_list_value")
    require(len(set(values)) == len(values), "duplicate_list_value")


class Registry:
    """Immutable, operator-pinned snapshot; local clock/custody is synthetic only.

    No registry selection from an envelope. A new registry/version requires a
    new trusted pin; external current revocation and clock remain formal HOLD.
    """
    def __init__(self, raw, *, pinned_sha256, evaluation_id):
        match(pinned_sha256, SHA, "registry_pin")
        match(evaluation_id, ID, "evaluation_id")
        require(sha(raw) == pinned_sha256, "registry_pin_mismatch")
        value = fields(decode(raw), {"protocol_version", "evaluation_id", "principals"})
        require(value["protocol_version"] == VERSION and value["evaluation_id"] == evaluation_id,
                "registry_scope")
        require(type(value["principals"]) is list and 0 < len(value["principals"]) <= 64,
                "registry_inventory")
        names, keys, roles, groups, pubs = set(), set(), set(), set(), set()
        for p in value["principals"]:
            fields(p, {"subject_id", "aliases", "role", "conflict_groups", "key_id",
                       "public_key", "not_before", "expires_at", "revoked"})
            subject = match(p["subject_id"], ID, "subject")
            kid = match(p["key_id"], ID, "key_id")
            _unique_strings(p["aliases"], ID)
            _unique_strings(p["conflict_groups"], ID)
            require(p["conflict_groups"], "missing_conflict_group")
            identifiers = [subject, kid, *p["aliases"]]
            require(len(set(identifiers)) == len(identifiers) and not names.intersection(identifiers),
                    "principal_namespace_conflict")
            names.update(identifiers)
            role = match(p["role"], ID, "role")
            require(role in ROLES and role not in roles, "role_conflict")
            roles.add(role)
            require(not groups.intersection(p["conflict_groups"]), "principal_conflict")
            groups.update(p["conflict_groups"])
            pub = b64decode(p["public_key"], 32)
            _validate_public_point(pub)
            require(pub not in pubs and kid not in keys, "shared_key")
            pubs.add(pub)
            keys.add(kid)
            require(type(p["revoked"]) is bool and instant(p["not_before"]) < instant(p["expires_at"]),
                    "key_validity")
        self._raw = raw
        self._pin = pinned_sha256
        self._evaluation = evaluation_id

    @property
    def digest(self):
        return self._pin

    @property
    def evaluation_id(self):
        return self._evaluation

    def verify(self, envelope_raw, payload_raw, *, artifact_type, artifact_id,
               role, parents, now):
        """Verify an exact caller-independent expected operation and parent list.

        The service must supply expected bindings from its trusted graph.
        Return an artifact digest, never a redeemable capability.
        """
        require(type(now) is datetime and now.tzinfo is not None
                and now.utcoffset() == timezone.utc.utcoffset(now), "trusted_utc_required")
        e = fields(decode(envelope_raw), ENVELOPE)
        decode(payload_raw)
        require(type(artifact_type) is str and artifact_type in ARTIFACT_ROLES
                and type(role) is str and role in ARTIFACT_ROLES[artifact_type], "artifact_role")
        match(artifact_id, ID, "artifact_id")
        _unique_strings(parents, SHA)
        require(len(parents) <= 4096, "parent_limit")
        expected_scope = {"issue": 620, "evaluation_id": self._evaluation, "operation": artifact_type}
        fields(e["scope"], expected_scope)
        require(type(e["scope"]["issue"]) is int and e["scope"] == expected_scope
                and e["schema_version"] == "1" and e["protocol_version"] == VERSION
                and e["signature_algorithm"] == "Ed25519"
                and e["evaluation_id"] == self._evaluation and e["artifact_type"] == artifact_type
                and e["artifact_id"] == artifact_id and e["registry_digest"] == self._pin
                and e["payload_digest"] == sha(payload_raw), "envelope_binding")
        _unique_strings(e["parent_receipt_digests"], SHA)
        require(e["parent_receipt_digests"] == parents, "parent_binding")
        match(e["signer_id"], ID, "signer")
        match(e["key_id"], ID, "key")
        p = next((p for p in decode(self._raw)["principals"]
                  if p["key_id"] == e["key_id"]), None)
        require(p is not None and p["subject_id"] == e["signer_id"] and p["role"] == role
                and not p["revoked"], "untrusted_signer")
        issued, expiry = instant(e["issued_at"]), instant(e["expires_at"])
        require(instant(p["not_before"]) <= issued <= now < expiry <= instant(p["expires_at"]),
                "signature_validity")
        signature = b64decode(e["signature"], 64)
        unsigned = {k: v for k, v in e.items() if k != "signature"}
        ReviewAuthentication(b"", b"")._verify_signature(
            b64decode(p["public_key"], 32), signature, DOMAIN + encode(unsigned))
        return sha(envelope_raw)


def formal_entrypoint(*args, **kwargs):
    """Hard-disabled pending independent acceptance and all eight external gates."""
    return {"approval_status": "HOLD", "formal_score_authorized": False,
            "live_authorized": False, "quality_go": False, "release_go": False,
            "formal_f1": None, "policy_status": "UNRATIFIED",
            "formal_scorer_identity": None, "billing_status": "HOLD",
            "external_gates": {"EG" + str(i): "HOLD" for i in range(1, 9)}}
