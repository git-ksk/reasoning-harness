use std::collections::{BTreeSet, HashSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::evidence_qualification::qualification_reasons;
use crate::{
    Proposition, ReasoningArtifact, SemanticDiagnosticTarget, SoftJudgeDecision, SoftJudgeRequest,
    validate_artifact,
};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDecidabilityDisposition {
    Permit,
    ForceAbstain,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDecidabilityReason {
    MissingTargetBinding,
    MissingPropositionBinding,
    NoEvidenceForExplicitRequirement,
    NoQualifiedEvidenceForExplicitRequirement,
    ConflictingQualifiedEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SemanticDecidabilityAssessment {
    pub disposition: SemanticDecidabilityDisposition,
    #[serde(default)]
    pub reasons: Vec<SemanticDecidabilityReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDecidabilityCalibrationFixture {
    pub id: String,
    pub pair_id: String,
    pub mutation_family: String,
    pub request: SoftJudgeRequest,
    pub artifact: ReasoningArtifact,
    pub expected_disposition: SemanticDecidabilityDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SemanticDecidabilityError {
    #[error("semantic decidability requires a valid reasoning artifact: {diagnostic_codes:?}")]
    InvalidArtifact { diagnostic_codes: Vec<String> },
}

pub fn assess_semantic_decidability(
    request: &SoftJudgeRequest,
    artifact: &ReasoningArtifact,
) -> Result<SemanticDecidabilityAssessment, SemanticDecidabilityError> {
    let validation = validate_artifact(artifact);
    if !validation.is_ok() {
        return Err(SemanticDecidabilityError::InvalidArtifact {
            diagnostic_codes: validation
                .diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.code.to_string())
                .collect(),
        });
    }

    let mut reasons = BTreeSet::new();
    let propositions = target_propositions(request, artifact, &mut reasons);

    for proposition in propositions {
        for requirement in artifact
            .evidence_requirements
            .iter()
            .filter(|requirement| requirement.proposition == proposition)
        {
            let candidates = artifact
                .evidence
                .iter()
                .filter(|evidence| evidence.facts.contains_key(&requirement.proposition.key))
                .collect::<Vec<_>>();

            if candidates.is_empty() {
                reasons.insert(SemanticDecidabilityReason::NoEvidenceForExplicitRequirement);
                continue;
            }

            let qualified = candidates
                .into_iter()
                .filter(|evidence| {
                    qualification_reasons(&artifact.authority_policy, requirement, evidence)
                        .is_empty()
                })
                .collect::<Vec<_>>();

            if qualified.is_empty() {
                reasons
                    .insert(SemanticDecidabilityReason::NoQualifiedEvidenceForExplicitRequirement);
                continue;
            }

            let distinct_values = qualified
                .iter()
                .filter_map(|evidence| evidence.facts.get(&requirement.proposition.key))
                .collect::<HashSet<_>>();
            if distinct_values.len() > 1 {
                reasons.insert(SemanticDecidabilityReason::ConflictingQualifiedEvidence);
            }
        }
    }

    let reasons = reasons.into_iter().collect::<Vec<_>>();
    let disposition = if reasons.is_empty() {
        SemanticDecidabilityDisposition::Permit
    } else {
        SemanticDecidabilityDisposition::ForceAbstain
    };

    Ok(SemanticDecidabilityAssessment {
        disposition,
        reasons,
    })
}

pub fn compose_semantic_decidability(
    base: SoftJudgeDecision,
    assessment: &SemanticDecidabilityAssessment,
) -> SoftJudgeDecision {
    match assessment.disposition {
        SemanticDecidabilityDisposition::Permit => base,
        SemanticDecidabilityDisposition::ForceAbstain => SoftJudgeDecision::Abstain,
    }
}

fn target_propositions(
    request: &SoftJudgeRequest,
    artifact: &ReasoningArtifact,
    reasons: &mut BTreeSet<SemanticDecidabilityReason>,
) -> Vec<Proposition> {
    let mut propositions = Vec::new();
    match &request.target {
        SemanticDiagnosticTarget::Proposition { proposition } => {
            push_unique(&mut propositions, proposition.clone());
        }
        SemanticDiagnosticTarget::CausalRelation { relation } => {
            for cause in &relation.causes {
                push_unique(&mut propositions, cause.clone());
            }
            push_unique(&mut propositions, relation.effect.clone());
        }
        SemanticDiagnosticTarget::Claim { claim_id } => {
            let Some(claim) = artifact.claims.iter().find(|claim| claim.id == *claim_id) else {
                reasons.insert(SemanticDecidabilityReason::MissingTargetBinding);
                return propositions;
            };
            match &claim.proposition {
                Some(proposition) => push_unique(&mut propositions, proposition.clone()),
                None => {
                    reasons.insert(SemanticDecidabilityReason::MissingPropositionBinding);
                }
            }
        }
        SemanticDiagnosticTarget::Inference { inference_id } => {
            let Some(inference) = artifact
                .inferences
                .iter()
                .find(|inference| inference.id == *inference_id)
            else {
                reasons.insert(SemanticDecidabilityReason::MissingTargetBinding);
                return propositions;
            };

            for claim_id in inference
                .premise_claim_ids
                .iter()
                .chain(std::iter::once(&inference.conclusion_claim_id))
            {
                let Some(claim) = artifact.claims.iter().find(|claim| claim.id == *claim_id) else {
                    reasons.insert(SemanticDecidabilityReason::MissingTargetBinding);
                    continue;
                };
                match &claim.proposition {
                    Some(proposition) => push_unique(&mut propositions, proposition.clone()),
                    None => {
                        reasons.insert(SemanticDecidabilityReason::MissingPropositionBinding);
                    }
                }
            }
        }
    }
    propositions
}

fn push_unique(propositions: &mut Vec<Proposition>, proposition: Proposition) {
    if !propositions.contains(&proposition) {
        propositions.push(proposition);
    }
}
