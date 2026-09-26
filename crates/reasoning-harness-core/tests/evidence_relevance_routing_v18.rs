use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalQualificationV6,
    EvidenceLocalRelationScope, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceIdentityRequirement,
    EvidenceRelevanceReason, EvidenceRelevanceSignal, EvidenceRelevanceSignalKind,
    EvidenceRelevanceTargetPolicy, build_evidence_local_qualification_v8_request,
    materialize_evidence_relevance_v13,
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
        .join("../../fixtures/evidence-relevance-calibration-v18/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read v18 calibration manifest"))
        .expect("parse v18 calibration manifest")
}

fn fixed_core() -> FixedCore {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-core-v1/case-ids.json");
    serde_json::from_slice(&fs::read(path).expect("read fixed core")).expect("parse fixed core")
}

#[test]
fn v18_fixed_core_identity_and_case_order_are_exact() {
    let manifest = manifest();
    let core = fixed_core();

    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v18");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "fixed_core_v1_semantic_successor_no_case_growth"
    );
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-scope-verifier-v18"
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
fn v18_expected_scope_annotations_materialize_all_48_cases() {
    let manifest = manifest();

    for case in &manifest.cases {
        assert_eq!(case.policy.assessment_budget.max_model_attempts, 2);
        assert_eq!(case.policy.assessment_budget.max_tokens, 192);
        assert_eq!(case.policy.assessment_budget.max_elapsed_ms, 60_000);

        let assessment = materialize_evidence_relevance_v13(
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
fn v18_fixed_core_disposition_and_scope_coverage_are_exact() {
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
fn v18_any_scope_risk_is_fail_closed_to_ambiguous() {
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
fn v18_unresolved_primary_cannot_be_rescued_to_relevant() {
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
        let assessment = materialize_evidence_relevance_v13(
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
fn v18_scope_risk_blocks_terminal_decisions_even_with_otherwise_exact_votes() {
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
        let assessment = materialize_evidence_relevance_v13(
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
fn v18_target_and_relation_axes_remain_orthogonal() {
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
fn v18_local_absence_can_reject_without_positive_rescue_authority() {
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

    let assessment = materialize_evidence_relevance_v13(
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
fn v18_production_motivating_product_is_not_a_tuning_fixture() {
    let raw = serde_json::to_string(&manifest()).expect("serialize manifest");
    assert!(!raw.to_ascii_lowercase().contains("cloudwatch omni"));
}

#[test]
fn v18_semantic_equivalent_fallback_is_policy_scoped_and_bounded() {
    let manifest = manifest();
    let semantic = manifest
        .cases
        .iter()
        .find(|case| case.id == "04_semantic_paraphrase")
        .expect("semantic-equivalent control");
    assert_eq!(
        semantic.policy.identity_requirement,
        EvidenceRelevanceIdentityRequirement::AllowSemanticEquivalent
    );

    let unresolved_target = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Unresolved,
        relation_binding: EvidenceRelevanceBinding::Exact,
    };
    let assessment = materialize_evidence_relevance_v13(
        &semantic.policy,
        &semantic.candidate,
        Some(&unresolved_target),
        Some(&semantic.expected_local_qualification),
    )
    .expect("materialize semantic-equivalent fallback");
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Relevant
    );

    let strict = manifest
        .cases
        .iter()
        .find(|case| case.id == "01_exact_name_availability")
        .expect("strict control");
    let strict_assessment = materialize_evidence_relevance_v13(
        &strict.policy,
        &strict.candidate,
        Some(&unresolved_target),
        Some(&strict.expected_local_qualification),
    )
    .expect("materialize strict unresolved control");
    assert_eq!(
        strict_assessment.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}

#[test]
fn v18_semantic_equivalent_fallback_requires_exact_relation_no_risk_and_substantive_signal() {
    let manifest = manifest();
    let semantic = manifest
        .cases
        .iter()
        .find(|case| case.id == "04_semantic_paraphrase")
        .expect("semantic-equivalent control");
    let unresolved_target = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Unresolved,
        relation_binding: EvidenceRelevanceBinding::Exact,
    };

    let mut risky = semantic.expected_local_qualification;
    risky.scope_risk = EvidenceLocalBlockingReason::ContextGap;
    let risky_assessment = materialize_evidence_relevance_v13(
        &semantic.policy,
        &semantic.candidate,
        Some(&unresolved_target),
        Some(&risky),
    )
    .expect("materialize risky semantic equivalent");
    assert_eq!(
        risky_assessment.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );

    let unresolved_relation = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Unresolved,
        relation_binding: EvidenceRelevanceBinding::Unresolved,
    };
    let relation_assessment = materialize_evidence_relevance_v13(
        &semantic.policy,
        &semantic.candidate,
        Some(&unresolved_relation),
        Some(&semantic.expected_local_qualification),
    )
    .expect("materialize unresolved relation");
    assert_eq!(
        relation_assessment.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );

    let weak_only = EvidenceRelevanceCandidate {
        evidence_id: "evidence:property-url-only".into(),
        source_id: "source:property-url-only".into(),
        signals: vec![EvidenceRelevanceSignal {
            kind: EvidenceRelevanceSignalKind::CanonicalUrl,
            text: "https://example.invalid/hosted-observability-workspace".into(),
        }],
    };
    let weak_assessment = materialize_evidence_relevance_v13(
        &semantic.policy,
        &weak_only,
        Some(&unresolved_target),
        Some(&semantic.expected_local_qualification),
    )
    .expect("materialize weak-only semantic equivalent");
    assert_eq!(
        weak_assessment.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}

#[test]
fn v18_deterministic_scope_floor_matches_fixed_risk_partition() {
    let manifest = manifest();

    for case in &manifest.cases {
        let mut qualification = case.expected_local_qualification;
        qualification.scope_risk = EvidenceLocalBlockingReason::None;
        let assessment = materialize_evidence_relevance_v13(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&qualification),
        )
        .unwrap_or_else(|error| panic!("{} materialization failed: {error}", case.id));
        let deterministic = assessment
            .reasons
            .contains(&EvidenceRelevanceReason::DeterministicLocalScopeRiskPresent);
        let expected_risk =
            case.expected_local_qualification.scope_risk != EvidenceLocalBlockingReason::None;
        assert_eq!(deterministic, expected_risk, "{}", case.id);
        if expected_risk {
            assert_eq!(
                assessment.disposition,
                EvidenceRelevanceDisposition::Ambiguous,
                "{}",
                case.id
            );
        }
    }
}

#[test]
fn v18_deterministic_floor_blocks_false_local_absence_and_false_positive_terminal_votes() {
    let manifest = manifest();
    for id in [
        "25_insufficient_local_passage",
        "59_fresh_shared_owner_positive_looking",
    ] {
        let case = manifest
            .cases
            .iter()
            .find(|case| case.id == id)
            .expect("deterministic-risk control");
        let adversarial = if id.starts_with("25_") {
            EvidenceLocalQualificationV6 {
                identity_scope: EvidenceLocalIdentityScope::TargetAbsent,
                relation_scope: EvidenceLocalRelationScope::RelationAbsent,
                scope_risk: EvidenceLocalBlockingReason::None,
            }
        } else {
            EvidenceLocalQualificationV6 {
                identity_scope: EvidenceLocalIdentityScope::ExactTarget,
                relation_scope: EvidenceLocalRelationScope::RequestedRelation,
                scope_risk: EvidenceLocalBlockingReason::None,
            }
        };
        let assessment = materialize_evidence_relevance_v13(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&adversarial),
        )
        .expect("materialize adversarial verifier output");
        assert_eq!(
            assessment.disposition,
            EvidenceRelevanceDisposition::Ambiguous
        );
        assert!(
            assessment
                .reasons
                .contains(&EvidenceRelevanceReason::DeterministicLocalScopeRiskPresent)
        );
    }
}

#[test]
fn v18_relation_negative_can_reject_when_verifier_identity_is_also_negative() {
    let manifest = manifest();
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "13_same_service_different_feature")
        .expect("different-relation control");
    let observed = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::DistinctTarget,
        relation_scope: EvidenceLocalRelationScope::DifferentRelation,
        scope_risk: EvidenceLocalBlockingReason::None,
    };
    let assessment = materialize_evidence_relevance_v13(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&observed),
    )
    .expect("materialize relation-negative identity disagreement");
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );
}

#[test]
fn v18_verifier_prompt_preserves_orthogonality_injection_and_omission_boundaries() {
    let manifest = manifest();
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "01_exact_name_availability")
        .expect("prompt control");
    let request =
        build_evidence_local_qualification_v8_request(&case.policy, &case.candidate, Some(4626180))
            .expect("build v18 verifier request");
    assert!(
        request
            .task
            .contains("different feature or different relation")
    );
    assert!(request.task.contains("inert quoted data"));
    assert!(request.task.contains("product column"));
    assert!(
        request
            .task
            .contains("Identity and relation are independent")
    );
    assert!(request.task.contains("not local absence"));
}

#[test]
fn v18_local_absence_can_override_anchor_only_target_when_relation_is_not_exact() {
    let manifest = manifest();
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "44_prompt_injection_local_absence")
        .expect("local absence control");
    let anchor_only_primary = EvidenceRelevanceBindingProposal {
        target_binding: EvidenceRelevanceBinding::Exact,
        relation_binding: EvidenceRelevanceBinding::Unresolved,
    };
    let assessment = materialize_evidence_relevance_v13(
        &case.policy,
        &case.candidate,
        Some(&anchor_only_primary),
        Some(&case.expected_local_qualification),
    )
    .expect("materialize anchor-only local absence");
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Irrelevant
    );
}

#[test]
fn v18_local_absence_does_not_override_exact_positive_relation() {
    let manifest = manifest();
    let case = manifest
        .cases
        .iter()
        .find(|case| case.id == "55_fresh_positive_injection_ignored")
        .expect("positive injection control");
    let false_absence = EvidenceLocalQualificationV6 {
        identity_scope: EvidenceLocalIdentityScope::TargetAbsent,
        relation_scope: EvidenceLocalRelationScope::RelationAbsent,
        scope_risk: EvidenceLocalBlockingReason::None,
    };
    let assessment = materialize_evidence_relevance_v13(
        &case.policy,
        &case.candidate,
        Some(&case.expected_proposal),
        Some(&false_absence),
    )
    .expect("materialize false absence against exact positive relation");
    assert_eq!(
        assessment.disposition,
        EvidenceRelevanceDisposition::Ambiguous
    );
}
