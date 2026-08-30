use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicState {
    Known,
    Supported,
    Inferred,
    Assumed,
    Contradicted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Evidence {
    pub id: String,
    pub source: String,
    pub observation: String,
    #[serde(default)]
    pub facts: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Proposition {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Claim {
    pub id: String,
    pub statement: String,
    pub state: EpistemicState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposition: Option<Proposition>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CandidateClaim {
    pub id: String,
    pub statement: String,
    pub proposed_state: EpistemicState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposition: Option<Proposition>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Inference {
    pub id: String,
    #[serde(default)]
    pub premise_claim_ids: Vec<String>,
    pub conclusion_claim_id: String,
    pub method: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FindingStrength {
    Hard,
    Soft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AdversarialFindingKind {
    Contradiction,
    Counterexample,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AdversarialFinding {
    pub id: String,
    pub detector: String,
    pub kind: AdversarialFindingKind,
    pub strength: FindingStrength,
    pub claim_id: String,
    pub proposition: Proposition,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationConclusion {
    Supported,
    Contradicted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VerificationReceipt {
    pub id: String,
    pub verifier: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_statement: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposition: Option<Proposition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_id: Option<String>,
    pub conclusion: VerificationConclusion,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HarnessInput {
    pub task: String,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    /// Harness-owned propositions that formalize hypotheses explicitly posed by the task.
    /// Candidates cannot add or mutate these targets.
    #[serde(default)]
    pub hypotheses: Vec<Proposition>,
    /// Harness-owned premises that the task explicitly permits reasoning to assume.
    /// These are input context, not candidate-authored epistemic labels.
    #[serde(default)]
    pub assumptions: Vec<Proposition>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReasoningCandidate {
    #[serde(default)]
    pub claims: Vec<CandidateClaim>,
    #[serde(default)]
    pub inferences: Vec<Inference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CandidateDiagnostic {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReasoningArtifact {
    pub task: String,
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    #[serde(default)]
    pub hypotheses: Vec<Proposition>,
    #[serde(default)]
    pub assumptions: Vec<Proposition>,
    #[serde(default)]
    pub candidate_diagnostics: Vec<CandidateDiagnostic>,
    #[serde(default)]
    pub verification_receipts: Vec<VerificationReceipt>,
    #[serde(default)]
    pub adversarial_findings: Vec<AdversarialFinding>,
    #[serde(default)]
    pub assumption_findings: Vec<crate::AssumptionFinding>,
    #[serde(default)]
    pub claims: Vec<Claim>,
    #[serde(default)]
    pub inferences: Vec<Inference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Accept,
    Reject,
    Unknown,
}
