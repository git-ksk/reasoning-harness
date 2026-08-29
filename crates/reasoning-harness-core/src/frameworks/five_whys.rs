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
/// This is intentionally a narrow syntactic heuristic, not a semantic causal judge. The
/// conclusion claim remains in the artifact as an assumption/unknown; only the invalid
/// causal edge is removed.
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

        let removed_conclusions = artifact
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
            .map(|inference| inference.conclusion_claim_id.clone())
            .collect::<HashSet<_>>();

        artifact.inferences.retain(|inference| {
            !(inference.method == "five_whys"
                && removed_conclusions.contains(&inference.conclusion_claim_id))
        });

        for claim in &mut artifact.claims {
            if removed_conclusions.contains(&claim.id)
                && matches!(
                    claim.state,
                    EpistemicState::Inferred | EpistemicState::Supported
                )
            {
                claim.state = EpistemicState::Assumed;
            }
        }

        Ok(artifact)
    }
}

fn is_lexical_restatement(left: &str, right: &str) -> bool {
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
    use super::is_lexical_restatement;

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
}
