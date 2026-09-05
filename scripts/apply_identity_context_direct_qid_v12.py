#!/usr/bin/env python3
"""Issue #196 v12: bounded direct-QID corroboration for trusted context only.

This does not relax the bare-surface rank-1 rule. The direct path is eligible
only after the exact Harness-owned trusted-context coordinate has been executed,
and the Harness still requires deterministic context compatibility.
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
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v11";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v12";\n',
        "schema-v12",
    )

    text = one(
        text,
        '''    let corroboration_rank = observation\n        .search_state\n        .get("corroboration_rank")\n        .and_then(Value::as_u64);\n''',
        '''    let corroboration_rank = observation\n        .search_state\n        .get("corroboration_rank")\n        .and_then(Value::as_u64);\n    let corroboration_mode = observation\n        .search_state\n        .get("corroboration_mode")\n        .and_then(Value::as_str);\n    let direct_wikibase_verified = observation\n        .search_state\n        .get("direct_wikibase_verified")\n        .and_then(Value::as_bool)\n        .unwrap_or(false);\n''',
        "read-direct-evidence",
    )

    text = one(
        text,
        '''    if corroboration_rank != Some(1) {\n        reasons.push("cross_source_agreement_not_rank1");\n    }\n''',
        '''    let rank1_search_supported = corroboration_rank == Some(1);\n    let direct_context_supported = context_required\n        && context_query_grounded\n        && direct_wikibase_verified\n        && corroboration_mode == Some("wikipedia_wikibase_direct");\n    if !rank1_search_supported && !direct_context_supported {\n        reasons.push("cross_source_identity_evidence_insufficient");\n    }\n''',
        "context-direct-evidence-gate",
    )

    text = one(
        text,
        '''            Value::String(if context_required {\n                "rank1_cross_source_agreement_with_trusted_context_metadata".into()\n            } else {\n                "rank1_cross_source_agreement".into()\n            }),\n''',
        '''            Value::String(if direct_context_supported {\n                "direct_wikibase_verification_with_trusted_context_metadata".into()\n            } else if context_required {\n                "rank1_cross_source_agreement_with_trusted_context_metadata".into()\n            } else {\n                "rank1_cross_source_agreement".into()\n            }),\n''',
        "direct-success-reason",
    )

    text = one(
        text,
        '''fn invoke_tool(case: CaseSpec, query: &str) -> Result<ToolObservation, ToolFailure> {\n    let allow_title_retry = case.identity_context.is_some()\n        && grounded_identity_context_search(case, query);\n''',
        '''fn invoke_tool(case: CaseSpec, query: &str) -> Result<ToolObservation, ToolFailure> {\n    let allow_title_retry = case.identity_context.is_some()\n        && grounded_identity_context_search(case, query);\n    let allow_direct_wikibase_fallback = allow_title_retry;\n''',
        "direct-fallback-runtime-boundary",
    )
    text = one(
        text,
        '''                "fact_key": case.fact_key,\n                "allow_title_retry": allow_title_retry\n''',
        '''                "fact_key": case.fact_key,\n                "allow_title_retry": allow_title_retry,\n                "allow_direct_wikibase_fallback": allow_direct_wikibase_fallback\n''',
        "direct-fallback-tool-argument",
    )

    text = one(
        text,
        'v == "cross_source_agreement_not_rank1"',
        'v == "cross_source_identity_evidence_insufficient"',
        "rank2-contract-reason-v12",
    )

    test_marker = '''    #[test]\n    fn context_query_without_compatible_candidate_metadata_is_not_admitted() {\n'''
    tests = r'''    #[test]
    fn trusted_context_direct_wikibase_verification_can_admit_without_search_rank() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = Value::Null;
        upstream.search_state["corroboration_mode"] = json!("wikipedia_wikibase_direct");
        upstream.search_state["direct_wikibase_verified"] = json!(true);
        upstream.search_state["corroboration_entity_label"] = json!("Alpha");
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity in Region");
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
            "direct_wikibase_verification_with_trusted_context_metadata"
        );
    }

    #[test]
    fn direct_wikibase_verification_never_relaxes_bare_surface_rank_rule() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = Value::Null;
        upstream.search_state["corroboration_mode"] = json!("wikipedia_wikibase_direct");
        upstream.search_state["direct_wikibase_verified"] = json!(true);
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity in Region");
        let qualified = qualify_identity_for_query(upstream, case(None), "Alpha");
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["outcome_kind"], "identity_insufficient");
        assert!(qualified.search_state["identity_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "cross_source_identity_evidence_insufficient"));
    }

    #[test]
    fn direct_wikibase_verification_without_context_metadata_remains_unadmitted() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = Value::Null;
        upstream.search_state["corroboration_mode"] = json!("wikipedia_wikibase_direct");
        upstream.search_state["direct_wikibase_verified"] = json!(true);
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity elsewhere");
        let qualified = qualify_identity_for_query(
            upstream,
            case(Some("Region")),
            "Alpha, Region",
        );
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["identity_context_metadata_compatible"], false);
    }

'''
    text = one(text, test_marker, tests + test_marker, "v12-contract-tests")

    text = one(
        text,
        '        identity_policy: "candidate-set membership is plausibility only; no-context fact admission keeps the rank1 cross-source gate; trusted-context fact admission additionally requires the Harness-bounded context query and deterministic context-token compatibility with the corroborating Wikidata label/description or Wikipedia top title",\n',
        '        identity_policy: "candidate-set membership is plausibility only; no-context fact admission keeps the rank1 cross-source gate; trusted-context fact admission requires the Harness-bounded context query plus deterministic context-token compatibility, and may use either rank1 search corroboration or a bounded direct fetch of the exact non-disambiguation Wikipedia Wikibase QID; direct-QID evidence never relaxes bare-surface admission",\n',
        "reported-v12-identity-policy",
    )

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
