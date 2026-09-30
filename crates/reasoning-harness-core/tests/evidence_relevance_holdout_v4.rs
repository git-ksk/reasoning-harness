use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v6, materialize_evidence_relevance_v19,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    status: String,
    source_rule: String,
    annotation_protocol_id: String,
    fixed_core_id: String,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: EvidenceRelevanceBindingProposal,
    expected_local_qualification: EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

fn load() -> Manifest {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/evidence-relevance-holdout-v4/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read holdout v4 manifest"))
        .expect("parse holdout v4 manifest")
}

#[test]
fn holdout_v4_identity_and_distribution_are_frozen_preobservation() {
    let manifest = load();
    assert_eq!(manifest.suite_id, "evidence-relevance-holdout-v4");
    assert_eq!(manifest.status, "fresh_unobserved_holdout");
    assert_eq!(
        manifest.source_rule,
        "independently_authored_after_successor_v4_semantics_freeze_no_holdout_v1_v2_v3_case_entity_or_surface_reuse"
    );
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v6"
    );
    assert_eq!(manifest.fixed_core_id, "evidence-relevance-fixed-core-v4");
    assert_eq!(manifest.cases.len(), 26);

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
    assert_eq!((relevant, irrelevant, ambiguous), (8, 10, 8));
}

#[test]
fn holdout_v4_expected_observations_are_exact_under_frozen_v6_v19_semantics() {
    for case in load().cases {
        let effective = derive_effective_evidence_local_qualification_v6(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} effective qualification: {error}", case.id));
        assert_eq!(
            effective, case.expected_local_qualification,
            "{} effective qualification",
            case.id
        );

        let assessment = materialize_evidence_relevance_v19(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} materialization: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{} materialization",
            case.id
        );
    }
}
