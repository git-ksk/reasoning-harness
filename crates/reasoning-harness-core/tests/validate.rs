use reasoning_harness_core::{
    Claim, EpistemicState, Evidence, ReasoningArtifact, validate_artifact,
};

#[test]
fn accepts_an_evidence_backed_known_claim() {
    let artifact = ReasoningArtifact {
        task: "fixture task".into(),
        evidence: vec![Evidence {
            id: "e1".into(),
            source: "fixture".into(),
            observation: "observed".into(),
            facts: Default::default(),
        }],
        claims: vec![Claim {
            id: "c1".into(),
            statement: "supported".into(),
            state: EpistemicState::Known,
            proposition: None,
            evidence_ids: vec!["e1".into()],
        }],
        ..Default::default()
    };
    assert!(validate_artifact(&artifact).is_ok());
}

#[test]
fn rejects_a_supported_claim_without_evidence() {
    let artifact = ReasoningArtifact {
        task: "fixture task".into(),
        claims: vec![Claim {
            id: "c1".into(),
            statement: "unsupported".into(),
            state: EpistemicState::Supported,
            proposition: None,
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
        task: "fixture task".into(),
        claims: vec![Claim {
            id: "c1".into(),
            statement: "bad ref".into(),
            state: EpistemicState::Inferred,
            proposition: None,
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

#[test]
fn rejects_an_inferred_claim_without_an_inference_edge() {
    let artifact = ReasoningArtifact {
        task: "fixture task".into(),
        claims: vec![Claim {
            id: "c1".into(),
            statement: "derived conclusion".into(),
            state: EpistemicState::Inferred,
            proposition: None,
            evidence_ids: vec![],
        }],
        ..Default::default()
    };

    assert!(
        validate_artifact(&artifact)
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "inferred_claim_without_inference")
    );
}

#[test]
fn rejects_invalid_harness_hypothesis() {
    let artifact = ReasoningArtifact {
        task: "fixture task".into(),
        hypotheses: vec![reasoning_harness_core::Proposition {
            key: "".into(),
            value: "true".into(),
        }],
        ..Default::default()
    };
    assert!(
        validate_artifact(&artifact)
            .diagnostics
            .iter()
            .any(|d| d.code == "invalid_hypothesis")
    );
}

#[test]
fn rejects_invalid_and_duplicate_harness_assumptions() {
    let artifact = ReasoningArtifact {
        task: "fixture task".into(),
        assumptions: vec![
            reasoning_harness_core::Proposition {
                key: "".into(),
                value: "true".into(),
            },
            reasoning_harness_core::Proposition {
                key: "scope.mode".into(),
                value: "test".into(),
            },
            reasoning_harness_core::Proposition {
                key: "scope.mode".into(),
                value: "test".into(),
            },
        ],
        ..Default::default()
    };
    let report = validate_artifact(&artifact);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "invalid_input_assumption")
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "duplicate_input_assumption")
    );
}
