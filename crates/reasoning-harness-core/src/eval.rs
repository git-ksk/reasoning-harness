use serde::Serialize;

use crate::{EpistemicState, ReasoningArtifact, validate_artifact};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvalMetrics {
    pub valid: bool,
    pub evidence_coverage: f64,
    pub explicit_unknown_rate: f64,
    pub accepted_without_evidence: usize,
}

pub fn evaluate(artifact: &ReasoningArtifact) -> EvalMetrics {
    let total_claims = artifact.claims.len();
    let evidence_bound = artifact
        .claims
        .iter()
        .filter(|claim| !claim.evidence_ids.is_empty())
        .count();
    let unknown = artifact
        .claims
        .iter()
        .filter(|claim| claim.state == EpistemicState::Unknown)
        .count();
    let accepted_without_evidence = artifact
        .claims
        .iter()
        .filter(|claim| {
            matches!(
                claim.state,
                EpistemicState::Known | EpistemicState::Supported
            ) && claim.evidence_ids.is_empty()
        })
        .count();

    let denominator = total_claims.max(1) as f64;
    EvalMetrics {
        valid: validate_artifact(artifact).is_ok(),
        evidence_coverage: evidence_bound as f64 / denominator,
        explicit_unknown_rate: unknown as f64 / denominator,
        accepted_without_evidence,
    }
}
