use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ModelAdapter, ModelBackedSoftJudgeError, ModelError, ModelErrorKind, ModelOutputFormat,
    ModelReasoningPreference, ModelRequest, ModelUsage, Proposition, SemanticDiagnosticKind,
    SemanticDiagnosticTarget, SoftJudgeDecision, SoftJudgeOutput, SoftJudgeRequest,
    SoftSemanticFinding,
};

pub const R2_MATERIALIZATION_CAPABILITY_ID: &str = "materialization-r2-capability-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationFailureClass {
    StudySetup,
    Credentials,
    Transport,
    ProviderError,
    RateLimit,
    Quota,
    ProviderUnavailable,
    Timeout,
    ProviderProtocol,
    UnsupportedCapability,
    MaterializationProtocol,
    TruncationProtocol,
    ProviderGenerationError,
}

impl MaterializationFailureClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::StudySetup => "study_setup",
            Self::Credentials => "credentials",
            Self::Transport => "transport",
            Self::ProviderError => "provider_error",
            Self::RateLimit => "rate_limit",
            Self::Quota => "quota",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::Timeout => "timeout",
            Self::ProviderProtocol => "provider_protocol",
            Self::UnsupportedCapability => "unsupported_capability",
            Self::MaterializationProtocol => "materialization_protocol",
            Self::TruncationProtocol => "truncation_protocol",
            Self::ProviderGenerationError => "provider_generation_error",
        }
    }
}

impl std::fmt::Display for MaterializationFailureClass {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializationCapabilityPreflight {
    pub capability_id: &'static str,
    pub materialization_contract: &'static str,
    pub protocol_compatible: bool,
    pub observed_decision: SoftJudgeDecision,
    pub model: String,
    pub usage: ModelUsage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationRepresentation {
    DecisionNoteObject,
    CompactDecisionNoteObject,
    NestedDecisionNoteObject,
}

impl MaterializationRepresentation {
    pub const ALL: [Self; 3] = [
        Self::DecisionNoteObject,
        Self::CompactDecisionNoteObject,
        Self::NestedDecisionNoteObject,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::DecisionNoteObject => "decision_note_object",
            Self::CompactDecisionNoteObject => "compact_decision_note_object",
            Self::NestedDecisionNoteObject => "nested_decision_note_object",
        }
    }
}

/// Research-only R2 output. The model owns only the semantic decision and an optional advisory
/// note; request-known finding identity/binding remains harness-owned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MaterializedDecisionOutput {
    pub decision: SoftJudgeDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CompactMaterializedDecisionOutput {
    d: SoftJudgeDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    n: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NestedMaterializedDecisionOutput {
    result: MaterializedDecisionOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializationObservation {
    pub representation: MaterializationRepresentation,
    pub decision: SoftJudgeDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory_note: Option<String>,
    pub materialized_output: SoftJudgeOutput,
    pub model: String,
    pub usage: ModelUsage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Error)]
pub enum MaterializationError {
    #[error("materialization request setup failed: {0}")]
    Setup(String),
    #[error("materialization adapter failed: {0}")]
    Model(#[from] ModelError),
    #[error("materialization returned invalid structured output: {message}")]
    InvalidOutput {
        message: String,
        model: String,
        usage: ModelUsage,
        finish_reason: Option<String>,
    },
}

impl MaterializationError {
    pub fn model_error_kind(&self) -> Option<ModelErrorKind> {
        match self {
            Self::Model(error) => Some(error.kind),
            Self::Setup(_) | Self::InvalidOutput { .. } => None,
        }
    }

    pub fn usage(&self) -> Option<&ModelUsage> {
        match self {
            Self::Setup(_) | Self::Model(_) => None,
            Self::InvalidOutput { usage, .. } => Some(usage),
        }
    }

    pub fn provider_model(&self) -> Option<&str> {
        match self {
            Self::Setup(_) | Self::Model(_) => None,
            Self::InvalidOutput { model, .. } => Some(model),
        }
    }

    pub fn finish_reason(&self) -> Option<&str> {
        match self {
            Self::Setup(_) | Self::Model(_) => None,
            Self::InvalidOutput { finish_reason, .. } => finish_reason.as_deref(),
        }
    }
}

pub fn classify_materialization_failure(
    error: &MaterializationError,
) -> MaterializationFailureClass {
    match error {
        MaterializationError::Setup(_) => MaterializationFailureClass::StudySetup,
        MaterializationError::Model(error) => match error.kind {
            ModelErrorKind::Credentials => MaterializationFailureClass::Credentials,
            ModelErrorKind::Transport => MaterializationFailureClass::Transport,
            ModelErrorKind::Provider => MaterializationFailureClass::ProviderError,
            ModelErrorKind::RateLimit => MaterializationFailureClass::RateLimit,
            ModelErrorKind::Quota => MaterializationFailureClass::Quota,
            ModelErrorKind::ProviderUnavailable => MaterializationFailureClass::ProviderUnavailable,
            ModelErrorKind::Timeout => MaterializationFailureClass::Timeout,
            ModelErrorKind::Protocol => MaterializationFailureClass::ProviderProtocol,
            ModelErrorKind::UnsupportedCapability => {
                MaterializationFailureClass::UnsupportedCapability
            }
        },
        MaterializationError::InvalidOutput { finish_reason, .. }
            if finish_reason
                .as_deref()
                .is_some_and(is_truncation_finish_reason) =>
        {
            MaterializationFailureClass::TruncationProtocol
        }
        MaterializationError::InvalidOutput { finish_reason, .. }
            if finish_reason
                .as_deref()
                .is_some_and(is_provider_generation_error_finish_reason) =>
        {
            MaterializationFailureClass::ProviderGenerationError
        }
        MaterializationError::InvalidOutput { .. } => {
            MaterializationFailureClass::MaterializationProtocol
        }
    }
}

/// Performs one protocol-only R2 capability probe. The observed semantic decision is deliberately
/// not scored: compatibility means only that the provider returned a payload that satisfies the
/// frozen decision-only materialization contract. This probe is independent of all calibration and
/// holdout corpora.
pub async fn run_materialization_capability_preflight(
    adapter: &dyn ModelAdapter,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<MaterializationCapabilityPreflight, MaterializationError> {
    let request = materialization_capability_preflight_request();
    let observation =
        run_model_backed_soft_judge_materialization(adapter, &request, max_tokens, random_seed)
            .await?;
    Ok(MaterializationCapabilityPreflight {
        capability_id: R2_MATERIALIZATION_CAPABILITY_ID,
        materialization_contract: crate::MATERIALIZATION_R2_CONTRACT_ID,
        protocol_compatible: true,
        observed_decision: observation.decision,
        model: observation.model,
        usage: observation.usage,
        finish_reason: observation.finish_reason,
    })
}

fn materialization_capability_preflight_request() -> SoftJudgeRequest {
    SoftJudgeRequest {
        id: "materialization-r2-capability-preflight-v1".into(),
        task: "Does the supplied context contradict the target proposition?".into(),
        kind: SemanticDiagnosticKind::Contradiction,
        target: SemanticDiagnosticTarget::Proposition {
            proposition: Proposition {
                key: "protocol.preflight".into(),
                value: "compatible".into(),
            },
        },
        context: vec![
            "For this protocol-only compatibility probe, protocol.preflight is compatible.".into(),
        ],
    }
}

fn is_truncation_finish_reason(reason: &str) -> bool {
    matches!(
        reason.trim().to_ascii_lowercase().as_str(),
        "length" | "max_tokens" | "max_output_tokens"
    )
}

fn is_provider_generation_error_finish_reason(reason: &str) -> bool {
    reason.trim().eq_ignore_ascii_case("error")
}

pub async fn run_model_backed_soft_judge_materialization(
    adapter: &dyn ModelAdapter,
    request: &SoftJudgeRequest,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<MaterializationObservation, MaterializationError> {
    run_model_backed_soft_judge_materialization_representation(
        adapter,
        request,
        MaterializationRepresentation::DecisionNoteObject,
        max_tokens,
        random_seed,
    )
    .await
}

pub async fn run_model_backed_soft_judge_materialization_representation(
    adapter: &dyn ModelAdapter,
    request: &SoftJudgeRequest,
    representation: MaterializationRepresentation,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<MaterializationObservation, MaterializationError> {
    let model_request = build_soft_judge_materialization_representation_request(
        request,
        representation,
        max_tokens,
        random_seed,
    )
    .map_err(|error| MaterializationError::Setup(error.to_string()))?;
    let response = adapter.generate(model_request).await?;
    let output = parse_materialized_decision_representation_output(representation, &response.text)
        .map_err(|error| MaterializationError::InvalidOutput {
            message: error.to_string(),
            model: response.model.clone(),
            usage: response.usage.clone(),
            finish_reason: response.finish_reason.clone(),
        })?;
    let materialized_output = materialize_soft_judge_output(request, &output);
    crate::semantic_judge::validate_output(request, &materialized_output).map_err(|error| {
        MaterializationError::InvalidOutput {
            message: error.to_string(),
            model: response.model.clone(),
            usage: response.usage.clone(),
            finish_reason: response.finish_reason.clone(),
        }
    })?;

    Ok(MaterializationObservation {
        representation,
        decision: output.decision,
        advisory_note: output.advisory_note,
        materialized_output,
        model: response.model,
        usage: response.usage,
        finish_reason: response.finish_reason,
    })
}

/// R2 changes the model-facing ownership contract intentionally: decision semantics remain the
/// v3 decision rule, while model-owned echoes of request-known kind/target are removed.
pub fn build_soft_judge_materialization_request(
    request: &SoftJudgeRequest,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<ModelRequest, ModelBackedSoftJudgeError> {
    build_soft_judge_materialization_representation_request(
        request,
        MaterializationRepresentation::DecisionNoteObject,
        max_tokens,
        random_seed,
    )
}

pub fn build_soft_judge_materialization_representation_request(
    request: &SoftJudgeRequest,
    representation: MaterializationRepresentation,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<ModelRequest, ModelBackedSoftJudgeError> {
    let request_json = serde_json::to_string_pretty(request)
        .map_err(|error| ModelBackedSoftJudgeError::InvalidStructuredOutput(error.to_string()))?;
    let decision_guidance = crate::semantic_judge::soft_judge_decision_guidance(request.kind);
    let schema = materialization_representation_schema(representation)?;

    Ok(ModelRequest {
        task: format!(
            "Evaluate this semantic diagnostic request:\n{request_json}\n\nDecision rule:\n{decision_guidance}\n\nUse only the supplied context. Return finding when the context affirmatively supports the requested diagnostic concern, no_finding when the context affirmatively resolves or negates that concern, and abstain only when the context is genuinely insufficient, mixed, or ambiguous. Return only the semantic decision plus an optional advisory_note. The harness owns finding kind and target and will copy them deterministically from the request only when decision=finding; do not return or infer those binding fields. The advisory note is untrusted explanation only and cannot add evidence or verification authority."
        ),
        system: Some(
            "You are a soft semantic diagnostic judge inside a reasoning harness. Your output is advisory only. Return finding, no_finding, or abstain plus an optional advisory_note using the requested schema. The harness, not the model, owns finding kind and target. You cannot create verification receipts, hard findings, epistemic-state promotion, verdicts, trusted evidence, or hidden-chain-of-thought grades."
                .into(),
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: match representation {
                MaterializationRepresentation::DecisionNoteObject => {
                    "soft_judge_materialized_decision".into()
                }
                _ => format!("soft_judge_materialized_{}", representation.id()),
            },
            schema,
        },
        max_tokens: Some(max_tokens),
        random_seed,
        reasoning_preference: Some(ModelReasoningPreference::Minimize),
    })
}

fn materialization_representation_schema(
    representation: MaterializationRepresentation,
) -> Result<serde_json::Value, ModelBackedSoftJudgeError> {
    let value = match representation {
        MaterializationRepresentation::DecisionNoteObject => {
            serde_json::to_value(schema_for!(MaterializedDecisionOutput))
        }
        MaterializationRepresentation::CompactDecisionNoteObject => {
            serde_json::to_value(schema_for!(CompactMaterializedDecisionOutput))
        }
        MaterializationRepresentation::NestedDecisionNoteObject => {
            serde_json::to_value(schema_for!(NestedMaterializedDecisionOutput))
        }
    };
    value.map_err(|error| ModelBackedSoftJudgeError::InvalidStructuredOutput(error.to_string()))
}

pub fn parse_materialized_decision_representation_output(
    representation: MaterializationRepresentation,
    text: &str,
) -> Result<MaterializedDecisionOutput, ModelBackedSoftJudgeError> {
    match representation {
        MaterializationRepresentation::DecisionNoteObject => {
            parse_materialized_decision_output(text)
        }
        MaterializationRepresentation::CompactDecisionNoteObject => {
            let output: CompactMaterializedDecisionOutput =
                parse_one_materialized_json_value(text)?;
            Ok(MaterializedDecisionOutput {
                decision: output.d,
                advisory_note: output.n,
            })
        }
        MaterializationRepresentation::NestedDecisionNoteObject => {
            let output: NestedMaterializedDecisionOutput = parse_one_materialized_json_value(text)?;
            Ok(output.result)
        }
    }
}

pub fn parse_materialized_decision_output(
    text: &str,
) -> Result<MaterializedDecisionOutput, ModelBackedSoftJudgeError> {
    parse_one_materialized_json_value(text)
}

fn parse_one_materialized_json_value<T: for<'de> Deserialize<'de>>(
    text: &str,
) -> Result<T, ModelBackedSoftJudgeError> {
    match serde_json::from_str::<T>(text) {
        Ok(output) => Ok(output),
        Err(strict_error) => {
            let mut stream = serde_json::Deserializer::from_str(text).into_iter::<T>();
            let Some(Ok(output)) = stream.next() else {
                return Err(ModelBackedSoftJudgeError::InvalidStructuredOutput(
                    strict_error.to_string(),
                ));
            };
            let remainder = &text[stream.byte_offset()..];
            let mut trailing_values =
                serde_json::Deserializer::from_str(remainder).into_iter::<serde_json::Value>();
            match trailing_values.next() {
                Some(Ok(_)) | None => Err(ModelBackedSoftJudgeError::InvalidStructuredOutput(
                    strict_error.to_string(),
                )),
                Some(Err(_)) => Ok(output),
            }
        }
    }
}

pub fn materialize_soft_judge_output(
    request: &SoftJudgeRequest,
    model_output: &MaterializedDecisionOutput,
) -> SoftJudgeOutput {
    let finding =
        (model_output.decision == SoftJudgeDecision::Finding).then(|| SoftSemanticFinding {
            kind: request.kind,
            target: request.target.clone(),
            note: model_output.advisory_note.clone(),
        });
    SoftJudgeOutput {
        decision: model_output.decision,
        finding,
    }
}
