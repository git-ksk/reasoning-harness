use std::{collections::BTreeSet, fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceBlockingCue, EvidenceExplicitLocalAbsence, EvidenceLocalQualificationV3,
    EvidenceLocalSupport, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy, materialize_evidence_relevance_v8,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Manifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    annotation_protocol_id: String,
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
    expected_local_qualification: EvidenceLocalQualificationV3,
    expected_disposition: EvidenceRelevanceDisposition,
}

fn manifest() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-calibration-v13/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read calibration manifest"))
        .expect("parse calibration manifest")
}

fn qualification_has_blocking_cue(qualification: &EvidenceLocalQualificationV3) -> bool {
    qualification.identity_mapping_cue == EvidenceBlockingCue::Present
        || qualification.ownership_scope_cue == EvidenceBlockingCue::Present
        || qualification.context_gap_cue == EvidenceBlockingCue::Present
}

#[test]
fn v13_expected_primary_and_guard_materialize_all_cases() {
    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v13");
    assert_eq!(manifest.issue, 462);
    assert_eq!(manifest.status, "fresh_unobserved_calibration");
    assert_eq!(
        manifest.source_rule,
        "synthetic_calibration_v13_atomic_orthogonal_binding_and_observable_blocking_cues_with_preobservation_label_review"
    );
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-atomic-annotation-v13"
    );
    assert_eq!(manifest.cases.len(), 81);

    for case in &manifest.cases {
        assert_eq!(case.policy.assessment_budget.max_model_attempts, 2);
        assert_eq!(case.policy.assessment_budget.max_tokens, 192);
        assert_eq!(case.policy.assessment_budget.max_elapsed_ms, 60_000);
        let assessment = materialize_evidence_relevance_v8(
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
fn v13_contains_eight_fresh_atomic_and_cue_families() {
    let manifest = manifest();
    let fresh = manifest
        .cases
        .iter()
        .filter(|case| {
            case.id
                .split_once('_')
                .and_then(|(prefix, _)| prefix.parse::<usize>().ok())
                .is_some_and(|number| (74..=81).contains(&number))
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
            "v13_fresh_atomic_positive",
            "v13_fresh_context_gap_cue",
            "v13_fresh_explicit_local_absence",
            "v13_fresh_identity_mapping_cue",
            "v13_fresh_ownership_scope_cue",
            "v13_fresh_relation_negative",
            "v13_fresh_target_relation_orthogonal",
        ])
    );
}

#[test]
fn v13_qualification_surface_covers_binary_cues_and_explicit_absence() {
    let manifest = manifest();
    let cue_cases = manifest
        .cases
        .iter()
        .filter(|case| qualification_has_blocking_cue(&case.expected_local_qualification))
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

    assert_eq!(cue_cases, 28);
    assert_eq!(explicit_absence, 9);
    assert_eq!(target_supported, 34);
    assert_eq!(target_not_supported, 24);
    assert_eq!(target_unresolved, 23);
}

#[test]
fn v13_target_and_relation_axes_are_observably_orthogonal() {
    let manifest = manifest();
    let sibling_same_relation = manifest
        .cases
        .iter()
        .find(|case| case.id == "75_v13_sibling_same_relation_no_cue")
        .expect("sibling same-relation control");
    assert_eq!(
        sibling_same_relation.expected_proposal.target_binding,
        reasoning_harness_core::EvidenceRelevanceBinding::Different
    );
    assert_eq!(
        sibling_same_relation.expected_proposal.relation_binding,
        reasoning_harness_core::EvidenceRelevanceBinding::Exact
    );
    assert_eq!(
        sibling_same_relation
            .expected_local_qualification
            .target_support,
        EvidenceLocalSupport::NotSupported
    );
    assert_eq!(
        sibling_same_relation
            .expected_local_qualification
            .relation_support,
        EvidenceLocalSupport::Supported
    );
    assert!(!qualification_has_blocking_cue(
        &sibling_same_relation.expected_local_qualification
    ));
}

#[test]
fn v13_explicit_local_absence_is_not_implicitly_a_context_gap() {
    let manifest = manifest();
    for id in [
        "28_generic_landing_target_absent",
        "29_explicit_no_target_information",
        "77_v13_explicit_generic_local_absence_no_gap",
    ] {
        let case = manifest
            .cases
            .iter()
            .find(|case| case.id == id)
            .unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(
            case.expected_local_qualification.explicit_local_absence,
            EvidenceExplicitLocalAbsence::Present,
            "{id}"
        );
        assert_eq!(
            case.expected_local_qualification.context_gap_cue,
            EvidenceBlockingCue::Absent,
            "{id}"
        );
    }
}

#[test]
fn v13_every_expected_blocking_cue_remains_fail_closed() {
    let manifest = manifest();
    for case in manifest
        .cases
        .iter()
        .filter(|case| qualification_has_blocking_cue(&case.expected_local_qualification))
    {
        assert_eq!(
            case.expected_disposition,
            EvidenceRelevanceDisposition::Ambiguous,
            "{}",
            case.id
        );
    }
}

#[test]
fn v13_annotation_protocol_is_internally_consistent_before_observation() {
    let manifest = manifest();
    for case in &manifest.cases {
        let q = &case.expected_local_qualification;
        let expected_relation_support = match case.expected_proposal.relation_binding {
            reasoning_harness_core::EvidenceRelevanceBinding::Exact => {
                EvidenceLocalSupport::Supported
            }
            reasoning_harness_core::EvidenceRelevanceBinding::Different => {
                EvidenceLocalSupport::NotSupported
            }
            reasoning_harness_core::EvidenceRelevanceBinding::Unresolved => {
                EvidenceLocalSupport::Unresolved
            }
        };
        assert_eq!(
            q.relation_support, expected_relation_support,
            "{} relation axis",
            case.id
        );

        let expected_target_support = match case.expected_proposal.target_binding {
            reasoning_harness_core::EvidenceRelevanceBinding::Exact => {
                Some(EvidenceLocalSupport::Supported)
            }
            reasoning_harness_core::EvidenceRelevanceBinding::Different => {
                Some(EvidenceLocalSupport::NotSupported)
            }
            reasoning_harness_core::EvidenceRelevanceBinding::Unresolved => None,
        };
        if let Some(expected) = expected_target_support {
            assert_eq!(q.target_support, expected, "{} target axis", case.id);
        } else if q.target_support == EvidenceLocalSupport::NotSupported {
            assert_eq!(
                q.explicit_local_absence,
                EvidenceExplicitLocalAbsence::Present,
                "{} unresolved-primary negative guard must be explicit local absence",
                case.id
            );
        } else {
            assert_eq!(
                q.target_support,
                EvidenceLocalSupport::Unresolved,
                "{} unresolved target axis",
                case.id
            );
        }

        if q.explicit_local_absence == EvidenceExplicitLocalAbsence::Present {
            assert_eq!(
                q.target_support,
                EvidenceLocalSupport::NotSupported,
                "{}",
                case.id
            );
            assert_eq!(
                q.relation_support,
                EvidenceLocalSupport::Unresolved,
                "{}",
                case.id
            );
            assert_eq!(
                q.context_gap_cue,
                EvidenceBlockingCue::Absent,
                "{}",
                case.id
            );
        }
    }
}

#[test]
fn v13_production_motivating_product_is_not_a_tuning_fixture() {
    let raw = serde_json::to_string(&manifest()).expect("serialize manifest");
    assert!(!raw.to_ascii_lowercase().contains("cloudwatch omni"));
}
