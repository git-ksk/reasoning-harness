use std::collections::HashSet;

use crate::{
    CandidateDiagnostic, Claim, EpistemicState, HarnessInput, Inference, ReasoningArtifact,
    ReasoningCandidate,
};

/// Materializes an untrusted model candidate using harness-owned task and evidence.
///
/// Strong epistemic states are downgraded until a trusted pass establishes them. Invalid
/// inference suggestions are dropped rather than allowed to invalidate unrelated claims;
/// every dropped edge is recorded in `candidate_diagnostics` so normalization is inspectable.
pub fn materialize_candidate(
    input: HarnessInput,
    candidate: ReasoningCandidate,
) -> ReasoningArtifact {
    let mut claims = candidate
        .claims
        .into_iter()
        .map(|claim| Claim {
            id: claim.id,
            statement: claim.statement,
            state: materialized_state(claim.proposed_state),
            proposition: claim.proposition,
            evidence_ids: claim.evidence_ids,
        })
        .collect::<Vec<_>>();
    for (index, proposition) in input.hypotheses.iter().enumerate() {
        if claims
            .iter()
            .any(|claim| claim.proposition.as_ref() == Some(proposition))
        {
            continue;
        }
        let base = format!("harness_hypothesis_{index}");
        let mut id = base.clone();
        let mut suffix = 1usize;
        while claims.iter().any(|claim| claim.id == id) {
            id = format!("{base}_{suffix}");
            suffix += 1;
        }
        claims.push(Claim {
            id,
            statement: format!("{} = {}", proposition.key, proposition.value),
            state: EpistemicState::Assumed,
            proposition: Some(proposition.clone()),
            evidence_ids: vec![],
        });
    }
    let claim_ids = claims
        .iter()
        .map(|claim| claim.id.as_str())
        .collect::<HashSet<_>>();
    let (inferences, candidate_diagnostics) =
        normalize_inferences(candidate.inferences, &claim_ids);

    ReasoningArtifact {
        task: input.task,
        evidence: input.evidence,
        hypotheses: input.hypotheses,
        candidate_diagnostics,
        verification_receipts: Vec::new(),
        adversarial_findings: Vec::new(),
        claims,
        inferences,
    }
}

fn normalize_inferences(
    inferences: Vec<Inference>,
    claim_ids: &HashSet<&str>,
) -> (Vec<Inference>, Vec<CandidateDiagnostic>) {
    let mut accepted = Vec::new();
    let mut diagnostics = Vec::new();
    let mut inference_ids = HashSet::new();

    for inference in inferences {
        let reason = if inference.id.trim().is_empty() {
            Some((
                "dropped_empty_inference_id",
                "inference id is empty".to_string(),
            ))
        } else if !inference_ids.insert(inference.id.clone()) {
            Some((
                "dropped_duplicate_inference_id",
                format!("duplicate inference id {}", inference.id),
            ))
        } else if inference.method.trim().is_empty() {
            Some((
                "dropped_empty_inference_method",
                format!("inference {} has an empty method", inference.id),
            ))
        } else if inference.premise_claim_ids.is_empty() {
            Some((
                "dropped_inference_without_premises",
                format!("inference {} has no premises", inference.id),
            ))
        } else if let Some(missing) = inference
            .premise_claim_ids
            .iter()
            .find(|premise| !claim_ids.contains(premise.as_str()))
        {
            Some((
                "dropped_missing_premise_claim",
                format!(
                    "inference {} references missing premise {}",
                    inference.id, missing
                ),
            ))
        } else if !claim_ids.contains(inference.conclusion_claim_id.as_str()) {
            Some((
                "dropped_missing_conclusion_claim",
                format!(
                    "inference {} references missing conclusion {}",
                    inference.id, inference.conclusion_claim_id
                ),
            ))
        } else {
            None
        };

        if let Some((code, message)) = reason {
            diagnostics.push(CandidateDiagnostic {
                code: code.into(),
                message,
            });
        } else {
            accepted.push(inference);
        }
    }

    (accepted, diagnostics)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CandidateClaim, HarnessInput};

    #[test]
    fn drops_invalid_inference_but_preserves_claim_and_diagnostic() {
        let candidate = ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: "c1".into(),
                statement: "claim".into(),
                proposed_state: EpistemicState::Assumed,
                proposition: None,
                evidence_ids: vec![],
            }],
            inferences: vec![Inference {
                id: "i1".into(),
                premise_claim_ids: vec!["e1".into()],
                conclusion_claim_id: "c1".into(),
                method: "candidate".into(),
            }],
        };
        let artifact = materialize_candidate(
            HarnessInput {
                task: "task".into(),
                evidence: vec![],
                hypotheses: vec![],
            },
            candidate,
        );

        assert_eq!(artifact.claims.len(), 1);
        assert!(artifact.inferences.is_empty());
        assert_eq!(artifact.candidate_diagnostics.len(), 1);
        assert_eq!(
            artifact.candidate_diagnostics[0].code,
            "dropped_missing_premise_claim"
        );
    }
}
