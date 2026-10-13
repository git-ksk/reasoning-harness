//! Root-authored development regressions; not an independently authored holdout.
use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceLocalRelationScope as Relation,
    EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition as Disposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v17, materialize_evidence_relevance_v29,
    materialize_evidence_relevance_v30,
};
use serde_json::{Value, json};

fn inputs(
    case: &Value,
) -> (
    EvidenceRelevanceTargetPolicy,
    EvidenceRelevanceCandidate,
    EvidenceRelevanceBindingProposal,
    EvidenceLocalQualificationV6,
) {
    let entity = case["entity"].as_str().unwrap_or("Silver Lens");
    let lambda = case["policy"] == "lambda";
    let policy = serde_json::from_value(json!({
        "policy_id":"launch-context-markers-520-521", "target_id":"target",
        "target_question":if lambda {"Provisioned Concurrencyは何を準備するか？".to_owned()} else {format!("When was {entity} launched?")},
        "entity":if lambda {json!({"canonical_id":"aws.lambda.provisioned-concurrency", "canonical_name":"AWS Lambda Provisioned Concurrency", "aliases":["Provisioned Concurrency"]})} else {json!({"canonical_id":"product.launch", "canonical_name":entity, "aliases":case.get("aliases").cloned().unwrap_or(json!([]))})},
        "relation":if lambda {"general"} else {"change_or_launch"},
        "identity_requirement":"require_harness_anchor",
        "assessment_budget":{"max_model_attempts":2,"max_tokens":192,"max_elapsed_ms":90000}
    })).unwrap();
    let candidate = serde_json::from_value(json!({
        "evidence_id":"e", "source_id":"s", "signals":case["signals"]
    }))
    .unwrap();
    let proposal =
        serde_json::from_value(json!({"target_binding":"exact","relation_binding":"exact"}))
            .unwrap();
    let raw = serde_json::from_value(json!({"identity_scope":"exact_target","relation_scope":"requested_relation","scope_risk":"none"})).unwrap();
    (policy, candidate, proposal, raw)
}

fn check(cases: Vec<Value>) {
    let mut failures = Vec::new();
    for case in cases {
        let (policy, candidate, proposal, raw) = inputs(&case);
        let relation = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap()
        .relation_scope;
        let disposition =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap()
                .disposition;
        let expected = case["expected_relevant"].as_bool().unwrap();
        if (relation == Relation::RequestedRelation) != expected
            || (disposition == Disposition::Relevant) != expected
        {
            failures.push(format!(
                "{}: expected={expected}, relation={relation:?}, disposition={disposition:?}",
                case["id"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn original_audit_56_cases_preserve_positives_and_withhold_40_regressions() {
    check(
        serde_json::from_str(include_str!(
            "../../../evaluation/engine-0.7.1-rereview-20261013/cases.json"
        ))
        .unwrap(),
    );
}

#[test]
fn governing_context_and_original_marker_boundaries() {
    let mut cases = Vec::new();
    for context in [
        "This is a fictional example.",
        "The following statement is false.",
        "This launch claim has been retracted.",
        "In this simulation.",
        "The launch announcement has been withdrawn.",
        "A fictional example.",
        "The statement is not true.",
    ] {
        for kind in [
            "heading",
            "source_title",
            "fact",
            "excerpt",
            "structured_metadata",
        ] {
            cases.push(json!({"id":format!("{kind}:{context}"),"signals":[{"kind":kind,"text":context},{"kind":"excerpt","text":"Silver Lens was launched in 2026."}],"expected_relevant":false}));
        }
    }
    for (text, expected) in [
        (
            "Silver Lens was launched in 2026. This statement is false.",
            false,
        ),
        (
            "Silver Lens was launched in 2026. The preceding claim was retracted.",
            false,
        ),
        (
            "The next statement is fictional. Silver Lens was launched in 2026.",
            false,
        ),
        (
            "Silver Lens was launched in 2026. The Silver Lens launch announcement was withdrawn.",
            false,
        ),
        (
            "Silver Lens was launched in 2026. Maple Queue's launch claim is false.",
            true,
        ),
        (
            "Maple Queue's launch claim was retracted. Silver Lens was launched in 2026.",
            true,
        ),
        (
            "Maple Queue was launched in 2026. This statement is false. Silver Lens was launched in 2026.",
            true,
        ),
        (
            "Silver Lens was launched in 2026. Maple Queue was launched in 2026. This statement is false.",
            true,
        ),
        (
            "Silver Lens was launched in 2026. This model supports false alarms.",
            true,
        ),
        (
            "Silver Lens was launched in 2026. It supports streaming.",
            true,
        ),
        (
            "~~Silver Lens was launched in 2026. Details follow.~~",
            false,
        ),
        (
            "~~Fictional draft. Silver Lens was launched in 2026.~~",
            false,
        ),
        (
            "~~Maple Queue was launched in 2026.~~ Silver Lens was launched in 2026.",
            true,
        ),
        (
            "Silver Lens was launched in 2026. ~~Maple Queue was launched in 2026.~~",
            true,
        ),
        (
            "❌ Maple Queue was launched in 2026. Silver Lens was launched in 2026.",
            true,
        ),
        (
            "Silver Lens was launched in 2026. ❌ Maple Queue was launched in 2026.",
            true,
        ),
        ("Silver Lens was launched in 2026. ❌", false),
        ("❌\nSilver Lens was launched in 2026.", false),
        ("Silver Lens w\u{0336}as launched in 2026.", false),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    for kind in ["source_title", "heading", "fact", "structured_metadata"] {
        cases.push(json!({"id":format!("other:{kind}"),"signals":[{"kind":kind,"text":"Maple Queue's launch claim is false."},{"kind":"excerpt","text":"Silver Lens was launched in 2026."}],"expected_relevant":true}));
    }
    for kind in ["canonical_url", "navigation_or_footer"] {
        cases.push(json!({"id":format!("documentary:{kind}"),"signals":[{"kind":kind,"text":"This is a fictional example. ❌"},{"kind":"excerpt","text":"Silver Lens was launched in 2026."}],"expected_relevant":true}));
    }
    cases.push(json!({"id":"alias-denial","entity":"Cedar 2.4.1","aliases":["Cedar LTS"],"signals":[{"kind":"heading","text":"Cedar LTS launch claim is false."},{"kind":"excerpt","text":"Cedar 2.4.1 was launched in 2026."}],"expected_relevant":false}));
    cases.push(json!({"id":"dotted-positive","entity":"GLM 5.3","signals":[{"kind":"heading","text":"Release notes"},{"kind":"excerpt","text":"Amazon Bedrock has made the latest version of GLM 5.3 from Z.ai generally available. This model is a powerful option."}],"expected_relevant":true}));
    check(cases);
}

#[test]
fn explicit_other_target_context_and_neutral_false_alarm_description() {
    check([
        "Silver Lens was launched in 2026. The launch claim about Maple Queue is false.",
        "The launch claim for Maple Queue is false. Silver Lens was launched in 2026.",
        "Silver Lens was launched in 2026. This is an example of false alarms.",
    ].into_iter().map(|text| json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":true})).collect());
}

#[test]
fn historical_v29_dispositions_remain_unchanged_for_original_audit() {
    let rows: Vec<Value> =
        include_str!("../../../evaluation/engine-0.7.1-rereview-20261013/results.jsonl")
            .lines()
            .map(|s| serde_json::from_str(s).unwrap())
            .collect();
    for row in rows {
        let (policy, candidate, proposal, raw) = inputs(&row["input"]);
        let historical =
            materialize_evidence_relevance_v29(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        assert_eq!(
            serde_json::to_value(historical.disposition).unwrap(),
            row["v29"],
            "{}",
            row["id"]
        );
    }
}
