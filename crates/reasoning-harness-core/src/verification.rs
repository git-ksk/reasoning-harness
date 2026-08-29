use crate::{
    Claim, EpistemicState, Evidence, HarnessError, Pass, ReasoningArtifact, VerificationConclusion,
    VerificationReceipt,
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
            id: format!("{}:{}", self.name(), claim.id),
            verifier: self.name().into(),
            claim_statement: None,
            proposition: Some(proposition.clone()),
            claim_id: Some(claim.id.clone()),
            conclusion,
            evidence_ids: observed.into_iter().map(|(id, _)| id).collect(),
        })
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

fn receipt_matches(receipt: &VerificationReceipt, claim: &Claim) -> bool {
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
            candidate_diagnostics: vec![],
            verification_receipts: vec![],
            claims: vec![claim("503")],
            inferences: vec![],
        };
        let pass = VerificationPass::new(vec![Box::new(StructuredFactVerifier)]);
        let result = pass.apply(artifact).unwrap();
        assert_eq!(result.claims[0].state, EpistemicState::Supported);
        assert_eq!(result.claims[0].statement, "http.status_code = 503");
    }
}
