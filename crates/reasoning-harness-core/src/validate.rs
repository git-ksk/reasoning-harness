use std::collections::HashSet;

use crate::{EpistemicState, ReasoningArtifact};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub fn validate_artifact(artifact: &ReasoningArtifact) -> ValidationReport {
    let mut diagnostics = Vec::new();
    let mut ids = HashSet::new();

    for evidence in &artifact.evidence {
        if !ids.insert(format!("evidence:{}", evidence.id)) {
            diagnostics.push(Diagnostic {
                code: "duplicate_evidence_id",
                message: format!("duplicate evidence id: {}", evidence.id),
            });
        }
    }

    let evidence_ids: HashSet<&str> = artifact.evidence.iter().map(|e| e.id.as_str()).collect();
    let claim_ids: HashSet<&str> = artifact.claims.iter().map(|c| c.id.as_str()).collect();

    for claim in &artifact.claims {
        if !ids.insert(format!("claim:{}", claim.id)) {
            diagnostics.push(Diagnostic {
                code: "duplicate_claim_id",
                message: format!("duplicate claim id: {}", claim.id),
            });
        }

        if matches!(
            claim.state,
            EpistemicState::Known | EpistemicState::Supported
        ) && claim.evidence_ids.is_empty()
        {
            diagnostics.push(Diagnostic {
                code: "accepted_claim_without_evidence",
                message: format!(
                    "claim {} requires evidence for state {:?}",
                    claim.id, claim.state
                ),
            });
        }

        for evidence_id in &claim.evidence_ids {
            if !evidence_ids.contains(evidence_id.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "missing_evidence_reference",
                    message: format!(
                        "claim {} references missing evidence {}",
                        claim.id, evidence_id
                    ),
                });
            }
        }
    }

    for inference in &artifact.inferences {
        for premise in &inference.premise_claim_ids {
            if !claim_ids.contains(premise.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "missing_premise_claim",
                    message: format!(
                        "inference {} references missing premise {}",
                        inference.id, premise
                    ),
                });
            }
        }
        if !claim_ids.contains(inference.conclusion_claim_id.as_str()) {
            diagnostics.push(Diagnostic {
                code: "missing_conclusion_claim",
                message: format!(
                    "inference {} references missing conclusion {}",
                    inference.id, inference.conclusion_claim_id
                ),
            });
        }
    }

    ValidationReport { diagnostics }
}
