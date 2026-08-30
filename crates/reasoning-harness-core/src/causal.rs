use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::frameworks::five_whys::is_lexical_restatement;
use crate::{FindingStrength, Proposition, ReasoningArtifact};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CausalRelation {
    pub causes: Vec<Proposition>,
    pub effect: Proposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CausalEvidenceConclusion {
    Supports,
    Refutes,
    AssociationOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CausalEvidence {
    pub id: String,
    pub source: String,
    pub relation: CausalRelation,
    pub conclusion: CausalEvidenceConclusion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CausalSupportStatus {
    Supported,
    Refuted,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CausalFindingKind {
    LexicalRestatement,
    UnsupportedCausalEdge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CausalFindingReason {
    LexicalRestatement,
    MissingPropositionBinding,
    MissingCausalEvidence,
    AssociationOnly,
    PartialSupport,
    DirectionMismatch,
    ExplicitRefutation,
    ConflictingEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CausalEdgeAssessment {
    pub edge_id: String,
    pub inference_id: String,
    pub premise_claim_ids: Vec<String>,
    pub conclusion_claim_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<CausalRelation>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    pub status: CausalSupportStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CausalFinding {
    pub id: String,
    pub detector: String,
    pub kind: CausalFindingKind,
    pub reason: CausalFindingReason,
    pub strength: FindingStrength,
    pub edge_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relation: Option<CausalRelation>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CausalInspection {
    #[serde(default)]
    pub assessments: Vec<CausalEdgeAssessment>,
    #[serde(default)]
    pub findings: Vec<CausalFinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CausalInputError {
    #[error("causal evidence id must not be empty")]
    EmptyEvidenceId,
    #[error("duplicate causal evidence id: {id}")]
    DuplicateEvidenceId { id: String },
    #[error("causal evidence {id} has an empty source")]
    EmptyEvidenceSource { id: String },
    #[error("causal evidence {id} relation has no causes")]
    EmptyCauseSet { id: String },
    #[error("causal evidence {id} has an empty {role} proposition key/value")]
    InvalidProposition { id: String, role: &'static str },
    #[error("causal evidence {id} repeats cause proposition {key}={value}")]
    DuplicateCause {
        id: String,
        key: String,
        value: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct CausalInspector {
    evidence: Vec<CausalEvidence>,
}

impl CausalInspector {
    pub fn new(evidence: Vec<CausalEvidence>) -> Result<Self, CausalInputError> {
        validate_causal_evidence(&evidence)?;
        Ok(Self { evidence })
    }

    pub fn inspect(&self, artifact: &ReasoningArtifact) -> CausalInspection {
        let mut inspection = CausalInspection::default();

        for inference in artifact
            .inferences
            .iter()
            .filter(|inference| matches!(inference.method.as_str(), "five_whys" | "causal_forward"))
        {
            if inference.method == "five_whys" {
                for premise_id in &inference.premise_claim_ids {
                    let edge_id = format!("{}:{}", inference.id, premise_id);
                    let premise = artifact.claims.iter().find(|claim| claim.id == *premise_id);
                    let conclusion = artifact
                        .claims
                        .iter()
                        .find(|claim| claim.id == inference.conclusion_claim_id);

                    if let (Some(premise), Some(conclusion)) = (premise, conclusion) {
                        if is_lexical_restatement(&premise.statement, &conclusion.statement) {
                            inspection.findings.push(CausalFinding {
                                id: format!("causal:{edge_id}:lexical"),
                                detector: "causal_inspector".into(),
                                kind: CausalFindingKind::LexicalRestatement,
                                reason: CausalFindingReason::LexicalRestatement,
                                strength: FindingStrength::Hard,
                                edge_id: edge_id.clone(),
                                relation: None,
                                evidence_ids: vec![],
                                message: "Five Whys edge lexically restates the effect".into(),
                            });
                        }
                    }

                    let relation = match (premise, conclusion) {
                        (Some(effect), Some(cause)) => {
                            match (&cause.proposition, &effect.proposition) {
                                (Some(cause), Some(effect)) => Some(CausalRelation {
                                    causes: vec![cause.clone()],
                                    effect: effect.clone(),
                                }),
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    self.assess_edge(
                        &mut inspection,
                        edge_id,
                        inference.id.clone(),
                        vec![premise_id.clone()],
                        inference.conclusion_claim_id.clone(),
                        relation,
                    );
                }
            } else {
                let premise_claims = inference
                    .premise_claim_ids
                    .iter()
                    .filter_map(|id| artifact.claims.iter().find(|claim| claim.id == *id))
                    .collect::<Vec<_>>();
                let conclusion = artifact
                    .claims
                    .iter()
                    .find(|claim| claim.id == inference.conclusion_claim_id);
                let relation = conclusion.and_then(|effect| {
                    let effect = effect.proposition.clone()?;
                    let causes = premise_claims
                        .iter()
                        .map(|claim| claim.proposition.clone())
                        .collect::<Option<Vec<_>>>()?;
                    (!causes.is_empty()).then_some(CausalRelation { causes, effect })
                });
                self.assess_edge(
                    &mut inspection,
                    inference.id.clone(),
                    inference.id.clone(),
                    inference.premise_claim_ids.clone(),
                    inference.conclusion_claim_id.clone(),
                    relation,
                );
            }
        }

        inspection
    }

    fn assess_edge(
        &self,
        inspection: &mut CausalInspection,
        edge_id: String,
        inference_id: String,
        premise_claim_ids: Vec<String>,
        conclusion_claim_id: String,
        relation: Option<CausalRelation>,
    ) {
        let Some(relation) = relation else {
            inspection.assessments.push(CausalEdgeAssessment {
                edge_id: edge_id.clone(),
                inference_id,
                premise_claim_ids,
                conclusion_claim_id,
                relation: None,
                evidence_ids: vec![],
                status: CausalSupportStatus::Unknown,
            });
            inspection.findings.push(CausalFinding {
                id: format!("causal:{edge_id}:binding"),
                detector: "causal_inspector".into(),
                kind: CausalFindingKind::UnsupportedCausalEdge,
                reason: CausalFindingReason::MissingPropositionBinding,
                strength: FindingStrength::Soft,
                edge_id,
                relation: None,
                evidence_ids: vec![],
                message: "causal edge cannot be bound to scoped propositions".into(),
            });
            return;
        };

        let exact = self
            .evidence
            .iter()
            .filter(|evidence| same_relation(&evidence.relation, &relation))
            .collect::<Vec<_>>();
        let supporting = exact
            .iter()
            .filter(|evidence| evidence.conclusion == CausalEvidenceConclusion::Supports)
            .copied()
            .collect::<Vec<_>>();
        let refuting = exact
            .iter()
            .filter(|evidence| evidence.conclusion == CausalEvidenceConclusion::Refutes)
            .copied()
            .collect::<Vec<_>>();
        let associations = exact
            .iter()
            .filter(|evidence| evidence.conclusion == CausalEvidenceConclusion::AssociationOnly)
            .copied()
            .collect::<Vec<_>>();

        let mut relevant_ids = exact
            .iter()
            .map(|evidence| evidence.id.clone())
            .collect::<Vec<_>>();
        let (status, finding) = if !supporting.is_empty() && !refuting.is_empty() {
            (
                CausalSupportStatus::Unknown,
                Some((
                    CausalFindingReason::ConflictingEvidence,
                    FindingStrength::Soft,
                    "trusted causal evidence conflicts; support remains unknown",
                )),
            )
        } else if !refuting.is_empty() {
            (
                CausalSupportStatus::Refuted,
                Some((
                    CausalFindingReason::ExplicitRefutation,
                    FindingStrength::Hard,
                    "trusted causal evidence explicitly refutes the proposed relation",
                )),
            )
        } else if !supporting.is_empty() {
            (CausalSupportStatus::Supported, None)
        } else if !associations.is_empty() {
            (
                CausalSupportStatus::Unknown,
                Some((
                    CausalFindingReason::AssociationOnly,
                    FindingStrength::Soft,
                    "available evidence establishes association only, not causation",
                )),
            )
        } else if let Some(partial) = self.partial_support(&relation) {
            relevant_ids = partial.iter().map(|evidence| evidence.id.clone()).collect();
            (
                CausalSupportStatus::Unknown,
                Some((
                    CausalFindingReason::PartialSupport,
                    FindingStrength::Soft,
                    "evidence supports only part of the complete causal relation",
                )),
            )
        } else if let Some(reverse) = self.reverse_support(&relation) {
            relevant_ids = reverse.iter().map(|evidence| evidence.id.clone()).collect();
            (
                CausalSupportStatus::Unknown,
                Some((
                    CausalFindingReason::DirectionMismatch,
                    FindingStrength::Soft,
                    "evidence supports the reverse direction but does not refute this direction",
                )),
            )
        } else {
            (
                CausalSupportStatus::Unknown,
                Some((
                    CausalFindingReason::MissingCausalEvidence,
                    FindingStrength::Soft,
                    "no harness-owned evidence supports or refutes the exact scoped causal relation",
                )),
            )
        };

        inspection.assessments.push(CausalEdgeAssessment {
            edge_id: edge_id.clone(),
            inference_id,
            premise_claim_ids,
            conclusion_claim_id,
            relation: Some(relation.clone()),
            evidence_ids: relevant_ids.clone(),
            status,
        });

        if let Some((reason, strength, message)) = finding {
            inspection.findings.push(CausalFinding {
                id: format!("causal:{edge_id}:{reason:?}"),
                detector: "causal_inspector".into(),
                kind: CausalFindingKind::UnsupportedCausalEdge,
                reason,
                strength,
                edge_id,
                relation: Some(relation),
                evidence_ids: relevant_ids,
                message: message.into(),
            });
        }
    }

    fn partial_support(&self, relation: &CausalRelation) -> Option<Vec<&CausalEvidence>> {
        if relation.causes.len() < 2 {
            return None;
        }
        let partial = self
            .evidence
            .iter()
            .filter(|evidence| evidence.conclusion == CausalEvidenceConclusion::Supports)
            .filter(|evidence| evidence.relation.effect == relation.effect)
            .filter(|evidence| {
                !evidence.relation.causes.is_empty()
                    && evidence.relation.causes.len() < relation.causes.len()
                    && evidence
                        .relation
                        .causes
                        .iter()
                        .all(|cause| relation.causes.contains(cause))
            })
            .collect::<Vec<_>>();
        (!partial.is_empty()).then_some(partial)
    }

    fn reverse_support(&self, relation: &CausalRelation) -> Option<Vec<&CausalEvidence>> {
        if relation.causes.len() != 1 {
            return None;
        }
        let reverse = CausalRelation {
            causes: vec![relation.effect.clone()],
            effect: relation.causes[0].clone(),
        };
        let reverse = self
            .evidence
            .iter()
            .filter(|evidence| evidence.conclusion == CausalEvidenceConclusion::Supports)
            .filter(|evidence| same_relation(&evidence.relation, &reverse))
            .collect::<Vec<_>>();
        (!reverse.is_empty()).then_some(reverse)
    }
}

fn validate_causal_evidence(evidence: &[CausalEvidence]) -> Result<(), CausalInputError> {
    let mut ids = HashSet::new();
    for record in evidence {
        if record.id.trim().is_empty() {
            return Err(CausalInputError::EmptyEvidenceId);
        }
        if !ids.insert(record.id.as_str()) {
            return Err(CausalInputError::DuplicateEvidenceId {
                id: record.id.clone(),
            });
        }
        if record.source.trim().is_empty() {
            return Err(CausalInputError::EmptyEvidenceSource {
                id: record.id.clone(),
            });
        }
        if record.relation.causes.is_empty() {
            return Err(CausalInputError::EmptyCauseSet {
                id: record.id.clone(),
            });
        }
        if proposition_is_empty(&record.relation.effect) {
            return Err(CausalInputError::InvalidProposition {
                id: record.id.clone(),
                role: "effect",
            });
        }

        let mut causes = HashSet::new();
        for cause in &record.relation.causes {
            if proposition_is_empty(cause) {
                return Err(CausalInputError::InvalidProposition {
                    id: record.id.clone(),
                    role: "cause",
                });
            }
            if !causes.insert((cause.key.as_str(), cause.value.as_str())) {
                return Err(CausalInputError::DuplicateCause {
                    id: record.id.clone(),
                    key: cause.key.clone(),
                    value: cause.value.clone(),
                });
            }
        }
    }
    Ok(())
}

fn proposition_is_empty(proposition: &Proposition) -> bool {
    proposition.key.trim().is_empty() || proposition.value.trim().is_empty()
}

fn same_relation(left: &CausalRelation, right: &CausalRelation) -> bool {
    if left.effect != right.effect {
        return false;
    }
    normalized_causes(&left.causes) == normalized_causes(&right.causes)
}

fn normalized_causes(causes: &[Proposition]) -> Vec<Proposition> {
    let mut causes = causes.to_vec();
    causes.sort_by(|a, b| (&a.key, &a.value).cmp(&(&b.key, &b.value)));
    causes.dedup();
    causes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Claim, EpistemicState, Inference};

    fn proposition(key: &str, value: &str) -> Proposition {
        Proposition {
            key: key.into(),
            value: value.into(),
        }
    }

    fn five_whys_artifact() -> ReasoningArtifact {
        let mut artifact = ReasoningArtifact::default();
        artifact.claims = vec![
            Claim {
                id: "effect".into(),
                statement: "Requests timed out".into(),
                state: EpistemicState::Assumed,
                proposition: Some(proposition("incident.requests_timed_out", "true")),
                evidence_ids: vec![],
            },
            Claim {
                id: "cause".into(),
                statement: "Database lock blocked progress".into(),
                state: EpistemicState::Assumed,
                proposition: Some(proposition("incident.db_lock_blocked", "true")),
                evidence_ids: vec![],
            },
        ];
        artifact.inferences = vec![Inference {
            id: "why-1".into(),
            premise_claim_ids: vec!["effect".into()],
            conclusion_claim_id: "cause".into(),
            method: "five_whys".into(),
        }];
        artifact
    }

    fn proposed_relation() -> CausalRelation {
        CausalRelation {
            causes: vec![proposition("incident.db_lock_blocked", "true")],
            effect: proposition("incident.requests_timed_out", "true"),
        }
    }

    #[test]
    fn malformed_harness_owned_causal_evidence_is_rejected() {
        let evidence = CausalEvidence {
            id: "ce1".into(),
            source: "".into(),
            relation: proposed_relation(),
            conclusion: CausalEvidenceConclusion::Supports,
        };

        assert_eq!(
            CausalInspector::new(vec![evidence]).unwrap_err(),
            CausalInputError::EmptyEvidenceSource { id: "ce1".into() }
        );
    }

    #[test]
    fn exact_harness_owned_support_marks_edge_supported() {
        let evidence = CausalEvidence {
            id: "ce1".into(),
            source: "fixture-oracle".into(),
            relation: proposed_relation(),
            conclusion: CausalEvidenceConclusion::Supports,
        };
        let inspection = CausalInspector::new(vec![evidence])
            .unwrap()
            .inspect(&five_whys_artifact());

        assert_eq!(inspection.assessments.len(), 1);
        assert_eq!(
            inspection.assessments[0].status,
            CausalSupportStatus::Supported
        );
        assert!(inspection.findings.is_empty());
    }

    #[test]
    fn exact_relation_support_ignores_cause_order() {
        let mut artifact = ReasoningArtifact::default();
        artifact.claims = vec![
            Claim {
                id: "cause-a".into(),
                statement: "Database lock persisted".into(),
                state: EpistemicState::Assumed,
                proposition: Some(proposition("deploy.db_lock", "true")),
                evidence_ids: vec![],
            },
            Claim {
                id: "cause-b".into(),
                statement: "Retry budget was exhausted".into(),
                state: EpistemicState::Assumed,
                proposition: Some(proposition("deploy.retry_exhausted", "true")),
                evidence_ids: vec![],
            },
            Claim {
                id: "effect".into(),
                statement: "Deployment failed".into(),
                state: EpistemicState::Assumed,
                proposition: Some(proposition("deploy.failed", "true")),
                evidence_ids: vec![],
            },
        ];
        artifact.inferences = vec![Inference {
            id: "cause-edge".into(),
            premise_claim_ids: vec!["cause-a".into(), "cause-b".into()],
            conclusion_claim_id: "effect".into(),
            method: "causal_forward".into(),
        }];
        let evidence = CausalEvidence {
            id: "ce1".into(),
            source: "fixture-oracle".into(),
            relation: CausalRelation {
                causes: vec![
                    proposition("deploy.retry_exhausted", "true"),
                    proposition("deploy.db_lock", "true"),
                ],
                effect: proposition("deploy.failed", "true"),
            },
            conclusion: CausalEvidenceConclusion::Supports,
        };

        let inspection = CausalInspector::new(vec![evidence])
            .unwrap()
            .inspect(&artifact);

        assert_eq!(
            inspection.assessments[0].status,
            CausalSupportStatus::Supported
        );
        assert!(inspection.findings.is_empty());
    }

    #[test]
    fn endpoint_truth_without_relation_evidence_remains_unknown() {
        let inspection = CausalInspector::default().inspect(&five_whys_artifact());

        assert_eq!(
            inspection.assessments[0].status,
            CausalSupportStatus::Unknown
        );
        assert_eq!(
            inspection.findings[0].reason,
            CausalFindingReason::MissingCausalEvidence
        );
        assert_eq!(inspection.findings[0].strength, FindingStrength::Soft);
    }

    #[test]
    fn association_only_is_soft_and_unknown() {
        let evidence = CausalEvidence {
            id: "ce1".into(),
            source: "observational-study".into(),
            relation: proposed_relation(),
            conclusion: CausalEvidenceConclusion::AssociationOnly,
        };
        let inspection = CausalInspector::new(vec![evidence])
            .unwrap()
            .inspect(&five_whys_artifact());

        assert_eq!(
            inspection.assessments[0].status,
            CausalSupportStatus::Unknown
        );
        assert_eq!(
            inspection.findings[0].reason,
            CausalFindingReason::AssociationOnly
        );
        assert_eq!(inspection.findings[0].strength, FindingStrength::Soft);
    }

    #[test]
    fn exact_refutation_is_hard() {
        let evidence = CausalEvidence {
            id: "ce1".into(),
            source: "fixture-oracle".into(),
            relation: proposed_relation(),
            conclusion: CausalEvidenceConclusion::Refutes,
        };
        let inspection = CausalInspector::new(vec![evidence])
            .unwrap()
            .inspect(&five_whys_artifact());

        assert_eq!(
            inspection.assessments[0].status,
            CausalSupportStatus::Refuted
        );
        assert_eq!(
            inspection.findings[0].reason,
            CausalFindingReason::ExplicitRefutation
        );
        assert_eq!(inspection.findings[0].strength, FindingStrength::Hard);
    }

    #[test]
    fn conflicting_records_remain_unknown() {
        let evidence = vec![
            CausalEvidence {
                id: "support".into(),
                source: "oracle-a".into(),
                relation: proposed_relation(),
                conclusion: CausalEvidenceConclusion::Supports,
            },
            CausalEvidence {
                id: "refute".into(),
                source: "oracle-b".into(),
                relation: proposed_relation(),
                conclusion: CausalEvidenceConclusion::Refutes,
            },
        ];
        let inspection = CausalInspector::new(evidence)
            .unwrap()
            .inspect(&five_whys_artifact());

        assert_eq!(
            inspection.assessments[0].status,
            CausalSupportStatus::Unknown
        );
        assert_eq!(
            inspection.findings[0].reason,
            CausalFindingReason::ConflictingEvidence
        );
    }

    #[test]
    fn reverse_support_does_not_refute_forward_relation() {
        let proposed = proposed_relation();
        let reverse = CausalRelation {
            causes: vec![proposed.effect.clone()],
            effect: proposed.causes[0].clone(),
        };
        let evidence = CausalEvidence {
            id: "reverse".into(),
            source: "fixture-oracle".into(),
            relation: reverse,
            conclusion: CausalEvidenceConclusion::Supports,
        };
        let inspection = CausalInspector::new(vec![evidence])
            .unwrap()
            .inspect(&five_whys_artifact());

        assert_eq!(
            inspection.assessments[0].status,
            CausalSupportStatus::Unknown
        );
        assert_eq!(
            inspection.findings[0].reason,
            CausalFindingReason::DirectionMismatch
        );
        assert_eq!(inspection.findings[0].strength, FindingStrength::Soft);
    }

    #[test]
    fn missing_binding_is_explicit_and_soft() {
        let mut artifact = five_whys_artifact();
        artifact.claims[1].proposition = None;
        let inspection = CausalInspector::default().inspect(&artifact);

        assert_eq!(
            inspection.assessments[0].status,
            CausalSupportStatus::Unknown
        );
        assert_eq!(
            inspection.findings[0].reason,
            CausalFindingReason::MissingPropositionBinding
        );
        assert_eq!(inspection.findings[0].strength, FindingStrength::Soft);
    }
}
