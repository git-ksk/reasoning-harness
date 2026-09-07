use std::collections::{BTreeMap, BTreeSet};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ModelOutputFormat, ModelRequest};

pub const INVESTIGATION_PLAN_CONTRACT_ID: &str = "reason-investigation-plan-v1";
pub const INVESTIGATION_ACTION_CONTRACT_ID: &str = "reason-investigation-action-v1";
pub const INVESTIGATION_RUNTIME_ID: &str = "bounded-investigation-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvestigationPlanProposal {
    pub targets: Vec<InvestigationTargetProposal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvestigationTargetProposal {
    pub id: String,
    pub question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_fact_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationTarget {
    pub id: String,
    pub question: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_fact_key: Option<String>,
    pub origin: InvestigationTargetOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationTargetOrigin {
    ModelProposedUntrusted,
    ExplicitHarnessInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationCapability {
    pub id: String,
    pub adapter: String,
    pub read_only: bool,
    #[serde(default)]
    pub supported_fact_keys: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationActionKind {
    Acquire,
    Stop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InvestigationActionProposal {
    pub action: InvestigationActionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationAction {
    pub action_index: usize,
    pub target_id: String,
    pub capability_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationActionRejection {
    Stopped,
    InvalidShape,
    UnknownTarget,
    UnknownCapability,
    CapabilityNotReadOnly,
    UnsupportedTargetKey,
    DuplicateAction,
    ActionBudgetExhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationObservationStatus {
    AppliedEvidence,
    RejectedEvidence,
    NoResult,
    Ambiguous,
    VerificationProgress,
    OperationalFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationStopReason {
    PlannerStop,
    TargetsExhausted,
    ActionBudget,
    RoundBudget,
    NoProgress,
    Resolved,
    OperationalTerminal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationPolicy {
    pub max_targets: usize,
    pub max_rounds: usize,
    pub max_actions: usize,
    pub max_no_progress_rounds: usize,
}

impl Default for InvestigationPolicy {
    fn default() -> Self {
        Self {
            max_targets: 4,
            max_rounds: 4,
            max_actions: 6,
            max_no_progress_rounds: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationActionRecord {
    pub round: usize,
    pub action: InvestigationAction,
    pub status: InvestigationObservationStatus,
    pub admitted_evidence: usize,
    pub verification_progress: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationTelemetry {
    pub runtime_id: &'static str,
    pub plan_contract: &'static str,
    pub action_contract: &'static str,
    pub targets: Vec<InvestigationTarget>,
    pub capabilities: Vec<InvestigationCapability>,
    pub rounds: usize,
    pub planner_calls: usize,
    /// Actions selected deterministically by the Harness because exactly one compatible untried
    /// target/capability pair remained. This is selection only and grants no authority.
    #[serde(default)]
    pub harness_unique_selections: usize,
    /// Follow-up actions selected deterministically after a typed `no_result` when the same exact
    /// investigation target has exactly one remaining explicitly fact-key-bound read-only
    /// capability. This does not merge target identities or grant evidence/finalization authority.
    #[serde(default)]
    pub harness_no_result_followup_selections: usize,
    pub rejected_actions: BTreeMap<InvestigationActionRejection, usize>,
    pub actions: Vec<InvestigationActionRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<InvestigationStopReason>,
}

#[derive(Debug, Clone)]
pub struct InvestigationState {
    policy: InvestigationPolicy,
    targets: BTreeMap<String, InvestigationTarget>,
    capabilities: BTreeMap<String, InvestigationCapability>,
    attempted_pairs: BTreeSet<(String, String)>,
    no_progress_rounds: usize,
    telemetry: InvestigationTelemetry,
}

pub fn investigation_plan_schema() -> Value {
    serde_json::to_value(schema_for!(InvestigationPlanProposal))
        .expect("investigation plan schema must serialize")
}

pub fn investigation_action_schema() -> Value {
    serde_json::to_value(schema_for!(InvestigationActionProposal))
        .expect("investigation action schema must serialize")
}

pub fn admit_investigation_plan(
    proposal: InvestigationPlanProposal,
    policy: &InvestigationPolicy,
) -> Result<Vec<InvestigationTarget>, &'static str> {
    if policy.max_targets == 0
        || proposal.targets.is_empty()
        || proposal.targets.len() > policy.max_targets
    {
        return Err("invalid_target_count");
    }
    let mut ids = BTreeSet::new();
    let mut targets = Vec::with_capacity(proposal.targets.len());
    for target in proposal.targets {
        let id = target.id.trim();
        let question = target.question.trim();
        if id.is_empty() || question.is_empty() || !ids.insert(id.to_string()) {
            return Err("invalid_target_identity");
        }
        let expected_fact_key = target
            .expected_fact_key
            .map(|key| key.trim().to_string())
            .filter(|key| !key.is_empty());
        targets.push(InvestigationTarget {
            id: id.to_string(),
            question: question.to_string(),
            expected_fact_key,
            origin: InvestigationTargetOrigin::ModelProposedUntrusted,
        });
    }
    Ok(targets)
}

impl InvestigationState {
    pub fn new(
        targets: Vec<InvestigationTarget>,
        capabilities: Vec<InvestigationCapability>,
        policy: InvestigationPolicy,
    ) -> Result<Self, &'static str> {
        if policy.max_rounds == 0
            || policy.max_actions == 0
            || policy.max_no_progress_rounds == 0
            || targets.is_empty()
            || capabilities.is_empty()
        {
            return Err("invalid_investigation_configuration");
        }
        let mut target_map = BTreeMap::new();
        for target in &targets {
            if target.id.trim().is_empty()
                || target_map
                    .insert(target.id.clone(), target.clone())
                    .is_some()
            {
                return Err("duplicate_target");
            }
        }
        let mut capability_map = BTreeMap::new();
        for capability in &capabilities {
            if capability.id.trim().is_empty()
                || capability_map
                    .insert(capability.id.clone(), capability.clone())
                    .is_some()
            {
                return Err("duplicate_capability");
            }
        }
        Ok(Self {
            policy,
            targets: target_map,
            capabilities: capability_map,
            attempted_pairs: BTreeSet::new(),
            no_progress_rounds: 0,
            telemetry: InvestigationTelemetry {
                runtime_id: INVESTIGATION_RUNTIME_ID,
                plan_contract: INVESTIGATION_PLAN_CONTRACT_ID,
                action_contract: INVESTIGATION_ACTION_CONTRACT_ID,
                targets,
                capabilities,
                rounds: 0,
                planner_calls: 0,
                harness_unique_selections: 0,
                harness_no_result_followup_selections: 0,
                rejected_actions: BTreeMap::new(),
                actions: vec![],
                stop_reason: None,
            },
        })
    }

    pub fn telemetry(&self) -> &InvestigationTelemetry {
        &self.telemetry
    }

    pub fn into_telemetry(self) -> InvestigationTelemetry {
        self.telemetry
    }

    pub fn target(&self, id: &str) -> Option<&InvestigationTarget> {
        self.targets.get(id)
    }

    fn compatible_untried_pairs(&self) -> Vec<(&InvestigationTarget, &InvestigationCapability)> {
        let mut pairs = Vec::new();
        for target in self.targets.values() {
            for capability in self.capabilities.values() {
                let key_compatible = target.expected_fact_key.as_ref().is_none_or(|key| {
                    capability.supported_fact_keys.is_empty()
                        || capability.supported_fact_keys.contains(key)
                });
                let untried = !self
                    .attempted_pairs
                    .contains(&(target.id.clone(), capability.id.clone()));
                if capability.read_only && key_compatible && untried {
                    pairs.push((target, capability));
                }
            }
        }
        pairs
    }

    pub fn remaining_action_count(&self) -> usize {
        self.compatible_untried_pairs().len()
    }

    /// Returns a Harness-owned action proposal only when selection is mechanically unambiguous.
    /// The selected target remains untrusted planning state and the capability remains subject to
    /// the ordinary read-only, acquisition, admission, and verification boundaries.
    pub fn unique_compatible_action_proposal(&self) -> Option<InvestigationActionProposal> {
        let pairs = self
            .compatible_untried_pairs()
            .into_iter()
            .filter(|(target, capability)| {
                target.expected_fact_key.as_ref().is_some_and(|key| {
                    !capability.supported_fact_keys.is_empty()
                        && capability.supported_fact_keys.contains(key)
                })
            })
            .collect::<Vec<_>>();
        if pairs.len() != 1 {
            return None;
        }
        let (target, capability) = pairs[0];
        let key = target.expected_fact_key.as_deref()?;
        if capability.supported_fact_keys.is_empty()
            || !capability.supported_fact_keys.contains(key)
        {
            return None;
        }
        Some(InvestigationActionProposal {
            action: InvestigationActionKind::Acquire,
            target_id: Some(target.id.clone()),
            capability_id: Some(capability.id.clone()),
        })
    }

    /// After a typed `no_result`, continue the same exact target without another stochastic
    /// selector call only when one explicitly key-bound read-only capability remains for that
    /// target. Other investigation targets are deliberately ignored for this narrow continuation:
    /// their identities/questions are neither merged nor treated as equivalent.
    pub fn unique_no_result_followup_proposal(&self) -> Option<InvestigationActionProposal> {
        let last = self.telemetry.actions.last()?;
        if last.status != InvestigationObservationStatus::NoResult {
            return None;
        }
        let target = self.targets.get(&last.action.target_id)?;
        let key = target.expected_fact_key.as_deref()?;
        let capabilities = self
            .capabilities
            .values()
            .filter(|capability| {
                capability.read_only
                    && !capability.supported_fact_keys.is_empty()
                    && capability.supported_fact_keys.contains(key)
                    && !self
                        .attempted_pairs
                        .contains(&(target.id.clone(), capability.id.clone()))
            })
            .collect::<Vec<_>>();
        if capabilities.len() != 1 {
            return None;
        }
        Some(InvestigationActionProposal {
            action: InvestigationActionKind::Acquire,
            target_id: Some(target.id.clone()),
            capability_id: Some(capabilities[0].id.clone()),
        })
    }

    pub fn capability(&self, id: &str) -> Option<&InvestigationCapability> {
        self.capabilities.get(id)
    }

    pub fn begin_round(&mut self) -> Result<usize, InvestigationStopReason> {
        if let Some(reason) = self.telemetry.stop_reason {
            return Err(reason);
        }
        if self.telemetry.rounds >= self.policy.max_rounds {
            self.stop(InvestigationStopReason::RoundBudget);
            return Err(InvestigationStopReason::RoundBudget);
        }
        self.telemetry.rounds += 1;
        Ok(self.telemetry.rounds)
    }

    pub fn note_planner_call(&mut self) {
        self.telemetry.planner_calls = self.telemetry.planner_calls.saturating_add(1);
    }

    pub fn note_harness_unique_selection(&mut self) {
        self.telemetry.harness_unique_selections =
            self.telemetry.harness_unique_selections.saturating_add(1);
    }

    pub fn note_harness_no_result_followup_selection(&mut self) {
        self.telemetry.harness_no_result_followup_selections = self
            .telemetry
            .harness_no_result_followup_selections
            .saturating_add(1);
    }

    pub fn validate_action(
        &mut self,
        proposal: InvestigationActionProposal,
    ) -> Result<Option<InvestigationAction>, InvestigationActionRejection> {
        if self.telemetry.stop_reason.is_some() {
            return self.reject(InvestigationActionRejection::Stopped);
        }
        if proposal.action == InvestigationActionKind::Stop {
            if proposal.target_id.is_some() || proposal.capability_id.is_some() {
                return self.reject(InvestigationActionRejection::InvalidShape);
            }
            self.stop(InvestigationStopReason::PlannerStop);
            return Ok(None);
        }
        if self.telemetry.actions.len() >= self.policy.max_actions {
            self.stop(InvestigationStopReason::ActionBudget);
            return self.reject(InvestigationActionRejection::ActionBudgetExhausted);
        }
        let Some(target_id) = proposal.target_id.filter(|value| !value.trim().is_empty()) else {
            return self.reject(InvestigationActionRejection::InvalidShape);
        };
        let Some(capability_id) = proposal
            .capability_id
            .filter(|value| !value.trim().is_empty())
        else {
            return self.reject(InvestigationActionRejection::InvalidShape);
        };
        let Some(target) = self.targets.get(&target_id) else {
            return self.reject(InvestigationActionRejection::UnknownTarget);
        };
        let Some(capability) = self.capabilities.get(&capability_id) else {
            return self.reject(InvestigationActionRejection::UnknownCapability);
        };
        if !capability.read_only {
            return self.reject(InvestigationActionRejection::CapabilityNotReadOnly);
        }
        if let Some(key) = target.expected_fact_key.as_deref() {
            if !capability.supported_fact_keys.is_empty()
                && !capability.supported_fact_keys.contains(key)
            {
                return self.reject(InvestigationActionRejection::UnsupportedTargetKey);
            }
        }
        if !self
            .attempted_pairs
            .insert((target_id.clone(), capability_id.clone()))
        {
            return self.reject(InvestigationActionRejection::DuplicateAction);
        }
        Ok(Some(InvestigationAction {
            action_index: self.telemetry.actions.len(),
            target_id,
            capability_id,
        }))
    }

    pub fn record_observation(
        &mut self,
        round: usize,
        action: InvestigationAction,
        status: InvestigationObservationStatus,
        admitted_evidence: usize,
        verification_progress: bool,
    ) -> Option<InvestigationStopReason> {
        let progress = admitted_evidence > 0 || verification_progress;
        if progress {
            self.no_progress_rounds = 0;
        } else {
            self.no_progress_rounds = self.no_progress_rounds.saturating_add(1);
        }
        self.telemetry.actions.push(InvestigationActionRecord {
            round,
            action,
            status,
            admitted_evidence,
            verification_progress,
        });
        if self.telemetry.actions.len() >= self.policy.max_actions {
            self.stop(InvestigationStopReason::ActionBudget);
        } else if self.no_progress_rounds >= self.policy.max_no_progress_rounds {
            self.stop(InvestigationStopReason::NoProgress);
        }
        self.telemetry.stop_reason
    }

    pub fn stop(&mut self, reason: InvestigationStopReason) {
        if self.telemetry.stop_reason.is_none() {
            self.telemetry.stop_reason = Some(reason);
        }
    }

    fn reject<T>(
        &mut self,
        reason: InvestigationActionRejection,
    ) -> Result<T, InvestigationActionRejection> {
        *self.telemetry.rejected_actions.entry(reason).or_default() += 1;
        Err(reason)
    }
}

pub fn build_investigation_plan_request(
    task: &str,
    capabilities: &[InvestigationCapability],
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let capabilities = serde_json::to_string_pretty(capabilities)?;
    Ok(ModelRequest {
        system: Some(
            "You are an untrusted investigation planner inside a verification harness. Return only the requested structured plan. Propose bounded questions to investigate; do not answer them, invent evidence, claim authority, select write capabilities, or decide correctness. expected_fact_key is only a selector hint when the task clearly names the fact family; omit it when uncertain.".into(),
        ),
        task: format!(
            "User task:\n{task}\n\nHarness-configured read-only capability descriptors:\n{capabilities}\n\nPropose concise investigation targets. Targets are untrusted planning objects, not hypotheses or verified facts. Do not include tool arguments, evidence, identity claims, authority classes, answers, or verdicts."
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: INVESTIGATION_PLAN_CONTRACT_ID.into(),
            schema: investigation_plan_schema(),
        },
        max_tokens,
        random_seed,
        reasoning_preference: None,
    })
}

pub fn build_investigation_action_request(
    task: &str,
    telemetry: &InvestigationTelemetry,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let targets = serde_json::to_string_pretty(&telemetry.targets)?;
    let capabilities = serde_json::to_string_pretty(&telemetry.capabilities)?;
    let prior_actions = serde_json::to_string_pretty(&telemetry.actions)?;
    Ok(ModelRequest {
        system: Some(
            "You are an untrusted action selector inside a bounded investigation harness. Return only the requested structured action. You may select one existing target ID and one existing read-only capability ID, or stop. Never create or edit a query, target, capability, identity context, evidence, authority, or verdict. The Harness validates every action and owns admission, verification, budgets, and correctness.".into(),
        ),
        task: format!(
            "User task:\n{task}\n\nCanonical Harness investigation targets:\n{targets}\n\nConfigured capability descriptors:\n{capabilities}\n\nPrior typed action outcomes:\n{prior_actions}\n\nSelect exactly one acquire action using existing IDs or stop. Prefer an untried capability for an unresolved target after no_result, ambiguous, rejected_evidence, or operational_failure. Never repeat an identical target/capability pair."
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: INVESTIGATION_ACTION_CONTRACT_ID.into(),
            schema: investigation_action_schema(),
        },
        max_tokens,
        random_seed,
        reasoning_preference: None,
    })
}

pub fn parse_investigation_plan(
    text: &str,
) -> Result<InvestigationPlanProposal, serde_json::Error> {
    serde_json::from_str(text)
}

pub fn parse_investigation_action(
    text: &str,
) -> Result<InvestigationActionProposal, serde_json::Error> {
    serde_json::from_str(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(id: &str, key: Option<&str>) -> InvestigationTarget {
        InvestigationTarget {
            id: id.into(),
            question: format!("question {id}"),
            expected_fact_key: key.map(str::to_string),
            origin: InvestigationTargetOrigin::ModelProposedUntrusted,
        }
    }

    fn capability(id: &str, keys: &[&str]) -> InvestigationCapability {
        InvestigationCapability {
            id: id.into(),
            adapter: "fixture_readonly".into(),
            read_only: true,
            supported_fact_keys: keys.iter().map(|key| (*key).to_string()).collect(),
        }
    }

    #[test]
    fn plan_schema_is_closed_and_targets_remain_untrusted() {
        let schema = investigation_plan_schema();
        assert_eq!(
            schema.get("additionalProperties"),
            Some(&serde_json::Value::Bool(false))
        );
        let targets = admit_investigation_plan(
            InvestigationPlanProposal {
                targets: vec![InvestigationTargetProposal {
                    id: "region".into(),
                    question: "Which region serves the deployment?".into(),
                    expected_fact_key: Some("service.region".into()),
                }],
            },
            &InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            targets[0].origin,
            InvestigationTargetOrigin::ModelProposedUntrusted
        );
    }

    #[test]
    fn planner_cannot_select_unknown_or_write_capability() {
        let mut write = capability("write", &["service.region"]);
        write.read_only = false;
        let mut state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            vec![write],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            state.validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("missing".into()),
            }),
            Err(InvestigationActionRejection::UnknownCapability)
        );
        assert_eq!(
            state.validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("write".into()),
            }),
            Err(InvestigationActionRejection::CapabilityNotReadOnly)
        );
    }

    #[test]
    fn duplicate_and_no_progress_actions_terminate_safely() {
        let mut state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            vec![
                capability("a", &["service.region"]),
                capability("b", &["service.region"]),
            ],
            InvestigationPolicy {
                max_no_progress_rounds: 2,
                ..InvestigationPolicy::default()
            },
        )
        .unwrap();
        let round = state.begin_round().unwrap();
        let first = state
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("a".into()),
            })
            .unwrap()
            .unwrap();
        state.record_observation(
            round,
            first,
            InvestigationObservationStatus::NoResult,
            0,
            false,
        );
        assert_eq!(
            state.validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("a".into()),
            }),
            Err(InvestigationActionRejection::DuplicateAction)
        );
        let second_round = state.begin_round().unwrap();
        let second = state
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("b".into()),
            })
            .unwrap()
            .unwrap();
        assert_eq!(
            state.record_observation(
                second_round,
                second,
                InvestigationObservationStatus::Ambiguous,
                0,
                false,
            ),
            Some(InvestigationStopReason::NoProgress)
        );
    }

    #[test]
    fn harness_selects_only_a_unique_compatible_untried_pair() {
        let state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            vec![
                capability("region-read", &["service.region"]),
                capability("inventory-read", &["inventory.count"]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            state.unique_compatible_action_proposal(),
            Some(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("region-read".into()),
            })
        );
    }

    #[test]
    fn harness_does_not_auto_select_keyless_or_wildcard_pairs() {
        let keyless = InvestigationState::new(
            vec![target("region", None)],
            vec![capability("region-read", &["service.region"])],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(keyless.remaining_action_count(), 1);
        assert_eq!(keyless.unique_compatible_action_proposal(), None);

        let wildcard = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            vec![capability("generic-read", &[])],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(wildcard.remaining_action_count(), 1);
        assert_eq!(wildcard.unique_compatible_action_proposal(), None);
    }

    #[test]
    fn harness_does_not_override_ambiguous_action_selection() {
        let state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            vec![
                capability("a", &["service.region"]),
                capability("b", &["service.region"]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(state.remaining_action_count(), 2);
        assert_eq!(state.unique_compatible_action_proposal(), None);
    }

    #[test]
    fn harness_unique_selection_requires_explicit_fact_key_binding() {
        let missing_key = InvestigationState::new(
            vec![target("region", None)],
            vec![capability("read", &["service.region"])],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(missing_key.unique_compatible_action_proposal(), None);

        let wildcard = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            vec![capability("read", &[])],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(wildcard.unique_compatible_action_proposal(), None);
    }

    #[test]
    fn no_result_can_reduce_selection_to_one_safe_follow_up() {
        let mut state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            vec![
                capability("a", &["service.region"]),
                capability("b", &["service.region"]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        let round = state.begin_round().unwrap();
        let first = state
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("a".into()),
            })
            .unwrap()
            .unwrap();
        state.record_observation(
            round,
            first,
            InvestigationObservationStatus::NoResult,
            0,
            false,
        );
        assert_eq!(
            state.unique_compatible_action_proposal(),
            Some(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("b".into()),
            })
        );
    }

    #[test]
    fn no_result_followup_stays_on_exact_target_despite_same_key_sibling() {
        let mut state = InvestigationState::new(
            vec![
                target("owner-primary", Some("routing.owner")),
                target("owner-secondary", Some("routing.owner")),
            ],
            vec![
                capability("cache", &["routing.owner"]),
                capability("registry", &["routing.owner"]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(state.unique_compatible_action_proposal(), None);
        assert_eq!(state.unique_no_result_followup_proposal(), None);

        let round = state.begin_round().unwrap();
        let first = state
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner-primary".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        state.record_observation(
            round,
            first,
            InvestigationObservationStatus::NoResult,
            0,
            false,
        );

        // Multiple global target/capability pairs remain, so the original #233 selector stays
        // conservative. The narrow follow-up selector continues only the exact target that just
        // produced no_result and never merges the sibling target identity/question.
        assert_eq!(state.unique_compatible_action_proposal(), None);
        assert_eq!(
            state.unique_no_result_followup_proposal(),
            Some(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner-primary".into()),
                capability_id: Some("registry".into()),
            })
        );
    }

    #[test]
    fn no_result_followup_requires_exactly_one_remaining_explicit_capability() {
        let mut state = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability("cache", &["routing.owner"]),
                capability("registry-a", &["routing.owner"]),
                capability("registry-b", &["routing.owner"]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        let round = state.begin_round().unwrap();
        let first = state
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        state.record_observation(
            round,
            first,
            InvestigationObservationStatus::NoResult,
            0,
            false,
        );
        assert_eq!(state.unique_no_result_followup_proposal(), None);
    }

    #[test]
    fn no_result_followup_does_not_trigger_for_other_typed_outcomes() {
        let mut state = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability("cache", &["routing.owner"]),
                capability("registry", &["routing.owner"]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        let round = state.begin_round().unwrap();
        let first = state
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        state.record_observation(
            round,
            first,
            InvestigationObservationStatus::RejectedEvidence,
            0,
            false,
        );
        assert_eq!(state.unique_no_result_followup_proposal(), None);
    }

    #[test]
    fn plan_and_action_requests_keep_authority_outside_model_contract() {
        let capabilities = vec![capability("lookup", &["service.region"])];
        let plan =
            build_investigation_plan_request("find the region", &capabilities, Some(128), Some(7))
                .unwrap();
        assert!(matches!(
            plan.output_format,
            ModelOutputFormat::JsonSchema { .. }
        ));
        assert!(plan.system.as_deref().unwrap().contains("do not answer"));
        assert!(
            !investigation_plan_schema()
                .to_string()
                .contains("authority_class")
        );

        let state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            capabilities,
            InvestigationPolicy::default(),
        )
        .unwrap();
        let action = build_investigation_action_request(
            "find the region",
            state.telemetry(),
            Some(128),
            Some(8),
        )
        .unwrap();
        assert!(matches!(
            action.output_format,
            ModelOutputFormat::JsonSchema { .. }
        ));
        assert!(
            action
                .task
                .contains("Never repeat an identical target/capability pair")
        );
        assert!(!investigation_action_schema().to_string().contains("query"));
    }

    #[test]
    fn capability_selection_is_key_bounded() {
        let mut state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            vec![capability("billing", &["account.total"])],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            state.validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("region".into()),
                capability_id: Some("billing".into()),
            }),
            Err(InvestigationActionRejection::UnsupportedTargetKey)
        );
    }
}
