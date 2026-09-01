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
