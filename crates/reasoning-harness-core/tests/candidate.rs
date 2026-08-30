use reasoning_harness_core::{
    CandidateClaim, EpistemicState, Evidence, HarnessInput, ReasoningCandidate,
    materialize_candidate,
};

#[test]
fn model_cannot_promote_its_own_claim_to_supported() {
    let input = HarnessInput {
        task: "decide from supplied evidence".into(),
        evidence: vec![Evidence {
            id: "e1".into(),
            source: "fixture".into(),
            observation: "observed fact".into(),
            facts: Default::default(),
        }],
        hypotheses: vec![],
        assumptions: vec![],
    };
    let candidate = ReasoningCandidate {
        claims: vec![CandidateClaim {
            id: "c1".into(),
            statement: "candidate assertion".into(),
            proposed_state: EpistemicState::Supported,
            proposition: None,
            evidence_ids: vec!["e1".into()],
        }],
        inferences: vec![],
    };

    let artifact = materialize_candidate(input, candidate);
    assert_eq!(artifact.claims[0].state, EpistemicState::Assumed);
}

#[test]
fn model_cannot_inject_evidence_into_harness_owned_input() {
    let input = HarnessInput {
        task: "decide from supplied evidence".into(),
        evidence: vec![],
        hypotheses: vec![],
        assumptions: vec![],
    };
    let candidate = ReasoningCandidate {
        claims: vec![CandidateClaim {
            id: "c1".into(),
            statement: "candidate assertion".into(),
            proposed_state: EpistemicState::Supported,
            proposition: None,
            evidence_ids: vec!["invented".into()],
        }],
        inferences: vec![],
    };

    let artifact = materialize_candidate(input, candidate);
    assert!(artifact.evidence.is_empty());
}

#[test]
fn candidate_schema_does_not_allow_model_owned_evidence() {
    let schema = reasoning_harness_core::reasoning_candidate_schema();
    let schema_text = serde_json::to_string(&schema).unwrap();
    assert!(!schema_text.contains("\"evidence\""));
    assert!(!schema_text.contains("CausalEvidence"));
    assert!(!schema_text.contains("CausalFinding"));
    assert!(!schema_text.contains("verification_receipts"));
    assert!(!schema_text.contains("\"assumptions\""));
    assert!(!schema_text.contains("AssumptionFinding"));
    assert!(schema_text.contains("proposed_state"));
}
