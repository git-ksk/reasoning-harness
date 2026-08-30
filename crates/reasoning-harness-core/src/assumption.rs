use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{EpistemicState, FindingStrength, HarnessError, Pass, Proposition, ReasoningArtifact};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssumptionSupportStatus {
    Supported,
    ExplicitInputAssumption,
    Unsupported,
    Unbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssumptionFindingKind {
    UnsupportedPremise,
    UnboundPremise,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AssumptionAssessment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposition: Option<Proposition>,
    #[serde(default)]
    pub claim_ids: Vec<String>,
    #[serde(default)]
    pub inference_ids: Vec<String>,
    pub status: AssumptionSupportStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AssumptionFinding {
    pub id: String,
    pub detector: String,
    pub kind: AssumptionFindingKind,
    pub strength: FindingStrength,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposition: Option<Proposition>,
    #[serde(default)]
    pub claim_ids: Vec<String>,
    #[serde(default)]
    pub inference_ids: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AssumptionInspection {
    #[serde(default)]
    pub assessments: Vec<AssumptionAssessment>,
    #[serde(default)]
    pub findings: Vec<AssumptionFinding>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AssumptionInspector;

impl AssumptionInspector {
    pub fn inspect(&self, artifact: &ReasoningArtifact) -> AssumptionInspection {
        let trusted_support = trusted_support_closure(artifact);
        let mut usages = BTreeMap::<String, PremiseUsage>::new();

        for inference in &artifact.inferences {
            for premise_id in &inference.premise_claim_ids {
                let Some(claim) = artifact.claims.iter().find(|claim| claim.id == *premise_id)
                else {
                    continue;
                };
                let key = claim
                    .proposition
                    .as_ref()
                    .map(proposition_key)
                    .unwrap_or_else(|| format!("unbound:{}", claim.id));
                let usage = usages.entry(key).or_insert_with(|| PremiseUsage {
                    proposition: claim.proposition.clone(),
                    claim_ids: Vec::new(),
                    inference_ids: Vec::new(),
                    trusted_supported: false,
                });
                usage.claim_ids.push(claim.id.clone());
                usage.inference_ids.push(inference.id.clone());
                usage.trusted_supported |= trusted_support.contains(claim.id.as_str());
            }
        }

        let mut inspection = AssumptionInspection::default();
        for mut usage in usages.into_values() {
            usage.claim_ids.sort();
            usage.claim_ids.dedup();
            usage.inference_ids.sort();
            usage.inference_ids.dedup();

            let status = match usage.proposition.as_ref() {
                Some(_) if usage.trusted_supported => AssumptionSupportStatus::Supported,
                Some(proposition) if artifact.assumptions.contains(proposition) => {
                    AssumptionSupportStatus::ExplicitInputAssumption
                }
                Some(_) => AssumptionSupportStatus::Unsupported,
                None => AssumptionSupportStatus::Unbound,
            };
            let assessment = AssumptionAssessment {
                proposition: usage.proposition.clone(),
                claim_ids: usage.claim_ids.clone(),
                inference_ids: usage.inference_ids.clone(),
                status,
            };
            match status {
                AssumptionSupportStatus::Unsupported => {
                    inspection.findings.push(AssumptionFinding {
                        id: finding_id("unsupported", &usage),
                        detector: "assumption_inspector".into(),
                        kind: AssumptionFindingKind::UnsupportedPremise,
                        strength: FindingStrength::Hard,
                        proposition: usage.proposition.clone(),
                        claim_ids: usage.claim_ids,
                        inference_ids: usage.inference_ids,
                        message: "typed premise has no trusted support and is not an explicit harness-owned assumption"
                            .into(),
                    });
                }
                AssumptionSupportStatus::Unbound => {
                    inspection.findings.push(AssumptionFinding {
                        id: finding_id("unbound", &usage),
                        detector: "assumption_inspector".into(),
                        kind: AssumptionFindingKind::UnboundPremise,
                        strength: FindingStrength::Soft,
                        proposition: None,
                        claim_ids: usage.claim_ids,
                        inference_ids: usage.inference_ids,
                        message: "premise has no typed proposition binding, so support cannot be checked deterministically"
                            .into(),
                    });
                }
                AssumptionSupportStatus::Supported
                | AssumptionSupportStatus::ExplicitInputAssumption => {}
            }
            inspection.assessments.push(assessment);
        }
        inspection
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AssumptionDiscoveryPass;

impl Pass for AssumptionDiscoveryPass {
    fn name(&self) -> &'static str {
        "assumption_discovery"
    }

    fn apply(&self, mut artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError> {
        artifact.assumption_findings = AssumptionInspector.inspect(&artifact).findings;
        Ok(artifact)
    }
}

fn trusted_support_closure(artifact: &ReasoningArtifact) -> BTreeSet<&str> {
    let mut supported = artifact
        .claims
        .iter()
        .filter(|claim| {
            matches!(
                claim.state,
                EpistemicState::Known | EpistemicState::Supported
            )
        })
        .map(|claim| claim.id.as_str())
        .collect::<BTreeSet<_>>();
    let explicit = artifact
        .claims
        .iter()
        .filter(|claim| {
            claim
                .proposition
                .as_ref()
                .is_some_and(|proposition| artifact.assumptions.contains(proposition))
        })
        .map(|claim| claim.id.as_str())
        .collect::<BTreeSet<_>>();

    loop {
        let mut changed = false;
        for inference in &artifact.inferences {
            let Some(conclusion) = artifact
                .claims
                .iter()
                .find(|claim| claim.id == inference.conclusion_claim_id)
            else {
                continue;
            };
            if conclusion.state != EpistemicState::Inferred
                || supported.contains(conclusion.id.as_str())
            {
                continue;
            }
            if inference.premise_claim_ids.iter().all(|premise| {
                supported.contains(premise.as_str()) || explicit.contains(premise.as_str())
            }) {
                supported.insert(conclusion.id.as_str());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    supported
}

#[derive(Debug)]
struct PremiseUsage {
    proposition: Option<Proposition>,
    claim_ids: Vec<String>,
    inference_ids: Vec<String>,
    trusted_supported: bool,
}

fn proposition_key(proposition: &Proposition) -> String {
    format!("typed:{}\u{0}{}", proposition.key, proposition.value)
}

fn finding_id(prefix: &str, usage: &PremiseUsage) -> String {
    let anchor = usage
        .claim_ids
        .first()
        .map(String::as_str)
        .unwrap_or("unknown");
    format!("assumption:{prefix}:{anchor}")
}

#[cfg(test)]
mod tests {
    use crate::{Claim, Inference, ReasoningArtifact};

    use super::*;

    fn proposition() -> Proposition {
        Proposition {
            key: "premise.enabled".into(),
            value: "true".into(),
        }
    }

    fn artifact(state: EpistemicState, bound: bool) -> ReasoningArtifact {
        ReasoningArtifact {
            task: "test".into(),
            evidence: vec![],
            hypotheses: vec![],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: Default::default(),
            candidate_diagnostics: vec![],
            verification_receipts: vec![],
            adversarial_findings: vec![],
            assumption_findings: vec![],
            evidence_qualification_findings: vec![],
            claims: vec![
                Claim {
                    id: "premise".into(),
                    statement: "premise".into(),
                    state,
                    proposition: bound.then(proposition),
                    evidence_ids: vec![],
                },
                Claim {
                    id: "result".into(),
                    statement: "result".into(),
                    state: EpistemicState::Assumed,
                    proposition: None,
                    evidence_ids: vec![],
                },
            ],
            inferences: vec![Inference {
                id: "i1".into(),
                premise_claim_ids: vec!["premise".into()],
                conclusion_claim_id: "result".into(),
                method: "fixture".into(),
            }],
        }
    }

    #[test]
    fn typed_untrusted_premise_is_hard_unsupported_process_finding() {
        let inspection = AssumptionInspector.inspect(&artifact(EpistemicState::Assumed, true));
        assert_eq!(
            inspection.assessments[0].status,
            AssumptionSupportStatus::Unsupported
        );
        assert_eq!(inspection.findings[0].strength, FindingStrength::Hard);
        assert_eq!(
            inspection.findings[0].kind,
            AssumptionFindingKind::UnsupportedPremise
        );
    }

    #[test]
    fn explicit_input_assumption_is_distinguished_without_a_finding() {
        let mut artifact = artifact(EpistemicState::Assumed, true);
        artifact.assumptions.push(proposition());
        let inspection = AssumptionInspector.inspect(&artifact);
        assert_eq!(
            inspection.assessments[0].status,
            AssumptionSupportStatus::ExplicitInputAssumption
        );
        assert!(inspection.findings.is_empty());
    }

    #[test]
    fn supported_premise_is_not_an_assumption_finding() {
        let inspection = AssumptionInspector.inspect(&artifact(EpistemicState::Supported, true));
        assert_eq!(
            inspection.assessments[0].status,
            AssumptionSupportStatus::Supported
        );
        assert!(inspection.findings.is_empty());
    }

    #[test]
    fn unbound_premise_remains_soft() {
        let inspection = AssumptionInspector.inspect(&artifact(EpistemicState::Assumed, false));
        assert_eq!(
            inspection.assessments[0].status,
            AssumptionSupportStatus::Unbound
        );
        assert_eq!(inspection.findings[0].strength, FindingStrength::Soft);
    }

    #[test]
    fn inferred_label_without_supported_derivation_is_not_trusted() {
        let mut artifact = artifact(EpistemicState::Inferred, true);
        artifact.inferences.insert(
            0,
            Inference {
                id: "derive-premise".into(),
                premise_claim_ids: vec!["result".into()],
                conclusion_claim_id: "premise".into(),
                method: "fixture".into(),
            },
        );
        let inspection = AssumptionInspector.inspect(&artifact);
        let premise = inspection
            .assessments
            .iter()
            .find(|assessment| assessment.proposition == Some(proposition()))
            .unwrap();
        assert_eq!(premise.status, AssumptionSupportStatus::Unsupported);
    }

    #[test]
    fn inferred_premise_with_supported_derivation_is_accepted_as_derived_support() {
        let mut artifact = artifact(EpistemicState::Inferred, true);
        artifact.claims.push(Claim {
            id: "root".into(),
            statement: "root".into(),
            state: EpistemicState::Supported,
            proposition: Some(Proposition {
                key: "root.verified".into(),
                value: "true".into(),
            }),
            evidence_ids: vec!["e1".into()],
        });
        artifact.inferences.insert(
            0,
            Inference {
                id: "derive-premise".into(),
                premise_claim_ids: vec!["root".into()],
                conclusion_claim_id: "premise".into(),
                method: "fixture".into(),
            },
        );
        let inspection = AssumptionInspector.inspect(&artifact);
        let premise = inspection
            .assessments
            .iter()
            .find(|assessment| assessment.proposition == Some(proposition()))
            .unwrap();
        assert_eq!(premise.status, AssumptionSupportStatus::Supported);
    }

    #[test]
    fn repeated_semantic_premise_is_reported_once_with_all_edges() {
        let mut artifact = artifact(EpistemicState::Assumed, true);
        artifact.inferences.push(Inference {
            id: "i2".into(),
            premise_claim_ids: vec!["premise".into()],
            conclusion_claim_id: "result".into(),
            method: "fixture".into(),
        });
        let inspection = AssumptionInspector.inspect(&artifact);
        assert_eq!(inspection.findings.len(), 1);
        assert_eq!(inspection.findings[0].inference_ids, vec!["i1", "i2"]);
    }
}
