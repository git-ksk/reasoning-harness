use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceLocalRelationScope, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v17, materialize_evidence_relevance_v30,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
struct Corpus {
    suite_id: String,
    candidate_sha: String,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    target: String,
    signals: Vec<Value>,
    expected_relevant: bool,
    category: String,
}

#[test]
fn independently_authored_launch_source_controls_v5() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/launch-514-515-independent-v5/manifest.json");
    let corpus: Corpus = serde_json::from_slice(&fs::read(path).expect("read frozen corpus"))
        .expect("parse independent corpus");
    assert_eq!(corpus.suite_id, "launch-514-515-independent-v5");
    assert_eq!(
        corpus.candidate_sha,
        "b73559853134887d9b077baa7433fb3c6a886629"
    );
    assert_eq!(corpus.cases.len(), 32);
    assert_eq!(
        corpus
            .cases
            .iter()
            .filter(|case| case.expected_relevant)
            .count(),
        12
    );
    let proposal: EvidenceRelevanceBindingProposal = serde_json::from_value(json!({
        "target_binding":"exact", "relation_binding":"exact"
    }))
    .unwrap();
    let raw: EvidenceLocalQualificationV6 = serde_json::from_value(json!({
        "identity_scope":"exact_target", "relation_scope":"requested_relation", "scope_risk":"none"
    }))
    .unwrap();
    let mut ids = std::collections::BTreeSet::new();
    let mut failures = Vec::new();
    for case in corpus.cases {
        assert!(ids.insert(case.id.clone()), "duplicate case ID");
        let policy: EvidenceRelevanceTargetPolicy = serde_json::from_value(json!({
            "policy_id":format!("independent:{}",case.id), "target_id":case.id,
            "target_question":format!("When was {} launched?",case.target),
            "entity":{"canonical_id":case.target,"canonical_name":case.target,"aliases":[]},
            "relation":"change_or_launch", "identity_requirement":"require_harness_anchor",
            "assessment_budget":{"max_model_attempts":2,"max_tokens":192,"max_elapsed_ms":90000}
        }))
        .unwrap();
        let candidate: EvidenceRelevanceCandidate = serde_json::from_value(json!({
            "evidence_id":case.id, "source_id":case.id, "signals":case.signals
        }))
        .unwrap();
        let effective = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let assessment =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap();
        println!(
            "{} category={} expected_relevant={} disposition={:?} relation={:?}",
            case.id,
            case.category,
            case.expected_relevant,
            assessment.disposition,
            effective.relation_scope
        );
        if (assessment.disposition == EvidenceRelevanceDisposition::Relevant)
            != case.expected_relevant
            || (case.expected_relevant
                && effective.relation_scope != EvidenceLocalRelationScope::RequestedRelation)
        {
            failures.push(case.id);
        }
    }
    assert!(
        failures.is_empty(),
        "independent control failures: {failures:?}"
    );
}
