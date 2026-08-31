use std::collections::BTreeMap;

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use thiserror::Error;

use crate::{
    ModelAdapter, ModelBackedSoftJudgeError, ModelError, ModelErrorKind, ModelOutputFormat,
    ModelRequest, ModelUsage, SoftJudgeDecision, SoftJudgeOutput, SoftJudgeRequest,
    SoftSemanticFinding, build_soft_judge_model_request,
};

/// Information-equivalent model-facing representations used only for R1 format-invariance
/// research.
///
/// Every R1a variant preserves the complete v3 output information so the model can satisfy the
/// unchanged v3 instruction that a finding preserve the requested kind and target. Reducing the
/// model-owned payload to a decision only is deliberately deferred to R2 materialization research.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoftJudgeRepresentation {
    V3FullJson,
    NestedResultObject,
    DecisionFindingTuple,
    CompactKeyObject,
}

impl SoftJudgeRepresentation {
    pub const ALL: [Self; 4] = [
        Self::V3FullJson,
        Self::NestedResultObject,
        Self::DecisionFindingTuple,
        Self::CompactKeyObject,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::V3FullJson => "v3_full_json",
            Self::NestedResultObject => "nested_result_object",
            Self::DecisionFindingTuple => "decision_finding_tuple",
            Self::CompactKeyObject => "compact_key_object",
        }
    }

    pub fn requested_output_format(self) -> &'static str {
        "json_schema"
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormatJudgeObservation {
    pub representation: SoftJudgeRepresentation,
    pub decision: SoftJudgeDecision,
    pub model: String,
    pub usage: ModelUsage,
}

#[derive(Debug, Error)]
pub enum FormatJudgeError {
    #[error("format-judge request setup failed: {0}")]
    Setup(String),
    #[error("format-judge adapter failed: {0}")]
    Model(#[from] ModelError),
    #[error("format-judge returned invalid representation: {message}")]
    InvalidRepresentation {
        message: String,
        model: String,
        usage: ModelUsage,
    },
}

impl FormatJudgeError {
    pub fn model_error_kind(&self) -> Option<ModelErrorKind> {
        match self {
            Self::Model(error) => Some(error.kind),
            Self::Setup(_) | Self::InvalidRepresentation { .. } => None,
        }
    }

    pub fn usage(&self) -> Option<&ModelUsage> {
        match self {
            Self::Setup(_) | Self::Model(_) => None,
            Self::InvalidRepresentation { usage, .. } => Some(usage),
        }
    }

    pub fn provider_model(&self) -> Option<&str> {
        match self {
            Self::Setup(_) | Self::Model(_) => None,
            Self::InvalidRepresentation { model, .. } => Some(model),
        }
    }
}

pub async fn run_model_backed_soft_judge_representation(
    adapter: &dyn ModelAdapter,
    request: &SoftJudgeRequest,
    representation: SoftJudgeRepresentation,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<FormatJudgeObservation, FormatJudgeError> {
    let model_request =
        build_soft_judge_representation_request(request, representation, max_tokens, random_seed)
            .map_err(|error| FormatJudgeError::Setup(error.to_string()))?;
    let response = adapter.generate(model_request).await?;
    let decision =
        parse_soft_judge_representation_decision(request, representation, &response.text).map_err(
            |error| FormatJudgeError::InvalidRepresentation {
                message: error.to_string(),
                model: response.model.clone(),
                usage: response.usage.clone(),
            },
        )?;
    Ok(FormatJudgeObservation {
        representation,
        decision,
        model: response.model,
        usage: response.usage,
    })
}

/// Builds an R1 request by taking the exact v3 primary request and replacing only its schema.
/// This intentionally has no fallback path: R1a measures representation under one requested
/// output-format class. Provider-side enforcement fidelity is a separate study coordinate.
pub fn build_soft_judge_representation_request(
    request: &SoftJudgeRequest,
    representation: SoftJudgeRepresentation,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<ModelRequest, ModelBackedSoftJudgeError> {
    let mut model_request = build_soft_judge_model_request(request, max_tokens, random_seed)?;
    if representation == SoftJudgeRepresentation::V3FullJson {
        return Ok(model_request);
    }
    model_request.output_format = ModelOutputFormat::JsonSchema {
        name: format!("soft_judge_{}", representation.id()),
        schema: representation_schema(representation),
    };
    Ok(model_request)
}

pub fn parse_soft_judge_representation_decision(
    request: &SoftJudgeRequest,
    representation: SoftJudgeRepresentation,
    text: &str,
) -> Result<SoftJudgeDecision, ModelBackedSoftJudgeError> {
    let output = match representation {
        SoftJudgeRepresentation::V3FullJson => {
            crate::semantic_judge::parse_and_validate_soft_output(request, text)?
        }
        SoftJudgeRepresentation::NestedResultObject => {
            let output: NestedResultObject = parse_one_json_value(text)?;
            output.result
        }
        SoftJudgeRepresentation::DecisionFindingTuple => {
            let output: DecisionFindingTuple = parse_one_json_value(text)?;
            SoftJudgeOutput {
                decision: output.0,
                finding: output.1,
            }
        }
        SoftJudgeRepresentation::CompactKeyObject => {
            let output: CompactKeyObject = parse_one_json_value(text)?;
            SoftJudgeOutput {
                decision: output.d,
                finding: output.f,
            }
        }
    };
    crate::semantic_judge::validate_output(request, &output)?;
    Ok(output.decision)
}

fn representation_schema(representation: SoftJudgeRepresentation) -> Value {
    match representation {
        SoftJudgeRepresentation::V3FullJson => {
            unreachable!("v3 representation uses the runtime schema")
        }
        SoftJudgeRepresentation::NestedResultObject => {
            serialize_schema(schema_for!(NestedResultObject))
        }
        SoftJudgeRepresentation::DecisionFindingTuple => {
            serialize_schema(schema_for!(DecisionFindingTuple))
        }
        SoftJudgeRepresentation::CompactKeyObject => {
            serialize_schema(schema_for!(CompactKeyObject))
        }
    }
}

fn serialize_schema(schema: schemars::Schema) -> Value {
    serde_json::to_value(schema).expect("R1 representation schema must be serializable")
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NestedResultObject {
    result: SoftJudgeOutput,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct DecisionFindingTuple(SoftJudgeDecision, Option<SoftSemanticFinding>);

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CompactKeyObject {
    d: SoftJudgeDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    f: Option<SoftSemanticFinding>,
}

fn parse_one_json_value<T: DeserializeOwned>(text: &str) -> Result<T, ModelBackedSoftJudgeError> {
    match serde_json::from_str::<T>(text) {
        Ok(value) => Ok(value),
        Err(strict_error) => {
            let mut stream = serde_json::Deserializer::from_str(text).into_iter::<T>();
            let Some(Ok(value)) = stream.next() else {
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
                Some(Err(_)) => Ok(value),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchedFormatDecision {
    pub fixture_id: String,
    pub trial: usize,
    pub seed: Option<u64>,
    /// `None` means the provider/protocol path did not produce a valid semantic decision.
    pub decision: Option<SoftJudgeDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatDecisionTransition {
    pub from: SoftJudgeDecision,
    pub to: SoftJudgeDecision,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FormatFlipReport {
    pub baseline: SoftJudgeRepresentation,
    pub variant: SoftJudgeRepresentation,
    pub matched_keys: usize,
    pub matched_successful_pairs: usize,
    pub operationally_incomplete_pairs: usize,
    pub changed_decisions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format_flip_rate: Option<f64>,
    pub transitions: Vec<FormatDecisionTransition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FormatComparisonError {
    #[error("duplicate matched format key for fixture={fixture_id} trial={trial} seed={seed:?}")]
    DuplicateKey {
        fixture_id: String,
        trial: usize,
        seed: Option<u64>,
    },
    #[error("baseline and variant matched-key sets differ")]
    KeySetMismatch,
}

pub fn compare_soft_judge_formats(
    baseline_representation: SoftJudgeRepresentation,
    baseline: &[MatchedFormatDecision],
    variant_representation: SoftJudgeRepresentation,
    variant: &[MatchedFormatDecision],
) -> Result<FormatFlipReport, FormatComparisonError> {
    type Key = (String, usize, Option<u64>);

    fn index(
        cases: &[MatchedFormatDecision],
    ) -> Result<BTreeMap<Key, Option<SoftJudgeDecision>>, FormatComparisonError> {
        let mut indexed = BTreeMap::new();
        for case in cases {
            let key = (case.fixture_id.clone(), case.trial, case.seed);
            if indexed.insert(key.clone(), case.decision).is_some() {
                return Err(FormatComparisonError::DuplicateKey {
                    fixture_id: key.0,
                    trial: key.1,
                    seed: key.2,
                });
            }
        }
        Ok(indexed)
    }

    let baseline = index(baseline)?;
    let variant = index(variant)?;
    if baseline.keys().ne(variant.keys()) {
        return Err(FormatComparisonError::KeySetMismatch);
    }

    let mut matched_successful_pairs = 0usize;
    let mut operationally_incomplete_pairs = 0usize;
    let mut changed_decisions = 0usize;
    let mut transitions = BTreeMap::<(SoftJudgeDecision, SoftJudgeDecision), usize>::new();

    for (key, baseline_decision) in &baseline {
        let variant_decision = variant
            .get(key)
            .expect("equal key sets guarantee a variant decision");
        match (baseline_decision, variant_decision) {
            (Some(from), Some(to)) => {
                matched_successful_pairs += 1;
                if from != to {
                    changed_decisions += 1;
                }
                *transitions.entry((*from, *to)).or_insert(0) += 1;
            }
            _ => operationally_incomplete_pairs += 1,
        }
    }

    Ok(FormatFlipReport {
        baseline: baseline_representation,
        variant: variant_representation,
        matched_keys: baseline.len(),
        matched_successful_pairs,
        operationally_incomplete_pairs,
        changed_decisions,
        format_flip_rate: (matched_successful_pairs > 0)
            .then(|| changed_decisions as f64 / matched_successful_pairs as f64),
        transitions: transitions
            .into_iter()
            .map(|((from, to), count)| FormatDecisionTransition { from, to, count })
            .collect(),
    })
}
