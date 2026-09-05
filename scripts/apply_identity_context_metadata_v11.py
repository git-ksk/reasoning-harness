#!/usr/bin/env python3
"""Issue #196 v11: bind trusted context to corroborating candidate metadata.

The adapter may expose candidate metadata; only the Harness decides whether it
is sufficient. Adapter-provided suggested queries are never planner actions.
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
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v10";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v11";\n',
        "schema-v11",
    )

    strict_suggestion = r'''fn suggested_query(state: &Value) -> Option<String> {
    if state
        .get("suggested_query_origin")
        .and_then(Value::as_str)
        != Some("harness_trusted_identity_context")
    {
        return None;
    }
    state
        .get("suggested_query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}
'''
    text = replace_fn(text, "suggested_query", strict_suggestion)

    metadata_helper = r'''fn trusted_identity_context_metadata_compatible(case: CaseSpec, state: &Value) -> bool {
    let Some(context) = case.identity_context else {
        return false;
    };
    let required = query_terms(context);
    if required.is_empty() {
        return false;
    }

    let mut observed = BTreeSet::new();
    for key in ["corroboration_entity_label", "corroboration_entity_description"] {
        if let Some(value) = state.get(key).and_then(Value::as_str) {
            observed.extend(query_terms(value));
        }
    }
    if let Some(title) = state
        .get("wikipedia_candidates")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(|value| value.get("title"))
        .and_then(Value::as_str)
    {
        observed.extend(query_terms(title));
    }
    required.is_subset(&observed)
}
'''
    text = insert_after_fn(text, "trusted_identity_context_search_remaining", metadata_helper)

    qualify = r'''fn qualify_identity_for_query(
    mut observation: ToolObservation,
    case: CaseSpec,
    query: &str,
) -> ToolObservation {
    let context_required = case.identity_context.is_some();
    let context_query_grounded = context_required && grounded_identity_context_search(case, query);
    let context_metadata_compatible =
        context_required && trusted_identity_context_metadata_compatible(case, &observation.search_state);
    let context_verified = context_query_grounded && context_metadata_compatible;
    let outcome_kind = state_kind(&observation.search_state).map(ToOwned::to_owned);

    if outcome_kind.as_deref() != Some("fact_resolved") {
        if let Some(state) = observation.search_state.as_object_mut() {
            state.insert(
                "identity_context_required".into(),
                Value::Bool(context_required),
            );
            state.insert(
                "identity_context_query_grounded".into(),
                Value::Bool(context_query_grounded),
            );
            state.insert(
                "identity_context_metadata_compatible".into(),
                Value::Bool(context_metadata_compatible),
            );
            state.insert(
                "identity_context_verified".into(),
                Value::Bool(context_verified),
            );

            if context_required
                && !context_query_grounded
                && outcome_kind.as_deref() != Some("invalid_query")
            {
                if let Some(query) = canonical_identity_context_query(case) {
                    state.insert("suggested_query".into(), Value::String(query));
                    state.insert(
                        "suggested_query_origin".into(),
                        Value::String("harness_trusted_identity_context".into()),
                    );
                    state.insert(
                        "suggested_action".into(),
                        Value::String("follow_suggested_query".into()),
                    );
                }
                let upstream_observation = observation.observation.clone();
                observation.observation = format!(
                    "Harness requires trusted identity context before identity sufficiency; canonical follow-up is available only from Harness-owned context; upstream_observation={upstream_observation}"
                );
            } else if context_query_grounded {
                state.remove("suggested_query");
                state.remove("suggested_query_origin");
                state.insert("suggested_action".into(), Value::String("stop".into()));
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
    if context_required && !context_query_grounded {
        reasons.push("trusted_identity_context_not_verified_by_query");
    } else if context_required && !context_metadata_compatible {
        reasons.push("trusted_identity_context_not_supported_by_candidate_metadata");
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
            "identity_context_query_grounded": context_query_grounded,
            "identity_context_metadata_compatible": context_metadata_compatible,
            "identity_context_verified": false,
            "suggested_action": "stop"
        });
        if context_required && !context_query_grounded {
            if let Some(query) = canonical_identity_context_query(case) {
                if let Some(state) = fallback.as_object_mut() {
                    state.insert("suggested_query".into(), Value::String(query));
                    state.insert(
                        "suggested_query_origin".into(),
                        Value::String("harness_trusted_identity_context".into()),
                    );
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
        "identity_context_query_grounded".into(),
        Value::Bool(context_query_grounded),
    );
    state.insert(
        "identity_context_metadata_compatible".into(),
        Value::Bool(context_metadata_compatible),
    );
    state.insert(
        "identity_context_verified".into(),
        Value::Bool(context_verified),
    );

    if reasons.is_empty() {
        state.remove("suggested_query");
        state.remove("suggested_query_origin");
        state.insert("identity_supported".into(), Value::Bool(true));
        state.insert(
            "identity_reason".into(),
            Value::String(if context_required {
                "rank1_cross_source_agreement_with_trusted_context_metadata".into()
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
    if context_required && !context_query_grounded {
        if let Some(query) = canonical_identity_context_query(case) {
            state.insert("suggested_query".into(), Value::String(query));
            state.insert(
                "suggested_query_origin".into(),
                Value::String("harness_trusted_identity_context".into()),
            );
            state.insert(
                "suggested_action".into(),
                Value::String("follow_suggested_query".into()),
            );
        }
    } else {
        state.remove("suggested_query");
        state.remove("suggested_query_origin");
        state.insert("suggested_action".into(), Value::String("stop".into()));
    }
    observation.observation = format!(
        "Harness identity qualification withheld upstream fact: reasons={}; trusted context requires both a bounded context query and deterministic compatibility with corroborating candidate metadata; upstream_observation={upstream_observation}",
        reasons.join(",")
    );
    observation
}
'''
    text = replace_fn(text, "qualify_identity_for_query", qualify)

    text = one(
        text,
        'fn invoke_tool(case: CaseSpec, query: &str) -> Result<ToolObservation, ToolFailure> {\n    let request = json!({\n',
        'fn invoke_tool(case: CaseSpec, query: &str) -> Result<ToolObservation, ToolFailure> {\n    let allow_title_retry = case.identity_context.is_some()\n        && grounded_identity_context_search(case, query);\n    let request = json!({\n',
        "context-bounded-title-retry-variable",
    )
    text = one(
        text,
        '                "allow_title_retry": false\n',
        '                "allow_title_retry": allow_title_retry\n',
        "context-bounded-title-retry-argument",
    )

    text = one(
        text,
        '    if let Some(value) = state.get("suggested_query").and_then(Value::as_str) {\n        if !value.trim().is_empty() {\n            out.insert(format!("suggested_query:{value}"));\n        }\n    }\n',
        '    if let Some(value) = suggested_query(state) {\n        out.insert(format!("suggested_query:{value}"));\n    }\n',
        "trusted-suggestion-only-progress",
    )

    trusted_test_state = '{"suggested_query":"Alpha, Region"}'
    trusted_test_state_with_origin = '{"suggested_query":"Alpha, Region","suggested_query_origin":"harness_trusted_identity_context"}'
    if text.count(trusted_test_state) != 2:
        raise SystemExit(
            f"trusted-suggestion-test-fixtures: expected 2 matches, found {text.count(trusted_test_state)}"
        )
    text = text.replace(trusted_test_state, trusted_test_state_with_origin)

    old_test = '''    fn rank1_fact_after_canonical_context_query_is_admitted() {\n        let mut upstream = rank2_fact();\n        upstream.search_state["corroboration_rank"] = json!(1);\n        let qualified = qualify_identity_for_query(\n            upstream,\n            case(Some("Region")),\n            "Alpha, Region",\n        );\n'''
    new_test = '''    fn rank1_fact_after_canonical_context_query_is_admitted() {\n        let mut upstream = rank2_fact();\n        upstream.search_state["corroboration_rank"] = json!(1);\n        upstream.search_state["corroboration_entity_label"] = json!("Alpha");\n        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity in Region");\n        let qualified = qualify_identity_for_query(\n            upstream,\n            case(Some("Region")),\n            "Alpha, Region",\n        );\n'''
    text = one(text, old_test, new_test, "metadata-backed-context-test")
    text = one(
        text,
        '            "rank1_cross_source_agreement_with_trusted_context_query"\n',
        '            "rank1_cross_source_agreement_with_trusted_context_metadata"\n',
        "metadata-backed-reason-test",
    )

    extra_tests = r'''    #[test]
    fn context_query_without_compatible_candidate_metadata_is_not_admitted() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = json!(1);
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity elsewhere");
        let qualified = qualify_identity_for_query(
            upstream,
            case(Some("Region")),
            "Alpha, Region",
        );
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["identity_context_query_grounded"], true);
        assert_eq!(qualified.search_state["identity_context_metadata_compatible"], false);
        assert_eq!(qualified.search_state["identity_context_verified"], false);
        assert!(qualified.search_state.get("suggested_query").is_none());
        assert_eq!(qualified.search_state["suggested_action"], "stop");
    }

    #[test]
    fn adapter_candidate_suggestion_is_not_a_planner_action() {
        let state = json!({
            "outcome_kind":"entity_disagreement",
            "suggested_query":"Alpha, Candidate Region"
        });
        assert!(suggested_query(&state).is_none());
        let action = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let error = planner_action_resolution(
            case(Some("Region")),
            &action,
            &obs(state),
            &BTreeSet::from([normalize_query("Alpha, Region")]),
        ).unwrap_err();
        assert_eq!(error, "missing_suggested_query");
    }

'''
    marker = '    #[test]\n    fn repair_limit_exhaustion_remains_safe_unknown() {\n'
    text = one(text, marker, extra_tests + marker, "v11-contract-tests")

    text = one(
        text,
        '        identity_policy: "candidate-set membership is plausibility only; fact admission requires non-disambiguation Wikipedia top identity == resolved entity and cross-source corroboration rank 1; when trusted identity context exists, the admitted fact must additionally come from the Harness-bounded context-conditioned query",\n',
        '        identity_policy: "candidate-set membership is plausibility only; no-context fact admission keeps the rank1 cross-source gate; trusted-context fact admission additionally requires the Harness-bounded context query and deterministic context-token compatibility with the corroborating Wikidata label/description or Wikipedia top title",\n',
        "reported-v11-identity-policy",
    )
    text = one(
        text,
        '        planner_action_policy: "planner is untrusted; Harness owns trusted-context compatibility and emits one canonical title-style surface-comma-context query whenever a bare observation with trusted context cannot yet satisfy identity; the planner may explicitly follow that exact suggestion or stop, never decide identity sufficiency, and invalid actions remain blocked before external request",\n',
        '        planner_action_policy: "planner is untrusted; only suggestions marked as Harness-owned trusted-context actions are executable; adapter/candidate suggested queries are observations only; the planner may follow the exact Harness suggestion or stop and never decides identity sufficiency",\n',
        "reported-v11-planner-policy",
    )

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
