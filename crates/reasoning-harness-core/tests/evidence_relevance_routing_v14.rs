use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceExplicitLocalAbsence, EvidenceLocalBlockingReason, EvidenceLocalQualificationV4,
    EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy, materialize_evidence_relevance_v9,
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
    expected_local_qualification: EvidenceLocalQualificationV4,
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
        .join("../../fixtures/evidence-relevance-calibration-v14/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read v14 calibration manifest"))
        .expect("parse v14 calibration manifest")
}

fn fixed_core() -> FixedCore {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-core-v1/case-ids.json");
    serde_json::from_slice(&fs::read(path).expect("read fixed core")).expect("parse fixed core")
}

#[test]
fn v14_fixed_core_identity_and_case_order_are_exact() {
    let manifest = manifest();
    let core = fixed_core();

    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v14");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "fixed_core_v1_semantic_coverage_compaction_no_case_growth"
    );
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-compact-guard-v14"
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

    let manifest_ids = manifest
        .cases
        .iter()
        .map(|case| case.id.clone())
        .collect::<Vec<_>>();
    assert_eq!(manifest_ids, core.case_ids);
    assert_eq!(
        manifest_ids.iter().collect::<BTreeSet<_>>().len(),
        manifest_ids.len()
    );
}

#[test]
fn v14_expected_proposal_and_compact_guard_materialize_all_48_cases() {
    let manifest = manifest();

    for case in &manifest.cases {
        assert_eq!(case.policy.assessment_budget.max_model_attempts, 2);
        assert_eq!(case.policy.assessment_budget.max_tokens, 192);
        assert_eq!(case.policy.assessment_budget.max_elapsed_ms, 60_000);

        let assessment = materialize_evidence_relevance_v9(
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
fn v14_fixed_core_has_balanced_disposition_and_blocker_coverage() {
    let manifest = manifest();

    let relevant = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Relevant)
        .count();
    let irrelevant = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Irrelevant)
        .count();
    let ambiguous = manifest
        .cases
        .iter()
        .filter(|case| case.expected_disposition == EvidenceRelevanceDisposition::Ambiguous)
        .count();

    assert_eq!((relevant, irrelevant, ambiguous), (14, 18, 16));

    let count_reason = |reason| {
        manifest
            .cases
            .iter()
            .filter(|case| case.expected_local_qualification.blocking_reason == reason)
            .count()
    };
    assert_eq!(count_reason(EvidenceLocalBlockingReason::None), 32);
    assert_eq!(
        count_reason(EvidenceLocalBlockingReason::IdentityMapping),
        5
    );
    assert_eq!(count_reason(EvidenceLocalBlockingReason::OwnershipScope), 1);
    assert_eq!(count_reason(EvidenceLocalBlockingReason::ContextGap), 5);
    assert_eq!(count_reason(EvidenceLocalBlockingReason::Multiple), 5);

    let explicit_absence = manifest
        .cases
        .iter()
        .filter(|case| {
            case.expected_local_qualification.explicit_local_absence
                == EvidenceExplicitLocalAbsence::Present
        })
        .count();
    assert_eq!(explicit_absence, 4);
}

#[test]
fn v14_every_blocking_reason_is_fail_closed_to_ambiguous() {
    let manifest = manifest();

    for case in manifest.cases.iter().filter(|case| {
        case.expected_local_qualification.blocking_reason != EvidenceLocalBlockingReason::None
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
fn v14_explicit_local_absence_is_negative_not_a_blocker() {
    let manifest = manifest();

    for case in manifest.cases.iter().filter(|case| {
        case.expected_local_qualification.explicit_local_absence
            == EvidenceExplicitLocalAbsence::Present
    }) {
        assert_eq!(
            case.expected_local_qualification.blocking_reason,
            EvidenceLocalBlockingReason::None,
            "{}",
            case.id
        );
        assert_eq!(
            case.expected_disposition,
            EvidenceRelevanceDisposition::Irrelevant,
            "{}",
            case.id
        );
    }
}

#[test]
fn v14_target_and_relation_axes_remain_orthogonal() {
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

    let exact_wrong_relation = manifest
        .cases
        .iter()
        .find(|case| case.id == "13_same_service_different_feature")
        .expect("exact target wrong relation");
    assert_eq!(
        exact_wrong_relation.expected_proposal.target_binding,
        EvidenceRelevanceBinding::Exact
    );
    assert_eq!(
        exact_wrong_relation.expected_proposal.relation_binding,
        EvidenceRelevanceBinding::Different
    );

    let sibling_wrong_relation = manifest
        .cases
        .iter()
        .find(|case| case.id == "76_v13_sibling_different_relation_no_cue")
        .expect("sibling wrong relation");
    assert_eq!(
        sibling_wrong_relation.expected_proposal.target_binding,
        EvidenceRelevanceBinding::Different
    );
    assert_eq!(
        sibling_wrong_relation.expected_proposal.relation_binding,
        EvidenceRelevanceBinding::Different
    );
}

#[test]
fn v14_each_compact_blocker_label_has_an_isolated_control() {
    let manifest = manifest();

    for reason in [
        EvidenceLocalBlockingReason::IdentityMapping,
        EvidenceLocalBlockingReason::OwnershipScope,
        EvidenceLocalBlockingReason::ContextGap,
    ] {
        assert!(
            manifest
                .cases
                .iter()
                .any(|case| case.expected_local_qualification.blocking_reason == reason),
            "missing isolated blocker {reason:?}"
        );
    }
}

#[test]
fn v14_production_motivating_product_is_not_a_tuning_fixture() {
    let raw = serde_json::to_string(&manifest()).expect("serialize manifest");
    assert!(!raw.to_ascii_lowercase().contains("cloudwatch omni"));
}
