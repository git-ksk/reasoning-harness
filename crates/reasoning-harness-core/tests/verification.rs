use std::collections::BTreeMap;

use reasoning_harness_core::{
    EpistemicState, Evidence, EvidenceQualificationPass, EvidenceRequirement, HarnessInput,
    Proposition, ReasoningCandidate, StrictAcceptancePolicy, TemporalValidity,
    TrustedVerificationPass, Verdict, VerificationConclusion, VerificationPass,
    VerificationReceipt, run_harness, structured_fact_verifier_for_input,
};

fn input() -> HarnessInput {
    HarnessInput {
        task: "What status was observed?".into(),
        evidence: vec![Evidence {
            id: "e1".into(),
            source: "request log".into(),
            observation: "HTTP 503".into(),
            facts: Default::default(),
            metadata: Default::default(),
        }],
        hypotheses: vec![],
        assumptions: vec![],
        evidence_requirements: vec![],
        authority_policy: Default::default(),
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

#[test]
fn stale_structured_fact_cannot_create_a_hard_receipt_when_qualification_is_required() {
    let input = HarnessInput {
        task: "What status is valid at the requested time?".into(),
        evidence: vec![Evidence {
            id: "e1".into(),
            source: "request log".into(),
            observation: "HTTP 503".into(),
            facts: BTreeMap::from([("http.status_code".into(), "503".into())]),
            metadata: reasoning_harness_core::EvidenceMetadata {
                temporal: Some(TemporalValidity {
                    effective_from_unix_seconds: Some(0),
                    effective_until_unix_seconds: Some(100),
                }),
                ..Default::default()
            },
        }],
        hypotheses: vec![Proposition {
            key: "http.status_code".into(),
            value: "503".into(),
        }],
        assumptions: vec![],
        evidence_requirements: vec![EvidenceRequirement {
            proposition: Proposition {
                key: "http.status_code".into(),
                value: "503".into(),
            },
            as_of_unix_seconds: Some(200),
            scope: None,
            minimum_authority_class: None,
        }],
        authority_policy: Default::default(),
    };
    let candidate: ReasoningCandidate = serde_json::from_value(serde_json::json!({
        "claims": [{
            "id": "c1",
            "statement": "The status is 503.",
            "proposed_state": "supported",
            "proposition": {"key": "http.status_code", "value": "503"},
            "evidence_ids": ["e1"]
        }],
        "inferences": []
    }))
    .unwrap();
    let verifier = structured_fact_verifier_for_input(&input);
    let passes: Vec<Box<dyn reasoning_harness_core::Pass>> = vec![
        Box::new(EvidenceQualificationPass),
        Box::new(VerificationPass::new(vec![verifier])),
    ];
    let outcome = run_harness(input, candidate, &passes, &StrictAcceptancePolicy).unwrap();

    assert_eq!(outcome.verdict, Verdict::Unknown);
    assert_eq!(outcome.artifact.claims[0].state, EpistemicState::Assumed);
    assert!(outcome.artifact.verification_receipts.is_empty());
    assert!(
        outcome
            .artifact
            .evidence_qualification_findings
            .iter()
            .any(|finding| {
                finding.reason == reasoning_harness_core::EvidenceQualificationFindingReason::Stale
            })
    );
}
