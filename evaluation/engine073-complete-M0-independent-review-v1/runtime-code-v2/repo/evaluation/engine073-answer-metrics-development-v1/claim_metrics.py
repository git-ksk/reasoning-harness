#!/usr/bin/env python3
"""Future-run claim arithmetic over externally reviewed, byte-bound observations.

This module does NOT split prose, judge support, verify retrieval/citation traces,
or grant acceptance. Those reviews and the complete M0 remain separate work.
Never apply it to preserved first observations as a replacement scorer.
"""
from collections import Counter, defaultdict
import re

STRATA = ("provider", "track", "arm", "family", "answerability")
GROUNDING = {"grounded", "unsupported", "contradicted", "unresolved_binding",
             "wrong_scope_or_time", "overstated"}
OPERATIONS = {"complete", "incomplete", "not_run"}


def nonempty(value, name):
    if not isinstance(value, str) or not value.strip():
        raise ValueError("invalid_" + name)
    return value


def unique_strings(value, name):
    if not isinstance(value, list):
        raise ValueError("invalid_" + name)
    for item in value:
        nonempty(item, name)
    if len(set(value)) != len(value):
        raise ValueError("duplicate_" + name)
    return set(value)


def digest(value):
    if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise ValueError("invalid_observation_sha256")
    return value


def stratum(plan):
    values = tuple(nonempty(plan.get(k), k) for k in STRATA)
    if plan["track"] not in {"e2e_normal", "helper_opt_in"}:
        raise ValueError("invalid_track")
    if plan["arm"] not in {"baseline", "candidate"}:
        raise ValueError("invalid_arm")
    if plan["answerability"] not in {"full", "partial", "none"}:
        raise ValueError("invalid_answerability")
    return values


def ratios(a, s, g, h):
    """Preparation document's explicit zero-denominator rules, not binary F1."""
    p = s / a if a else None
    r = h / g if g else None
    if not g:
        f1, reason = None, "no_required_supported_groups"
    elif not a:
        f1, reason = 0.0, "no_displayed_fact_claims"
    else:
        f1 = 2 * p * r / (p + r) if p + r else 0.0
        reason = None
    return {"precision": p, "recall": r, "f1": f1,
            "f1_null_or_zero_reason": reason}


def score_case(plan, review=None):
    """Count claims only after complete external semantic/segmentation review.

    plan is a caller-created join envelope, not the immutable pre-run gold file.
    Its observation_sha256 identifies the externally captured complete observation
    bundle, including all displayed artifacts. It is null when no bundle exists.
    The caller must compute/check that bundle; comparing two digest fields alone
    does not establish trace integrity. Never add an output hash to frozen gold.
    proposition_id encodes reviewer-declared semantic identity (including negation,
    strength, scope and time). All occurrences remain visible for safety auditing.
    """
    if not isinstance(plan, dict):
        raise ValueError("invalid_plan")
    case_id = nonempty(plan.get("id"), "case_id")
    group = stratum(plan)
    required = unique_strings(plan.get("required_groups"), "required_groups")
    bound_hash = plan.get("observation_sha256")
    if bound_hash is not None:
        digest(bound_hash)
    if review is None:
        # Missing annotation does not prove that no request/run occurred.
        operation, reviewed, claims, violations = "unobserved", False, [], []
        reviewer = None
    else:
        if not isinstance(review, dict) or review.get("id") != case_id:
            raise ValueError("misbound_case_id")
        if tuple(review.get(k) for k in STRATA) != group:
            raise ValueError("misbound_stratum")
        operation = review.get("operation")
        if operation not in OPERATIONS:
            raise ValueError("invalid_operation")
        observed_hash = review.get("observation_sha256")
        if operation != "not_run":
            digest(bound_hash)
            digest(observed_hash)
        if observed_hash != bound_hash:
            raise ValueError("misbound_observation")
        reviewed = review.get("review_complete")
        if type(reviewed) is not bool:
            raise ValueError("review_complete_requires_boolean")
        reviewer = review.get("reviewer")
        if not isinstance(reviewer, dict):
            raise ValueError("invalid_reviewer")
        nonempty(reviewer.get("id"), "reviewer_id")
        if reviewer.get("role") not in {"human", "agent", "host"}:
            raise ValueError("invalid_reviewer_role")
        claims = review.get("claims")
        if not isinstance(claims, list):
            raise ValueError("invalid_claims")
        violations = review.get("hard_violations")
        if not isinstance(violations, list):
            raise ValueError("invalid_hard_violations")
        for violation in violations:
            nonempty(violation, "hard_violation")
        if operation == "not_run" and claims:
            raise ValueError("not_run_cannot_have_displayed_claims")
    occurrences, propositions = set(), defaultdict(list)
    for claim in claims:
        if not isinstance(claim, dict):
            raise ValueError("invalid_claim")
        occurrence = nonempty(claim.get("id"), "claim_occurrence_id")
        if occurrence in occurrences:
            raise ValueError("duplicate_claim_occurrence_id")
        occurrences.add(occurrence)
        proposition = nonempty(claim.get("proposition_id"), "proposition_id")
        if claim.get("grounding") not in GROUNDING:
            raise ValueError("invalid_grounding")
        hits = unique_strings(claim.get("satisfied_groups"), "satisfied_groups")
        if not hits <= required:
            raise ValueError("foreign_required_group")
        if hits and claim["grounding"] != "grounded":
            raise ValueError("unsupported_claim_cannot_earn_recall")
        propositions[proposition].append(claim)
    available = operation == "complete" and reviewed
    blockers = []
    if operation != "complete":
        blockers.append("operationally_" + operation)
    if not reviewed:
        blockers.append("semantic_review_incomplete")
    a = s = h = None
    matched = set()
    if available:
        a = len(propositions)
        supported = {p: cs for p, cs in propositions.items()
                     if all(c["grounding"] == "grounded" for c in cs)}
        s = len(supported)
        for cs in supported.values():
            for c in cs:
                matched.update(c["satisfied_groups"])
        h = len(matched)
        metrics = ratios(a, s, len(required), h)
    else:
        metrics = {"precision": None, "recall": None, "f1": None,
                   "f1_null_or_zero_reason": "unavailable_operation_or_review"}
    return {"id": case_id, **dict(zip(STRATA, group)),
            "observation_sha256": bound_hash, "operation": operation,
            "review_complete": reviewed, "reviewer": reviewer,
            "semantic_metrics_available": available, "blockers": blockers,
            "counts": {"displayed_unique_claims_A": a, "grounded_claims_S": s,
                       "required_groups_G": len(required), "satisfied_groups_H": h,
                       "displayed_claim_occurrences": len(occurrences)},
            "matched_groups": sorted(matched), **metrics,
            "known_hard_violations": dict(Counter(violations)),
            "full_display_safety_audit_complete": available}


def report(plans, reviews):
    """Report every planned row, with separate provider/track/arm/family layers.

    Missing rows remain explicit and reduce delivered_required_coverage. Semantic
    micro P/R use the SAME complete and reviewed subset; partial output never
    earns delivered credit. No threshold or release approval is inferred.
    """
    if not isinstance(plans, list) or not plans or not isinstance(reviews, list):
        raise ValueError("invalid_plans_or_reviews")
    ids = [p.get("id") if isinstance(p, dict) else None for p in plans]
    for value in ids:
        nonempty(value, "planned_id")
    if len(set(ids)) != len(ids):
        raise ValueError("duplicate_planned_id")
    index = {}
    for review in reviews:
        if not isinstance(review, dict) or review.get("id") not in ids:
            raise ValueError("foreign_or_missing_review_id")
        if review["id"] in index:
            raise ValueError("duplicate_review_id")
        index[review["id"]] = review
    cases = [score_case(plan, index.get(plan["id"])) for plan in plans]
    groups = defaultdict(list)
    for case in cases:
        groups[tuple(case[k] for k in STRATA)].append(case)
    summaries = []
    for group, rows in sorted(groups.items()):
        complete = [r for r in rows if r["semantic_metrics_available"]]
        sums = {k: sum(r["counts"][k] for r in complete) for k in
                ["displayed_unique_claims_A", "grounded_claims_S",
                 "required_groups_G", "satisfied_groups_H"]}
        planned_g = sum(r["counts"]["required_groups_G"] for r in rows)
        f1s = [r["f1"] for r in complete if r["f1"] is not None]
        violations = Counter()
        for row in rows:
            violations.update(row["known_hard_violations"])
        micro = ratios(sums["displayed_unique_claims_A"], sums["grounded_claims_S"],
                       sums["required_groups_G"], sums["satisfied_groups_H"]) if complete else {
            "precision": None, "recall": None, "f1": None,
            "f1_null_or_zero_reason": "no_complete_reviewed_rows"}
        summaries.append({**dict(zip(STRATA, group)), "planned_rows": len(rows),
                          "complete_reviewed_rows": len(complete),
                          "operational_states": dict(Counter(r["operation"] for r in rows)),
                          "unavailable_rows": len(rows) - len(complete),
                          "counts_complete_same_subset": sums, "micro": micro,
                          "macro_f1": sum(f1s) / len(f1s) if f1s else None,
                          "macro_defined_rows": len(f1s),
                          "macro_null_complete_rows": len(complete) - len(f1s),
                          "planned_required_groups": planned_g,
                          "delivered_required_coverage": sums["satisfied_groups_H"] / planned_g if planned_g else None,
                          "known_hard_violations": dict(violations),
                          "safety_unreviewed_rows": sum(not r["full_display_safety_audit_complete"] for r in rows)})
    return {"cases": cases, "strata": summaries, "acceptance": "NOT_EVALUATED",
            "scope": "Externally annotated claim arithmetic only; no trace/support verification, case quality gate, paired utility, live run, CLI or release acceptance."}
