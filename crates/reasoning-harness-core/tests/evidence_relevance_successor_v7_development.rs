use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v9, materialize_evidence_relevance_v22,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    status: String,
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
        .join("../../fixtures/evidence-relevance-successor-v7-development/manifest.json");
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn successor_v7_development_labels_are_self_consistent() {
    let manifest = load();
    assert_eq!(
        manifest.suite_id,
        "evidence-relevance-successor-v7-development"
    );
    assert_eq!(manifest.status, "reusable_development_calibration");
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v9"
    );
    assert_eq!(
        manifest.fixed_core_id,
        "evidence-relevance-fixed-core-successor-v7-development-v1"
    );
    assert_eq!(manifest.cases.len(), 12);

    for case in &manifest.cases {
        let effective = derive_effective_evidence_local_qualification_v9(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap();
        assert_eq!(
            effective, case.expected_local_qualification,
            "{} effective label",
            case.id
        );

        let assessment = materialize_evidence_relevance_v22(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap();
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{} disposition",
            case.id
        );
    }
}
