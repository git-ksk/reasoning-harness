use std::collections::BTreeSet;

use crate::{
    AdversarialFinding, AdversarialFindingKind, FindingStrength, HarnessError, Pass,
    ReasoningArtifact,
};

/// Produces typed adversarial findings without granting discovery output verdict authority.
pub trait AdversarialDetector: Send + Sync {
    fn name(&self) -> &'static str;
    fn discover(&self, artifact: &ReasoningArtifact) -> Vec<AdversarialFinding>;
}

/// Deterministic detector over harness-owned structured facts.
#[derive(Debug, Clone, Copy, Default)]
pub struct StructuredFactConflictDetector;

impl AdversarialDetector for StructuredFactConflictDetector {
    fn name(&self) -> &'static str {
        "structured_fact_conflict"
    }

    fn discover(&self, artifact: &ReasoningArtifact) -> Vec<AdversarialFinding> {
        let mut findings = Vec::new();
        for claim in &artifact.claims {
            let Some(proposition) = claim.proposition.as_ref() else {
                continue;
            };
            let observations = artifact
                .evidence
                .iter()
                .filter_map(|evidence| {
                    evidence
                        .facts
                        .get(&proposition.key)
                        .map(|value| (evidence.id.as_str(), value.as_str()))
                })
                .collect::<Vec<_>>();
            let contrary = observations
                .iter()
                .filter(|(_, value)| *value != proposition.value)
                .map(|(id, _)| (*id).to_string())
                .collect::<Vec<_>>();
            if contrary.is_empty() {
                continue;
            }

            let distinct_values = observations
                .iter()
                .map(|(_, value)| *value)
                .collect::<BTreeSet<_>>();
            let kind = if distinct_values.len() > 1 {
                AdversarialFindingKind::Contradiction
            } else {
                AdversarialFindingKind::Counterexample
            };
            findings.push(AdversarialFinding {
                id: format!("{}:{}", self.name(), claim.id),
                detector: self.name().into(),
                kind,
                strength: FindingStrength::Hard,
                claim_id: claim.id.clone(),
                proposition: proposition.clone(),
                evidence_ids: contrary,
                message: match kind {
                    AdversarialFindingKind::Contradiction => {
                        "harness-owned structured evidence contains conflicting values".into()
                    }
                    AdversarialFindingKind::Counterexample => {
                        "harness-owned structured evidence contains a counterexample".into()
                    }
                },
            });
        }
        findings
    }
}

/// Runs adversarial detectors after candidate materialization. Findings are observational;
/// hard verdict authority still comes from trusted verification passes.
pub struct AdversarialDiscoveryPass {
    detectors: Vec<Box<dyn AdversarialDetector>>,
}

impl AdversarialDiscoveryPass {
    pub fn new(detectors: Vec<Box<dyn AdversarialDetector>>) -> Self {
        Self { detectors }
    }
}

impl Pass for AdversarialDiscoveryPass {
    fn name(&self) -> &'static str {
        "adversarial_discovery"
    }

    fn apply(&self, mut artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError> {
        artifact.adversarial_findings = self
            .detectors
            .iter()
            .flat_map(|detector| detector.discover(&artifact))
            .collect();
        Ok(artifact)
    }
}

/// Soft findings are intentionally observational only. Keeping this helper in core makes
/// the trust distinction explicit for future semantic/model-backed discovery adapters.
pub fn record_soft_finding(artifact: &mut ReasoningArtifact, mut finding: AdversarialFinding) {
    finding.strength = FindingStrength::Soft;
    artifact.adversarial_findings.push(finding);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{Claim, EpistemicState, Evidence, Proposition};

    use super::*;

    fn artifact(values: &[&str], proposed: &str) -> ReasoningArtifact {
        ReasoningArtifact {
            task: "test".into(),
            evidence: values
                .iter()
                .enumerate()
                .map(|(index, value)| Evidence {
                    id: format!("e{index}"),
                    source: "fixture".into(),
                    observation: format!("value={value}"),
                    facts: BTreeMap::from([("feature.enabled".into(), (*value).into())]),
                })
                .collect(),
            hypotheses: vec![],
            candidate_diagnostics: vec![],
            verification_receipts: vec![],
            adversarial_findings: vec![],
            claims: vec![Claim {
                id: "c1".into(),
                statement: "provider prose".into(),
                state: EpistemicState::Assumed,
                proposition: Some(Proposition {
                    key: "feature.enabled".into(),
                    value: proposed.into(),
                }),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        }
    }

    #[test]
    fn single_opposing_value_is_counterexample() {
        let result = AdversarialDiscoveryPass::new(vec![Box::new(StructuredFactConflictDetector)])
            .apply(artifact(&["false"], "true"))
            .unwrap();
        assert_eq!(result.adversarial_findings.len(), 1);
        assert_eq!(
            result.adversarial_findings[0].kind,
            AdversarialFindingKind::Counterexample
        );
        assert_eq!(
            result.adversarial_findings[0].strength,
            FindingStrength::Hard
        );
    }

    #[test]
    fn conflicting_observed_values_are_contradiction() {
        let result = AdversarialDiscoveryPass::new(vec![Box::new(StructuredFactConflictDetector)])
            .apply(artifact(&["true", "false"], "true"))
            .unwrap();
        assert_eq!(
            result.adversarial_findings[0].kind,
            AdversarialFindingKind::Contradiction
        );
    }

    #[test]
    fn soft_finding_never_changes_claim_state() {
        let mut artifact = artifact(&[], "true");
        record_soft_finding(
            &mut artifact,
            AdversarialFinding {
                id: "soft-1".into(),
                detector: "semantic_suggestion".into(),
                kind: AdversarialFindingKind::Counterexample,
                strength: FindingStrength::Hard,
                claim_id: "c1".into(),
                proposition: Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                },
                evidence_ids: vec![],
                message: "unverified suggestion".into(),
            },
        );
        assert_eq!(artifact.claims[0].state, EpistemicState::Assumed);
        assert_eq!(
            artifact.adversarial_findings[0].strength,
            FindingStrength::Soft
        );
    }
}
