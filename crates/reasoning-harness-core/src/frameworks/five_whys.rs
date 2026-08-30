use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{EpistemicState, HarnessError, Pass, ReasoningArtifact};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhyLink {
    pub effect: String,
    pub cause: String,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FiveWhysTrace {
    pub symptom: String,
    #[serde(default)]
    pub links: Vec<WhyLink>,
    pub root_cause: String,
}

pub fn validate_trace(trace: &FiveWhysTrace) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for (index, link) in trace.links.iter().enumerate() {
        if is_lexical_restatement(&link.effect, &link.cause) {
            diagnostics.push(format!(
                "why link {index} restates the effect instead of identifying a distinct cause"
            ));
        }
    }
    diagnostics
}

/// Removes Five Whys edges that are deterministically recognizable lexical restatements.
///
/// This is intentionally a narrow syntactic heuristic, not a semantic causal judge. Cleanup is
/// localized to the exact offending inference edge. A conclusion is downgraded only when it has
/// no surviving Five Whys support, and a `supported` claim is never downgraded because that state
/// may have been established independently by a trusted verifier.
#[derive(Debug, Clone, Copy, Default)]
pub struct FiveWhysRestatementPass;

impl Pass for FiveWhysRestatementPass {
    fn name(&self) -> &'static str {
        "five_whys_restatement"
    }

    fn apply(&self, mut artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError> {
        let statements = artifact
            .claims
            .iter()
            .map(|claim| (claim.id.as_str(), claim.statement.as_str()))
            .collect::<std::collections::HashMap<_, _>>();

        let removed_inference_ids = artifact
            .inferences
            .iter()
            .filter(|inference| inference.method == "five_whys")
            .filter(|inference| {
                let Some(conclusion) = statements.get(inference.conclusion_claim_id.as_str())
                else {
                    return false;
                };
                inference.premise_claim_ids.iter().any(|premise_id| {
                    statements
                        .get(premise_id.as_str())
                        .is_some_and(|premise| is_lexical_restatement(premise, conclusion))
                })
            })
            .map(|inference| inference.id.clone())
            .collect::<HashSet<_>>();

        let removed_conclusions = artifact
            .inferences
            .iter()
            .filter(|inference| removed_inference_ids.contains(&inference.id))
            .map(|inference| inference.conclusion_claim_id.clone())
            .collect::<HashSet<_>>();

        artifact
            .inferences
            .retain(|inference| !removed_inference_ids.contains(&inference.id));

        let surviving_five_whys_conclusions = artifact
            .inferences
            .iter()
            .filter(|inference| inference.method == "five_whys")
            .map(|inference| inference.conclusion_claim_id.clone())
            .collect::<HashSet<_>>();

        for claim in &mut artifact.claims {
            if removed_conclusions.contains(&claim.id)
                && !surviving_five_whys_conclusions.contains(&claim.id)
                && claim.state == EpistemicState::Inferred
            {
                claim.state = EpistemicState::Assumed;
            }
        }

        Ok(artifact)
    }
}

pub(crate) fn is_lexical_restatement(left: &str, right: &str) -> bool {
    let left = content_tokens(left);
    let right = content_tokens(right);
    if left.is_empty() || right.is_empty() {
        return false;
    }
    if left == right {
        return true;
    }

    let intersection = left.intersection(&right).count();
    let smaller = left.len().min(right.len());
    intersection >= 2 && intersection * 3 >= smaller * 2
}

fn content_tokens(value: &str) -> HashSet<String> {
    value
        .split(|character: char| !character.is_alphanumeric())
        .filter_map(|token| {
            let token = token.to_lowercase();
            let canonical = match token.as_str() {
                "timed" | "timeout" | "timeouts" => "timeout",
                "failed" | "failure" | "failures" => "fail",
                "occurred" | "occurs" | "occur" => "occur",
                _ => token.as_str(),
            };
            if matches!(
                canonical,
                "a" | "an"
                    | "the"
                    | "in"
                    | "on"
                    | "at"
                    | "of"
                    | "to"
                    | "is"
                    | "was"
                    | "were"
                    | "be"
                    | "and"
                    | "or"
            ) || canonical.len() < 2
            {
                None
            } else {
                Some(canonical.to_string())
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{FiveWhysRestatementPass, is_lexical_restatement};
    use crate::{Claim, EpistemicState, Inference, Pass, ReasoningArtifact};

    #[test]
    fn detects_timeout_restatement() {
        assert!(is_lexical_restatement(
            "The job timed out.",
            "A timeout occurred in the job."
        ));
    }

    #[test]
    fn does_not_conflate_distinct_cause() {
        assert!(!is_lexical_restatement(
            "The job timed out.",
            "The database lock blocked progress."
        ));
    }

    #[test]
    fn cleanup_is_local_to_the_offending_edge() {
        let mut artifact = ReasoningArtifact::default();
        artifact.claims = vec![
            Claim {
                id: "effect_bad".into(),
                statement: "The job timed out".into(),
                state: EpistemicState::Assumed,
                proposition: None,
                evidence_ids: vec![],
            },
            Claim {
                id: "effect_good".into(),
                statement: "The database lock blocked progress".into(),
                state: EpistemicState::Assumed,
                proposition: None,
                evidence_ids: vec![],
            },
            Claim {
                id: "cause".into(),
                statement: "A timeout occurred in the job".into(),
                state: EpistemicState::Inferred,
                proposition: None,
                evidence_ids: vec![],
            },
        ];
        artifact.inferences = vec![
            Inference {
                id: "bad".into(),
                premise_claim_ids: vec!["effect_bad".into()],
                conclusion_claim_id: "cause".into(),
                method: "five_whys".into(),
            },
            Inference {
                id: "good".into(),
                premise_claim_ids: vec!["effect_good".into()],
                conclusion_claim_id: "cause".into(),
                method: "five_whys".into(),
            },
        ];

        let artifact = FiveWhysRestatementPass.apply(artifact).unwrap();

        assert_eq!(artifact.inferences.len(), 1);
        assert_eq!(artifact.inferences[0].id, "good");
        assert_eq!(artifact.claims[2].state, EpistemicState::Inferred);
    }

    #[test]
    fn cleanup_does_not_downgrade_independently_supported_claim() {
        let mut artifact = ReasoningArtifact::default();
        artifact.claims = vec![
            Claim {
                id: "effect".into(),
                statement: "The job timed out".into(),
                state: EpistemicState::Assumed,
                proposition: None,
                evidence_ids: vec![],
            },
            Claim {
                id: "cause".into(),
                statement: "A timeout occurred in the job".into(),
                state: EpistemicState::Supported,
                proposition: None,
                evidence_ids: vec![],
            },
        ];
        artifact.inferences = vec![Inference {
            id: "bad".into(),
            premise_claim_ids: vec!["effect".into()],
            conclusion_claim_id: "cause".into(),
            method: "five_whys".into(),
        }];

        let artifact = FiveWhysRestatementPass.apply(artifact).unwrap();

        assert!(artifact.inferences.is_empty());
        assert_eq!(artifact.claims[1].state, EpistemicState::Supported);
    }
}
