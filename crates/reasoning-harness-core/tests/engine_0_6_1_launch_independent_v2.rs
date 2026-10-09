use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceLocalRelationScope, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v17, materialize_evidence_relevance_v30,
};
use serde::Deserialize;
use serde_json::json;
use std::{fs, path::PathBuf};

#[derive(Deserialize)]
struct Corpus {
    suite_id: String,
    issue: u64,
    status: String,
    protocol: String,
    precommitted_expected_positive: usize,
    precommitted_expected_negative: usize,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    target: String,
    signals: Vec<serde_json::Value>,
    expected_relevant: bool,
}

#[test]
fn independently_fixed_launch_authority_matrix_v2() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/engine-0.6.1-launch-independent-v2/manifest.json");
    let corpus: Corpus = serde_json::from_slice(&fs::read(&path).expect("read fixed corpus"))
        .expect("parse fixed corpus");
    assert_eq!(corpus.suite_id, "engine-0.6.1-launch-independent-v2");
    assert_eq!(corpus.issue, 479);
    assert_eq!(corpus.status, "fresh_unobserved_holdout");
    assert_eq!(corpus.protocol, "effective_v17_materialize_v30");
    assert_eq!(corpus.cases.len(), 30);
    assert_eq!(corpus.precommitted_expected_positive, 12);
    assert_eq!(corpus.precommitted_expected_negative, 18);
    let proposal: EvidenceRelevanceBindingProposal = serde_json::from_value(json!({
        "target_binding": "exact", "relation_binding": "exact"
    }))
    .unwrap();
    let raw: EvidenceLocalQualificationV6 = serde_json::from_value(json!({
        "identity_scope":"exact_target","relation_scope":"requested_relation","scope_risk":"none"
    }))
    .unwrap();
    let mut mismatches = Vec::new();
    for case in corpus.cases {
        let policy: EvidenceRelevanceTargetPolicy = serde_json::from_value(json!({
            "policy_id":format!("engine-0.6.1:{}",case.id),
            "target_id":case.id,
            "target_question":format!("When was {} launched?",case.target),
            "entity":{
                "canonical_id":format!("engine-0.6.1:{}",case.target),
                "canonical_name":case.target,
                "aliases":[]
            },
            "relation":"change_or_launch",
            "identity_requirement":"require_harness_anchor",
            "assessment_budget":{"max_model_attempts":2,"max_tokens":192,"max_elapsed_ms":90000}
        }))
        .unwrap();
        let candidate: EvidenceRelevanceCandidate = serde_json::from_value(json!({
            "evidence_id":format!("evidence:{}",case.id),
            "source_id":format!("source:{}",case.id),
            "signals":case.signals,
        }))
        .unwrap();
        let effective = derive_effective_evidence_local_qualification_v17(
            &policy,
            &candidate,
            Some(&proposal),
            Some(&raw),
        )
        .unwrap();
        let disposition =
            materialize_evidence_relevance_v30(&policy, &candidate, Some(&proposal), Some(&raw))
                .unwrap()
                .disposition;
        let positive = disposition == EvidenceRelevanceDisposition::Relevant;
        if positive != case.expected_relevant
            || (case.expected_relevant
                && effective.relation_scope != EvidenceLocalRelationScope::RequestedRelation)
        {
            mismatches.push(format!(
                "{} expected_relevant={} observed={:?} relation={:?}",
                case.id, case.expected_relevant, disposition, effective.relation_scope
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "precommitted holdout misses:\n{}",
        mismatches.join("\n")
    );
}
