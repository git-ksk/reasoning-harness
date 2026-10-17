#!/usr/bin/env python3
"""Future #520 source-qualification metric gate; synthetic API only.

The historic first-observation archives are never read or re-scored by this
module. Inputs are already independently reviewed, frozen future observations.
This gate is cumulative with #531 real-answer and #540 execution safety.
"""
import json
from pathlib import Path

_POLICY = Path(__file__).resolve().parents[1] / "engine073-source-gate-policy-v1.json"
_AXES = ("v17_requested_relation", "v30_relevant")
_TRUTH = ("positive", "negative")


def policy():
    return json.loads(_POLICY.read_text())


def score(rows, controls, config=None):
    """Each row: id, gold ('positive'|'negative'), complete, v17_requested_relation,
    v30_relevant; controls: complete/failed regression counts, history byte check,
    critical known defect list, operational failure count, frozen first-observation
    provenance assertions. Labels must NOT be derived from candidate outputs.
    """
    p = config or policy()
    if not isinstance(rows, list) or not isinstance(controls, dict):
        raise ValueError("invalid rows or controls")
    required_controls = (
        "known_regressions_total", "known_regressions_passed",
        "original_audit_total", "original_audit_passed",
        "historical_byte_equivalent", "known_unfixed_critical_defects",
        "operational_failures", "labels_frozen_before_observation",
        "candidate_frozen_before_observation", "first_observation_immutable",
    )
    if not all(k in controls for k in required_controls):
        raise ValueError("missing control evidence")
    if not isinstance(controls["known_unfixed_critical_defects"], list):
        raise ValueError("invalid critical defects")
    if any(type(controls[k]) is not int or controls[k] < 0 for k in (
        "known_regressions_total", "known_regressions_passed",
        "original_audit_total", "original_audit_passed", "operational_failures",
    )):
        raise ValueError("invalid regression/operational counts")
    if not all(type(controls[k]) is bool for k in (
        "historical_byte_equivalent", "labels_frozen_before_observation",
        "candidate_frozen_before_observation", "first_observation_immutable",
    )):
        raise ValueError("invalid immutable provenance checks")
    ids = set()
    counts = {axis: dict(tp=0, fp=0, tn=0, fn=0) for axis in _AXES}
    pos = neg = incomplete = 0
    for row in rows:
        if not isinstance(row, dict) or not isinstance(row.get("id"), str) or not row["id"] or row["id"] in ids:
            raise ValueError("missing/duplicate case identity")
        ids.add(row["id"])
        gold = row.get("gold")
        if gold not in _TRUTH or type(row.get("complete")) is not bool:
            raise ValueError("missing gold label/completeness")
        if not all(type(row.get(a)) is bool for a in _AXES):
            raise ValueError("missing/nonboolean public API observation")
        pos += int(gold == "positive")
        neg += int(gold == "negative")
        if not row["complete"]:
            incomplete += 1
            continue
        for axis in _AXES:
            pred = row[axis]
            key = ("tp" if pred else "fn") if gold == "positive" else ("fp" if pred else "tn")
            counts[axis][key] += 1

    blockers = []
    cohort = p["cohort"]
    if len(rows) < cohort["minimum_total"] or pos < cohort["minimum_positive"] or neg < cohort["minimum_negative"]:
        blockers.append("insufficient_independent_cohort")
    if incomplete or controls["operational_failures"]:
        blockers.append("incomplete_or_operational_failure")
    if (controls["known_regressions_total"] <= 0
        or controls["known_regressions_passed"] != controls["known_regressions_total"]
        or controls["original_audit_total"] != 14
        or controls["original_audit_passed"] != controls["original_audit_total"]):
        blockers.append("known_regressions_or_original_audit_not_100_percent")
    if not controls["historical_byte_equivalent"]:
        blockers.append("historical_api_byte_regression")
    if controls["known_unfixed_critical_defects"]:
        blockers.append("known_critical_defect")
    if not all(controls[k] for k in (
        "labels_frozen_before_observation",
        "candidate_frozen_before_observation",
        "first_observation_immutable",
    )):
        blockers.append("provenance_or_first_observation_violation")
    metrics = {}
    for axis in _AXES:
        c = counts[axis]
        tp, fp, fn, tn = (c[k] for k in ("tp", "fp", "fn", "tn"))
        precision = tp/(tp+fp) if tp+fp else None
        recall = tp/(tp+fn) if tp+fn else None
        f1 = 2*tp/(2*tp+fp+fn) if 2*tp+fp+fn else 0.0
        f05 = 1.25*tp/(1.25*tp+fp+0.25*fn) if (1.25*tp+fp+0.25*fn) else 0.0
        metrics[axis] = dict(**c, precision=precision, recall=recall, f1=f1,
                             f0_5_diagnostic=f05)
        if fp > 0:
            blockers.append(axis + ":unsafe_negative_admitted")
        if precision is None or precision < p["thresholds"]["minimum_precision"]:
            blockers.append(axis + ":precision_below_floor_or_undefined")
        if recall is None or recall < p["thresholds"]["minimum_recall"]:
            blockers.append(axis + ":recall_below_floor_or_undefined")
        if f1 < p["thresholds"]["minimum_f1"]:
            blockers.append(axis + ":f1_below_floor")
    return {"gate_pass":not blockers, "blockers":blockers,
            "observations":len(rows), "positives":pos, "negatives":neg,
            "incomplete":incomplete, "axes":metrics,
            "scope":"#520 future synthetic source API only, NOT #531 real-answer/CLI approval"}


def main():
    raise SystemExit("Use score(rows, controls) from the future frozen evaluator; "
                     "historic archives must not be scored or modified.")


if __name__ == "__main__":
    main()
