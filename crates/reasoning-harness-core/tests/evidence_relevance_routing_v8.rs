use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceNegativeTargetConfirmation, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    materialize_evidence_relevance_v4,
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
    expected_proposal: EvidenceRelevanceBindingProposal,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expected_negative_target_confirmation: Option<EvidenceNegativeTargetConfirmation>,
    expected_disposition: EvidenceRelevanceDisposition,
}

fn manifest() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v8/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read calibration manifest"))
        .expect("parse calibration manifest")
}

#[test]
fn fresh_calibration_v8_expected_bindings_materialize_deterministically() {
    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v8");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "synthetic_calibration_only_v8_negative_target_confirmation_successor"
    );
    assert_eq!(manifest.cases.len(), 32);

    for case in &manifest.cases {
        assert_eq!(case.policy.assessment_budget.max_model_attempts, 2);
        assert_eq!(case.policy.assessment_budget.max_tokens, 192);
        assert_eq!(case.policy.assessment_budget.max_elapsed_ms, 60_000);
        if case.expected_proposal.target_binding == EvidenceRelevanceBinding::Different {
            assert!(
                case.expected_negative_target_confirmation.is_some(),
                "{} negative target case lacks confirmation label",
                case.id
            );
        } else {
            assert!(
                case.expected_negative_target_confirmation.is_none(),
                "{} non-negative target case unexpectedly carries confirmation",
                case.id
            );
        }
    }

    for case in manifest.cases {
        assert!(!case.id.trim().is_empty());
        assert!(!case.family.trim().is_empty());
        assert!(!case.task.trim().is_empty());
        let assessment = materialize_evidence_relevance_v4(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            case.expected_negative_target_confirmation,
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
    assert!(negatives >= 10);
    assert!(ambiguous >= 7);
}

#[test]
fn successor_adds_fresh_negative_confirmation_cases() {
    let manifest = manifest();
    let fresh = manifest
        .cases
        .iter()
        .filter(|case| {
            case.id.starts_with("27_")
                || case.id.starts_with("28_")
                || case.id.starts_with("29_")
                || case.id.starts_with("30_")
                || case.id.starts_with("31_")
                || case.id.starts_with("32_")
        })
        .count();
    assert_eq!(fresh, 6);

    let distinct = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_negative_target_confirmation
                == Some(EvidenceNegativeTargetConfirmation::ConfirmedDistinctEntity)
        })
        .count();
    let absent = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_negative_target_confirmation
                == Some(EvidenceNegativeTargetConfirmation::ConfirmedTargetAbsent)
        })
        .count();
    assert!(distinct >= 6);
    assert!(absent >= 4);
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
        let assessment = materialize_evidence_relevance_v4(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            case.expected_negative_target_confirmation,
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
