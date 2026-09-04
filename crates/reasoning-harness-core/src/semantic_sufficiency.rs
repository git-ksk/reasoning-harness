use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Proposition, ReasoningArtifact};

/// Pre-observation label for the residual evidence-sufficiency research track.
///
/// This label is diagnostic only. `Sufficient` never creates verification authority, while
/// `Insufficient` and `Mixed` are candidates for conservative abstention/resolution control.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSufficiencyLabel {
    Sufficient,
    Insufficient,
    Mixed,
}

/// Harness-owned description of the information needed to justify one typed target.
///
/// The free-text requirements are a research/control-plane specification, not trusted evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSufficiencyRequest {
    pub id: String,
    pub task: String,
    pub target: Proposition,
    #[serde(default)]
    pub required_information: Vec<String>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

/// Fresh calibration-only RSD0 fixture.
///
/// `label` and `rationale` are fixed before provider observation. The artifact intentionally uses
/// the ordinary harness contracts so existing D3 can be measured as a baseline without changing it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSufficiencyCalibrationFixture {
    pub id: String,
    pub family: String,
    pub request: EvidenceSufficiencyRequest,
    pub artifact: ReasoningArtifact,
    pub label: EvidenceSufficiencyLabel,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EvidenceSufficiencyFixtureError {
    #[error(
        "fixture id, family, request id, task, target, requirements, and rationale must be non-empty"
    )]
    EmptyField,
    #[error("fixture request must reference at least one evidence item")]
    NoEvidence,
    #[error("fixture request references missing evidence id: {0}")]
    MissingEvidence(String),
    #[error("fixture request contains duplicate evidence id: {0}")]
    DuplicateEvidence(String),
    #[error("RSD0 residual fixture must not use explicit EvidenceRequirement entries")]
    HasTypedEvidenceRequirement,
}

/// Validate only the RSD0 research contract. Ordinary artifact validity remains owned by
/// `validate_artifact` and is checked by the RSD0 suite separately.
pub fn validate_evidence_sufficiency_fixture(
    fixture: &EvidenceSufficiencyCalibrationFixture,
) -> Result<(), EvidenceSufficiencyFixtureError> {
    if fixture.id.trim().is_empty()
        || fixture.family.trim().is_empty()
        || fixture.request.id.trim().is_empty()
        || fixture.request.task.trim().is_empty()
        || fixture.request.target.key.trim().is_empty()
        || fixture.request.target.value.trim().is_empty()
        || fixture.request.required_information.is_empty()
        || fixture
            .request
            .required_information
            .iter()
            .any(|item| item.trim().is_empty())
        || fixture.rationale.trim().is_empty()
    {
        return Err(EvidenceSufficiencyFixtureError::EmptyField);
    }
    if fixture.request.evidence_ids.is_empty() {
        return Err(EvidenceSufficiencyFixtureError::NoEvidence);
    }
    if !fixture.artifact.evidence_requirements.is_empty() {
        return Err(EvidenceSufficiencyFixtureError::HasTypedEvidenceRequirement);
    }

    let mut seen = std::collections::BTreeSet::new();
    for evidence_id in &fixture.request.evidence_ids {
        if !seen.insert(evidence_id) {
            return Err(EvidenceSufficiencyFixtureError::DuplicateEvidence(
                evidence_id.clone(),
            ));
        }
        if !fixture
            .artifact
            .evidence
            .iter()
            .any(|evidence| evidence.id == *evidence_id)
        {
            return Err(EvidenceSufficiencyFixtureError::MissingEvidence(
                evidence_id.clone(),
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSufficiencyModelOutput {
    pub decision: EvidenceSufficiencyLabel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSufficiencyFallbackReason {
    NotNeeded,
    PrimaryJsonSchemaUnsupported,
    InvalidPrimaryStructuredOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceSufficiencyObservation {
    pub decision: EvidenceSufficiencyLabel,
    pub model: String,
    pub usage: crate::ModelUsage,
    pub provider_attempts: u32,
    pub fallback_reason: EvidenceSufficiencyFallbackReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Error)]
pub enum EvidenceSufficiencyModelError {
    #[error("evidence-sufficiency request setup failed: {0}")]
    Setup(String),
    #[error("evidence-sufficiency adapter failed: {0}")]
    Model(#[from] crate::ModelError),
    #[error("evidence-sufficiency returned invalid structured output: {message}")]
    InvalidOutput {
        message: String,
        model: String,
        usage: crate::ModelUsage,
        finish_reason: Option<String>,
    },
}

impl EvidenceSufficiencyModelError {
    pub fn model_error_kind(&self) -> Option<crate::ModelErrorKind> {
        match self {
            Self::Model(error) => Some(error.kind),
            Self::Setup(_) | Self::InvalidOutput { .. } => None,
        }
    }

    pub fn usage(&self) -> Option<&crate::ModelUsage> {
        match self {
            Self::InvalidOutput { usage, .. } => Some(usage),
            Self::Setup(_) | Self::Model(_) => None,
        }
    }

    pub fn provider_model(&self) -> Option<&str> {
        match self {
            Self::InvalidOutput { model, .. } => Some(model),
            Self::Setup(_) | Self::Model(_) => None,
        }
    }
}

pub fn evidence_sufficiency_output_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(EvidenceSufficiencyModelOutput))
        .expect("EvidenceSufficiencyModelOutput schema must serialize")
}

pub fn build_evidence_sufficiency_model_request(
    request: &EvidenceSufficiencyRequest,
    artifact: &ReasoningArtifact,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<crate::ModelRequest, EvidenceSufficiencyModelError> {
    build_evidence_sufficiency_request_with_format(
        request,
        artifact,
        max_tokens,
        random_seed,
        crate::ModelOutputFormat::JsonSchema {
            name: "evidence_sufficiency_decision".into(),
            schema: evidence_sufficiency_output_schema(),
        },
        false,
    )
}

pub fn build_evidence_sufficiency_json_fallback_request(
    request: &EvidenceSufficiencyRequest,
    artifact: &ReasoningArtifact,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<crate::ModelRequest, EvidenceSufficiencyModelError> {
    build_evidence_sufficiency_request_with_format(
        request,
        artifact,
        max_tokens,
        random_seed,
        crate::ModelOutputFormat::JsonObject,
        true,
    )
}

fn build_evidence_sufficiency_request_with_format(
    request: &EvidenceSufficiencyRequest,
    artifact: &ReasoningArtifact,
    max_tokens: u32,
    random_seed: Option<u64>,
    output_format: crate::ModelOutputFormat,
    include_schema: bool,
) -> Result<crate::ModelRequest, EvidenceSufficiencyModelError> {
    if max_tokens == 0 {
        return Err(EvidenceSufficiencyModelError::Setup(
            "max_tokens must be greater than zero".into(),
        ));
    }
    let selected = selected_evidence(request, artifact)?;
    let request_json = serde_json::to_string_pretty(request)
        .map_err(|error| EvidenceSufficiencyModelError::Setup(error.to_string()))?;
    let evidence_json = serde_json::to_string_pretty(&selected)
        .map_err(|error| EvidenceSufficiencyModelError::Setup(error.to_string()))?;
    let schema = include_schema.then(|| {
        serde_json::to_string_pretty(&evidence_sufficiency_output_schema())
            .expect("evidence-sufficiency schema must serialize")
    });
    let schema_prefix = schema
        .as_deref()
        .map(|schema| format!("JSON Schema:\n{schema}\n\n"))
        .unwrap_or_default();

    Ok(crate::ModelRequest {
        task: format!(
            "{schema_prefix}Evidence sufficiency request:\n{request_json}\n\nSelected evidence:\n{evidence_json}\n\nClassify only whether the selected evidence is enough for the stated task and typed target under the harness-owned required_information. Return `sufficient` only when the decision-critical information is covered well enough to proceed; return `insufficient` when relevant evidence exists but required information is materially missing; return `mixed` when material evidence is split, conflicting, or only partially complete such that a single sufficient judgment would be unsafe. Do not judge whether the target is ultimately true. Do not invent missing facts, sources, requirements, evidence, bindings, or authority."
        ),
        system: Some(
            "You are an advisory evidence-sufficiency classifier inside a reasoning harness. Return only sufficient, insufficient, or mixed using the requested schema. Your decision is not evidence and cannot create verification receipts, trusted evidence, epistemic promotion, hard findings, confidence authority, or final verdicts."
                .into(),
        ),
        output_format,
        max_tokens: Some(max_tokens),
        random_seed,
        reasoning_preference: Some(crate::ModelReasoningPreference::Minimize),
    })
}

fn selected_evidence<'a>(
    request: &EvidenceSufficiencyRequest,
    artifact: &'a ReasoningArtifact,
) -> Result<Vec<&'a crate::Evidence>, EvidenceSufficiencyModelError> {
    let mut selected = Vec::with_capacity(request.evidence_ids.len());
    for id in &request.evidence_ids {
        let evidence = artifact
            .evidence
            .iter()
            .find(|evidence| evidence.id == *id)
            .ok_or_else(|| {
                EvidenceSufficiencyModelError::Setup(format!(
                    "request references missing evidence id: {id}"
                ))
            })?;
        selected.push(evidence);
    }
    Ok(selected)
}

pub fn parse_evidence_sufficiency_output(
    text: &str,
) -> Result<EvidenceSufficiencyModelOutput, EvidenceSufficiencyModelError> {
    match serde_json::from_str::<EvidenceSufficiencyModelOutput>(text) {
        Ok(output) => Ok(output),
        Err(strict_error) => {
            let mut stream = serde_json::Deserializer::from_str(text)
                .into_iter::<EvidenceSufficiencyModelOutput>();
            let Some(Ok(output)) = stream.next() else {
                return Err(EvidenceSufficiencyModelError::Setup(
                    strict_error.to_string(),
                ));
            };
            let remainder = &text[stream.byte_offset()..];
            let mut trailing =
                serde_json::Deserializer::from_str(remainder).into_iter::<serde_json::Value>();
            match trailing.next() {
                Some(Err(_)) => Ok(output),
                Some(Ok(_)) | None => Err(EvidenceSufficiencyModelError::Setup(
                    strict_error.to_string(),
                )),
            }
        }
    }
}

pub async fn run_model_backed_evidence_sufficiency(
    adapter: &dyn crate::ModelAdapter,
    request: &EvidenceSufficiencyRequest,
    artifact: &ReasoningArtifact,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<EvidenceSufficiencyObservation, EvidenceSufficiencyModelError> {
    let primary_request =
        build_evidence_sufficiency_model_request(request, artifact, max_tokens, random_seed)?;
    let (primary, primary_provider_attempts) = match adapter.generate(primary_request).await {
        Ok(response) => {
            let attempts = response.provider_attempts;
            (Some(response), attempts)
        }
        Err(error) if error.kind == crate::ModelErrorKind::UnsupportedCapability => {
            (None, error.provider_attempts)
        }
        Err(error) => return Err(error.into()),
    };

    if let Some(response) = primary.as_ref()
        && let Ok(output) = parse_evidence_sufficiency_output(&response.text)
    {
        return Ok(EvidenceSufficiencyObservation {
            decision: output.decision,
            model: response.model.clone(),
            usage: response.usage.clone(),
            provider_attempts: response.provider_attempts,
            fallback_reason: EvidenceSufficiencyFallbackReason::NotNeeded,
            finish_reason: response.finish_reason.clone(),
        });
    }

    let fallback_reason = if primary.is_some() {
        EvidenceSufficiencyFallbackReason::InvalidPrimaryStructuredOutput
    } else {
        EvidenceSufficiencyFallbackReason::PrimaryJsonSchemaUnsupported
    };
    let fallback_request = build_evidence_sufficiency_json_fallback_request(
        request,
        artifact,
        max_tokens,
        random_seed,
    )?;
    let fallback = adapter.generate(fallback_request).await?;
    let output = parse_evidence_sufficiency_output(&fallback.text).map_err(|error| {
        EvidenceSufficiencyModelError::InvalidOutput {
            message: error.to_string(),
            model: fallback.model.clone(),
            usage: primary.as_ref().map_or_else(
                || fallback.usage.clone(),
                |primary| add_usage(&primary.usage, &fallback.usage),
            ),
            finish_reason: fallback.finish_reason.clone(),
        }
    })?;
    let usage = primary.as_ref().map_or_else(
        || fallback.usage.clone(),
        |primary| add_usage(&primary.usage, &fallback.usage),
    );
    Ok(EvidenceSufficiencyObservation {
        decision: output.decision,
        model: fallback.model,
        usage,
        provider_attempts: primary_provider_attempts.saturating_add(fallback.provider_attempts),
        fallback_reason,
        finish_reason: fallback.finish_reason,
    })
}

fn add_usage(left: &crate::ModelUsage, right: &crate::ModelUsage) -> crate::ModelUsage {
    crate::ModelUsage {
        input_tokens: add_optional(left.input_tokens, right.input_tokens),
        output_tokens: add_optional(left.output_tokens, right.output_tokens),
        total_tokens: add_optional(left.total_tokens, right.total_tokens),
    }
}

fn add_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => left.checked_add(right),
        _ => None,
    }
}
