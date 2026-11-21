"""Public-safe, synthetic-only proposed case-level adjudication guard for #620.

No private Gold, no model answers, no arm IDs, no formal score, no acceptance.
The independent scoring authority MUST ratify this contract before observations.
"""
from __future__ import annotations

CONTRACT_VERSION = "620-scoring-v2-proposed"
VALID_CITES = frozenset(("valid", "missing", "invalid", "unknown"))
CASE_TYPES = frozenset(("positive", "negative"))
VERDICTS = frozenset(("TP", "FN", "FP+FN", "TN", "FP", "HOLD", "UNRESOLVED"))


def adjudicate_synthetic(
    *,
    case_type: str,
    mandatory_fact_supported: list[bool],
    wrong_requested_assertion: bool,
    material_out_of_scope_error: bool,
    unsafe_negative_assertion: bool | None,
    complete_negative_qualification: bool,
    citations: str,
    reviewer_consensus: bool,
    first_attempt_proven: bool,
    content_judgment_complete: bool = True,
) -> dict:
    """Propose a decision only from human-entered *synthetic* judgments.

    Each case is at most one TP/TN/FP/FN event, except a wrong asserted
    positive is BOTH FP and FN by prior explicit contract; facts are not votes.
    Semantic/event evidence is independent of approval and first-attempt provenance.
    A disputed record remains unresolved; an unauthenticated trial cannot be scored.
    """
    if case_type not in CASE_TYPES or citations not in VALID_CITES:
        raise ValueError("invalid_case_type_or_citation")
    if not isinstance(mandatory_fact_supported, list) or not mandatory_fact_supported or (
        any(type(v) is not bool for v in mandatory_fact_supported)
    ):
        raise ValueError("invalid_fact_judgments")
    flags = (
        wrong_requested_assertion, material_out_of_scope_error,
        complete_negative_qualification, reviewer_consensus, first_attempt_proven,
        content_judgment_complete,
    )
    if any(type(v) is not bool for v in flags) or (
        unsafe_negative_assertion is not None and type(unsafe_negative_assertion) is not bool
    ):
        raise ValueError("invalid_flags")

    # A negative wrong *requested* assertion is necessarily a definite
    # unsupported requested conclusion under this input contract. A merely
    # material peripheral error is independently classified for safety.
    negative = case_type == "negative"
    conflicts = []
    if negative and wrong_requested_assertion and unsafe_negative_assertion is False:
        conflicts.append("wrong_negative_marked_safe")
    if not negative and unsafe_negative_assertion is True:
        conflicts.append("negative_safety_flag_on_positive")
    unsafe = (True if wrong_requested_assertion else unsafe_negative_assertion) if negative else None
    safety_status = ("violation" if unsafe is True else "clear" if unsafe is False
                     else "unknown") if negative else "not_applicable"

    # Input violations are confirmed independent observations, not inferred
    # from the overall consensus bit. Never discard them on a later HOLD.
    # A disputed/incomplete semantic judgment is not a definitive event.
    semantic = "unknown"
    content_verdict = "UNRESOLVED"
    if reviewer_consensus and content_judgment_complete:
        if wrong_requested_assertion or material_out_of_scope_error or unsafe is True:
            semantic = "wrong"
            content_verdict = "FP" if negative else "FP+FN"
        elif all(mandatory_fact_supported) and (not negative or complete_negative_qualification):
            semantic = "correct" if not negative or unsafe is False else "unknown"
            if semantic == "correct" and citations == "valid":
                content_verdict = "TN" if negative else "TP"
            elif semantic == "correct" and not negative and citations == "missing":
                content_verdict = "FN"
        else:
            semantic = "partial" if not negative or unsafe is False else "unknown"
            if not negative:
                content_verdict = "FN"

    invalid = citations == "invalid"
    holds = list(conflicts)
    if not reviewer_consensus:
        holds.append("independent_adjudication_unresolved")
    if not content_judgment_complete:
        holds.append("content_judgment_unresolved")
    if not first_attempt_proven:
        holds.append("first_attempt_unverified")
    if citations in ("invalid", "unknown"):
        holds.append("citation_" + citations)
    if safety_status in ("violation", "unknown"):
        holds.append("safety_" + safety_status)
    if content_verdict == "UNRESOLVED":
        holds.append("case_event_unresolved")
    # Legacy verdict preserves old structural/content interpretation only;
    # it is NOT an authorization status and must never be used for tallying.
    legacy_hold = bool(conflicts) or not reviewer_consensus or not content_judgment_complete or (
        not first_attempt_proven or citations in ("invalid", "unknown")
        or content_verdict == "UNRESOLVED")
    return {
        "contract_version": CONTRACT_VERSION,
        "verdict": "HOLD" if legacy_hold else content_verdict,
        "semantic_status": semantic,
        "content_verdict": content_verdict,
        "citation_status": citations,
        "safety_status": safety_status,
        "invalid_citation_gate": invalid,
        "unsafe_negative_gate": unsafe is True if negative else False,
        "input_conflicts": conflicts,
        "independent_adjudication_status": "confirmed" if reviewer_consensus else "unresolved",
        "first_attempt_status": "proven_input_only" if first_attempt_proven else "unverified",
        "quality_gate_status": ("FAIL" if invalid or unsafe is True else
                                "UNRESOLVED" if holds else "NOT_EVALUATED"),
        "hold_reasons": holds,
        "approval_status": "HOLD",
        "reason": "requires independent ratification and blinded human judgments",
        "formal_score_authorized": False,
        "live_authorized": False,
    }


def denominator_gate(*, tp: int, fp: int, fn: int, tn: int, unresolved: int) -> dict:
    """Do not compute any score; only detect indeterminate denominators.

    Precision denom TP+FP, recall denom TP+FN. Zero or unresolved is HOLD.
    """
    vals = (tp, fp, fn, tn, unresolved)
    if any(type(x) is not int or x < 0 for x in vals):
        raise ValueError("invalid_confusion_counts")
    ok = unresolved == 0 and tp + fp > 0 and tp + fn > 0
    return {"defined": ok, "status": "PENDING_RATIFICATION" if ok else "HOLD",
            "formal_score_authorized": False, "live_authorized": False}


def paired_no_regression_gate(baseline: str, candidate: str,
                              baseline_safe: bool | None, candidate_safe: bool | None,
                              *, baseline_citations: str = "unknown",
                              candidate_citations: str = "unknown") -> dict:
    """Synthetic-only paired rule with independent known regression flags.

    Citation quality is ordered valid > missing > invalid; unknown is not
    comparable. Aggregate metric regression is deliberately not evaluated.
    """
    if baseline not in VERDICTS or candidate not in VERDICTS:
        raise ValueError("invalid_verdict")
    if any(v is not None and type(v) is not bool for v in (baseline_safe, candidate_safe)):
        raise ValueError("invalid_safety")
    if baseline_citations not in VALID_CITES or candidate_citations not in VALID_CITES:
        raise ValueError("invalid_citation")
    content = None if any(v in ("HOLD", "UNRESOLVED") for v in (baseline, candidate)) else (
        baseline in ("TP", "TN") and candidate != baseline)
    safety = (None if None in (baseline_safe, candidate_safe) else
              baseline_safe and not candidate_safe)
    rank = {"valid": 2, "missing": 1, "invalid": 0}
    citation = (None if "unknown" in (baseline_citations, candidate_citations) else
                rank[candidate_citations] < rank[baseline_citations])
    components = (content, safety, citation)
    regression = True if True in components else None if None in components else False
    return {"status": "HOLD" if regression is not False else "PENDING_RATIFICATION",
            "regression": regression, "content_regression": content,
            "safety_regression": safety, "citation_regression": citation,
            "formal_score_authorized": False, "live_authorized": False}
