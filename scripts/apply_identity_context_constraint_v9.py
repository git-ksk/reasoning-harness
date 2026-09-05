#!/usr/bin/env python3
"""Issue #196 candidate: trusted context constrains identity sufficiency.

This transformer is intentionally entity-agnostic. It does not contain or
special-case any development or historical holdout entity.
"""
from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")


def one(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def fn_bounds(text: str, name: str) -> tuple[int, int]:
    marker = f"fn {name}("
    start = text.find(marker)
    if start < 0:
        raise SystemExit(f"function not found: {name}")
    brace = text.find("{", start)
    if brace < 0:
        raise SystemExit(f"function opening brace not found: {name}")
    depth = 0
    for i in range(brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return start, i + 1
    raise SystemExit(f"function closing brace not found: {name}")


def replace_fn(text: str, name: str, replacement: str) -> str:
    start, end = fn_bounds(text, name)
    return text[:start] + replacement.rstrip() + text[end:]


def insert_after_fn(text: str, name: str, addition: str) -> str:
    _, end = fn_bounds(text, name)
    return text[:end] + "\n\n" + addition.rstrip() + text[end:]


def main() -> int:
    text = PATH.read_text(encoding="utf-8")
    text = one(
        text,
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v8";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v9";\n',
        "schema-v9",
    )

    helpers = r'''fn canonical_identity_context_query(case: CaseSpec) -> Option<String> {
    case.identity_context
        .map(|context| format!("{}, {}", case.initial_query, context))
}

fn trusted_identity_context_search_remaining(
    case: CaseSpec,
    tried_queries: &BTreeSet<String>,
) -> bool {
    canonical_identity_context_query(case)
        .is_some_and(|query| !tried_queries.contains(&normalize_query(&query)))
}
'''
    text = insert_after_fn(text, "suggested_query", helpers)

    qualify = r'''fn qualify_identity(observation: ToolObservation, case: CaseSpec) -> ToolObservation {
    qualify_identity_for_query(observation, case, case.initial_query)
}

fn qualify_identity_for_query(
    mut observation: ToolObservation,
    case: CaseSpec,
    query: &str,
) -> ToolObservation {
    let context_required = case.identity_context.is_some();
    let context_verified = context_required && grounded_identity_context_search(case, query);
    let outcome_kind = state_kind(&observation.search_state).map(ToOwned::to_owned);

    if outcome_kind.as_deref() != Some("fact_resolved") {
        if context_required
            && !context_verified
            && outcome_kind.as_deref() != Some("invalid_query")
        {
            if let Some(state) = observation.search_state.as_object_mut() {
                state.insert("identity_context_required".into(), Value::Bool(true));
                state.insert("identity_context_verified".into(), Value::Bool(false));
                if let Some(query) = canonical_identity_context_query(case) {
                    state.insert("suggested_query".into(), Value::String(query));
                    state.insert(
                        "suggested_action".into(),
                        Value::String("follow_suggested_query".into()),
                    );
                }
                let upstream_observation = observation.observation.clone();
                observation.observation = format!(
                    "Harness requires trusted identity context before identity sufficiency; canonical follow-up is available only from Harness-owned context; upstream_observation={upstream_observation}"
                );
            }
        }
        return observation;
    }

    let resolved_entity = observation
        .search_state
        .get("resolved_entity")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let corroboration_rank = observation
        .search_state
        .get("corroboration_rank")
        .and_then(Value::as_u64);
    let wikipedia_top = observation
        .search_state
        .get("wikipedia_candidates")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .cloned();

    let mut reasons = Vec::<&str>::new();
    if resolved_entity.as_deref().is_none_or(str::is_empty) {
        reasons.push("missing_resolved_entity");
    }

    match wikipedia_top.as_ref() {
        None => reasons.push("missing_wikipedia_top_candidate"),
        Some(top) => {
            if top
                .get("disambiguation")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                reasons.push("wikipedia_top_is_disambiguation");
            }
            let wikipedia_top_entity = top.get("wikibase_item").and_then(Value::as_str);
            if wikipedia_top_entity.is_none_or(str::is_empty) {
                reasons.push("missing_wikipedia_top_entity");
            } else if wikipedia_top_entity != resolved_entity.as_deref() {
                reasons.push("resolved_entity_differs_from_wikipedia_top");
            }
        }
    }

    if corroboration_rank != Some(1) {
        reasons.push("cross_source_agreement_not_rank1");
    }
    if context_required && !context_verified {
        reasons.push("trusted_identity_context_not_verified_by_query");
    }

    let upstream_observation = observation.observation.clone();
    let Some(state) = observation.search_state.as_object_mut() else {
        observation.facts.clear();
        let mut fallback = json!({
            "outcome_kind": "identity_insufficient",
            "upstream_outcome_kind": "fact_resolved",
            "identity_supported": false,
            "identity_reasons": ["invalid_search_state_shape"],
            "identity_context_required": context_required,
            "identity_context_verified": false,
            "suggested_action": "stop"
        });
        if context_required && !context_verified {
            if let Some(query) = canonical_identity_context_query(case) {
                if let Some(state) = fallback.as_object_mut() {
                    state.insert("suggested_query".into(), Value::String(query));
                    state.insert(
                        "suggested_action".into(),
                        Value::String("follow_suggested_query".into()),
                    );
                }
            }
        }
        observation.search_state = fallback;
        observation.observation = format!(
            "Harness identity qualification withheld upstream fact: invalid search-state shape; upstream_observation={upstream_observation}"
        );
        return observation;
    };

    state.insert(
        "identity_context_required".into(),
        Value::Bool(context_required),
    );
    state.insert(
        "identity_context_verified".into(),
        Value::Bool(context_verified),
    );

    if reasons.is_empty() {
        state.insert("identity_supported".into(), Value::Bool(true));
        state.insert(
            "identity_reason".into(),
            Value::String(if context_required {
                "rank1_cross_source_agreement_with_trusted_context_query".into()
            } else {
                "rank1_cross_source_agreement".into()
            }),
        );
        return observation;
    }

    observation.facts.clear();
    state.insert(
        "upstream_outcome_kind".into(),
        Value::String("fact_resolved".into()),
    );
    state.insert(
        "outcome_kind".into(),
        Value::String("identity_insufficient".into()),
    );
    state.insert("identity_supported".into(), Value::Bool(false));
    state.insert("identity_reasons".into(), json!(reasons));
    if context_required && !context_verified {
        if let Some(query) = canonical_identity_context_query(case) {
            state.insert("suggested_query".into(), Value::String(query));
            state.insert(
                "suggested_action".into(),
                Value::String("follow_suggested_query".into()),
            );
        }
    } else {
        state.remove("suggested_query");
        state.insert("suggested_action".into(), Value::String("stop".into()));
    }
    observation.observation = format!(
        "Harness identity qualification withheld upstream fact: reasons={}; trusted context is a sufficiency constraint, not planner-owned evidence; if a canonical suggested_query is present the planner may explicitly follow it, otherwise stop; upstream_observation={upstream_observation}",
        reasons.join(",")
    );
    observation
}
'''
    text = replace_fn(text, "qualify_identity", qualify)

    old_invoke = '''    Ok(qualify_identity(\n        ToolObservation {\n            facts,\n            observation,\n            search_state,\n        },\n        case,\n    ))\n'''
    new_invoke = '''    Ok(qualify_identity_for_query(\n        ToolObservation {\n            facts,\n            observation,\n            search_state,\n        },\n        case,\n        query,\n    ))\n'''
    text = one(text, old_invoke, new_invoke, "invoke-query-aware-qualification")

    old_context_available = '    let context_search_available = case.identity_context.is_some();\n'
    if text.count(old_context_available) != 2:
        raise SystemExit(
            f"context-search-availability: expected 2 matches, found {text.count(old_context_available)}"
        )
    text = text.replace(
        old_context_available,
        '    let context_search_available = trusted_identity_context_search_remaining(case, tried_queries);\n',
    )

    text = one(
        text,
        '    prior_frozen_holdout_reused: bool,\n    identity_policy: &\'static str,\n',
        '    prior_frozen_holdout_reused: bool,\n    historical_frozen_holdouts_reused: bool,\n    identity_policy: &\'static str,\n',
        "report-historical-holdout-isolation-field",
    )
    text = one(
        text,
        '    recovered_after_invalid_action: bool,\n    traces: Vec<RoundTrace>,\n',
        '    recovered_after_invalid_action: bool,\n    context_unverified_fact_admissions: usize,\n    traces: Vec<RoundTrace>,\n',
        "case-report-context-invariant",
    )
    text = one(
        text,
        '    false_acceptances: usize,\n    false_abstentions: usize,\n',
        '    false_acceptances: usize,\n    false_abstentions: usize,\n    semantic_false_decisions: usize,\n    context_unverified_fact_admissions: usize,\n',
        "aggregate-semantic-fields",
    )
    text = one(
        text,
        '    let mut follow_suggested_query_actions = 0usize;\n',
        '    let mut follow_suggested_query_actions = 0usize;\n    let mut context_unverified_fact_admissions = 0usize;\n',
        "run-case-context-counter",
    )
    text = one(
        text,
        '        if let Some(value) = observation.facts.get(case.fact_key).cloned() {\n            observed_value = Some(value.clone());\n',
        '        if let Some(value) = observation.facts.get(case.fact_key).cloned() {\n            if case.identity_context.is_some()\n                && observation\n                    .search_state\n                    .get("identity_context_verified")\n                    .and_then(Value::as_bool)\n                    != Some(true)\n            {\n                context_unverified_fact_admissions += 1;\n            }\n            observed_value = Some(value.clone());\n',
        "terminal-context-invariant",
    )
    text = one(
        text,
        '        recovered_after_invalid_action: invalid_action_observations > 0 && passed,\n        traces,\n',
        '        recovered_after_invalid_action: invalid_action_observations > 0 && passed,\n        context_unverified_fact_admissions,\n        traces,\n',
        "case-report-context-value",
    )

    semantic_helper = r'''fn is_semantic_false_decision(
    expected: ExpectedOutcome,
    final_outcome: ExpectedOutcome,
) -> bool {
    final_outcome != ExpectedOutcome::Unknown && final_outcome != expected
}
'''
    text = insert_after_fn(text, "is_false_acceptance", semantic_helper)
    text = one(
        text,
        '    let operational_unresolved = samples\n',
        '    let semantic_false_decisions = samples\n        .iter()\n        .filter(|sample| is_semantic_false_decision(sample.expected, sample.final_outcome))\n        .count();\n    let context_unverified_fact_admissions = samples\n        .iter()\n        .map(|sample| sample.context_unverified_fact_admissions)\n        .sum();\n    let operational_unresolved = samples\n',
        "aggregate-semantic-calculation",
    )
    text = one(
        text,
        '        false_acceptances,\n        false_abstentions,\n        operational_unresolved,\n',
        '        false_acceptances,\n        false_abstentions,\n        semantic_false_decisions,\n        context_unverified_fact_admissions,\n        operational_unresolved,\n',
        "aggregate-semantic-values",
    )

    test_insert = r'''    #[test]
    fn bare_rank1_fact_with_trusted_context_is_not_admitted() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = json!(1);
        let qualified = qualify_identity_for_query(upstream, case(Some("Region")), "Alpha");
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["outcome_kind"], "identity_insufficient");
        assert_eq!(qualified.search_state["identity_context_required"], true);
        assert_eq!(qualified.search_state["identity_context_verified"], false);
        assert_eq!(qualified.search_state["suggested_query"], "Alpha, Region");
        assert!(qualified.search_state["identity_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "trusted_identity_context_not_verified_by_query"));
    }

    #[test]
    fn rank1_fact_after_canonical_context_query_is_admitted() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = json!(1);
        let qualified = qualify_identity_for_query(
            upstream,
            case(Some("Region")),
            "Alpha, Region",
        );
        assert_eq!(qualified.facts.get("alpha.fact").map(String::as_str), Some("Q9"));
        assert_eq!(qualified.search_state["identity_supported"], true);
        assert_eq!(qualified.search_state["identity_context_verified"], true);
        assert_eq!(
            qualified.search_state["identity_reason"],
            "rank1_cross_source_agreement_with_trusted_context_query"
        );
    }

    #[test]
    fn ambiguous_bare_surface_with_trusted_context_gets_canonical_followup() {
        let qualified = qualify_identity_for_query(
            obs(json!({"outcome_kind":"ambiguous","external_requests":2})),
            case(Some("Region")),
            "Alpha",
        );
        assert_eq!(qualified.search_state["outcome_kind"], "ambiguous");
        assert_eq!(qualified.search_state["suggested_query"], "Alpha, Region");
        assert_eq!(qualified.search_state["suggested_action"], "follow_suggested_query");
        assert_eq!(qualified.search_state["identity_context_verified"], false);
    }

    #[test]
    fn ambiguity_after_context_query_does_not_loop_the_same_context() {
        let qualified = qualify_identity_for_query(
            obs(json!({"outcome_kind":"ambiguous","external_requests":2,"suggested_action":"stop"})),
            case(Some("Region")),
            "Alpha, Region",
        );
        assert!(qualified.search_state.get("suggested_query").is_none());
        let tried = BTreeSet::from([
            normalize_query("Alpha"),
            normalize_query("Alpha, Region"),
        ]);
        assert!(!trusted_identity_context_search_remaining(case(Some("Region")), &tried));
    }

    #[test]
    fn semantic_false_decision_is_symmetric_for_terminal_wrong_answers() {
        assert!(is_semantic_false_decision(
            ExpectedOutcome::Accept,
            ExpectedOutcome::Reject
        ));
        assert!(is_semantic_false_decision(
            ExpectedOutcome::Reject,
            ExpectedOutcome::Accept
        ));
        assert!(is_semantic_false_decision(
            ExpectedOutcome::Unknown,
            ExpectedOutcome::Accept
        ));
        assert!(is_semantic_false_decision(
            ExpectedOutcome::Unknown,
            ExpectedOutcome::Reject
        ));
        assert!(!is_semantic_false_decision(
            ExpectedOutcome::Accept,
            ExpectedOutcome::Unknown
        ));
        assert!(!is_semantic_false_decision(
            ExpectedOutcome::Accept,
            ExpectedOutcome::Accept
        ));
    }

'''
    marker = '    #[test]\n    fn repair_limit_exhaustion_remains_safe_unknown() {\n'
    text = one(text, marker, test_insert + marker, "v9-contract-tests")

    text = one(
        text,
        '        prior_frozen_holdout_reused: false,\n        identity_policy:',
        '        prior_frozen_holdout_reused: false,\n        historical_frozen_holdouts_reused: false,\n        identity_policy:',
        "report-historical-holdout-isolation-value",
    )
    text = one(
        text,
        '        identity_policy: "candidate-set membership is plausibility only; fact admission requires non-disambiguation Wikipedia top identity == resolved entity and cross-source corroboration rank 1",\n',
        '        identity_policy: "candidate-set membership is plausibility only; fact admission requires non-disambiguation Wikipedia top identity == resolved entity and cross-source corroboration rank 1; when trusted identity context exists, the admitted fact must additionally come from the Harness-bounded context-conditioned query",\n',
        "reported-identity-policy",
    )
    text = one(
        text,
        '        planner_action_policy: "planner is untrusted; Harness keeps rank1 identity sufficiency unchanged, emits one canonical title-style surface-comma-context query only from separately supplied trusted identity context after identity insufficiency, and executes it only when the planner explicitly selects follow_suggested_query; free-form search is unavailable while that suggestion exists; invalid actions preserve the suggestion and are blocked before external request",\n',
        '        planner_action_policy: "planner is untrusted; Harness owns trusted-context compatibility and emits one canonical title-style surface-comma-context query whenever a bare observation with trusted context cannot yet satisfy identity; the planner may explicitly follow that exact suggestion or stop, never decide identity sufficiency, and invalid actions remain blocked before external request",\n',
        "reported-planner-policy",
    )
    text = one(
        text,
        '        evaluation_policy: "fresh development cases only; prior #193 holdout is not executed or used for tuning; expected unknown treats any fact-level Accept/Reject decision as a semantic false acceptance; a new holdout will be frozen only after this candidate stabilizes",\n',
        '        evaluation_policy: "fresh Issue #196 development split only; historical #193 and #195 frozen holdouts are not executed, replayed, or used for tuning; terminal wrong answers are counted symmetrically as semantic_false_decisions; context-unverified fact admission is a hard safety violation",\n',
        "reported-evaluation-policy",
    )

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
