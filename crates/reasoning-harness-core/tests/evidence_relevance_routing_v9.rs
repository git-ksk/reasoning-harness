use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceNegativeTargetConfirmationV2, EvidencePositiveTargetConfirmation,
    EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy, materialize_evidence_relevance_v5,
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
    expected_negative_target_confirmation: Option<EvidenceNegativeTargetConfirmationV2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expected_positive_target_confirmation: Option<EvidencePositiveTargetConfirmation>,
    expected_disposition: EvidenceRelevanceDisposition,
}

fn manifest() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v9/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read calibration manifest"))
        .expect("parse calibration manifest")
}

#[test]
fn fresh_calibration_v9_expected_bindings_materialize_deterministically() {
    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v9");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "synthetic_calibration_v9_dual_confirmation_with_fresh_identity_cases"
    );
    assert_eq!(manifest.cases.len(), 47);

    for case in &manifest.cases {
        assert_eq!(case.policy.assessment_budget.max_model_attempts, 2);
        assert_eq!(case.policy.assessment_budget.max_tokens, 192);
        assert_eq!(case.policy.assessment_budget.max_elapsed_ms, 60_000);

        let expects_negative =
            case.expected_proposal.target_binding != EvidenceRelevanceBinding::Exact;
        let expects_positive = case.expected_proposal.target_binding
            == EvidenceRelevanceBinding::Exact
            && case.expected_proposal.relation_binding == EvidenceRelevanceBinding::Exact;
        assert_eq!(
            case.expected_negative_target_confirmation.is_some(),
            expects_negative,
            "{} negative confirmation routing mismatch",
            case.id
        );
        assert_eq!(
            case.expected_positive_target_confirmation.is_some(),
            expects_positive,
            "{} positive confirmation routing mismatch",
            case.id
        );
        assert!(
            !(case.expected_negative_target_confirmation.is_some()
                && case.expected_positive_target_confirmation.is_some())
        );

        let assessment = materialize_evidence_relevance_v5(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            case.expected_negative_target_confirmation,
            case.expected_positive_target_confirmation,
        )
        .unwrap_or_else(|error| panic!("{} materialization failed: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{} materialized unexpected disposition",
            case.id
        );
    }
}

#[test]
fn v9_contains_independent_fresh_identity_stress_set() {
    let manifest = manifest();
    let fresh = manifest
        .cases
        .iter()
        .filter(|case| {
            case.id
                .split_once('_')
                .and_then(|(prefix, _)| prefix.parse::<usize>().ok())
                .is_some_and(|number| (33..=47).contains(&number))
        })
        .collect::<Vec<_>>();
    assert_eq!(fresh.len(), 15);

    let required_families = BTreeSet::from([
        "fresh_exact_binding_conflict",
        "fresh_positive_exact",
        "fresh_shared_table_ownership",
        "fresh_explicit_distinct",
        "fresh_sibling_overlap",
        "fresh_generic_local_absence",
        "fresh_explicit_local_absence",
        "fresh_rename_uncertainty",
        "fresh_successor_uncertainty",
        "fresh_alias_uncertainty",
        "fresh_partial_truncated",
        "fresh_prompt_injection",
        "fresh_url_only",
        "fresh_mixed_multi_product",
        "fresh_relation_mismatch",
    ]);
    let observed = fresh
        .iter()
        .map(|case| case.family.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(observed, required_families);
}

#[test]
fn v9_confirmation_subtypes_cover_safe_negative_and_abstention_paths() {
    let manifest = manifest();
    let distinct = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_negative_target_confirmation
                == Some(EvidenceNegativeTargetConfirmationV2::ConfirmedDistinctEntity)
        })
        .count();
    let absent = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_negative_target_confirmation
                == Some(EvidenceNegativeTargetConfirmationV2::ConfirmedLocalTargetAbsent)
        })
        .count();
    let unconfirmed = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_negative_target_confirmation
                == Some(EvidenceNegativeTargetConfirmationV2::NotConfirmed)
        })
        .count();
    let positive = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_positive_target_confirmation
                == Some(EvidencePositiveTargetConfirmation::ConfirmedTargetLocalBinding)
        })
        .count();

    assert_eq!(distinct, 8);
    assert_eq!(absent, 7);
    assert_eq!(unconfirmed, 15);
    assert_eq!(positive, 13);
}

#[test]
fn v9_exact_relation_mismatch_needs_no_confirmation() {
    let manifest = manifest();
    for case in manifest.cases.iter().filter(|case| {
        case.expected_proposal.target_binding == EvidenceRelevanceBinding::Exact
            && case.expected_proposal.relation_binding == EvidenceRelevanceBinding::Different
    }) {
        assert!(
            case.expected_negative_target_confirmation.is_none(),
            "{}",
            case.id
        );
        assert!(
            case.expected_positive_target_confirmation.is_none(),
            "{}",
            case.id
        );
        assert_eq!(
            case.expected_disposition,
            EvidenceRelevanceDisposition::Irrelevant
        );
    }
}

#[test]
fn v9_production_motivating_product_is_not_a_tuning_fixture() {
    let raw = serde_json::to_string(&manifest()).expect("serialize manifest");
    assert!(!raw.to_ascii_lowercase().contains("cloudwatch omni"));
}
