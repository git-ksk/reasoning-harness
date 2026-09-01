use std::time::Instant;

use serde::{Deserialize, Deserializer, Serialize, de};
use thiserror::Error;

use crate::{
    D3_DECIDABILITY_CONTRACT_ID, EvidenceSufficiencyLabel, EvidenceSufficiencyModelError,
    EvidenceSufficiencyObservation, EvidenceSufficiencyRequest, ModelAdapter, Proposition,
    ReasoningArtifact, SemanticDecidabilityAssessment, SemanticDecidabilityDisposition,
    SemanticDecidabilityError, SemanticDiagnosticKind, SemanticDiagnosticTarget, SoftJudgeRequest,
    assess_semantic_decidability, run_model_backed_evidence_sufficiency,
};

pub const BASELINE_ANSWER_SAFETY_CONFIGURATION_ID: &str = "grounded-finalization-v1";
pub const D3_SUFFICIENCY_ANSWER_SAFETY_CONFIGURATION_ID: &str = "d3-sufficiency-answer-gate-v1";
pub const EVIDENCE_SUFFICIENCY_RSD1_CONTRACT_ID: &str = "evidence-sufficiency-coordinate-rsd1-v1";
pub const GENERIC_ANSWER_SUFFICIENCY_REQUIREMENT_POLICY_ID: &str =
    "generic-answer-sufficiency-requirements-v1";
pub const ANSWER_SAFETY_IDENTITY_VERSION: &str = "answer-safety-identity-v1";

const GENERIC_REQUIREMENT_DECISION_COVERAGE: &str = "The selected evidence must cover the decision-critical information needed by the stated task to justify the typed target, rather than merely being related to it.";
const GENERIC_REQUIREMENT_NO_OVERREACH: &str = "The target must not rely on missing facts, unresolved conflicting evidence, or a causal or general conclusion stronger than the selected evidence supports.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerSafetyProfile {
    Baseline,
    D3SufficiencyV1,
}

impl AnswerSafetyProfile {
    pub const fn configuration_id(self) -> &'static str {
        match self {
            Self::Baseline => BASELINE_ANSWER_SAFETY_CONFIGURATION_ID,
            Self::D3SufficiencyV1 => D3_SUFFICIENCY_ANSWER_SAFETY_CONFIGURATION_ID,
        }
    }

    pub const fn rollback_profile(self) -> Option<Self> {
        match self {
            Self::Baseline => None,
            Self::D3SufficiencyV1 => Some(Self::Baseline),
        }
    }

    pub fn identity(self) -> AnswerSafetyIdentity {
        match self {
            Self::Baseline => AnswerSafetyIdentity {
                identity_version: ANSWER_SAFETY_IDENTITY_VERSION.into(),
                profile: self,
                configuration_id: BASELINE_ANSWER_SAFETY_CONFIGURATION_ID.into(),
                decidability_contract: None,
                sufficiency_contract: None,
                requirement_policy: None,
                rollback_configuration_id: None,
            },
            Self::D3SufficiencyV1 => AnswerSafetyIdentity {
                identity_version: ANSWER_SAFETY_IDENTITY_VERSION.into(),
                profile: self,
                configuration_id: D3_SUFFICIENCY_ANSWER_SAFETY_CONFIGURATION_ID.into(),
                decidability_contract: Some(D3_DECIDABILITY_CONTRACT_ID.into()),
                sufficiency_contract: Some(EVIDENCE_SUFFICIENCY_RSD1_CONTRACT_ID.into()),
                requirement_policy: Some(GENERIC_ANSWER_SUFFICIENCY_REQUIREMENT_POLICY_ID.into()),
                rollback_configuration_id: Some(BASELINE_ANSWER_SAFETY_CONFIGURATION_ID.into()),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnswerSafetyIdentity {
    identity_version: String,
    profile: AnswerSafetyProfile,
    configuration_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    decidability_contract: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    sufficiency_contract: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    requirement_policy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    rollback_configuration_id: Option<String>,
}

impl AnswerSafetyIdentity {
    pub fn identity_version(&self) -> &str {
        &self.identity_version
    }

    pub const fn profile(&self) -> AnswerSafetyProfile {
        self.profile
    }

    pub fn configuration_id(&self) -> &str {
        &self.configuration_id
    }

    pub fn decidability_contract(&self) -> Option<&str> {
        self.decidability_contract.as_deref()
    }

    pub fn sufficiency_contract(&self) -> Option<&str> {
        self.sufficiency_contract.as_deref()
    }

    pub fn requirement_policy(&self) -> Option<&str> {
        self.requirement_policy.as_deref()
    }

    pub fn rollback_configuration_id(&self) -> Option<&str> {
        self.rollback_configuration_id.as_deref()
    }
}

#[derive(Deserialize)]
struct AnswerSafetyIdentityWire {
    identity_version: String,
    profile: AnswerSafetyProfile,
    configuration_id: String,
    #[serde(default)]
    decidability_contract: Option<String>,
    #[serde(default)]
    sufficiency_contract: Option<String>,
    #[serde(default)]
    requirement_policy: Option<String>,
    #[serde(default)]
    rollback_configuration_id: Option<String>,
}

impl<'de> Deserialize<'de> for AnswerSafetyIdentity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AnswerSafetyIdentityWire::deserialize(deserializer)?;
        let expected = wire.profile.identity();
        let actual = AnswerSafetyIdentity {
            identity_version: wire.identity_version,
            profile: wire.profile,
            configuration_id: wire.configuration_id,
            decidability_contract: wire.decidability_contract,
            sufficiency_contract: wire.sufficiency_contract,
            requirement_policy: wire.requirement_policy,
            rollback_configuration_id: wire.rollback_configuration_id,
        };
        if actual != expected {
            return Err(de::Error::custom(
                "answer safety identity does not match its canonical profile",
            ));
        }
        Ok(actual)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerSafetyDisposition {
    PreserveBaseline,
    ForceVerification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnswerSafetyReason {
    D3Precondition,
    EvidenceInsufficient,
    EvidenceMixed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnswerSafetyObservation {
    pub runtime: AnswerSafetyIdentity,
    pub target: Proposition,
    pub disposition: AnswerSafetyDisposition,
    #[serde(default)]
    pub reasons: Vec<AnswerSafetyReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decidability: Option<SemanticDecidabilityAssessment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sufficiency: Option<EvidenceSufficiencyObservation>,
    pub latency_ms: u128,
}

#[derive(Debug, Error)]
pub enum AnswerSafetyError {
    #[error("answer safety requested model must not be empty")]
    InvalidRequestedModel,
    #[error("D3 answer-safety precondition failed: {0}")]
    Decidability(#[from] SemanticDecidabilityError),
    #[error("evidence-sufficiency answer gate failed: {0}")]
    Sufficiency(#[from] EvidenceSufficiencyModelError),
}

pub fn build_answer_sufficiency_request(
    target: &Proposition,
    artifact: &ReasoningArtifact,
) -> EvidenceSufficiencyRequest {
    EvidenceSufficiencyRequest {
        id: format!("answer-sufficiency:{}={}", target.key, target.value),
        task: artifact.task.clone(),
        target: target.clone(),
        required_information: vec![
            GENERIC_REQUIREMENT_DECISION_COVERAGE.into(),
            GENERIC_REQUIREMENT_NO_OVERREACH.into(),
        ],
        evidence_ids: artifact
            .evidence
            .iter()
            .map(|evidence| evidence.id.clone())
            .collect(),
    }
}

pub async fn run_answer_safety_gate(
    profile: AnswerSafetyProfile,
    adapter: &dyn ModelAdapter,
    requested_model: &str,
    target: &Proposition,
    artifact: &ReasoningArtifact,
    max_tokens: u32,
    random_seed: Option<u64>,
) -> Result<AnswerSafetyObservation, AnswerSafetyError> {
    let runtime = profile.identity();
    if profile == AnswerSafetyProfile::Baseline {
        return Ok(AnswerSafetyObservation {
            runtime,
            target: target.clone(),
            disposition: AnswerSafetyDisposition::PreserveBaseline,
            reasons: vec![],
            decidability: None,
            sufficiency: None,
            latency_ms: 0,
        });
    }
    if requested_model.trim().is_empty() {
        return Err(AnswerSafetyError::InvalidRequestedModel);
    }

    let d3_request = SoftJudgeRequest {
        id: format!("answer-d3:{}={}", target.key, target.value),
        task: artifact.task.clone(),
        kind: SemanticDiagnosticKind::UnsupportedPremise,
        target: SemanticDiagnosticTarget::Proposition {
            proposition: target.clone(),
        },
        context: vec![],
    };
    let decidability = assess_semantic_decidability(&d3_request, artifact)?;
    if decidability.disposition == SemanticDecidabilityDisposition::ForceAbstain {
        return Ok(AnswerSafetyObservation {
            runtime,
            target: target.clone(),
            disposition: AnswerSafetyDisposition::ForceVerification,
            reasons: vec![AnswerSafetyReason::D3Precondition],
            decidability: Some(decidability),
            sufficiency: None,
            latency_ms: 0,
        });
    }

    let request = build_answer_sufficiency_request(target, artifact);
    let started = Instant::now();
    let sufficiency =
        run_model_backed_evidence_sufficiency(adapter, &request, artifact, max_tokens, random_seed)
            .await?;
    let (disposition, reasons) = match sufficiency.decision {
        EvidenceSufficiencyLabel::Sufficient => (AnswerSafetyDisposition::PreserveBaseline, vec![]),
        EvidenceSufficiencyLabel::Insufficient => (
            AnswerSafetyDisposition::ForceVerification,
            vec![AnswerSafetyReason::EvidenceInsufficient],
        ),
        EvidenceSufficiencyLabel::Mixed => (
            AnswerSafetyDisposition::ForceVerification,
            vec![AnswerSafetyReason::EvidenceMixed],
        ),
    };
    Ok(AnswerSafetyObservation {
        runtime,
        target: target.clone(),
        disposition,
        reasons,
        decidability: Some(decidability),
        sufficiency: Some(sufficiency),
        latency_ms: started.elapsed().as_millis(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{
        Evidence, EvidenceMetadata, ModelOutputFormat, ModelRequest, ModelResponse, ModelUsage,
    };

    use super::*;

    struct FixedAdapter {
        decision: &'static str,
    }

    impl ModelAdapter for FixedAdapter {
        fn generate<'a>(
            &'a self,
            request: ModelRequest,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<Output = Result<ModelResponse, crate::ModelError>>
                    + Send
                    + 'a,
            >,
        > {
            Box::pin(async move {
                assert!(matches!(
                    request.output_format,
                    ModelOutputFormat::JsonSchema { .. }
                ));
                Ok(ModelResponse {
                    text: format!(r#"{{"decision":"{}"}}"#, self.decision),
                    model: "fixed".into(),
                    usage: ModelUsage::default(),
                    finish_reason: Some("stop".into()),
                })
            })
        }
    }

    fn artifact() -> ReasoningArtifact {
        ReasoningArtifact {
            task: "Decide whether deployment is safe.".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "fixture".into(),
                observation: "deployment checks passed".into(),
                facts: BTreeMap::from([("deployment.safe".into(), "true".into())]),
                metadata: EvidenceMetadata::default(),
            }],
            ..Default::default()
        }
    }

    fn target() -> Proposition {
        Proposition {
            key: "deployment.safe".into(),
            value: "true".into(),
        }
    }

    #[tokio::test]
    async fn sufficient_can_only_preserve_baseline() {
        let observation = run_answer_safety_gate(
            AnswerSafetyProfile::D3SufficiencyV1,
            &FixedAdapter {
                decision: "sufficient",
            },
            "fixed",
            &target(),
            &artifact(),
            64,
            Some(1),
        )
        .await
        .unwrap();
        assert_eq!(
            observation.disposition,
            AnswerSafetyDisposition::PreserveBaseline
        );
        assert!(observation.reasons.is_empty());
    }

    #[tokio::test]
    async fn non_sufficient_can_only_force_verification() {
        for decision in ["insufficient", "mixed"] {
            let observation = run_answer_safety_gate(
                AnswerSafetyProfile::D3SufficiencyV1,
                &FixedAdapter { decision },
                "fixed",
                &target(),
                &artifact(),
                64,
                Some(1),
            )
            .await
            .unwrap();
            assert_eq!(
                observation.disposition,
                AnswerSafetyDisposition::ForceVerification
            );
        }
    }

    #[tokio::test]
    async fn baseline_profile_never_calls_model() {
        let observation = run_answer_safety_gate(
            AnswerSafetyProfile::Baseline,
            &FixedAdapter {
                decision: "insufficient",
            },
            "",
            &target(),
            &artifact(),
            0,
            None,
        )
        .await
        .unwrap();
        assert_eq!(
            observation.disposition,
            AnswerSafetyDisposition::PreserveBaseline
        );
        assert!(observation.sufficiency.is_none());
        assert!(observation.decidability.is_none());
    }

    #[test]
    fn successor_identity_has_explicit_baseline_rollback() {
        let identity = AnswerSafetyProfile::D3SufficiencyV1.identity();
        assert_eq!(
            identity.rollback_configuration_id(),
            Some(BASELINE_ANSWER_SAFETY_CONFIGURATION_ID)
        );
        assert_eq!(
            identity.decidability_contract(),
            Some(D3_DECIDABILITY_CONTRACT_ID)
        );
        assert_eq!(
            identity.sufficiency_contract(),
            Some(EVIDENCE_SUFFICIENCY_RSD1_CONTRACT_ID)
        );
        assert_eq!(
            identity.requirement_policy(),
            Some(GENERIC_ANSWER_SUFFICIENCY_REQUIREMENT_POLICY_ID)
        );
    }

    #[test]
    fn request_builder_uses_only_existing_evidence_ids() {
        let request = build_answer_sufficiency_request(&target(), &artifact());
        assert_eq!(request.evidence_ids, vec!["e1"]);
        assert_eq!(request.required_information.len(), 2);
    }
}
