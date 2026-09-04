use std::{collections::BTreeMap, future::Future, pin::Pin};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CausalRelation, ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat,
    ModelReasoningPreference, ModelRequest, ModelUsage, Proposition, soft_judge_output_schema,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDiagnosticKind {
    Contradiction,
    Counterexample,
    UnsupportedPremise,
    CausalGap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SemanticDiagnosticTarget {
    Proposition { proposition: Proposition },
    CausalRelation { relation: CausalRelation },
    Claim { claim_id: String },
    Inference { inference_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SoftJudgeRequest {
    pub id: String,
    pub task: String,
    pub kind: SemanticDiagnosticKind,
    pub target: SemanticDiagnosticTarget,
    #[serde(default)]
    pub context: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SoftJudgeIdentity {
    pub judge_id: String,
    pub model_id: String,
    pub configuration_id: String,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SoftJudgeDecision {
    Finding,
    NoFinding,
    Abstain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SoftSemanticFinding {
    pub kind: SemanticDiagnosticKind,
    pub target: SemanticDiagnosticTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SoftJudgeOutput {
    pub decision: SoftJudgeDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<SoftSemanticFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftJudgeObservation {
    pub judge: SoftJudgeIdentity,
    pub request_id: String,
    pub decision: SoftJudgeDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<SoftSemanticFinding>,
}

pub trait SoftDiagnosticJudge: Send + Sync {
    /// Harness/adapter-owned provenance. Model output cannot choose this identity.
    fn identity(&self) -> SoftJudgeIdentity;

    /// Returns only the untrusted semantic decision payload. The harness attaches provenance.
    fn judge<'a>(
        &'a self,
        request: &'a SoftJudgeRequest,
    ) -> Pin<Box<dyn Future<Output = Result<SoftJudgeOutput, SoftJudgeError>> + Send + 'a>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SoftJudgeError {
    #[error("soft judge failed: {0}")]
    Judge(String),
    #[error("soft judge identity fields must not be empty")]
    InvalidIdentity,
    #[error("soft judge output is invalid: {0}")]
    InvalidOutput(String),
}

pub async fn run_soft_judge(
    judge: &dyn SoftDiagnosticJudge,
    request: &SoftJudgeRequest,
) -> Result<SoftJudgeObservation, SoftJudgeError> {
    let identity = judge.identity();
    validate_identity(&identity)?;
    let output = judge.judge(request).await?;
    validate_output(request, &output)?;
    Ok(SoftJudgeObservation {
        judge: identity,
        request_id: request.id.clone(),
        decision: output.decision,
        finding: output.finding,
    })
}

pub(crate) fn validate_output(
    request: &SoftJudgeRequest,
    output: &SoftJudgeOutput,
) -> Result<(), SoftJudgeError> {
    match (output.decision, &output.finding) {
        (SoftJudgeDecision::Finding, None) => Err(SoftJudgeError::InvalidOutput(
            "finding decision must include a typed soft finding".into(),
        )),
        (SoftJudgeDecision::NoFinding | SoftJudgeDecision::Abstain, Some(_)) => Err(
            SoftJudgeError::InvalidOutput("non-finding decision must not include a finding".into()),
        ),
        (SoftJudgeDecision::Finding, Some(finding))
            if finding.kind != request.kind || finding.target != request.target =>
        {
            Err(SoftJudgeError::InvalidOutput(
                "soft finding kind/target does not match request".into(),
            ))
        }
        _ => Ok(()),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelBackedSoftJudgeObservation {
    pub observation: SoftJudgeObservation,
    pub model: String,
    pub usage: ModelUsage,
    pub provider_attempts: u32,
    pub fallback_reason: SoftJudgeFallbackReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoftJudgeFallbackReason {
    NotNeeded,
    PrimaryJsonSchemaUnsupported,
    InvalidPrimaryStructuredOutput,
}

#[derive(Debug, Error)]
pub enum ModelBackedSoftJudgeError {
    #[error("model-backed soft judge adapter failed: {0}")]
    Model(#[from] ModelError),
    #[error("model-backed soft judge returned invalid structured output: {0}")]
    InvalidStructuredOutput(String),
    #[error(transparent)]
    SoftJudge(#[from] SoftJudgeError),
}

impl ModelBackedSoftJudgeError {
    pub fn model_error_kind(&self) -> Option<ModelErrorKind> {
        match self {
            Self::Model(error) => Some(error.kind),
            Self::InvalidStructuredOutput(_) | Self::SoftJudge(_) => None,
        }
    }
}

pub struct ModelBackedSoftJudge<'a> {
    adapter: &'a dyn ModelAdapter,
    identity: SoftJudgeIdentity,
    max_tokens: u32,
    random_seed: Option<u64>,
}

impl<'a> ModelBackedSoftJudge<'a> {
    pub fn new(
        adapter: &'a dyn ModelAdapter,
        identity: SoftJudgeIdentity,
        max_tokens: u32,
        random_seed: Option<u64>,
    ) -> Self {
        Self {
            adapter,
            identity,
            max_tokens,
            random_seed,
        }
    }
}

impl SoftDiagnosticJudge for ModelBackedSoftJudge<'_> {
    fn identity(&self) -> SoftJudgeIdentity {
        self.identity.clone()
    }

    fn judge<'a>(
        &'a self,
        request: &'a SoftJudgeRequest,
    ) -> Pin<Box<dyn Future<Output = Result<SoftJudgeOutput, SoftJudgeError>> + Send + 'a>> {
        Box::pin(async move {
            let result = run_model_backed_soft_judge(
                self.adapter,
                self.identity.clone(),
                request,
                self.max_tokens,
                self.random_seed,
            )
            .await
            .map_err(|error| SoftJudgeError::Judge(error.to_string()))?;
            Ok(SoftJudgeOutput {
                decision: result.observation.decision,
                finding: result.observation.finding,
            })
        })
    }
}

pub async fn run_model_backed_soft_judge(
    adapter: &dyn ModelAdapter,
    identity: SoftJudgeIdentity,
    request: &SoftJudgeRequest,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<ModelBackedSoftJudgeObservation, ModelBackedSoftJudgeError> {
    validate_identity(&identity)?;
    let primary_request = build_soft_judge_model_request(request, max_tokens, random_seed)?;
    let (primary, primary_provider_attempts) = match adapter.generate(primary_request).await {
        Ok(response) => {
            let attempts = response.provider_attempts;
            (Some(response), attempts)
        }
        Err(error) if error.kind == ModelErrorKind::UnsupportedCapability => {
            (None, error.provider_attempts)
        }
        Err(error) => return Err(error.into()),
    };

    if let Some(response) = primary.as_ref() {
        if let Ok(output) = parse_and_validate_soft_output(request, &response.text) {
            return Ok(ModelBackedSoftJudgeObservation {
                observation: SoftJudgeObservation {
                    judge: identity,
                    request_id: request.id.clone(),
                    decision: output.decision,
                    finding: output.finding,
                },
                model: response.model.clone(),
                usage: response.usage.clone(),
                provider_attempts: response.provider_attempts,
                fallback_reason: SoftJudgeFallbackReason::NotNeeded,
            });
        }
    }

    let fallback_reason = if primary.is_some() {
        SoftJudgeFallbackReason::InvalidPrimaryStructuredOutput
    } else {
        SoftJudgeFallbackReason::PrimaryJsonSchemaUnsupported
    };

    let fallback_request =
        build_soft_judge_json_fallback_request(request, max_tokens, random_seed)?;
    let fallback = adapter.generate(fallback_request).await?;
    let output = parse_and_validate_soft_output(request, &fallback.text).map_err(|error| {
        let first = primary.as_ref().map_or_else(
            || "primary schema mode unsupported".to_string(),
            |response| {
                format!(
                    "primary_bytes={} primary_shape={} primary_finish={}",
                    response.text.len(),
                    structured_output_shape(&response.text),
                    finish_reason_class(response.finish_reason.as_deref())
                )
            },
        );
        ModelBackedSoftJudgeError::InvalidStructuredOutput(format!(
            "{first}; fallback_bytes={} fallback_shape={} fallback_finish={}; {error}",
            fallback.text.len(),
            structured_output_shape(&fallback.text),
            finish_reason_class(fallback.finish_reason.as_deref())
        ))
    })?;
    let usage = primary.as_ref().map_or_else(
        || fallback.usage.clone(),
        |primary| add_model_usage(&primary.usage, &fallback.usage),
    );
    Ok(ModelBackedSoftJudgeObservation {
        observation: SoftJudgeObservation {
            judge: identity,
            request_id: request.id.clone(),
            decision: output.decision,
            finding: output.finding,
        },
        model: fallback.model,
        usage,
        provider_attempts: primary_provider_attempts.saturating_add(fallback.provider_attempts),
        fallback_reason,
    })
}

pub fn build_soft_judge_model_request(
    request: &SoftJudgeRequest,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<ModelRequest, ModelBackedSoftJudgeError> {
    let request_json = serde_json::to_string_pretty(request)
        .map_err(|error| ModelBackedSoftJudgeError::InvalidStructuredOutput(error.to_string()))?;
    let decision_guidance = soft_judge_decision_guidance(request.kind);
    Ok(ModelRequest {
        task: format!(
            "Evaluate this semantic diagnostic request:\n{request_json}\n\nDecision rule:\n{decision_guidance}\n\nUse only the supplied context. Return finding when the context affirmatively supports the requested diagnostic concern, no_finding when the context affirmatively resolves or negates that concern, and abstain only when the context is genuinely insufficient, mixed, or ambiguous. A finding must exactly preserve the requested kind and target. Do not invent evidence or verification authority."
        ),
        system: Some(
            "You are a soft semantic diagnostic judge inside a reasoning harness. Your output is advisory only. Return finding, no_finding, or abstain using the requested schema. You cannot create verification receipts, hard findings, epistemic-state promotion, verdicts, trusted evidence, or hidden-chain-of-thought grades."
                .into(),
        ),
        output_format: ModelOutputFormat::JsonSchema {
            name: "soft_judge_output".into(),
            schema: soft_judge_output_schema(),
        },
        max_tokens: Some(max_tokens),
        random_seed,
        reasoning_preference: Some(ModelReasoningPreference::Minimize),
    })
}

pub fn build_soft_judge_json_fallback_request(
    request: &SoftJudgeRequest,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<ModelRequest, ModelBackedSoftJudgeError> {
    let request_json = serde_json::to_string_pretty(request)
        .map_err(|error| ModelBackedSoftJudgeError::InvalidStructuredOutput(error.to_string()))?;
    let schema = serde_json::to_string_pretty(&soft_judge_output_schema())
        .map_err(|error| ModelBackedSoftJudgeError::InvalidStructuredOutput(error.to_string()))?;
    let decision_guidance = soft_judge_decision_guidance(request.kind);
    Ok(ModelRequest {
        task: format!(
            "JSON Schema:\n{schema}\n\nSemantic diagnostic request:\n{request_json}\n\nDecision rule:\n{decision_guidance}\n\nUse only the supplied context. Return finding when the context affirmatively supports the requested diagnostic concern, no_finding when the context affirmatively resolves or negates that concern, and abstain only when the context is genuinely insufficient, mixed, or ambiguous. Return exactly one JSON object conforming to the schema. A finding must exactly preserve the requested kind and target."
        ),
        system: Some(
            "You are a soft semantic diagnostic judge inside a reasoning harness. Return exactly one JSON object and no prose. Your output is advisory only and cannot create verification authority, hard findings, epistemic promotion, or verdicts."
                .into(),
        ),
        output_format: ModelOutputFormat::JsonObject,
        max_tokens: Some(max_tokens),
        random_seed,
        reasoning_preference: Some(ModelReasoningPreference::Minimize),
    })
}

pub(crate) fn soft_judge_decision_guidance(kind: SemanticDiagnosticKind) -> &'static str {
    match kind {
        SemanticDiagnosticKind::Contradiction => {
            "contradiction: finding means the supplied context contains a statement or observation that is semantically incompatible with the target under the same relevant binding and scope; lexical difference, synonymy, paraphrase, or equivalent wording alone is not a contradiction; no_finding means the supplied context semantically agrees with or supports the target, including a clear paraphrase or equivalent expression; abstain means binding, authority, scope, or applicability prevents deciding conflict versus agreement"
        }
        SemanticDiagnosticKind::Counterexample => {
            "counterexample: finding means the supplied context contains a concrete incompatible case that is applicable to the target generalization; no_finding means the supplied context affirmatively contains no applicable incompatible case for the requested check or an apparent contrary case is clearly outside the target scope; abstain means applicability of the contrary case to the target scope is uncertain"
        }
        SemanticDiagnosticKind::UnsupportedPremise => {
            "unsupported_premise: finding means the supplied context affirmatively indicates the target premise is introduced without support; no_finding means the supplied context directly or semantically supplies the premise, including a clear paraphrase with an unambiguous binding; abstain means support is partial, unbound, or uncertain in applicability or binding"
        }
        SemanticDiagnosticKind::CausalGap => {
            "causal_gap: finding means the supplied context affirmatively establishes that directional support for the requested causal relation is missing, for example correlation-only evidence, temporal or mechanism-only evidence without direction, explicit confounding, or an explicit viable reverse-causal alternative when direction remains undistinguished; no_finding means the supplied context explicitly supports the requested causal direction sufficiently for the requested relation and scope; abstain means some directional evidence exists but its adequacy is mixed, partial, scoped, or uncertain; imperfect causal evidence alone is not a finding"
        }
    }
}

fn finish_reason_class(finish_reason: Option<&str>) -> &'static str {
    match finish_reason {
        Some("stop") => "stop",
        Some("length") => "length",
        Some("tool_calls") => "tool_calls",
        Some("content_filter") => "content_filter",
        Some(_) => "other",
        None => "missing",
    }
}

fn structured_output_shape(text: &str) -> String {
    let trimmed = text.trim_start();
    if trimmed.is_empty() {
        return "empty".into();
    }
    if trimmed.starts_with('{') {
        return "json_object_start".into();
    }
    if trimmed.starts_with('[') {
        return "json_array_start".into();
    }
    if trimmed.starts_with("```") {
        return "markdown_fence".into();
    }
    if trimmed.starts_with('<') {
        return "markup_prefix".into();
    }
    if let Some(byte_offset) = trimmed.find('{') {
        let char_offset = trimmed[..byte_offset].chars().count();
        return format!("leading_text_then_json_object:{char_offset}");
    }
    "non_json_text".into()
}

pub fn parse_soft_judge_output(text: &str) -> Result<SoftJudgeOutput, serde_json::Error> {
    match serde_json::from_str::<SoftJudgeOutput>(text) {
        Ok(output) => Ok(output),
        Err(strict_error) => {
            let mut stream =
                serde_json::Deserializer::from_str(text).into_iter::<SoftJudgeOutput>();
            let Some(Ok(output)) = stream.next() else {
                return Err(strict_error);
            };
            let remainder = &text[stream.byte_offset()..];
            let mut trailing_values =
                serde_json::Deserializer::from_str(remainder).into_iter::<serde_json::Value>();
            match trailing_values.next() {
                Some(Ok(_)) => Err(strict_error),
                Some(Err(_)) => Ok(output),
                None => Err(strict_error),
            }
        }
    }
}

pub(crate) fn parse_and_validate_soft_output(
    request: &SoftJudgeRequest,
    text: &str,
) -> Result<SoftJudgeOutput, ModelBackedSoftJudgeError> {
    let output = parse_soft_judge_output(text)
        .map_err(|error| ModelBackedSoftJudgeError::InvalidStructuredOutput(error.to_string()))?;
    validate_output(request, &output)?;
    Ok(output)
}

fn validate_identity(identity: &SoftJudgeIdentity) -> Result<(), SoftJudgeError> {
    if identity.judge_id.trim().is_empty()
        || identity.model_id.trim().is_empty()
        || identity.configuration_id.trim().is_empty()
    {
        Err(SoftJudgeError::InvalidIdentity)
    } else {
        Ok(())
    }
}

fn add_model_usage(left: &ModelUsage, right: &ModelUsage) -> ModelUsage {
    ModelUsage {
        input_tokens: add_optional_usage(left.input_tokens, right.input_tokens),
        output_tokens: add_optional_usage(left.output_tokens, right.output_tokens),
        total_tokens: add_optional_usage(left.total_tokens, right.total_tokens),
    }
}

fn add_optional_usage(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => left.checked_add(right),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationLabel {
    Positive,
    Negative,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftJudgeCalibrationFixture {
    pub id: String,
    pub request: SoftJudgeRequest,
    pub label: CalibrationLabel,
    #[serde(default)]
    pub recorded_observations: Vec<SoftJudgeObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoftJudgeMetrics {
    pub judge: SoftJudgeIdentity,
    pub cases: usize,
    pub labelled_cases: usize,
    pub ambiguous_cases: usize,
    pub ambiguous_abstentions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ambiguous_abstention_rate: Option<f64>,
    pub finding_decisions: usize,
    pub no_finding_decisions: usize,
    pub abstentions: usize,
    pub decision_coverage: f64,
    pub true_positives: usize,
    pub false_positives: usize,
    pub true_negatives: usize,
    pub false_negatives: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recall: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoftJudgeAgreement {
    pub cases: usize,
    pub cases_with_multiple_non_abstain_votes: usize,
    pub comparable_pairs: usize,
    pub agreeing_pairs: usize,
    pub disagreeing_pairs: usize,
    pub abstain_votes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_pairwise_agreement: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub krippendorff_alpha_nominal: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoftJudgeCalibrationReport {
    pub cases: usize,
    pub judges: Vec<SoftJudgeMetrics>,
    pub agreement: SoftJudgeAgreement,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SoftJudgeCalibrationError {
    #[error("calibration fixture id must not be empty")]
    EmptyFixtureId,
    #[error("duplicate calibration fixture id: {0}")]
    DuplicateFixtureId(String),
    #[error("soft judge request id must not be empty")]
    EmptyRequestId,
    #[error("duplicate soft judge request id: {0}")]
    DuplicateRequestId(String),
    #[error("soft judge identity fields must not be empty in fixture {fixture_id}")]
    EmptyJudgeIdentity { fixture_id: String },
    #[error("duplicate judge {judge_id} in calibration fixture {fixture_id}")]
    DuplicateJudgeObservation {
        fixture_id: String,
        judge_id: String,
    },
    #[error("observation request id {observed} does not match fixture request {expected}")]
    ObservationRequestMismatch { observed: String, expected: String },
    #[error("finding decision must include a typed soft finding in fixture {fixture_id}")]
    MissingFinding { fixture_id: String },
    #[error("non-finding decision must not include a finding in fixture {fixture_id}")]
    UnexpectedFinding { fixture_id: String },
    #[error("soft finding does not match the requested kind/target in fixture {fixture_id}")]
    FindingTargetMismatch { fixture_id: String },
    #[error("judge {judge_id} is missing an observation for fixture {fixture_id}")]
    MissingJudgeObservation {
        fixture_id: String,
        judge_id: String,
    },
    #[error("calibration corpus contains no judge observations")]
    NoJudgeObservations,
    #[error("judge identity changed across fixtures for judge id {judge_id}")]
    JudgeIdentityMismatch { judge_id: String },
}

pub fn aggregate_soft_judge_calibration(
    fixtures: &[SoftJudgeCalibrationFixture],
) -> Result<SoftJudgeCalibrationReport, SoftJudgeCalibrationError> {
    validate_calibration_fixtures(fixtures)?;

    let identities = collect_identities(fixtures)?;
    let mut judges = Vec::with_capacity(identities.len());
    for identity in identities.values() {
        judges.push(metrics_for_judge(fixtures, identity)?);
    }

    Ok(SoftJudgeCalibrationReport {
        cases: fixtures.len(),
        judges,
        agreement: agreement(fixtures),
    })
}

pub fn validate_calibration_fixtures(
    fixtures: &[SoftJudgeCalibrationFixture],
) -> Result<(), SoftJudgeCalibrationError> {
    let mut fixture_ids = std::collections::BTreeSet::new();
    let mut request_ids = std::collections::BTreeSet::new();

    for fixture in fixtures {
        if fixture.id.trim().is_empty() {
            return Err(SoftJudgeCalibrationError::EmptyFixtureId);
        }
        if !fixture_ids.insert(fixture.id.as_str()) {
            return Err(SoftJudgeCalibrationError::DuplicateFixtureId(
                fixture.id.clone(),
            ));
        }
        if fixture.request.id.trim().is_empty() {
            return Err(SoftJudgeCalibrationError::EmptyRequestId);
        }
        if !request_ids.insert(fixture.request.id.as_str()) {
            return Err(SoftJudgeCalibrationError::DuplicateRequestId(
                fixture.request.id.clone(),
            ));
        }

        let mut judges = std::collections::BTreeSet::new();
        for observation in &fixture.recorded_observations {
            if observation.judge.judge_id.trim().is_empty()
                || observation.judge.model_id.trim().is_empty()
                || observation.judge.configuration_id.trim().is_empty()
            {
                return Err(SoftJudgeCalibrationError::EmptyJudgeIdentity {
                    fixture_id: fixture.id.clone(),
                });
            }
            if !judges.insert(observation.judge.judge_id.as_str()) {
                return Err(SoftJudgeCalibrationError::DuplicateJudgeObservation {
                    fixture_id: fixture.id.clone(),
                    judge_id: observation.judge.judge_id.clone(),
                });
            }
            if observation.request_id != fixture.request.id {
                return Err(SoftJudgeCalibrationError::ObservationRequestMismatch {
                    observed: observation.request_id.clone(),
                    expected: fixture.request.id.clone(),
                });
            }
            match (observation.decision, &observation.finding) {
                (SoftJudgeDecision::Finding, None) => {
                    return Err(SoftJudgeCalibrationError::MissingFinding {
                        fixture_id: fixture.id.clone(),
                    });
                }
                (SoftJudgeDecision::NoFinding | SoftJudgeDecision::Abstain, Some(_)) => {
                    return Err(SoftJudgeCalibrationError::UnexpectedFinding {
                        fixture_id: fixture.id.clone(),
                    });
                }
                (SoftJudgeDecision::Finding, Some(finding))
                    if finding.kind != fixture.request.kind
                        || finding.target != fixture.request.target =>
                {
                    return Err(SoftJudgeCalibrationError::FindingTargetMismatch {
                        fixture_id: fixture.id.clone(),
                    });
                }
                _ => {}
            }
        }
    }

    let identities = collect_identities(fixtures)?;
    if identities.is_empty() {
        return Err(SoftJudgeCalibrationError::NoJudgeObservations);
    }
    for fixture in fixtures {
        for judge_id in identities.keys() {
            if !fixture
                .recorded_observations
                .iter()
                .any(|observation| &observation.judge.judge_id == judge_id)
            {
                return Err(SoftJudgeCalibrationError::MissingJudgeObservation {
                    fixture_id: fixture.id.clone(),
                    judge_id: judge_id.clone(),
                });
            }
        }
    }
    Ok(())
}

fn collect_identities(
    fixtures: &[SoftJudgeCalibrationFixture],
) -> Result<BTreeMap<String, SoftJudgeIdentity>, SoftJudgeCalibrationError> {
    let mut identities = BTreeMap::new();
    for fixture in fixtures {
        for observation in &fixture.recorded_observations {
            match identities.get(&observation.judge.judge_id) {
                Some(existing) if existing != &observation.judge => {
                    return Err(SoftJudgeCalibrationError::JudgeIdentityMismatch {
                        judge_id: observation.judge.judge_id.clone(),
                    });
                }
                Some(_) => {}
                None => {
                    identities.insert(
                        observation.judge.judge_id.clone(),
                        observation.judge.clone(),
                    );
                }
            }
        }
    }
    Ok(identities)
}

fn metrics_for_judge(
    fixtures: &[SoftJudgeCalibrationFixture],
    identity: &SoftJudgeIdentity,
) -> Result<SoftJudgeMetrics, SoftJudgeCalibrationError> {
    let mut metrics = SoftJudgeMetrics {
        judge: identity.clone(),
        cases: fixtures.len(),
        labelled_cases: 0,
        ambiguous_cases: 0,
        ambiguous_abstentions: 0,
        ambiguous_abstention_rate: None,
        finding_decisions: 0,
        no_finding_decisions: 0,
        abstentions: 0,
        decision_coverage: 0.0,
        true_positives: 0,
        false_positives: 0,
        true_negatives: 0,
        false_negatives: 0,
        precision: None,
        recall: None,
    };

    for fixture in fixtures {
        let observation = fixture
            .recorded_observations
            .iter()
            .find(|observation| observation.judge.judge_id == identity.judge_id)
            .ok_or_else(|| SoftJudgeCalibrationError::MissingJudgeObservation {
                fixture_id: fixture.id.clone(),
                judge_id: identity.judge_id.clone(),
            })?;

        match observation.decision {
            SoftJudgeDecision::Finding => metrics.finding_decisions += 1,
            SoftJudgeDecision::NoFinding => metrics.no_finding_decisions += 1,
            SoftJudgeDecision::Abstain => metrics.abstentions += 1,
        }

        match fixture.label {
            CalibrationLabel::Ambiguous => {
                metrics.ambiguous_cases += 1;
                if observation.decision == SoftJudgeDecision::Abstain {
                    metrics.ambiguous_abstentions += 1;
                }
            }
            CalibrationLabel::Positive => {
                metrics.labelled_cases += 1;
                if observation.decision == SoftJudgeDecision::Finding {
                    metrics.true_positives += 1;
                } else {
                    metrics.false_negatives += 1;
                }
            }
            CalibrationLabel::Negative => {
                metrics.labelled_cases += 1;
                match observation.decision {
                    SoftJudgeDecision::Finding => metrics.false_positives += 1,
                    SoftJudgeDecision::NoFinding => metrics.true_negatives += 1,
                    SoftJudgeDecision::Abstain => {}
                }
            }
        }
    }

    let decided = metrics.finding_decisions + metrics.no_finding_decisions;
    metrics.decision_coverage = rate(decided, metrics.cases);
    metrics.ambiguous_abstention_rate =
        optional_rate(metrics.ambiguous_abstentions, metrics.ambiguous_cases);
    metrics.precision = optional_rate(
        metrics.true_positives,
        metrics.true_positives + metrics.false_positives,
    );
    metrics.recall = optional_rate(
        metrics.true_positives,
        metrics.true_positives + metrics.false_negatives,
    );
    Ok(metrics)
}

fn agreement(fixtures: &[SoftJudgeCalibrationFixture]) -> SoftJudgeAgreement {
    let mut comparable_pairs = 0usize;
    let mut agreeing_pairs = 0usize;
    let mut disagreeing_pairs = 0usize;
    let mut abstain_votes = 0usize;
    let mut cases_with_multiple_non_abstain_votes = 0usize;

    for fixture in fixtures {
        abstain_votes += fixture
            .recorded_observations
            .iter()
            .filter(|observation| observation.decision == SoftJudgeDecision::Abstain)
            .count();
        let decisions = fixture
            .recorded_observations
            .iter()
            .filter(|observation| observation.decision != SoftJudgeDecision::Abstain)
            .map(|observation| observation.decision)
            .collect::<Vec<_>>();
        if decisions.len() >= 2 {
            cases_with_multiple_non_abstain_votes += 1;
        }
        for left in 0..decisions.len() {
            for right in (left + 1)..decisions.len() {
                comparable_pairs += 1;
                if decisions[left] == decisions[right] {
                    agreeing_pairs += 1;
                } else {
                    disagreeing_pairs += 1;
                }
            }
        }
    }

    SoftJudgeAgreement {
        cases: fixtures.len(),
        cases_with_multiple_non_abstain_votes,
        comparable_pairs,
        agreeing_pairs,
        disagreeing_pairs,
        abstain_votes,
        observed_pairwise_agreement: optional_rate(agreeing_pairs, comparable_pairs),
        krippendorff_alpha_nominal: krippendorff_alpha_nominal(fixtures),
    }
}

fn krippendorff_alpha_nominal(fixtures: &[SoftJudgeCalibrationFixture]) -> Option<f64> {
    // Abstentions are missing ratings, not a third semantic verdict. Coincidences are
    // normalized per unit as in Krippendorff's nominal alpha so units with more judges
    // do not receive quadratic weight.
    let mut coincidence = [[0.0_f64; 2]; 2];
    for fixture in fixtures {
        let ratings = fixture
            .recorded_observations
            .iter()
            .filter_map(|observation| match observation.decision {
                SoftJudgeDecision::Finding => Some(0usize),
                SoftJudgeDecision::NoFinding => Some(1usize),
                SoftJudgeDecision::Abstain => None,
            })
            .collect::<Vec<_>>();
        if ratings.len() < 2 {
            continue;
        }
        let weight = 1.0 / (ratings.len() - 1) as f64;
        for (left_index, left) in ratings.iter().enumerate() {
            for (right_index, right) in ratings.iter().enumerate() {
                if left_index != right_index {
                    coincidence[*left][*right] += weight;
                }
            }
        }
    }

    let marginals = [
        coincidence[0][0] + coincidence[0][1],
        coincidence[1][0] + coincidence[1][1],
    ];
    let total = marginals[0] + marginals[1];
    if total <= 1.0 {
        return None;
    }
    let observed_disagreement = (coincidence[0][1] + coincidence[1][0]) / total;
    let expected_disagreement = (2.0 * marginals[0] * marginals[1]) / (total * (total - 1.0));
    if expected_disagreement == 0.0 {
        return None;
    }
    Some(1.0 - observed_disagreement / expected_disagreement)
}

fn rate(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn optional_rate(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| rate(numerator, denominator))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(id: &str) -> SoftJudgeIdentity {
        SoftJudgeIdentity {
            judge_id: id.into(),
            model_id: format!("{id}-model"),
            configuration_id: "v1".into(),
        }
    }

    fn proposition_target() -> SemanticDiagnosticTarget {
        SemanticDiagnosticTarget::Proposition {
            proposition: Proposition {
                key: "feature.enabled".into(),
                value: "true".into(),
            },
        }
    }

    fn observation(
        judge: SoftJudgeIdentity,
        request_id: &str,
        decision: SoftJudgeDecision,
    ) -> SoftJudgeObservation {
        SoftJudgeObservation {
            judge,
            request_id: request_id.into(),
            decision,
            finding: (decision == SoftJudgeDecision::Finding).then(|| SoftSemanticFinding {
                kind: SemanticDiagnosticKind::Contradiction,
                target: proposition_target(),
                note: None,
            }),
        }
    }

    #[test]
    fn abstention_is_preserved_and_positive_abstention_reduces_recall() {
        let fixtures = vec![SoftJudgeCalibrationFixture {
            id: "positive".into(),
            request: SoftJudgeRequest {
                id: "request-positive".into(),
                task: "check contradiction".into(),
                kind: SemanticDiagnosticKind::Contradiction,
                target: proposition_target(),
                context: vec![],
            },
            label: CalibrationLabel::Positive,
            recorded_observations: vec![observation(
                identity("judge-a"),
                "request-positive",
                SoftJudgeDecision::Abstain,
            )],
        }];
        let report = aggregate_soft_judge_calibration(&fixtures).unwrap();
        assert_eq!(report.judges[0].abstentions, 1);
        assert_eq!(report.judges[0].false_negatives, 1);
        assert_eq!(report.judges[0].recall, Some(0.0));
        assert_eq!(report.judges[0].decision_coverage, 0.0);
    }

    #[test]
    fn ambiguous_labels_do_not_enter_precision_or_recall() {
        let fixtures = vec![SoftJudgeCalibrationFixture {
            id: "ambiguous".into(),
            request: SoftJudgeRequest {
                id: "request-ambiguous".into(),
                task: "check contradiction".into(),
                kind: SemanticDiagnosticKind::Contradiction,
                target: proposition_target(),
                context: vec![],
            },
            label: CalibrationLabel::Ambiguous,
            recorded_observations: vec![observation(
                identity("judge-a"),
                "request-ambiguous",
                SoftJudgeDecision::Finding,
            )],
        }];
        let report = aggregate_soft_judge_calibration(&fixtures).unwrap();
        let metrics = &report.judges[0];
        assert_eq!(metrics.ambiguous_cases, 1);
        assert_eq!(metrics.ambiguous_abstentions, 0);
        assert_eq!(metrics.ambiguous_abstention_rate, Some(0.0));
        assert_eq!(metrics.labelled_cases, 0);
        assert_eq!(metrics.precision, None);
        assert_eq!(metrics.recall, None);
    }

    #[test]
    fn ambiguous_abstention_rate_is_absent_when_corpus_has_no_ambiguous_cases() {
        let fixtures = vec![SoftJudgeCalibrationFixture {
            id: "negative".into(),
            request: SoftJudgeRequest {
                id: "request-negative".into(),
                task: "check contradiction".into(),
                kind: SemanticDiagnosticKind::Contradiction,
                target: proposition_target(),
                context: vec![],
            },
            label: CalibrationLabel::Negative,
            recorded_observations: vec![observation(
                identity("judge-a"),
                "request-negative",
                SoftJudgeDecision::NoFinding,
            )],
        }];
        let report = aggregate_soft_judge_calibration(&fixtures).unwrap();
        let metrics = &report.judges[0];
        assert_eq!(metrics.ambiguous_cases, 0);
        assert_eq!(metrics.ambiguous_abstentions, 0);
        assert_eq!(metrics.ambiguous_abstention_rate, None);
    }

    #[test]
    fn agreement_excludes_abstentions_without_erasing_them() {
        let fixture = SoftJudgeCalibrationFixture {
            id: "case".into(),
            request: SoftJudgeRequest {
                id: "request".into(),
                task: "check contradiction".into(),
                kind: SemanticDiagnosticKind::Contradiction,
                target: proposition_target(),
                context: vec![],
            },
            label: CalibrationLabel::Positive,
            recorded_observations: vec![
                observation(identity("judge-a"), "request", SoftJudgeDecision::Finding),
                observation(identity("judge-b"), "request", SoftJudgeDecision::Finding),
                observation(identity("judge-c"), "request", SoftJudgeDecision::Abstain),
            ],
        };
        let report = aggregate_soft_judge_calibration(&[fixture]).unwrap();
        assert_eq!(report.agreement.abstain_votes, 1);
        assert_eq!(report.agreement.comparable_pairs, 1);
        assert_eq!(report.agreement.agreeing_pairs, 1);
        assert_eq!(report.agreement.observed_pairwise_agreement, Some(1.0));
        assert_eq!(report.agreement.krippendorff_alpha_nominal, None);
    }

    #[test]
    fn nominal_alpha_is_one_for_perfect_agreement_across_both_categories() {
        let make_fixture = |id: &str, decision: SoftJudgeDecision, label: CalibrationLabel| {
            SoftJudgeCalibrationFixture {
                id: id.into(),
                request: SoftJudgeRequest {
                    id: format!("request-{id}"),
                    task: "check contradiction".into(),
                    kind: SemanticDiagnosticKind::Contradiction,
                    target: proposition_target(),
                    context: vec![],
                },
                label,
                recorded_observations: vec![
                    observation(identity("judge-a"), &format!("request-{id}"), decision),
                    observation(identity("judge-b"), &format!("request-{id}"), decision),
                ],
            }
        };
        let report = aggregate_soft_judge_calibration(&[
            make_fixture(
                "positive",
                SoftJudgeDecision::Finding,
                CalibrationLabel::Positive,
            ),
            make_fixture(
                "negative",
                SoftJudgeDecision::NoFinding,
                CalibrationLabel::Negative,
            ),
        ])
        .unwrap();
        assert_eq!(report.agreement.krippendorff_alpha_nominal, Some(1.0));
    }

    #[test]
    fn finish_reason_diagnostic_uses_bounded_classes() {
        assert_eq!(finish_reason_class(Some("stop")), "stop");
        assert_eq!(finish_reason_class(Some("length")), "length");
        assert_eq!(
            finish_reason_class(Some("provider-specific-value")),
            "other"
        );
        assert_eq!(finish_reason_class(None), "missing");
    }

    #[test]
    fn output_shape_diagnostic_is_content_free_and_distinguishes_common_wrappers() {
        assert_eq!(
            structured_output_shape("  {\"decision\":\"abstain\"}"),
            "json_object_start"
        );
        assert_eq!(
            structured_output_shape("```json\n{}\n```"),
            "markdown_fence"
        );
        assert_eq!(
            structured_output_shape("<think>hidden</think>\n{}"),
            "markup_prefix"
        );
        assert_eq!(
            structured_output_shape("analysis first\n{}"),
            "leading_text_then_json_object:15"
        );
        assert_eq!(structured_output_shape("plain prose"), "non_json_text");
        assert_eq!(structured_output_shape("   "), "empty");
    }

    #[test]
    fn finding_contract_has_no_hard_authority_field() {
        let finding = SoftSemanticFinding {
            kind: SemanticDiagnosticKind::Contradiction,
            target: proposition_target(),
            note: None,
        };
        let value = serde_json::to_value(finding).unwrap();
        assert!(value.get("strength").is_none());
        assert!(value.get("verification_receipt").is_none());
        assert!(value.get("verdict").is_none());
    }
}
