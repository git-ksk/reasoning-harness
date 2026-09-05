#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

OPERATIONAL_TERMINALS = {"unavailable", "timed_out", "operational_failure"}


def load(path):
    if not path or not Path(path).exists():
        return None
    return json.loads(Path(path).read_text())


def summarize_resolution_layer(report):
    if report is None:
        return None
    samples = report.get("samples", [])
    operational = [s for s in samples if s.get("terminal_status") in OPERATIONAL_TERMINALS]
    semantic = [s for s in samples if s.get("terminal_status") not in OPERATIONAL_TERMINALS]
    false_acceptances = [
        s for s in semantic
        if s.get("expected") != "accept" and s.get("final_verdict") == "accept"
    ]
    false_abstentions = [
        s for s in semantic
        if s.get("expected") == "accept" and s.get("final_verdict") == "unknown"
    ]
    semantic_passes = [s for s in semantic if s.get("passed")]
    expected_unknown_semantic = [s for s in semantic if s.get("expected") == "unknown"]
    correct_unknown_semantic = [
        s for s in expected_unknown_semantic if s.get("final_verdict") == "unknown"
    ]
    raw_passes = sum(1 for s in samples if s.get("passed"))
    return {
        "samples": len(samples),
        "raw_passes": raw_passes,
        "raw_success_rate": (raw_passes / len(samples)) if samples else 0.0,
        "semantic_safety_clean": len(false_acceptances) == 0,
        "semantic_capability_clean": len(semantic_passes) == len(semantic),
        "infrastructure_clean": len(operational) == 0,
        "full_run_clean": bool(samples) and raw_passes == len(samples),
        "operational_unresolved": len(operational),
        "operational_rate": (len(operational) / len(samples)) if samples else 0.0,
        "semantic_samples": len(semantic),
        "semantic_passes": len(semantic_passes),
        "semantic_success_rate": (len(semantic_passes) / len(semantic)) if semantic else 0.0,
        "false_acceptances": len(false_acceptances),
        "false_abstentions": len(false_abstentions),
        "expected_unknown_semantic": len(expected_unknown_semantic),
        "correct_unknown_semantic": len(correct_unknown_semantic),
        "operational_failure_samples": [
            {"id": s.get("id"), "trial": s.get("trial"), "terminal_status": s.get("terminal_status")}
            for s in operational
        ],
        "semantic_failure_samples": [
            {
                "id": s.get("id"),
                "trial": s.get("trial"),
                "expected": s.get("expected"),
                "final_verdict": s.get("final_verdict"),
                "terminal_status": s.get("terminal_status"),
            }
            for s in semantic if not s.get("passed")
        ],
    }


def summarize_agentic(report):
    if report is None:
        return None
    aggregate = report.get("aggregate", {})
    samples = report.get("samples", [])

    invalid_by_sample = []
    invalid_action_observations = 0
    invalid_actions_blocked_before_external_request = 0
    for sample in samples:
        invalid_traces = [
            trace for trace in sample.get("traces", [])
            if (trace.get("search_state") or {}).get("outcome_kind") == "invalid_query"
        ]
        if invalid_traces:
            invalid_by_sample.append((sample, invalid_traces))
        invalid_action_observations += len(invalid_traces)
        invalid_actions_blocked_before_external_request += sum(
            1 for trace in invalid_traces
            if (trace.get("search_state") or {}).get("external_requests") == 0
        )

    samples_with_invalid_actions = len(invalid_by_sample)
    repaired_after_invalid_action = sum(
        1 for sample, _ in invalid_by_sample if sample.get("passed")
    )
    valid_tool_observations = sum(
        1
        for sample in samples
        for trace in sample.get("traces", [])
        if (trace.get("search_state") or {}).get("outcome_kind") != "invalid_query"
    )

    sample_count = aggregate.get("samples", 0)
    passed_samples = aggregate.get("passed_samples", 0)
    false_acceptances = aggregate.get("false_acceptances", 0)
    operational_unresolved = aggregate.get("operational_unresolved", 0)
    planner_failures = aggregate.get("planner_failures", 0)
    tool_failures = aggregate.get("tool_failures", 0)
    budget_exhaustions = aggregate.get("budget_exhaustions", 0)

    return {
        "samples": sample_count,
        "passed_samples": passed_samples,
        "raw_success_rate": aggregate.get("expectation_success_rate", 0.0),
        "semantic_safety_clean": false_acceptances == 0,
        "agent_capability_clean": (
            sample_count > 0
            and passed_samples == sample_count
            and planner_failures == 0
            and budget_exhaustions == 0
        ),
        "infrastructure_clean": operational_unresolved == 0 and tool_failures == 0,
        "full_run_clean": sample_count > 0 and passed_samples == sample_count,
        "false_acceptances": false_acceptances,
        "false_abstentions": aggregate.get("false_abstentions", 0),
        "operational_unresolved": operational_unresolved,
        "planner_failures": planner_failures,
        "tool_failures": tool_failures,
        "budget_exhaustions": budget_exhaustions,
        "no_progress_stops": aggregate.get("no_progress_stops", 0),
        "duplicate_query_stops": aggregate.get("duplicate_query_stops", 0),
        "invalid_action_observations": invalid_action_observations,
        "samples_with_invalid_actions": samples_with_invalid_actions,
        "invalid_action_sample_rate": (
            samples_with_invalid_actions / len(samples) if samples else 0.0
        ),
        "repaired_after_invalid_action": repaired_after_invalid_action,
        "invalid_action_repair_success_rate": (
            repaired_after_invalid_action / samples_with_invalid_actions
            if samples_with_invalid_actions else 0.0
        ),
        "invalid_actions_blocked_before_external_request": invalid_actions_blocked_before_external_request,
        "valid_tool_observations": valid_tool_observations,
        "mean_rounds": aggregate.get("mean_rounds", 0.0),
        "mean_tool_calls": aggregate.get("mean_tool_calls", 0.0),
        "mean_planner_calls": aggregate.get("mean_planner_calls", 0.0),
        "mean_model_tokens": aggregate.get("mean_model_tokens", 0.0),
        "p50_elapsed_ms": aggregate.get("p50_elapsed_ms", 0),
        "p95_elapsed_ms": aggregate.get("p95_elapsed_ms", 0),
        "stop_reasons": [
            {
                "id": sample.get("id"),
                "trial": sample.get("trial"),
                "expected": sample.get("expected"),
                "final_outcome": sample.get("final_outcome"),
                "stop_reason": sample.get("stop_reason"),
                "operational_failure": sample.get("operational_failure"),
                "rounds": sample.get("rounds"),
                "tool_calls": sample.get("tool_calls"),
                "planner_calls": sample.get("planner_calls"),
                "invalid_actions": sum(
                    1
                    for trace in sample.get("traces", [])
                    if (trace.get("search_state") or {}).get("outcome_kind") == "invalid_query"
                ),
                "passed": sample.get("passed"),
            }
            for sample in samples
        ],
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--layer-a")
    parser.add_argument("--layer-b")
    parser.add_argument("--layer-c")
    parser.add_argument("--output")
    args = parser.parse_args()

    result = {
        "schema_version": "reason-mcp-layered-benchmark-summary-v2",
        "interpretation_order": [
            "semantic_safety",
            "agent_capability",
            "infrastructure_health",
            "efficiency",
        ],
        "layer_a_adapter": summarize_resolution_layer(load(args.layer_a)),
        "layer_b_fixed_policy": summarize_resolution_layer(load(args.layer_b)),
        "layer_c_agentic_planner": summarize_agentic(load(args.layer_c)),
        "notes": [
            "Semantic safety, agent capability, and infrastructure health are separate signals; do not collapse them into one score.",
            "Operational failures are excluded from semantic success/failure attribution for Layers A/B.",
            "Expected-unknown cases are only credited as semantic unknowns when the terminal status is non-operational.",
            "Layer C reports planner, tool, budget, no-progress, duplicate-stop, invalid-action, and repair provenance separately.",
            "An invalid planner action can be rejected before external acquisition; successful recovery is reported separately from first-action quality.",
            "Holdout results must not be used to retune the planner or budgets after observation.",
        ],
    }
    text = json.dumps(result, ensure_ascii=False, indent=2)
    if args.output:
        Path(args.output).write_text(text + "\n")
    print(text)


if __name__ == "__main__":
    main()
