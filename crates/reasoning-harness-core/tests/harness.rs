use reasoning_harness_core::{
    Claim, EpistemicState, HarnessError, ReasoningArtifact, StrictAcceptancePolicy, Verdict,
    run_harness, run_passes,
};

#[test]
fn validates_input_even_when_there_are_no_passes() {
    let artifact = ReasoningArtifact {
        claims: vec![Claim {
            id: "c1".into(),
            statement: "unsupported".into(),
            state: EpistemicState::Supported,
            evidence_ids: vec![],
        }],
        ..Default::default()
    };

    assert!(matches!(
        run_passes(artifact, &[]),
        Err(HarnessError::InvalidInput { .. })
    ));
}

#[test]
fn strict_policy_preserves_unknown_as_a_successful_outcome() {
    let artifact = ReasoningArtifact {
        claims: vec![Claim {
            id: "c1".into(),
            statement: "not enough evidence".into(),
            state: EpistemicState::Unknown,
            evidence_ids: vec![],
        }],
        ..Default::default()
    };

    let outcome = run_harness(artifact, &[], &StrictAcceptancePolicy).unwrap();
    assert_eq!(outcome.verdict, Verdict::Unknown);
}

#[test]
fn strict_policy_rejects_contradicted_state() {
    let artifact = ReasoningArtifact {
        claims: vec![Claim {
            id: "c1".into(),
            statement: "conflict detected".into(),
            state: EpistemicState::Contradicted,
            evidence_ids: vec![],
        }],
        ..Default::default()
    };

    let outcome = run_harness(artifact, &[], &StrictAcceptancePolicy).unwrap();
    assert_eq!(outcome.verdict, Verdict::Reject);
}
