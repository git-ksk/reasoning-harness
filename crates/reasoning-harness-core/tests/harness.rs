use reasoning_harness_core::{
    CandidateClaim, EpistemicState, Evidence, HarnessError, HarnessInput, ReasoningArtifact,
    ReasoningCandidate, StrictAcceptancePolicy, Verdict, run_harness, run_passes,
};

#[test]
fn validates_input_even_when_there_are_no_passes() {
    let artifact = ReasoningArtifact {
        task: "test invalid input".into(),
        claims: vec![reasoning_harness_core::Claim {
            id: "c1".into(),
            statement: "unsupported".into(),
            state: EpistemicState::Supported,
            proposition: None,
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
    let input = HarnessInput {
        task: "answer only when evidence is sufficient".into(),
        evidence: vec![],
        hypotheses: vec![],
        assumptions: vec![],
    };
    let candidate = ReasoningCandidate {
        claims: vec![CandidateClaim {
            id: "c1".into(),
            statement: "not enough evidence".into(),
            proposed_state: EpistemicState::Unknown,
            proposition: None,
            evidence_ids: vec![],
        }],
        inferences: vec![],
    };

    let outcome = run_harness(input, candidate, &[], &StrictAcceptancePolicy).unwrap();
    assert_eq!(outcome.verdict, Verdict::Unknown);
}

#[test]
fn model_proposed_contradiction_cannot_force_runtime_reject() {
    let input = HarnessInput {
        task: "check a proposed contradiction".into(),
        evidence: vec![Evidence {
            id: "e1".into(),
            source: "fixture".into(),
            observation: "fact".into(),
            facts: Default::default(),
        }],
        hypotheses: vec![],
        assumptions: vec![],
    };
    let candidate = ReasoningCandidate {
        claims: vec![CandidateClaim {
            id: "c1".into(),
            statement: "conflict detected".into(),
            proposed_state: EpistemicState::Contradicted,
            proposition: None,
            evidence_ids: vec!["e1".into()],
        }],
        inferences: vec![],
    };

    let outcome = run_harness(input, candidate, &[], &StrictAcceptancePolicy).unwrap();
    assert_eq!(outcome.verdict, Verdict::Unknown);
    assert_eq!(outcome.artifact.claims[0].state, EpistemicState::Assumed);
}
