use reasoning_harness_core::{
    SelectiveAbstentionPolicy, SoftDecisionProbe, SoftJudgeDecision, StabilityRiskSignal,
    apply_selective_abstention, assess_soft_decision_stability,
};

fn probe(id: &str, decision: Option<SoftJudgeDecision>) -> SoftDecisionProbe {
    SoftDecisionProbe {
        probe_id: id.into(),
        decision,
    }
}

#[test]
fn unanimous_complete_probes_preserve_the_soft_decision() {
    let assessment = assess_soft_decision_stability(&[
        probe("seed-1", Some(SoftJudgeDecision::Finding)),
        probe("seed-2", Some(SoftJudgeDecision::Finding)),
        probe("seed-3", Some(SoftJudgeDecision::Finding)),
    ]);
    assert_eq!(
        assessment.unanimous_decision,
        Some(SoftJudgeDecision::Finding)
    );
    assert!(assessment.risk_signals.is_empty());
    for policy in [
        SelectiveAbstentionPolicy::DisagreementOnly,
        SelectiveAbstentionPolicy::CompleteUnanimity,
    ] {
        let outcome = apply_selective_abstention(&assessment, policy);
        assert_eq!(outcome.decision, SoftJudgeDecision::Finding);
        assert!(!outcome.escalated_to_abstain);
    }
}

#[test]
fn disagreement_escalates_to_abstain_without_majority_voting() {
    let assessment = assess_soft_decision_stability(&[
        probe("seed-1", Some(SoftJudgeDecision::Finding)),
        probe("seed-2", Some(SoftJudgeDecision::Finding)),
        probe("seed-3", Some(SoftJudgeDecision::Abstain)),
    ]);
    assert!(
        assessment
            .risk_signals
            .contains(&StabilityRiskSignal::DecisionDisagreement)
    );
    assert_eq!(assessment.unanimous_decision, None);
    let outcome =
        apply_selective_abstention(&assessment, SelectiveAbstentionPolicy::DisagreementOnly);
    assert_eq!(outcome.decision, SoftJudgeDecision::Abstain);
    // Two finding votes do not become truth; disagreement forces a conservative abstention.
    assert!(outcome.escalated_to_abstain);
}

#[test]
fn operational_failure_stays_separate_from_semantic_disagreement() {
    let assessment = assess_soft_decision_stability(&[
        probe("seed-1", Some(SoftJudgeDecision::NoFinding)),
        probe("seed-2", None),
        probe("seed-3", Some(SoftJudgeDecision::NoFinding)),
    ]);
    assert_eq!(
        assessment.unanimous_decision,
        Some(SoftJudgeDecision::NoFinding)
    );
    assert!(
        assessment
            .risk_signals
            .contains(&StabilityRiskSignal::OperationalIncomplete)
    );
    assert!(
        !assessment
            .risk_signals
            .contains(&StabilityRiskSignal::DecisionDisagreement)
    );

    let loose =
        apply_selective_abstention(&assessment, SelectiveAbstentionPolicy::DisagreementOnly);
    assert_eq!(loose.decision, SoftJudgeDecision::NoFinding);

    let strict =
        apply_selective_abstention(&assessment, SelectiveAbstentionPolicy::CompleteUnanimity);
    assert_eq!(strict.decision, SoftJudgeDecision::Abstain);
    assert!(strict.escalated_to_abstain);
}

#[test]
fn no_successful_probe_is_never_converted_to_no_finding() {
    let assessment =
        assess_soft_decision_stability(&[probe("seed-1", None), probe("seed-2", None)]);
    assert!(
        assessment
            .risk_signals
            .contains(&StabilityRiskSignal::NoSuccessfulObservation)
    );
    for policy in [
        SelectiveAbstentionPolicy::DisagreementOnly,
        SelectiveAbstentionPolicy::CompleteUnanimity,
    ] {
        assert_eq!(
            apply_selective_abstention(&assessment, policy).decision,
            SoftJudgeDecision::Abstain
        );
    }
}

#[test]
fn unanimous_abstain_is_preserved_not_counted_as_escalation() {
    let assessment = assess_soft_decision_stability(&[
        probe("r1", Some(SoftJudgeDecision::Abstain)),
        probe("r2", Some(SoftJudgeDecision::Abstain)),
    ]);
    let outcome =
        apply_selective_abstention(&assessment, SelectiveAbstentionPolicy::CompleteUnanimity);
    assert_eq!(outcome.decision, SoftJudgeDecision::Abstain);
    assert!(!outcome.escalated_to_abstain);
}
