//! Additive, opt-in, source-local compatibility metadata for #490.
//! Never changes persisted v1 conflict flags, canonical prose, citations, or verification.
//! A trusted *host* must review source equivalence; model output is not a reviewer.

use std::collections::{BTreeMap, BTreeSet};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    Evidence, ReasoningArtifact, SourceAttributedClaim, SourceAttributionBinding,
    SourceAttributionFinalization, SourceAttributionFinalizationStatus,
    SourceAttributionTargetPolicy, SourceAttributionTransformKind,
    finalize_source_attributed_answer, validate_source_attribution_state,
};

pub const SOURCE_RECONCILIATION_VIEW_CONTRACT_ID: &str = "harness-source-reconciliation-view-v1";
pub const SOURCE_RECONCILIATION_REVIEW_CONTRACT_ID: &str =
    "harness-source-compatibility-trusted-review-v1";

/// Independently versioned, opt-in presentation; never changes old view/v1 data.
pub const SOURCE_RECONCILIATION_TARGET_ANSWER_CONTRACT_ID: &str =
    "harness-source-reconciliation-target-answer-v1";

/// Capability from a trusted host, never recovered from a persisted review or model output.
/// The host, not this library, is responsible for authenticating its reviewer policy.
#[derive(Debug, Clone)]
pub struct TrustedSourceReviewAuthority {
    policy_id: String,
}

impl TrustedSourceReviewAuthority {
    pub fn new(policy_id: impl Into<String>) -> Result<Self, SourceReconciliationError> {
        let policy_id = policy_id.into();
        if policy_id.trim().is_empty() || policy_id.trim() != policy_id {
            return Err(SourceReconciliationError::InvalidAuthority);
        }
        Ok(Self { policy_id })
    }
    pub fn policy_id(&self) -> &str {
        &self.policy_id
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceReviewAnchor {
    pub target_policy: SourceAttributionTargetPolicy,
    pub claim: SourceAttributedClaim,
    pub bindings: Vec<SourceAttributionBinding>,
    /// The full admitted source records (text, scope/time, provenance) are
    /// bound to the review to reject tampering after session reload.
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrustedSourceCompatibilityReview {
    pub contract_id: String,
    pub reviewer_policy_id: String,
    pub first: SourceReviewAnchor,
    pub second: SourceReviewAnchor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SourceReconciliationStatus {
    Qualified,
    ReviewedCompatible,
    Conflict,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceReconciliationView {
    pub contract_id: String,
    pub status: SourceReconciliationStatus,
    /// Original, immutable source-local prose and citations; v1 Conflict is preserved.
    pub original: SourceAttributionFinalization,
    pub reviewed_compatible_target_ids: Vec<String>,
    pub remaining_conflict_target_ids: Vec<String>,
    pub accepted_review_policy_ids: Vec<String>,
}

/// One target's exact source-qualified output, without invented paraphrases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceReconciliationTargetAnswer {
    pub target_id: String,
    /// Additional reviewed-compatible metadata; does not promote truth.
    pub status: SourceReconciliationStatus,
    /// The *original* v1 source-specific prose and citations for this target.
    pub original: SourceAttributionFinalization,
}

/// Mixed turns no longer hide safe target-local compatible wording behind
/// a global Conflict label. Preserves the entire original global finalization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceReconciliationTargetPresentation {
    pub contract_id: String,
    pub original: SourceAttributionFinalization,
    pub target_answers: Vec<SourceReconciliationTargetAnswer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SourceReconciliationError {
    #[error("reviewer authority is invalid or missing")]
    InvalidAuthority,
    #[error("source attribution is invalid: {0}")]
    InvalidSource(String),
    #[error("claim does not exist: {0}")]
    UnknownClaim(String),
    #[error("review crosses target or hard-verification boundary")]
    WrongTargetOrHardVerification,
    #[error("review requires distinct exact source-local quotes")]
    NotDistinctSourceQuotes,
    #[error("obviously incompatible polarity or numeric values")]
    DeterministicConflict,
    #[error("evidence scope, effective time, authority or source version differs")]
    DifferentEvidenceContext,
    #[error("duplicate or extraneous review")]
    DuplicateOrExtraneousReview,
    #[error("persisted review cannot authenticate its own reviewer")]
    UntrustedReview,
    #[error("persisted review no longer matches the exact source snapshot")]
    StaleOrTamperedReview,
}

fn validate(artifact: &ReasoningArtifact) -> Result<(), SourceReconciliationError> {
    if let Some(issue) = validate_source_attribution_state(artifact).first() {
        return Err(SourceReconciliationError::InvalidSource(format!(
            "{}: {}",
            issue.code, issue.message
        )));
    }
    Ok(())
}

pub fn capture_source_review_anchor(
    artifact: &ReasoningArtifact,
    claim_id: &str,
) -> Result<SourceReviewAnchor, SourceReconciliationError> {
    validate(artifact)?;
    let claim = artifact
        .source_attribution
        .claims
        .iter()
        .find(|claim| claim.id == claim_id)
        .ok_or_else(|| SourceReconciliationError::UnknownClaim(claim_id.to_owned()))?
        .clone();
    let target_policy = artifact
        .source_attribution
        .targets
        .iter()
        .find(|policy| policy.target_id == claim.target_id)
        .ok_or(SourceReconciliationError::WrongTargetOrHardVerification)?
        .clone();
    if target_policy.hard_verification_required {
        return Err(SourceReconciliationError::WrongTargetOrHardVerification);
    }
    let bindings = claim
        .binding_ids
        .iter()
        .map(|id| {
            artifact
                .source_attribution
                .bindings
                .iter()
                .find(|binding| binding.id == *id && binding.target_id == claim.target_id)
                .cloned()
                .ok_or_else(|| {
                    SourceReconciliationError::InvalidSource(format!("missing binding {id}"))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let evidence = bindings
        .iter()
        .map(|binding: &SourceAttributionBinding| {
            artifact
                .evidence
                .iter()
                .find(|item| item.id == binding.evidence_id)
                .cloned()
                .ok_or_else(|| {
                    SourceReconciliationError::InvalidSource(format!(
                        "missing evidence {}",
                        binding.evidence_id
                    ))
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SourceReviewAnchor {
        target_policy,
        claim,
        bindings,
        evidence,
    })
}

fn tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(str::to_ascii_lowercase)
        .collect()
}

fn numbers(value: &str) -> BTreeSet<String> {
    tokens(value)
        .into_iter()
        .filter(|word| word.chars().any(|ch| ch.is_ascii_digit()))
        .collect()
}

fn polarity_cues(value: &str) -> BTreeSet<String> {
    // Conservative hard safeguards; NOT a semantic inference oracle.
    const CUES: &[&str] = &[
        "not",
        "no",
        "never",
        "without",
        "disabled",
        "unavailable",
        "unsupported",
        "false",
        "denied",
        "failed",
        "cancelled",
        "canceled",
        "prohibited",
        "withdrawn",
        "removed",
        "discontinued",
        "formerly",
        "previously",
    ];
    tokens(value)
        .into_iter()
        .filter(|word| CUES.contains(&word.as_str()))
        .collect()
}

fn modality_cues(value: &str) -> BTreeSet<String> {
    // Do not let a bad reviewer silently convert possibility to certainty,
    // historical to current, or narrower to universal scope. Safe
    // synonyms (may/might/could) deliberately share an advisory marker.
    let words = tokens(value);
    let mut cues = BTreeSet::new();
    if words
        .iter()
        .any(|w| matches!(w.as_str(), "may" | "might" | "could" | "possibly"))
    {
        cues.insert("possibility".into());
    }
    if words.iter().any(|w| matches!(w.as_str(), "will" | "shall")) {
        cues.insert("future".into());
    }
    if words
        .iter()
        .any(|w| matches!(w.as_str(), "was" | "were" | "had"))
    {
        cues.insert("past".into());
    }
    if words.iter().any(|w| matches!(w.as_str(), "if" | "unless")) {
        cues.insert("conditional".into());
    }
    for token in [
        "must",
        "should",
        "some",
        "all",
        "only",
        "always",
        "now",
        "currently",
        "today",
    ] {
        if words.iter().any(|word| word == token) {
            cues.insert(token.into());
        }
    }
    cues
}

fn safe_pair(
    left: &SourceReviewAnchor,
    right: &SourceReviewAnchor,
) -> Result<(), SourceReconciliationError> {
    if left.claim.target_id != right.claim.target_id
        || left.target_policy != right.target_policy
        || left.target_policy.hard_verification_required
    {
        return Err(SourceReconciliationError::WrongTargetOrHardVerification);
    }
    if left.claim.id == right.claim.id
        || left.claim.statement.trim() == right.claim.statement.trim()
        || left.claim.transform.kind != SourceAttributionTransformKind::ExactQuote
        || right.claim.transform.kind != SourceAttributionTransformKind::ExactQuote
        || left.bindings.len() != 1
        || right.bindings.len() != 1
    {
        return Err(SourceReconciliationError::NotDistinctSourceQuotes);
    }
    if left.evidence[0].metadata != right.evidence[0].metadata
        || left.bindings[0].source_version != right.bindings[0].source_version
    {
        return Err(SourceReconciliationError::DifferentEvidenceContext);
    }
    if numbers(&left.claim.statement) != numbers(&right.claim.statement)
        || polarity_cues(&left.claim.statement) != polarity_cues(&right.claim.statement)
        || modality_cues(&left.claim.statement) != modality_cues(&right.claim.statement)
    {
        return Err(SourceReconciliationError::DeterministicConflict);
    }
    Ok(())
}

/// This records the trusted host's human/oracle determination; it does not
/// discover semantic equivalence. Calling it with a model's unverified
/// equivalence claim would violate the caller-side authority contract.
pub fn record_trusted_source_equivalence(
    artifact: &ReasoningArtifact,
    authority: &TrustedSourceReviewAuthority,
    first_claim_id: &str,
    second_claim_id: &str,
) -> Result<TrustedSourceCompatibilityReview, SourceReconciliationError> {
    let first = capture_source_review_anchor(artifact, first_claim_id)?;
    let second = capture_source_review_anchor(artifact, second_claim_id)?;
    safe_pair(&first, &second)?;
    Ok(TrustedSourceCompatibilityReview {
        contract_id: SOURCE_RECONCILIATION_REVIEW_CONTRACT_ID.into(),
        reviewer_policy_id: authority.policy_id().into(),
        first,
        second,
    })
}

fn pair_key(left: &str, right: &str) -> (String, String) {
    if left < right {
        (left.into(), right.into())
    } else {
        (right.into(), left.into())
    }
}

/// Produces new metadata only. The v1 artifact and its original status,
/// text and citations remain byte-exactly the same, including on replay.
pub fn reconcile_source_attributed_answer(
    artifact: &ReasoningArtifact,
    target_ids: &[String],
    authority: Option<&TrustedSourceReviewAuthority>,
    reviews: &[TrustedSourceCompatibilityReview],
) -> Result<SourceReconciliationView, SourceReconciliationError> {
    validate(artifact)?;
    // Reject duplicate target selection that would otherwise duplicate the
    // legacy citations in a new overlay result.
    if target_ids.iter().collect::<BTreeSet<_>>().len() != target_ids.len() || reviews.len() > 1024
    {
        return Err(SourceReconciliationError::DuplicateOrExtraneousReview);
    }
    let original = finalize_source_attributed_answer(artifact, target_ids)
        .map_err(|err| SourceReconciliationError::InvalidSource(err.to_string()))?;
    if !reviews.is_empty() && authority.is_none() {
        return Err(SourceReconciliationError::InvalidAuthority);
    }
    let mut approved = BTreeMap::<String, BTreeSet<(String, String)>>::new();
    for review in reviews {
        let trusted = authority.ok_or(SourceReconciliationError::InvalidAuthority)?;
        if review.contract_id != SOURCE_RECONCILIATION_REVIEW_CONTRACT_ID
            || review.reviewer_policy_id != trusted.policy_id()
        {
            return Err(SourceReconciliationError::UntrustedReview);
        }
        let first = capture_source_review_anchor(artifact, &review.first.claim.id)?;
        let second = capture_source_review_anchor(artifact, &review.second.claim.id)?;
        if first != review.first || second != review.second {
            return Err(SourceReconciliationError::StaleOrTamperedReview);
        }
        if !target_ids.contains(&first.claim.target_id) {
            return Err(SourceReconciliationError::DuplicateOrExtraneousReview);
        }
        safe_pair(&first, &second)?;
        if !approved
            .entry(first.claim.target_id.clone())
            .or_default()
            .insert(pair_key(&first.claim.id, &second.claim.id))
        {
            return Err(SourceReconciliationError::DuplicateOrExtraneousReview);
        }
    }
    let mut compatible = Vec::new();
    let mut remaining = Vec::new();
    for target_id in &original.conflict_target_ids {
        let claims = artifact
            .source_attribution
            .claims
            .iter()
            .filter(|claim| claim.target_id == *target_id)
            .collect::<Vec<_>>();
        // With no host review there is no reason to allocate a potentially
        // quadratic pair matrix for a large target. Large reviewed groups
        // require a separate bounded successor design.
        if !approved.contains_key(target_id) || claims.len() > 16 {
            remaining.push(target_id.clone());
            continue;
        }
        let actual_pairs = claims
            .iter()
            .enumerate()
            .flat_map(|(i, left)| {
                claims
                    .iter()
                    .skip(i + 1)
                    .filter(move |right| left.statement.trim() != right.statement.trim())
                    .map(move |right| pair_key(&left.id, &right.id))
            })
            .collect::<BTreeSet<_>>();
        // Pairwise complete review is required: NO transitive closure or
        // automatic suppression of a source-local contradiction.
        if !actual_pairs.is_empty()
            && approved
                .get(target_id)
                .is_some_and(|reviews| *reviews == actual_pairs)
        {
            compatible.push(target_id.clone());
        } else {
            remaining.push(target_id.clone());
        }
    }
    let status = match original.status {
        SourceAttributionFinalizationStatus::Unresolved => SourceReconciliationStatus::Unresolved,
        SourceAttributionFinalizationStatus::Qualified => SourceReconciliationStatus::Qualified,
        SourceAttributionFinalizationStatus::Conflict if remaining.is_empty() => {
            SourceReconciliationStatus::ReviewedCompatible
        }
        SourceAttributionFinalizationStatus::Conflict => SourceReconciliationStatus::Conflict,
    };
    Ok(SourceReconciliationView {
        contract_id: SOURCE_RECONCILIATION_VIEW_CONTRACT_ID.into(),
        status,
        original,
        reviewed_compatible_target_ids: compatible,
        remaining_conflict_target_ids: remaining,
        accepted_review_policy_ids: if reviews.is_empty() {
            vec![]
        } else {
            vec![
                authority
                    .expect("review requires authority")
                    .policy_id()
                    .into(),
            ]
        },
    })
}

/// Opt-in target-local presentation of already authorized source reviews.
///
/// This never creates or approves a review. It rechecks every exact source
/// snapshot and policy through the existing reconciliation gate, then emits
/// only the legacy canonical per-source text and original citations. Mixed
/// turns can expose which target has reviewed-compatible wording, without
/// clearing the global legacy Conflict or contaminating other targets.
pub fn reconcile_source_attributed_targets(
    artifact: &ReasoningArtifact,
    target_ids: &[String],
    authority: Option<&TrustedSourceReviewAuthority>,
    reviews: &[TrustedSourceCompatibilityReview],
) -> Result<SourceReconciliationTargetPresentation, SourceReconciliationError> {
    let global = reconcile_source_attributed_answer(artifact, target_ids, authority, reviews)?;
    let reviewed = global
        .reviewed_compatible_target_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let unresolved_conflicts = global
        .remaining_conflict_target_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut target_answers = Vec::with_capacity(target_ids.len());
    let mut combined_citations = Vec::new();
    let mut combined_claim_ids = Vec::new();

    for target_id in target_ids {
        // The global gate has already authenticated every review and checked
        // the exact source/target binding. Do not repeat deep review validation
        // per target or infer approval from model-provided data.
        let original = finalize_source_attributed_answer(artifact, std::slice::from_ref(target_id))
            .map_err(|error| SourceReconciliationError::InvalidSource(error.to_string()))?;
        let status = match original.status {
            SourceAttributionFinalizationStatus::Qualified => SourceReconciliationStatus::Qualified,
            SourceAttributionFinalizationStatus::Unresolved => {
                SourceReconciliationStatus::Unresolved
            }
            SourceAttributionFinalizationStatus::Conflict
                if reviewed.contains(target_id.as_str())
                    && !unresolved_conflicts.contains(target_id.as_str()) =>
            {
                SourceReconciliationStatus::ReviewedCompatible
            }
            SourceAttributionFinalizationStatus::Conflict
                if unresolved_conflicts.contains(target_id.as_str())
                    && !reviewed.contains(target_id.as_str()) =>
            {
                SourceReconciliationStatus::Conflict
            }
            _ => {
                return Err(SourceReconciliationError::InvalidSource(
                    "target-local disposition differs from checked global review state".into(),
                ));
            }
        };
        combined_citations.extend(original.citations.iter().cloned());
        combined_claim_ids.extend(original.claim_ids.iter().cloned());
        target_answers.push(SourceReconciliationTargetAnswer {
            target_id: target_id.clone(),
            status,
            original,
        });
    }

    // Fail-closed if a change to either canonical finalizer ever causes
    // target-local citation/claim aggregation to diverge from legacy output.
    if combined_citations != global.original.citations
        || combined_claim_ids != global.original.claim_ids
    {
        return Err(SourceReconciliationError::InvalidSource(
            "target-local output no longer matches original citation order".into(),
        ));
    }
    Ok(SourceReconciliationTargetPresentation {
        contract_id: SOURCE_RECONCILIATION_TARGET_ANSWER_CONTRACT_ID.into(),
        original: global.original,
        target_answers,
    })
}
