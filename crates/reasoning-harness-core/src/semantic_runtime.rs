use serde::{Deserialize, Deserializer, Serialize, de};
use thiserror::Error;

use crate::{
    MaterializationError, MaterializedDecisionOutput, ModelAdapter, ModelBackedSoftJudgeError,
    ModelUsage, ReasoningArtifact, SemanticDecidabilityAssessment, SemanticDecidabilityError,
    SoftJudgeFallbackReason, SoftJudgeIdentity, SoftJudgeObservation, SoftJudgeRequest,
    assess_semantic_decidability, compose_semantic_decidability, materialize_soft_judge_output,
    run_model_backed_soft_judge, run_model_backed_soft_judge_materialization,
};

pub const SOFT_SEMANTIC_V3_CONFIGURATION_ID: &str = "soft-semantic-v3";
pub const MATERIALIZATION_R2_CONTRACT_ID: &str = "materialization-r2-v1";
pub const D3_DECIDABILITY_CONTRACT_ID: &str = "deterministic-explicit-typed-preconditions-v1";
pub const SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID: &str = "semantic-decidability-d3-v1";
pub const SEMANTIC_RUNTIME_IDENTITY_VERSION: &str = "semantic-runtime-identity-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticRuntimeProfile {
    SoftSemanticV3,
    SemanticDecidabilityD3V1,
}

impl SemanticRuntimeProfile {
    pub const fn configuration_id(self) -> &'static str {
        match self {
            Self::SoftSemanticV3 => SOFT_SEMANTIC_V3_CONFIGURATION_ID,
            Self::SemanticDecidabilityD3V1 => SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID,
        }
    }

    pub const fn rollback_profile(self) -> Option<Self> {
        match self {
            Self::SoftSemanticV3 => None,
            Self::SemanticDecidabilityD3V1 => Some(Self::SoftSemanticV3),
        }
    }

    pub fn identity(self) -> SemanticRuntimeIdentity {
        match self {
            Self::SoftSemanticV3 => SemanticRuntimeIdentity {
                identity_version: SEMANTIC_RUNTIME_IDENTITY_VERSION.into(),
                profile: self,
                configuration_id: SOFT_SEMANTIC_V3_CONFIGURATION_ID.into(),
                semantic_baseline: SOFT_SEMANTIC_V3_CONFIGURATION_ID.into(),
                materialization_contract: None,
                decidability_contract: None,
                rollback_configuration_id: None,
            },
            Self::SemanticDecidabilityD3V1 => SemanticRuntimeIdentity {
                identity_version: SEMANTIC_RUNTIME_IDENTITY_VERSION.into(),
                profile: self,
                configuration_id: SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID.into(),
                semantic_baseline: SOFT_SEMANTIC_V3_CONFIGURATION_ID.into(),
                materialization_contract: Some(MATERIALIZATION_R2_CONTRACT_ID.into()),
                decidability_contract: Some(D3_DECIDABILITY_CONTRACT_ID.into()),
                rollback_configuration_id: Some(SOFT_SEMANTIC_V3_CONFIGURATION_ID.into()),
            },
        }
    }
}

/// Stabilization keeps the previously characterized v3 runtime as the default. D3 adoption is a
/// separate, explicit change to this constant after the operational contract is merged.
pub const DEFAULT_SEMANTIC_RUNTIME_PROFILE: SemanticRuntimeProfile =
    SemanticRuntimeProfile::SoftSemanticV3;

pub const fn default_semantic_runtime_profile() -> SemanticRuntimeProfile {
    DEFAULT_SEMANTIC_RUNTIME_PROFILE
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SemanticRuntimeIdentity {
    identity_version: String,
    profile: SemanticRuntimeProfile,
    configuration_id: String,
    semantic_baseline: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    materialization_contract: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    decidability_contract: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rollback_configuration_id: Option<String>,
}

impl SemanticRuntimeIdentity {
    pub fn identity_version(&self) -> &str {
        &self.identity_version
    }

    pub const fn profile(&self) -> SemanticRuntimeProfile {
        self.profile
    }

    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }

    pub fn semantic_baseline(&self) -> &str {
        &self.semantic_baseline
    }

    pub fn materialization_contract(&self) -> Option<&str> {
        self.materialization_contract.as_deref()
    }

    pub fn decidability_contract(&self) -> Option<&str> {
        self.decidability_contract.as_deref()
    }

    pub fn rollback_configuration_id(&self) -> Option<&str> {
        self.rollback_configuration_id.as_deref()
    }
}

#[derive(Deserialize)]
struct SemanticRuntimeIdentityWire {
    identity_version: String,
    profile: SemanticRuntimeProfile,
    configuration_id: String,
    semantic_baseline: String,
    #[serde(default)]
    materialization_contract: Option<String>,
    #[serde(default)]
    decidability_contract: Option<String>,
    #[serde(default)]
    rollback_configuration_id: Option<String>,
}

impl<'de> Deserialize<'de> for SemanticRuntimeIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = SemanticRuntimeIdentityWire::deserialize(deserializer)?;
        let expected = wire.profile.identity();
        let actual = SemanticRuntimeIdentity {
            identity_version: wire.identity_version,
            profile: wire.profile,
            configuration_id: wire.configuration_id,
            semantic_baseline: wire.semantic_baseline,
            materialization_contract: wire.materialization_contract,
            decidability_contract: wire.decidability_contract,
            rollback_configuration_id: wire.rollback_configuration_id,
        };
        if actual != expected {
            return Err(de::Error::custom(
                "semantic runtime identity does not match its canonical profile",
            ));
        }
        Ok(actual)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticRuntimeObservation {
    pub runtime: SemanticRuntimeIdentity,
    pub observation: SoftJudgeObservation,
    pub base_decision: crate::SoftJudgeDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decidability: Option<SemanticDecidabilityAssessment>,
    pub model: String,
    pub usage: ModelUsage,
    pub provider_attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<SoftJudgeFallbackReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Error)]
pub enum SemanticRuntimeError {
    #[error("semantic runtime requested model must not be empty")]
    InvalidRequestedModel,
    #[error("soft-semantic-v3 runtime failed: {0}")]
    Baseline(#[source] ModelBackedSoftJudgeError),
    #[error("semantic decidability precondition failed: {0}")]
    Decidability(#[from] SemanticDecidabilityError),
    #[error("D3 materialization failed: {0}")]
    Materialization(#[from] MaterializationError),
}

pub async fn run_default_semantic_runtime(
    adapter: &dyn ModelAdapter,
    requested_model: &str,
    request: &SoftJudgeRequest,
    artifact: &ReasoningArtifact,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<SemanticRuntimeObservation, SemanticRuntimeError> {
    run_semantic_runtime(
        default_semantic_runtime_profile(),
        adapter,
        requested_model,
        request,
        artifact,
        max_tokens,
        random_seed,
    )
    .await
}

pub async fn run_semantic_runtime(
    profile: SemanticRuntimeProfile,
    adapter: &dyn ModelAdapter,
    requested_model: &str,
    request: &SoftJudgeRequest,
    artifact: &ReasoningArtifact,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<SemanticRuntimeObservation, SemanticRuntimeError> {
    if requested_model.trim().is_empty() {
        return Err(SemanticRuntimeError::InvalidRequestedModel);
    }

    let runtime = profile.identity();
    let judge_identity = SoftJudgeIdentity {
        judge_id: format!("semantic-runtime:{}", runtime.configuration_id),
        model_id: requested_model.to_string(),
        configuration_id: runtime.configuration_id.clone(),
    };

    match profile {
        SemanticRuntimeProfile::SoftSemanticV3 => {
            let result = run_model_backed_soft_judge(
                adapter,
                judge_identity,
                request,
                max_tokens,
                random_seed,
            )
            .await
            .map_err(SemanticRuntimeError::Baseline)?;
            let base_decision = result.observation.decision;
            Ok(SemanticRuntimeObservation {
                runtime,
                observation: result.observation,
                base_decision,
                decidability: None,
                model: result.model,
                usage: result.usage,
                provider_attempts: result.provider_attempts,
                fallback_reason: Some(result.fallback_reason),
                finish_reason: None,
            })
        }
        SemanticRuntimeProfile::SemanticDecidabilityD3V1 => {
            // The harness-owned typed gate is evaluated independently of the model response. It
            // can only preserve the R2 base decision or force abstention after materialization.
            let decidability = assess_semantic_decidability(request, artifact)?;
            let base = run_model_backed_soft_judge_materialization(
                adapter,
                request,
                max_tokens,
                random_seed,
            )
            .await?;
            let base_decision = base.decision;
            let decision = compose_semantic_decidability(base_decision, &decidability);
            let output = materialize_soft_judge_output(
                request,
                &MaterializedDecisionOutput {
                    decision,
                    advisory_note: base.advisory_note,
                },
            );
            Ok(SemanticRuntimeObservation {
                runtime,
                observation: SoftJudgeObservation {
                    judge: judge_identity,
                    request_id: request.id.clone(),
                    decision: output.decision,
                    finding: output.finding,
                },
                base_decision,
                decidability: Some(decidability),
                model: base.model,
                usage: base.usage,
                provider_attempts: 1,
                fallback_reason: None,
                finish_reason: base.finish_reason,
            })
        }
    }
}
