use std::collections::HashSet;

use serde::Serialize;

use crate::{EpistemicState, ReasoningArtifact};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    let mut evidence_ids = HashSet::new();
    let mut claim_ids = HashSet::new();
    let mut inference_ids = HashSet::new();

    for evidence in &artifact.evidence {
        if evidence.id.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_evidence_id",
                message: "evidence id must not be empty".into(),
            });
        }
        if !evidence_ids.insert(evidence.id.as_str()) {
            diagnostics.push(Diagnostic {
                code: "duplicate_evidence_id",
                message: format!("duplicate evidence id: {}", evidence.id),
            });
        }
        if evidence.source.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_evidence_source",
                message: format!("evidence {} has an empty source", evidence.id),
            });
        }
        if evidence.observation.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_evidence_observation",
                message: format!("evidence {} has an empty observation", evidence.id),
            });
        }
    }

    for claim in &artifact.claims {
        if claim.id.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_claim_id",
                message: "claim id must not be empty".into(),
            });
        }
        if !claim_ids.insert(claim.id.as_str()) {
            diagnostics.push(Diagnostic {
                code: "duplicate_claim_id",
                message: format!("duplicate claim id: {}", claim.id),
            });
        }
        if claim.statement.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_claim_statement",
                message: format!("claim {} has an empty statement", claim.id),
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
        if inference.id.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_inference_id",
                message: "inference id must not be empty".into(),
            });
        }
        if !inference_ids.insert(inference.id.as_str()) {
            diagnostics.push(Diagnostic {
                code: "duplicate_inference_id",
                message: format!("duplicate inference id: {}", inference.id),
            });
        }
        if inference.method.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_inference_method",
                message: format!("inference {} has an empty method", inference.id),
            });
        }
        if inference.premise_claim_ids.is_empty() {
            diagnostics.push(Diagnostic {
                code: "inference_without_premises",
                message: format!("inference {} has no premises", inference.id),
            });
        }

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

    let inferred_conclusions: HashSet<&str> = artifact
        .inferences
        .iter()
        .map(|inference| inference.conclusion_claim_id.as_str())
        .collect();
    for claim in &artifact.claims {
        if claim.state == EpistemicState::Inferred
            && !inferred_conclusions.contains(claim.id.as_str())
        {
            diagnostics.push(Diagnostic {
                code: "inferred_claim_without_inference",
                message: format!("claim {} is inferred but has no inference edge", claim.id),
            });
        }
    }

    ValidationReport { diagnostics }
}
