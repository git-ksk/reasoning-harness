use std::collections::{BTreeMap, BTreeSet};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ModelOutputFormat, ModelReasoningPreference, ModelRequest};

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

/// Schema-only model-facing contract for the narrow shared exact read-only fact family. Runtime
/// parsing deliberately remains `InvestigationPlanProposal`, so provider fallback/unconstrained
/// output is retained exactly instead of being repaired or inferred by Harness.
#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(deny_unknown_fields)]
struct ModelFacingExactInvestigationPlanProposal {
    targets: Vec<ModelFacingExactInvestigationTargetProposal>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(deny_unknown_fields)]
struct ModelFacingExactInvestigationTargetProposal {
    id: String,
    question: String,
    #[schemars(length(min = 1))]
    expected_fact_key: String,
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
    /// Optional Harness-owned acquisition-selection precedence. Higher values are preferred only
    /// by the narrow deterministic exact-key selector; this grants no evidence or answer authority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_priority: Option<u32>,
    #[serde(default)]
    pub supported_fact_keys: BTreeSet<String>,
}

#[derive(Serialize)]
struct ModelVisibleInvestigationCapability<'a> {
    id: &'a str,
    adapter: &'a str,
    read_only: bool,
    supported_fact_keys: &'a BTreeSet<String>,
}

fn serialize_model_visible_capabilities(
    capabilities: &[InvestigationCapability],
) -> Result<String, serde_json::Error> {
    let visible = capabilities
        .iter()
        .map(|capability| ModelVisibleInvestigationCapability {
            id: &capability.id,
            adapter: &capability.adapter,
            read_only: capability.read_only,
            supported_fact_keys: &capability.supported_fact_keys,
        })
        .collect::<Vec<_>>();
    serde_json::to_string_pretty(&visible)
}

fn shared_exact_read_only_fact_key(capabilities: &[InvestigationCapability]) -> Option<&str> {
    let read_only = capabilities
        .iter()
        .filter(|capability| capability.read_only)
        .collect::<Vec<_>>();
    if read_only.len() < 2 {
        return None;
    }

    let mut shared_key = None;
    for capability in read_only {
        if capability.supported_fact_keys.len() != 1 {
            return None;
        }
        let key = capability.supported_fact_keys.iter().next()?.as_str();
        if key.is_empty() || key.trim() != key {
            return None;
        }
        match shared_key {
            None => shared_key = Some(key),
            Some(expected) if expected == key => {}
            Some(_) => return None,
        }
    }
    shared_key
}

fn constrain_exact_fact_key_schema(schema: &mut Value, exact_key: &str) -> bool {
    match schema {
        Value::Object(object) => {
            if let Some(Value::Object(properties)) = object.get_mut("properties")
                && properties.contains_key("id")
                && properties.contains_key("question")
                && let Some(Value::Object(expected_fact_key)) =
                    properties.get_mut("expected_fact_key")
            {
                expected_fact_key.insert("enum".into(), serde_json::json!([exact_key]));
                return true;
            }
            object
                .values_mut()
                .any(|value| constrain_exact_fact_key_schema(value, exact_key))
        }
        Value::Array(values) => values
            .iter_mut()
            .any(|value| constrain_exact_fact_key_schema(value, exact_key)),
        _ => false,
    }
}

fn investigation_plan_schema_for_capabilities(capabilities: &[InvestigationCapability]) -> Value {
    let Some(exact_key) = shared_exact_read_only_fact_key(capabilities) else {
        return investigation_plan_schema();
    };

    let mut schema = serde_json::to_value(schema_for!(ModelFacingExactInvestigationPlanProposal))
        .expect("strict investigation plan schema must serialize");
    assert!(
        constrain_exact_fact_key_schema(&mut schema, exact_key),
        "strict investigation target schema must expose expected_fact_key"
    );
    schema
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
    /// `acquire` executes one existing Harness target/capability pair; `stop` ends planning.
    pub action: InvestigationActionKind,
    /// Required and non-empty for `acquire`; must be omitted for `stop`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,
    /// Required and non-empty for `acquire`; must be omitted for `stop`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_id: Option<String>,
}

/// Schema-only model-facing contract. Runtime parsing deliberately remains the broader
/// `InvestigationActionProposal` so unconstrained/provider-fallback output is still retained
/// exactly and rejected by Harness-owned validation instead of being repaired or inferred.
#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
enum ModelFacingInvestigationActionProposal {
    Acquire {
        #[schemars(length(min = 1))]
        target_id: String,
        #[schemars(length(min = 1))]
        capability_id: String,
    },
    Stop {},
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationActionRejectionRecord {
    pub round: usize,
    pub proposal: InvestigationActionProposal,
    pub reason: InvestigationActionRejection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationPrecedenceSkipReason {
    TerminalOrActionBudget,
    NoEligibleExactTarget,
    SameKeySibling,
    MultipleEligibleTargets,
    MissingPriority,
    HighestPriorityTie,
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
    /// Actions selected by the Harness from an explicit exact-key capability set only when one
    /// target identity and one highest configured selection priority are mechanically unique.
    #[serde(default)]
    pub harness_precedence_selections: usize,
    /// Diagnostic-only reasons why #261 precedence was not selected when that selector was
    /// evaluated. These counts never participate in admission, correctness, or release scoring.
    #[serde(default)]
    pub precedence_skip_reasons: BTreeMap<InvestigationPrecedenceSkipReason, usize>,
    pub rejected_actions: BTreeMap<InvestigationActionRejection, usize>,
    /// Diagnostic-only rejected proposals retained so a subsequent planner round can see the
    /// exact typed validation failure instead of repeating the same invalid shape blindly.
    #[serde(default)]
    pub action_rejection_records: Vec<InvestigationActionRejectionRecord>,
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
    serde_json::to_value(schema_for!(ModelFacingInvestigationActionProposal))
        .expect("investigation action schema must serialize")
}

fn investigation_action_schema_for_telemetry(telemetry: &InvestigationTelemetry) -> Value {
    let mut schema = investigation_action_schema();
    let branches = schema["oneOf"]
        .as_array()
        .expect("investigation action schema must be a closed union")
        .clone();
    let acquire_template = branches
        .iter()
        .find(|branch| branch["properties"]["action"]["const"] == "acquire")
        .expect("investigation action schema must expose acquire")
        .clone();
    let stop = branches
        .iter()
        .find(|branch| branch["properties"]["action"]["const"] == "stop")
        .expect("investigation action schema must expose stop")
        .clone();
    let attempted_pairs = telemetry
        .actions
        .iter()
        .map(|record| {
            (
                record.action.target_id.as_str(),
                record.action.capability_id.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();

    let mut available = Vec::new();
    for target in &telemetry.targets {
        for capability in &telemetry.capabilities {
            let key_compatible = target.expected_fact_key.as_ref().is_none_or(|key| {
                capability.supported_fact_keys.is_empty()
                    || capability.supported_fact_keys.contains(key)
            });
            let untried = !attempted_pairs.contains(&(target.id.as_str(), capability.id.as_str()));
            if !capability.read_only || !key_compatible || !untried {
                continue;
            }

            let mut branch = acquire_template.clone();
            branch["properties"]["target_id"]["const"] = Value::String(target.id.clone());
            branch["properties"]["capability_id"]["const"] = Value::String(capability.id.clone());
            available.push(branch);
        }
    }
    available.push(stop);
    schema["oneOf"] = Value::Array(available);
    schema
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
        // Preserve the provider-proposed key exactly. Harness-owned exact-key selectors must not
        // gain compatibility through trimming or normalization at plan admission.
        let expected_fact_key = target.expected_fact_key;
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
                harness_precedence_selections: 0,
                precedence_skip_reasons: BTreeMap::new(),
                rejected_actions: BTreeMap::new(),
                action_rejection_records: vec![],
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

    /// Selects a read-only acquisition by explicit Harness-owned precedence only when both the
    /// target identity and the highest priority capability are mechanically unique. This selector
    /// deliberately excludes keyless/wildcard compatibility, already-attempted pairs, missing
    /// precedence, ties, and terminal/action-budget states. It is selection-only and cannot admit
    /// evidence, merge target identities, or promote verification/finalization authority.
    pub fn unique_precedence_action_proposal(&self) -> Option<InvestigationActionProposal> {
        self.precedence_action_proposal_result().ok()
    }

    /// Runtime-facing variant that preserves the exact #261 selector semantics while recording a
    /// diagnostic-only reason when precedence cannot select an action.
    pub fn unique_precedence_action_proposal_with_diagnostic(
        &mut self,
    ) -> Option<InvestigationActionProposal> {
        match self.precedence_action_proposal_result() {
            Ok(proposal) => Some(proposal),
            Err(reason) => {
                *self
                    .telemetry
                    .precedence_skip_reasons
                    .entry(reason)
                    .or_default() += 1;
                None
            }
        }
    }

    fn precedence_action_proposal_result(
        &self,
    ) -> Result<InvestigationActionProposal, InvestigationPrecedenceSkipReason> {
        if self.telemetry.stop_reason.is_some()
            || self.telemetry.actions.len() >= self.policy.max_actions
        {
            return Err(InvestigationPrecedenceSkipReason::TerminalOrActionBudget);
        }

        let mut eligible_targets = Vec::new();
        let mut same_key_sibling_blocked = false;
        for target in self.targets.values() {
            let Some(key) = target.expected_fact_key.as_deref() else {
                continue;
            };
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
            if capabilities.is_empty() {
                continue;
            }
            if self.targets.values().any(|sibling| {
                sibling.id != target.id && sibling.expected_fact_key.as_deref() == Some(key)
            }) {
                same_key_sibling_blocked = true;
                continue;
            }
            eligible_targets.push((target, capabilities));
        }

        if eligible_targets.is_empty() {
            return Err(if same_key_sibling_blocked {
                InvestigationPrecedenceSkipReason::SameKeySibling
            } else {
                InvestigationPrecedenceSkipReason::NoEligibleExactTarget
            });
        }
        if eligible_targets.len() != 1 {
            return Err(InvestigationPrecedenceSkipReason::MultipleEligibleTargets);
        }
        let (target, capabilities) = &eligible_targets[0];
        if capabilities
            .iter()
            .any(|capability| capability.selection_priority.is_none())
        {
            return Err(InvestigationPrecedenceSkipReason::MissingPriority);
        }
        let highest = capabilities
            .iter()
            .filter_map(|capability| capability.selection_priority)
            .max()
            .ok_or(InvestigationPrecedenceSkipReason::MissingPriority)?;
        let best = capabilities
            .iter()
            .filter(|capability| capability.selection_priority == Some(highest))
            .collect::<Vec<_>>();
        if best.len() != 1 {
            return Err(InvestigationPrecedenceSkipReason::HighestPriorityTie);
        }

        Ok(InvestigationActionProposal {
            action: InvestigationActionKind::Acquire,
            target_id: Some(target.id.clone()),
            capability_id: Some(best[0].id.clone()),
        })
    }

    /// After a typed `no_result`, continue the same exact target without another stochastic
    /// selector call only when one explicitly key-bound read-only capability remains for that
    /// target. Other investigation targets are deliberately ignored for this narrow continuation:
    /// their identities/questions are neither merged nor treated as equivalent.
    pub fn unique_no_result_followup_proposal(&self) -> Option<InvestigationActionProposal> {
        if self.telemetry.stop_reason.is_some() {
            return None;
        }
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

    pub fn note_harness_precedence_selection(&mut self) {
        self.telemetry.harness_precedence_selections = self
            .telemetry
            .harness_precedence_selections
            .saturating_add(1);
    }

    pub fn validate_action(
        &mut self,
        proposal: InvestigationActionProposal,
    ) -> Result<Option<InvestigationAction>, InvestigationActionRejection> {
        if self.telemetry.stop_reason.is_some() {
            return self.reject_action(&proposal, InvestigationActionRejection::Stopped);
        }
        if proposal.action == InvestigationActionKind::Stop {
            if proposal.target_id.is_some() || proposal.capability_id.is_some() {
                return self.reject_action(&proposal, InvestigationActionRejection::InvalidShape);
            }
            self.stop(InvestigationStopReason::PlannerStop);
            return Ok(None);
        }
        if self.telemetry.actions.len() >= self.policy.max_actions {
            self.stop(InvestigationStopReason::ActionBudget);
            return self.reject_action(
                &proposal,
                InvestigationActionRejection::ActionBudgetExhausted,
            );
        }
        let Some(target_id) = proposal
            .target_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        else {
            return self.reject_action(&proposal, InvestigationActionRejection::InvalidShape);
        };
        let Some(capability_id) = proposal
            .capability_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        else {
            return self.reject_action(&proposal, InvestigationActionRejection::InvalidShape);
        };
        let Some(target) = self.targets.get(target_id) else {
            return self.reject_action(&proposal, InvestigationActionRejection::UnknownTarget);
        };
        let Some(capability) = self.capabilities.get(capability_id) else {
            return self.reject_action(&proposal, InvestigationActionRejection::UnknownCapability);
        };
        if !capability.read_only {
            return self.reject_action(
                &proposal,
                InvestigationActionRejection::CapabilityNotReadOnly,
            );
        }
        if let Some(key) = target.expected_fact_key.as_deref() {
            if !capability.supported_fact_keys.is_empty()
                && !capability.supported_fact_keys.contains(key)
            {
                return self.reject_action(
                    &proposal,
                    InvestigationActionRejection::UnsupportedTargetKey,
                );
            }
        }
        if !self
            .attempted_pairs
            .insert((target_id.to_string(), capability_id.to_string()))
        {
            return self.reject_action(&proposal, InvestigationActionRejection::DuplicateAction);
        }
        Ok(Some(InvestigationAction {
            action_index: self.telemetry.actions.len(),
            target_id: target_id.to_string(),
            capability_id: capability_id.to_string(),
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

    fn reject_action<T>(
        &mut self,
        proposal: &InvestigationActionProposal,
        reason: InvestigationActionRejection,
    ) -> Result<T, InvestigationActionRejection> {
        *self.telemetry.rejected_actions.entry(reason).or_default() += 1;
        self.telemetry
            .action_rejection_records
            .push(InvestigationActionRejectionRecord {
                round: self.telemetry.rounds,
                proposal: proposal.clone(),
                reason,
            });
        Err(reason)
    }
}

pub fn build_investigation_plan_request(
    task: &str,
    capabilities: &[InvestigationCapability],
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let exact_key = shared_exact_read_only_fact_key(capabilities);
    let schema = investigation_plan_schema_for_capabilities(capabilities);
    let capabilities = serialize_model_visible_capabilities(capabilities)?;
    let (system, task) = exact_key.map_or_else(
        || {
            (
                "You are an untrusted investigation planner inside a verification harness. Return only the requested structured plan. Propose bounded questions to investigate; do not answer them, invent evidence, claim authority, select write capabilities, or decide correctness. expected_fact_key is only a selector hint when the task clearly names the fact family; omit it when uncertain.".to_string(),
                format!(
                    "User task:\n{task}\n\nHarness-configured read-only capability descriptors:\n{capabilities}\n\nPropose concise investigation targets. Targets are untrusted planning objects, not hypotheses or verified facts. Do not include tool arguments, evidence, identity claims, authority classes, answers, or verdicts."
                ),
            )
        },
        |key| {
            let key_instruction = format!(
                "For every target, expected_fact_key is required and must be exactly the Harness-configured key `{key}`. Do not omit, alter, normalize, or substitute this key."
            );
            (
                format!(
                    "You are an untrusted investigation planner inside a verification harness. Return only the requested structured plan. Propose bounded questions to investigate; do not answer them, invent evidence, claim authority, select write capabilities, or decide correctness. {key_instruction}"
                ),
                format!(
                    "User task:\n{task}\n\nHarness-configured read-only capability descriptors:\n{capabilities}\n\n{key_instruction}\n\nPropose concise investigation targets. Targets are untrusted planning objects, not hypotheses or verified facts. Do not include tool arguments, evidence, identity claims, authority classes, answers, or verdicts."
                ),
            )
        },
    );
    Ok(ModelRequest {
        system: Some(system),
        task,
        output_format: ModelOutputFormat::JsonSchema {
            name: INVESTIGATION_PLAN_CONTRACT_ID.into(),
            schema,
        },
        max_tokens,
        random_seed,
        reasoning_preference: Some(ModelReasoningPreference::Minimize),
    })
}

pub fn build_investigation_action_request(
    task: &str,
    telemetry: &InvestigationTelemetry,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
) -> Result<ModelRequest, serde_json::Error> {
    let targets = serde_json::to_string_pretty(&telemetry.targets)?;
    let capabilities = serialize_model_visible_capabilities(&telemetry.capabilities)?;
    let prior_actions = serde_json::to_string_pretty(&telemetry.actions)?;
    let prior_rejections = serde_json::to_string_pretty(&telemetry.action_rejection_records)?;
    Ok(ModelRequest {
        system: Some(
            "You are an untrusted action selector inside a bounded investigation harness. Return only the requested structured action. An acquire action requires a non-empty existing target_id and a non-empty existing read-only capability_id. A stop action must omit both target_id and capability_id. Never create or edit a query, target, capability, identity context, evidence, authority, or verdict. The Harness validates every action and owns admission, verification, budgets, and correctness.".into(),
        ),
        task: format!(
            "User task:\n{task}\n\nCanonical Harness investigation targets:\n{targets}\n\nConfigured capability descriptors:\n{capabilities}\n\nPrior typed action outcomes:\n{prior_actions}\n\nPrior typed action validation rejections:\n{prior_rejections}\n\nSelect exactly one acquire action using existing IDs or stop. For acquire, include both non-empty target_id and capability_id. For stop, omit both IDs. Prefer an untried capability for an unresolved target after no_result, ambiguous, rejected_evidence, or operational_failure. Correct any prior typed validation rejection instead of repeating the same invalid proposal. Never repeat an identical target/capability pair."
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: INVESTIGATION_ACTION_CONTRACT_ID.into(),
            schema: investigation_action_schema_for_telemetry(telemetry),
        },
        max_tokens,
        random_seed,
        reasoning_preference: Some(ModelReasoningPreference::Minimize),
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
            selection_priority: None,
            supported_fact_keys: keys.iter().map(|key| (*key).to_string()).collect(),
        }
    }

    fn capability_with_priority(
        id: &str,
        keys: &[&str],
        selection_priority: u32,
    ) -> InvestigationCapability {
        let mut capability = capability(id, keys);
        capability.selection_priority = Some(selection_priority);
        capability
    }

    fn find_target_schema(value: &Value) -> Option<&serde_json::Map<String, Value>> {
        match value {
            Value::Object(object) => {
                if let Some(Value::Object(properties)) = object.get("properties")
                    && properties.contains_key("id")
                    && properties.contains_key("question")
                    && properties.contains_key("expected_fact_key")
                {
                    return Some(object);
                }
                object.values().find_map(find_target_schema)
            }
            Value::Array(values) => values.iter().find_map(find_target_schema),
            _ => None,
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
    fn shared_exact_read_only_family_uses_required_exact_key_schema() {
        let capabilities = vec![
            capability("cache", &["routing.owner"]),
            capability("registry", &["routing.owner"]),
        ];
        let schema = investigation_plan_schema_for_capabilities(&capabilities);
        assert_ne!(schema, investigation_plan_schema());

        let target_schema = find_target_schema(&schema).expect("strict target schema");
        let required = target_schema["required"]
            .as_array()
            .expect("strict target required fields");
        assert!(required.contains(&serde_json::json!("expected_fact_key")));
        assert_eq!(
            target_schema["properties"]["expected_fact_key"]["minLength"],
            1
        );
        assert_eq!(
            target_schema["properties"]["expected_fact_key"]["enum"],
            serde_json::json!(["routing.owner"])
        );
    }

    #[test]
    fn non_shared_fact_families_keep_optional_plan_schema() {
        let baseline = investigation_plan_schema();
        let cases = vec![
            vec![capability("single", &["routing.owner"])],
            vec![capability("wildcard-a", &[]), capability("wildcard-b", &[])],
            vec![capability("empty-a", &[""]), capability("empty-b", &[""])],
            vec![
                capability("uncanonical-a", &[" routing.owner"]),
                capability("uncanonical-b", &[" routing.owner"]),
            ],
            vec![
                capability("multi-a", &["routing.owner", "routing.region"]),
                capability("multi-b", &["routing.owner", "routing.region"]),
            ],
            vec![
                capability("owner", &["routing.owner"]),
                capability("region", &["routing.region"]),
            ],
        ];
        for capabilities in cases {
            assert_eq!(
                investigation_plan_schema_for_capabilities(&capabilities),
                baseline
            );
        }
    }

    #[test]
    fn strict_plan_request_requires_and_names_exact_shared_key() {
        let capabilities = vec![
            capability("cache", &["routing.owner"]),
            capability("registry", &["routing.owner"]),
        ];
        let request =
            build_investigation_plan_request("find the owner", &capabilities, Some(128), Some(7))
                .unwrap();
        let system = request.system.as_deref().unwrap();
        assert!(system.contains("expected_fact_key is required"));
        assert!(system.contains("`routing.owner`"));
        assert!(system.contains("Do not omit, alter, normalize, or substitute"));
        assert!(request.task.contains("`routing.owner`"));
    }

    #[test]
    fn selection_priority_does_not_change_strict_plan_contract_or_request() {
        let plain = vec![
            capability("cache", &["routing.owner"]),
            capability("registry", &["routing.owner"]),
        ];
        let prioritized = vec![
            capability_with_priority("cache", &["routing.owner"], 20),
            capability_with_priority("registry", &["routing.owner"], 10),
        ];
        assert_eq!(
            investigation_plan_schema_for_capabilities(&plain),
            investigation_plan_schema_for_capabilities(&prioritized)
        );
        let plain_request =
            build_investigation_plan_request("find the owner", &plain, Some(128), Some(7)).unwrap();
        let prioritized_request =
            build_investigation_plan_request("find the owner", &prioritized, Some(128), Some(7))
                .unwrap();
        assert_eq!(plain_request.task, prioritized_request.task);
        assert_eq!(plain_request.system, prioritized_request.system);
        assert!(!prioritized_request.task.contains("selection_priority"));
        assert!(
            !prioritized_request
                .system
                .unwrap()
                .contains("selection_priority")
        );
    }

    #[test]
    fn runtime_plan_parser_still_accepts_missing_expected_fact_key() {
        let proposal = parse_investigation_plan(
            r#"{"targets":[{"id":"owner","question":"Who owns routing?"}]}"#,
        )
        .expect("runtime parser stays broad for unconstrained provider output");
        assert_eq!(proposal.targets.len(), 1);
        assert_eq!(proposal.targets[0].expected_fact_key, None);
    }

    #[test]
    fn strict_plan_schema_generation_does_not_change_action_schema() {
        let before = investigation_action_schema();
        let capabilities = vec![
            capability("cache", &["routing.owner"]),
            capability("registry", &["routing.owner"]),
        ];
        let _ = investigation_plan_schema_for_capabilities(&capabilities);
        assert_eq!(investigation_action_schema(), before);
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
    fn plan_admission_preserves_noncanonical_expected_fact_key_without_exact_selection() {
        let targets = admit_investigation_plan(
            InvestigationPlanProposal {
                targets: vec![InvestigationTargetProposal {
                    id: "owner".into(),
                    question: "Who owns routing?".into(),
                    expected_fact_key: Some(" routing.owner ".into()),
                }],
            },
            &InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            targets[0].expected_fact_key.as_deref(),
            Some(" routing.owner ")
        );

        let mut state = InvestigationState::new(
            targets,
            vec![capability("cache", &["routing.owner"])],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(state.unique_compatible_action_proposal(), None);
        assert_eq!(
            state.validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            }),
            Err(InvestigationActionRejection::UnsupportedTargetKey)
        );
    }

    #[test]
    fn plan_admission_keeps_canonical_missing_and_empty_expected_fact_keys_unchanged() {
        let targets = admit_investigation_plan(
            InvestigationPlanProposal {
                targets: vec![
                    InvestigationTargetProposal {
                        id: "owner".into(),
                        question: "Who owns routing?".into(),
                        expected_fact_key: Some("routing.owner".into()),
                    },
                    InvestigationTargetProposal {
                        id: "region".into(),
                        question: "Which region?".into(),
                        expected_fact_key: None,
                    },
                    InvestigationTargetProposal {
                        id: "empty".into(),
                        question: "Which empty-key target?".into(),
                        expected_fact_key: Some(String::new()),
                    },
                ],
            },
            &InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            targets[0].expected_fact_key.as_deref(),
            Some("routing.owner")
        );
        assert_eq!(targets[1].expected_fact_key, None);
        assert_eq!(targets[2].expected_fact_key.as_deref(), Some(""));
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
    fn precedence_selects_unique_highest_priority_for_one_exact_target() {
        let state = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability_with_priority("registry", &["routing.owner"], 10),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(state.unique_compatible_action_proposal(), None);
        assert_eq!(
            state.unique_precedence_action_proposal(),
            Some(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
        );
    }

    #[test]
    fn precedence_then_no_result_uses_existing_exact_target_followup() {
        let mut state = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability_with_priority("registry", &["routing.owner"], 10),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        let round = state.begin_round().unwrap();
        let first = state
            .validate_action(state.unique_precedence_action_proposal().unwrap())
            .unwrap()
            .unwrap();
        state.note_harness_precedence_selection();
        state.record_observation(
            round,
            first,
            InvestigationObservationStatus::NoResult,
            0,
            false,
        );
        assert_eq!(
            state.unique_no_result_followup_proposal(),
            Some(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("registry".into()),
            })
        );
        assert_eq!(state.telemetry().harness_precedence_selections, 1);
        assert_eq!(state.telemetry().harness_unique_selections, 0);
        assert_eq!(state.telemetry().harness_no_result_followup_selections, 0);
    }

    #[test]
    fn precedence_falls_back_on_tie_or_missing_priority() {
        let tied = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability_with_priority("registry", &["routing.owner"], 20),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(tied.unique_precedence_action_proposal(), None);

        let missing = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability("registry", &["routing.owner"]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(missing.unique_precedence_action_proposal(), None);
    }

    #[test]
    fn precedence_never_merges_same_key_sibling_targets() {
        let mut state = InvestigationState::new(
            vec![
                target("owner-primary", Some("routing.owner")),
                target("owner-secondary", Some("routing.owner")),
            ],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability_with_priority("registry", &["routing.owner"], 10),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(state.unique_precedence_action_proposal(), None);

        for (capability_id, status, admitted_evidence, verification_progress) in [
            ("cache", InvestigationObservationStatus::NoResult, 0, false),
            (
                "registry",
                InvestigationObservationStatus::VerificationProgress,
                1,
                true,
            ),
        ] {
            let round = state.begin_round().unwrap();
            let action = state
                .validate_action(InvestigationActionProposal {
                    action: InvestigationActionKind::Acquire,
                    target_id: Some("owner-primary".into()),
                    capability_id: Some(capability_id.into()),
                })
                .unwrap()
                .unwrap();
            state.record_observation(
                round,
                action,
                status,
                admitted_evidence,
                verification_progress,
            );
        }

        assert_eq!(state.remaining_action_count(), 2);
        assert_eq!(state.unique_precedence_action_proposal(), None);
    }

    #[test]
    fn precedence_excludes_keyless_wildcard_non_read_only_and_attempted_pairs() {
        let keyless = InvestigationState::new(
            vec![target("owner", None)],
            vec![capability_with_priority("cache", &["routing.owner"], 20)],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(keyless.unique_precedence_action_proposal(), None);

        let wildcard = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![capability_with_priority("generic", &[], 20)],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(wildcard.unique_precedence_action_proposal(), None);

        let mut write = capability_with_priority("write", &["routing.owner"], 30);
        write.read_only = false;
        let non_read_only = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                write,
                capability_with_priority("cache", &["routing.owner"], 20),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            non_read_only.unique_precedence_action_proposal(),
            Some(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
        );

        let mut attempted = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability_with_priority("registry", &["routing.owner"], 10),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        let action = attempted
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        let round = attempted.begin_round().unwrap();
        attempted.record_observation(
            round,
            action,
            InvestigationObservationStatus::Ambiguous,
            0,
            false,
        );
        assert_eq!(
            attempted.unique_precedence_action_proposal(),
            Some(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("registry".into()),
            })
        );
    }

    #[test]
    fn precedence_respects_terminal_and_action_budget_states() {
        let mut terminal = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability_with_priority("registry", &["routing.owner"], 10),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        terminal.stop(InvestigationStopReason::RoundBudget);
        assert_eq!(terminal.unique_precedence_action_proposal(), None);

        let mut budgeted = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability_with_priority("registry", &["routing.owner"], 10),
            ],
            InvestigationPolicy {
                max_actions: 1,
                max_no_progress_rounds: 2,
                ..InvestigationPolicy::default()
            },
        )
        .unwrap();
        let round = budgeted.begin_round().unwrap();
        let action = budgeted
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        budgeted.record_observation(
            round,
            action,
            InvestigationObservationStatus::VerificationProgress,
            1,
            true,
        );
        assert_eq!(
            budgeted.telemetry().stop_reason,
            Some(InvestigationStopReason::ActionBudget)
        );
        assert_eq!(budgeted.unique_precedence_action_proposal(), None);
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
        for status in [
            InvestigationObservationStatus::AppliedEvidence,
            InvestigationObservationStatus::RejectedEvidence,
            InvestigationObservationStatus::Ambiguous,
            InvestigationObservationStatus::VerificationProgress,
            InvestigationObservationStatus::OperationalFailure,
        ] {
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
            state.record_observation(round, first, status, 0, false);
            assert_eq!(
                state.unique_no_result_followup_proposal(),
                None,
                "{status:?}"
            );
        }
    }

    #[test]
    fn no_result_followup_does_not_promote_keyless_or_wildcard_capabilities() {
        let mut keyless = InvestigationState::new(
            vec![target("owner", None)],
            vec![
                capability("cache", &[]),
                capability("registry", &["routing.owner"]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        let round = keyless.begin_round().unwrap();
        let first = keyless
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        keyless.record_observation(
            round,
            first,
            InvestigationObservationStatus::NoResult,
            0,
            false,
        );
        assert_eq!(keyless.unique_no_result_followup_proposal(), None);

        let mut wildcard = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![
                capability("cache", &["routing.owner"]),
                capability("generic", &[]),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();
        let round = wildcard.begin_round().unwrap();
        let first = wildcard
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        wildcard.record_observation(
            round,
            first,
            InvestigationObservationStatus::NoResult,
            0,
            false,
        );
        assert_eq!(wildcard.unique_no_result_followup_proposal(), None);
    }

    #[test]
    fn no_result_followup_respects_terminal_budgets() {
        for policy in [
            InvestigationPolicy {
                max_actions: 1,
                ..InvestigationPolicy::default()
            },
            InvestigationPolicy {
                max_no_progress_rounds: 1,
                ..InvestigationPolicy::default()
            },
        ] {
            let mut state = InvestigationState::new(
                vec![target("owner", Some("routing.owner"))],
                vec![
                    capability("cache", &["routing.owner"]),
                    capability("registry", &["routing.owner"]),
                ],
                policy,
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
            assert!(
                state
                    .record_observation(
                        round,
                        first,
                        InvestigationObservationStatus::NoResult,
                        0,
                        false,
                    )
                    .is_some()
            );
            assert_eq!(state.unique_no_result_followup_proposal(), None);
        }
    }

    #[test]
    fn no_result_followup_executes_validated_action_without_extra_planner_call() {
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

        state.note_planner_call();
        let first_round = state.begin_round().unwrap();
        let cache = state
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner-primary".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        state.record_observation(
            first_round,
            cache,
            InvestigationObservationStatus::NoResult,
            0,
            false,
        );

        let second_round = state.begin_round().unwrap();
        let proposal = state.unique_no_result_followup_proposal().unwrap();
        state.note_harness_no_result_followup_selection();
        let registry = state.validate_action(proposal).unwrap().unwrap();
        assert_eq!(registry.target_id, "owner-primary");
        assert_eq!(registry.capability_id, "registry");
        state.record_observation(
            second_round,
            registry,
            InvestigationObservationStatus::AppliedEvidence,
            1,
            true,
        );

        let telemetry = state.telemetry();
        assert_eq!(telemetry.planner_calls, 1);
        assert_eq!(telemetry.harness_no_result_followup_selections, 1);
        assert_eq!(telemetry.actions.len(), 2);
        assert!(telemetry.rejected_actions.is_empty());
        assert_eq!(telemetry.actions[1].action.capability_id, "registry");
        assert_eq!(telemetry.actions[1].admitted_evidence, 1);
        assert!(telemetry.actions[1].verification_progress);
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
        assert_eq!(
            plan.reasoning_preference,
            Some(ModelReasoningPreference::Minimize)
        );
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
        assert_eq!(
            action.reasoning_preference,
            Some(ModelReasoningPreference::Minimize)
        );
        assert!(
            action
                .task
                .contains("Never repeat an identical target/capability pair")
        );
        assert!(!investigation_action_schema().to_string().contains("query"));
    }

    #[test]
    fn harness_owned_selection_priority_is_model_invisible() {
        let plain = vec![capability("lookup", &["service.region"])];
        let prioritized = vec![capability_with_priority("lookup", &["service.region"], 20)];

        let plain_plan =
            build_investigation_plan_request("find the region", &plain, Some(128), Some(7))
                .unwrap();
        let prioritized_plan =
            build_investigation_plan_request("find the region", &prioritized, Some(128), Some(7))
                .unwrap();
        assert_eq!(plain_plan.task, prioritized_plan.task);
        assert!(!prioritized_plan.task.contains("selection_priority"));

        let plain_state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            plain,
            InvestigationPolicy::default(),
        )
        .unwrap();
        let prioritized_state = InvestigationState::new(
            vec![target("region", Some("service.region"))],
            prioritized,
            InvestigationPolicy::default(),
        )
        .unwrap();
        assert_eq!(
            prioritized_state.telemetry().capabilities[0].selection_priority,
            Some(20)
        );

        let plain_action = build_investigation_action_request(
            "find the region",
            plain_state.telemetry(),
            Some(128),
            Some(8),
        )
        .unwrap();
        let prioritized_action = build_investigation_action_request(
            "find the region",
            prioritized_state.telemetry(),
            Some(128),
            Some(8),
        )
        .unwrap();
        assert_eq!(plain_action.task, prioritized_action.task);
        assert!(!prioritized_action.task.contains("selection_priority"));
    }

    #[test]
    fn action_schema_structurally_discriminates_acquire_and_stop() {
        let schema = investigation_action_schema();
        let branches = schema["oneOf"].as_array().expect("closed union branches");
        assert_eq!(branches.len(), 2);

        let branch = |action: &str| {
            branches
                .iter()
                .find(|branch| branch["properties"]["action"]["const"] == action)
                .unwrap_or_else(|| panic!("missing {action} branch"))
        };

        let acquire = branch("acquire");
        assert_eq!(acquire["additionalProperties"], false);
        assert_eq!(
            acquire["required"],
            serde_json::json!(["action", "target_id", "capability_id"])
        );
        assert_eq!(acquire["properties"]["target_id"]["type"], "string");
        assert_eq!(acquire["properties"]["target_id"]["minLength"], 1);
        assert_eq!(acquire["properties"]["capability_id"]["type"], "string");
        assert_eq!(acquire["properties"]["capability_id"]["minLength"], 1);

        let stop = branch("stop");
        assert_eq!(stop["additionalProperties"], false);
        assert_eq!(stop["required"], serde_json::json!(["action"]));
        assert!(stop["properties"].get("target_id").is_none());
        assert!(stop["properties"].get("capability_id").is_none());
    }

    #[test]
    fn unconstrained_missing_capability_output_is_retained_then_rejected_fail_closed() {
        let proposal = parse_investigation_action(r#"{"action":"acquire","target_id":"owner"}"#)
            .expect("broad runtime parser retains exact provider output");
        assert_eq!(proposal.target_id.as_deref(), Some("owner"));
        assert_eq!(proposal.capability_id, None);

        let mut state = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![capability("cache", &["routing.owner"])],
            InvestigationPolicy::default(),
        )
        .unwrap();
        state.begin_round().unwrap();
        assert_eq!(
            state.validate_action(proposal),
            Err(InvestigationActionRejection::InvalidShape)
        );
    }

    #[test]
    fn invalid_shape_rejections_retain_exact_proposal_and_round() {
        let mut state = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![capability("cache", &["routing.owner"])],
            InvestigationPolicy::default(),
        )
        .unwrap();
        let round = state.begin_round().unwrap();
        let proposals = [
            InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: None,
                capability_id: Some("cache".into()),
            },
            InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("   ".into()),
            },
            InvestigationActionProposal {
                action: InvestigationActionKind::Stop,
                target_id: Some("owner".into()),
                capability_id: None,
            },
        ];

        for proposal in &proposals {
            assert_eq!(
                state.validate_action(proposal.clone()),
                Err(InvestigationActionRejection::InvalidShape)
            );
        }

        let telemetry = state.telemetry();
        assert_eq!(
            telemetry.rejected_actions[&InvestigationActionRejection::InvalidShape],
            3
        );
        assert_eq!(telemetry.action_rejection_records.len(), 3);
        for (record, proposal) in telemetry.action_rejection_records.iter().zip(proposals) {
            assert_eq!(record.round, round);
            assert_eq!(record.reason, InvestigationActionRejection::InvalidShape);
            assert_eq!(record.proposal, proposal);
        }
    }

    #[test]
    fn action_request_schema_excludes_attempted_pair_but_keeps_untried_pair_and_stop() {
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
        let action = state
            .validate_action(InvestigationActionProposal {
                action: InvestigationActionKind::Acquire,
                target_id: Some("owner".into()),
                capability_id: Some("cache".into()),
            })
            .unwrap()
            .unwrap();
        state.record_observation(
            round,
            action,
            InvestigationObservationStatus::RejectedEvidence,
            0,
            false,
        );

        let request = build_investigation_action_request(
            "find the owner",
            state.telemetry(),
            Some(128),
            Some(9),
        )
        .unwrap();
        let ModelOutputFormat::JsonSchema { schema, .. } = request.output_format else {
            panic!("action request must use JSON Schema");
        };
        let branches = schema["oneOf"].as_array().expect("closed union branches");
        let acquire_pairs = branches
            .iter()
            .filter(|branch| branch["properties"]["action"]["const"] == "acquire")
            .map(|branch| {
                (
                    branch["properties"]["target_id"]["const"]
                        .as_str()
                        .expect("target const"),
                    branch["properties"]["capability_id"]["const"]
                        .as_str()
                        .expect("capability const"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(acquire_pairs, vec![("owner", "registry")]);
        assert_eq!(
            branches
                .iter()
                .filter(|branch| branch["properties"]["action"]["const"] == "stop")
                .count(),
            1
        );

        let duplicate = parse_investigation_action(
            r#"{"action":"acquire","target_id":"owner","capability_id":"cache"}"#,
        )
        .expect("runtime parser must remain broad");
        assert_eq!(
            state.validate_action(duplicate),
            Err(InvestigationActionRejection::DuplicateAction)
        );
    }

    #[test]
    fn action_request_schema_matches_current_runtime_pair_compatibility() {
        let targets = vec![
            target("bound", Some("routing.owner")),
            target("keyless", None),
        ];
        let mut write = capability("write", &["routing.owner"]);
        write.read_only = false;
        let capabilities = vec![
            capability("exact", &["routing.owner"]),
            capability("wrong-key", &["service.region"]),
            capability("wildcard", &[]),
            write,
        ];
        let state = InvestigationState::new(
            targets.clone(),
            capabilities.clone(),
            InvestigationPolicy::default(),
        )
        .unwrap();
        let request = build_investigation_action_request(
            "inspect routing",
            state.telemetry(),
            Some(128),
            Some(17),
        )
        .unwrap();
        let ModelOutputFormat::JsonSchema { schema, .. } = request.output_format else {
            panic!("action request must use JSON Schema");
        };
        let schema_pairs = schema["oneOf"]
            .as_array()
            .expect("closed union branches")
            .iter()
            .filter(|branch| branch["properties"]["action"]["const"] == "acquire")
            .map(|branch| {
                (
                    branch["properties"]["target_id"]["const"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                    branch["properties"]["capability_id"]["const"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                )
            })
            .collect::<BTreeSet<_>>();

        let mut runtime_pairs = BTreeSet::new();
        for target in &targets {
            for capability in &capabilities {
                let mut probe = InvestigationState::new(
                    targets.clone(),
                    capabilities.clone(),
                    InvestigationPolicy::default(),
                )
                .unwrap();
                probe.begin_round().unwrap();
                if probe
                    .validate_action(InvestigationActionProposal {
                        action: InvestigationActionKind::Acquire,
                        target_id: Some(target.id.clone()),
                        capability_id: Some(capability.id.clone()),
                    })
                    .is_ok()
                {
                    runtime_pairs.insert((target.id.clone(), capability.id.clone()));
                }
            }
        }
        assert_eq!(schema_pairs, runtime_pairs);
    }

    #[test]
    fn action_request_exposes_typed_rejection_feedback_and_shape_rules() {
        let mut state = InvestigationState::new(
            vec![target("owner", Some("routing.owner"))],
            vec![capability_with_priority("cache", &["routing.owner"], 20)],
            InvestigationPolicy::default(),
        )
        .unwrap();
        state.begin_round().unwrap();
        let invalid = InvestigationActionProposal {
            action: InvestigationActionKind::Acquire,
            target_id: Some("owner".into()),
            capability_id: None,
        };
        assert_eq!(
            state.validate_action(invalid),
            Err(InvestigationActionRejection::InvalidShape)
        );

        let request = build_investigation_action_request(
            "find the owner",
            state.telemetry(),
            Some(128),
            Some(9),
        )
        .unwrap();
        assert!(
            request
                .task
                .contains("Prior typed action validation rejections")
        );
        assert!(request.task.contains("invalid_shape"));
        assert!(
            request
                .task
                .contains("For acquire, include both non-empty target_id and capability_id")
        );
        assert!(request.task.contains("For stop, omit both IDs"));
        assert!(!request.task.contains("selection_priority"));
    }

    #[test]
    fn precedence_skip_diagnostic_reports_same_key_sibling_without_changing_selection() {
        let mut state = InvestigationState::new(
            vec![
                target("owner-primary", Some("routing.owner")),
                target("owner-sibling", Some("routing.owner")),
            ],
            vec![
                capability_with_priority("cache", &["routing.owner"], 20),
                capability_with_priority("registry", &["routing.owner"], 10),
            ],
            InvestigationPolicy::default(),
        )
        .unwrap();

        assert_eq!(state.unique_precedence_action_proposal(), None);
        assert!(state.telemetry().precedence_skip_reasons.is_empty());
        assert_eq!(
            state.unique_precedence_action_proposal_with_diagnostic(),
            None
        );
        assert_eq!(
            state.telemetry().precedence_skip_reasons
                [&InvestigationPrecedenceSkipReason::SameKeySibling],
            1
        );
        assert_eq!(state.telemetry().harness_precedence_selections, 0);
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
