use reasoning_harness_core::{
    Claim, EpistemicState, Evidence, ReasoningArtifact, validate_artifact,
};

#[test]
fn accepts_an_evidence_backed_known_claim() {
    let artifact = ReasoningArtifact {
        evidence: vec![Evidence {
            id: "e1".into(),
            source: "fixture".into(),
            observation: "observed".into(),
        }],
        claims: vec![Claim {
            id: "c1".into(),
            statement: "supported".into(),
            state: EpistemicState::Known,
            evidence_ids: vec!["e1".into()],
        }],
        ..Default::default()
    };
    assert!(validate_artifact(&artifact).is_ok());
}

#[test]
fn rejects_a_supported_claim_without_evidence() {
    let artifact = ReasoningArtifact {
        claims: vec![Claim {
            id: "c1".into(),
            statement: "unsupported".into(),
            state: EpistemicState::Supported,
            evidence_ids: vec![],
        }],
        ..Default::default()
    };
    let report = validate_artifact(&artifact);
    assert!(!report.is_ok());
    assert_eq!(
        report.diagnostics[0].code,
        "accepted_claim_without_evidence"
    );
}

#[test]
fn rejects_references_to_missing_evidence() {
    let artifact = ReasoningArtifact {
        claims: vec![Claim {
            id: "c1".into(),
            statement: "bad ref".into(),
            state: EpistemicState::Inferred,
            evidence_ids: vec!["missing".into()],
        }],
        ..Default::default()
    };
    assert!(
        validate_artifact(&artifact)
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "missing_evidence_reference")
    );
}
