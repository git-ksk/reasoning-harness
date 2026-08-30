use std::collections::HashSet;

use crate::{
    CandidateDiagnostic, Claim, EpistemicState, HarnessInput, Inference, ReasoningArtifact,
    ReasoningCandidate,
};

/// Materializes an untrusted model candidate using harness-owned task and evidence.
///
/// Strong epistemic states are downgraded until a trusted pass establishes them. Duplicate
/// untrusted claim IDs and invalid inference suggestions are isolated rather than allowed to
/// invalidate unrelated claims; every dropped item is recorded in `candidate_diagnostics`.
pub fn materialize_candidate(
    input: HarnessInput,
    candidate: ReasoningCandidate,
) -> ReasoningArtifact {
    let valid_evidence_ids = input
        .evidence
        .iter()
        .map(|evidence| evidence.id.clone())
        .collect::<HashSet<_>>();
    let (mut claims, mut candidate_diagnostics) =
        normalize_claims(candidate.claims, &valid_evidence_ids);
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
    let (inferences, inference_diagnostics) =
        normalize_inferences(candidate.inferences, &claim_ids);
    candidate_diagnostics.extend(inference_diagnostics);

    ReasoningArtifact {
        task: input.task,
        evidence: input.evidence,
        hypotheses: input.hypotheses,
        assumptions: input.assumptions,
        evidence_requirements: input.evidence_requirements,
        authority_policy: input.authority_policy,
        candidate_diagnostics,
        verification_receipts: Vec::new(),
        adversarial_findings: Vec::new(),
        assumption_findings: Vec::new(),
        evidence_qualification_findings: Vec::new(),
        claims,
        inferences,
    }
}

fn normalize_claims(
    claims: Vec<crate::CandidateClaim>,
    valid_evidence_ids: &HashSet<String>,
) -> (Vec<Claim>, Vec<CandidateDiagnostic>) {
    let mut accepted = Vec::new();
    let mut diagnostics = Vec::new();
    let mut claim_ids = HashSet::new();

    for claim in claims {
        if !claim_ids.insert(claim.id.clone()) {
            diagnostics.push(CandidateDiagnostic {
                code: "dropped_duplicate_claim_id".into(),
                message: format!("duplicate candidate claim id {}", claim.id),
            });
            continue;
        }
        let mut evidence_ids = Vec::new();
        for evidence_id in claim.evidence_ids {
            if valid_evidence_ids.contains(&evidence_id) {
                evidence_ids.push(evidence_id);
            } else {
                diagnostics.push(CandidateDiagnostic {
                    code: "dropped_missing_candidate_evidence_reference".into(),
                    message: format!(
                        "candidate claim {} references missing evidence {}",
                        claim.id, evidence_id
                    ),
                });
            }
        }
        accepted.push(Claim {
            id: claim.id,
            statement: claim.statement,
            state: materialized_state(claim.proposed_state),
            proposition: claim.proposition,
            evidence_ids,
        });
    }

    (accepted, diagnostics)
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
    use crate::{CandidateClaim, Evidence, HarnessInput};

    #[test]
    fn drops_duplicate_candidate_claim_id_but_preserves_first_claim() {
        let candidate = ReasoningCandidate {
            claims: vec![
                CandidateClaim {
                    id: "c1".into(),
                    statement: "first".into(),
                    proposed_state: EpistemicState::Supported,
                    proposition: None,
                    evidence_ids: vec![],
                },
                CandidateClaim {
                    id: "c1".into(),
                    statement: "duplicate".into(),
                    proposed_state: EpistemicState::Known,
                    proposition: None,
                    evidence_ids: vec![],
                },
            ],
            inferences: vec![],
        };
        let artifact = materialize_candidate(
            HarnessInput {
                task: "task".into(),
                evidence: vec![],
                hypotheses: vec![],
                assumptions: vec![],
                evidence_requirements: vec![],
                authority_policy: Default::default(),
            },
            candidate,
        );

        assert_eq!(artifact.claims.len(), 1);
        assert_eq!(artifact.claims[0].statement, "first");
        assert_eq!(artifact.claims[0].state, EpistemicState::Assumed);
        assert_eq!(artifact.candidate_diagnostics.len(), 1);
        assert_eq!(
            artifact.candidate_diagnostics[0].code,
            "dropped_duplicate_claim_id"
        );
    }

    #[test]
    fn drops_missing_candidate_evidence_reference_but_keeps_valid_reference() {
        let candidate = ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: "c1".into(),
                statement: "claim".into(),
                proposed_state: EpistemicState::Supported,
                proposition: None,
                evidence_ids: vec!["e1".into(), "invented".into()],
            }],
            inferences: vec![],
        };
        let artifact = materialize_candidate(
            HarnessInput {
                task: "task".into(),
                evidence: vec![Evidence {
                    id: "e1".into(),
                    source: "fixture".into(),
                    observation: "observed".into(),
                    facts: Default::default(),
                    metadata: Default::default(),
                }],
                hypotheses: vec![],
                assumptions: vec![],
                evidence_requirements: vec![],
                authority_policy: Default::default(),
            },
            candidate,
        );

        assert_eq!(artifact.claims[0].evidence_ids, vec!["e1"]);
        assert_eq!(artifact.claims[0].state, EpistemicState::Assumed);
        assert_eq!(artifact.candidate_diagnostics.len(), 1);
        assert_eq!(
            artifact.candidate_diagnostics[0].code,
            "dropped_missing_candidate_evidence_reference"
        );
    }

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
                assumptions: vec![],
                evidence_requirements: vec![],
                authority_policy: Default::default(),
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
