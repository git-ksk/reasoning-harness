use crate::{
    EpistemicState, HarnessError, Pass, ReasoningArtifact, VerificationConclusion,
    VerificationReceipt,
};

/// Applies harness-owned verification receipts after candidate materialization.
///
/// Receipts are never part of `ReasoningCandidate` or the model prompt. A trusted
/// deterministic oracle or explicitly trusted verifier creates them after candidate
/// generation. Each receipt binds to the exact claim statement and may additionally
/// bind to a claim ID.
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

    fn apply(&self, mut artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError> {
        for receipt in &self.receipts {
            let matches = artifact
                .claims
                .iter()
                .enumerate()
                .filter(|(_, claim)| receipt_matches(receipt, claim))
                .map(|(index, _)| index)
                .collect::<Vec<_>>();

            if matches.len() != 1 {
                return Err(HarnessError::Pass {
                    pass: self.name(),
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
            artifact.verification_receipts.push(receipt.clone());
        }

        Ok(artifact)
    }
}

fn receipt_matches(receipt: &VerificationReceipt, claim: &crate::Claim) -> bool {
    receipt.claim_statement == claim.statement
        && receipt
            .claim_id
            .as_ref()
            .is_none_or(|claim_id| claim_id == &claim.id)
}
