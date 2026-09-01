use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use crate::{EpistemicState, Proposition, ReasoningArtifact, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FinalClaimMode {
    Grounded,
    Uncertain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FinalAnswerClaim {
    pub proposition: Proposition,
    pub mode: FinalClaimMode,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FinalAnswerCandidate {
    pub text: String,
    #[serde(default)]
    pub factual_claims: Vec<FinalAnswerClaim>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalizationStatus {
    GroundedAnswer,
    QualifiedPartialAnswer,
    Unresolved,
    Abstain,
    RequiresVerification,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalizationResult {
    pub status: FinalizationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub factual_claims: usize,
    pub covered_claims: usize,
    pub factual_claim_coverage: f64,
    #[serde(default)]
    pub uncovered_propositions: Vec<Proposition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinalizationPolicy {
    pub allow_qualified_partial: bool,
}

impl Default for FinalizationPolicy {
    fn default() -> Self {
        Self {
            allow_qualified_partial: true,
        }
    }
}

pub fn final_answer_candidate_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(FinalAnswerCandidate))
        .expect("FinalAnswerCandidate schema must serialize")
}

pub trait FinalAnswerRenderer: Send + Sync {
    fn render(&self, artifact: &ReasoningArtifact, verdict: Verdict) -> FinalAnswerCandidate;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CanonicalFinalAnswerRenderer;

impl FinalAnswerRenderer for CanonicalFinalAnswerRenderer {
    fn render(&self, artifact: &ReasoningArtifact, verdict: Verdict) -> FinalAnswerCandidate {
        if verdict == Verdict::Reject {
            return FinalAnswerCandidate {
                text: "abstain: verified reasoning contains a contradiction".into(),
                factual_claims: vec![],
            };
        }

        let factual_claims = artifact
            .claims
            .iter()
            .filter_map(|claim| {
                let proposition = claim.proposition.clone()?;
                let mode = match claim.state {
                    EpistemicState::Known | EpistemicState::Supported => FinalClaimMode::Grounded,
                    EpistemicState::Inferred
                    | EpistemicState::Assumed
                    | EpistemicState::Unknown => FinalClaimMode::Uncertain,
                    EpistemicState::Contradicted => return None,
                };
                Some(FinalAnswerClaim { proposition, mode })
            })
            .collect::<Vec<_>>();

        let text = if factual_claims.is_empty() {
            "unresolved: no grounded factual proposition is available".into()
        } else {
            factual_claims
                .iter()
                .map(|claim| match claim.mode {
                    FinalClaimMode::Grounded => {
                        format!("{} = {}", claim.proposition.key, claim.proposition.value)
                    }
                    FinalClaimMode::Uncertain => format!(
                        "uncertain({} = {})",
                        claim.proposition.key, claim.proposition.value
                    ),
                })
                .collect::<Vec<_>>()
                .join("; ")
        };

        FinalAnswerCandidate {
            text,
            factual_claims,
        }
    }
}

pub fn finalize_answer(
    artifact: &ReasoningArtifact,
    verdict: Verdict,
    candidate: FinalAnswerCandidate,
    policy: FinalizationPolicy,
) -> FinalizationResult {
    if verdict == Verdict::Reject {
        return FinalizationResult {
            status: FinalizationStatus::Abstain,
            text: None,
            factual_claims: candidate.factual_claims.len(),
            covered_claims: 0,
            factual_claim_coverage: coverage(0, candidate.factual_claims.len()),
            uncovered_propositions: candidate
                .factual_claims
                .into_iter()
                .map(|claim| claim.proposition)
                .collect(),
        };
    }

    let mut covered_claims = 0usize;
    let mut uncovered_propositions = Vec::new();
    let mut has_uncertain = false;

    for final_claim in &candidate.factual_claims {
        let covered = artifact.claims.iter().any(|claim| {
            if claim.proposition.as_ref() != Some(&final_claim.proposition) {
                return false;
            }
            match final_claim.mode {
                FinalClaimMode::Grounded => matches!(
                    claim.state,
                    EpistemicState::Known | EpistemicState::Supported
                ),
                FinalClaimMode::Uncertain => matches!(
                    claim.state,
                    EpistemicState::Known
                        | EpistemicState::Supported
                        | EpistemicState::Inferred
                        | EpistemicState::Assumed
                        | EpistemicState::Unknown
                ),
            }
        });
        if covered {
            covered_claims += 1;
            has_uncertain |= final_claim.mode == FinalClaimMode::Uncertain;
        } else {
            uncovered_propositions.push(final_claim.proposition.clone());
        }
    }

    let factual_claims = candidate.factual_claims.len();
    let factual_claim_coverage = coverage(covered_claims, factual_claims);
    if !uncovered_propositions.is_empty() {
        return FinalizationResult {
            status: FinalizationStatus::RequiresVerification,
            text: None,
            factual_claims,
            covered_claims,
            factual_claim_coverage,
            uncovered_propositions,
        };
    }

    if factual_claims == 0 || verdict == Verdict::Unknown {
        let status = if factual_claims > 0 && has_uncertain && policy.allow_qualified_partial {
            FinalizationStatus::QualifiedPartialAnswer
        } else {
            FinalizationStatus::Unresolved
        };
        return FinalizationResult {
            status,
            text: (status == FinalizationStatus::QualifiedPartialAnswer).then_some(candidate.text),
            factual_claims,
            covered_claims,
            factual_claim_coverage,
            uncovered_propositions,
        };
    }

    let status = if has_uncertain {
        if policy.allow_qualified_partial {
            FinalizationStatus::QualifiedPartialAnswer
        } else {
            FinalizationStatus::Abstain
        }
    } else {
        FinalizationStatus::GroundedAnswer
    };
    FinalizationResult {
        status,
        text: matches!(
            status,
            FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
        )
        .then_some(candidate.text),
        factual_claims,
        covered_claims,
        factual_claim_coverage,
        uncovered_propositions,
    }
}

fn coverage(covered: usize, total: usize) -> f64 {
    if total == 0 {
        1.0
    } else {
        covered as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Claim;

    fn artifact(state: EpistemicState) -> ReasoningArtifact {
        ReasoningArtifact {
            claims: vec![Claim {
                id: "c1".into(),
                statement: "feature.enabled = true".into(),
                state,
                proposition: Some(Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                }),
                evidence_ids: vec![],
            }],
            ..Default::default()
        }
    }

    #[test]
    fn grounded_claim_requires_supported_artifact_proposition() {
        let candidate = FinalAnswerCandidate {
            text: "enabled".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                },
                mode: FinalClaimMode::Grounded,
            }],
        };
        let result = finalize_answer(
            &artifact(EpistemicState::Supported),
            Verdict::Accept,
            candidate,
            FinalizationPolicy::default(),
        );
        assert_eq!(result.status, FinalizationStatus::GroundedAnswer);
        assert_eq!(result.factual_claim_coverage, 1.0);
    }

    #[test]
    fn renderer_cannot_introduce_a_new_grounded_fact() {
        let candidate = FinalAnswerCandidate {
            text: "wrong region".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: Proposition {
                    key: "deployment.region".into(),
                    value: "r2".into(),
                },
                mode: FinalClaimMode::Grounded,
            }],
        };
        let result = finalize_answer(
            &artifact(EpistemicState::Supported),
            Verdict::Accept,
            candidate,
            FinalizationPolicy::default(),
        );
        assert_eq!(result.status, FinalizationStatus::RequiresVerification);
        assert!(result.text.is_none());
        assert_eq!(result.factual_claim_coverage, 0.0);
    }

    #[test]
    fn unknown_can_render_only_as_qualified_uncertainty() {
        let candidate = CanonicalFinalAnswerRenderer
            .render(&artifact(EpistemicState::Unknown), Verdict::Unknown);
        let result = finalize_answer(
            &artifact(EpistemicState::Unknown),
            Verdict::Unknown,
            candidate,
            FinalizationPolicy::default(),
        );
        assert_eq!(result.status, FinalizationStatus::QualifiedPartialAnswer);
        assert_eq!(result.factual_claim_coverage, 1.0);
    }
}
