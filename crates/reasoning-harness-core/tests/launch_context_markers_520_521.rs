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
fn post_v1_development_controls_preserve_factual_signal_floor() {
    // v1's original RED observation/expectations remain immutable. Once seen,
    // these are development controls, with P04's heading-only expectation
    // corrected here explicitly, never rescored as independent acceptance.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v1/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case| json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive" && case["case_id"]!="P04"
    })).collect());
    check([
        "If Silver Lens entered general availability, customers could sign up.",
        "Silver Lens has not reached GA.",
        "Silver Lens reached GA according to an unverified rumor.",
        "Silver Lens reached GA?",
        "Maple Queue reached GA while Silver Lens remains in preview.",
        "Documentation for Silver Lens entered general availability.",
        "Silver Lens reached GA metrics for another product.",
        "Silver Lens entered general availability equipment markets.",
    ].into_iter().map(|text|json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":false})).collect());
}

#[test]
fn post_v2_development_controls_preserve_statement_scope() {
    // v2's RED first observation is retained. Replaying seen cases is only
    // development regression coverage, never fresh independent acceptance.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v2/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case|json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive"
    })).collect());
    for heading in [
        "❌ Launch announcement",
        "~~Release announcement~~",
        "Retraction notice",
    ] {
        check(vec![
            json!({"id":heading,"signals":[{"kind":"heading","text":heading},{"kind":"excerpt","text":"Silver Lens was launched in 2026."}],"expected_relevant":false}),
        ]);
    }
    check([
        "Silver Lens might have been launched according to the release record.",
        "Silver Lens was launched according to an unverified release record.",
        "Documentation for Silver Lens was launched according to the release record.",
        "Silver Lens was launched? It is available according to the release record.",
    ].into_iter().map(|text|json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":false})).collect());
}

#[test]
fn generic_disclaimer_wrappers_and_passage_references() {
    for context in [
        "Erratum: this report is false.",
        "Note: this announcement was withdrawn.",
        "What comes next is fictional.",
        "Everything below is hypothetical.",
        "This passage is fictional.",
        "The author stated that this description is false.",
        "This text is fictional.",
        "The publisher retracted it.",
        "Correction note: the claim above was retracted.",
    ] {
        check(vec![
            json!({"id":context,"signals":[{"kind":"excerpt","text":format!("{context} Silver Lens was launched in 2026.")}],"expected_relevant":false}),
        ]);
    }
    for context in [
        "This model supports fictional characters.",
        "Note: this model produces false alarms.",
        "Update: the release includes an editor.",
        "~~Silver Lens offers offline editing.~~",
    ] {
        check(vec![
            json!({"id":context,"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched in 2026. {context}")}],"expected_relevant":true}),
        ]);
    }
}

#[test]
fn post_v3_development_controls_keep_unbound_pronouns_withheld() {
    // P323 is intentionally corrected only in this development replay. The
    // original v3 expectation and RED observation are never rewritten.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v3/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case|json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive" && case["case_id"]!="P323"
    })).collect());
    for context in [
        "This dispatch is hypothetical.",
        "That bulletin is fictitious.",
        "The publisher said that it was fictional.",
    ] {
        check(vec![
            json!({"id":context,"signals":[{"kind":"excerpt","text":format!("{context} Silver Lens was launched in 2026.")}],"expected_relevant":false}),
        ]);
    }
}

#[test]
fn post_v4_development_controls_preserve_direct_objects_and_plural_context() {
    // Seen v4 inputs are development controls. Its original RED first result
    // and unchanged frozen labels remain separate immutable evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v4/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case|json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive"
    })).collect());
    for text in [
        "If Maple Instruments launched Silver Lens on 26 May 2026.",
        "Maple Instruments reportedly launched Silver Lens on 26 May 2026.",
        "Maple Instruments launched documentation for Silver Lens on 26 May 2026.",
        "Maple Instruments launched Silver Lens documentation on 26 May 2026.",
        "Maple Instruments launched Silver Lens's portal on 26 May 2026.",
        "Maple Instruments introduced Silver Lens according to an unverified rumor.",
        "No Company launched Silver Lens in 2026.",
        "Nobody launched Silver Lens in 2026.",
        "Who launched Silver Lens in 2026?",
        "The statement that Silver Lens uses magnesium and was launched is false.",
        "In the unverified chronology, Silver Lens was introduced on 26 May 2026.",
        "These entries are fictional. Silver Lens was launched on 26 May 2026.",
        "Those records were withdrawn. Silver Lens was launched on 26 May 2026.",
    ] {
        check(vec![
            json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":false}),
        ]);
    }
}

#[test]
fn post_v5_development_controls_bind_feature_denials_to_their_antecedent() {
    // Preserve v5's original RED result. These seen cases only guard repairs.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v5/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case|json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive"
    })).collect());
    for text in [
        "Silver Lens supports face unlock. This report is fictional. Silver Lens went live in 2026.",
        "Silver Lens was launched in 2026. That feature claim is false.",
        "That feature claim is false. Silver Lens went live in 2026.",
        "The following statement is false. Silver Lens went live in 2026.",
    ] {
        check(vec![
            json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":false}),
        ]);
    }
    for text in [
        "Silver Lens supports face unlock. That statement is false. Silver Lens went live in 2026.",
        "Silver Lens went live in 2026. The following claim is false. Silver Lens supports face unlock.",
    ] {
        check(vec![
            json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":true}),
        ]);
    }
}

#[test]
fn post_v6_development_controls_and_generic_documentary_disclaimers() {
    // v6's first RED score stays immutable; this is development regression.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v6/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case|json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive"
    })).collect());
    for (kind, context) in [
        ("heading", "A collection of fictional records"),
        ("source_title", "Historical examples are fabricated"),
        ("fact", "All entries are hypothetical."),
    ] {
        check(vec![
            json!({"id":context,"signals":[{"kind":kind,"text":context},{"kind":"excerpt","text":"Silver Lens was launched in 2026."}],"expected_relevant":false}),
        ]);
    }
    for kind in ["heading", "source_title", "excerpt", "fact"] {
        check(vec![
            json!({"id":kind,"signals":[{"kind":kind,"text":"Correction: Maple Queue's launch claim is false."},{"kind":"excerpt","text":"Silver Lens was launched in 2026."}],"expected_relevant":true}),
        ]);
    }
    check(vec![
        json!({"id":"property-is-launch","signals":[{"kind":"excerpt","text":"Silver Lens has a fictional launch date. Silver Lens was launched in 2026."}],"expected_relevant":false}),
        json!({"id":"possession-is-claim","signals":[{"kind":"excerpt","text":"Silver Lens has a false claim. Silver Lens was launched in 2026."}],"expected_relevant":false}),
    ]);
}

#[test]
fn post_v7_development_controls_keep_foreign_frames_local_and_global_colons_denied() {
    // v7's original RED first observation is preserved without rescoring.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v7/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case|json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive"
    })).collect());
    for prefix in [
        "Everything below is fictional",
        "This report is fictional",
        "Note: all entries are hypothetical",
        "The entire passage is fabricated",
    ] {
        check(vec![
            json!({"id":prefix,"signals":[{"kind":"heading","text":format!("{prefix}: Maple Queue was launched in 2026.")},{"kind":"fact","text":"Silver Lens was launched in 2026."}],"expected_relevant":false}),
        ]);
    }
    for predicate in [
        "launched",
        "was launched",
        "has been launched",
        "was introduced",
        "has been introduced",
        "rolled out",
        "was rolled out",
        "has rolled out",
        "went live",
        "is generally available",
        "is now generally available",
        "became generally available",
        "entered general availability",
        "reached GA",
    ] {
        check(vec![
            json!({"id":predicate,"signals":[{"kind":"heading","text":format!("Fictional release account: Maple Queue {predicate} in 2026.")},{"kind":"fact","text":"Silver Lens was launched in 2026."}],"expected_relevant":true}),
        ]);
    }
}

#[test]
fn post_v8_development_controls_preserve_collective_denials_and_actor_adjuncts() {
    // v8's original RED first result is immutable; seen cases are development.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v8/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case|json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive"
    })).collect());
    for reference in [
        "These statements were retracted",
        "This report is false",
        "All claims are fictional",
        "Both were retracted",
        "All were fictional",
    ] {
        check(vec![
            json!({"id":reference,"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched in 2026. Maple Queue was introduced in 2026. {reference}.")}],"expected_relevant":false}),
        ]);
    }
    for prefix in [
        "On 2 May 2026,",
        "As of May 2026,",
        "In the published chronology,",
    ] {
        check(vec![
            json!({"id":prefix,"signals":[{"kind":"fact","text":format!("{prefix} Maple Instruments launched Silver Lens.")}],"expected_relevant":true}),
        ]);
        check(vec![
            json!({"id":format!("negated:{prefix}"),"signals":[{"kind":"fact","text":format!("{prefix} No Company launched Silver Lens.")}],"expected_relevant":false}),
        ]);
    }
    check(vec![
        json!({"id":"invented-actor-adjunct","signals":[{"kind":"fact","text":"In an unverified chronology, Maple Instruments launched Silver Lens."}],"expected_relevant":false}),
    ]);
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
