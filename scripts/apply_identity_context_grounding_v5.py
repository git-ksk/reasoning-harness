#!/usr/bin/env python3
"""Apply Harness-owned identity-context grounding after the v4 candidate.

The planner remains untrusted. Search reformulations may add only explicit trusted
identity context supplied separately from the target property/claim. Tool/Harness
suggestions remain selectable only through follow_suggested_query.
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
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v4";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v5";\n',
        "schema-v5",
    )

    text = replace_once(
        text,
        '    initial_query: &\'static str,\n    property_id: &\'static str,\n',
        '    initial_query: &\'static str,\n    identity_context: Option<&\'static str>,\n    property_id: &\'static str,\n',
        "case-trusted-identity-context",
    )

    cases = {
        '            task: "Determine the country of the city Vienna, Austria using external evidence.",\n            initial_query: "Vienna",\n            property_id: "P17",\n': '            task: "Determine the country of the city Vienna, Austria using external evidence.",\n            initial_query: "Vienna",\n            identity_context: Some("Austria"),\n            property_id: "P17",\n',
        '            initial_query: "Madrid",\n            property_id: "P17",\n': '            initial_query: "Madrid",\n            identity_context: Some("Spain"),\n            property_id: "P17",\n',
        '            task: "Check the claim that Vienna is in France. Acquire external evidence before deciding whether the claim is supported.",\n            initial_query: "Vienna",\n            property_id: "P17",\n': '            task: "Check the claim that Vienna is in France. Acquire external evidence before deciding whether the claim is supported.",\n            initial_query: "Vienna",\n            identity_context: None,\n            property_id: "P17",\n',
        '            initial_query: "Oxford",\n            property_id: "P17",\n': '            initial_query: "Oxford",\n            identity_context: Some("England"),\n            property_id: "P17",\n',
        '            initial_query: "Alexandria",\n            property_id: "P17",\n': '            initial_query: "Alexandria",\n            identity_context: Some("Egypt"),\n            property_id: "P17",\n',
        '            initial_query: "Victoria",\n            property_id: "P17",\n': '            initial_query: "Victoria",\n            identity_context: None,\n            property_id: "P17",\n',
        '            initial_query: "Georgia",\n            property_id: "P17",\n': '            initial_query: "Georgia",\n            identity_context: None,\n            property_id: "P17",\n',
    }
    for i, (old, new) in enumerate(cases.items(), 1):
        text = replace_once(text, old, new, f"dev-case-context-{i}")

    old_normalize = '''fn normalize_query(value: &str) -> String {\n    value\n        .split_whitespace()\n        .collect::<Vec<_>>()\n        .join(" ")\n        .to_lowercase()\n}\n'''
    new_normalize = '''fn query_terms(value: &str) -> BTreeSet<String> {\n    value\n        .split(|ch: char| !ch.is_alphanumeric())\n        .map(str::trim)\n        .filter(|value| !value.is_empty())\n        .map(str::to_lowercase)\n        .collect()\n}\n\nfn normalize_query(value: &str) -> String {\n    query_terms(value).into_iter().collect::<Vec<_>>().join(" ")\n}\n\nfn grounded_identity_context_search(case: CaseSpec, query: &str) -> bool {\n    let initial = query_terms(case.initial_query);\n    let proposed = query_terms(query);\n    if initial.is_empty() || !initial.is_subset(&proposed) {\n        return false;\n    }\n    let Some(context) = case.identity_context else {\n        return false;\n    };\n    let allowed_context = query_terms(context);\n    if allowed_context.is_empty() || !allowed_context.is_subset(&proposed) {\n        return false;\n    }\n    let allowed = initial.union(&allowed_context).cloned().collect::<BTreeSet<_>>();\n    proposed.is_subset(&allowed)\n}\n\nfn identity_context_progress_item(case: CaseSpec, query: &str) -> Option<String> {\n    grounded_identity_context_search(case, query).then(|| {\n        format!(\n            "trusted_identity_context:{}",\n            normalize_query(case.identity_context.unwrap_or_default())\n        )\n    })\n}\n'''
    text = replace_once(text, old_normalize, new_normalize, "canonical-query-and-grounding")

    old_resolver_sig = '''fn planner_action_resolution(\n    action: &PlannerAction,\n    observation: &ToolObservation,\n    tried_queries: &BTreeSet<String>,\n) -> Result<(Option<String>, bool), &'static str> {\n'''
    new_resolver_sig = '''fn planner_action_resolution(\n    case: CaseSpec,\n    action: &PlannerAction,\n    observation: &ToolObservation,\n    tried_queries: &BTreeSet<String>,\n) -> Result<(Option<String>, bool), &'static str> {\n'''
    text = replace_once(text, old_resolver_sig, new_resolver_sig, "resolver-case-context")

    old_search = '''            if tried_queries.contains(&normalize_query(&query)) {\n                return Err("duplicate_search_query");\n            }\n            Ok((Some(query), false))\n'''
    new_search = '''            if tried_queries.contains(&normalize_query(&query)) {\n                return Err("duplicate_search_query");\n            }\n            if !grounded_identity_context_search(case, &query) {\n                return Err("search_not_grounded_in_trusted_identity_context");\n            }\n            Ok((Some(query), false))\n'''
    text = replace_once(text, old_search, new_search, "reject-ungrounded-search")

    # All runtime/test resolver calls now receive the trusted case input.
    text = text.replace(
        'planner_action_resolution(&action, &planner_observation, &tried_queries)',
        'planner_action_resolution(case, &action, &planner_observation, &tried_queries)',
    )

    old_target_sig = 'fn target_progress_items(state: &Value) -> BTreeSet<String> {\n    let mut out = BTreeSet::new();\n'
    new_target_sig = 'fn target_progress_items(case: CaseSpec, query: &str, state: &Value) -> BTreeSet<String> {\n    let mut out = BTreeSet::new();\n    if let Some(item) = identity_context_progress_item(case, query) {\n        out.insert(item);\n    }\n'
    text = replace_once(text, old_target_sig, new_target_sig, "context-is-target-progress")
    text = replace_once(
        text,
        '        let new_items = target_progress_items(&observation.search_state)\n',
        '        let new_items = target_progress_items(case, &query, &observation.search_state)\n',
        "runtime-context-progress",
    )

    old_prompt_header = '''    let available_suggestion = suggested_query(&observation.search_state)\n        .filter(|query| !tried_queries.contains(&normalize_query(query)));\n    let available_actions = if available_suggestion.is_some() {\n        "search, follow_suggested_query, or stop"\n    } else {\n        "search or stop (follow_suggested_query is unavailable in this state)"\n    };\n'''
    new_prompt_header = '''    let available_suggestion = suggested_query(&observation.search_state)\n        .filter(|query| !tried_queries.contains(&normalize_query(query)));\n    let context_search_available = case.identity_context.is_some();\n    let available_actions = match (context_search_available, available_suggestion.is_some()) {\n        (true, true) => "search, follow_suggested_query, or stop",\n        (true, false) => "search or stop (follow_suggested_query is unavailable in this state)",\n        (false, true) => "follow_suggested_query or stop (search cannot add unstated identity context)",\n        (false, false) => "stop only (no trusted identity context or validated suggested query is available)",\n    };\n'''
    text = replace_once(text, old_prompt_header, new_prompt_header, "prompt-available-actions")

    old_format = '''        "Task: {}\\nProperty to retrieve: {} ({})\\nLast qualified tool observation: {}\\nCompact search state: {}\\nAlready tried normalized queries: {}\\nRound: {}. Remaining tool-call budget: {}.\\nAvailable typed actions now: {}. {} If a validated suggested_query is available, prefer follow_suggested_query instead of regenerating or editing it. Never decide whether the target fact is true and never override the identity gate; the Harness owns identity sufficiency, evidence admission, and final correctness.",\n        case.task,\n        case.property_id,\n        case.value_kind,\n'''
    new_format = '''        "Task: {}\\nTrusted identity context (Harness-owned input, distinct from target property/claim): {}\\nProperty to retrieve: {} ({})\\nLast qualified tool observation: {}\\nCompact search state: {}\\nAlready tried normalized queries: {}\\nRound: {}. Remaining tool-call budget: {}.\\nAvailable typed actions now: {}. {} Search may add only the trusted identity context shown above; target-property words or candidate-generated context are not identity context. If a validated suggested_query is available, prefer follow_suggested_query instead of regenerating or editing it. Never decide whether the target fact is true and never override the identity gate; the Harness owns identity sufficiency, evidence admission, and final correctness.",\n        case.task,\n        case.identity_context.unwrap_or("<none>"),\n        case.property_id,\n        case.value_kind,\n'''
    text = replace_once(text, old_format, new_format, "prompt-trusted-context")

    old_follow = '''    let follow_available = suggested_query(&observation.search_state)\n        .is_some_and(|query| !tried_queries.contains(&normalize_query(&query)));\n    let action_contract = if follow_available {\n        "Return exactly one JSON object: {\\"action\\":\\"search\\",\\"query\\":\\"entity label/title\\"}, {\\"action\\":\\"follow_suggested_query\\",\\"query\\":null}, or {\\"action\\":\\"stop\\",\\"query\\":null}."\n    } else {\n        "Return exactly one JSON object: {\\"action\\":\\"search\\",\\"query\\":\\"entity label/title\\"} or {\\"action\\":\\"stop\\",\\"query\\":null}. follow_suggested_query is unavailable in this state."\n    };\n'''
    new_follow = '''    let follow_available = suggested_query(&observation.search_state)\n        .is_some_and(|query| !tried_queries.contains(&normalize_query(&query)));\n    let context_search_available = case.identity_context.is_some();\n    let action_contract = match (context_search_available, follow_available) {\n        (true, true) => "Return exactly one JSON object: {\\"action\\":\\"search\\",\\"query\\":\\"entity surface plus only trusted identity context\\"}, {\\"action\\":\\"follow_suggested_query\\",\\"query\\":null}, or {\\"action\\":\\"stop\\",\\"query\\":null}.",\n        (true, false) => "Return exactly one JSON object: {\\"action\\":\\"search\\",\\"query\\":\\"entity surface plus only trusted identity context\\"} or {\\"action\\":\\"stop\\",\\"query\\":null}.",\n        (false, true) => "Return exactly one JSON object: {\\"action\\":\\"follow_suggested_query\\",\\"query\\":null} or {\\"action\\":\\"stop\\",\\"query\\":null}. search is unavailable because no trusted identity context exists.",\n        (false, false) => "Return exactly {\\"action\\":\\"stop\\",\\"query\\":null}. No trusted identity context or suggested query is available.",\n    };\n'''
    text = replace_once(text, old_follow, new_follow, "system-action-contract")

    text = replace_once(
        text,
        '        "You are an evidence-search planner inside a bounded verification harness. {action_contract} You propose actions only. You do not decide truth, identity sufficiency, evidence admission, or final correctness. If identity is insufficient and the task explicitly supplies disambiguating context that has not yet been searched, issue one search using only that stated context before stopping. If no legitimate disambiguation exists, stop."\n',
        '        "You are an evidence-search planner inside a bounded verification harness. {action_contract} You propose actions only. You do not decide truth, identity sufficiency, evidence admission, or final correctness. Trusted identity context is supplied separately by the Harness; never derive identity context from the target property, candidate list, or your own world knowledge. If no trusted context or validated suggested query is available, stop."\n',
        "system-untrusted-planner-context",
    )

    # Replace the v4 tests wholesale with context-grounding aware synthetic tests.
    start = text.index('\n#[cfg(test)]\nmod v4_contract_tests {')
    end = text.index('\n#[tokio::main]\n', start)
    tests = r'''
#[cfg(test)]
mod v5_contract_tests {
    use super::*;

    fn case_with_context(context: Option<&'static str>) -> CaseSpec {
        CaseSpec {
            id: "synthetic",
            task: "synthetic",
            initial_query: "Alpha",
            identity_context: context,
            property_id: "P1",
            value_kind: "entity",
            fact_key: "alpha.fact",
            target_value: "Q1",
            expected: ExpectedOutcome::Unknown,
        }
    }

    fn obs(state: Value) -> ToolObservation {
        ToolObservation {
            facts: BTreeMap::new(),
            observation: "test".into(),
            search_state: state,
        }
    }

    #[test]
    fn unavailable_follow_is_typed_invalid_with_zero_external_requests() {
        let case = case_with_context(Some("Region"));
        let action = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let tried = BTreeSet::new();
        let error = planner_action_resolution(case, &action, &obs(json!({"outcome_kind":"identity_insufficient"})), &tried)
            .expect_err("follow without suggestion must be unavailable");
        assert_eq!(error, "missing_suggested_query");
        let feedback = invalid_planner_action_feedback(&action, error);
        assert_eq!(feedback.search_state["external_requests"], 0);
    }

    #[test]
    fn planner_cannot_invent_identity_context_from_target_or_candidates() {
        let case = case_with_context(None);
        let action = PlannerAction { action: "search".into(), query: Some("Alpha country".into()) };
        assert_eq!(
            planner_action_resolution(case, &action, &obs(json!({"outcome_kind":"ambiguous"})), &BTreeSet::new()),
            Err("search_not_grounded_in_trusted_identity_context")
        );
        let feedback = invalid_planner_action_feedback(&action, "search_not_grounded_in_trusted_identity_context");
        assert_eq!(feedback.search_state["external_requests"], 0);
    }

    #[test]
    fn exact_trusted_context_is_allowed_but_extra_terms_are_not() {
        let case = case_with_context(Some("Region"));
        let valid = PlannerAction { action: "search".into(), query: Some("Alpha (Region)".into()) };
        let (query, followed) = planner_action_resolution(case, &valid, &obs(json!({})), &BTreeSet::new()).unwrap();
        assert_eq!(query.as_deref(), Some("Alpha (Region)"));
        assert!(!followed);

        let invented = PlannerAction { action: "search".into(), query: Some("Alpha city Region".into()) };
        assert_eq!(
            planner_action_resolution(case, &invented, &obs(json!({})), &BTreeSet::new()),
            Err("search_not_grounded_in_trusted_identity_context")
        );
    }

    #[test]
    fn duplicate_queries_are_semantic_over_punctuation() {
        let case = case_with_context(Some("Region"));
        let tried = BTreeSet::from([normalize_query("Alpha Region")]);
        let duplicate = PlannerAction { action: "search".into(), query: Some("Alpha (Region)".into()) };
        assert_eq!(
            planner_action_resolution(case, &duplicate, &obs(json!({})), &tried),
            Err("duplicate_search_query")
        );
    }

    #[test]
    fn trusted_context_use_is_target_progress_only_once() {
        let case = case_with_context(Some("Region"));
        let state = json!({"outcome_kind":"search_unresolved","wikidata_candidate_ids":["Q9"]});
        let progress = target_progress_items(case, "Alpha Region", &state);
        assert_eq!(progress, BTreeSet::from(["trusted_identity_context:region".to_string()]));
        assert!(!novelty_items(&state).is_empty());
    }

    #[test]
    fn candidate_churn_is_novelty_not_target_progress() {
        let case = case_with_context(None);
        let a = json!({"outcome_kind":"search_unresolved","wikidata_candidate_ids":["Q1"]});
        let b = json!({"outcome_kind":"search_unresolved","wikidata_candidate_ids":["Q2"]});
        assert!(target_progress_items(case, "Alpha", &a).is_empty());
        assert!(target_progress_items(case, "Alpha", &b).is_empty());
        assert_ne!(novelty_items(&a), novelty_items(&b));
    }

    #[test]
    fn a_b_a_cycle_is_rejected_before_tool_execution() {
        let case = case_with_context(Some("Region"));
        let tried = BTreeSet::from([normalize_query("Alpha"), normalize_query("Alpha Region")]);
        let action = PlannerAction { action: "search".into(), query: Some("Alpha (Region)".into()) };
        assert_eq!(
            planner_action_resolution(case, &action, &obs(json!({})), &tried),
            Err("duplicate_search_query")
        );
    }

    #[test]
    fn repair_limit_exhaustion_remains_safe_unknown() {
        let case = case_with_context(None);
        let tried = BTreeSet::new();
        let mut observation = obs(json!({"outcome_kind":"ambiguous"}));
        let invalid = PlannerAction { action: "search".into(), query: Some("Alpha country".into()) };
        let mut exhausted = false;
        for attempt in 0..=MAX_ACTION_REPAIRS_PER_OBSERVATION {
            let reason = planner_action_resolution(case, &invalid, &observation, &tried).unwrap_err();
            if attempt == MAX_ACTION_REPAIRS_PER_OBSERVATION {
                exhausted = true;
                break;
            }
            observation = invalid_planner_action_feedback(&invalid, reason);
        }
        assert!(exhausted);
        assert!(observation.facts.is_empty());
        assert!(!StopReason::PlannerActionRepairExhausted.operational());
    }
}
'''
    text = text[:start] + "\n" + tests + text[end:]

    text = replace_once(
        text,
        '        planner_action_policy: "planner chooses only currently available typed actions; Harness validates action availability, returns unavailable actions as typed no-external-request feedback, permits at most two bounded repairs, and substitutes exact tool-provided suggested_query only when follow_suggested_query is explicitly selected",\n',
        '        planner_action_policy: "planner is untrusted; Harness admits search only when its added terms are exactly grounded in separately supplied trusted identity context, never target-property/candidate/model context; follow_suggested_query selects the exact validated suggestion; invalid actions are blocked before external request with bounded repair",\n',
        "reported-grounding-policy",
    )

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
