use reasoning_harness_core::{
    EpistemicState, Evidence, HarnessInput, ReasoningCandidate, StrictAcceptancePolicy,
    TrustedVerificationPass, Verdict, VerificationConclusion, VerificationReceipt, run_harness,
};

fn input() -> HarnessInput {
    HarnessInput {
        task: "What status was observed?".into(),
        evidence: vec![Evidence {
            id: "e1".into(),
            source: "request log".into(),
            observation: "HTTP 503".into(),
            facts: Default::default(),
        }],
        hypotheses: vec![],
        assumptions: vec![],
    }
}

fn candidate() -> ReasoningCandidate {
    serde_json::from_value(serde_json::json!({
        "claims": [{
            "id": "model-id-can-vary",
            "statement": "The observed status code was 503.",
            "proposed_state": "supported",
            "evidence_ids": ["e1"]
        }],
        "inferences": []
    }))
    .unwrap()
}

#[test]
fn trusted_receipt_can_promote_exact_bound_claim() {
    let pass = TrustedVerificationPass::new(vec![VerificationReceipt {
        id: "vr1".into(),
        verifier: "test_oracle".into(),
        claim_statement: Some("The observed status code was 503.".into()),
        proposition: None,
        claim_id: None,
        conclusion: VerificationConclusion::Supported,
        evidence_ids: vec!["e1".into()],
    }]);
    let passes: Vec<Box<dyn reasoning_harness_core::Pass>> = vec![Box::new(pass)];
    let outcome = run_harness(input(), candidate(), &passes, &StrictAcceptancePolicy).unwrap();

    assert_eq!(outcome.verdict, Verdict::Accept);
    assert_eq!(outcome.artifact.claims[0].state, EpistemicState::Supported);
    assert_eq!(outcome.artifact.verification_receipts.len(), 1);
}

#[test]
fn trusted_contradiction_can_reject_exact_bound_claim() {
    let pass = TrustedVerificationPass::new(vec![VerificationReceipt {
        id: "vr1".into(),
        verifier: "test_oracle".into(),
        claim_statement: Some("The observed status code was 503.".into()),
        proposition: None,
        claim_id: None,
        conclusion: VerificationConclusion::Contradicted,
        evidence_ids: vec!["e1".into()],
    }]);
    let passes: Vec<Box<dyn reasoning_harness_core::Pass>> = vec![Box::new(pass)];
    let outcome = run_harness(input(), candidate(), &passes, &StrictAcceptancePolicy).unwrap();

    assert_eq!(outcome.verdict, Verdict::Reject);
    assert_eq!(
        outcome.artifact.claims[0].state,
        EpistemicState::Contradicted
    );
}

#[test]
fn receipt_fails_closed_when_statement_does_not_match() {
    let pass = TrustedVerificationPass::new(vec![VerificationReceipt {
        id: "vr1".into(),
        verifier: "test_oracle".into(),
        claim_statement: Some("A different claim.".into()),
        proposition: None,
        claim_id: None,
        conclusion: VerificationConclusion::Supported,
        evidence_ids: vec!["e1".into()],
    }]);
    let passes: Vec<Box<dyn reasoning_harness_core::Pass>> = vec![Box::new(pass)];
    let error = run_harness(input(), candidate(), &passes, &StrictAcceptancePolicy).unwrap_err();

    assert!(error.to_string().contains("matched 0 claims"));
}
