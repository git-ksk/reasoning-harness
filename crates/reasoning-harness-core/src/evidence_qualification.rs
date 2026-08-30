use std::collections::BTreeSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ApplicabilityScope, Evidence, EvidenceAuthorityPolicy, EvidenceRequirement, FindingStrength,
    HarnessError, Pass, Proposition, ReasoningArtifact, ScopeCoverage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQualificationStatus {
    Qualified,
    Disqualified,
    Unknown,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQualificationFindingKind {
    TemporalMismatch,
    ScopeMismatch,
    AuthorityMismatch,
    Conflict,
    MissingMetadata,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQualificationFindingReason {
    Stale,
    NotYetValid,
    ScopeMismatch,
    ScopeExpansion,
    InsufficientAuthority,
    MissingTemporalMetadata,
    MissingScopeMetadata,
    MissingProvenanceMetadata,
    UnknownProvenanceClass,
    ConflictingQualifiedEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceQualificationAssessment {
    pub proposition: Proposition,
    pub evidence_id: String,
    pub status: EvidenceQualificationStatus,
    #[serde(default)]
    pub reasons: Vec<EvidenceQualificationFindingReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceQualificationFinding {
    pub id: String,
    pub detector: String,
    pub kind: EvidenceQualificationFindingKind,
    pub reason: EvidenceQualificationFindingReason,
    pub strength: FindingStrength,
    pub proposition: Proposition,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceQualificationInspection {
    #[serde(default)]
    pub assessments: Vec<EvidenceQualificationAssessment>,
    #[serde(default)]
    pub findings: Vec<EvidenceQualificationFinding>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EvidenceQualificationInspector;

impl EvidenceQualificationInspector {
    pub fn inspect(&self, artifact: &ReasoningArtifact) -> EvidenceQualificationInspection {
        let mut inspection = EvidenceQualificationInspection::default();

        for (requirement_index, requirement) in artifact.evidence_requirements.iter().enumerate() {
            let candidates = artifact
                .evidence
                .iter()
                .filter(|evidence| evidence.facts.contains_key(&requirement.proposition.key))
                .collect::<Vec<_>>();
            let mut qualified = Vec::new();

            for evidence in candidates {
                let reasons =
                    qualification_reasons(&artifact.authority_policy, requirement, evidence);
                let status = if reasons.iter().any(|reason| is_hard_reason(*reason)) {
                    EvidenceQualificationStatus::Disqualified
                } else if reasons.is_empty() {
                    EvidenceQualificationStatus::Qualified
                } else {
                    EvidenceQualificationStatus::Unknown
                };
                if status == EvidenceQualificationStatus::Qualified {
                    qualified.push(evidence);
                }
                inspection
                    .assessments
                    .push(EvidenceQualificationAssessment {
                        proposition: requirement.proposition.clone(),
                        evidence_id: evidence.id.clone(),
                        status,
                        reasons: reasons.clone(),
                    });
                for reason in reasons {
                    inspection.findings.push(finding_for_reason(
                        requirement_index,
                        requirement,
                        evidence,
                        reason,
                    ));
                }
            }

            let distinct_values = qualified
                .iter()
                .filter_map(|evidence| evidence.facts.get(&requirement.proposition.key))
                .collect::<BTreeSet<_>>();
            if distinct_values.len() > 1 {
                let evidence_ids = qualified
                    .iter()
                    .map(|evidence| evidence.id.clone())
                    .collect::<Vec<_>>();
                inspection.findings.push(EvidenceQualificationFinding {
                    id: format!("evidence_qualification:{requirement_index}:conflict"),
                    detector: "evidence_qualification_inspector".into(),
                    kind: EvidenceQualificationFindingKind::Conflict,
                    reason: EvidenceQualificationFindingReason::ConflictingQualifiedEvidence,
                    strength: FindingStrength::Hard,
                    proposition: requirement.proposition.clone(),
                    evidence_ids,
                    message: "multiple evidence records qualify for the same requirement but contain conflicting structured values"
                        .into(),
                });
            }
        }

        inspection
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EvidenceQualificationPass;

impl Pass for EvidenceQualificationPass {
    fn name(&self) -> &'static str {
        "evidence_qualification"
    }

    fn apply(&self, mut artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError> {
        artifact.evidence_qualification_findings =
            EvidenceQualificationInspector.inspect(&artifact).findings;
        Ok(artifact)
    }
}

pub(crate) fn qualification_reasons(
    authority_policy: &EvidenceAuthorityPolicy,
    requirement: &EvidenceRequirement,
    evidence: &Evidence,
) -> Vec<EvidenceQualificationFindingReason> {
    let mut reasons = BTreeSet::new();

    if let Some(as_of) = requirement.as_of_unix_seconds {
        match &evidence.metadata.temporal {
            Some(validity) => {
                if validity
                    .effective_from_unix_seconds
                    .is_some_and(|from| as_of < from)
                {
                    reasons.insert(EvidenceQualificationFindingReason::NotYetValid);
                }
                if validity
                    .effective_until_unix_seconds
                    .is_some_and(|until| as_of > until)
                {
                    reasons.insert(EvidenceQualificationFindingReason::Stale);
                }
            }
            None => {
                reasons.insert(EvidenceQualificationFindingReason::MissingTemporalMetadata);
            }
        }
    }

    if let Some(required_scope) = &requirement.scope {
        match &evidence.metadata.scope {
            Some(evidence_scope) => {
                reasons.extend(scope_reasons(required_scope, evidence_scope));
            }
            None => {
                reasons.insert(EvidenceQualificationFindingReason::MissingScopeMetadata);
            }
        }
    }

    if let Some(minimum_class) = &requirement.minimum_authority_class {
        match &evidence.metadata.provenance_class {
            None => {
                reasons.insert(EvidenceQualificationFindingReason::MissingProvenanceMetadata);
            }
            Some(class) => {
                let minimum_rank = authority_policy.ranks.get(minimum_class);
                let evidence_rank = authority_policy.ranks.get(class);
                match (minimum_rank, evidence_rank) {
                    (Some(minimum_rank), Some(evidence_rank)) if evidence_rank < minimum_rank => {
                        reasons.insert(EvidenceQualificationFindingReason::InsufficientAuthority);
                    }
                    (Some(_), Some(_)) => {}
                    (_, None) => {
                        reasons.insert(EvidenceQualificationFindingReason::UnknownProvenanceClass);
                    }
                    // Validation rejects a requirement whose minimum class is missing from policy.
                    (None, _) => {}
                }
            }
        }
    }

    reasons.into_iter().collect()
}

fn scope_reasons(
    required: &ApplicabilityScope,
    evidence: &ApplicabilityScope,
) -> BTreeSet<EvidenceQualificationFindingReason> {
    let mut reasons = BTreeSet::new();
    for (dimension, required_coverage) in required {
        let Some(evidence_coverage) = evidence.get(dimension) else {
            reasons.insert(EvidenceQualificationFindingReason::MissingScopeMetadata);
            continue;
        };
        match (evidence_coverage, required_coverage) {
            (ScopeCoverage::Any, _) => {}
            (ScopeCoverage::Values { .. }, ScopeCoverage::Any) => {
                reasons.insert(EvidenceQualificationFindingReason::ScopeExpansion);
            }
            (
                ScopeCoverage::Values {
                    values: evidence_values,
                },
                ScopeCoverage::Values {
                    values: required_values,
                },
            ) => {
                if required_values.is_subset(evidence_values) {
                    continue;
                }
                if required_values.is_disjoint(evidence_values) {
                    reasons.insert(EvidenceQualificationFindingReason::ScopeMismatch);
                } else {
                    reasons.insert(EvidenceQualificationFindingReason::ScopeExpansion);
                }
            }
        }
    }
    reasons
}

fn is_hard_reason(reason: EvidenceQualificationFindingReason) -> bool {
    matches!(
        reason,
        EvidenceQualificationFindingReason::Stale
            | EvidenceQualificationFindingReason::NotYetValid
            | EvidenceQualificationFindingReason::ScopeMismatch
            | EvidenceQualificationFindingReason::ScopeExpansion
            | EvidenceQualificationFindingReason::InsufficientAuthority
            | EvidenceQualificationFindingReason::ConflictingQualifiedEvidence
    )
}

fn finding_for_reason(
    requirement_index: usize,
    requirement: &EvidenceRequirement,
    evidence: &Evidence,
    reason: EvidenceQualificationFindingReason,
) -> EvidenceQualificationFinding {
    let (kind, message) = match reason {
        EvidenceQualificationFindingReason::Stale => (
            EvidenceQualificationFindingKind::TemporalMismatch,
            "evidence validity ended before the required as-of time",
        ),
        EvidenceQualificationFindingReason::NotYetValid => (
            EvidenceQualificationFindingKind::TemporalMismatch,
            "evidence validity begins after the required as-of time",
        ),
        EvidenceQualificationFindingReason::ScopeMismatch => (
            EvidenceQualificationFindingKind::ScopeMismatch,
            "evidence applicability scope does not overlap the required scope",
        ),
        EvidenceQualificationFindingReason::ScopeExpansion => (
            EvidenceQualificationFindingKind::ScopeMismatch,
            "the required scope is broader than the evidence applicability scope",
        ),
        EvidenceQualificationFindingReason::InsufficientAuthority => (
            EvidenceQualificationFindingKind::AuthorityMismatch,
            "evidence provenance authority is below the harness-owned minimum",
        ),
        EvidenceQualificationFindingReason::MissingTemporalMetadata => (
            EvidenceQualificationFindingKind::MissingMetadata,
            "temporal qualification is required but evidence has no temporal metadata",
        ),
        EvidenceQualificationFindingReason::MissingScopeMetadata => (
            EvidenceQualificationFindingKind::MissingMetadata,
            "scope qualification is required but evidence lacks the required scope binding",
        ),
        EvidenceQualificationFindingReason::MissingProvenanceMetadata => (
            EvidenceQualificationFindingKind::MissingMetadata,
            "authority qualification is required but evidence has no provenance class",
        ),
        EvidenceQualificationFindingReason::UnknownProvenanceClass => (
            EvidenceQualificationFindingKind::MissingMetadata,
            "evidence provenance class is not ranked by the harness-owned authority policy",
        ),
        EvidenceQualificationFindingReason::ConflictingQualifiedEvidence => {
            unreachable!("conflict findings are created after per-evidence qualification")
        }
    };
    EvidenceQualificationFinding {
        id: format!(
            "evidence_qualification:{requirement_index}:{}:{reason:?}",
            evidence.id
        )
        .to_lowercase(),
        detector: "evidence_qualification_inspector".into(),
        kind,
        reason,
        strength: if is_hard_reason(reason) {
            FindingStrength::Hard
        } else {
            FindingStrength::Soft
        },
        proposition: requirement.proposition.clone(),
        evidence_ids: vec![evidence.id.clone()],
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::{
        EvidenceAuthorityPolicy, EvidenceMetadata, EvidenceRequirement, HarnessInput,
        TemporalValidity, materialize_candidate,
    };

    use super::*;

    fn values(items: &[&str]) -> ScopeCoverage {
        ScopeCoverage::Values {
            values: items.iter().map(|item| (*item).to_string()).collect(),
        }
    }

    fn artifact() -> ReasoningArtifact {
        materialize_candidate(
            HarnessInput {
                task: "qualify evidence".into(),
                evidence: vec![Evidence {
                    id: "e1".into(),
                    source: "fixture".into(),
                    observation: "enabled".into(),
                    facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
                    metadata: EvidenceMetadata {
                        temporal: Some(TemporalValidity {
                            effective_from_unix_seconds: Some(100),
                            effective_until_unix_seconds: Some(200),
                        }),
                        scope: Some(BTreeMap::from([("region".into(), values(&["r1"]))])),
                        provenance_class: Some("primary".into()),
                    },
                }],
                hypotheses: vec![],
                assumptions: vec![],
                evidence_requirements: vec![EvidenceRequirement {
                    proposition: Proposition {
                        key: "feature.enabled".into(),
                        value: "true".into(),
                    },
                    as_of_unix_seconds: Some(150),
                    scope: Some(BTreeMap::from([("region".into(), values(&["r1"]))])),
                    minimum_authority_class: Some("primary".into()),
                }],
                authority_policy: EvidenceAuthorityPolicy {
                    ranks: BTreeMap::from([("secondary".into(), 10), ("primary".into(), 20)]),
                },
            },
            Default::default(),
        )
    }

    #[test]
    fn exact_metadata_match_is_qualified() {
        let inspection = EvidenceQualificationInspector.inspect(&artifact());
        assert_eq!(inspection.assessments.len(), 1);
        assert_eq!(
            inspection.assessments[0].status,
            EvidenceQualificationStatus::Qualified
        );
        assert!(inspection.findings.is_empty());
    }

    #[test]
    fn narrower_evidence_cannot_support_any_scope_requirement() {
        let mut artifact = artifact();
        artifact.evidence_requirements[0].scope =
            Some(BTreeMap::from([("region".into(), ScopeCoverage::Any)]));
        let inspection = EvidenceQualificationInspector.inspect(&artifact);
        assert_eq!(
            inspection.assessments[0].reasons,
            vec![EvidenceQualificationFindingReason::ScopeExpansion]
        );
        assert_eq!(inspection.findings[0].strength, FindingStrength::Hard);
    }

    #[test]
    fn missing_required_metadata_remains_unknown_and_soft() {
        let mut artifact = artifact();
        artifact.evidence[0].metadata = EvidenceMetadata::default();
        let inspection = EvidenceQualificationInspector.inspect(&artifact);
        assert_eq!(
            inspection.assessments[0].status,
            EvidenceQualificationStatus::Unknown
        );
        assert_eq!(inspection.findings.len(), 3);
        assert!(
            inspection
                .findings
                .iter()
                .all(|finding| finding.strength == FindingStrength::Soft)
        );
    }

    #[test]
    fn qualification_pass_is_observational_even_for_hard_mismatch() {
        let mut artifact = artifact();
        artifact.evidence_requirements[0].as_of_unix_seconds = Some(250);
        artifact.claims.push(crate::Claim {
            id: "c1".into(),
            statement: "feature.enabled = true".into(),
            state: crate::EpistemicState::Supported,
            proposition: Some(Proposition {
                key: "feature.enabled".into(),
                value: "true".into(),
            }),
            evidence_ids: vec!["e1".into()],
        });
        let result = EvidenceQualificationPass.apply(artifact).unwrap();
        assert_eq!(result.claims[0].state, crate::EpistemicState::Supported);
        assert!(
            result
                .evidence_qualification_findings
                .iter()
                .any(|finding| {
                    finding.reason == EvidenceQualificationFindingReason::Stale
                        && finding.strength == FindingStrength::Hard
                })
        );
    }

    #[test]
    fn overlapping_scope_subset_is_supported() {
        let mut artifact = artifact();
        artifact.evidence[0].metadata.scope = Some(BTreeMap::from([(
            "region".into(),
            ScopeCoverage::Values {
                values: BTreeSet::from(["r1".into(), "r2".into()]),
            },
        )]));
        let inspection = EvidenceQualificationInspector.inspect(&artifact);
        assert!(inspection.findings.is_empty());
    }
}
