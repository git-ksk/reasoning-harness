#!/usr/bin/env python3
"""Precommitted quality gates for *future* independent Engine evaluations.

Metrics are adjunct gates, not a substitute for safety, paired baseline, provider,
normal-product, citation, human-label or distribution acceptance. Existing first
observations are never modified or re-scored by this module.
"""
from __future__ import annotations

import json
from pathlib import Path


POLICY_FILE = Path(__file__).resolve().parents[1] / "engine-07-realworld-protocol-v1.json"
LABELS = {"positive", "negative", "ambiguous"}
DECISIONS = {"accept", "withhold", "abstain"}


def score_quality(rows: list[dict], policy: dict, expected_count: int = 24) -> dict:
    """Score exactly one provider/track/candidate group with independently frozen labels.

    Required row: {id, expected, decision, complete, hard_violations}.
    Never infer expected labels from model output. No rows may be dropped.
    """
    if not isinstance(expected_count, int) or isinstance(expected_count, bool) or expected_count < 1:
        raise ValueError("expected_count must be a positive integer")
    if not isinstance(rows, list):
        raise ValueError("rows must be a list")
    if not isinstance(policy, dict):
        raise ValueError("policy must be an object")
    required = {
        "minimum_f1", "minimum_precision", "minimum_recall",
        "min_clear_positive_examples", "min_clear_negative_examples",
        "minimum_complete_rows_per_provider_track",
    }
    if not required.issubset(policy):
        raise ValueError("missing required policy thresholds")
    for key in ("minimum_f1", "minimum_precision", "minimum_recall"):
        v = policy[key]
        if isinstance(v, bool) or not isinstance(v, (int, float)) or not 0 < v <= 1:
            raise ValueError("invalid quality threshold: " + key)
    for key in ("min_clear_positive_examples", "min_clear_negative_examples"):
        v = policy[key]
        if isinstance(v, bool) or not isinstance(v, int) or v < 1:
            raise ValueError("invalid minimum class count: " + key)

    minimum_rows = policy["minimum_complete_rows_per_provider_track"]
    if isinstance(minimum_rows, bool) or not isinstance(minimum_rows, int) or minimum_rows < 1:
        raise ValueError("invalid minimum cohort size")
    if expected_count < minimum_rows:
        raise ValueError("expected_count smaller than precommitted cohort size")

    ids = set()
    group = None
    tp = fp = tn = fn = ambiguous = incomplete = hard = 0
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("invalid row")
        id_ = row.get("id")
        if not isinstance(id_, str) or not id_ or id_ in ids:
            raise ValueError("missing or duplicate id")
        ids.add(id_)
        provider, track, arm = row.get("provider"), row.get("track"), row.get("arm")
        if (not isinstance(provider, str) or not provider
                or track not in ("e2e_normal", "helper_opt_in") or arm != "candidate"):
            raise ValueError("unrecognized provider/track/candidate arm")
        row_group = (provider, track, arm)
        if group is None:
            group = row_group
        elif group != row_group:
            raise ValueError("pooled provider/track groups prohibited")
        if row.get("expected") not in LABELS or row.get("decision") not in DECISIONS:
            raise ValueError("unrecognized gold label or decision")
        if type(row.get("complete")) is not bool:
            raise ValueError("missing complete status")
        violations = row.get("hard_violations")
        if not isinstance(violations, list) or not all(isinstance(x, str) and x for x in violations):
            raise ValueError("invalid hard violations")
        hard += len(violations)
        if not row["complete"]:
            incomplete += 1
            continue
        if row["expected"] == "ambiguous":
            ambiguous += 1
            # An unqualified accept in an ambiguous case must never be hidden
            # by excluding that case from the binary score denominator.
            hard += int(row["decision"] == "accept")
        elif row["expected"] == "positive":
            tp += int(row["decision"] == "accept")
            fn += int(row["decision"] != "accept")
        else:
            fp += int(row["decision"] == "accept")
            tn += int(row["decision"] != "accept")

    positive = tp + fn
    negative = tn + fp
    precision = tp / (tp + fp) if tp + fp else None
    recall = tp / positive if positive else None
    f1 = (2 * tp / (2 * tp + fp + fn)) if (2 * tp + fp + fn) else 0.0
    f05 = (1.25 * tp / (1.25 * tp + fp + 0.25 * fn)) if (1.25 * tp + fp + 0.25 * fn) else 0.0
    reasons = []
    if len(rows) != expected_count:
        reasons.append("missing_or_extra_observations")
    if incomplete:
        reasons.append("operationally_incomplete")
    if hard:
        reasons.append("hard_safety_violation")
    if positive < policy["min_clear_positive_examples"] or negative < policy["min_clear_negative_examples"]:
        reasons.append("insufficient_clear_class_samples")
    if precision is None:
        reasons.append("undefined_precision_no_positive_predictions")
    elif precision < policy["minimum_precision"]:
        reasons.append("precision_below_minimum")
    if recall is None or recall < policy["minimum_recall"]:
        reasons.append("recall_below_minimum")
    if f1 < policy["minimum_f1"]:
        reasons.append("f1_below_minimum")
    return {
        "metric_gate_pass": not reasons,
        "provider": group[0] if group else None,
        "track": group[1] if group else None,
        "arm": group[2] if group else None,
        "blockers": reasons,
        "rows": len(rows), "expected_rows": expected_count,
        "tp": tp, "fp": fp, "fn": fn, "tn": tn,
        "ambiguous": ambiguous, "incomplete": incomplete, "hard_violations": hard,
        "precision": precision, "recall": recall, "f1": f1, "f0_5_diagnostic": f05,
        "f0_5_stretch_met": f05 >= policy["f_beta_0_5_stretch_goal_not_gate"],
        "binary_denominator": positive + negative,
        "accepted_coverage": (tp + fp) / len(rows) if rows else 0.0,
        "scope": "metric gate only; NEVER standalone CLI/source release approval",
    }


def configured_policy() -> dict:
    return json.loads(POLICY_FILE.read_text())["gates"]["candidate_quality"]
