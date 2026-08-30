use crate::{
    Claim, EpistemicState, Evidence, EvidenceAuthorityPolicy, EvidenceRequirement, HarnessError,
    HarnessInput, Pass, ReasoningArtifact, VerificationConclusion, VerificationReceipt,
};

/// Produces hard verification receipts from a candidate claim and harness-owned evidence.
///
/// Implementations belong to the trusted correctness boundary. A verifier must not
/// delegate its final hard conclusion to the same untrusted model that authored the claim.
pub trait Verifier: Send + Sync {
    fn name(&self) -> &'static str;
    fn verify(&self, claim: &Claim, evidence: &[Evidence]) -> Option<VerificationReceipt>;
}

/// A deterministic verifier for canonical `key=value` facts embedded in evidence.
///
/// If all observed values for the proposition key equal the proposed value, the claim is
/// supported. Any observed different value is a hard counterexample and contradicts it.
/// Missing structured facts produce no receipt and therefore preserve uncertainty.
#[derive(Debug, Clone, Copy, Default)]
pub struct StructuredFactVerifier;

impl Verifier for StructuredFactVerifier {
    fn name(&self) -> &'static str {
        "structured_fact_equality"
    }

    fn verify(&self, claim: &Claim, evidence: &[Evidence]) -> Option<VerificationReceipt> {
        let proposition = claim.proposition.as_ref()?;
        let observed = evidence
            .iter()
            .filter_map(|item| {
                item.facts
                    .get(&proposition.key)
                    .map(|value| (item.id.clone(), value))
            })
            .collect::<Vec<_>>();
        structured_receipt(self.name(), claim, proposition, observed)
    }
}

fn structured_receipt(
    verifier: &'static str,
    claim: &Claim,
    proposition: &crate::Proposition,
    observed: Vec<(String, &String)>,
) -> Option<VerificationReceipt> {
    if observed.is_empty() {
        return None;
    }
    let conclusion = if observed
        .iter()
        .all(|(_, value)| *value == &proposition.value)
    {
        VerificationConclusion::Supported
    } else {
        VerificationConclusion::Contradicted
    };
    Some(VerificationReceipt {
        id: format!("{verifier}:{}", claim.id),
        verifier: verifier.into(),
        claim_statement: None,
        proposition: Some(proposition.clone()),
        claim_id: Some(claim.id.clone()),
        conclusion,
        evidence_ids: observed.into_iter().map(|(id, _)| id).collect(),
    })
}

/// Deterministic structured-fact verifier that applies harness-owned evidence qualification
/// requirements before a fact can create a hard receipt. Missing/insufficient qualification
/// withholds the receipt and therefore preserves uncertainty rather than forcing rejection.
#[derive(Debug, Clone, Default)]
pub struct QualifiedStructuredFactVerifier {
    requirements: Vec<EvidenceRequirement>,
    authority_policy: EvidenceAuthorityPolicy,
}

impl QualifiedStructuredFactVerifier {
    pub fn new(
        requirements: Vec<EvidenceRequirement>,
        authority_policy: EvidenceAuthorityPolicy,
    ) -> Self {
        Self {
            requirements,
            authority_policy,
        }
    }
}

impl Verifier for QualifiedStructuredFactVerifier {
    fn name(&self) -> &'static str {
        "qualified_structured_fact_equality"
    }

    fn verify(&self, claim: &Claim, evidence: &[Evidence]) -> Option<VerificationReceipt> {
        let proposition = claim.proposition.as_ref()?;
        let requirement = self
            .requirements
            .iter()
            .find(|requirement| requirement.proposition.key == proposition.key);
        let observed = evidence
            .iter()
            .filter(|item| item.facts.contains_key(&proposition.key))
            .filter(|item| {
                requirement.is_none_or(|requirement| {
                    crate::evidence_qualification::qualification_reasons(
                        &self.authority_policy,
                        requirement,
                        item,
                    )
                    .is_empty()
                })
            })
            .filter_map(|item| {
                item.facts
                    .get(&proposition.key)
                    .map(|value| (item.id.clone(), value))
            })
            .collect::<Vec<_>>();

        // When qualification requirements are active, multiple qualified records with
        // conflicting values are an evidence conflict, not a hard verdict. The
        // qualification inspector records that conflict separately; verification
        // withholds a receipt so the claim remains uncertain.
        if requirement.is_some()
            && observed
                .iter()
                .map(|(_, value)| value.as_str())
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1
        {
            return None;
        }
        structured_receipt(self.name(), claim, proposition, observed)
    }
}

/// Selects the compatibility verifier when no qualification requirements exist and the
/// qualification-aware verifier otherwise. This preserves historical behavior for old inputs.
pub fn structured_fact_verifier_for_input(input: &HarnessInput) -> Box<dyn Verifier> {
    if input.evidence_requirements.is_empty() {
        Box::new(StructuredFactVerifier)
    } else {
        Box::new(QualifiedStructuredFactVerifier::new(
            input.evidence_requirements.clone(),
            input.authority_policy.clone(),
        ))
    }
}

/// Runs one or more trusted verifier adapters after candidate materialization.
pub struct VerificationPass {
    verifiers: Vec<Box<dyn Verifier>>,
}

impl VerificationPass {
    pub fn new(verifiers: Vec<Box<dyn Verifier>>) -> Self {
        Self { verifiers }
    }
}

impl Pass for VerificationPass {
    fn name(&self) -> &'static str {
        "verification"
    }

    fn apply(&self, artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError> {
        let receipts = artifact
            .claims
            .iter()
            .flat_map(|claim| {
                self.verifiers
                    .iter()
                    .filter_map(|verifier| verifier.verify(claim, &artifact.evidence))
            })
            .collect::<Vec<_>>();
        apply_receipts(artifact, &receipts, self.name())
    }
}

/// Applies explicitly supplied harness-owned receipts after candidate materialization.
///
/// This remains a conservative compatibility mode for external verifiers that already
/// produce receipts. New integrations should prefer a typed `Verifier` adapter.
#[derive(Debug, Clone, Default)]
pub struct TrustedVerificationPass {
    receipts: Vec<VerificationReceipt>,
}

impl TrustedVerificationPass {
    pub fn new(receipts: Vec<VerificationReceipt>) -> Self {
        Self { receipts }
    }
}

impl Pass for TrustedVerificationPass {
    fn name(&self) -> &'static str {
        "trusted_verification"
    }

    fn apply(&self, artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError> {
        apply_receipts(artifact, &self.receipts, self.name())
    }
}

fn apply_receipts(
    mut artifact: ReasoningArtifact,
    receipts: &[VerificationReceipt],
    pass: &'static str,
) -> Result<ReasoningArtifact, HarnessError> {
    for receipt in receipts {
        let matches = artifact
            .claims
            .iter()
            .enumerate()
            .filter(|(_, claim)| receipt_matches(receipt, claim))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        if matches.len() != 1 {
            return Err(HarnessError::Pass {
                pass,
                message: format!(
                    "verification receipt {} matched {} claims; expected exactly one",
                    receipt.id,
                    matches.len()
                ),
            });
        }

        let claim = &mut artifact.claims[matches[0]];
        match receipt.conclusion {
            VerificationConclusion::Supported => {
                claim.state = EpistemicState::Supported;
                claim.evidence_ids = receipt.evidence_ids.clone();
            }
            VerificationConclusion::Contradicted => {
                claim.state = EpistemicState::Contradicted;
            }
        }
        // A proposition-bound hard verifier owns the authoritative rendering.
        // Provider prose remains useful while unverified, but once a machine fact is
        // accepted or contradicted we must not present arbitrary model wording as if
        // the verifier endorsed it. Legacy statement-bound receipts retain their
        // exact statement because the statement itself is part of their binding.
        if receipt.claim_statement.is_none() {
            if let Some(proposition) = &receipt.proposition {
                claim.statement = canonical_statement(proposition);
            }
        }
        artifact.verification_receipts.push(receipt.clone());
    }

    Ok(artifact)
}

fn canonical_statement(proposition: &crate::Proposition) -> String {
    format!("{} = {}", proposition.key, proposition.value)
}

pub(crate) fn receipt_matches(receipt: &VerificationReceipt, claim: &Claim) -> bool {
    let statement_matches = receipt
        .claim_statement
        .as_ref()
        .is_none_or(|statement| statement == &claim.statement);
    let proposition_matches = receipt
        .proposition
        .as_ref()
        .is_none_or(|proposition| claim.proposition.as_ref() == Some(proposition));
    let id_matches = receipt
        .claim_id
        .as_ref()
        .is_none_or(|claim_id| claim_id == &claim.id);
    statement_matches && proposition_matches && id_matches
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    fn evidence(value: &str) -> Evidence {
        Evidence {
            id: "e1".into(),
            source: "machine fact".into(),
            observation: format!("status={value}"),
            facts: BTreeMap::from([("http.status_code".into(), value.into())]),
            metadata: Default::default(),
        }
    }

    fn claim(value: &str) -> Claim {
        Claim {
            id: "c1".into(),
            statement: "provider prose may vary".into(),
            state: EpistemicState::Assumed,
            proposition: Some(crate::Proposition {
                key: "http.status_code".into(),
                value: value.into(),
            }),
            evidence_ids: vec![],
        }
    }

    #[test]
    fn structured_fact_verifier_supports_equal_fact() {
        let receipt = StructuredFactVerifier
            .verify(&claim("503"), &[evidence("503")])
            .unwrap();
        assert_eq!(receipt.conclusion, VerificationConclusion::Supported);
    }

    #[test]
    fn structured_fact_verifier_contradicts_different_fact() {
        let receipt = StructuredFactVerifier
            .verify(&claim("200"), &[evidence("503")])
            .unwrap();
        assert_eq!(receipt.conclusion, VerificationConclusion::Contradicted);
    }

    #[test]
    fn structured_verification_replaces_provider_prose_with_canonical_fact() {
        let artifact = ReasoningArtifact {
            task: "status".into(),
            evidence: vec![evidence("503")],
            hypotheses: vec![],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: Default::default(),
            candidate_diagnostics: vec![],
            verification_receipts: vec![],
            adversarial_findings: vec![],
            assumption_findings: vec![],
            evidence_qualification_findings: vec![],
            claims: vec![claim("503")],
            inferences: vec![],
        };
        let pass = VerificationPass::new(vec![Box::new(StructuredFactVerifier)]);
        let result = pass.apply(artifact).unwrap();
        assert_eq!(result.claims[0].state, EpistemicState::Supported);
        assert_eq!(result.claims[0].statement, "http.status_code = 503");
    }

    fn qualified_requirement(
        as_of: Option<i64>,
        minimum_authority_class: Option<&str>,
    ) -> EvidenceRequirement {
        EvidenceRequirement {
            proposition: crate::Proposition {
                key: "http.status_code".into(),
                value: "503".into(),
            },
            as_of_unix_seconds: as_of,
            scope: None,
            minimum_authority_class: minimum_authority_class.map(str::to_string),
        }
    }

    #[test]
    fn qualified_verifier_supports_only_qualified_evidence() {
        let mut item = evidence("503");
        item.metadata.temporal = Some(crate::TemporalValidity {
            effective_from_unix_seconds: Some(100),
            effective_until_unix_seconds: Some(300),
        });
        let verifier = QualifiedStructuredFactVerifier::new(
            vec![qualified_requirement(Some(200), None)],
            Default::default(),
        );
        let receipt = verifier.verify(&claim("503"), &[item]).unwrap();
        assert_eq!(receipt.conclusion, VerificationConclusion::Supported);
        assert_eq!(receipt.verifier, "qualified_structured_fact_equality");
    }

    #[test]
    fn qualified_verifier_withholds_stale_evidence() {
        let mut item = evidence("503");
        item.metadata.temporal = Some(crate::TemporalValidity {
            effective_from_unix_seconds: Some(0),
            effective_until_unix_seconds: Some(100),
        });
        let verifier = QualifiedStructuredFactVerifier::new(
            vec![qualified_requirement(Some(200), None)],
            Default::default(),
        );
        assert!(verifier.verify(&claim("503"), &[item]).is_none());
    }

    #[test]
    fn qualified_verifier_withholds_insufficient_authority() {
        let mut item = evidence("503");
        item.metadata.provenance_class = Some("secondary".into());
        let verifier = QualifiedStructuredFactVerifier::new(
            vec![qualified_requirement(None, Some("primary"))],
            crate::EvidenceAuthorityPolicy {
                ranks: BTreeMap::from([("secondary".into(), 10), ("primary".into(), 20)]),
            },
        );
        assert!(verifier.verify(&claim("503"), &[item]).is_none());
    }

    #[test]
    fn qualified_verifier_withholds_conflicting_qualified_evidence() {
        let verifier = QualifiedStructuredFactVerifier::new(
            vec![qualified_requirement(None, None)],
            Default::default(),
        );
        assert!(
            verifier
                .verify(&claim("503"), &[evidence("503"), evidence("200")])
                .is_none()
        );
    }

    #[test]
    fn same_key_opposite_value_cannot_bypass_qualification_requirement() {
        let mut item = evidence("503");
        item.metadata.temporal = Some(crate::TemporalValidity {
            effective_from_unix_seconds: Some(0),
            effective_until_unix_seconds: Some(100),
        });
        let verifier = QualifiedStructuredFactVerifier::new(
            vec![qualified_requirement(Some(200), None)],
            Default::default(),
        );
        assert!(verifier.verify(&claim("200"), &[item]).is_none());
    }
}
