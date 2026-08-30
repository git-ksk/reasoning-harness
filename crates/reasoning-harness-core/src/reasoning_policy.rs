use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AcceptancePolicy, ApplicabilityScope, AssumptionInspector, EpistemicState,
    EvidenceAuthorityPolicy, EvidenceQualificationInspector, EvidenceRequirement,
    GroundedResolutionPolicy, Proposition, ReasoningArtifact, ResolverClass, ScopeCoverage,
    SemanticDiagnosticTarget, SoftJudgeDecision, SoftJudgeObservation, StrictAcceptancePolicy,
    VerificationReceipt, evidence_qualification::qualification_reasons, validate_artifact,
    verification::receipt_matches,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvidenceConstraints {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_of_unix_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<ApplicabilityScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_authority_class: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoftFindingEscalation {
    #[default]
    Ignore,
    RequestEvidence,
    RequestDeterministicVerification,
    HumanReview,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningPolicyLayer {
    pub layer_id: String,
    #[serde(default)]
    pub evidence: PolicyEvidenceConstraints,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_derived_support: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_resolver_classes: Option<BTreeSet<ResolverClass>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soft_finding_escalation: Option<SoftFindingEscalation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasoningPolicy {
    pub version_id: String,
    #[serde(default)]
    pub source_layers: Vec<String>,
    #[serde(default)]
    pub evidence: PolicyEvidenceConstraints,
    #[serde(default = "default_true")]
    pub allow_derived_support: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_resolver_classes: Option<BTreeSet<ResolverClass>>,
    #[serde(default)]
    pub soft_finding_escalation: SoftFindingEscalation,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEscalationAction {
    RequestEvidence,
    RequestDeterministicVerification,
    HumanReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEscalation {
    pub source_request_id: String,
    pub action: PolicyEscalationAction,
    pub target: SemanticDiagnosticTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyInvalidationReason {
    ReceiptNoLongerAdmissible,
    EvidenceNoLongerQualified,
    HardAuthorityNotReconstructable,
    DerivedSupportDisabled,
    DependencyInvalidated,
    UpstreamStateChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PolicyInvalidationTarget {
    VerificationReceipt { receipt_id: String },
    Claim { claim_id: String },
    Inference { inference_id: String },
    Finalization,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyInvalidation {
    pub target: PolicyInvalidationTarget,
    pub reason: PolicyInvalidationReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReasoningPolicyTransition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_policy_version: Option<String>,
    pub policy: ReasoningPolicy,
    pub artifact: ReasoningArtifact,
    #[serde(default)]
    pub invalidations: Vec<PolicyInvalidation>,
    pub finalization_invalidated: bool,
    pub verdict_after_re_evaluation: crate::Verdict,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReasoningPolicyError {
    #[error("reasoning policy version id must not be empty")]
    EmptyVersionId,
    #[error("reasoning policy layer id must not be empty")]
    EmptyLayerId,
    #[error("unknown authority class in reasoning policy: {0}")]
    UnknownAuthorityClass(String),
    #[error("invalid reasoning policy scope: {0}")]
    InvalidPolicyScope(String),
    #[error("policy scope constraints are disjoint for dimension {0}")]
    DisjointScope(String),
    #[error("input artifact is invalid before policy transition: {0:?}")]
    InvalidInputArtifact(Vec<String>),
    #[error("artifact is invalid after policy transition: {0:?}")]
    InvalidOutputArtifact(Vec<String>),
}

pub fn compose_reasoning_policy(
    version_id: impl Into<String>,
    layers: &[ReasoningPolicyLayer],
    authority_policy: &EvidenceAuthorityPolicy,
) -> Result<ReasoningPolicy, ReasoningPolicyError> {
    let version_id = version_id.into();
    if version_id.trim().is_empty() {
        return Err(ReasoningPolicyError::EmptyVersionId);
    }

    let mut effective = ReasoningPolicy {
        version_id,
        source_layers: Vec::with_capacity(layers.len()),
        evidence: PolicyEvidenceConstraints::default(),
        allow_derived_support: true,
        allowed_resolver_classes: None,
        soft_finding_escalation: SoftFindingEscalation::Ignore,
    };

    for layer in layers {
        if layer.layer_id.trim().is_empty() {
            return Err(ReasoningPolicyError::EmptyLayerId);
        }
        effective.source_layers.push(layer.layer_id.clone());

        if let Some(as_of) = layer.evidence.as_of_unix_seconds {
            // Temporal evaluation time is contextual rather than monotonic. Later scope wins,
            // and every policy application re-runs qualification against the effective value.
            effective.evidence.as_of_unix_seconds = Some(as_of);
        }
        effective.evidence.scope = intersect_optional_scopes(
            effective.evidence.scope.take(),
            layer.evidence.scope.clone(),
        )?;
        effective.evidence.minimum_authority_class = stricter_authority(
            effective.evidence.minimum_authority_class.take(),
            layer.evidence.minimum_authority_class.clone(),
            authority_policy,
        )?;

        if let Some(allow) = layer.allow_derived_support {
            effective.allow_derived_support &= allow;
        }
        effective.allowed_resolver_classes = intersect_optional_sets(
            effective.allowed_resolver_classes.take(),
            layer.allowed_resolver_classes.clone(),
        );
        if let Some(escalation) = layer.soft_finding_escalation {
            // Escalation is advisory control flow, not authority; task/run policy may override it.
            effective.soft_finding_escalation = escalation;
        }
    }

    validate_reasoning_policy(&effective, authority_policy)?;
    Ok(effective)
}

pub fn validate_reasoning_policy(
    policy: &ReasoningPolicy,
    authority_policy: &EvidenceAuthorityPolicy,
) -> Result<(), ReasoningPolicyError> {
    if policy.version_id.trim().is_empty() {
        return Err(ReasoningPolicyError::EmptyVersionId);
    }
    if policy
        .source_layers
        .iter()
        .any(|layer| layer.trim().is_empty())
    {
        return Err(ReasoningPolicyError::EmptyLayerId);
    }
    if let Some(class) = &policy.evidence.minimum_authority_class {
        authority_rank(class, authority_policy)?;
    }
    if let Some(scope) = &policy.evidence.scope {
        for (dimension, coverage) in scope {
            if dimension.trim().is_empty() {
                return Err(ReasoningPolicyError::InvalidPolicyScope(
                    "scope dimension must not be empty".into(),
                ));
            }
            if let ScopeCoverage::Values { values } = coverage {
                if values.is_empty() {
                    return Err(ReasoningPolicyError::InvalidPolicyScope(format!(
                        "scope dimension {dimension} has no values"
                    )));
                }
                if values.iter().any(|value| value.trim().is_empty()) {
                    return Err(ReasoningPolicyError::InvalidPolicyScope(format!(
                        "scope dimension {dimension} has an empty value"
                    )));
                }
            }
        }
    }
    Ok(())
}

pub fn constrain_resolution_policy(
    policy: &ReasoningPolicy,
    base: &GroundedResolutionPolicy,
    authority_policy: &EvidenceAuthorityPolicy,
) -> Result<GroundedResolutionPolicy, ReasoningPolicyError> {
    validate_reasoning_policy(policy, authority_policy)?;
    let mut constrained = base.clone();
    if let Some(allowed) = &policy.allowed_resolver_classes {
        constrained.budget.allowed_resolver_classes = constrained
            .budget
            .allowed_resolver_classes
            .intersection(allowed)
            .copied()
            .collect();
    }
    constrained.budget.required_authority_class = stricter_authority(
        constrained.budget.required_authority_class.take(),
        policy.evidence.minimum_authority_class.clone(),
        authority_policy,
    )?;
    Ok(constrained)
}

pub fn escalation_for_soft_observation(
    policy: &ReasoningPolicy,
    observation: &SoftJudgeObservation,
) -> Option<PolicyEscalation> {
    if observation.decision != SoftJudgeDecision::Finding {
        return None;
    }
    let finding = observation.finding.as_ref()?;
    let action = match policy.soft_finding_escalation {
        SoftFindingEscalation::Ignore => return None,
        SoftFindingEscalation::RequestEvidence => PolicyEscalationAction::RequestEvidence,
        SoftFindingEscalation::RequestDeterministicVerification => {
            PolicyEscalationAction::RequestDeterministicVerification
        }
        SoftFindingEscalation::HumanReview => PolicyEscalationAction::HumanReview,
    };
    Some(PolicyEscalation {
        source_request_id: observation.request_id.clone(),
        action,
        target: finding.target.clone(),
    })
}

pub fn apply_reasoning_policy(
    artifact: &ReasoningArtifact,
    previous_policy: Option<&ReasoningPolicy>,
    policy: &ReasoningPolicy,
) -> Result<ReasoningPolicyTransition, ReasoningPolicyError> {
    validate_reasoning_policy(policy, &artifact.authority_policy)?;
    if let Some(previous_policy) = previous_policy {
        validate_reasoning_policy(previous_policy, &artifact.authority_policy)?;
    }
    let input_report = validate_artifact(artifact);
    if !input_report.is_ok() {
        return Err(ReasoningPolicyError::InvalidInputArtifact(
            input_report
                .diagnostics
                .into_iter()
                .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
                .collect(),
        ));
    }

    let mut next = artifact.clone();
    next.evidence_requirements = effective_requirements(&next, policy)?;
    let mut invalidations = Vec::new();
    let mut invalid_claim_ids = BTreeSet::new();
    let mut invalid_receipt_ids = BTreeSet::new();

    for receipt in &next.verification_receipts {
        let Some(claim_index) = matching_claim_index(&next, receipt) else {
            continue;
        };
        let claim = &next.claims[claim_index];
        if claim_policy_qualification_ok(&next, claim, &receipt.evidence_ids) {
            continue;
        }
        invalid_receipt_ids.insert(receipt.id.clone());
        invalid_claim_ids.insert(claim.id.clone());
        invalidations.push(PolicyInvalidation {
            target: PolicyInvalidationTarget::VerificationReceipt {
                receipt_id: receipt.id.clone(),
            },
            reason: PolicyInvalidationReason::ReceiptNoLongerAdmissible,
        });
    }
    next.verification_receipts
        .retain(|receipt| !invalid_receipt_ids.contains(&receipt.id));

    for claim in &next.claims {
        if invalid_claim_ids.contains(&claim.id) {
            continue;
        }
        if claim.state == EpistemicState::Inferred && !policy.allow_derived_support {
            invalid_claim_ids.insert(claim.id.clone());
            invalidations.push(PolicyInvalidation {
                target: PolicyInvalidationTarget::Claim {
                    claim_id: claim.id.clone(),
                },
                reason: PolicyInvalidationReason::DerivedSupportDisabled,
            });
            continue;
        }
        match claim.state {
            EpistemicState::Supported | EpistemicState::Contradicted
                if !has_retained_receipt_for_claim(&next, claim) =>
            {
                invalid_claim_ids.insert(claim.id.clone());
                invalidations.push(PolicyInvalidation {
                    target: PolicyInvalidationTarget::Claim {
                        claim_id: claim.id.clone(),
                    },
                    reason: PolicyInvalidationReason::HardAuthorityNotReconstructable,
                });
            }
            EpistemicState::Known
                if !claim_policy_qualification_ok(&next, claim, &claim.evidence_ids) =>
            {
                invalid_claim_ids.insert(claim.id.clone());
                invalidations.push(PolicyInvalidation {
                    target: PolicyInvalidationTarget::Claim {
                        claim_id: claim.id.clone(),
                    },
                    reason: PolicyInvalidationReason::EvidenceNoLongerQualified,
                });
            }
            _ => {}
        }
    }

    let direct_invalid_ids = invalid_claim_ids.clone();
    let invalid_inference_ids =
        propagate_dependency_invalidations(&next, &mut invalid_claim_ids, &mut invalidations);
    next.inferences
        .retain(|inference| !invalid_inference_ids.contains(&inference.id));

    for claim in &mut next.claims {
        if invalid_claim_ids.contains(&claim.id) {
            claim.state = EpistemicState::Assumed;
            claim.evidence_ids.clear();
            if direct_invalid_ids.contains(&claim.id)
                && !invalidations.iter().any(|invalidation| {
                    invalidation.target
                        == PolicyInvalidationTarget::Claim {
                            claim_id: claim.id.clone(),
                        }
                })
            {
                invalidations.push(PolicyInvalidation {
                    target: PolicyInvalidationTarget::Claim {
                        claim_id: claim.id.clone(),
                    },
                    reason: PolicyInvalidationReason::EvidenceNoLongerQualified,
                });
            }
        }
    }

    // Findings derived from evidence qualification or inference usage belong to the old
    // snapshot. Recompute them after policy invalidation so removed edges are not referenced.
    next.evidence_qualification_findings = EvidenceQualificationInspector.inspect(&next).findings;
    next.assumption_findings = AssumptionInspector.inspect(&next).findings;

    let finalization_invalidated = !invalid_claim_ids.is_empty();
    if finalization_invalidated {
        invalidations.push(PolicyInvalidation {
            target: PolicyInvalidationTarget::Finalization,
            reason: PolicyInvalidationReason::UpstreamStateChanged,
        });
    }

    let output_report = validate_artifact(&next);
    if !output_report.is_ok() {
        return Err(ReasoningPolicyError::InvalidOutputArtifact(
            output_report
                .diagnostics
                .into_iter()
                .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
                .collect(),
        ));
    }
    let verdict_after_re_evaluation = StrictAcceptancePolicy.decide(&next);

    Ok(ReasoningPolicyTransition {
        previous_policy_version: previous_policy.map(|previous| previous.version_id.clone()),
        policy: policy.clone(),
        artifact: next,
        invalidations,
        finalization_invalidated,
        verdict_after_re_evaluation,
    })
}

fn effective_requirements(
    artifact: &ReasoningArtifact,
    policy: &ReasoningPolicy,
) -> Result<Vec<EvidenceRequirement>, ReasoningPolicyError> {
    let mut requirements = artifact
        .evidence_requirements
        .iter()
        .cloned()
        .map(|requirement| (requirement.proposition.key.clone(), requirement))
        .collect::<BTreeMap<_, _>>();

    let propositions = artifact
        .hypotheses
        .iter()
        .chain(
            artifact
                .claims
                .iter()
                .filter_map(|claim| claim.proposition.as_ref()),
        )
        .cloned()
        .fold(
            BTreeMap::<String, Proposition>::new(),
            |mut map, proposition| {
                map.entry(proposition.key.clone()).or_insert(proposition);
                map
            },
        );

    let has_policy_constraints = policy.evidence.as_of_unix_seconds.is_some()
        || policy.evidence.scope.is_some()
        || policy.evidence.minimum_authority_class.is_some();

    for (key, proposition) in propositions {
        if !requirements.contains_key(&key) && has_policy_constraints {
            requirements.insert(
                key.clone(),
                EvidenceRequirement {
                    proposition,
                    as_of_unix_seconds: policy.evidence.as_of_unix_seconds,
                    scope: policy.evidence.scope.clone(),
                    minimum_authority_class: policy.evidence.minimum_authority_class.clone(),
                },
            );
        }
    }

    for requirement in requirements.values_mut() {
        if requirement.as_of_unix_seconds.is_none() {
            requirement.as_of_unix_seconds = policy.evidence.as_of_unix_seconds;
        }
        requirement.scope =
            intersect_optional_scopes(requirement.scope.take(), policy.evidence.scope.clone())?;
        requirement.minimum_authority_class = stricter_authority(
            requirement.minimum_authority_class.take(),
            policy.evidence.minimum_authority_class.clone(),
            &artifact.authority_policy,
        )?;
    }

    Ok(requirements.into_values().collect())
}

fn matching_claim_index(
    artifact: &ReasoningArtifact,
    receipt: &VerificationReceipt,
) -> Option<usize> {
    let matches = artifact
        .claims
        .iter()
        .enumerate()
        .filter(|(_, claim)| receipt_matches(receipt, claim))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    (matches.len() == 1).then_some(matches[0])
}

fn has_retained_receipt_for_claim(artifact: &ReasoningArtifact, claim: &crate::Claim) -> bool {
    artifact
        .verification_receipts
        .iter()
        .any(|receipt| receipt_matches(receipt, claim))
}

fn claim_policy_qualification_ok(
    artifact: &ReasoningArtifact,
    claim: &crate::Claim,
    evidence_ids: &[String],
) -> bool {
    let Some(proposition) = &claim.proposition else {
        return true;
    };
    let Some(requirement) = artifact
        .evidence_requirements
        .iter()
        .find(|requirement| requirement.proposition.key == proposition.key)
    else {
        return true;
    };
    if evidence_ids.is_empty() {
        return false;
    }
    evidence_ids.iter().all(|evidence_id| {
        artifact
            .evidence
            .iter()
            .find(|evidence| &evidence.id == evidence_id)
            .is_some_and(|evidence| {
                qualification_reasons(&artifact.authority_policy, requirement, evidence).is_empty()
            })
    })
}

fn propagate_dependency_invalidations(
    artifact: &ReasoningArtifact,
    invalid_claim_ids: &mut BTreeSet<String>,
    invalidations: &mut Vec<PolicyInvalidation>,
) -> BTreeSet<String> {
    let mut invalid_inference_ids = BTreeSet::new();
    let mut queue = invalid_claim_ids.iter().cloned().collect::<VecDeque<_>>();
    while let Some(invalidated) = queue.pop_front() {
        for inference in &artifact.inferences {
            if !inference.premise_claim_ids.contains(&invalidated) {
                continue;
            }
            if invalid_inference_ids.insert(inference.id.clone()) {
                invalidations.push(PolicyInvalidation {
                    target: PolicyInvalidationTarget::Inference {
                        inference_id: inference.id.clone(),
                    },
                    reason: PolicyInvalidationReason::DependencyInvalidated,
                });
            }
            let Some(conclusion) = artifact
                .claims
                .iter()
                .find(|claim| claim.id == inference.conclusion_claim_id)
            else {
                continue;
            };
            if invalid_claim_ids.contains(&conclusion.id)
                || has_retained_receipt_for_claim(artifact, conclusion)
            {
                continue;
            }
            if invalid_claim_ids.insert(conclusion.id.clone()) {
                invalidations.push(PolicyInvalidation {
                    target: PolicyInvalidationTarget::Claim {
                        claim_id: conclusion.id.clone(),
                    },
                    reason: PolicyInvalidationReason::DependencyInvalidated,
                });
                queue.push_back(conclusion.id.clone());
            }
        }
    }
    invalid_inference_ids
}

fn stricter_authority(
    left: Option<String>,
    right: Option<String>,
    authority_policy: &EvidenceAuthorityPolicy,
) -> Result<Option<String>, ReasoningPolicyError> {
    match (left, right) {
        (None, None) => Ok(None),
        (Some(class), None) | (None, Some(class)) => {
            authority_rank(&class, authority_policy)?;
            Ok(Some(class))
        }
        (Some(left), Some(right)) => {
            let left_rank = authority_rank(&left, authority_policy)?;
            let right_rank = authority_rank(&right, authority_policy)?;
            Ok(Some(if right_rank > left_rank { right } else { left }))
        }
    }
}

fn authority_rank(
    class: &str,
    authority_policy: &EvidenceAuthorityPolicy,
) -> Result<u16, ReasoningPolicyError> {
    authority_policy
        .ranks
        .get(class)
        .copied()
        .ok_or_else(|| ReasoningPolicyError::UnknownAuthorityClass(class.into()))
}

fn intersect_optional_sets<T: Ord + Clone>(
    left: Option<BTreeSet<T>>,
    right: Option<BTreeSet<T>>,
) -> Option<BTreeSet<T>> {
    match (left, right) {
        (None, None) => None,
        (Some(set), None) | (None, Some(set)) => Some(set),
        (Some(left), Some(right)) => Some(left.intersection(&right).cloned().collect()),
    }
}

fn intersect_optional_scopes(
    left: Option<ApplicabilityScope>,
    right: Option<ApplicabilityScope>,
) -> Result<Option<ApplicabilityScope>, ReasoningPolicyError> {
    match (left, right) {
        (None, None) => Ok(None),
        (Some(scope), None) | (None, Some(scope)) => Ok(Some(scope)),
        (Some(left), Some(right)) => Ok(Some(intersect_scopes(&left, &right)?)),
    }
}

fn intersect_scopes(
    left: &ApplicabilityScope,
    right: &ApplicabilityScope,
) -> Result<ApplicabilityScope, ReasoningPolicyError> {
    let dimensions = left
        .keys()
        .chain(right.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut result = ApplicabilityScope::new();
    for dimension in dimensions {
        let coverage = match (left.get(&dimension), right.get(&dimension)) {
            (Some(left), Some(right)) => intersect_coverage(&dimension, left, right)?,
            (Some(coverage), None) | (None, Some(coverage)) => coverage.clone(),
            (None, None) => continue,
        };
        result.insert(dimension, coverage);
    }
    Ok(result)
}

fn intersect_coverage(
    dimension: &str,
    left: &ScopeCoverage,
    right: &ScopeCoverage,
) -> Result<ScopeCoverage, ReasoningPolicyError> {
    match (left, right) {
        (ScopeCoverage::Any, coverage) | (coverage, ScopeCoverage::Any) => Ok(coverage.clone()),
        (ScopeCoverage::Values { values: left }, ScopeCoverage::Values { values: right }) => {
            let values = left.intersection(right).cloned().collect::<BTreeSet<_>>();
            if values.is_empty() {
                return Err(ReasoningPolicyError::DisjointScope(dimension.into()));
            }
            Ok(ScopeCoverage::Values { values })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CanonicalFinalAnswerRenderer, Claim, Evidence, EvidenceMetadata, FinalAnswerRenderer,
        FinalizationPolicy, Inference, SoftJudgeIdentity, SoftSemanticFinding, TemporalValidity,
        VerificationConclusion, finalize_answer,
    };

    fn authority_policy() -> EvidenceAuthorityPolicy {
        EvidenceAuthorityPolicy {
            ranks: BTreeMap::from([
                ("untrusted".into(), 0),
                ("secondary".into(), 1),
                ("primary".into(), 2),
            ]),
        }
    }

    fn values(items: &[&str]) -> ScopeCoverage {
        ScopeCoverage::Values {
            values: items.iter().map(|item| (*item).to_string()).collect(),
        }
    }

    fn region_scope(items: &[&str]) -> ApplicabilityScope {
        BTreeMap::from([("region".into(), values(items))])
    }

    #[test]
    fn policy_composition_is_restrictive_for_authority_scope_capabilities_and_resolvers() {
        let layers = vec![
            ReasoningPolicyLayer {
                layer_id: "global".into(),
                evidence: PolicyEvidenceConstraints {
                    as_of_unix_seconds: Some(100),
                    scope: Some(region_scope(&["r1", "r2"])),
                    minimum_authority_class: Some("secondary".into()),
                },
                allow_derived_support: Some(true),
                allowed_resolver_classes: Some(BTreeSet::from([
                    ResolverClass::EvidenceAcquisition,
                    ResolverClass::DeterministicVerifier,
                ])),
                soft_finding_escalation: Some(SoftFindingEscalation::RequestEvidence),
            },
            ReasoningPolicyLayer {
                layer_id: "domain".into(),
                evidence: PolicyEvidenceConstraints {
                    as_of_unix_seconds: None,
                    scope: Some(region_scope(&["r2"])),
                    minimum_authority_class: Some("primary".into()),
                },
                allow_derived_support: Some(false),
                allowed_resolver_classes: Some(BTreeSet::from([
                    ResolverClass::EvidenceAcquisition,
                ])),
                soft_finding_escalation: None,
            },
            ReasoningPolicyLayer {
                layer_id: "run".into(),
                evidence: PolicyEvidenceConstraints {
                    as_of_unix_seconds: Some(200),
                    scope: None,
                    minimum_authority_class: Some("secondary".into()),
                },
                allow_derived_support: Some(true),
                allowed_resolver_classes: None,
                soft_finding_escalation: Some(
                    SoftFindingEscalation::RequestDeterministicVerification,
                ),
            },
        ];
        let policy = compose_reasoning_policy("policy-v1", &layers, &authority_policy()).unwrap();
        assert_eq!(policy.source_layers, vec!["global", "domain", "run"]);
        assert_eq!(policy.evidence.as_of_unix_seconds, Some(200));
        assert_eq!(policy.evidence.scope, Some(region_scope(&["r2"])));
        assert_eq!(
            policy.evidence.minimum_authority_class.as_deref(),
            Some("primary")
        );
        assert!(!policy.allow_derived_support);
        assert_eq!(
            policy.allowed_resolver_classes,
            Some(BTreeSet::from([ResolverClass::EvidenceAcquisition]))
        );
        assert_eq!(
            policy.soft_finding_escalation,
            SoftFindingEscalation::RequestDeterministicVerification
        );
    }

    fn supported_artifact() -> ReasoningArtifact {
        let proposition = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        ReasoningArtifact {
            task: "is enabled".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "fixture".into(),
                observation: "enabled".into(),
                facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
                metadata: EvidenceMetadata {
                    temporal: Some(TemporalValidity {
                        effective_from_unix_seconds: Some(100),
                        effective_until_unix_seconds: Some(200),
                    }),
                    scope: Some(region_scope(&["r1"])),
                    provenance_class: Some("secondary".into()),
                },
            }],
            hypotheses: vec![proposition.clone()],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: authority_policy(),
            candidate_diagnostics: vec![],
            verification_receipts: vec![VerificationReceipt {
                id: "receipt-1".into(),
                verifier: "fixture".into(),
                claim_statement: None,
                proposition: Some(proposition.clone()),
                claim_id: Some("c1".into()),
                conclusion: VerificationConclusion::Supported,
                evidence_ids: vec!["e1".into()],
            }],
            adversarial_findings: vec![],
            assumption_findings: vec![],
            evidence_qualification_findings: vec![],
            claims: vec![Claim {
                id: "c1".into(),
                statement: "feature.enabled = true".into(),
                state: EpistemicState::Supported,
                proposition: Some(proposition),
                evidence_ids: vec!["e1".into()],
            }],
            inferences: vec![],
        }
    }

    fn policy(version: &str, as_of: i64, authority: &str, scope: &[&str]) -> ReasoningPolicy {
        compose_reasoning_policy(
            version,
            &[ReasoningPolicyLayer {
                layer_id: "run".into(),
                evidence: PolicyEvidenceConstraints {
                    as_of_unix_seconds: Some(as_of),
                    scope: Some(region_scope(scope)),
                    minimum_authority_class: Some(authority.into()),
                },
                allow_derived_support: None,
                allowed_resolver_classes: None,
                soft_finding_escalation: None,
            }],
            &authority_policy(),
        )
        .unwrap()
    }

    #[test]
    fn stricter_authority_invalidates_receipt_claim_and_grounded_finalization_without_mutating_old_snapshot()
     {
        let artifact = supported_artifact();
        let previous = policy("policy-v1", 150, "secondary", &["r1"]);
        let accepted = apply_reasoning_policy(&artifact, None, &previous).unwrap();
        assert_eq!(accepted.artifact.claims[0].state, EpistemicState::Supported);
        assert_eq!(accepted.verdict_after_re_evaluation, crate::Verdict::Accept);

        let stricter = policy("policy-v2", 150, "primary", &["r1"]);
        let transitioned = apply_reasoning_policy(&artifact, Some(&previous), &stricter).unwrap();
        assert_eq!(
            transitioned.previous_policy_version.as_deref(),
            Some("policy-v1")
        );
        assert_eq!(
            transitioned.artifact.claims[0].state,
            EpistemicState::Assumed
        );
        assert!(transitioned.artifact.verification_receipts.is_empty());
        assert!(transitioned.finalization_invalidated);
        assert_eq!(
            transitioned.verdict_after_re_evaluation,
            crate::Verdict::Unknown
        );
        assert_eq!(artifact.claims[0].state, EpistemicState::Supported);
        assert_eq!(artifact.verification_receipts.len(), 1);

        let candidate = CanonicalFinalAnswerRenderer.render(
            &transitioned.artifact,
            transitioned.verdict_after_re_evaluation,
        );
        let finalization = finalize_answer(
            &transitioned.artifact,
            transitioned.verdict_after_re_evaluation,
            candidate,
            FinalizationPolicy::default(),
        );
        assert_ne!(
            finalization.status,
            crate::FinalizationStatus::GroundedAnswer
        );
    }

    #[test]
    fn temporal_and_scope_policy_changes_requalify_existing_evidence() {
        let artifact = supported_artifact();
        let temporal = policy("temporal", 250, "secondary", &["r1"]);
        let result = apply_reasoning_policy(&artifact, None, &temporal).unwrap();
        assert_eq!(result.artifact.claims[0].state, EpistemicState::Assumed);

        let broader_scope = policy("scope", 150, "secondary", &["r1", "r2"]);
        let result = apply_reasoning_policy(&artifact, None, &broader_scope).unwrap();
        assert_eq!(result.artifact.claims[0].state, EpistemicState::Assumed);
    }

    #[test]
    fn invalidation_propagates_through_inference_dependencies() {
        let mut artifact = supported_artifact();
        artifact.claims.push(Claim {
            id: "c2".into(),
            statement: "feature.usable = true".into(),
            state: EpistemicState::Inferred,
            proposition: Some(Proposition {
                key: "feature.usable".into(),
                value: "true".into(),
            }),
            evidence_ids: vec![],
        });
        artifact.inferences.push(Inference {
            id: "i1".into(),
            premise_claim_ids: vec!["c1".into()],
            conclusion_claim_id: "c2".into(),
            method: "derived".into(),
        });
        let stricter = policy("policy-v2", 150, "primary", &["r1"]);
        let transitioned = apply_reasoning_policy(&artifact, None, &stricter).unwrap();
        assert_eq!(
            transitioned.artifact.claims[0].state,
            EpistemicState::Assumed
        );
        assert_eq!(
            transitioned.artifact.claims[1].state,
            EpistemicState::Assumed
        );
        assert!(transitioned.invalidations.iter().any(|invalidation| {
            invalidation.reason == PolicyInvalidationReason::DependencyInvalidated
        }));
        assert!(transitioned.artifact.inferences.is_empty());
        assert!(transitioned.artifact.assumption_findings.is_empty());
    }

    #[test]
    fn soft_finding_can_trigger_work_but_never_authority() {
        let policy = ReasoningPolicy {
            version_id: "v1".into(),
            source_layers: vec![],
            evidence: Default::default(),
            allow_derived_support: true,
            allowed_resolver_classes: None,
            soft_finding_escalation: SoftFindingEscalation::RequestEvidence,
        };
        let target = SemanticDiagnosticTarget::Proposition {
            proposition: Proposition {
                key: "feature.enabled".into(),
                value: "true".into(),
            },
        };
        let observation = SoftJudgeObservation {
            judge: SoftJudgeIdentity {
                judge_id: "judge".into(),
                model_id: "model".into(),
                configuration_id: "v1".into(),
            },
            request_id: "request".into(),
            decision: SoftJudgeDecision::Finding,
            finding: Some(SoftSemanticFinding {
                kind: crate::SemanticDiagnosticKind::Contradiction,
                target: target.clone(),
                note: None,
            }),
        };
        let escalation = escalation_for_soft_observation(&policy, &observation).unwrap();
        assert_eq!(escalation.action, PolicyEscalationAction::RequestEvidence);
        assert_eq!(escalation.target, target);
    }

    #[test]
    fn direct_policy_input_fails_closed_on_invalid_identity_authority_and_scope() {
        let mut invalid = ReasoningPolicy {
            version_id: "".into(),
            source_layers: vec![],
            evidence: Default::default(),
            allow_derived_support: true,
            allowed_resolver_classes: None,
            soft_finding_escalation: SoftFindingEscalation::Ignore,
        };
        assert_eq!(
            validate_reasoning_policy(&invalid, &authority_policy()),
            Err(ReasoningPolicyError::EmptyVersionId)
        );

        invalid.version_id = "v1".into();
        invalid.evidence.minimum_authority_class = Some("missing".into());
        assert_eq!(
            validate_reasoning_policy(&invalid, &authority_policy()),
            Err(ReasoningPolicyError::UnknownAuthorityClass(
                "missing".into()
            ))
        );

        invalid.evidence.minimum_authority_class = None;
        invalid.evidence.scope = Some(BTreeMap::from([(
            "region".into(),
            ScopeCoverage::Values {
                values: BTreeSet::new(),
            },
        )]));
        assert!(matches!(
            validate_reasoning_policy(&invalid, &authority_policy()),
            Err(ReasoningPolicyError::InvalidPolicyScope(_))
        ));
    }

    #[test]
    fn supported_state_without_reconstructable_receipt_is_invalidated_on_policy_transition() {
        let mut artifact = supported_artifact();
        artifact.verification_receipts.clear();
        let policy = policy("v1", 150, "secondary", &["r1"]);
        let transition = apply_reasoning_policy(&artifact, None, &policy).unwrap();
        assert_eq!(transition.artifact.claims[0].state, EpistemicState::Assumed);
        assert!(transition.invalidations.iter().any(|invalidation| {
            invalidation.reason == PolicyInvalidationReason::HardAuthorityNotReconstructable
        }));
    }

    #[test]
    fn resolution_policy_is_only_tightened() {
        let reasoning = compose_reasoning_policy(
            "v1",
            &[ReasoningPolicyLayer {
                layer_id: "global".into(),
                evidence: PolicyEvidenceConstraints {
                    minimum_authority_class: Some("primary".into()),
                    ..Default::default()
                },
                allowed_resolver_classes: Some(BTreeSet::from([
                    ResolverClass::EvidenceAcquisition,
                ])),
                ..Default::default()
            }],
            &authority_policy(),
        )
        .unwrap();
        let mut base = GroundedResolutionPolicy::default();
        base.budget
            .allowed_resolver_classes
            .insert(ResolverClass::DeterministicVerifier);
        base.budget.required_authority_class = Some("secondary".into());
        let constrained =
            constrain_resolution_policy(&reasoning, &base, &authority_policy()).unwrap();
        assert_eq!(
            constrained.budget.allowed_resolver_classes,
            BTreeSet::from([ResolverClass::EvidenceAcquisition])
        );
        assert_eq!(
            constrained.budget.required_authority_class.as_deref(),
            Some("primary")
        );
    }
}
