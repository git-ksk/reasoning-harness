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
fn post_v9_development_controls_withhold_unresolved_adverse_references() {
    // v9's GREEN first observation remains attached to its original candidate;
    // the broader root scope checks below require a new fixed-source holdout.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../fixtures/launch-520-521-independent-v9/corpus.json"
    ))
    .unwrap();
    check(corpus["cases"].as_array().unwrap().iter().map(|case|json!({
        "id":case["case_id"],"entity":case["policy"]["entity"]["canonical_name"],
        "aliases":case["policy"]["entity"]["aliases"],"signals":case["candidate"]["signals"],
        "expected_relevant":case["label"]=="positive"
    })).collect());
    for denial in [
        "Both of them were retracted.",
        "They were both retracted.",
        "The two of them were withdrawn.",
        "Both of them are not true.",
    ] {
        check(vec![
            json!({"id":denial,"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched in 2026. Maple Queue was introduced in 2026. {denial}")}],"expected_relevant":false}),
        ]);
    }
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

#[test]
fn published_072_retraction_residuals_are_seen_development_controls() {
    check(
        serde_json::from_str(include_str!(
            "../../../evaluation/engine-0.7.2-verification-audit-20261014/cases.json"
        ))
        .unwrap(),
    );
}

#[test]
fn successor_retraction_and_non_event_contexts_preserve_scope() {
    let launch = "Silver Lens was launched in 2026.";
    let mut cases = Vec::new();
    for context in [
        "The newsroom retracts this claim.",
        "We retract this claim.",
        "We hereby retract the preceding statement.",
        "The newsroom denies this claim.",
        "We deny it.",
        "We withdraw this claim.",
        "The publisher withdraws it.",
        "This event never happened.",
        "That did not happen.",
        "That didn't happen.",
        "The launch never took place.",
        "This report is not factual.",
        "This is not a real event.",
        "This event did not occur.",
        "The Silver Lens launch did not happen.",
    ] {
        for kind in [
            "excerpt",
            "fact",
            "heading",
            "source_title",
            "structured_metadata",
        ] {
            for before in [true, false] {
                let signals = if kind == "excerpt" {
                    json!([{"kind":kind,"text":if before {format!("{context} {launch}")} else {format!("{launch} {context}")}}])
                } else if before {
                    json!([{"kind":kind,"text":context},{"kind":"excerpt","text":launch}])
                } else {
                    json!([{"kind":"excerpt","text":launch},{"kind":kind,"text":context}])
                };
                cases.push(json!({"id":format!("{kind}:{before}:{context}"),"signals":signals,"expected_relevant":false}));
            }
        }
    }
    for context in [
        "We do not retract this claim.",
        "The publisher does not deny it.",
        "We never withdraw this claim.",
        "We don't retract this claim.",
        "We do not hereby retract this claim.",
        "The publisher doesn't withdraw it.",
        "The model supports retractable lenses.",
        "The model supports withdrawal of consent.",
        "Silver Lens supports fictional events that never happened.",
        "Maple Queue's launch claim is retracted.",
        "We retract Maple Queue's launch claim.",
        "We withdraw the launch claim about Maple Queue.",
        "The launch claim about Maple Queue is not factual.",
        "Maple Queue was launched in 2026. We retract this claim.",
        "Maple Queue was launched in 2026. We deny it.",
        "Silver Lens supports face unlock. We withdraw it.",
        "Maple Queue was launched in 2026. That did not happen.",
        "Maple Queue was launched in 2026. This event never happened.",
        "Silver Lens supports face unlock. We retract this claim.",
    ] {
        cases.push(json!({"id":context,"signals":[{"kind":"excerpt","text":format!("{launch} {context}")}],"expected_relevant":true}));
    }
    // An explicit singular local denial can govern a feature or another target;
    // broad or unresolved disclaimers must still withhold the launch.
    for text in [
        "Silver Lens supports face unlock. We retract this claim. Silver Lens was launched in 2026.",
        "Silver Lens was launched in 2026. We retract the following statement. Silver Lens supports face unlock.",
        "Silver Lens was launched in 2026. Maple Queue was introduced in 2026. We retract this claim.",
    ] {
        cases.push(
            json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":true}),
        );
    }
    for text in [
        "Silver Lens was launched in 2026. Maple Queue was introduced in 2026. We retract all these claims.",
        "Silver Lens supports face unlock. We retract this report. Silver Lens was launched in 2026.",
        "Silver Lens was launched in 2026. We retract Maple Queue's launch claim and this claim.",
        "Silver Lens was launched in 2026. Maple Queue was introduced in 2026. We withdraw them.",
        "Silver Lens was launched in 2026. We retract Maple Queue's launch claim, including this claim.",
        "Silver Lens was launched in 2026. Silver Lens supports face unlock. We retract this claim and that claim.",
        "Silver Lens was launched in 2026. Maple Queue was introduced in 2026. That did not happen and neither did the preceding event.",
    ] {
        cases.push(
            json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":false}),
        );
    }
    check(cases);
}

#[test]
fn engine073_independent_v2_seen_replay_preserves_full_scope() {
    // Keep v2's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v2/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn engine073_independent_v3_seen_replay_preserves_full_scope() {
    // Keep v3's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v3/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn engine073_independent_v4_seen_replay_preserves_full_scope() {
    // Keep v4's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v4/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn engine073_independent_v5_seen_replay_preserves_full_scope() {
    // Keep v5's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v5/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn engine073_independent_v6_seen_replay_preserves_full_scope() {
    // Keep v6's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v6/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn engine073_independent_v7_seen_replay_preserves_full_scope() {
    // Keep v7's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v7/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn explicit_property_context_cannot_hide_global_or_launch_denials() {
    for text in [
        "Silver Lens was launched in 2026. No launch of Silver Lens took place.",
        "Silver Lens was launched in 2026. No launch for Silver Lens occurred.",
        "Silver Lens was launched in 2026. We retract the original claim.",
        "Silver Lens was launched in 2026. We withdraw the new claim.",
        "Silver Lens was launched in 2026. We retract the exact claim.",
        "Silver Lens was launched in 2026. We withdraw the specified claim.",
        "Silver Lens was launched in 2026. The truth report is withdrawn.",
        "Silver Lens was launched in 2026. We withdraw the debut claim.",
        "Silver Lens was launched in 2026. Maple Queue was launched in 2026. We withdraw one launch claim in this list.",
        "Silver Lens was launched in 2026. The launch did not take place.",
        "Silver Lens was launched in 2026. There was no launch.",
        "Silver Lens was launched in 2026. This was not an actual event.",
        "Silver Lens was launched in 2026. This wasn't a real event.",
        "Silver Lens was launched in 2026. That never actually happened.",
        "Silver Lens was launched in 2026. The launch did not really take place.",
        "Silver Lens was launched in 2026. Withdrawal of this launch claim.",
        "Silver Lens was launched in 2026. We retract the packaging claim and this launch claim.",
        "Silver Lens was launched in 2026. The report is fictional because the packaging claim is withdrawn.",
        "Silver Lens was launched in 2026. The report is fictional because we retract the packaging claim.",
        "Silver Lens was launched in 2026. This report is not true because we withdraw the packaging claim.",
        "Silver Lens was launched in 2026. This report is false because we deny that Silver Lens is made from titanium.",
        "Silver Lens was launched in 2026. We retract our claim that Maple Queue was launched and this claim.",
    ] {
        check(vec![
            json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":false}),
        ]);
    }
    for text in [
        "No launch of Maple Queue took place. Silver Lens was launched in 2026.",
        "Silver Lens was launched in 2026. No launch delay took place.",
        "We deny that Silver Lens is made from titanium. Silver Lens was launched in 2026.",
        "Silver Lens was launched in 2026. We retract the warranty claim.",
        "Silver Lens was launched in 2026. Its pricing report is not factual.",
        "Silver Lens was launched in 2026. This claim has not been withdrawn.",
        "Silver Lens was launched in 2026. The carrying-strap claim for Silver Lens is withdrawn.",
        "Silver Lens was launched in 2026. Silver Lens's packaging claim is false.",
    ] {
        check(vec![
            json!({"id":text,"signals":[{"kind":"excerpt","text":text}],"expected_relevant":true}),
        ]);
    }
}

#[test]
fn scoped_object_preserves_full_target_and_alias_names() {
    check(vec![
        json!({"id":"article-owner","entity":"The Arc","signals":[{"kind":"excerpt","text":"The Arc was launched in 2026. The Arc claim is withdrawn."}],"expected_relevant":false}),
        json!({"id":"article-property","entity":"The Arc","signals":[{"kind":"excerpt","text":"The Arc was launched in 2026. The Arc warranty claim is withdrawn."}],"expected_relevant":true}),
        json!({"id":"longer-alias-owner","entity":"Cedar","aliases":["Cedar LTS"],"signals":[{"kind":"excerpt","text":"Cedar was launched in 2026. Cedar LTS claim is withdrawn."}],"expected_relevant":false}),
        json!({"id":"longer-alias-property","entity":"Cedar","aliases":["Cedar LTS"],"signals":[{"kind":"excerpt","text":"Cedar was launched in 2026. Cedar LTS warranty claim is withdrawn."}],"expected_relevant":true}),
    ]);
}

#[test]
fn limited_property_notice_cannot_override_launch_or_unresolved_retraction() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "We retract the refraction specification for Silver Lens.",
            true,
        ),
        ("We retract the launch claim for Silver Lens.", false),
        ("We retract the launch claim.", false),
        ("We retract the statement.", false),
        (
            "The entire report is retracted: the refraction specification is withdrawn.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[
            {"kind":"source_title","text":"Limited retraction in the abstract"},
            {"kind":"excerpt","text":"Silver Lens was launched in 2026."},
            {"kind":"excerpt","text":format!("{text} The notice applies only to that physical specification.")}
        ],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn contracted_non_events_and_generic_objects_preserve_the_launch_boundary() {
    let mut cases = Vec::new();
    for (text, expected) in [
        ("Silver Lens wasn't launched.", false),
        ("Silver Lens hasn't been launched.", false),
        ("There has been no launch of Silver Lens.", false),
        ("No launch has occurred for Silver Lens.", false),
        ("We don't retract the Silver Lens launch claim.", true),
        ("We hereby retract the preceding statement.", false),
        ("The two of them were withdrawn.", false),
        ("We retract this sentence.", false),
        ("We withdraw the declaration.", false),
        ("We retract the findings.", false),
        ("We withdraw this source content.", false),
        ("We withdraw the download link.", true),
        ("The old mailing address was withdrawn.", true),
        (
            "We deny the assertion about the resin content of Silver Lens.",
            true,
        ),
        ("Maple Queue wasn't launched.", true),
        ("There has been no launch of Maple Queue.", true),
        (
            "We retract Maple Queue's launch assertion, which appeared beside the record about Silver Lens.",
            true,
        ),
        (
            "We retract Silver Lens's launch assertion, which appeared beside the record about Maple Queue.",
            false,
        ),
        (
            "We retract Maple Queue's launch assertion, which also retracts the launch statement about Silver Lens.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched in 2026."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn explicit_compound_properties_cannot_mask_launch_or_unresolved_objects() {
    let mut cases = Vec::new();
    for (text, expected) in [
        ("No actual launch of Silver Lens occurred.", false),
        (
            "The entire article is withdrawn: We withdraw Maple Queue's launch claim.",
            false,
        ),
        (
            "The entire report is false: We withdraw Maple Queue's launch claim.",
            false,
        ),
        (
            "A withdrawn report: We withdraw Maple Queue's launch claim.",
            false,
        ),
        (
            "A withdrawn report: We withdraw both the thread-pitch claim and the friction-coefficient claim.",
            false,
        ),
        (
            "The report is false: We withdraw both the thread-pitch claim and the friction-coefficient claim.",
            false,
        ),
        ("No real launch of Silver Lens happened.", false),
        (
            "We withdraw both the thread-pitch claim about Silver Lens and the friction-coefficient claim about Silver Lens.",
            true,
        ),
        (
            "We withdraw both the thread-pitch claim about Silver Lens and the launch claim about Silver Lens.",
            false,
        ),
        (
            "We withdraw both the thread-pitch claim about Silver Lens and the preceding claim.",
            false,
        ),
        (
            "We withdraw both the thread-pitch claim about Silver Lens and the friction-coefficient claim about Silver Lens. Both withdrawn objects are named in this sentence. No actual launch of Silver Lens occurred.",
            false,
        ),
        (
            "We withdraw the lead-content claim for Silver Lens. The service benchmark for Silver Lens never happened.",
            true,
        ),
        ("The report for Silver Lens did not take place.", false),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched in 2026."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn component_properties_do_not_erase_event_or_source_denials() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "We deny the physical launch-button specification for Silver Lens. This denial concerns the hardware button, not the product-history event.",
            true,
        ),
        (
            "We deny the launch claim for Silver Lens. This denial concerns the hardware button, not the product-history event.",
            false,
        ),
        ("I deny that Silver Lens's release latch is brass.", true),
        ("We retract the launch date claim for Silver Lens.", false),
        ("We withdraw the release claim for Silver Lens.", false),
        (
            "We withdraw the launch-button width claim for Silver Lens. No actual launch of Silver Lens occurred.",
            false,
        ),
        (
            "The entire report is false: We withdraw the launch-button width claim for Silver Lens.",
            false,
        ),
        (
            "We withdraw the launch claim for Maple Queue. The withdrawn launch owner is explicitly Maple Queue.",
            true,
        ),
        (
            "We withdraw a launch claim. No owner is explicitly Maple Queue.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched in 2026."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn foreign_non_event_context_cannot_clear_raw_risks_or_missing_target_ownership() {
    let case = json!({"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":"Silver Lens and Maple Queue are different products. No launch of Maple Queue occurred. The omitted event is the launch of Maple Queue."}]});
    let (policy, candidate, proposal, raw) = inputs(&case);
    assert_eq!(
        materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
            .unwrap()
            .disposition,
        Disposition::Relevant
    );
    for risk in [
        reasoning_harness_core::EvidenceLocalBlockingReason::ContextGap,
        reasoning_harness_core::EvidenceLocalBlockingReason::IdentityMapping,
        reasoning_harness_core::EvidenceLocalBlockingReason::OwnershipScope,
    ] {
        let mut raw = raw;
        raw.scope_risk = risk;
        let qualified = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        assert_ne!(
            qualified.scope_risk,
            reasoning_harness_core::EvidenceLocalBlockingReason::None
        );
        assert_ne!(
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap()
                .disposition,
            Disposition::Relevant
        );
    }
    let missing = json!({"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":"Silver Lens and Maple Queue share a product row. The product owner column is omitted. No launch of Maple Queue occurred."}]});
    let (policy, candidate, proposal, raw) = inputs(&missing);
    assert_ne!(
        materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
            .unwrap()
            .disposition,
        Disposition::Relevant
    );
}

#[test]
fn engine073_independent_v8_seen_replay_preserves_full_scope() {
    // Keep v8's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v8/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn explicit_local_owner_resolution_cannot_erase_target_or_collective_denials() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "The owner of the retracted statement is Maple Queue, a different product from Silver Lens whose launch claim is withdrawn.",
            false,
        ),
        (
            "Silver Lens was launched. Author reply: I deny its launch claim. The reply explicitly identifies its launch-claim owner as Maple Queue.",
            false,
        ),
        (
            "Author reply: I deny its launch claim. The reply explicitly identifies its launch-claim owner as Maple Queue.",
            true,
        ),
        (
            "I deny Silver Lens's launch claim. The owner of the denied statement is Maple Queue.",
            false,
        ),
        (
            "We retract all claims. The owner of the retracted statement is Maple Queue.",
            false,
        ),
        (
            "We retract this claim. No owner of the retracted statement is Maple Queue.",
            false,
        ),
        (
            "The active retraction applies to Maple Queue's launch statement.",
            true,
        ),
        (
            "The active retraction applies to Silver Lens's launch statement.",
            false,
        ),
        (
            "That claim—specifically the launch claim for Maple Queue—is withdrawn by the publisher.",
            true,
        ),
        (
            "That claim—specifically the launch claim for Silver Lens—is withdrawn by the publisher.",
            false,
        ),
        (
            "The entire report is false: The active retraction applies to Maple Queue's launch statement.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn typed_component_quotes_cannot_erase_launch_or_whole_source_denials() {
    let mut cases = Vec::new();
    for (text, expected) in [
        ("The unknown retraction object is socket plating.", false),
        (
            "I deny this statement: \"The release handle is made of aluminium, the whole article is withdrawn.\"",
            false,
        ),
        (
            "I deny this statement about Silver Lens: \"The release handle is made of aluminium.\"",
            true,
        ),
        (
            "I deny this statement about Silver Lens: \"Silver Lens was launched.\"",
            false,
        ),
        (
            "The entire report is withdrawn: \"The release handle is made of aluminium.\"",
            false,
        ),
        (
            "We deny this property statement: \"Silver Lens's launch socket is nickel plated.\" The object of denial is socket plating.",
            true,
        ),
        (
            "The retracted item is this seal-width claim, and its subject is physical seal width.",
            true,
        ),
        (
            "The retracted item is this launch claim, and its subject is physical seal width.",
            false,
        ),
        (
            "The retracted item is the preceding claim, and its subject is physical seal width.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn engine073_independent_v9_seen_replay_preserves_full_scope() {
    // Keep v9's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v9/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn typed_refs_cannot_borrow_a_different_property_or_erase_a_missing_owner() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "Object: the button-width claim for Silver Lens. The object is withdrawn.",
            true,
        ),
        (
            "Object: the launch claim for Silver Lens. The object is withdrawn.",
            false,
        ),
        (
            "Object: the button-width claim for Silver Lens. The subject is withdrawn.",
            false,
        ),
        (
            "Object: the button-width claim for Silver Lens. Silver Lens was launched. We withdraw this claim.",
            false,
        ),
        (
            "Item: the document page-count claim for Silver Lens. I withdraw that item—the launch claim for Silver Lens.",
            false,
        ),
        (
            "Owner supplement: Absent. Author reply: I deny its launch claim.",
            false,
        ),
        ("No owner of the retracted statement is Maple Queue.", false),
        (
            "Subject: the launch statement for Silver Lens. We deny the statement about this subject.",
            false,
        ),
        (
            "Object: the button-width claim for Silver Lens. We withdraw that object.",
            true,
        ),
        (
            "Object: the button-width claim for Silver Lens. Subject: the launch statement for Silver Lens. We withdraw that claim.",
            false,
        ),
        (
            "Object: the launch claim for Silver Lens. Quoted text: The release handle is brass. We retract this object.",
            false,
        ),
        (
            "Item: an unspecified launch claim. We withdraw this item.",
            false,
        ),
        (
            "Object: the button-width claim for Silver Lens. We withdraw the entire report.",
            false,
        ),
        (
            "Item: a singular claim. That claim, namely the button-width statement for Silver Lens, is withdrawn.",
            true,
        ),
        (
            "Item: the launch claim for Silver Lens. That claim, namely the button-width statement for Silver Lens, is withdrawn.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    cases.push(json!({"id":"generic_reference_cannot_borrow_metadata_object","signals":[{"kind":"source_title","text":"Object: the button-width claim for Silver Lens"},{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":"We retract this claim."}],"expected_relevant":false}));
    check(cases);
}

#[test]
fn opaque_clip_name_cannot_clear_input_or_actual_context_gaps() {
    let case = json!({"entity":"Silver Clip","signals":[{"kind":"excerpt","text":"Silver Clip was launched."},{"kind":"excerpt","text":"Subject: release-knob material on Silver Clip. Author reply: I retract the claim about this subject. Owner supplement: Silver Clip; the subject is mechanical-knob material."}]});
    let (policy, candidate, proposal, raw) = inputs(&case);
    assert_eq!(
        materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
            .unwrap()
            .disposition,
        Disposition::Relevant
    );
    for risk in [
        reasoning_harness_core::EvidenceLocalBlockingReason::ContextGap,
        reasoning_harness_core::EvidenceLocalBlockingReason::IdentityMapping,
        reasoning_harness_core::EvidenceLocalBlockingReason::OwnershipScope,
    ] {
        let mut raw = raw;
        raw.scope_risk = risk;
        assert_ne!(
            derive_effective_evidence_local_qualification_v17(
                &policy,
                &candidate,
                Some(&proposal),
                Some(&raw)
            )
            .unwrap()
            .scope_risk,
            reasoning_harness_core::EvidenceLocalBlockingReason::None
        );
        assert_ne!(
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap()
                .disposition,
            Disposition::Relevant
        );
    }
    for text in [
        "The paragraph is clipped and the product owner is omitted.",
        "The required material is outside this excerpt.",
    ] {
        let mut candidate = candidate.clone();
        candidate
            .signals
            .push(reasoning_harness_core::EvidenceRelevanceSignal {
                kind: reasoning_harness_core::EvidenceRelevanceSignalKind::Excerpt,
                text: text.to_owned(),
            });
        assert_ne!(
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap()
                .disposition,
            Disposition::Relevant
        );
    }
}

#[test]
fn engine073_independent_v10_seen_replay_preserves_full_scope() {
    // Keep v9's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v10/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn typed_owner_resolution_requires_one_complete_owner_and_no_target_denial() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "Object: the launch claim belonging to Maple Queue. This launch claim is denied.",
            true,
        ),
        (
            "Object: a launch statement. Owner supplement: Maple Queue. Author reply: I deny this object.",
            true,
        ),
        (
            "Object: a launch statement. Owner supplement: Silver Lens. Author reply: I deny this object.",
            false,
        ),
        (
            "Object: a launch statement. Owner supplement: Absent. Author reply: I deny this object.",
            false,
        ),
        (
            "Object: a launch statement. Owner supplement: Unknown Owner. Author reply: I deny this object.",
            false,
        ),
        (
            "Object: a launch statement. Owner supplement: Maple Queue. Owner supplement: Cedar Bolt. Author reply: I deny this object.",
            false,
        ),
        (
            "Object: a launch statement. Owner supplement: Maple Queue. Owner supplement: Maple Queue. Author reply: I deny this object.",
            true,
        ),
        (
            "Object: the launch statement for Silver Lens. Owner supplement: Maple Queue. Author reply: I deny this object.",
            false,
        ),
        (
            "Object: an unspecified launch statement. Owner supplement: Maple Queue. Author reply: I deny this object.",
            false,
        ),
        (
            "Silver Lens's launch claim is denied. Object: a launch statement. Owner supplement: Maple Queue. Author reply: I deny this object.",
            false,
        ),
        (
            "Owner supplement: Maple Queue. Owner supplement: Cedar Bolt. Author reply: I deny its launch claim.",
            false,
        ),
        (
            "Object: a launch statement. Owner supplement: Maple Queue or Silver Lens. Author reply: I deny this object.",
            false,
        ),
        (
            "Object: a launch statement. Owner supplement: Maple Queue. We retract the entire report.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn complete_property_apposition_cannot_overrule_event_or_whole_report_scope() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "I retract that claim—the complete surface-material statement for the physical launch cover on Silver Lens.",
            true,
        ),
        (
            "The claim about Silver Lens is denied—specifically, the physical thickness statement for the launch panel on Silver Lens.",
            true,
        ),
        (
            "I retract the launch claim—the surface-material statement for the physical launch cover on Silver Lens.",
            false,
        ),
        (
            "I retract the entire report—the surface-material statement for the physical launch cover on Silver Lens.",
            false,
        ),
        (
            "I retract that claim—the surface-material statement and the launch statement for Silver Lens.",
            false,
        ),
        (
            "Object: the launch claim for Silver Lens. I retract that claim—the surface-material statement for the physical launch cover on Silver Lens.",
            false,
        ),
        ("The denial concerns the document title font.", true),
        ("The denial concerns the launch statement.", false),
        ("The denial concerns the same unspecified property.", false),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn typed_scope_exclusion_requires_one_complete_retraction_clause() {
    let mut cases = Vec::new();
    for text in [
        "Object: the button-width claim for Silver Lens. We retract this object because Silver Lens was not launched.",
        "Object: the button-width claim for Silver Lens. We retract this object although Silver Lens was never launched.",
        "Object: the button-width claim for Silver Lens. Silver Lens was never launched before we deny this object.",
        "Object: the button-width claim for Silver Lens. We retract this object given that Silver Lens was never launched.",
        "Object: the button-width claim for Silver Lens. This launch claim is denied.",
        "Object: the button-width claim for Silver Lens. The object is denied while Silver Lens was never launched.",
        "Object: the button-width claim for Silver Lens. We deny this object alongside the launch claim for Silver Lens.",
        "Silver Lens was not launched while I retract that claim—the physical cover-width claim for Silver Lens.",
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":false}));
    }
    check(cases);
}

#[test]
fn engine073_independent_v12_seen_replay_preserves_full_scope() {
    // Keep v9's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v12/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn separated_adverse_clauses_all_need_proven_non_target_scope() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "Object: the physical launch-flange width claim for Silver Lens. We retract this object because Silver Lens does not exist.",
            false,
        ),
        (
            "Object: the physical launch-flange width claim for Silver Lens. We retract this object because we deny the printed font specification for Silver Lens.",
            true,
        ),
        (
            "Object: the physical launch-flange width claim for Silver Lens. Because we deny the launch claim for Maple Queue, we retract this object.",
            true,
        ),
        (
            "Object: the physical launch event width claim for Silver Lens. We withdraw this object.",
            false,
        ),
        (
            "Object: the physical launch-flange width claim for Silver Lens. We retract this object because we deny that launch claim.",
            false,
        ),
        (
            "Object: the physical launch-flange width claim for Silver Lens. We retract this object because we deny the launch claim for Maple Queue.",
            true,
        ),
        (
            "Object: the physical launch-flange width claim for Silver Lens. We retract this object as Silver Lens never existed.",
            false,
        ),
        (
            "We deny the launch claim for Maple Queue. The current denial names another owner and the claim is fabricated.",
            false,
        ),
        (
            "We deny the launch claim for Maple Queue. The launch denial is fully foreign and the object is the launch claim for Silver Lens.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn role_owner_frames_do_not_cross_bind_or_retype_property_as_event() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "Object: an unowned launch statement. Owner of the subject: Maple Queue. We withdraw the object.",
            false,
        ),
        (
            "Object: an unowned launch statement. Owner of the object: Maple Queue. We withdraw the object.",
            true,
        ),
        (
            "Object: the launch claim for Silver Lens. Owner of the object: Maple Queue. We withdraw the object.",
            false,
        ),
        (
            "Object: an unowned launch statement. The sole product owner of the object is either Maple Queue or Silver Lens. We withdraw the object.",
            false,
        ),
        (
            "Item: an unowned launch statement. Owner supplement:\nMaple Queue\nWe retract that launch claim.",
            true,
        ),
        (
            "Object: the physical release-button width statement for Maple Queue. Owner of the object: Maple Queue. We retract that launch claim.",
            false,
        ),
        (
            "Object: an unowned launch statement. Owner of the object: Maple Queue. Owner of the object: Cedar Bolt. We withdraw the object.",
            false,
        ),
        (
            "Object: an unowned launch statement. Owner of the object: Maple Queue. Owner of the subject: Cedar Bolt. We withdraw the object.",
            true,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn engine073_independent_v13_seen_replay_preserves_full_scope() {
    // Keep v9's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v13/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn positive_reporting_shells_preserve_every_inner_denial_scope() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "Object: the physical release-button width claim for Silver Lens. We retract this object and report that we deny the printed button-label colour claim for Silver Lens.",
            true,
        ),
        (
            "Object: the physical release-button width claim for Silver Lens. We retract this object and report that we deny the launch claim for Silver Lens.",
            false,
        ),
        (
            "Object: the physical release-button width claim for Silver Lens. We retract this object and report that we deny that launch claim.",
            false,
        ),
        (
            "Object: the physical release-button width claim for Silver Lens. We retract this object and report that we withdraw the entire report.",
            false,
        ),
        (
            "Subject: the launch statement for Maple Queue. Object: the physical release-button width claim for Silver Lens. We retract the subject and report that we deny that launch claim.",
            true,
        ),
        (
            "Subject: the launch statement for Silver Lens. Object: the physical release-button width claim for Silver Lens. We retract the subject and report that we deny that launch claim.",
            false,
        ),
        (
            "Object: the physical release-button width claim for Silver Lens. We retract the object and report that we deny that launch claim.",
            false,
        ),
        (
            "Item: an unowned launch statement. Owner supplement for the item:\nMaple Queue\nWe retract this item and report that we deny the launch statement for Cedar Bolt.",
            true,
        ),
        (
            "Item: an unowned launch statement. Owner supplement for the object:\nMaple Queue\nWe retract this item and report that we deny the launch statement for Cedar Bolt.",
            false,
        ),
        (
            "Subject: the launch statement for Silver Lens. I retract that claim, namely the complete release-button width specification for Silver Lens.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn canonical_name_notes_and_scope_annotations_do_not_erase_conflicts_or_gaps() {
    for (entity, explanation) in [
        (
            "Silver Clip",
            "Name note: Clip is part of the complete target name.",
        ),
        (
            "Silver Lens",
            "Every clause is fully scoped outside target launch.",
        ),
    ] {
        for tail in [
            "",
            " The owner column is outside this excerpt.",
            " The document is missing required context.",
        ] {
            let case = json!({"entity":entity,"signals":[
                {"kind":"excerpt","text":format!("{entity} was launched.")},
                {"kind":"excerpt","text":format!("Object: an unowned launch statement. Owner supplement for the object: Maple Queue. We retract this object and report that we deny the printed release-label colour claim for {entity}. {explanation}{tail}")}
            ]});
            let (policy, candidate, proposal, raw) = inputs(&case);
            let effective = derive_effective_evidence_local_qualification_v17(
                &policy,
                &candidate,
                Some(&proposal),
                Some(&raw),
            )
            .unwrap();
            let disposition = materialize_evidence_relevance_v30(
                &policy,
                &candidate,
                Some(&proposal),
                Some(&raw),
            )
            .unwrap()
            .disposition;
            if tail.is_empty() {
                assert_eq!(
                    disposition,
                    Disposition::Relevant,
                    "{entity}: {effective:?}"
                );
            } else {
                assert_ne!(
                    disposition,
                    Disposition::Relevant,
                    "{entity}{tail}: {effective:?}"
                );
                assert_ne!(
                    effective.scope_risk,
                    reasoning_harness_core::EvidenceLocalBlockingReason::None
                );
            }
        }
        for adverse in [
            format!("We deny the launch claim for {entity}."),
            "We deny that launch claim.".to_owned(),
            "We withdraw the entire report.".to_owned(),
        ] {
            let case = json!({"id":&adverse,"entity":entity,"signals":[{"kind":"excerpt","text":format!("{entity} was launched.")},{"kind":"excerpt","text":format!("{adverse} {explanation}")}],"expected_relevant":false});
            check(vec![case]);
        }
        // Keep the coarse risk present, so this tests the new projection's
        // raw-risk gate rather than changing historical model-risk semantics.
        let case = json!({"entity":entity,"signals":[{"kind":"excerpt","text":format!("{entity} was launched.")},{"kind":"excerpt","text":format!("Object: an unowned launch statement. Owner supplement for the object: Maple Queue. We retract this object and report that we deny the printed release-label colour claim for {entity}. {explanation}")}]});
        let (policy, candidate, proposal, mut raw) = inputs(&case);
        for risk in [
            "context_gap",
            "ownership_scope",
            "identity_mapping",
            "multiple",
        ] {
            raw.scope_risk = serde_json::from_value(json!(risk)).unwrap();
            let effective = derive_effective_evidence_local_qualification_v17(
                &policy,
                &candidate,
                Some(&proposal),
                Some(&raw),
            )
            .unwrap();
            assert_ne!(
                effective.scope_risk,
                reasoning_harness_core::EvidenceLocalBlockingReason::None
            );
            assert_ne!(
                materialize_evidence_relevance_v30(
                    &policy,
                    &candidate,
                    Some(&proposal),
                    Some(&raw)
                )
                .unwrap()
                .disposition,
                Disposition::Relevant
            );
        }
    }
}

#[test]
fn engine073_independent_v14_seen_replay_preserves_full_scope() {
    // Keep v9's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v14/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn complete_property_declarations_never_override_event_or_collective_denials() {
    let mut cases = Vec::new();
    for (text, expected) in [
        (
            "Object: the physical release-cover colour claim for Silver Lens, specifically \"the cover is not red\". We retract this object.",
            true,
        ),
        (
            "Object: the physical release-cover colour claim for Silver Lens, specifically \"Silver Lens was not launched\". We retract this object.",
            false,
        ),
        (
            "Object: the physical release-cover colour claim for Silver Lens, specifically \"the cover is not red and Silver Lens was not launched\". We retract this object.",
            false,
        ),
        (
            "Object: the physical release-cover colour claim for Silver Lens, specifically \"the entire report is not true\". We retract this object.",
            false,
        ),
        (
            "Object: the launch statement for Silver Lens. I withdraw this object and only this fully named thickness property.",
            false,
        ),
        (
            "Object: an unowned launch statement. Owner supplement for the object: unknown. I withdraw this object and only this fully named thickness property.",
            false,
        ),
        (
            "Object: the physical release-cover thickness claim for Silver Lens. I withdraw this object and only this fully named thickness property.",
            true,
        ),
        (
            "Item: the printed font-style of the word \"Launch\" in the document title for Silver Lens. We retract the item and deny the launch claim for Silver Lens.",
            false,
        ),
        (
            "I deny that launch claim. Denial object: the complete printed-word typography attribute in that title.",
            false,
        ),
        (
            "We deny that launch claim. The report completely names the foreign event being withdrawn.",
            false,
        ),
        (
            "I deny that claim—the complete width specification for the physical release socket on Silver Lens—and I withdraw the launch statement for Silver Lens.",
            false,
        ),
    ] {
        cases.push(json!({"id":text,"signals":[{"kind":"excerpt","text":"Silver Lens was launched."},{"kind":"excerpt","text":text}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn property_value_absence_and_proper_name_notes_preserve_actual_missing_context() {
    for (entity, note) in [
        (
            "Silver Lens",
            "Property note: the paper colour is not specified; this note concerns only that complete paper-colour attribute and does not omit the launch record.",
        ),
        (
            "Silver Clip",
            "Proper-name note: Clip is the last word of the complete product name, not omitted source context.",
        ),
        (
            "Silver Clip",
            "Name annotation: the proper product name ends in Clip and is complete.",
        ),
    ] {
        let case = json!({"entity":entity,"signals":[
            {"kind":"excerpt","text":format!("{entity} was launched.")},
            {"kind":"excerpt","text":format!("Object: the paper-colour specification for the printed document about {entity}. We deny that object. {note}")}
        ]});
        let (policy, candidate, proposal, raw) = inputs(&case);
        assert_eq!(
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap()
                .disposition,
            Disposition::Relevant,
            "{entity}: {note}"
        );
        for missing in [
            "The owner column is omitted from this document.",
            "The launch record is missing from this excerpt.",
        ] {
            let mut candidate = candidate.clone();
            candidate
                .signals
                .push(reasoning_harness_core::EvidenceRelevanceSignal {
                    kind: reasoning_harness_core::EvidenceRelevanceSignalKind::Excerpt,
                    text: missing.into(),
                });
            let effective = derive_effective_evidence_local_qualification_v17(
                &policy,
                &candidate,
                Some(&proposal),
                Some(&raw),
            )
            .unwrap();
            assert_ne!(
                effective.scope_risk,
                reasoning_harness_core::EvidenceLocalBlockingReason::None
            );
            assert_ne!(
                materialize_evidence_relevance_v30(
                    &policy,
                    &candidate,
                    Some(&proposal),
                    Some(&raw)
                )
                .unwrap()
                .disposition,
                Disposition::Relevant
            );
        }
        for risk in [
            "context_gap",
            "identity_mapping",
            "ownership_scope",
            "multiple",
        ] {
            let mut raw = raw;
            raw.scope_risk = serde_json::from_value(json!(risk)).unwrap();
            assert_ne!(
                materialize_evidence_relevance_v30(
                    &policy,
                    &candidate,
                    Some(&proposal),
                    Some(&raw)
                )
                .unwrap()
                .disposition,
                Disposition::Relevant,
                "{risk}: {entity}: {note}"
            );
        }
    }
}

#[test]
fn engine073_independent_v15_seen_replay_preserves_full_scope() {
    // Keep v15's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v15/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn unknown_launch_owner_never_inherits_an_adjacent_foreign_owner() {
    let mut cases = Vec::new();
    for note in [
        "its owner is not stated",
        "its owner is not identified",
        "its owner is unknown",
    ] {
        for prefix in [
            "We retract the claim about the colour of Silver Lens button, and we deny the launch of Maple Queue.",
            "We deny the launch of Maple Queue.",
        ] {
            cases.push(json!({"id":format!("unknown-owner:{prefix}:{note}"), "signals":[
                {"kind":"excerpt","text":format!("Silver Lens was launched. {prefix} We also withdraw that other launch claim; {note}.")}
            ],"expected_relevant":false}));
        }
    }
    cases.push(json!({"id":"explicit-foreign-owner", "signals":[
        {"kind":"excerpt","text":"Silver Lens was launched. We deny the launch of Maple Queue."}
    ],"expected_relevant":true}));
    check(cases);
}

#[test]
fn measured_property_denial_does_not_clear_launch_or_existence_denial() {
    let mut cases = Vec::new();
    for quantity in ["two millimetres", "three millimeters", "7 grams"] {
        cases.push(json!({"id":format!("property:{quantity}"),"signals":[
            {"kind":"excerpt","text":format!("Silver Lens was launched. We deny that claim: the thickness of Silver Lens booklet cover is {quantity}. The denial object is the cover-thickness property.")}
        ],"expected_relevant":true}));
        for adverse in [
            "We also deny that Silver Lens was launched.",
            "The editor records that Silver Lens does not exist.",
            "We withdraw the entire source.",
        ] {
            cases.push(json!({"id":format!("property-plus-adverse:{quantity}:{adverse}"),"signals":[
                {"kind":"excerpt","text":format!("Silver Lens was launched. We deny that claim: the thickness of Silver Lens booklet cover is {quantity}. {adverse}")}
            ],"expected_relevant":false}));
        }
    }
    for proposition in [
        "Silver Lens was launched",
        "Silver Lens is available",
        "the launch of Silver Lens occurred",
        "the thickness of Silver Lens booklet cover is two launch events",
        "the thickness of Silver Lens booklet cover is two millimetres and Silver Lens does not exist",
    ] {
        cases.push(json!({"id":format!("non-property:{proposition}"),"signals":[
            {"kind":"excerpt","text":format!("Silver Lens was launched. We deny that claim: {proposition}.")}
        ],"expected_relevant":false}));
    }
    check(cases);
}

#[test]
fn engine073_independent_v16_seen_replay_preserves_full_scope() {
    // Keep v16's first FAIL unchanged. This replay is development evidence.
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evaluation/engine073-launch-independent-v16/cases.json"
    ))
    .unwrap();
    let mut failures = Vec::new();
    for case in corpus["cases"].as_array().unwrap() {
        let input = &case["input"];
        let policy = serde_json::from_value(input["policy"].clone()).unwrap();
        let candidate = serde_json::from_value(input["candidate"].clone()).unwrap();
        let proposal = serde_json::from_value(input["proposal"].clone()).unwrap();
        let raw = serde_json::from_value(input["raw"].clone()).unwrap();
        let qualification = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        if (qualification.relation_scope == Relation::RequestedRelation)
            != case["expected"]["v17_requested_relation"]
                .as_bool()
                .unwrap()
            || (assessment.disposition == Disposition::Relevant)
                != case["expected"]["v30_relevant"].as_bool().unwrap()
        {
            failures.push(format!(
                "{}: {qualification:?}, {:?}",
                case["id"], assessment.disposition
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn complete_correction_frames_cannot_swallow_scope_or_owner_conflicts() {
    let mut cases = Vec::new();
    for correction in [
        "We withdraw the statement that the manual title of Silver Lens uses blue ink.",
        "The material of Silver Lens pad is separately withdrawn.",
        "The editor reports a withdrawal of the printed font claim on Silver Lens label.",
    ] {
        cases.push(json!({"id":format!("property:{correction}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. {correction}")}],"expected_relevant":true}));
        for denial in [
            "We deny that Silver Lens was launched.",
            "The editor records that Silver Lens does not exist.",
            "We withdraw the whole source.",
            "We also withdraw an unowned launch claim; its owner is not identified.",
        ] {
            cases.push(json!({"id":format!("compound:{correction}:{denial}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. {correction} {denial}")}],"expected_relevant":false}));
        }
    }
    for (subject, owner, expected) in [
        ("the launch of Maple Queue", "Maple Queue", true),
        ("the launch of Silver Lens", "Maple Queue", false),
        ("the launch of Maple Queue", "Silver Lens", false),
        ("the launch of Maple Queue", "Pine Hall", false),
        ("an unowned launch claim", "Maple Queue", false),
    ] {
        cases.push(json!({"id":format!("apposition:{subject}:{owner}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Subject: {subject}. We withdraw this Subject, the event claim owned by {owner}.")}],"expected_relevant":expected}));
    }
    check(cases);
}

#[test]
fn coordinated_typed_retraction_requires_every_named_role_to_be_safe() {
    let mut cases = Vec::new();
    for (object, subject, expected) in [
        (
            "the colour of Silver Lens cap",
            "the launch of Maple Queue",
            true,
        ),
        (
            "the colour of Silver Lens cap",
            "the launch of Silver Lens",
            false,
        ),
        (
            "the launch of Silver Lens",
            "the launch of Maple Queue",
            false,
        ),
        (
            "the colour of Silver Lens cap",
            "an unowned launch claim",
            false,
        ),
    ] {
        cases.push(json!({"id":format!("two-roles:{object}:{subject}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. Object: {object}. Subject: {subject}. We withdraw the Object and the Subject.")}],"expected_relevant":expected}));
    }
    cases.push(json!({"id":"missing-second-role","signals":[{"kind":"excerpt","text":"Silver Lens was launched. Object: the colour of Silver Lens cap. We withdraw the Object and the Subject."}],"expected_relevant":false}));
    check(cases);
}

#[test]
fn foreign_antecedent_cannot_bind_a_different_or_unknown_owned_launch() {
    let mut cases = Vec::new();
    for reference in [
        "that other launch claim",
        "another launch claim",
        "a different launch claim",
        "that launch claim",
    ] {
        for note in [
            "its owner was not specified",
            "the other launch claim has no named owner",
            "the owner of that other claim is unstated",
            "its owner is not stated",
        ] {
            cases.push(json!({"id":format!("unknown:{reference}:{note}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. We retract the claim about the colour of Silver Lens button, and we deny the launch of Maple Queue. We also withdraw {reference}; {note}.")}],"expected_relevant":false}));
        }
        if reference != "that launch claim" {
            cases.push(json!({"id":format!("distinct:{reference}"),"signals":[{"kind":"excerpt","text":format!("Silver Lens was launched. We deny the launch of Maple Queue. We withdraw {reference}.")}],"expected_relevant":false}));
        }
    }
    cases.push(json!({"id":"adjacent-same-foreign-claim","signals":[{"kind":"excerpt","text":"Silver Lens was launched. We deny the launch of Maple Queue. We withdraw that launch claim."}],"expected_relevant":true}));
    for qualifier in [
        "No second owner is assigned.",
        "No other owner is named.",
        "No additional owner is listed.",
    ] {
        cases.push(json!({"id":format!("unique-owner:{qualifier}"),"signals":[
            {"kind":"excerpt","text":"Silver Lens was launched."},
            {"kind":"excerpt","text":format!("Object: a launch statement. Author response: \"I deny that object.\" Adjacent owner supplement: Maple Queue owns the denied launch statement. {qualifier}")}
        ],"expected_relevant":true}));
    }
    check(cases);
}
