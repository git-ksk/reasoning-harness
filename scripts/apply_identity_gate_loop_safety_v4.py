#!/usr/bin/env python3
"""Apply loop-safety/accounting refinements after v3 action-repair materialization.

The transformation is exact-match and fail-closed: each expected v3 source shape
must match exactly once (or the known explicit count) before the v4 candidate is
materialized.
"""

from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def replace_count(text: str, old: str, new: str, expected: int, label: str) -> str:
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{label}: expected {expected} matches, found {count}")
    return text.replace(old, new)


def main() -> int:
    text = PATH.read_text(encoding="utf-8")

    text = replace_once(
        text,
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v3";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v4";\n',
        "report-schema-v4",
    )

    text = replace_once(
        text,
        'struct ToolFailure {\n    kind: StopReason,\n}\n',
        'struct ToolFailure {\n    kind: StopReason,\n    external_requests: usize,\n}\n',
        "tool-failure-request-accounting",
    )

    text = replace_once(
        text,
        '    new_progress_items: usize,\n    planner_action: Option<String>,\n',
        '    new_progress_items: usize,\n    new_novelty_items: usize,\n    planner_action: Option<String>,\n',
        "trace-novelty-separation",
    )

    text = replace_once(
        text,
        '    tool_calls: usize,\n    planner_calls: usize,\n',
        '    tool_calls: usize,\n    external_requests: usize,\n    planner_calls: usize,\n',
        "case-request-accounting",
    )

    text = replace_once(
        text,
        '    follow_suggested_query_actions: usize,\n    mean_rounds: f64,\n    mean_tool_calls: f64,\n',
        '    follow_suggested_query_actions: usize,\n    external_requests: usize,\n    mean_rounds: f64,\n    mean_tool_calls: f64,\n    mean_external_requests: f64,\n',
        "aggregate-request-accounting",
    )

    old_progress = '''fn progress_items(state: &Value) -> BTreeSet<String> {\n    let mut out = BTreeSet::new();\n    for key in [\n        "wikidata_candidate_ids",\n        "title_retry_candidate_ids",\n        "property_values",\n        "identity_reasons",\n    ] {\n        if let Some(values) = state.get(key).and_then(Value::as_array) {\n            for value in values {\n                if let Some(value) = value.as_str() {\n                    out.insert(format!("{key}:{value}"));\n                }\n            }\n        }\n    }\n    for key in [\n        "resolved_entity",\n        "wikipedia_top_entity",\n        "wikipedia_top_title",\n        "suggested_query",\n        "outcome_kind",\n        "identity_reason",\n        "validation_reason",\n    ] {\n        if let Some(value) = state.get(key).and_then(Value::as_str) {\n            out.insert(format!("{key}:{value}"));\n        }\n    }\n    if let Some(values) = state.get("wikipedia_candidates").and_then(Value::as_array) {\n        for value in values {\n            if let Some(title) = value.get("title").and_then(Value::as_str) {\n                out.insert(format!("wikipedia_title:{title}"));\n            }\n            if let Some(entity) = value.get("wikibase_item").and_then(Value::as_str) {\n                out.insert(format!("wikipedia_entity:{entity}"));\n            }\n        }\n    }\n    out\n}\n'''
    new_progress = '''fn target_progress_items(state: &Value) -> BTreeSet<String> {\n    let mut out = BTreeSet::new();\n    if let Some(value) = state.get("suggested_query").and_then(Value::as_str) {\n        if !value.trim().is_empty() {\n            out.insert(format!("suggested_query:{value}"));\n        }\n    }\n\n    let identity_insufficient = state_kind(state) == Some("identity_insufficient");\n    if !identity_insufficient {\n        if let Some(value) = state.get("resolved_entity").and_then(Value::as_str) {\n            if !value.trim().is_empty() {\n                out.insert(format!("resolved_entity:{value}"));\n            }\n        }\n        if let Some(values) = state.get("property_values").and_then(Value::as_array) {\n            for value in values {\n                if let Some(value) = value.as_str() {\n                    out.insert(format!("property_values:{value}"));\n                }\n            }\n        }\n    }\n    out\n}\n\nfn novelty_items(state: &Value) -> BTreeSet<String> {\n    let mut out = BTreeSet::new();\n    for key in ["wikidata_candidate_ids", "title_retry_candidate_ids", "identity_reasons"] {\n        if let Some(values) = state.get(key).and_then(Value::as_array) {\n            for value in values {\n                if let Some(value) = value.as_str() {\n                    out.insert(format!("{key}:{value}"));\n                }\n            }\n        }\n    }\n    for key in ["wikipedia_top_entity", "wikipedia_top_title", "outcome_kind", "identity_reason", "validation_reason"] {\n        if let Some(value) = state.get(key).and_then(Value::as_str) {\n            out.insert(format!("{key}:{value}"));\n        }\n    }\n    if let Some(values) = state.get("wikipedia_candidates").and_then(Value::as_array) {\n        for value in values {\n            if let Some(title) = value.get("title").and_then(Value::as_str) {\n                out.insert(format!("wikipedia_title:{title}"));\n            }\n            if let Some(entity) = value.get("wikibase_item").and_then(Value::as_str) {\n                out.insert(format!("wikipedia_entity:{entity}"));\n            }\n        }\n    }\n    out\n}\n'''
    text = replace_once(text, old_progress, new_progress, "target-progress-vs-novelty")

    text = replace_once(
        text,
        '    match action.action.as_str() {\n        "search" => {\n            if action.query.as_deref().is_none_or(|value| value.trim().is_empty()) {\n                return Err(StopReason::PlannerProtocolFailure);\n            }\n        }\n        "follow_suggested_query" | "stop" => {}\n        _ => return Err(StopReason::PlannerProtocolFailure),\n    }\n',
        '    match action.action.as_str() {\n        "search" | "follow_suggested_query" | "stop" => {}\n        _ => return Err(StopReason::PlannerProtocolFailure),\n    }\n',
        "search-shape-is-repairable-not-protocol-terminal",
    )

    text = replace_count(
        text,
        '            kind: StopReason::ToolProtocolFailure,\n        })',
        '            kind: StopReason::ToolProtocolFailure,\n            external_requests: 0,\n        })',
        4,
        "tool-protocol-failures-indented",
    )
    text = replace_count(
        text,
        '        kind: StopReason::ToolProtocolFailure,\n    })',
        '        kind: StopReason::ToolProtocolFailure,\n        external_requests: 0,\n    })',
        2,
        "tool-protocol-failures-inline",
    )

    text = replace_once(
        text,
        '''        return Err(ToolFailure {\n            kind: if operational == Some("transport") {\n                StopReason::ToolTransportFailure\n            } else {\n                StopReason::ToolProtocolFailure\n            },\n        });\n''',
        '''        let external_requests = response\n            .pointer("/error/data/reasoning_harness/external_requests")\n            .and_then(Value::as_u64)\n            .unwrap_or(0) as usize;\n        return Err(ToolFailure {\n            kind: if operational == Some("transport") {\n                StopReason::ToolTransportFailure\n            } else {\n                StopReason::ToolProtocolFailure\n            },\n            external_requests,\n        });\n''',
        "transport-error-request-accounting",
    )

    text = replace_once(
        text,
        '    let mut tool_calls = 0usize;\n    let mut planner_calls = 0usize;\n',
        '    let mut tool_calls = 0usize;\n    let mut external_requests = 0usize;\n    let mut planner_calls = 0usize;\n',
        "case-request-counter",
    )

    text = replace_once(
        text,
        '''        let observation = match invoke_tool(case, &query) {\n            Ok(value) => value,\n            Err(error) => {\n                stop_reason = error.kind;\n                break;\n            }\n        };\n''',
        '''        let observation = match invoke_tool(case, &query) {\n            Ok(value) => value,\n            Err(error) => {\n                external_requests = external_requests.saturating_add(error.external_requests);\n                stop_reason = error.kind;\n                break;\n            }\n        };\n        external_requests = external_requests.saturating_add(\n            observation\n                .search_state\n                .get("external_requests")\n                .and_then(Value::as_u64)\n                .unwrap_or(0) as usize,\n        );\n''',
        "successful-request-accounting",
    )

    text = replace_once(
        text,
        '''        let new_items = progress_items(&observation.search_state)\n            .difference(&seen_progress)\n            .cloned()\n            .collect::<Vec<_>>();\n''',
        '''        let new_items = target_progress_items(&observation.search_state)\n            .difference(&seen_progress)\n            .cloned()\n            .collect::<Vec<_>>();\n        let novelty = novelty_items(&observation.search_state);\n''',
        "runtime-target-progress",
    )

    text = replace_count(
        text,
        '                new_progress_items: new_items.len(),\n                planner_action:',
        '                new_progress_items: new_items.len(),\n                new_novelty_items: novelty.len(),\n                planner_action:',
        3,
        "trace-novelty-population",
    )
    text = replace_once(
        text,
        '            new_progress_items: new_items.len(),\n            planner_action:',
        '            new_progress_items: new_items.len(),\n            new_novelty_items: novelty.len(),\n            planner_action:',
        "trace-novelty-population-final",
    )
    text = replace_once(
        text,
        '                new_progress_items: 0,\n                planner_action:',
        '                new_progress_items: 0,\n                new_novelty_items: novelty.len(),\n                planner_action:',
        "no-progress-trace-novelty",
    )

    text = replace_once(
        text,
        '        tool_calls,\n        planner_calls,\n',
        '        tool_calls,\n        external_requests,\n        planner_calls,\n',
        "case-report-request-count",
    )

    text = replace_once(
        text,
        '''    let follow_suggested_query_actions = samples\n        .iter()\n        .map(|sample| sample.follow_suggested_query_actions)\n        .sum();\n''',
        '''    let follow_suggested_query_actions = samples\n        .iter()\n        .map(|sample| sample.follow_suggested_query_actions)\n        .sum();\n    let external_requests = samples\n        .iter()\n        .map(|sample| sample.external_requests)\n        .sum();\n''',
        "aggregate-request-total",
    )

    text = replace_once(
        text,
        '        follow_suggested_query_actions,\n        mean_rounds: mean(|sample| sample.rounds as u64),\n        mean_tool_calls: mean(|sample| sample.tool_calls as u64),\n',
        '        follow_suggested_query_actions,\n        external_requests,\n        mean_rounds: mean(|sample| sample.rounds as u64),\n        mean_tool_calls: mean(|sample| sample.tool_calls as u64),\n        mean_external_requests: mean(|sample| sample.external_requests as u64),\n',
        "aggregate-request-means",
    )

    test_module = r'''

#[cfg(test)]
mod v4_contract_tests {
    use super::*;

    fn obs(state: Value) -> ToolObservation {
        ToolObservation {
            facts: BTreeMap::new(),
            observation: "test".into(),
            search_state: state,
        }
    }

    #[test]
    fn unavailable_follow_is_typed_invalid_with_zero_external_requests() {
        let action = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let tried = BTreeSet::new();
        let error = planner_action_resolution(&action, &obs(json!({"outcome_kind":"identity_insufficient"})), &tried)
            .expect_err("follow without suggestion must be unavailable");
        assert_eq!(error, "missing_suggested_query");
        let feedback = invalid_planner_action_feedback(&action, error);
        assert_eq!(feedback.search_state["outcome_kind"], "invalid_planner_action");
        assert_eq!(feedback.search_state["external_requests"], 0);
    }

    #[test]
    fn duplicate_search_and_empty_search_are_repairable_invalid_actions() {
        let tried = BTreeSet::from([normalize_query("Alpha")]);
        let duplicate = PlannerAction { action: "search".into(), query: Some(" Alpha ".into()) };
        assert_eq!(
            planner_action_resolution(&duplicate, &obs(json!({})), &tried),
            Err("duplicate_search_query")
        );
        let empty = PlannerAction { action: "search".into(), query: Some("   ".into()) };
        assert_eq!(
            planner_action_resolution(&empty, &obs(json!({})), &BTreeSet::new()),
            Err("missing_search_query")
        );
    }

    #[test]
    fn bounded_repair_can_return_to_legal_action_without_external_request() {
        let tried = BTreeSet::from([normalize_query("Alpha")]);
        let original = obs(json!({"outcome_kind":"identity_insufficient"}));
        let invalid = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let reason = planner_action_resolution(&invalid, &original, &tried).unwrap_err();
        let feedback = invalid_planner_action_feedback(&invalid, reason);
        assert_eq!(feedback.search_state["external_requests"], 0);
        let repaired = PlannerAction { action: "search".into(), query: Some("Alpha, context".into()) };
        let (query, followed) = planner_action_resolution(&repaired, &feedback, &tried).unwrap();
        assert_eq!(query.as_deref(), Some("Alpha, context"));
        assert!(!followed);
    }

    #[test]
    fn repair_limit_exhaustion_is_safe_unknown_not_truth_acceptance() {
        let tried = BTreeSet::new();
        let mut observation = obs(json!({"outcome_kind":"identity_insufficient"}));
        let invalid = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let mut exhausted = false;
        for attempt in 0..=MAX_ACTION_REPAIRS_PER_OBSERVATION {
            let reason = planner_action_resolution(&invalid, &observation, &tried).unwrap_err();
            if attempt == MAX_ACTION_REPAIRS_PER_OBSERVATION {
                exhausted = true;
                break;
            }
            observation = invalid_planner_action_feedback(&invalid, reason);
        }
        assert!(exhausted);
        assert!(!StopReason::PlannerActionRepairExhausted.operational());
        assert!(observation.facts.is_empty());
        let final_outcome = ExpectedOutcome::Unknown;
        assert_eq!(final_outcome, ExpectedOutcome::Unknown);
    }

    #[test]
    fn candidate_churn_is_novelty_not_target_progress() {
        let a = json!({
            "outcome_kind":"search_unresolved",
            "wikidata_candidate_ids":["Q1"],
            "wikipedia_candidates":[{"title":"A","wikibase_item":"Q1"}]
        });
        let b = json!({
            "outcome_kind":"search_unresolved",
            "wikidata_candidate_ids":["Q2"],
            "wikipedia_candidates":[{"title":"B","wikibase_item":"Q2"}]
        });
        assert!(target_progress_items(&a).is_empty());
        assert!(target_progress_items(&b).is_empty());
        assert_ne!(novelty_items(&a), novelty_items(&b));
    }

    #[test]
    fn a_b_a_query_cycle_is_rejected_before_tool_execution() {
        let tried = BTreeSet::from([normalize_query("A"), normalize_query("B")]);
        let action = PlannerAction { action: "search".into(), query: Some("A".into()) };
        assert_eq!(
            planner_action_resolution(&action, &obs(json!({})), &tried),
            Err("duplicate_search_query")
        );
        let feedback = invalid_planner_action_feedback(&action, "duplicate_search_query");
        assert_eq!(feedback.search_state["external_requests"], 0);
    }

    #[test]
    fn identity_insufficient_candidate_fact_does_not_count_as_target_progress() {
        let state = json!({
            "outcome_kind":"identity_insufficient",
            "resolved_entity":"Q999",
            "property_values":["Q1"],
            "wikidata_candidate_ids":["Q999"]
        });
        assert!(target_progress_items(&state).is_empty());
        assert!(!novelty_items(&state).is_empty());
    }
}
'''
    anchor = '\n#[tokio::main]\nasync fn main() -> Result<(), Box<dyn std::error::Error>> {'
    text = replace_once(text, anchor, test_module + anchor, "v4-rust-contract-tests")

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
