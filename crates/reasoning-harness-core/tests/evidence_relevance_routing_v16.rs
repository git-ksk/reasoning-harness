use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    materialize_evidence_relevance_v11,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    annotation_protocol_id: String,
    fixed_core_id: String,
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
    expected_local_qualification: EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

#[derive(Debug, Deserialize)]
struct FixedCore {
    schema_version: String,
    issue: u64,
    source_suite: String,
    source_freeze_tag: String,
    selection_rule: String,
    growth_policy: String,
    case_count: usize,
    case_ids: Vec<String>,
}

fn manifest() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v16/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read v16 calibration manifest"))
        .expect("parse v16 calibration manifest")
}

fn fixed_core() -> FixedCore {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-core-v1/case-ids.json");
    serde_json::from_slice(&fs::read(path).expect("read fixed core")).expect("parse fixed core")
}

#[test]
fn v16_fixed_core_identity_and_case_order_are_exact() {
    let manifest = manifest();
    let core = fixed_core();

    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v16");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "fixed_core_v1_semantic_successor_no_case_growth"
    );
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-scope-verifier-v16"
    );
    assert_eq!(manifest.fixed_core_id, "evidence-relevance-fixed-core-v1");

    assert_eq!(core.schema_version, "evidence-relevance-fixed-core-v1");
    assert_eq!(core.issue, 462);
    assert_eq!(core.source_suite, "evidence-relevance-calibration-v13");
    assert_eq!(
        core.source_freeze_tag,
        "engine-0.6-evidence-relevance-calibration-v13-freeze"
    );
    assert_eq!(core.growth_policy, "fixed_no_new_cases");
    assert!(
        core.selection_rule
            .contains("selected before any v14 live observation")
    );
    assert_eq!(core.case_count, 48);
    assert_eq!(manifest.cases.len(), 48);

    let ids = manifest
        .cases
        .iter()
        .map(|case| case.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(ids, core.case_ids);
    assert_eq!(ids.iter().collect::<BTreeSet<_>>().len(), ids.len());
}

#[test]
fn v16_expected_scope_annotations_materialize_all_48_cases() {
    let manifest = manifest();

    for case in &manifest.cases {
        assert_eq!(case.policy.assessment_budget.max_model_attempts, 2);
        assert_eq!(case.policy.assessment_budget.max_tokens, 192);
        assert_eq!(case.policy.assessment_budget.max_elapsed_ms, 60_000);

        let assessment = materialize_evidence_relevance_v11(
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
fn v16_fixed_core_disposition_and_scope_coverage_are_exact() {
    let manifest = manifest();

    let count_disposition = |value| {
        manifest
            .cases
            .iter()
            .filter(|case| case.expected_disposition == value)
            .count()
    };
    assert_eq!(
        (
            count_disposition(EvidenceRelevanceDisposition::Relevant),
            count_disposition(EvidenceRelevanceDisposition::Irrelevant),
            count_disposition(EvidenceRelevanceDisposition::Ambiguous),
        ),
        (14, 18, 16)
    );

    let count_identity = |value| {
        manifest
            .cases
            .iter()
            .filter(|case| case.expected_local_qualification.identity_scope == value)
            .count()
    };
    assert_eq!(count_identity(EvidenceLocalIdentityScope::ExactTarget), 19);
    assert_eq!(
        count_identity(EvidenceLocalIdentityScope::DistinctTarget),
        10
    );
    assert_eq!(count_identity(EvidenceLocalIdentityScope::TargetAbsent), 5);
    assert_eq!(count_identity(EvidenceLocalIdentityScope::Unresolved), 14);

    let count_relation = |value| {
        manifest
            .cases
            .iter()
            .filter(|case| case.expected_local_qualification.relation_scope == value)
            .count()
    };
    assert_eq!(
        count_relation(EvidenceLocalRelationScope::RequestedRelation),
        37
    );
    assert_eq!(
        count_relation(EvidenceLocalRelationScope::DifferentRelation),
        4
    );
    assert_eq!(
        count_relation(EvidenceLocalRelationScope::RelationAbsent),
        5
    );
    assert_eq!(count_relation(EvidenceLocalRelationScope::Unresolved), 2);

    let count_risk = |value| {
        manifest
            .cases
            .iter()
            .filter(|case| case.expected_local_qualification.scope_risk == value)
            .count()
    };
    assert_eq!(count_risk(EvidenceLocalBlockingReason::None), 32);
    assert_eq!(count_risk(EvidenceLocalBlockingReason::IdentityMapping), 5);
    assert_eq!(count_risk(EvidenceLocalBlockingReason::OwnershipScope), 1);
    assert_eq!(count_risk(EvidenceLocalBlockingReason::ContextGap), 5);
    assert_eq!(count_risk(EvidenceLocalBlockingReason::Multiple), 5);
}

#[test]
fn v16_any_scope_risk_is_fail_closed_to_ambiguous() {
    let manifest = manifest();

    for case in manifest.cases.iter().filter(|case| {
        case.expected_local_qualification.scope_risk != EvidenceLocalBlockingReason::None
    }) {
        assert_eq!(
            case.expected_disposition,
            EvidenceRelevanceDisposition::Ambiguous,
            "{}",
            case.id
        );
    }
}

#[test]
fn v16_unresolved_primary_cannot_be_rescued_to_relevant() {
    let manifest = manifest();
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "01_exact_name_availability")
        .expect("positive anchored control");
    let positive_verifier = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::ExactTarget,
        relation_scope: EvidenceLocalRelationScope::RequestedRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };

    for proposal in [
        EvidenceRelevanceBindingProposal {
            target_binding: EvidenceRelevanceBinding::Unresolved,
            relation_binding: EvidenceRelevanceBinding::Exact,
        },
        EvidenceRelevanceBindingProposal {
            target_binding: EvidenceRelevanceBinding::Exact,
            relation_binding: EvidenceRelevanceBinding::Unresolved,
        },
        EvidenceRelevanceBindingProposal {
            target_binding: EvidenceRelevanceBinding::Unresolved,
            relation_binding: EvidenceRelevanceBinding::Unresolved,
        },
    ] {
        let assessment = materialize_evidence_relevance_v11(
            &case.policy,
            &case.candidate,
            Some(&proposal),
            Some(&positive_verifier),
        )
        .expect("materialize");
        assert_ne!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Relevant
        );
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Ambiguous
        );
    }
}

#[test]
fn v16_scope_risk_blocks_terminal_decisions_even_with_otherwise_exact_votes() {
    let manifest = manifest();
    let positive = manifest
        .cases
        .iter()
        .find(|case| case.id == "01_exact_name_availability")
        .expect("positive control");

    for risk in [
        EvidenceLocalBlockingReason::IdentityMapping,
        EvidenceLocalBlockingReason::OwnershipScope,
        EvidenceLocalBlockingReason::ContextGap,
        EvidenceLocalBlockingReason::Multiple,
    ] {
        let qualification = EvidenceLocalQualificationV6 {
            identity_scope: EvidenceLocalIdentityScope::ExactTarget,
            relation_scope: EvidenceLocalRelationScope::RequestedRelation,
            scope_risk: risk,
        };
        let assessment = materialize_evidence_relevance_v11(
            &positive.policy,
            &positive.candidate,
            Some(&positive.expected_proposal),
            Some(&qualification),
        )
        .expect("materialize");
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Ambiguous
        );
    }
}

#[test]
fn v16_target_and_relation_axes_remain_orthogonal() {
    let manifest = manifest();

    let sibling_same_relation = manifest
        .cases
        .iter()
        .find(|case| case.id == "14_sibling_product_overlap")
        .expect("sibling same relation");
    assert_eq!(
        sibling_same_relation.expected_proposal.target_binding,
        EvidenceRelevanceBinding::Different
    );
    assert_eq!(
        sibling_same_relation.expected_proposal.relation_binding,
        EvidenceRelevanceBinding::Exact
    );
    assert_eq!(
        sibling_same_relation
            .expected_local_qualification
            .identity_scope,
        EvidenceLocalIdentityScope::DistinctTarget
    );
    assert_eq!(
        sibling_same_relation
            .expected_local_qualification
            .relation_scope,
        EvidenceLocalRelationScope::RequestedRelation
    );

    let exact_wrong_relation = manifest
        .cases
        .iter()
        .find(|case| case.id == "13_same_service_different_feature")
        .expect("exact target wrong relation");
    assert_eq!(
        exact_wrong_relation
            .expected_local_qualification
            .identity_scope,
        EvidenceLocalIdentityScope::ExactTarget
    );
    assert_eq!(
        exact_wrong_relation
            .expected_local_qualification
            .relation_scope,
        EvidenceLocalRelationScope::DifferentRelation
    );
}

#[test]
fn v16_local_absence_can_reject_without_positive_rescue_authority() {
    let manifest = manifest();
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "44_prompt_injection_local_absence")
        .expect("explicit local absence control");
    assert_eq!(
        case.expected_proposal.target_binding,
        EvidenceRelevanceBinding::Unresolved
    );
    assert_eq!(
        case.expected_proposal.relation_binding,
        EvidenceRelevanceBinding::Unresolved
    );
    assert_eq!(
        case.expected_local_qualification.identity_scope,
        EvidenceLocalIdentityScope::TargetAbsent
    );
    assert_eq!(
        case.expected_local_qualification.relation_scope,
        EvidenceLocalRelationScope::RelationAbsent
    );

    let assessment = materialize_evidence_relevance_v11(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&case.expected_local_qualification),
    )
    .expect("materialize");
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );
}

#[test]
fn v16_production_motivating_product_is_not_a_tuning_fixture() {
    let raw = serde_json::to_string(&manifest()).expect("serialize manifest");
    assert!(!raw.to_ascii_lowercase().contains("cloudwatch omni"));
}
