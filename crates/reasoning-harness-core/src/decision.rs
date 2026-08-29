use crate::{EpistemicState, ReasoningArtifact, Verdict};

/// Decides the epistemic outcome of an already-valid reasoning artifact.
///
/// Acceptance policy is part of the trusted runtime boundary. Provider adapters and
/// reasoning passes cannot override the verdict produced here.
pub trait AcceptancePolicy: Send + Sync {
    fn decide(&self, artifact: &ReasoningArtifact) -> Verdict;
}

/// Conservative aggregate policy for the initial research harness.
///
/// Contradictions reject the artifact. Explicit assumptions or unknowns preserve
/// uncertainty. An empty artifact is unknown rather than vacuously accepted.
#[derive(Debug, Clone, Copy, Default)]
pub struct StrictAcceptancePolicy;

impl AcceptancePolicy for StrictAcceptancePolicy {
    fn decide(&self, artifact: &ReasoningArtifact) -> Verdict {
        if artifact.claims.is_empty() {
            return Verdict::Unknown;
        }

        if artifact
            .claims
            .iter()
            .any(|claim| claim.state == EpistemicState::Contradicted)
        {
            return Verdict::Reject;
        }

        if artifact.claims.iter().any(|claim| {
            matches!(
                claim.state,
                EpistemicState::Assumed | EpistemicState::Unknown
            )
        }) {
            return Verdict::Unknown;
        }

        Verdict::Accept
    }
}
