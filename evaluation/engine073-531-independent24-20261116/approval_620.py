"""#620 prospective approval-wire validator, not a permission issuer.

Expected payload is built by a protected coordinator from frozen receipt
parents, not selected by the artifact. No external trust checkpoint or
formal scorer identity is enrolled here; permission is always HOLD.
"""
from auth620 import fields, decode, sha, match, SHA, ID, instant, require, formal_entrypoint

APPROVAL_ROLES = {
    "normative_ratification": "normative_ratifier",
    "implementation_acceptance": "implementation_acceptor",
    "run_permission": "run_approver",
    "score_permission": "score_approver",
    "final_acceptance": "final_approver",
    "release_permission": "release_approver",
}
DIGESTS = {"inventory_digest", "conditions_digest", "contract_digest", "policy_digest",
           "formal_implementation_digest", "design_receipt_digest",
           "independence_attestation_digest", "isolation_attestation_digest",
           "registry_digest", "checkpoint_digest"}
BASE_FIELDS = DIGESTS | {"evaluation_id", "legacy_annotation_receipts",
                         "implementation_acceptance_digest", "operation", "nonce", "decision",
                         "not_before", "expires_at", "parent_receipt_digests"}
EXTRA = {"run_permission": {"billing_evidence_digest", "schedule_digest"},
         "score_permission": {"closure_digest", "lock_digest", "opening_digest"},
         "final_acceptance": {"quality_result_digest"}, "release_permission": {"final_acceptance_digest"}}


def verify_approval(*, registry, kind, raw, expected_raw, envelope, now, parents):
    """Check future approval wire and DAG bindings; never authorize.

    Expected bytes, parents and current time must come from the protected
    coordinator. A caller's self-declared identity, billing or checkpoint
    digests are not external proofs and cannot be promoted by this function.
    """
    require(type(kind) is str and kind in APPROVAL_ROLES, "approval_kind")
    extra = EXTRA.get(kind, set())
    p = fields(decode(raw), BASE_FIELDS | extra)
    require(raw == expected_raw, "approval_expected_binding")
    for k in DIGESTS | extra:
        match(p[k], SHA, "approval_digest")
    match(p["evaluation_id"], ID, "approval_evaluation")
    match(p["nonce"], ID, "approval_nonce")
    require(p["operation"] == kind and p["registry_digest"] == registry.digest
            and p["evaluation_id"] == registry.evaluation_id, "approval_context")
    require(type(p["legacy_annotation_receipts"]) is list and len(p["legacy_annotation_receipts"]) == 3,
            "approval_annotation_count")
    for x in p["legacy_annotation_receipts"]:
        match(x, SHA, "approval_annotation")
    require(len(set(p["legacy_annotation_receipts"])) == 3, "approval_annotation_duplicate")
    if kind in {"normative_ratification", "implementation_acceptance"}:
        require(p["implementation_acceptance_digest"] is None, "prospective_implementation_cycle")
    else:
        match(p["implementation_acceptance_digest"], SHA, "approval_implementation")
    require(type(p["decision"]) is str and p["decision"] in ("PERMIT", "HOLD", "DENY"),
            "approval_decision")
    require(p["parent_receipt_digests"] == parents, "approval_parents")
    start,end=instant(p["not_before"]),instant(p["expires_at"])
    require(start <= now < end, "approval_validity")
    signed=decode(envelope)
    require(instant(signed["issued_at"]) <= start
            and instant(signed["expires_at"]) == end, "approval_envelope_validity")
    receipt=registry.verify(envelope,raw,artifact_type=kind,artifact_id=p["nonce"],
                           role=APPROVAL_ROLES[kind],parents=parents,now=now)
    return {**formal_entrypoint(), "signature_receipt":receipt, "signed_decision":p["decision"],
            "operation":kind, "receipt_only":True}
