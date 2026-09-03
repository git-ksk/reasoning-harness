use std::collections::{BTreeMap, BTreeSet, VecDeque};

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

use crate::{EpistemicState, Proposition, ReasoningArtifact, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FinalClaimMode {
    Grounded,
    Uncertain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FinalAnswerClaim {
    pub proposition: Proposition,
    pub mode: FinalClaimMode,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FinalAnswerCandidate {
    pub text: String,
    #[serde(default)]
    pub factual_claims: Vec<FinalAnswerClaim>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalizationStatus {
    GroundedAnswer,
    QualifiedPartialAnswer,
    Unresolved,
    Abstain,
    RequiresVerification,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalizationResult {
    pub status: FinalizationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub factual_claims: usize,
    pub covered_claims: usize,
    pub factual_claim_coverage: f64,
    #[serde(default)]
    pub uncovered_propositions: Vec<Proposition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FinalizationPolicy {
    pub allow_qualified_partial: bool,
}

impl Default for FinalizationPolicy {
    fn default() -> Self {
        Self {
            allow_qualified_partial: true,
        }
    }
}

pub fn final_answer_candidate_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(FinalAnswerCandidate))
        .expect("FinalAnswerCandidate schema must serialize")
}

pub trait FinalAnswerRenderer: Send + Sync {
    fn render(&self, artifact: &ReasoningArtifact, verdict: Verdict) -> FinalAnswerCandidate;
}

/// Deterministically render only harness-owned requested targets that are already authorized by the
/// final artifact. This is a recovery primitive for model renderers that omit or rename structured
/// claims after the harness has reached `Accept`; it never derives authority from model prose.
///
/// Recovery is intentionally all-or-nothing across the requested target set. Safe partial recovery is
/// a separate product policy concern: this helper refuses to present an apparently complete grounded
/// answer when any requested target lacks exact `Known`/`Supported` authority.
fn exact_target_authorized(artifact: &ReasoningArtifact, target: &Proposition) -> bool {
    let matching = artifact
        .claims
        .iter()
        .filter(|claim| claim.proposition.as_ref() == Some(target))
        .collect::<Vec<_>>();
    if matching.is_empty()
        || !matching.iter().all(|claim| {
            matches!(
                claim.state,
                EpistemicState::Known | EpistemicState::Supported
            )
        })
    {
        return false;
    }
    if artifact.verification_receipts.iter().any(|receipt| {
        receipt.proposition.as_ref() == Some(target)
            && receipt.conclusion == crate::VerificationConclusion::Contradicted
    }) {
        return false;
    }
    if artifact
        .evidence_qualification_findings
        .iter()
        .any(|finding| finding.proposition == *target)
    {
        return false;
    }
    !artifact.adversarial_findings.iter().any(|finding| {
        finding.proposition == *target && finding.strength == crate::FindingStrength::Hard
    })
}

fn canonical_verified_target_candidate(
    artifact: &ReasoningArtifact,
    targets: &[Proposition],
) -> Option<FinalAnswerCandidate> {
    if targets.is_empty() {
        return None;
    }

    let mut factual_claims = Vec::new();
    for target in targets {
        if factual_claims
            .iter()
            .any(|claim: &FinalAnswerClaim| claim.proposition == *target)
        {
            continue;
        }
        if !exact_target_authorized(artifact, target) {
            return None;
        }
        factual_claims.push(FinalAnswerClaim {
            proposition: target.clone(),
            mode: FinalClaimMode::Grounded,
        });
    }

    if factual_claims.is_empty() {
        return None;
    }
    let text = factual_claims
        .iter()
        .map(|claim| format!("{} = {}", claim.proposition.key, claim.proposition.value))
        .collect::<Vec<_>>()
        .join("; ");
    Some(FinalAnswerCandidate {
        text,
        factual_claims,
    })
}

/// Deterministically render only harness-owned requested targets that are already authorized by the
/// final artifact after the Harness has reached `Accept`. It never derives authority from model prose
/// or fuzzy proposition matching.
pub fn canonical_verified_target_answer(
    artifact: &ReasoningArtifact,
    verdict: Verdict,
    targets: &[Proposition],
) -> Option<FinalAnswerCandidate> {
    (verdict == Verdict::Accept)
        .then(|| canonical_verified_target_candidate(artifact, targets))
        .flatten()
}

/// Preserve exact requested targets as a target-only qualified partial answer when the artifact-global
/// verdict remains `Unknown` solely because other candidate state is unresolved. The global verdict is
/// not promoted. Every requested target must already be exact `Known`/`Supported`; target-local
/// qualification/adversarial/contradiction signals fail closed, and `Reject` is never eligible.
pub fn canonical_verified_target_partial_answer(
    artifact: &ReasoningArtifact,
    verdict: Verdict,
    targets: &[Proposition],
) -> Option<(FinalAnswerCandidate, FinalizationResult)> {
    if verdict != Verdict::Unknown
        || artifact
            .claims
            .iter()
            .any(|claim| claim.state == EpistemicState::Contradicted)
    {
        return None;
    }
    let has_non_target_unresolved = artifact.claims.iter().any(|claim| {
        !claim
            .proposition
            .as_ref()
            .is_some_and(|proposition| targets.contains(proposition))
            && matches!(
                claim.state,
                EpistemicState::Inferred | EpistemicState::Assumed | EpistemicState::Unknown
            )
    });
    if !has_non_target_unresolved {
        return None;
    }

    let mut candidate = canonical_verified_target_candidate(artifact, targets)?;
    let grounded = candidate.text.clone();
    candidate.text = format!(
        "verified partial: {grounded}; other generated claims remain unresolved and are omitted"
    );
    let factual_claims = candidate.factual_claims.len();
    let finalization = FinalizationResult {
        status: FinalizationStatus::QualifiedPartialAnswer,
        text: Some(candidate.text.clone()),
        factual_claims,
        covered_claims: factual_claims,
        factual_claim_coverage: 1.0,
        uncovered_propositions: vec![],
    };
    Some((candidate, finalization))
}

fn receipt_has_existing_evidence(
    artifact: &ReasoningArtifact,
    receipt: &crate::VerificationReceipt,
) -> bool {
    !receipt.evidence_ids.is_empty()
        && receipt.evidence_ids.iter().all(|evidence_id| {
            artifact
                .evidence
                .iter()
                .any(|evidence| evidence.id == *evidence_id)
        })
}

fn bound_receipt_evidence_ids(
    artifact: &ReasoningArtifact,
    claim: &crate::Claim,
    conclusion: crate::VerificationConclusion,
) -> BTreeSet<String> {
    artifact
        .verification_receipts
        .iter()
        .filter(|receipt| {
            receipt.conclusion == conclusion && crate::verification::receipt_matches(receipt, claim)
        })
        .flat_map(|receipt| receipt.evidence_ids.iter().cloned())
        .collect()
}

fn exact_directly_verified_target_claims(
    artifact: &ReasoningArtifact,
    target: &Proposition,
) -> Option<BTreeSet<String>> {
    if artifact
        .evidence_qualification_findings
        .iter()
        .any(|finding| finding.proposition == *target)
        || artifact.adversarial_findings.iter().any(|finding| {
            finding.proposition == *target && finding.strength == crate::FindingStrength::Hard
        })
    {
        return None;
    }

    let matching = artifact
        .claims
        .iter()
        .filter(|claim| claim.proposition.as_ref() == Some(target))
        .collect::<Vec<_>>();
    if matching.is_empty()
        || matching
            .iter()
            .any(|claim| claim.state != EpistemicState::Supported)
    {
        return None;
    }

    let mut ids = BTreeSet::new();
    for claim in matching {
        let directly_supported = artifact.verification_receipts.iter().any(|receipt| {
            receipt.conclusion == crate::VerificationConclusion::Supported
                && receipt.proposition.as_ref() == Some(target)
                && crate::verification::receipt_matches(receipt, claim)
                && receipt_has_existing_evidence(artifact, receipt)
        });
        let contradicted = artifact.verification_receipts.iter().any(|receipt| {
            receipt.conclusion == crate::VerificationConclusion::Contradicted
                && crate::verification::receipt_matches(receipt, claim)
                && receipt_has_existing_evidence(artifact, receipt)
        });
        if !directly_supported || contradicted {
            return None;
        }
        ids.insert(claim.id.clone());
    }
    Some(ids)
}

fn reject_problem_claims(
    artifact: &ReasoningArtifact,
    targets: &[Proposition],
) -> Option<(BTreeSet<String>, BTreeSet<String>)> {
    let target_keys = targets
        .iter()
        .map(|target| target.key.as_str())
        .collect::<BTreeSet<_>>();
    let mut problematic = BTreeSet::new();
    let mut contradicted = BTreeSet::new();

    for claim in &artifact.claims {
        if !matches!(
            claim.state,
            EpistemicState::Contradicted
                | EpistemicState::Assumed
                | EpistemicState::Unknown
                | EpistemicState::Inferred
        ) {
            continue;
        }
        let proposition = claim.proposition.as_ref()?;
        if targets.contains(proposition) || target_keys.contains(proposition.key.as_str()) {
            return None;
        }
        if claim.state == EpistemicState::Contradicted {
            let has_direct_contradiction = artifact.verification_receipts.iter().any(|receipt| {
                receipt.conclusion == crate::VerificationConclusion::Contradicted
                    && receipt.proposition.as_ref() == Some(proposition)
                    && crate::verification::receipt_matches(receipt, claim)
                    && receipt_has_existing_evidence(artifact, receipt)
            });
            if !has_direct_contradiction {
                return None;
            }
            contradicted.insert(claim.id.clone());
        }
        problematic.insert(claim.id.clone());
    }

    (!contradicted.is_empty()).then_some((problematic, contradicted))
}

fn inference_component_intersects(
    artifact: &ReasoningArtifact,
    starts: &BTreeSet<String>,
    blocked: &BTreeSet<String>,
) -> bool {
    let known_claim_ids = artifact
        .claims
        .iter()
        .map(|claim| claim.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut adjacency: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for inference in &artifact.inferences {
        if !known_claim_ids.contains(inference.conclusion_claim_id.as_str())
            || inference
                .premise_claim_ids
                .iter()
                .any(|premise| !known_claim_ids.contains(premise.as_str()))
        {
            return true;
        }
        for premise in &inference.premise_claim_ids {
            adjacency
                .entry(premise.as_str())
                .or_default()
                .insert(inference.conclusion_claim_id.as_str());
            adjacency
                .entry(inference.conclusion_claim_id.as_str())
                .or_default()
                .insert(premise.as_str());
        }
    }

    let mut visited = BTreeSet::new();
    let mut queue = starts.iter().map(String::as_str).collect::<VecDeque<_>>();
    while let Some(current) = queue.pop_front() {
        if !visited.insert(current) {
            continue;
        }
        if blocked.contains(current) {
            return true;
        }
        if let Some(neighbors) = adjacency.get(current) {
            queue.extend(neighbors.iter().copied());
        }
    }
    false
}

fn claim_evidence_footprint(
    artifact: &ReasoningArtifact,
    claim_ids: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut evidence = BTreeSet::new();
    for claim in artifact
        .claims
        .iter()
        .filter(|claim| claim_ids.contains(&claim.id))
    {
        evidence.extend(claim.evidence_ids.iter().cloned());
        evidence.extend(bound_receipt_evidence_ids(
            artifact,
            claim,
            crate::VerificationConclusion::Supported,
        ));
        evidence.extend(bound_receipt_evidence_ids(
            artifact,
            claim,
            crate::VerificationConclusion::Contradicted,
        ));
    }
    evidence
}

/// Expose exact requested targets as a qualified target-only result even when the artifact-global
/// verdict is `Reject`, but only when the rejection is demonstrably isolated from those targets in
/// the typed artifact. Each target must have direct trusted `Supported` verification. Every
/// contradicted blocker must itself be directly verification-bound, use a different proposition key,
/// share no evidence with the target, and have no typed inference/dependency path to the target.
/// Unknown/unresolved non-target state is subject to the same structural isolation check. The global
/// `Reject` is preserved and the returned result is explicitly `QualifiedPartialAnswer`.
pub fn canonical_verified_target_reject_partial_answer(
    artifact: &ReasoningArtifact,
    verdict: Verdict,
    targets: &[Proposition],
) -> Option<(FinalAnswerCandidate, FinalizationResult)> {
    if verdict != Verdict::Reject || targets.is_empty() {
        return None;
    }

    let mut target_claim_ids = BTreeSet::new();
    for target in targets {
        target_claim_ids.extend(exact_directly_verified_target_claims(artifact, target)?);
    }
    let (problematic_claim_ids, _contradicted_claim_ids) =
        reject_problem_claims(artifact, targets)?;

    if inference_component_intersects(artifact, &target_claim_ids, &problematic_claim_ids) {
        return None;
    }
    let target_evidence = claim_evidence_footprint(artifact, &target_claim_ids);
    let problem_evidence = claim_evidence_footprint(artifact, &problematic_claim_ids);
    if !target_evidence.is_disjoint(&problem_evidence) {
        return None;
    }

    let mut candidate = canonical_verified_target_candidate(artifact, targets)?;
    let grounded = candidate.text.clone();
    candidate.text = format!(
        "verified target only: {grounded}; full reasoning artifact remains rejected because structurally independent non-target state was contradicted"
    );
    let factual_claims = candidate.factual_claims.len();
    let finalization = FinalizationResult {
        status: FinalizationStatus::QualifiedPartialAnswer,
        text: Some(candidate.text.clone()),
        factual_claims,
        covered_claims: factual_claims,
        factual_claim_coverage: 1.0,
        uncovered_propositions: vec![],
    };
    Some((candidate, finalization))
}

/// Recover an exact requested target when a stochastic renderer downgraded that same exact
/// proposition to `uncertain` despite existing Harness-owned authority. The renderer output only
/// selects this recovery path; it never supplies authority. Exact proposition identity, artifact
/// authority, qualification/adversarial checks, and the existing Accept/Unknown recovery boundaries
/// remain authoritative. `Reject` is never recovered here.
pub fn recover_verified_target_renderer_downgrade(
    artifact: &ReasoningArtifact,
    verdict: Verdict,
    targets: &[Proposition],
    rendered: &FinalAnswerCandidate,
    finalization: &FinalizationResult,
) -> Option<(FinalAnswerCandidate, FinalizationResult)> {
    if finalization.status != FinalizationStatus::QualifiedPartialAnswer {
        return None;
    }

    let exact_requested_target_downgraded = rendered.factual_claims.iter().any(|claim| {
        claim.mode == FinalClaimMode::Uncertain && targets.contains(&claim.proposition)
    });
    if !exact_requested_target_downgraded {
        return None;
    }

    match verdict {
        Verdict::Accept => {
            let recovered = canonical_verified_target_answer(artifact, verdict, targets)?;
            let recovered_finalization = finalize_answer(
                artifact,
                verdict,
                recovered.clone(),
                FinalizationPolicy::default(),
            );
            (recovered_finalization.status == FinalizationStatus::GroundedAnswer)
                .then_some((recovered, recovered_finalization))
        }
        Verdict::Unknown => canonical_verified_target_partial_answer(artifact, verdict, targets),
        Verdict::Reject => None,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CanonicalFinalAnswerRenderer;

impl FinalAnswerRenderer for CanonicalFinalAnswerRenderer {
    fn render(&self, artifact: &ReasoningArtifact, verdict: Verdict) -> FinalAnswerCandidate {
        if verdict == Verdict::Reject {
            return FinalAnswerCandidate {
                text: "abstain: verified reasoning contains a contradiction".into(),
                factual_claims: vec![],
            };
        }

        let factual_claims = artifact
            .claims
            .iter()
            .filter_map(|claim| {
                let proposition = claim.proposition.clone()?;
                let mode = match claim.state {
                    EpistemicState::Known | EpistemicState::Supported => FinalClaimMode::Grounded,
                    EpistemicState::Inferred
                    | EpistemicState::Assumed
                    | EpistemicState::Unknown => FinalClaimMode::Uncertain,
                    EpistemicState::Contradicted => return None,
                };
                Some(FinalAnswerClaim { proposition, mode })
            })
            .collect::<Vec<_>>();

        let text = if factual_claims.is_empty() {
            "unresolved: no grounded factual proposition is available".into()
        } else {
            factual_claims
                .iter()
                .map(|claim| match claim.mode {
                    FinalClaimMode::Grounded => {
                        format!("{} = {}", claim.proposition.key, claim.proposition.value)
                    }
                    FinalClaimMode::Uncertain => format!(
                        "uncertain({} = {})",
                        claim.proposition.key, claim.proposition.value
                    ),
                })
                .collect::<Vec<_>>()
                .join("; ")
        };

        FinalAnswerCandidate {
            text,
            factual_claims,
        }
    }
}

pub fn finalize_answer(
    artifact: &ReasoningArtifact,
    verdict: Verdict,
    candidate: FinalAnswerCandidate,
    policy: FinalizationPolicy,
) -> FinalizationResult {
    if verdict == Verdict::Reject {
        return FinalizationResult {
            status: FinalizationStatus::Abstain,
            text: None,
            factual_claims: candidate.factual_claims.len(),
            covered_claims: 0,
            factual_claim_coverage: coverage(0, candidate.factual_claims.len()),
            uncovered_propositions: candidate
                .factual_claims
                .into_iter()
                .map(|claim| claim.proposition)
                .collect(),
        };
    }

    let mut covered_claims = 0usize;
    let mut uncovered_propositions = Vec::new();
    let mut has_uncertain = false;

    for final_claim in &candidate.factual_claims {
        let covered = artifact.claims.iter().any(|claim| {
            if claim.proposition.as_ref() != Some(&final_claim.proposition) {
                return false;
            }
            match final_claim.mode {
                FinalClaimMode::Grounded => matches!(
                    claim.state,
                    EpistemicState::Known | EpistemicState::Supported
                ),
                FinalClaimMode::Uncertain => matches!(
                    claim.state,
                    EpistemicState::Known
                        | EpistemicState::Supported
                        | EpistemicState::Inferred
                        | EpistemicState::Assumed
                        | EpistemicState::Unknown
                ),
            }
        });
        if covered {
            covered_claims += 1;
            has_uncertain |= final_claim.mode == FinalClaimMode::Uncertain;
        } else {
            uncovered_propositions.push(final_claim.proposition.clone());
        }
    }

    let factual_claims = candidate.factual_claims.len();
    let factual_claim_coverage = coverage(covered_claims, factual_claims);
    if !uncovered_propositions.is_empty() {
        return FinalizationResult {
            status: FinalizationStatus::RequiresVerification,
            text: None,
            factual_claims,
            covered_claims,
            factual_claim_coverage,
            uncovered_propositions,
        };
    }

    if factual_claims == 0 || verdict == Verdict::Unknown {
        let status = if factual_claims > 0 && has_uncertain && policy.allow_qualified_partial {
            FinalizationStatus::QualifiedPartialAnswer
        } else {
            FinalizationStatus::Unresolved
        };
        return FinalizationResult {
            status,
            text: (status == FinalizationStatus::QualifiedPartialAnswer).then_some(candidate.text),
            factual_claims,
            covered_claims,
            factual_claim_coverage,
            uncovered_propositions,
        };
    }

    let status = if has_uncertain {
        if policy.allow_qualified_partial {
            FinalizationStatus::QualifiedPartialAnswer
        } else {
            FinalizationStatus::Abstain
        }
    } else {
        FinalizationStatus::GroundedAnswer
    };
    FinalizationResult {
        status,
        text: matches!(
            status,
            FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
        )
        .then_some(candidate.text),
        factual_claims,
        covered_claims,
        factual_claim_coverage,
        uncovered_propositions,
    }
}

fn coverage(covered: usize, total: usize) -> f64 {
    if total == 0 {
        1.0
    } else {
        covered as f64 / total as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Claim, Evidence, EvidenceMetadata, Inference, VerificationConclusion, VerificationReceipt,
    };

    fn artifact(state: EpistemicState) -> ReasoningArtifact {
        ReasoningArtifact {
            claims: vec![Claim {
                id: "c1".into(),
                statement: "feature.enabled = true".into(),
                state,
                proposition: Some(Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                }),
                evidence_ids: vec![],
            }],
            ..Default::default()
        }
    }

    #[test]
    fn canonical_verified_target_recovery_uses_exact_authorized_targets() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let recovered = canonical_verified_target_answer(
            &artifact(EpistemicState::Supported),
            Verdict::Accept,
            std::slice::from_ref(&target),
        )
        .expect("supported exact target should be recoverable");
        assert_eq!(recovered.text, "feature.enabled = true");
        assert_eq!(
            recovered.factual_claims,
            vec![FinalAnswerClaim {
                proposition: target,
                mode: FinalClaimMode::Grounded,
            }]
        );
    }

    #[test]
    fn canonical_verified_target_recovery_rejects_key_drift_and_unknown_verdict() {
        let exact = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let drifted = Proposition {
            key: "feature enabled".into(),
            value: "true".into(),
        };
        let current = artifact(EpistemicState::Supported);
        assert!(
            canonical_verified_target_answer(
                &current,
                Verdict::Accept,
                std::slice::from_ref(&drifted),
            )
            .is_none()
        );
        assert!(
            canonical_verified_target_answer(
                &current,
                Verdict::Unknown,
                std::slice::from_ref(&exact),
            )
            .is_none()
        );
    }

    #[test]
    fn canonical_verified_target_recovery_requires_every_requested_target() {
        let exact = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let missing = Proposition {
            key: "feature.region".into(),
            value: "us-east-1".into(),
        };
        assert!(
            canonical_verified_target_answer(
                &artifact(EpistemicState::Supported),
                Verdict::Accept,
                &[exact, missing],
            )
            .is_none()
        );
    }

    #[test]
    fn canonical_verified_target_recovery_deduplicates_exact_targets() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let recovered = canonical_verified_target_answer(
            &artifact(EpistemicState::Known),
            Verdict::Accept,
            &[target.clone(), target],
        )
        .expect("known exact target should be recoverable");
        assert_eq!(recovered.factual_claims.len(), 1);
    }

    #[test]
    fn target_scoped_partial_preserves_verified_target_without_changing_unknown_verdict() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let mut current = artifact(EpistemicState::Supported);
        current.claims.push(Claim {
            id: "extra".into(),
            statement: "unrelated detail".into(),
            state: EpistemicState::Unknown,
            proposition: Some(Proposition {
                key: "feature.owner".into(),
                value: "team-a".into(),
            }),
            evidence_ids: vec![],
        });
        let (candidate, finalization) = canonical_verified_target_partial_answer(
            &current,
            Verdict::Unknown,
            std::slice::from_ref(&target),
        )
        .expect("unrelated unresolved claim must not erase an exact authorized target");
        assert_eq!(candidate.factual_claims.len(), 1);
        assert_eq!(candidate.factual_claims[0].proposition, target);
        assert_eq!(
            finalization.status,
            FinalizationStatus::QualifiedPartialAnswer
        );
        assert_eq!(finalization.factual_claim_coverage, 1.0);
    }

    #[test]
    fn renderer_downgrade_recovery_grounds_exact_authorized_target_under_unknown() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let mut current = artifact(EpistemicState::Supported);
        current.claims.push(Claim {
            id: "extra".into(),
            statement: "unrelated detail".into(),
            state: EpistemicState::Unknown,
            proposition: Some(Proposition {
                key: "feature.owner".into(),
                value: "team-a".into(),
            }),
            evidence_ids: vec![],
        });
        let rendered = FinalAnswerCandidate {
            text: "The feature may be enabled.".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: target.clone(),
                mode: FinalClaimMode::Uncertain,
            }],
        };
        let initial = finalize_answer(
            &current,
            Verdict::Unknown,
            rendered.clone(),
            FinalizationPolicy::default(),
        );
        assert_eq!(initial.status, FinalizationStatus::QualifiedPartialAnswer);

        let (recovered, recovered_finalization) = recover_verified_target_renderer_downgrade(
            &current,
            Verdict::Unknown,
            std::slice::from_ref(&target),
            &rendered,
            &initial,
        )
        .expect("exact authorized target downgrade should be recoverable");

        assert_eq!(
            recovered.factual_claims,
            vec![FinalAnswerClaim {
                proposition: target,
                mode: FinalClaimMode::Grounded,
            }]
        );
        assert_eq!(
            recovered_finalization.status,
            FinalizationStatus::QualifiedPartialAnswer
        );
        assert_eq!(recovered_finalization.factual_claim_coverage, 1.0);
    }

    #[test]
    fn renderer_downgrade_recovery_promotes_accept_rendering_only_from_artifact_authority() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let current = artifact(EpistemicState::Supported);
        let rendered = FinalAnswerCandidate {
            text: "The feature may be enabled.".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: target.clone(),
                mode: FinalClaimMode::Uncertain,
            }],
        };
        let initial = finalize_answer(
            &current,
            Verdict::Accept,
            rendered.clone(),
            FinalizationPolicy::default(),
        );
        let (_, recovered_finalization) = recover_verified_target_renderer_downgrade(
            &current,
            Verdict::Accept,
            std::slice::from_ref(&target),
            &rendered,
            &initial,
        )
        .expect("Accept target with exact authority should recover");
        assert_eq!(
            recovered_finalization.status,
            FinalizationStatus::GroundedAnswer
        );
    }

    #[test]
    fn renderer_downgrade_recovery_rejects_fuzzy_or_unrequested_renderer_claims() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let current = artifact(EpistemicState::Supported);
        let rendered = FinalAnswerCandidate {
            text: "The feature may be enabled.".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: Proposition {
                    key: "feature enabled".into(),
                    value: "true".into(),
                },
                mode: FinalClaimMode::Uncertain,
            }],
        };
        let initial = FinalizationResult {
            status: FinalizationStatus::QualifiedPartialAnswer,
            text: Some(rendered.text.clone()),
            factual_claims: 1,
            covered_claims: 1,
            factual_claim_coverage: 1.0,
            uncovered_propositions: vec![],
        };
        assert!(
            recover_verified_target_renderer_downgrade(
                &current,
                Verdict::Accept,
                std::slice::from_ref(&target),
                &rendered,
                &initial,
            )
            .is_none()
        );
    }

    #[test]
    fn renderer_downgrade_recovery_never_overrides_reject_or_target_qualification() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let rendered = FinalAnswerCandidate {
            text: "The feature may be enabled.".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: target.clone(),
                mode: FinalClaimMode::Uncertain,
            }],
        };
        let qualified = FinalizationResult {
            status: FinalizationStatus::QualifiedPartialAnswer,
            text: Some(rendered.text.clone()),
            factual_claims: 1,
            covered_claims: 1,
            factual_claim_coverage: 1.0,
            uncovered_propositions: vec![],
        };
        assert!(
            recover_verified_target_renderer_downgrade(
                &artifact(EpistemicState::Supported),
                Verdict::Reject,
                std::slice::from_ref(&target),
                &rendered,
                &qualified,
            )
            .is_none()
        );

        let mut current = artifact(EpistemicState::Supported);
        current.claims.push(Claim {
            id: "extra".into(),
            statement: "unrelated detail".into(),
            state: EpistemicState::Unknown,
            proposition: Some(Proposition {
                key: "feature.owner".into(),
                value: "team-a".into(),
            }),
            evidence_ids: vec![],
        });
        current
            .evidence_qualification_findings
            .push(crate::EvidenceQualificationFinding {
                id: "qualification".into(),
                detector: "test".into(),
                kind: crate::EvidenceQualificationFindingKind::MissingMetadata,
                reason: crate::EvidenceQualificationFindingReason::MissingTemporalMetadata,
                strength: crate::FindingStrength::Soft,
                proposition: target.clone(),
                evidence_ids: vec![],
                message: "missing target metadata".into(),
            });
        assert!(
            recover_verified_target_renderer_downgrade(
                &current,
                Verdict::Unknown,
                std::slice::from_ref(&target),
                &rendered,
                &qualified,
            )
            .is_none()
        );
    }

    fn verified_reject_artifact() -> (ReasoningArtifact, Proposition) {
        let target = Proposition {
            key: "service.failover_region".into(),
            value: "eu-west-1".into(),
        };
        let blocker = Proposition {
            key: "telemetry.mode".into(),
            value: "healthy".into(),
        };
        let target_evidence = Evidence {
            id: "target-e".into(),
            source: "target fixture".into(),
            observation: "service.failover_region=eu-west-1".into(),
            facts: BTreeMap::from([("service.failover_region".into(), "eu-west-1".into())]),
            metadata: EvidenceMetadata::default(),
        };
        let blocker_evidence = Evidence {
            id: "blocker-e".into(),
            source: "blocker fixture".into(),
            observation: "telemetry.mode=degraded".into(),
            facts: BTreeMap::from([("telemetry.mode".into(), "degraded".into())]),
            metadata: EvidenceMetadata::default(),
        };
        let artifact = ReasoningArtifact {
            task: "resolve failover region".into(),
            evidence: vec![target_evidence, blocker_evidence],
            hypotheses: vec![target.clone()],
            claims: vec![
                Claim {
                    id: "target".into(),
                    statement: "service.failover_region = eu-west-1".into(),
                    state: EpistemicState::Supported,
                    proposition: Some(target.clone()),
                    evidence_ids: vec!["target-e".into()],
                },
                Claim {
                    id: "blocker".into(),
                    statement: "telemetry.mode = healthy".into(),
                    state: EpistemicState::Contradicted,
                    proposition: Some(blocker.clone()),
                    evidence_ids: vec![],
                },
            ],
            verification_receipts: vec![
                VerificationReceipt {
                    id: "target-supported".into(),
                    verifier: "fixture".into(),
                    claim_statement: None,
                    proposition: Some(target.clone()),
                    claim_id: Some("target".into()),
                    conclusion: VerificationConclusion::Supported,
                    evidence_ids: vec!["target-e".into()],
                },
                VerificationReceipt {
                    id: "blocker-contradicted".into(),
                    verifier: "fixture".into(),
                    claim_statement: None,
                    proposition: Some(blocker),
                    claim_id: Some("blocker".into()),
                    conclusion: VerificationConclusion::Contradicted,
                    evidence_ids: vec!["blocker-e".into()],
                },
            ],
            ..Default::default()
        };
        (artifact, target)
    }

    #[test]
    fn reject_target_partial_exposes_directly_verified_target_when_blocker_is_structurally_unrelated()
     {
        let (current, target) = verified_reject_artifact();
        let (candidate, finalization) = canonical_verified_target_reject_partial_answer(
            &current,
            Verdict::Reject,
            std::slice::from_ref(&target),
        )
        .expect("directly verified target should survive an isolated non-target rejection");

        assert_eq!(
            candidate.factual_claims,
            vec![FinalAnswerClaim {
                proposition: target,
                mode: FinalClaimMode::Grounded,
            }]
        );
        assert_eq!(
            finalization.status,
            FinalizationStatus::QualifiedPartialAnswer
        );
        assert_eq!(finalization.factual_claim_coverage, 1.0);
        assert!(
            finalization
                .text
                .as_deref()
                .is_some_and(|text| text.contains("full reasoning artifact remains rejected"))
        );
    }

    #[test]
    fn reject_target_partial_fails_closed_on_target_contradiction() {
        let (mut current, target) = verified_reject_artifact();
        current
            .claims
            .iter_mut()
            .find(|claim| claim.id == "target")
            .unwrap()
            .state = EpistemicState::Contradicted;
        current
            .verification_receipts
            .iter_mut()
            .find(|receipt| receipt.id == "target-supported")
            .unwrap()
            .conclusion = VerificationConclusion::Contradicted;
        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn reject_target_partial_fails_closed_on_same_key_blocker() {
        let (mut current, target) = verified_reject_artifact();
        let blocker = current
            .claims
            .iter_mut()
            .find(|claim| claim.id == "blocker")
            .unwrap();
        blocker.proposition = Some(Proposition {
            key: target.key.clone(),
            value: "us-east-1".into(),
        });
        let receipt = current
            .verification_receipts
            .iter_mut()
            .find(|receipt| receipt.id == "blocker-contradicted")
            .unwrap();
        receipt.proposition = blocker.proposition.clone();

        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn reject_target_partial_fails_closed_on_shared_dependency() {
        let (mut current, target) = verified_reject_artifact();
        current.claims.push(Claim {
            id: "shared".into(),
            statement: "shared input".into(),
            state: EpistemicState::Supported,
            proposition: Some(Proposition {
                key: "deployment.source".into(),
                value: "config".into(),
            }),
            evidence_ids: vec!["target-e".into()],
        });
        current.inferences = vec![
            Inference {
                id: "shared-to-target".into(),
                premise_claim_ids: vec!["shared".into()],
                conclusion_claim_id: "target".into(),
                method: "lookup".into(),
            },
            Inference {
                id: "shared-to-blocker".into(),
                premise_claim_ids: vec!["shared".into()],
                conclusion_claim_id: "blocker".into(),
                method: "lookup".into(),
            },
        ];
        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn reject_target_partial_fails_closed_on_target_qualification_failure() {
        let (mut current, target) = verified_reject_artifact();
        current
            .evidence_qualification_findings
            .push(crate::EvidenceQualificationFinding {
                id: "target-qualification".into(),
                detector: "fixture".into(),
                kind: crate::EvidenceQualificationFindingKind::ScopeMismatch,
                reason: crate::EvidenceQualificationFindingReason::ScopeMismatch,
                strength: crate::FindingStrength::Hard,
                proposition: target.clone(),
                evidence_ids: vec!["target-e".into()],
                message: "target scope mismatch".into(),
            });
        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn reject_target_partial_fails_closed_on_causal_coupling() {
        let (mut current, target) = verified_reject_artifact();
        current.inferences.push(Inference {
            id: "target-causes-blocker".into(),
            premise_claim_ids: vec!["target".into()],
            conclusion_claim_id: "blocker".into(),
            method: "causal".into(),
        });
        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn reject_target_partial_fails_closed_when_target_and_blocker_share_evidence() {
        let (mut current, target) = verified_reject_artifact();
        current
            .verification_receipts
            .iter_mut()
            .find(|receipt| receipt.id == "blocker-contradicted")
            .unwrap()
            .evidence_ids = vec!["target-e".into()];
        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn reject_target_partial_requires_typed_non_target_blockers() {
        let (mut current, target) = verified_reject_artifact();
        current.claims.push(Claim {
            id: "unbound".into(),
            statement: "unbound unresolved state".into(),
            state: EpistemicState::Unknown,
            proposition: None,
            evidence_ids: vec![],
        });
        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn reject_target_partial_requires_evidence_bound_direct_verification() {
        let (mut current, target) = verified_reject_artifact();
        current
            .verification_receipts
            .iter_mut()
            .find(|receipt| receipt.id == "target-supported")
            .unwrap()
            .evidence_ids = vec![];
        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn reject_target_partial_requires_direct_supported_verification() {
        let (mut current, target) = verified_reject_artifact();
        current
            .verification_receipts
            .retain(|receipt| receipt.id != "target-supported");
        assert!(
            canonical_verified_target_reject_partial_answer(
                &current,
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn target_scoped_partial_fails_closed_when_requested_target_itself_is_unresolved() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        assert!(
            canonical_verified_target_partial_answer(
                &artifact(EpistemicState::Unknown),
                Verdict::Unknown,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn target_scoped_partial_requires_an_actual_non_target_blocker() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        assert!(
            canonical_verified_target_partial_answer(
                &artifact(EpistemicState::Supported),
                Verdict::Unknown,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn target_scoped_partial_fails_closed_on_target_qualification_finding() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let mut current = artifact(EpistemicState::Supported);
        current.claims.push(Claim {
            id: "extra".into(),
            statement: "unrelated detail".into(),
            state: EpistemicState::Unknown,
            proposition: Some(Proposition {
                key: "feature.owner".into(),
                value: "team-a".into(),
            }),
            evidence_ids: vec![],
        });
        current
            .evidence_qualification_findings
            .push(crate::EvidenceQualificationFinding {
                id: "qualification".into(),
                detector: "test".into(),
                kind: crate::EvidenceQualificationFindingKind::MissingMetadata,
                reason: crate::EvidenceQualificationFindingReason::MissingTemporalMetadata,
                strength: crate::FindingStrength::Soft,
                proposition: target.clone(),
                evidence_ids: vec![],
                message: "missing target metadata".into(),
            });
        assert!(
            canonical_verified_target_partial_answer(
                &current,
                Verdict::Unknown,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn target_scoped_partial_never_overrides_reject() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        assert!(
            canonical_verified_target_partial_answer(
                &artifact(EpistemicState::Supported),
                Verdict::Reject,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn target_scoped_partial_rejects_mixed_authority_for_the_same_exact_target() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let mut current = artifact(EpistemicState::Supported);
        current.claims.push(Claim {
            id: "duplicate".into(),
            statement: "same target unresolved".into(),
            state: EpistemicState::Unknown,
            proposition: Some(target.clone()),
            evidence_ids: vec![],
        });
        assert!(
            canonical_verified_target_partial_answer(
                &current,
                Verdict::Unknown,
                std::slice::from_ref(&target),
            )
            .is_none()
        );
    }

    #[test]
    fn grounded_claim_requires_supported_artifact_proposition() {
        let candidate = FinalAnswerCandidate {
            text: "enabled".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                },
                mode: FinalClaimMode::Grounded,
            }],
        };
        let result = finalize_answer(
            &artifact(EpistemicState::Supported),
            Verdict::Accept,
            candidate,
            FinalizationPolicy::default(),
        );
        assert_eq!(result.status, FinalizationStatus::GroundedAnswer);
        assert_eq!(result.factual_claim_coverage, 1.0);
    }

    #[test]
    fn renderer_cannot_introduce_a_new_grounded_fact() {
        let candidate = FinalAnswerCandidate {
            text: "wrong region".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: Proposition {
                    key: "deployment.region".into(),
                    value: "r2".into(),
                },
                mode: FinalClaimMode::Grounded,
            }],
        };
        let result = finalize_answer(
            &artifact(EpistemicState::Supported),
            Verdict::Accept,
            candidate,
            FinalizationPolicy::default(),
        );
        assert_eq!(result.status, FinalizationStatus::RequiresVerification);
        assert!(result.text.is_none());
        assert_eq!(result.factual_claim_coverage, 0.0);
    }

    #[test]
    fn unknown_can_render_only_as_qualified_uncertainty() {
        let candidate = CanonicalFinalAnswerRenderer
            .render(&artifact(EpistemicState::Unknown), Verdict::Unknown);
        let result = finalize_answer(
            &artifact(EpistemicState::Unknown),
            Verdict::Unknown,
            candidate,
            FinalizationPolicy::default(),
        );
        assert_eq!(result.status, FinalizationStatus::QualifiedPartialAnswer);
        assert_eq!(result.factual_claim_coverage, 1.0);
    }
}
