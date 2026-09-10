#!/usr/bin/env python3
"""Prospective operational-observability and conservative-bound evaluator.

This module is intentionally independent of frozen natural-language E2E surfaces.
It consumes only fields present in canonical case reports shared by control and
candidate coordinates. Candidate-only diagnostic sidecars are not accepted.
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

SCHEMA_VERSION = "operational-observability-bounds-v1"
BOUND_IDENTITY = "natural-language-e2e-operational-bounds-v13"

OBSERVED = "observed"
CENSORED = "censored"
NOT_APPLICABLE = "not_applicable"

OBSERVED_PRE_TERMINAL_WITNESS = "observed_pre_terminal_witness"
OBSERVED_CLEAN_COMPLETION = "observed_clean_completion"
CENSORED_OPERATIONAL_TERMINAL = "censored_operational_terminal_before_witness"
CENSORED_PREREQUISITE = "censored_prerequisite_metric"
NOT_APPLICABLE_CASE_KIND = "not_applicable_case_kind"
NOT_APPLICABLE_PREREQUISITE_FALSE = "not_applicable_prerequisite_false"

HIGHER_IS_BETTER = "higher_is_better"
LOWER_IS_BETTER = "lower_is_better"
NON_WORSE = "non_worse"
STRICT_IMPROVEMENT = "strict_improvement"

PASS = "PASS"
FAIL = "FAIL"
INCONCLUSIVE = "INCONCLUSIVE"

HARD_ZERO_FIELDS = (
    "correctness_boundary_violations",
    "unsupported_structured_claims",
    "unsupported_exposed_assertions",
    "exposed_text_contract_violations",
    "missed_target_insufficiency",
    "identity_unsafe_admission",
    "mcp_output_authority_self_promotion",
    "session_external_calls_replayed",
    "duplicate_action_rejections",
)


@dataclass(frozen=True)
class Observation:
    state: str
    value: bool | int | float | None
    reason: str

    def as_dict(self) -> dict[str, Any]:
        return {"state": self.state, "value": self.value, "reason": self.reason}


def is_operational_terminal(case: dict[str, Any]) -> bool:
    """Return whether ordinary semantic completion was preempted operationally."""
    if isinstance(case.get("operational_failure"), dict):
        return True
    if case.get("stop_reason") == "operational_terminal":
        return True
    if int(case.get("generation_failure_observed") or 0) > 0:
        return True
    if int(case.get("typed_operational_action_failures") or 0) > 0:
        return True
    return False


def _na(reason: str = NOT_APPLICABLE_CASE_KIND) -> Observation:
    return Observation(NOT_APPLICABLE, None, reason)


def _clean(value: Any) -> Observation:
    return Observation(OBSERVED, value, OBSERVED_CLEAN_COMPLETION)


def _positive_witness(value: Any) -> Observation:
    return Observation(OBSERVED, value, OBSERVED_PRE_TERMINAL_WITNESS)


def _censored(reason: str = CENSORED_OPERATIONAL_TERMINAL) -> Observation:
    return Observation(CENSORED, None, reason)


def _investigation(case: dict[str, Any]) -> bool:
    return case.get("kind") == "investigation"


def _followup_case(case: dict[str, Any]) -> bool:
    return case.get("coverage_contract_kind") == "no_result_followup_observational"


def classify_case(case: dict[str, Any]) -> dict[str, Observation]:
    """Classify v13 scoring observations for one canonical case report."""
    terminal = is_operational_terminal(case)
    out: dict[str, Observation] = {}
    if terminal and "kind" not in case:
        raise ValueError(
            f"{case.get('id')}: operational terminal missing case kind; observability cannot be derived"
        )
    if terminal and _investigation(case) and "coverage_contract_kind" not in case:
        raise ValueError(
            f"{case.get('id')}: operational terminal missing coverage contract kind"
        )

    if not _investigation(case):
        for metric in (
            "target_recalled",
            "tool_selection_success",
            "avoidable_followup_stall",
            "trigger_exposed",
            "continuation_eligible",
            "mechanism_conformant",
            "downstream_followup_useful",
        ):
            out[metric] = _na()
    else:
        target = bool(case.get("target_recalled"))
        out["target_recalled"] = (
            _positive_witness(True)
            if terminal and target
            else _censored()
            if terminal
            else _clean(target)
        )

        tool = bool(case.get("tool_selection_success"))
        out["tool_selection_success"] = (
            _positive_witness(True)
            if terminal and tool
            else _censored()
            if terminal
            else _clean(tool)
        )

        if _followup_case(case):
            action_count = int(case.get("action_count") or 0)
            stall = bool(case.get("avoidable_followup_stall"))
            if terminal:
                out["avoidable_followup_stall"] = (
                    _positive_witness(False) if action_count > 0 else _censored()
                )
            else:
                out["avoidable_followup_stall"] = _clean(stall)

            trigger = case.get("trigger_exposed")
            if terminal:
                if trigger is True:
                    out["trigger_exposed"] = _positive_witness(True)
                elif (
                    case.get("trigger_first_relevant_capability") is not None
                    and case.get("trigger_first_relevant_status") is not None
                ):
                    out["trigger_exposed"] = _positive_witness(False)
                else:
                    out["trigger_exposed"] = _censored()
            else:
                out["trigger_exposed"] = _clean(bool(trigger))

            trigger_obs = out["trigger_exposed"]
            if trigger_obs.state == CENSORED:
                out["continuation_eligible"] = _censored(CENSORED_PREREQUISITE)
            elif trigger_obs.value is False:
                out["continuation_eligible"] = _na(NOT_APPLICABLE_PREREQUISITE_FALSE)
            else:
                eligible = case.get("continuation_eligible")
                if isinstance(eligible, bool):
                    out["continuation_eligible"] = (
                        _positive_witness(eligible) if terminal else _clean(eligible)
                    )
                else:
                    out["continuation_eligible"] = _censored(CENSORED_PREREQUISITE)

            eligible_obs = out["continuation_eligible"]
            if eligible_obs.state == CENSORED:
                out["mechanism_conformant"] = _censored(CENSORED_PREREQUISITE)
            elif eligible_obs.state == NOT_APPLICABLE or eligible_obs.value is False:
                out["mechanism_conformant"] = _na(NOT_APPLICABLE_PREREQUISITE_FALSE)
            else:
                conformant = case.get("mechanism_conformant")
                if terminal:
                    if conformant is True:
                        out["mechanism_conformant"] = _positive_witness(True)
                    elif conformant is False and case.get("mechanism_followup_status") is not None:
                        out["mechanism_conformant"] = _positive_witness(False)
                    else:
                        out["mechanism_conformant"] = _censored()
                else:
                    out["mechanism_conformant"] = _clean(bool(conformant))

            mechanism_obs = out["mechanism_conformant"]
            if mechanism_obs.state == CENSORED:
                out["downstream_followup_useful"] = _censored(CENSORED_PREREQUISITE)
            elif mechanism_obs.state == NOT_APPLICABLE or mechanism_obs.value is False:
                out["downstream_followup_useful"] = _na(NOT_APPLICABLE_PREREQUISITE_FALSE)
            else:
                useful = case.get("downstream_followup_useful")
                if terminal:
                    if useful is True:
                        out["downstream_followup_useful"] = _positive_witness(True)
                    elif case.get("mechanism_followup_status") is not None:
                        out["downstream_followup_useful"] = _positive_witness(False)
                    else:
                        out["downstream_followup_useful"] = _censored()
                else:
                    out["downstream_followup_useful"] = _clean(bool(useful))
        else:
            for metric in (
                "avoidable_followup_stall",
                "trigger_exposed",
                "continuation_eligible",
                "mechanism_conformant",
                "downstream_followup_useful",
            ):
                out[metric] = _na()

    if terminal and ("false_abstention" not in case or "target_grounded" not in case):
        out["false_abstention"] = _censored()
        out["target_grounded"] = _censored()
    elif "false_abstention" not in case or "target_grounded" not in case:
        out["false_abstention"] = _na(NOT_APPLICABLE_CASE_KIND)
        out["target_grounded"] = _na(NOT_APPLICABLE_CASE_KIND)
    else:
        grounded = bool(case.get("target_grounded"))
        if terminal:
            out["target_grounded"] = _positive_witness(True) if grounded else _censored()
            out["false_abstention"] = _positive_witness(False) if grounded else _censored()
        else:
            out["target_grounded"] = _clean(grounded)
            out["false_abstention"] = _clean(bool(case.get("false_abstention")))

    correctness = int(case.get("correctness_boundary_violations") or 0)
    if terminal and correctness == 0:
        out["correctness_boundary_violation"] = _censored()
    elif terminal:
        out["correctness_boundary_violation"] = _positive_witness(True)
    else:
        out["correctness_boundary_violation"] = _clean(correctness > 0)

    return out



def validate_canonical_report(report: dict[str, Any]) -> None:
    if "scoring_input" in report:
        raise ValueError("diagnostic sidecars are not valid scoring input")
    if not isinstance(report.get("cases"), list):
        raise ValueError("canonical report must contain a cases array")
    if not isinstance(report.get("aggregate"), dict):
        raise ValueError("canonical report must contain an aggregate object")
    schema = report.get("schema_version")
    if schema is not None and not str(schema).startswith("reason-natural-language-e2e-"):
        raise ValueError(f"unsupported canonical report schema: {schema}")

def classify_report(report: dict[str, Any]) -> list[dict[str, Any]]:
    validate_canonical_report(report)
    classified = []
    for case in report.get("cases") or []:
        observations = classify_case(case)
        classified.append(
            {
                "id": case.get("id"),
                "kind": case.get("kind"),
                "operational_terminal": is_operational_terminal(case),
                "observations": {k: v.as_dict() for k, v in observations.items()},
            }
        )
    return classified


def _select(classified: Iterable[dict[str, Any]], metric: str) -> list[tuple[str, Observation]]:
    selected: list[tuple[str, Observation]] = []
    for case in classified:
        raw = (case.get("observations") or {}).get(metric)
        if raw is None:
            continue
        selected.append(
            (
                str(case.get("id")),
                Observation(raw["state"], raw.get("value"), raw["reason"]),
            )
        )
    return selected


def binary_bounds(
    classified: Iterable[dict[str, Any]], metric: str, *, as_rate: bool
) -> dict[str, Any]:
    selected = _select(classified, metric)
    observed = [(cid, obs) for cid, obs in selected if obs.state == OBSERVED]
    censored = [(cid, obs) for cid, obs in selected if obs.state == CENSORED]
    na = [(cid, obs) for cid, obs in selected if obs.state == NOT_APPLICABLE]
    observed_true = sum(bool(obs.value) for _, obs in observed)
    denominator = len(observed) + len(censored)
    lower = observed_true
    upper = observed_true + len(censored)
    if as_rate:
        lower_value = lower / denominator if denominator else None
        upper_value = upper / denominator if denominator else None
    else:
        lower_value = lower
        upper_value = upper
    return {
        "metric": metric,
        "kind": "rate" if as_rate else "count",
        "lower_bound": lower_value,
        "upper_bound": upper_value,
        "observed_case_ids": [cid for cid, _ in observed],
        "censored_case_ids": [cid for cid, _ in censored],
        "not_applicable_case_ids": [cid for cid, _ in na],
        "observed_count": len(observed),
        "censored_count": len(censored),
        "not_applicable_count": len(na),
        "bound_identity": BOUND_IDENTITY,
    }


def conditional_rate_bounds(
    classified: Iterable[dict[str, Any]], denominator_metric: str, numerator_metric: str
) -> dict[str, Any]:
    cases = list(classified)
    possibilities: set[tuple[int, int]] = {(0, 0)}
    contributing: list[str] = []
    censored: list[str] = []
    na: list[str] = []

    for case in cases:
        cid = str(case.get("id"))
        obs = case.get("observations") or {}
        d = obs.get(denominator_metric)
        n = obs.get(numerator_metric)
        if d is None or n is None:
            continue

        d_state, d_value = d["state"], d.get("value")
        n_state, n_value = n["state"], n.get("value")
        local: set[tuple[int, int]]
        if d_state == NOT_APPLICABLE:
            local = {(0, 0)}
            na.append(cid)
        elif d_state == OBSERVED and d_value is False:
            local = {(0, 0)}
            na.append(cid)
        elif d_state == OBSERVED and d_value is True:
            contributing.append(cid)
            if n_state == OBSERVED:
                local = {(1, int(bool(n_value)))}
            elif n_state == CENSORED:
                local = {(1, 0), (1, 1)}
                censored.append(cid)
            else:
                raise ValueError(f"{cid}: numerator {numerator_metric} N/A with true denominator")
        elif d_state == CENSORED:
            censored.append(cid)
            local = {(0, 0), (1, 0), (1, 1)}
        else:
            raise ValueError(f"{cid}: unsupported denominator observation {d!r}")

        possibilities = {
            (d0 + d1, n0 + n1)
            for d0, n0 in possibilities
            for d1, n1 in local
        }

    defined_rates = [n / d for d, n in possibilities if d > 0]
    undefined_possible = any(d == 0 for d, _ in possibilities)
    return {
        "metric": f"{numerator_metric}_rate_given_{denominator_metric}",
        "kind": "conditional_rate",
        "lower_bound": min(defined_rates) if defined_rates else None,
        "upper_bound": max(defined_rates) if defined_rates else None,
        "undefined_possible": undefined_possible,
        "possible_aggregate_states": len(possibilities),
        "observed_denominator_true_case_ids": contributing,
        "censored_case_ids": sorted(set(censored)),
        "not_applicable_case_ids": sorted(set(na)),
        "bound_identity": BOUND_IDENTITY,
    }


def report_bounds(report: dict[str, Any]) -> dict[str, Any]:
    classified = classify_report(report)
    bounds = {
        "target_recall": binary_bounds(classified, "target_recalled", as_rate=True),
        "tool_selection_success_rate": binary_bounds(
            classified, "tool_selection_success", as_rate=True
        ),
        "false_abstentions": binary_bounds(classified, "false_abstention", as_rate=False),
        "avoidable_followup_stalls": binary_bounds(
            classified, "avoidable_followup_stall", as_rate=False
        ),
        "trigger_exposed_cases": binary_bounds(classified, "trigger_exposed", as_rate=False),
        "trigger_reachability_rate": binary_bounds(classified, "trigger_exposed", as_rate=True),
        "mechanism_conformance_rate": conditional_rate_bounds(
            classified, "continuation_eligible", "mechanism_conformant"
        ),
        "downstream_useful_followup_rate": conditional_rate_bounds(
            classified, "mechanism_conformant", "downstream_followup_useful"
        ),
    }
    return {
        "schema_version": SCHEMA_VERSION,
        "bound_identity": BOUND_IDENTITY,
        "source_schema_version": report.get("schema_version"),
        "source_corpus_identity": report.get("corpus_identity"),
        "source_scoring_identity": report.get("scoring_identity"),
        "coordinate_role": report.get("coordinate_role"),
        "provider": report.get("provider"),
        "model": report.get("model"),
        "operational_failures": (report.get("aggregate") or {}).get("operational_failures"),
        "cases": classified,
        "bounds": bounds,
    }


def compare_bound(
    candidate_value: float | int,
    control_bound: dict[str, Any],
    *,
    direction: str,
    requirement: str,
) -> dict[str, Any]:
    lower = control_bound.get("lower_bound")
    upper = control_bound.get("upper_bound")
    if lower is None or upper is None or control_bound.get("undefined_possible") is True:
        return {
            "classification": INCONCLUSIVE,
            "reason": "control bound is not fully defined",
            "candidate_value": candidate_value,
            "control_lower_bound": lower,
            "control_upper_bound": upper,
            "direction": direction,
            "requirement": requirement,
        }
    if direction not in (HIGHER_IS_BETTER, LOWER_IS_BETTER):
        raise ValueError(f"unsupported direction: {direction}")
    if requirement not in (NON_WORSE, STRICT_IMPROVEMENT):
        raise ValueError(f"unsupported requirement: {requirement}")

    if direction == HIGHER_IS_BETTER and requirement == NON_WORSE:
        classification = PASS if candidate_value >= upper else FAIL if candidate_value < lower else INCONCLUSIVE
    elif direction == HIGHER_IS_BETTER:
        classification = PASS if candidate_value > upper else FAIL if candidate_value <= lower else INCONCLUSIVE
    elif requirement == NON_WORSE:
        classification = PASS if candidate_value <= lower else FAIL if candidate_value > upper else INCONCLUSIVE
    else:
        classification = PASS if candidate_value < lower else FAIL if candidate_value >= upper else INCONCLUSIVE

    return {
        "classification": classification,
        "candidate_value": candidate_value,
        "control_lower_bound": lower,
        "control_upper_bound": upper,
        "direction": direction,
        "requirement": requirement,
    }


def row_safety_gate(report: dict[str, Any]) -> dict[str, Any]:
    aggregate = report.get("aggregate") or {}
    violations = {field: int(aggregate.get(field) or 0) for field in HARD_ZERO_FIELDS}
    return {
        "passed": all(value == 0 for value in violations.values()),
        "violations": violations,
    }


def candidate_hard_gate(report: dict[str, Any]) -> dict[str, Any]:
    aggregate = report.get("aggregate") or {}
    operational = int(aggregate.get("operational_failures") or 0)
    safety = row_safety_gate(report)
    return {
        "passed": operational == 0 and safety["passed"],
        "operational_failures": operational,
        "safety": safety,
    }


def paired_classification(
    control: dict[str, Any],
    candidate: dict[str, Any],
    requirements: list[dict[str, str]],
) -> dict[str, Any]:
    hard = candidate_hard_gate(candidate)
    control_safety = row_safety_gate(control)
    control_result = report_bounds(control)
    candidate_result = report_bounds(candidate)
    if not control_safety["passed"]:
        return {
            "schema_version": SCHEMA_VERSION,
            "classification": FAIL,
            "reason": "control has an observed correctness/safety violation",
            "control_safety_gate": control_safety,
            "candidate_hard_gate": hard,
            "control": control_result,
            "candidate": candidate_result,
            "comparisons": [],
        }
    if not hard["passed"]:
        return {
            "schema_version": SCHEMA_VERSION,
            "classification": FAIL,
            "reason": "candidate operational/correctness-safety hard gate failed",
            "control_safety_gate": control_safety,
            "candidate_hard_gate": hard,
            "control": control_result,
            "candidate": candidate_result,
            "comparisons": [],
        }

    comparisons = []
    for requirement in requirements:
        metric = requirement["metric"]
        c_bound = control_result["bounds"][metric]
        n_bound = candidate_result["bounds"][metric]
        if n_bound.get("lower_bound") != n_bound.get("upper_bound") or n_bound.get("undefined_possible"):
            comparisons.append(
                {
                    "metric": metric,
                    "classification": FAIL,
                    "reason": "candidate metric is not exactly observed despite hard-gate completion",
                }
            )
            continue
        comparison = compare_bound(
            n_bound["lower_bound"],
            c_bound,
            direction=requirement["direction"],
            requirement=requirement["requirement"],
        )
        comparisons.append({"metric": metric, **comparison})

    if any(x["classification"] == FAIL for x in comparisons):
        classification = FAIL
    elif any(x["classification"] == INCONCLUSIVE for x in comparisons):
        classification = INCONCLUSIVE
    else:
        classification = PASS
    return {
        "schema_version": SCHEMA_VERSION,
        "classification": classification,
        "control_safety_gate": control_safety,
        "candidate_hard_gate": hard,
        "control": control_result,
        "candidate": candidate_result,
        "comparisons": comparisons,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()
    report = json.loads(Path(args.report).read_text(encoding="utf-8"))
    result = report_bounds(report)
    text = json.dumps(result, indent=2, sort_keys=True)
    print(text)
    if args.output:
        Path(args.output).write_text(text + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
