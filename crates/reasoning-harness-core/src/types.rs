use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScopeCoverage {
    Any,
    Values { values: BTreeSet<String> },
}

pub type ApplicabilityScope = BTreeMap<String, ScopeCoverage>;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TemporalValidity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_from_unix_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_until_unix_seconds: Option<i64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal: Option<TemporalValidity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<ApplicabilityScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_class: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Evidence {
    pub id: String,
    pub source: String,
    pub observation: String,
    #[serde(default)]
    pub facts: BTreeMap<String, String>,
    /// Harness-owned qualification metadata. Candidates cannot create or modify evidence.
    #[serde(default)]
    pub metadata: EvidenceMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct Proposition {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceAuthorityPolicy {
    /// Domain-neutral authority ordering supplied by the harness. Higher ranks are stronger.
    #[serde(default)]
    pub ranks: BTreeMap<String, u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceRequirement {
    pub proposition: Proposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of_unix_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<ApplicabilityScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_authority_class: Option<String>,
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
    /// Harness-owned qualification requirements for proposition evidence.
    #[serde(default)]
    pub evidence_requirements: Vec<EvidenceRequirement>,
    /// Domain-neutral mapping from provenance classes to comparable authority ranks.
    #[serde(default)]
    pub authority_policy: EvidenceAuthorityPolicy,
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
    pub evidence_requirements: Vec<EvidenceRequirement>,
    #[serde(default)]
    pub authority_policy: EvidenceAuthorityPolicy,
    #[serde(default)]
    pub candidate_diagnostics: Vec<CandidateDiagnostic>,
    #[serde(default)]
    pub verification_receipts: Vec<VerificationReceipt>,
    #[serde(default)]
    pub adversarial_findings: Vec<AdversarialFinding>,
    #[serde(default)]
    pub assumption_findings: Vec<crate::AssumptionFinding>,
    #[serde(default)]
    pub evidence_qualification_findings: Vec<crate::EvidenceQualificationFinding>,
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
