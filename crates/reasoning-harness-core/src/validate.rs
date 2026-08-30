use std::collections::HashSet;

use serde::Serialize;

use crate::{ApplicabilityScope, EpistemicState, ReasoningArtifact, ScopeCoverage};

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

fn validate_scope(scope: &ApplicabilityScope, owner: &str, diagnostics: &mut Vec<Diagnostic>) {
    for (dimension, coverage) in scope {
        if dimension.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_scope_dimension",
                message: format!("{owner} has an empty scope dimension"),
            });
        }
        if let ScopeCoverage::Values { values } = coverage {
            if values.is_empty() {
                diagnostics.push(Diagnostic {
                    code: "empty_scope_values",
                    message: format!("{owner} scope dimension {dimension} has no values"),
                });
            }
            if values.iter().any(|value| value.trim().is_empty()) {
                diagnostics.push(Diagnostic {
                    code: "empty_scope_value",
                    message: format!("{owner} scope dimension {dimension} has an empty value"),
                });
            }
        }
    }
}

pub fn validate_artifact(artifact: &ReasoningArtifact) -> ValidationReport {
    let mut diagnostics = Vec::new();
    let mut evidence_ids = HashSet::new();
    let mut claim_ids = HashSet::new();
    let mut inference_ids = HashSet::new();

    if artifact.task.trim().is_empty() {
        diagnostics.push(Diagnostic {
            code: "empty_task",
            message: "reasoning artifact task must not be empty".into(),
        });
    }

    let mut hypotheses = HashSet::new();
    for hypothesis in &artifact.hypotheses {
        if hypothesis.key.trim().is_empty() || hypothesis.value.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "invalid_hypothesis",
                message: "harness-owned hypothesis key/value must not be empty".into(),
            });
        }
        if !hypotheses.insert((hypothesis.key.as_str(), hypothesis.value.as_str())) {
            diagnostics.push(Diagnostic {
                code: "duplicate_hypothesis",
                message: format!(
                    "duplicate harness-owned hypothesis: {}={}",
                    hypothesis.key, hypothesis.value
                ),
            });
        }
    }

    let mut assumptions = HashSet::new();
    for assumption in &artifact.assumptions {
        if assumption.key.trim().is_empty() || assumption.value.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "invalid_input_assumption",
                message: "harness-owned assumption key/value must not be empty".into(),
            });
        }
        if !assumptions.insert((assumption.key.as_str(), assumption.value.as_str())) {
            diagnostics.push(Diagnostic {
                code: "duplicate_input_assumption",
                message: format!(
                    "duplicate harness-owned assumption: {}={}",
                    assumption.key, assumption.value
                ),
            });
        }
    }

    for class in artifact.authority_policy.ranks.keys() {
        if class.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_authority_class",
                message: "authority policy class must not be empty".into(),
            });
        }
    }

    let mut evidence_requirements = HashSet::new();
    for requirement in &artifact.evidence_requirements {
        if requirement.proposition.key.trim().is_empty()
            || requirement.proposition.value.trim().is_empty()
        {
            diagnostics.push(Diagnostic {
                code: "invalid_evidence_requirement_proposition",
                message: "evidence requirement proposition key/value must not be empty".into(),
            });
        }
        if !evidence_requirements.insert(requirement.proposition.key.as_str()) {
            diagnostics.push(Diagnostic {
                code: "duplicate_evidence_requirement_key",
                message: format!(
                    "multiple evidence requirements target proposition key {}",
                    requirement.proposition.key
                ),
            });
        }
        if let Some(scope) = &requirement.scope {
            validate_scope(scope, "evidence requirement", &mut diagnostics);
        }
        if let Some(class) = &requirement.minimum_authority_class {
            if class.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "empty_minimum_authority_class",
                    message: "evidence requirement minimum authority class must not be empty"
                        .into(),
                });
            } else if !artifact.authority_policy.ranks.contains_key(class) {
                diagnostics.push(Diagnostic {
                    code: "unknown_minimum_authority_class",
                    message: format!(
                        "evidence requirement references authority class not present in policy: {class}"
                    ),
                });
            }
        }
    }

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
        if let Some(temporal) = &evidence.metadata.temporal {
            if matches!(
                (
                    temporal.effective_from_unix_seconds,
                    temporal.effective_until_unix_seconds
                ),
                (Some(from), Some(until)) if from > until
            ) {
                diagnostics.push(Diagnostic {
                    code: "invalid_evidence_temporal_window",
                    message: format!(
                        "evidence {} effective_from is after effective_until",
                        evidence.id
                    ),
                });
            }
        }
        if let Some(scope) = &evidence.metadata.scope {
            validate_scope(
                scope,
                &format!("evidence {}", evidence.id),
                &mut diagnostics,
            );
        }
        if evidence
            .metadata
            .provenance_class
            .as_ref()
            .is_some_and(|class| class.trim().is_empty())
        {
            diagnostics.push(Diagnostic {
                code: "empty_evidence_provenance_class",
                message: format!("evidence {} has an empty provenance class", evidence.id),
            });
        }
        for (key, value) in &evidence.facts {
            if key.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "empty_evidence_fact_key",
                    message: format!("evidence {} has an empty structured fact key", evidence.id),
                });
            }
            if value.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "empty_evidence_fact_value",
                    message: format!(
                        "evidence {} structured fact {} has an empty value",
                        evidence.id, key
                    ),
                });
            }
        }
    }

    let mut receipt_ids = HashSet::new();
    for receipt in &artifact.verification_receipts {
        if receipt.id.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_verification_receipt_id",
                message: "verification receipt id must not be empty".into(),
            });
        }
        if !receipt_ids.insert(receipt.id.as_str()) {
            diagnostics.push(Diagnostic {
                code: "duplicate_verification_receipt_id",
                message: format!("duplicate verification receipt id: {}", receipt.id),
            });
        }
        if receipt.verifier.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_verifier",
                message: format!("verification receipt {} has an empty verifier", receipt.id),
            });
        }
        if receipt.evidence_ids.is_empty() {
            diagnostics.push(Diagnostic {
                code: "verification_without_evidence",
                message: format!("verification receipt {} has no evidence", receipt.id),
            });
        }
        if receipt
            .claim_statement
            .as_ref()
            .is_some_and(|statement| statement.trim().is_empty())
        {
            diagnostics.push(Diagnostic {
                code: "empty_verification_claim_statement",
                message: format!(
                    "verification receipt {} has an empty claim statement",
                    receipt.id
                ),
            });
        }
        if receipt.claim_statement.is_none() && receipt.proposition.is_none() {
            diagnostics.push(Diagnostic {
                code: "verification_without_binding",
                message: format!(
                    "verification receipt {} has neither statement nor proposition binding",
                    receipt.id
                ),
            });
        }
        for evidence_id in &receipt.evidence_ids {
            if !evidence_ids.contains(evidence_id.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "verification_missing_evidence_reference",
                    message: format!(
                        "verification receipt {} references missing evidence {}",
                        receipt.id, evidence_id
                    ),
                });
            }
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
        if let Some(proposition) = &claim.proposition {
            if proposition.key.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "empty_proposition_key",
                    message: format!("claim {} has an empty proposition key", claim.id),
                });
            }
            if proposition.value.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "empty_proposition_value",
                    message: format!("claim {} has an empty proposition value", claim.id),
                });
            }
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

    for receipt in &artifact.verification_receipts {
        let matching_claims = artifact
            .claims
            .iter()
            .filter(|claim| {
                receipt
                    .claim_statement
                    .as_ref()
                    .is_none_or(|statement| statement == &claim.statement)
                    && receipt
                        .proposition
                        .as_ref()
                        .is_none_or(|proposition| claim.proposition.as_ref() == Some(proposition))
                    && receipt
                        .claim_id
                        .as_ref()
                        .is_none_or(|claim_id| claim_id == &claim.id)
            })
            .count();
        if matching_claims != 1 {
            diagnostics.push(Diagnostic {
                code: "verification_claim_binding_invalid",
                message: format!(
                    "verification receipt {} matched {} claims; expected exactly one",
                    receipt.id, matching_claims
                ),
            });
        }
    }

    for finding in &artifact.adversarial_findings {
        if finding.id.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_adversarial_finding_id",
                message: "adversarial finding id must not be empty".into(),
            });
        }
        if finding.detector.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_adversarial_detector",
                message: format!("adversarial finding {} has an empty detector", finding.id),
            });
        }
        if finding.proposition.key.trim().is_empty() || finding.proposition.value.trim().is_empty()
        {
            diagnostics.push(Diagnostic {
                code: "invalid_adversarial_proposition",
                message: format!(
                    "adversarial finding {} has an empty proposition key/value",
                    finding.id
                ),
            });
        }
        if !claim_ids.contains(finding.claim_id.as_str()) {
            diagnostics.push(Diagnostic {
                code: "adversarial_missing_claim",
                message: format!(
                    "adversarial finding {} references missing claim {}",
                    finding.id, finding.claim_id
                ),
            });
        }
        if finding.strength == crate::FindingStrength::Hard && finding.evidence_ids.is_empty() {
            diagnostics.push(Diagnostic {
                code: "hard_adversarial_without_evidence",
                message: format!("hard adversarial finding {} has no evidence", finding.id),
            });
        }
        for evidence_id in &finding.evidence_ids {
            if !evidence_ids.contains(evidence_id.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "adversarial_missing_evidence",
                    message: format!(
                        "adversarial finding {} references missing evidence {}",
                        finding.id, evidence_id
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

    let mut assumption_finding_ids = HashSet::new();
    for finding in &artifact.assumption_findings {
        if finding.id.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_assumption_finding_id",
                message: "assumption finding id must not be empty".into(),
            });
        }
        if !assumption_finding_ids.insert(finding.id.as_str()) {
            diagnostics.push(Diagnostic {
                code: "duplicate_assumption_finding_id",
                message: format!("duplicate assumption finding id: {}", finding.id),
            });
        }
        if finding.detector.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_assumption_detector",
                message: format!("assumption finding {} has an empty detector", finding.id),
            });
        }
        if let Some(proposition) = &finding.proposition {
            if proposition.key.trim().is_empty() || proposition.value.trim().is_empty() {
                diagnostics.push(Diagnostic {
                    code: "invalid_assumption_proposition",
                    message: format!(
                        "assumption finding {} has an empty proposition key/value",
                        finding.id
                    ),
                });
            }
        }
        if finding.strength == crate::FindingStrength::Hard && finding.proposition.is_none() {
            diagnostics.push(Diagnostic {
                code: "hard_assumption_without_binding",
                message: format!(
                    "hard assumption finding {} has no proposition binding",
                    finding.id
                ),
            });
        }
        if finding.claim_ids.is_empty() || finding.inference_ids.is_empty() {
            diagnostics.push(Diagnostic {
                code: "assumption_finding_without_usage",
                message: format!(
                    "assumption finding {} must reference at least one claim and inference",
                    finding.id
                ),
            });
        }
        for claim_id in &finding.claim_ids {
            if !claim_ids.contains(claim_id.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "assumption_missing_claim",
                    message: format!(
                        "assumption finding {} references missing claim {}",
                        finding.id, claim_id
                    ),
                });
            }
        }
        for inference_id in &finding.inference_ids {
            if !inference_ids.contains(inference_id.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "assumption_missing_inference",
                    message: format!(
                        "assumption finding {} references missing inference {}",
                        finding.id, inference_id
                    ),
                });
            }
        }
    }

    let mut evidence_qualification_finding_ids = HashSet::new();
    for finding in &artifact.evidence_qualification_findings {
        if finding.id.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_evidence_qualification_finding_id",
                message: "evidence qualification finding id must not be empty".into(),
            });
        }
        if !evidence_qualification_finding_ids.insert(finding.id.as_str()) {
            diagnostics.push(Diagnostic {
                code: "duplicate_evidence_qualification_finding_id",
                message: format!(
                    "duplicate evidence qualification finding id: {}",
                    finding.id
                ),
            });
        }
        if finding.detector.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: "empty_evidence_qualification_detector",
                message: format!(
                    "evidence qualification finding {} has an empty detector",
                    finding.id
                ),
            });
        }
        if finding.proposition.key.trim().is_empty() || finding.proposition.value.trim().is_empty()
        {
            diagnostics.push(Diagnostic {
                code: "invalid_evidence_qualification_proposition",
                message: format!(
                    "evidence qualification finding {} has an empty proposition key/value",
                    finding.id
                ),
            });
        }
        if finding.evidence_ids.is_empty() {
            diagnostics.push(Diagnostic {
                code: "evidence_qualification_without_evidence",
                message: format!(
                    "evidence qualification finding {} has no evidence references",
                    finding.id
                ),
            });
        }
        for evidence_id in &finding.evidence_ids {
            if !evidence_ids.contains(evidence_id.as_str()) {
                diagnostics.push(Diagnostic {
                    code: "evidence_qualification_missing_evidence",
                    message: format!(
                        "evidence qualification finding {} references missing evidence {}",
                        finding.id, evidence_id
                    ),
                });
            }
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
