#!/usr/bin/env python3
"""Apply Harness-owned canonical context follow-up after the v5 candidate.

The planner remains untrusted. Rank>1 evidence is never admitted directly.
When a rank>1 fact is withheld and the case has separately supplied trusted
identity context, the Harness exposes one canonical suggested_query. The planner
must explicitly choose follow_suggested_query; the Harness does not silently
rewrite a free-form query.
"""
from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")


def one(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def main() -> int:
    text = PATH.read_text(encoding="utf-8")

    text = one(
        text,
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v5";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v6";\n',
        "schema-v6",
    )

    old_resolver = '''fn planner_action_resolution(\n    case: CaseSpec,\n    action: &PlannerAction,\n    observation: &ToolObservation,\n    tried_queries: &BTreeSet<String>,\n) -> Result<(Option<String>, bool), &'static str> {\n    match action.action.as_str() {\n        "search" => {\n            let query = action\n                .query\n                .as_deref()\n                .map(str::trim)\n                .filter(|value| !value.is_empty())\n                .ok_or("missing_search_query")?\n                .to_string();\n            if tried_queries.contains(&normalize_query(&query)) {\n                return Err("duplicate_search_query");\n            }\n            if !grounded_identity_context_search(case, &query) {\n                return Err("search_not_grounded_in_trusted_identity_context");\n            }\n            Ok((Some(query), false))\n        }\n        "follow_suggested_query" => {\n            let query = suggested_query(&observation.search_state)\n                .ok_or("missing_suggested_query")?;\n            if tried_queries.contains(&normalize_query(&query)) {\n                return Err("suggested_query_already_tried");\n            }\n            Ok((Some(query), true))\n        }\n        "stop" => Ok((None, false)),\n        _ => Err("unknown_action"),\n    }\n}\n\nfn invalid_planner_action_feedback(action: &PlannerAction, reason: &str) -> ToolObservation {\n    ToolObservation {\n        facts: BTreeMap::new(),\n        observation: format!(\n            "planner action rejected before tool/external request: action={}; reason={}; choose only an available typed action; if the task supplies explicit disambiguating context that has not been searched, use it with search; otherwise stop",\n            action.action, reason\n        ),\n        search_state: json!({\n            "outcome_kind": "invalid_planner_action",\n            "invalid_action": action.action,\n            "invalid_query": action.query,\n            "validation_reason": reason,\n            "external_requests": 0,\n            "suggested_action": "search_or_stop"\n        }),\n    }\n}\n'''
    new_resolver = '''fn planner_action_resolution(\n    case: CaseSpec,\n    action: &PlannerAction,\n    observation: &ToolObservation,\n    tried_queries: &BTreeSet<String>,\n) -> Result<(Option<String>, bool), &'static str> {\n    match action.action.as_str() {\n        "search" => {\n            if suggested_query(&observation.search_state).is_some() {\n                return Err("canonical_suggestion_available_use_follow");\n            }\n            let query = action\n                .query\n                .as_deref()\n                .map(str::trim)\n                .filter(|value| !value.is_empty())\n                .ok_or("missing_search_query")?\n                .to_string();\n            if tried_queries.contains(&normalize_query(&query)) {\n                return Err("duplicate_search_query");\n            }\n            if !grounded_identity_context_search(case, &query) {\n                return Err("search_not_grounded_in_trusted_identity_context");\n            }\n            Ok((Some(query), false))\n        }\n        "follow_suggested_query" => {\n            let query = suggested_query(&observation.search_state)\n                .ok_or("missing_suggested_query")?;\n            if tried_queries.contains(&normalize_query(&query)) {\n                return Err("suggested_query_already_tried");\n            }\n            Ok((Some(query), true))\n        }\n        "stop" => Ok((None, false)),\n        _ => Err("unknown_action"),\n    }\n}\n\nfn invalid_planner_action_feedback(\n    action: &PlannerAction,\n    reason: &str,\n    prior: &ToolObservation,\n) -> ToolObservation {\n    let mut search_state = prior.search_state.clone();\n    if let Some(state) = search_state.as_object_mut() {\n        state.insert("outcome_kind".into(), Value::String("invalid_planner_action".into()));\n        state.insert("invalid_action".into(), Value::String(action.action.clone()));\n        state.insert("invalid_query".into(), json!(action.query));\n        state.insert("validation_reason".into(), Value::String(reason.into()));\n        state.insert("external_requests".into(), json!(0));\n    }\n    ToolObservation {\n        facts: BTreeMap::new(),\n        observation: format!(\n            "planner action rejected before tool/external request: action={}; reason={}; preserve the current Harness-advertised action set and choose only an available typed action",\n            action.action, reason\n        ),\n        search_state,\n    }\n}\n'''
    text = one(text, old_resolver, new_resolver, "canonical-follow-resolver")

    text = one(
        text,
        'fn qualify_identity(mut observation: ToolObservation) -> ToolObservation {\n',
        'fn qualify_identity(mut observation: ToolObservation, case: CaseSpec) -> ToolObservation {\n',
        "qualify-case-context",
    )

    old_insufficient = '''    state.insert(\n        "suggested_action".into(),\n        Value::String("search_with_existing_context_or_stop".into()),\n    );\n    observation.observation = format!(\n        "Harness identity qualification withheld upstream fact: reasons={}; use only stated task/observation context to seek stronger entity identity evidence, otherwise stop; upstream_observation={upstream_observation}",\n        reasons.join(",")\n    );\n'''
    new_insufficient = '''    if let Some(context) = case.identity_context {\n        state.insert(\n            "suggested_query".into(),\n            Value::String(format!("{} {}", case.initial_query, context)),\n        );\n        state.insert(\n            "suggested_action".into(),\n            Value::String("follow_suggested_query".into()),\n        );\n    } else {\n        state.insert("suggested_action".into(), Value::String("stop".into()));\n    }\n    observation.observation = format!(\n        "Harness identity qualification withheld upstream fact: reasons={}; if a canonical suggested_query is present the planner may explicitly follow it, otherwise stop; upstream_observation={upstream_observation}",\n        reasons.join(",")\n    );\n'''
    text = one(text, old_insufficient, new_insufficient, "canonical-context-suggestion")

    old_invoke = '''    Ok(qualify_identity(ToolObservation {\n        facts,\n        observation,\n        search_state,\n    }))\n'''
    new_invoke = '''    Ok(qualify_identity(\n        ToolObservation {\n            facts,\n            observation,\n            search_state,\n        },\n        case,\n    ))\n'''
    text = one(text, old_invoke, new_invoke, "invoke-qualify-case")

    old_actions = '''    let context_search_available = case.identity_context.is_some();\n    let available_actions = match (context_search_available, available_suggestion.is_some()) {\n        (true, true) => "search, follow_suggested_query, or stop",\n        (true, false) => "search or stop (follow_suggested_query is unavailable in this state)",\n        (false, true) => "follow_suggested_query or stop (search cannot add unstated identity context)",\n        (false, false) => "stop only (no trusted identity context or validated suggested query is available)",\n    };\n'''
    new_actions = '''    let context_search_available = case.identity_context.is_some();\n    let available_actions = if available_suggestion.is_some() {\n        "follow_suggested_query or stop (canonical Harness suggestion available; free-form search is unavailable)"\n    } else if context_search_available {\n        "search or stop (no canonical suggestion is available)"\n    } else {\n        "stop only (no trusted identity context or validated suggested query is available)"\n    };\n'''
    text = one(text, old_actions, new_actions, "follow-first-available-actions")

    old_prompt = '''        "Task: {}\\nTrusted identity context (Harness-owned input, distinct from target property/claim): {}\\nProperty to retrieve: {} ({})\\nLast qualified tool observation: {}\\nCompact search state: {}\\nAlready tried normalized queries: {}\\nRound: {}. Remaining tool-call budget: {}.\\nAvailable typed actions now: {}. {} Search may add only the trusted identity context shown above; target-property words or candidate-generated context are not identity context. If a validated suggested_query is available, prefer follow_suggested_query instead of regenerating or editing it. Never decide whether the target fact is true and never override the identity gate; the Harness owns identity sufficiency, evidence admission, and final correctness.",\n'''
    new_prompt = '''        "Task: {}\\nTrusted identity context (Harness-owned input, distinct from target property/claim): {}\\nProperty to retrieve: {} ({})\\nLast qualified tool observation: {}\\nCompact search state: {}\\nAlready tried normalized queries: {}\\nRound: {}. Remaining tool-call budget: {}.\\nAvailable typed actions now: {}. {} If a validated suggested_query is available, choose follow_suggested_query to execute that exact query; do not regenerate or edit it. If search is available, its query may contain only the entity-surface tokens and all trusted-identity-context tokens, with no connector/descriptive/target-property/candidate-derived words. Never decide whether the target fact is true and never override the identity gate; the Harness owns identity sufficiency, evidence admission, and final correctness.",\n'''
    text = one(text, old_prompt, new_prompt, "prompt-no-plus-token")

    old_contract = '''    let context_search_available = case.identity_context.is_some();\n    let action_contract = match (context_search_available, follow_available) {\n        (true, true) => "Return exactly one JSON object: {\\"action\\":\\"search\\",\\"query\\":\\"entity surface plus only trusted identity context\\"}, {\\"action\\":\\"follow_suggested_query\\",\\"query\\":null}, or {\\"action\\":\\"stop\\",\\"query\\":null}.",\n        (true, false) => "Return exactly one JSON object: {\\"action\\":\\"search\\",\\"query\\":\\"entity surface plus only trusted identity context\\"} or {\\"action\\":\\"stop\\",\\"query\\":null}.",\n        (false, true) => "Return exactly one JSON object: {\\"action\\":\\"follow_suggested_query\\",\\"query\\":null} or {\\"action\\":\\"stop\\",\\"query\\":null}. search is unavailable because no trusted identity context exists.",\n        (false, false) => "Return exactly {\\"action\\":\\"stop\\",\\"query\\":null}. No trusted identity context or suggested query is available.",\n    };\n'''
    new_contract = '''    let context_search_available = case.identity_context.is_some();\n    let action_contract = if follow_available {\n        "Return exactly one JSON object: {\\"action\\":\\"follow_suggested_query\\",\\"query\\":null} or {\\"action\\":\\"stop\\",\\"query\\":null}. A canonical Harness suggestion exists, so search is unavailable."\n    } else if context_search_available {\n        "Return exactly one JSON object: {\\"action\\":\\"search\\",\\"query\\":\\"entity surface and trusted identity context only; no other words\\"} or {\\"action\\":\\"stop\\",\\"query\\":null}."\n    } else {\n        "Return exactly {\\"action\\":\\"stop\\",\\"query\\":null}. No trusted identity context or suggested query is available."\n    };\n'''
    text = one(text, old_contract, new_contract, "follow-first-system-contract")

    text = text.replace(
        'planner_observation = invalid_planner_action_feedback(&action, reason);',
        'planner_observation = invalid_planner_action_feedback(&action, reason, &planner_observation);',
    )

    start = text.index('\n#[cfg(test)]\nmod v5_contract_tests {')
    end = text.index('\n#[tokio::main]\n', start)
    tests = r'''
#[cfg(test)]
mod v6_contract_tests {
    use super::*;

    fn case(context: Option<&'static str>) -> CaseSpec {
        CaseSpec {
            id: "synthetic",
            task: "synthetic",
            initial_query: "Alpha",
            identity_context: context,
            property_id: "P1",
            value_kind: "entity",
            fact_key: "alpha.fact",
            target_value: "Q9",
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

    fn rank2_fact() -> ToolObservation {
        ToolObservation {
            facts: BTreeMap::from([("alpha.fact".into(), "Q9".into())]),
            observation: "upstream rank2 fact".into(),
            search_state: json!({
                "outcome_kind": "fact_resolved",
                "resolved_entity": "Q1",
                "corroboration_rank": 2,
                "wikipedia_candidates": [{"title":"Alpha","wikibase_item":"Q1","disambiguation":false}],
                "property_values": ["Q9"]
            }),
        }
    }

    #[test]
    fn rank2_with_trusted_context_remains_unadmitted_and_gets_canonical_suggestion() {
        let qualified = qualify_identity(rank2_fact(), case(Some("Region")));
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["outcome_kind"], "identity_insufficient");
        assert_eq!(qualified.search_state["identity_supported"], false);
        assert_eq!(qualified.search_state["suggested_query"], "Alpha Region");
        assert_eq!(qualified.search_state["suggested_action"], "follow_suggested_query");
        assert!(qualified.search_state["identity_reasons"].as_array().unwrap().iter().any(|v| v == "cross_source_agreement_not_rank1"));
    }

    #[test]
    fn rank2_without_trusted_context_remains_unadmitted_and_stops() {
        let qualified = qualify_identity(rank2_fact(), case(None));
        assert!(qualified.facts.is_empty());
        assert!(qualified.search_state.get("suggested_query").is_none());
        assert_eq!(qualified.search_state["suggested_action"], "stop");
    }

    #[test]
    fn canonical_follow_executes_exact_suggestion() {
        let c = case(Some("Region"));
        let observation = obs(json!({"suggested_query":"Alpha Region"}));
        let tried = BTreeSet::from([normalize_query("Alpha")]);
        let action = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let (query, followed) = planner_action_resolution(c, &action, &observation, &tried).unwrap();
        assert_eq!(query.as_deref(), Some("Alpha Region"));
        assert!(followed);
    }

    #[test]
    fn freeform_search_is_unavailable_while_canonical_suggestion_exists() {
        let c = case(Some("Region"));
        let observation = obs(json!({"suggested_query":"Alpha Region"}));
        let action = PlannerAction { action: "search".into(), query: Some("Alpha Region".into()) };
        assert_eq!(
            planner_action_resolution(c, &action, &observation, &BTreeSet::new()),
            Err("canonical_suggestion_available_use_follow")
        );
    }

    #[test]
    fn invalid_feedback_preserves_suggestion_and_blocks_external_request() {
        let prior = obs(json!({
            "outcome_kind":"identity_insufficient",
            "suggested_query":"Alpha Region",
            "suggested_action":"follow_suggested_query"
        }));
        let action = PlannerAction { action: "search".into(), query: Some("Alpha plus Region".into()) };
        let feedback = invalid_planner_action_feedback(&action, "canonical_suggestion_available_use_follow", &prior);
        assert_eq!(feedback.search_state["suggested_query"], "Alpha Region");
        assert_eq!(feedback.search_state["suggested_action"], "follow_suggested_query");
        assert_eq!(feedback.search_state["external_requests"], 0);
    }

    #[test]
    fn unavailable_follow_is_typed_invalid_with_zero_external_requests() {
        let c = case(Some("Region"));
        let action = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let prior = obs(json!({"outcome_kind":"identity_insufficient"}));
        let error = planner_action_resolution(c, &action, &prior, &BTreeSet::new())
            .expect_err("follow without suggestion must be unavailable");
        assert_eq!(error, "missing_suggested_query");
        let feedback = invalid_planner_action_feedback(&action, error, &prior);
        assert_eq!(feedback.search_state["external_requests"], 0);
    }

    #[test]
    fn planner_cannot_invent_identity_context() {
        let c = case(None);
        let action = PlannerAction { action: "search".into(), query: Some("Alpha country".into()) };
        assert_eq!(
            planner_action_resolution(c, &action, &obs(json!({})), &BTreeSet::new()),
            Err("search_not_grounded_in_trusted_identity_context")
        );
    }

    #[test]
    fn exact_trusted_context_is_allowed_without_suggestion_but_extra_terms_are_not() {
        let c = case(Some("Region"));
        let valid = PlannerAction { action: "search".into(), query: Some("Alpha (Region)".into()) };
        let (query, followed) = planner_action_resolution(c, &valid, &obs(json!({})), &BTreeSet::new()).unwrap();
        assert_eq!(query.as_deref(), Some("Alpha (Region)"));
        assert!(!followed);

        let invented = PlannerAction { action: "search".into(), query: Some("Alpha plus Region".into()) };
        assert_eq!(
            planner_action_resolution(c, &invented, &obs(json!({})), &BTreeSet::new()),
            Err("search_not_grounded_in_trusted_identity_context")
        );
    }

    #[test]
    fn semantic_duplicate_and_a_b_a_cycle_are_rejected() {
        let c = case(Some("Region"));
        let tried = BTreeSet::from([normalize_query("Alpha"), normalize_query("Alpha Region")]);
        let action = PlannerAction { action: "search".into(), query: Some("Alpha (Region)".into()) };
        assert_eq!(
            planner_action_resolution(c, &action, &obs(json!({})), &tried),
            Err("duplicate_search_query")
        );
    }

    #[test]
    fn candidate_churn_is_novelty_not_target_progress() {
        let c = case(None);
        let a = json!({"outcome_kind":"search_unresolved","wikidata_candidate_ids":["Q1"]});
        let b = json!({"outcome_kind":"search_unresolved","wikidata_candidate_ids":["Q2"]});
        assert!(target_progress_items(c, "Alpha", &a).is_empty());
        assert!(target_progress_items(c, "Alpha", &b).is_empty());
        assert_ne!(novelty_items(&a), novelty_items(&b));
    }

    #[test]
    fn repair_limit_exhaustion_remains_safe_unknown() {
        let c = case(None);
        let tried = BTreeSet::new();
        let mut observation = obs(json!({"outcome_kind":"ambiguous"}));
        let invalid = PlannerAction { action: "search".into(), query: Some("Alpha country".into()) };
        let mut exhausted = false;
        for attempt in 0..=MAX_ACTION_REPAIRS_PER_OBSERVATION {
            let reason = planner_action_resolution(c, &invalid, &observation, &tried).unwrap_err();
            if attempt == MAX_ACTION_REPAIRS_PER_OBSERVATION {
                exhausted = true;
                break;
            }
            observation = invalid_planner_action_feedback(&invalid, reason, &observation);
        }
        assert!(exhausted);
        assert!(observation.facts.is_empty());
        assert!(!StopReason::PlannerActionRepairExhausted.operational());
    }
}
'''
    text = text[:start] + "\n" + tests + text[end:]

    text = one(
        text,
        '        planner_action_policy: "planner is untrusted; Harness admits search only when its added terms are exactly grounded in separately supplied trusted identity context, never target-property/candidate/model context; follow_suggested_query selects the exact validated suggestion; invalid actions are blocked before external request with bounded repair",\n',
        '        planner_action_policy: "planner is untrusted; Harness keeps rank1 identity sufficiency unchanged, emits one canonical query only from separately supplied trusted identity context after identity insufficiency, and executes it only when the planner explicitly selects follow_suggested_query; free-form search is unavailable while that suggestion exists; invalid actions preserve the suggestion and are blocked before external request",\n',
        "report-policy-v6",
    )

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
