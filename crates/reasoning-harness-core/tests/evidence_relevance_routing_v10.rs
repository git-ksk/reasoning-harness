use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceNegativeSafetyDecision, EvidencePositiveSafetyDecision, EvidenceRelevanceBinding,
    EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate, EvidenceRelevanceDisposition,
    EvidenceRelevanceTargetPolicy, materialize_evidence_relevance_v6,
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
    expected_negative_safety_decision: Option<EvidenceNegativeSafetyDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expected_positive_safety_decision: Option<EvidencePositiveSafetyDecision>,
    expected_disposition: EvidenceRelevanceDisposition,
}

fn manifest() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v10/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read calibration manifest"))
        .expect("parse calibration manifest")
}

#[test]
fn fresh_calibration_v10_expected_bindings_materialize_deterministically() {
    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v10");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "synthetic_calibration_v10_action_safety_structured_confirmation_with_fresh_cases"
    );
    assert_eq!(manifest.cases.len(), 56);

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
            case.expected_negative_safety_decision.is_some(),
            expects_negative,
            "{}",
            case.id
        );
        assert_eq!(
            case.expected_positive_safety_decision.is_some(),
            expects_positive,
            "{}",
            case.id
        );
        assert!(
            !(case.expected_negative_safety_decision.is_some()
                && case.expected_positive_safety_decision.is_some())
        );

        let assessment = materialize_evidence_relevance_v6(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            case.expected_negative_safety_decision,
            case.expected_positive_safety_decision,
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
fn v10_contains_independent_fresh_action_safety_stress_set() {
    let manifest = manifest();
    let fresh = manifest
        .cases
        .iter()
        .filter(|case| {
            case.id
                .split_once('_')
                .and_then(|(prefix, _)| prefix.parse::<usize>().ok())
                .is_some_and(|number| (48..=56).contains(&number))
        })
        .collect::<Vec<_>>();
    assert_eq!(fresh.len(), 9);
    let required_families = BTreeSet::from([
        "v10_fresh_positive_distributed_scope",
        "v10_fresh_positive_freshness_separate",
        "v10_fresh_negative_sibling_scope",
        "v10_fresh_negative_comparison_context",
        "v10_fresh_negative_generic_absence",
        "v10_fresh_ambiguous_alias_mapping",
        "v10_fresh_ambiguous_shared_ownership",
        "v10_fresh_positive_prompt_injection",
        "v10_fresh_positive_boundary_abstain",
    ]);
    let observed = fresh
        .iter()
        .map(|case| case.family.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(observed, required_families);
}

#[test]
fn v10_action_decisions_cover_reject_accept_and_abstain_paths() {
    let manifest = manifest();
    let reject = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_negative_safety_decision
                == Some(EvidenceNegativeSafetyDecision::SafeToReject)
        })
        .count();
    let negative_abstain = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_negative_safety_decision == Some(EvidenceNegativeSafetyDecision::Abstain)
        })
        .count();
    let positive_abstain = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_positive_safety_decision == Some(EvidencePositiveSafetyDecision::Abstain)
        })
        .count();
    let accept = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_positive_safety_decision
                == Some(EvidencePositiveSafetyDecision::SafeToAccept)
        })
        .count();
    assert_eq!(reject, 18);
    assert_eq!(negative_abstain, 17);
    assert_eq!(accept, 16);
    assert_eq!(positive_abstain, 1);
}

#[test]
fn v10_exact_relation_mismatch_needs_no_safety_decision() {
    let manifest = manifest();
    for case in manifest.cases.iter().filter(|case| {
        case.expected_proposal.target_binding == EvidenceRelevanceBinding::Exact
            && case.expected_proposal.relation_binding == EvidenceRelevanceBinding::Different
    }) {
        assert!(
            case.expected_negative_safety_decision.is_none(),
            "{}",
            case.id
        );
        assert!(
            case.expected_positive_safety_decision.is_none(),
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
fn v10_production_motivating_product_is_not_a_tuning_fixture() {
    let raw = serde_json::to_string(&manifest()).expect("serialize manifest");
    assert!(!raw.to_ascii_lowercase().contains("cloudwatch omni"));
}
