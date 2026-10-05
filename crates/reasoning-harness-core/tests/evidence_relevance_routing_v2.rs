use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceProposal,
    EvidenceRelevanceTargetPolicy, materialize_evidence_relevance,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Case {
    id: String,
    family: String,
    task: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: EvidenceRelevanceDisposition,
    expected_disposition: EvidenceRelevanceDisposition,
}

fn manifest() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v2/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read calibration manifest"))
        .expect("parse calibration manifest")
}

#[test]
fn fresh_calibration_v2_expected_proposals_materialize_deterministically() {
    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v2");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "synthetic_only_production_incident_excluded"
    );
    assert_eq!(manifest.cases.len(), 26);

    for case in manifest.cases {
        assert!(!case.id.trim().is_empty());
        assert!(!case.family.trim().is_empty());
        assert!(!case.task.trim().is_empty());
        let assessment = materialize_evidence_relevance(
            &case.policy,
            &case.candidate,
            Some(&EvidenceRelevanceProposal {
                disposition: case.expected_proposal,
            }),
        )
        .unwrap_or_else(|error| panic!("{} materialization failed: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{} materialized unexpected disposition",
            case.id
        );
        assert_eq!(assessment.target_id, case.policy.target_id);
        assert_eq!(assessment.evidence_id, case.candidate.evidence_id);
        assert_eq!(assessment.source_id, case.candidate.source_id);
    }
}

#[test]
fn calibration_contains_required_positive_negative_and_ambiguous_families() {
    let manifest = manifest();
    let positives = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Relevant)
        .count();
    let negatives = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Irrelevant)
        .count();
    let ambiguous = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Ambiguous)
        .count();

    assert!(positives >= 10);
    assert!(negatives >= 7);
    assert!(ambiguous >= 5);
}

#[test]
fn production_motivating_product_is_not_a_tuning_fixture() {
    let raw = serde_json::to_string(&manifest()).expect("serialize manifest");
    assert!(!raw.to_ascii_lowercase().contains("cloudwatch omni"));
}

#[test]
fn wrong_target_expected_cases_never_materialize_relevant() {
    let manifest = manifest();
    for case in manifest
        .cases
        .iter()
        .filter(|case| case.family.starts_with("negative_"))
    {
        let assessment = materialize_evidence_relevance(
            &case.policy,
            &case.candidate,
            Some(&EvidenceRelevanceProposal {
                disposition: case.expected_proposal,
            }),
        )
        .unwrap();
        assert_ne!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Relevant,
            "{} unexpectedly retained wrong-target material",
            case.id
        );
    }
}
