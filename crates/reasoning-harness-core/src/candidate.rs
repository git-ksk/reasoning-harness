use crate::{Claim, EpistemicState, HarnessInput, ReasoningArtifact, ReasoningCandidate};

/// Materializes an untrusted model candidate using harness-owned task and evidence.
///
/// The model is allowed to propose epistemic states, but it cannot directly create a
/// trusted `known`, `supported`, `inferred`, or `contradicted` claim. Those states must
/// be established later by harness-owned verification passes.
pub fn materialize_candidate(
    input: HarnessInput,
    candidate: ReasoningCandidate,
) -> ReasoningArtifact {
    let claims = candidate
        .claims
        .into_iter()
        .map(|claim| Claim {
            id: claim.id,
            statement: claim.statement,
            state: materialized_state(claim.proposed_state),
            proposition: claim.proposition,
            evidence_ids: claim.evidence_ids,
        })
        .collect();

    ReasoningArtifact {
        task: input.task,
        evidence: input.evidence,
        verification_receipts: Vec::new(),
        claims,
        inferences: candidate.inferences,
    }
}

fn materialized_state(proposed: EpistemicState) -> EpistemicState {
    match proposed {
        EpistemicState::Unknown => EpistemicState::Unknown,
        EpistemicState::Assumed => EpistemicState::Assumed,
        EpistemicState::Known
        | EpistemicState::Supported
        | EpistemicState::Inferred
        | EpistemicState::Contradicted => EpistemicState::Assumed,
    }
}
