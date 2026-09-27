use std::{fs, path::PathBuf};

use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy,
    derive_effective_evidence_local_qualification_v2, materialize_evidence_relevance_v16,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
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

fn fixture_path(version: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../fixtures/evidence-relevance-calibration-{version}/manifest.json"
    ))
}

fn manifest() -> Manifest {
    serde_json::from_slice(&fs::read(fixture_path("v22")).expect("read v22 manifest"))
        .expect("parse v22 manifest")
}

#[test]
fn v22_is_semantically_identical_to_frozen_v21_cases() {
    let v21: Value =
        serde_json::from_slice(&fs::read(fixture_path("v21")).expect("read frozen v21 manifest"))
            .expect("parse frozen v21 manifest");
    let v22: Value =
        serde_json::from_slice(&fs::read(fixture_path("v22")).expect("read v22 manifest"))
            .expect("parse v22 manifest");

    assert_eq!(v21["cases"], v22["cases"]);
    assert_eq!(v22["cases"].as_array().expect("v22 cases").len(), 48);
}

#[test]
fn v22_reuses_v21_semantic_contract_and_fixed_core() {
    let manifest = manifest();
    assert_eq!(manifest.suite_id, "evidence-relevance-calibration-v22");
    assert_eq!(
        manifest.annotation_protocol_id,
        "evidence-relevance-effective-qualification-v21"
    );
    assert_eq!(manifest.fixed_core_id, "evidence-relevance-fixed-core-v1");
    assert_eq!(manifest.cases.len(), 48);
}

#[test]
fn v22_expected_cases_remain_exact_under_v21_semantics() {
    for case in manifest().cases {
        let effective = derive_effective_evidence_local_qualification_v2(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} qualification: {error}", case.id));
        assert_eq!(effective, case.expected_local_qualification, "{}", case.id);

        let assessment = materialize_evidence_relevance_v16(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&case.expected_local_qualification),
        )
        .unwrap_or_else(|error| panic!("{} materialization: {error}", case.id));
        assert_eq!(
            assessment.disposition, case.expected_disposition,
            "{}",
            case.id
        );
    }
}
