use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::SoftJudgeDecision;

/// One bounded research probe for the same semantic request.
///
/// `None` is operational incompleteness, never a semantic decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftDecisionProbe {
    pub probe_id: String,
    pub decision: Option<SoftJudgeDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StabilityRiskSignal {
    DecisionDisagreement,
    OperationalIncomplete,
    NoSuccessfulObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftDecisionStabilityAssessment {
    pub expected_probes: usize,
    pub successful_probes: usize,
    pub failed_probes: usize,
    pub observed_decisions: Vec<SoftJudgeDecision>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unanimous_decision: Option<SoftJudgeDecision>,
    pub risk_signals: Vec<StabilityRiskSignal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectiveAbstentionPolicy {
    /// Escalate only semantic disagreement. Operational incompleteness remains visible but does
    /// not itself change a unanimous successful decision.
    DisagreementOnly,
    /// Require every configured probe to succeed and agree. Missing probes are risk, not evidence,
    /// and conservatively produce `abstain`.
    CompleteUnanimity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectiveAbstentionOutcome {
    pub policy: SelectiveAbstentionPolicy,
    pub decision: SoftJudgeDecision,
    pub escalated_to_abstain: bool,
    pub risk_signals: Vec<StabilityRiskSignal>,
}

pub fn assess_soft_decision_stability(
    probes: &[SoftDecisionProbe],
) -> SoftDecisionStabilityAssessment {
    let expected_probes = probes.len();
    let successful_probes = probes
        .iter()
        .filter(|probe| probe.decision.is_some())
        .count();
    let failed_probes = expected_probes - successful_probes;
    let observed = probes
        .iter()
        .filter_map(|probe| probe.decision)
        .collect::<BTreeSet<_>>();
    let observed_decisions = observed.iter().copied().collect::<Vec<_>>();
    let unanimous_decision = (observed.len() == 1).then(|| *observed.iter().next().unwrap());

    let mut risk_signals = BTreeSet::new();
    if observed.len() > 1 {
        risk_signals.insert(StabilityRiskSignal::DecisionDisagreement);
    }
    if failed_probes > 0 {
        risk_signals.insert(StabilityRiskSignal::OperationalIncomplete);
    }
    if successful_probes == 0 {
        risk_signals.insert(StabilityRiskSignal::NoSuccessfulObservation);
    }

    SoftDecisionStabilityAssessment {
        expected_probes,
        successful_probes,
        failed_probes,
        observed_decisions,
        unanimous_decision,
        risk_signals: risk_signals.into_iter().collect(),
    }
}

pub fn apply_selective_abstention(
    assessment: &SoftDecisionStabilityAssessment,
    policy: SelectiveAbstentionPolicy,
) -> SelectiveAbstentionOutcome {
    let disagreement = assessment
        .risk_signals
        .contains(&StabilityRiskSignal::DecisionDisagreement);
    let incomplete = assessment
        .risk_signals
        .contains(&StabilityRiskSignal::OperationalIncomplete);
    let no_success = assessment
        .risk_signals
        .contains(&StabilityRiskSignal::NoSuccessfulObservation);

    let must_abstain = disagreement
        || no_success
        || (policy == SelectiveAbstentionPolicy::CompleteUnanimity && incomplete);
    let decision = if must_abstain {
        SoftJudgeDecision::Abstain
    } else {
        assessment
            .unanimous_decision
            .unwrap_or(SoftJudgeDecision::Abstain)
    };
    let escalated_to_abstain = decision == SoftJudgeDecision::Abstain
        && !no_success
        && (disagreement || (policy == SelectiveAbstentionPolicy::CompleteUnanimity && incomplete))
        && assessment.unanimous_decision != Some(SoftJudgeDecision::Abstain);

    SelectiveAbstentionOutcome {
        policy,
        decision,
        escalated_to_abstain,
        risk_signals: assessment.risk_signals.clone(),
    }
}
