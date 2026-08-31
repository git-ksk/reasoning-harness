use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ModelAdapter, ModelBackedSoftJudgeError, ModelError, ModelErrorKind, ModelOutputFormat,
    ModelReasoningPreference, ModelRequest, ModelUsage, SoftJudgeDecision, SoftJudgeOutput,
    SoftJudgeRequest, SoftSemanticFinding,
};

/// Research-only R2 output. The model owns only the semantic decision and an optional advisory
/// note; request-known finding identity/binding remains harness-owned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MaterializedDecisionOutput {
    pub decision: SoftJudgeDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub advisory_note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterializationObservation {
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

pub async fn run_model_backed_soft_judge_materialization(
    adapter: &dyn ModelAdapter,
    request: &SoftJudgeRequest,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<MaterializationObservation, MaterializationError> {
    let model_request = build_soft_judge_materialization_request(request, max_tokens, random_seed)
        .map_err(|error| MaterializationError::Setup(error.to_string()))?;
    let response = adapter.generate(model_request).await?;
    let output = parse_materialized_decision_output(&response.text).map_err(|error| {
        MaterializationError::InvalidOutput {
            message: error.to_string(),
            model: response.model.clone(),
            usage: response.usage.clone(),
            finish_reason: response.finish_reason.clone(),
        }
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
    let request_json = serde_json::to_string_pretty(request)
        .map_err(|error| ModelBackedSoftJudgeError::InvalidStructuredOutput(error.to_string()))?;
    let decision_guidance = crate::semantic_judge::soft_judge_decision_guidance(request.kind);
    let schema = serde_json::to_value(schema_for!(MaterializedDecisionOutput))
        .map_err(|error| ModelBackedSoftJudgeError::InvalidStructuredOutput(error.to_string()))?;

    Ok(ModelRequest {
        task: format!(
            "Evaluate this semantic diagnostic request:\n{request_json}\n\nDecision rule:\n{decision_guidance}\n\nUse only the supplied context. Return finding when the context affirmatively supports the requested diagnostic concern, no_finding when the context affirmatively resolves or negates that concern, and abstain only when the context is genuinely insufficient, mixed, or ambiguous. Return only the semantic decision plus an optional advisory_note. The harness owns finding kind and target and will copy them deterministically from the request only when decision=finding; do not return or infer those binding fields. The advisory note is untrusted explanation only and cannot add evidence or verification authority."
        ),
        system: Some(
            "You are a soft semantic diagnostic judge inside a reasoning harness. Your output is advisory only. Return finding, no_finding, or abstain plus an optional advisory_note using the requested schema. The harness, not the model, owns finding kind and target. You cannot create verification receipts, hard findings, epistemic-state promotion, verdicts, trusted evidence, or hidden-chain-of-thought grades."
                .into(),
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: "soft_judge_materialized_decision".into(),
            schema,
        },
        max_tokens: Some(max_tokens),
        random_seed,
        reasoning_preference: Some(ModelReasoningPreference::Minimize),
    })
}

pub fn parse_materialized_decision_output(
    text: &str,
) -> Result<MaterializedDecisionOutput, ModelBackedSoftJudgeError> {
    match serde_json::from_str::<MaterializedDecisionOutput>(text) {
        Ok(output) => Ok(output),
        Err(strict_error) => {
            let mut stream =
                serde_json::Deserializer::from_str(text).into_iter::<MaterializedDecisionOutput>();
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
