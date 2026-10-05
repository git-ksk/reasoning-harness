use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceExplicitLocalAbsence, EvidenceLocalQualification, EvidenceLocalSupport,
    EvidenceQualificationRisk, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy, materialize_evidence_relevance_v7,
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
    expected_local_qualification: EvidenceLocalQualification,
    expected_disposition: EvidenceRelevanceDisposition,
}

fn manifest() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v12/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read calibration manifest"))
        .expect("parse calibration manifest")
}

fn qualification_has_risk(qualification: &EvidenceLocalQualification) -> bool {
    qualification.identity_mapping_risk != EvidenceQualificationRisk::Absent
        || qualification.ownership_scope_risk != EvidenceQualificationRisk::Absent
        || qualification.context_completeness_risk != EvidenceQualificationRisk::Absent
}

#[test]
fn v12_expected_primary_and_guard_materialize_all_cases() {
    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v12");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "synthetic_calibration_v12_concrete_local_risk_semantics_with_fresh_cases"
    );
    assert_eq!(manifest.cases.len(), 73);

    for case in &manifest.cases {
        assert_eq!(case.policy.assessment_budget.max_model_attempts, 2);
        assert_eq!(case.policy.assessment_budget.max_tokens, 192);
        assert_eq!(case.policy.assessment_budget.max_elapsed_ms, 60_000);
        let assessment = materialize_evidence_relevance_v7(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} materialization failed: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{}",
            case.id
        );
    }
}

#[test]
fn v12_contains_eight_fresh_concrete_risk_semantic_families() {
    let manifest = manifest();
    let fresh = manifest
        .cases
        .iter()
        .filter(|case| {
            case.id
                .split_once('_')
                .and_then(|(prefix, _)| prefix.parse::<usize>().ok())
                .is_some_and(|number| (66..=73).contains(&number))
        })
        .collect::<Vec<_>>();
    assert_eq!(fresh.len(), 8);
    let observed = fresh
        .iter()
        .map(|case| case.family.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed,
        BTreeSet::from([
            "v12_fresh_harness_alias_no_open_world_risk",
            "v12_fresh_canonical_no_hypothetical_risk",
            "v12_fresh_explicit_identity_risk",
            "v12_fresh_single_owner_no_scope_risk",
            "v12_fresh_shared_owner_risk",
            "v12_fresh_complete_context_no_risk",
            "v12_fresh_omitted_referent_context_risk",
            "v12_fresh_clear_other_target_no_hypothetical_risk",
        ])
    );
}

#[test]
fn v12_qualification_surface_covers_support_risk_and_explicit_absence() {
    let manifest = manifest();
    let risk_cases = manifest
        .cases
        .iter()
        .filter(|case| qualification_has_risk(&case.expected_local_qualification))
        .count();
    let explicit_absence = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_local_qualification.explicit_local_absence
                == EvidenceExplicitLocalAbsence::Present
        })
        .count();
    let target_supported = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_local_qualification.target_support == EvidenceLocalSupport::Supported
        })
        .count();
    let target_not_supported = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_local_qualification.target_support == EvidenceLocalSupport::NotSupported
        })
        .count();
    let target_unresolved = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_local_qualification.target_support == EvidenceLocalSupport::Unresolved
        })
        .count();

    assert_eq!(risk_cases, 25);
    assert_eq!(explicit_absence, 6);
    assert_eq!(target_supported, 31);
    assert_eq!(target_not_supported, 21);
    assert_eq!(target_unresolved, 21);
}

#[test]
fn v12_every_ambiguous_case_has_unresolved_support_or_fail_closed_risk() {
    let manifest = manifest();
    for case in manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Ambiguous)
    {
        let q = &case.expected_local_qualification;
        assert!(
            qualification_has_risk(q)
                || q.target_support == EvidenceLocalSupport::Unresolved
                || q.relation_support == EvidenceLocalSupport::Unresolved,
            "{}",
            case.id
        );
    }
}

#[test]
fn v12_production_motivating_product_is_not_a_tuning_fixture() {
    let raw = serde_json::to_string(&manifest()).expect("serialize manifest");
    assert!(!raw.to_ascii_lowercase().contains("cloudwatch omni"));
}
