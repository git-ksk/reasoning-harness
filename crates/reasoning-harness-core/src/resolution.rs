use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    AdversarialDiscoveryPass, AssumptionDiscoveryPass, CanonicalFinalAnswerRenderer,
    CausalRelation, Evidence, EvidenceQualificationPass, EvidenceRequirement, FinalAnswerRenderer,
    FinalizationPolicy, FinalizationResult, FinalizationStatus, HarnessError, HarnessInput,
    HarnessOutcome, Proposition, ReasoningCandidate, StrictAcceptancePolicy,
    StructuredFactConflictDetector, TrustedVerificationPass, Verdict, VerificationPass,
    VerificationReceipt, finalize_answer, frameworks::five_whys::FiveWhysRestatementPass,
    run_harness, structured_fact_verifier_for_input,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolverClass {
    EvidenceAcquisition,
    DeterministicVerifier,
    CandidateRevision,
    HumanReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionReason {
    MissingSupport,
    UnsupportedPremise,
    EvidenceQualification,
    HardRefutation,
    FinalizationCoverage,
    ExplicitRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResolutionTarget {
    Proposition { proposition: Proposition },
    CausalRelation { relation: CausalRelation },
    EvidenceQualification { requirement: EvidenceRequirement },
    ClaimRevision { claim_id: String },
    HumanReview { claim_id: Option<String> },
}

impl ResolutionTarget {
    fn proposition(&self) -> Option<&Proposition> {
        match self {
            Self::Proposition { proposition } => Some(proposition),
            Self::EvidenceQualification { requirement } => Some(&requirement.proposition),
            Self::CausalRelation { .. } | Self::ClaimRevision { .. } | Self::HumanReview { .. } => {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionRequestBudget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_added_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_elapsed_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionRequest {
    pub id: String,
    pub reason: ResolutionReason,
    pub target: ResolutionTarget,
    pub resolver_class: ResolverClass,
    #[serde(default)]
    pub budget: ResolutionRequestBudget,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionCost {
    #[serde(default)]
    pub added_tokens: u64,
    #[serde(default)]
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionBudget {
    pub max_attempts: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_added_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_elapsed_ms: Option<u64>,
    #[serde(default)]
    pub allowed_resolver_classes: BTreeSet<ResolverClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_authority_class: Option<String>,
}

impl Default for ResolutionBudget {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            max_added_tokens: None,
            max_elapsed_ms: None,
            allowed_resolver_classes: BTreeSet::from([ResolverClass::EvidenceAcquisition]),
            required_authority_class: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroundedResolutionPolicy {
    pub budget: ResolutionBudget,
    #[serde(default)]
    pub revise_refuted: bool,
    #[serde(default)]
    pub allow_human_review: bool,
    #[serde(default = "default_proposition_resolver_class")]
    pub proposition_resolver_class: ResolverClass,
    #[serde(default)]
    pub finalization: FinalizationPolicyConfig,
}

fn default_proposition_resolver_class() -> ResolverClass {
    ResolverClass::EvidenceAcquisition
}

impl Default for GroundedResolutionPolicy {
    fn default() -> Self {
        Self {
            budget: ResolutionBudget::default(),
            revise_refuted: false,
            allow_human_review: false,
            proposition_resolver_class: ResolverClass::EvidenceAcquisition,
            finalization: FinalizationPolicyConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinalizationPolicyConfig {
    #[serde(default = "default_true")]
    pub allow_qualified_partial: bool,
}

fn default_true() -> bool {
    true
}

impl Default for FinalizationPolicyConfig {
    fn default() -> Self {
        Self {
            allow_qualified_partial: true,
        }
    }
}

impl From<FinalizationPolicyConfig> for FinalizationPolicy {
    fn from(value: FinalizationPolicyConfig) -> Self {
        Self {
            allow_qualified_partial: value.allow_qualified_partial,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcquiredEvidenceMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_at_unix_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retrieved_at_unix_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<crate::ApplicabilityScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_authority_class: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcquiredEvidence {
    pub id: String,
    pub source: String,
    pub observation: String,
    #[serde(default)]
    pub facts: BTreeMap<String, String>,
    /// Resolver-supplied acquisition metadata. This is untrusted until admission converts it into
    /// Harness-owned `EvidenceMetadata` under configured policy.
    #[serde(default)]
    pub acquisition_metadata: AcquiredEvidenceMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResolutionResolverContribution {
    AcquiredEvidence { evidence: Vec<AcquiredEvidence> },
    CandidateRevision { candidate: ReasoningCandidate },
    NoResult,
    HumanReviewRequired,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionResolverOutput {
    pub contribution: ResolutionResolverContribution,
    #[serde(default)]
    pub cost: ResolutionCost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionAdapterErrorKind {
    Unavailable,
    MalformedOutput,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionAdapterError {
    pub kind: ResolutionAdapterErrorKind,
    #[serde(default)]
    pub cost: ResolutionCost,
}

pub trait ResolutionResolver: Send + Sync {
    fn name(&self) -> &'static str;
    fn class(&self) -> ResolverClass;
    fn resolve(
        &self,
        request: &ResolutionRequest,
        attempt_index: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustedVerifierResolutionOutput {
    #[serde(default)]
    pub receipts: Vec<VerificationReceipt>,
    #[serde(default)]
    pub cost: ResolutionCost,
}

pub trait TrustedResolutionVerifier: Send + Sync {
    fn name(&self) -> &'static str;
    fn verify(
        &self,
        request: &ResolutionRequest,
        artifact: &crate::ReasoningArtifact,
        attempt_index: usize,
    ) -> Result<TrustedVerifierResolutionOutput, ResolutionAdapterError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAdmissionRejection {
    UntrustedSource,
    MissingTrustedMetadata,
    InvalidEvidence,
    MissingObservationTime,
    MissingRetrievalTime,
    MissingScopeMetadata,
    MissingAuthorityClaim,
    Stale,
    NotYetValid,
    ScopeMismatch,
    ScopeExpansion,
    UnknownAuthorityClass,
    InsufficientAuthority,
    AuthorityClaimMismatch,
}

pub trait EvidenceAdmissionPolicy: Send + Sync {
    fn admit(
        &self,
        resolver_name: &str,
        request: &ResolutionRequest,
        acquired: &AcquiredEvidence,
    ) -> Result<Evidence, EvidenceAdmissionRejection>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RejectAllEvidenceAdmission;

impl EvidenceAdmissionPolicy for RejectAllEvidenceAdmission {
    fn admit(
        &self,
        _resolver_name: &str,
        _request: &ResolutionRequest,
        _acquired: &AcquiredEvidence,
    ) -> Result<Evidence, EvidenceAdmissionRejection> {
        Err(EvidenceAdmissionRejection::UntrustedSource)
    }
}

pub trait GroundingPipeline: Send + Sync {
    fn run(
        &self,
        input: HarnessInput,
        candidate: ReasoningCandidate,
        trusted_receipts: &[VerificationReceipt],
    ) -> Result<HarnessOutcome, HarnessError>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StandardGroundingPipeline;

impl GroundingPipeline for StandardGroundingPipeline {
    fn run(
        &self,
        input: HarnessInput,
        candidate: ReasoningCandidate,
        trusted_receipts: &[VerificationReceipt],
    ) -> Result<HarnessOutcome, HarnessError> {
        let structured_verifier = structured_fact_verifier_for_input(&input);
        let passes: Vec<Box<dyn crate::Pass>> = vec![
            Box::new(AdversarialDiscoveryPass::new(vec![Box::new(
                StructuredFactConflictDetector,
            )])),
            Box::new(EvidenceQualificationPass),
            Box::new(VerificationPass::new(vec![structured_verifier])),
            Box::new(TrustedVerificationPass::new(trusted_receipts.to_vec())),
            Box::new(FiveWhysRestatementPass),
            Box::new(AssumptionDiscoveryPass),
        ];
        run_harness(input, candidate, &passes, &StrictAcceptancePolicy)
    }
}

pub trait ResolutionPlanner: Send + Sync {
    fn plan(
        &self,
        outcome: &HarnessOutcome,
        policy: &GroundedResolutionPolicy,
    ) -> Vec<ResolutionRequest>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultResolutionPlanner;

impl ResolutionPlanner for DefaultResolutionPlanner {
    fn plan(
        &self,
        outcome: &HarnessOutcome,
        policy: &GroundedResolutionPolicy,
    ) -> Vec<ResolutionRequest> {
        if outcome.verdict == Verdict::Reject {
            if !policy.revise_refuted {
                return vec![];
            }
            return outcome
                .artifact
                .claims
                .iter()
                .find(|claim| claim.state == crate::EpistemicState::Contradicted)
                .map(|claim| ResolutionRequest {
                    id: format!("resolution:revise:{}", claim.id),
                    reason: ResolutionReason::HardRefutation,
                    target: ResolutionTarget::ClaimRevision {
                        claim_id: claim.id.clone(),
                    },
                    resolver_class: ResolverClass::CandidateRevision,
                    budget: ResolutionRequestBudget::default(),
                })
                .into_iter()
                .collect();
        }

        if outcome.verdict != Verdict::Unknown {
            return vec![];
        }

        let mut requests = Vec::new();
        let mut propositions = BTreeSet::new();

        // Exact Harness-owned targets take precedence over model-selected/generated claims.
        // This prevents a candidate that omits the requested target (or emits unrelated unresolved
        // claims first) from consuming the bounded resolution budget before the task target is
        // attempted. Resolution still only acquires evidence; admission and ordinary trusted
        // re-verification remain mandatory before the target can become authoritative.
        for proposition in harness_owned_resolution_targets(&outcome.artifact) {
            if !target_requires_resolution(&outcome.artifact, &proposition)
                || !propositions.insert((proposition.key.clone(), proposition.value.clone()))
            {
                continue;
            }

            if let Some(requirement) = outcome
                .artifact
                .evidence_requirements
                .iter()
                .find(|requirement| requirement.proposition == proposition)
                .cloned()
            {
                requests.push(ResolutionRequest {
                    id: request_id("qualify", &proposition),
                    reason: ResolutionReason::EvidenceQualification,
                    target: ResolutionTarget::EvidenceQualification { requirement },
                    resolver_class: policy.proposition_resolver_class,
                    budget: ResolutionRequestBudget::default(),
                });
            } else {
                requests.push(ResolutionRequest {
                    id: request_id("support", &proposition),
                    reason: ResolutionReason::MissingSupport,
                    target: ResolutionTarget::Proposition {
                        proposition: proposition.clone(),
                    },
                    resolver_class: policy.proposition_resolver_class,
                    budget: ResolutionRequestBudget::default(),
                });
            }
        }

        for finding in &outcome.artifact.evidence_qualification_findings {
            if !propositions.insert((
                finding.proposition.key.clone(),
                finding.proposition.value.clone(),
            )) {
                continue;
            }
            let requirement = outcome
                .artifact
                .evidence_requirements
                .iter()
                .find(|requirement| requirement.proposition.key == finding.proposition.key)
                .cloned()
                .unwrap_or(EvidenceRequirement {
                    proposition: finding.proposition.clone(),
                    as_of_unix_seconds: None,
                    scope: None,
                    minimum_authority_class: policy.budget.required_authority_class.clone(),
                });
            requests.push(ResolutionRequest {
                id: request_id("qualify", &finding.proposition),
                reason: ResolutionReason::EvidenceQualification,
                target: ResolutionTarget::EvidenceQualification { requirement },
                resolver_class: policy.proposition_resolver_class,
                budget: ResolutionRequestBudget::default(),
            });
        }

        for finding in &outcome.artifact.assumption_findings {
            let Some(proposition) = finding.proposition.as_ref() else {
                continue;
            };
            if !propositions.insert((proposition.key.clone(), proposition.value.clone())) {
                continue;
            }
            requests.push(ResolutionRequest {
                id: request_id("assumption", proposition),
                reason: ResolutionReason::UnsupportedPremise,
                target: ResolutionTarget::Proposition {
                    proposition: proposition.clone(),
                },
                resolver_class: policy.proposition_resolver_class,
                budget: ResolutionRequestBudget::default(),
            });
        }

        for claim in &outcome.artifact.claims {
            if !matches!(
                claim.state,
                crate::EpistemicState::Assumed
                    | crate::EpistemicState::Unknown
                    | crate::EpistemicState::Inferred
            ) {
                continue;
            }
            if let Some(proposition) = &claim.proposition {
                if !propositions.insert((proposition.key.clone(), proposition.value.clone())) {
                    continue;
                }
                requests.push(ResolutionRequest {
                    id: request_id("support", proposition),
                    reason: ResolutionReason::MissingSupport,
                    target: ResolutionTarget::Proposition {
                        proposition: proposition.clone(),
                    },
                    resolver_class: policy.proposition_resolver_class,
                    budget: ResolutionRequestBudget::default(),
                });
            } else if policy.allow_human_review {
                requests.push(ResolutionRequest {
                    id: format!("resolution:human:{}", claim.id),
                    reason: ResolutionReason::MissingSupport,
                    target: ResolutionTarget::HumanReview {
                        claim_id: Some(claim.id.clone()),
                    },
                    resolver_class: ResolverClass::HumanReview,
                    budget: ResolutionRequestBudget::default(),
                });
            }
        }

        requests
    }
}

fn harness_owned_resolution_targets(artifact: &crate::ReasoningArtifact) -> Vec<Proposition> {
    let mut seen = BTreeSet::new();
    artifact
        .hypotheses
        .iter()
        .cloned()
        .chain(
            artifact
                .evidence_requirements
                .iter()
                .map(|requirement| requirement.proposition.clone()),
        )
        .filter(|proposition| seen.insert((proposition.key.clone(), proposition.value.clone())))
        .collect()
}

fn target_requires_resolution(
    artifact: &crate::ReasoningArtifact,
    proposition: &Proposition,
) -> bool {
    let matching = artifact
        .claims
        .iter()
        .filter(|claim| claim.proposition.as_ref() == Some(proposition))
        .collect::<Vec<_>>();
    matching.is_empty()
        || matching.iter().any(|claim| {
            matches!(
                claim.state,
                crate::EpistemicState::Assumed
                    | crate::EpistemicState::Unknown
                    | crate::EpistemicState::Inferred
            )
        })
}

fn request_id(prefix: &str, proposition: &Proposition) -> String {
    format!(
        "resolution:{prefix}:{}={}",
        proposition.key, proposition.value
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionAttemptStatus {
    AppliedEvidence,
    AppliedCandidateRevision,
    AppliedVerification,
    NoResult,
    RejectedUntrustedEvidence,
    AdapterUnavailable,
    MalformedOutput,
    AdapterFailed,
    HumanReviewRequired,
    BudgetExceeded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionAttempt {
    pub attempt_index: usize,
    pub request: ResolutionRequest,
    pub adapter_name: String,
    pub status: ResolutionAttemptStatus,
    pub cost: ResolutionCost,
    #[serde(default)]
    pub admitted_evidence_ids: Vec<String>,
    #[serde(default)]
    pub verification_receipts: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admission_rejection: Option<EvidenceAdmissionRejection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionTerminalStatus {
    ResolvedSupported,
    ResolvedQualified,
    ResolvedRefuted,
    Exhausted,
    Unavailable,
    HumanReviewRequired,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionUsage {
    pub attempts: usize,
    pub added_tokens: u64,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroundedResolutionOutcome {
    pub initial_verdict: Verdict,
    pub final_verdict: Verdict,
    pub terminal_status: ResolutionTerminalStatus,
    pub final_artifact: crate::ReasoningArtifact,
    pub attempts: Vec<ResolutionAttempt>,
    pub usage: ResolutionUsage,
    #[serde(default)]
    pub request_usage: BTreeMap<String, ResolutionUsage>,
    pub finalization: FinalizationResult,
}

#[derive(Debug, Error)]
pub enum ResolutionError {
    #[error("harness failed during grounded resolution: {0}")]
    Harness(#[from] HarnessError),
    #[error("resolution policy is invalid: {0}")]
    InvalidPolicy(String),
    #[error("trusted evidence admission changed acquired evidence content")]
    AdmissionChangedContent,
}

pub struct GroundedResolutionRuntime<'a> {
    pub pipeline: &'a dyn GroundingPipeline,
    pub planner: &'a dyn ResolutionPlanner,
    pub evidence_admission: &'a dyn EvidenceAdmissionPolicy,
    pub resolvers: &'a [&'a dyn ResolutionResolver],
    pub trusted_verifiers: &'a [&'a dyn TrustedResolutionVerifier],
    pub renderer: &'a dyn FinalAnswerRenderer,
}

impl<'a> GroundedResolutionRuntime<'a> {
    pub fn run(
        &self,
        input: HarnessInput,
        candidate: ReasoningCandidate,
        policy: &GroundedResolutionPolicy,
    ) -> Result<GroundedResolutionOutcome, ResolutionError> {
        validate_policy(&input, policy)?;
        let mut current_input = input;
        let mut current_candidate = candidate;
        let mut trusted_receipts = Vec::new();
        let mut current = self.pipeline.run(
            current_input.clone(),
            current_candidate.clone(),
            &trusted_receipts,
        )?;
        let initial_verdict = current.verdict;
        let mut attempts = Vec::new();
        let mut usage = ResolutionUsage::default();

        loop {
            if current.verdict == Verdict::Accept {
                let rendered = self.renderer.render(&current.artifact, current.verdict);
                let finalization = finalize_answer(
                    &current.artifact,
                    current.verdict,
                    rendered,
                    policy.finalization.into(),
                );
                if matches!(
                    finalization.status,
                    FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
                ) {
                    let terminal = if finalization.status == FinalizationStatus::GroundedAnswer {
                        ResolutionTerminalStatus::ResolvedSupported
                    } else {
                        ResolutionTerminalStatus::ResolvedQualified
                    };
                    return Ok(outcome(
                        initial_verdict,
                        current,
                        terminal,
                        attempts,
                        usage,
                        finalization,
                    ));
                }
                if finalization.status == FinalizationStatus::RequiresVerification {
                    let requests = finalization
                        .uncovered_propositions
                        .iter()
                        .enumerate()
                        .map(|(index, proposition)| ResolutionRequest {
                            id: format!(
                                "resolution:finalization:{index}:{}={}",
                                proposition.key, proposition.value
                            ),
                            reason: ResolutionReason::FinalizationCoverage,
                            target: ResolutionTarget::Proposition {
                                proposition: proposition.clone(),
                            },
                            resolver_class: policy.proposition_resolver_class,
                            budget: ResolutionRequestBudget::default(),
                        })
                        .collect::<Vec<_>>();
                    if let Some(terminal) = self.execute_next(
                        &requests,
                        policy,
                        &mut current_input,
                        &mut current_candidate,
                        &mut trusted_receipts,
                        &mut current,
                        &mut attempts,
                        &mut usage,
                    )? {
                        let finalization = self.finalize_terminal(&current, policy);
                        return Ok(outcome(
                            initial_verdict,
                            current,
                            terminal,
                            attempts,
                            usage,
                            finalization,
                        ));
                    }
                    continue;
                }
            }

            if current.verdict == Verdict::Reject && !policy.revise_refuted {
                let finalization = self.finalize_terminal(&current, policy);
                return Ok(outcome(
                    initial_verdict,
                    current,
                    ResolutionTerminalStatus::ResolvedRefuted,
                    attempts,
                    usage,
                    finalization,
                ));
            }

            let requests = self.planner.plan(&current, policy);
            if requests.is_empty() {
                let terminal = if usage.attempts >= policy.budget.max_attempts {
                    ResolutionTerminalStatus::Exhausted
                } else if current.verdict == Verdict::Reject {
                    ResolutionTerminalStatus::ResolvedRefuted
                } else {
                    ResolutionTerminalStatus::Unavailable
                };
                let finalization = self.finalize_terminal(&current, policy);
                return Ok(outcome(
                    initial_verdict,
                    current,
                    terminal,
                    attempts,
                    usage,
                    finalization,
                ));
            }

            if let Some(terminal) = self.execute_next(
                &requests,
                policy,
                &mut current_input,
                &mut current_candidate,
                &mut trusted_receipts,
                &mut current,
                &mut attempts,
                &mut usage,
            )? {
                let finalization = self.finalize_terminal(&current, policy);
                return Ok(outcome(
                    initial_verdict,
                    current,
                    terminal,
                    attempts,
                    usage,
                    finalization,
                ));
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_next(
        &self,
        requests: &[ResolutionRequest],
        policy: &GroundedResolutionPolicy,
        current_input: &mut HarnessInput,
        current_candidate: &mut ReasoningCandidate,
        trusted_receipts: &mut Vec<VerificationReceipt>,
        current: &mut HarnessOutcome,
        attempts: &mut Vec<ResolutionAttempt>,
        usage: &mut ResolutionUsage,
    ) -> Result<Option<ResolutionTerminalStatus>, ResolutionError> {
        if usage.attempts >= policy.budget.max_attempts {
            return Ok(Some(ResolutionTerminalStatus::Exhausted));
        }

        let Some(request) = requests.iter().find(|request| {
            policy
                .budget
                .allowed_resolver_classes
                .contains(&request.resolver_class)
                && request_has_attempt_budget(request, attempts)
        }) else {
            let any_allowed = requests.iter().any(|request| {
                policy
                    .budget
                    .allowed_resolver_classes
                    .contains(&request.resolver_class)
            });
            return Ok(Some(if any_allowed {
                ResolutionTerminalStatus::Exhausted
            } else {
                ResolutionTerminalStatus::Unavailable
            }));
        };
        let attempt_index = usage.attempts;
        ensure_resolution_hypothesis(current_input, request);

        if request.resolver_class == ResolverClass::DeterministicVerifier {
            let Some(verifier) = self.trusted_verifiers.first() else {
                return Ok(Some(ResolutionTerminalStatus::Unavailable));
            };
            match verifier.verify(request, &current.artifact, attempt_index) {
                Ok(output) => {
                    if !can_consume(&policy.budget, usage, output.cost)
                        || !can_consume_request(&request.budget, attempts, request, output.cost)
                    {
                        record_budget_exceeded(
                            attempts,
                            usage,
                            attempt_index,
                            request,
                            verifier.name(),
                            output.cost,
                        );
                        return Ok(Some(ResolutionTerminalStatus::Exhausted));
                    }
                    consume(usage, output.cost);
                    let receipt_count = output.receipts.len();
                    trusted_receipts.extend(output.receipts);
                    attempts.push(ResolutionAttempt {
                        attempt_index,
                        request: request.clone(),
                        adapter_name: verifier.name().into(),
                        status: if receipt_count == 0 {
                            ResolutionAttemptStatus::NoResult
                        } else {
                            ResolutionAttemptStatus::AppliedVerification
                        },
                        cost: output.cost,
                        admitted_evidence_ids: vec![],
                        verification_receipts: receipt_count,
                        admission_rejection: None,
                    });
                    *current = self.pipeline.run(
                        current_input.clone(),
                        current_candidate.clone(),
                        trusted_receipts,
                    )?;
                    return Ok(None);
                }
                Err(error) => {
                    return Ok(self.record_adapter_error(
                        attempts,
                        usage,
                        policy,
                        attempt_index,
                        request,
                        verifier.name(),
                        error,
                    ));
                }
            }
        }

        let Some(resolver) = self
            .resolvers
            .iter()
            .find(|resolver| resolver.class() == request.resolver_class)
            .copied()
        else {
            return Ok(Some(ResolutionTerminalStatus::Unavailable));
        };

        match resolver.resolve(request, attempt_index) {
            Ok(output) => {
                if !can_consume(&policy.budget, usage, output.cost)
                    || !can_consume_request(&request.budget, attempts, request, output.cost)
                {
                    record_budget_exceeded(
                        attempts,
                        usage,
                        attempt_index,
                        request,
                        resolver.name(),
                        output.cost,
                    );
                    return Ok(Some(ResolutionTerminalStatus::Exhausted));
                }
                consume(usage, output.cost);
                match output.contribution {
                    ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                        let mut admitted = Vec::new();
                        for raw in evidence {
                            if !valid_acquired_evidence(&raw)
                                || current_input.evidence.iter().any(|item| item.id == raw.id)
                            {
                                attempts.push(ResolutionAttempt {
                                    attempt_index,
                                    request: request.clone(),
                                    adapter_name: resolver.name().into(),
                                    status: ResolutionAttemptStatus::MalformedOutput,
                                    cost: output.cost,
                                    admitted_evidence_ids: vec![],
                                    verification_receipts: 0,
                                    admission_rejection: None,
                                });
                                return Ok(None);
                            }
                            match self
                                .evidence_admission
                                .admit(resolver.name(), request, &raw)
                            {
                                Ok(evidence) => {
                                    if evidence.id != raw.id
                                        || evidence.source != raw.source
                                        || evidence.observation != raw.observation
                                        || evidence.facts != raw.facts
                                    {
                                        return Err(ResolutionError::AdmissionChangedContent);
                                    }
                                    admitted.push(evidence);
                                }
                                Err(rejection) => {
                                    attempts.push(ResolutionAttempt {
                                        attempt_index,
                                        request: request.clone(),
                                        adapter_name: resolver.name().into(),
                                        status: ResolutionAttemptStatus::RejectedUntrustedEvidence,
                                        cost: output.cost,
                                        admitted_evidence_ids: vec![],
                                        verification_receipts: 0,
                                        admission_rejection: Some(rejection),
                                    });
                                    return Ok(None);
                                }
                            }
                        }
                        ensure_requirement(current_input, request, policy);
                        let ids = admitted
                            .iter()
                            .map(|evidence| evidence.id.clone())
                            .collect();
                        current_input.evidence.extend(admitted);
                        attempts.push(ResolutionAttempt {
                            attempt_index,
                            request: request.clone(),
                            adapter_name: resolver.name().into(),
                            status: ResolutionAttemptStatus::AppliedEvidence,
                            cost: output.cost,
                            admitted_evidence_ids: ids,
                            verification_receipts: 0,
                            admission_rejection: None,
                        });
                    }
                    ResolutionResolverContribution::CandidateRevision { candidate } => {
                        *current_candidate = candidate;
                        attempts.push(ResolutionAttempt {
                            attempt_index,
                            request: request.clone(),
                            adapter_name: resolver.name().into(),
                            status: ResolutionAttemptStatus::AppliedCandidateRevision,
                            cost: output.cost,
                            admitted_evidence_ids: vec![],
                            verification_receipts: 0,
                            admission_rejection: None,
                        });
                    }
                    ResolutionResolverContribution::NoResult => {
                        attempts.push(ResolutionAttempt {
                            attempt_index,
                            request: request.clone(),
                            adapter_name: resolver.name().into(),
                            status: ResolutionAttemptStatus::NoResult,
                            cost: output.cost,
                            admitted_evidence_ids: vec![],
                            verification_receipts: 0,
                            admission_rejection: None,
                        });
                    }
                    ResolutionResolverContribution::HumanReviewRequired => {
                        attempts.push(ResolutionAttempt {
                            attempt_index,
                            request: request.clone(),
                            adapter_name: resolver.name().into(),
                            status: ResolutionAttemptStatus::HumanReviewRequired,
                            cost: output.cost,
                            admitted_evidence_ids: vec![],
                            verification_receipts: 0,
                            admission_rejection: None,
                        });
                        return Ok(Some(ResolutionTerminalStatus::HumanReviewRequired));
                    }
                }
                *current = self.pipeline.run(
                    current_input.clone(),
                    current_candidate.clone(),
                    trusted_receipts,
                )?;
                Ok(None)
            }
            Err(error) => Ok(self.record_adapter_error(
                attempts,
                usage,
                policy,
                attempt_index,
                request,
                resolver.name(),
                error,
            )),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn record_adapter_error(
        &self,
        attempts: &mut Vec<ResolutionAttempt>,
        usage: &mut ResolutionUsage,
        policy: &GroundedResolutionPolicy,
        attempt_index: usize,
        request: &ResolutionRequest,
        adapter_name: &str,
        error: ResolutionAdapterError,
    ) -> Option<ResolutionTerminalStatus> {
        if !can_consume(&policy.budget, usage, error.cost)
            || !can_consume_request(&request.budget, attempts, request, error.cost)
        {
            record_budget_exceeded(
                attempts,
                usage,
                attempt_index,
                request,
                adapter_name,
                error.cost,
            );
            return Some(ResolutionTerminalStatus::Exhausted);
        }
        consume(usage, error.cost);
        let status = match error.kind {
            ResolutionAdapterErrorKind::Unavailable => ResolutionAttemptStatus::AdapterUnavailable,
            ResolutionAdapterErrorKind::MalformedOutput => ResolutionAttemptStatus::MalformedOutput,
            ResolutionAdapterErrorKind::Failed => ResolutionAttemptStatus::AdapterFailed,
        };
        attempts.push(ResolutionAttempt {
            attempt_index,
            request: request.clone(),
            adapter_name: adapter_name.into(),
            status,
            cost: error.cost,
            admitted_evidence_ids: vec![],
            verification_receipts: 0,
            admission_rejection: None,
        });
        (error.kind == ResolutionAdapterErrorKind::Unavailable)
            .then_some(ResolutionTerminalStatus::Unavailable)
    }

    fn finalize_terminal(
        &self,
        current: &HarnessOutcome,
        policy: &GroundedResolutionPolicy,
    ) -> FinalizationResult {
        let rendered = self.renderer.render(&current.artifact, current.verdict);
        finalize_answer(
            &current.artifact,
            current.verdict,
            rendered,
            policy.finalization.into(),
        )
    }
}

fn outcome(
    initial_verdict: Verdict,
    current: HarnessOutcome,
    terminal_status: ResolutionTerminalStatus,
    attempts: Vec<ResolutionAttempt>,
    usage: ResolutionUsage,
    finalization: FinalizationResult,
) -> GroundedResolutionOutcome {
    let request_usage = request_usage(&attempts);
    GroundedResolutionOutcome {
        initial_verdict,
        final_verdict: current.verdict,
        terminal_status,
        final_artifact: current.artifact,
        attempts,
        usage,
        request_usage,
        finalization,
    }
}

fn validate_policy(
    input: &HarnessInput,
    policy: &GroundedResolutionPolicy,
) -> Result<(), ResolutionError> {
    if policy.budget.max_attempts == 0 {
        return Err(ResolutionError::InvalidPolicy(
            "max_attempts must be greater than zero".into(),
        ));
    }
    if policy.budget.allowed_resolver_classes.is_empty() {
        return Err(ResolutionError::InvalidPolicy(
            "at least one resolver class must be allowed".into(),
        ));
    }
    if let Some(required) = &policy.budget.required_authority_class {
        if !input.authority_policy.ranks.contains_key(required) {
            return Err(ResolutionError::InvalidPolicy(format!(
                "required authority class is absent from harness policy: {required}"
            )));
        }
    }
    Ok(())
}

fn valid_acquired_evidence(evidence: &AcquiredEvidence) -> bool {
    !evidence.id.trim().is_empty()
        && !evidence.source.trim().is_empty()
        && !evidence.observation.trim().is_empty()
        && evidence
            .facts
            .iter()
            .all(|(key, value)| !key.trim().is_empty() && !value.trim().is_empty())
}

fn ensure_resolution_hypothesis(input: &mut HarnessInput, request: &ResolutionRequest) {
    let Some(proposition) = request.target.proposition() else {
        return;
    };
    if !input
        .hypotheses
        .iter()
        .any(|existing| existing == proposition)
    {
        input.hypotheses.push(proposition.clone());
    }
}

fn ensure_requirement(
    input: &mut HarnessInput,
    request: &ResolutionRequest,
    policy: &GroundedResolutionPolicy,
) {
    let explicit = match &request.target {
        ResolutionTarget::EvidenceQualification { requirement } => Some(requirement.clone()),
        _ => None,
    };
    let proposition = request.target.proposition().cloned();
    let Some(proposition) = proposition else {
        return;
    };

    if let Some(existing) = input
        .evidence_requirements
        .iter_mut()
        .find(|requirement| requirement.proposition.key == proposition.key)
    {
        if let Some(required) = &policy.budget.required_authority_class {
            let required_rank = input.authority_policy.ranks.get(required).copied();
            let existing_rank = existing
                .minimum_authority_class
                .as_ref()
                .and_then(|class| input.authority_policy.ranks.get(class))
                .copied();
            if existing_rank
                .zip(required_rank)
                .is_none_or(|(current, required)| current < required)
            {
                existing.minimum_authority_class = Some(required.clone());
            }
        }
        return;
    }

    if let Some(mut requirement) = explicit {
        if requirement.minimum_authority_class.is_none() {
            requirement.minimum_authority_class = policy.budget.required_authority_class.clone();
        }
        input.evidence_requirements.push(requirement);
    } else if policy.budget.required_authority_class.is_some() {
        input.evidence_requirements.push(EvidenceRequirement {
            proposition,
            as_of_unix_seconds: None,
            scope: None,
            minimum_authority_class: policy.budget.required_authority_class.clone(),
        });
    }
}

fn request_has_attempt_budget(request: &ResolutionRequest, attempts: &[ResolutionAttempt]) -> bool {
    request.budget.max_attempts.is_none_or(|limit| {
        attempts
            .iter()
            .filter(|attempt| attempt.request.id == request.id)
            .count()
            < limit
    })
}

fn can_consume_request(
    budget: &ResolutionRequestBudget,
    attempts: &[ResolutionAttempt],
    request: &ResolutionRequest,
    cost: ResolutionCost,
) -> bool {
    let usage = request_usage_for(attempts, &request.id);
    if let Some(limit) = budget.max_added_tokens {
        if usage.added_tokens.saturating_add(cost.added_tokens) > limit {
            return false;
        }
    }
    if let Some(limit) = budget.max_elapsed_ms {
        if usage.elapsed_ms.saturating_add(cost.elapsed_ms) > limit {
            return false;
        }
    }
    true
}

fn request_usage(attempts: &[ResolutionAttempt]) -> BTreeMap<String, ResolutionUsage> {
    let ids = attempts
        .iter()
        .map(|attempt| attempt.request.id.clone())
        .collect::<BTreeSet<_>>();
    ids.into_iter()
        .map(|id| {
            let usage = request_usage_for(attempts, &id);
            (id, usage)
        })
        .collect()
}

fn request_usage_for(attempts: &[ResolutionAttempt], request_id: &str) -> ResolutionUsage {
    let relevant = attempts
        .iter()
        .filter(|attempt| attempt.request.id == request_id)
        .collect::<Vec<_>>();
    ResolutionUsage {
        attempts: relevant.len(),
        added_tokens: relevant
            .iter()
            .filter(|attempt| attempt.status != ResolutionAttemptStatus::BudgetExceeded)
            .map(|attempt| attempt.cost.added_tokens)
            .sum(),
        elapsed_ms: relevant
            .iter()
            .filter(|attempt| attempt.status != ResolutionAttemptStatus::BudgetExceeded)
            .map(|attempt| attempt.cost.elapsed_ms)
            .sum(),
    }
}

fn can_consume(budget: &ResolutionBudget, usage: &ResolutionUsage, cost: ResolutionCost) -> bool {
    if let Some(limit) = budget.max_added_tokens {
        if usage.added_tokens.saturating_add(cost.added_tokens) > limit {
            return false;
        }
    }
    if let Some(limit) = budget.max_elapsed_ms {
        if usage.elapsed_ms.saturating_add(cost.elapsed_ms) > limit {
            return false;
        }
    }
    true
}

fn consume(usage: &mut ResolutionUsage, cost: ResolutionCost) {
    usage.attempts += 1;
    usage.added_tokens = usage.added_tokens.saturating_add(cost.added_tokens);
    usage.elapsed_ms = usage.elapsed_ms.saturating_add(cost.elapsed_ms);
}

fn record_budget_exceeded(
    attempts: &mut Vec<ResolutionAttempt>,
    usage: &mut ResolutionUsage,
    attempt_index: usize,
    request: &ResolutionRequest,
    adapter_name: &str,
    cost: ResolutionCost,
) {
    usage.attempts += 1;
    attempts.push(ResolutionAttempt {
        attempt_index,
        request: request.clone(),
        adapter_name: adapter_name.into(),
        status: ResolutionAttemptStatus::BudgetExceeded,
        cost,
        admitted_evidence_ids: vec![],
        verification_receipts: 0,
        admission_rejection: None,
    });
}

pub fn default_grounded_resolution_runtime<'a>(
    evidence_admission: &'a dyn EvidenceAdmissionPolicy,
    resolvers: &'a [&'a dyn ResolutionResolver],
    trusted_verifiers: &'a [&'a dyn TrustedResolutionVerifier],
) -> GroundedResolutionRuntime<'a> {
    static PIPELINE: StandardGroundingPipeline = StandardGroundingPipeline;
    static PLANNER: DefaultResolutionPlanner = DefaultResolutionPlanner;
    static RENDERER: CanonicalFinalAnswerRenderer = CanonicalFinalAnswerRenderer;
    GroundedResolutionRuntime {
        pipeline: &PIPELINE,
        planner: &PLANNER,
        evidence_admission,
        resolvers,
        trusted_verifiers,
        renderer: &RENDERER,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CandidateClaim, EpistemicState, EvidenceAuthorityPolicy, EvidenceMetadata, TemporalValidity,
    };

    struct OneEvidenceResolver {
        raw: AcquiredEvidence,
    }

    impl ResolutionResolver for OneEvidenceResolver {
        fn name(&self) -> &'static str {
            "fixture_acquirer"
        }

        fn class(&self) -> ResolverClass {
            ResolverClass::EvidenceAcquisition
        }

        fn resolve(
            &self,
            _request: &ResolutionRequest,
            _attempt_index: usize,
        ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
            Ok(ResolutionResolverOutput {
                contribution: ResolutionResolverContribution::AcquiredEvidence {
                    evidence: vec![self.raw.clone()],
                },
                cost: ResolutionCost {
                    added_tokens: 10,
                    elapsed_ms: 5,
                },
            })
        }
    }

    struct PrimaryAdmission;

    impl EvidenceAdmissionPolicy for PrimaryAdmission {
        fn admit(
            &self,
            _resolver_name: &str,
            _request: &ResolutionRequest,
            acquired: &AcquiredEvidence,
        ) -> Result<Evidence, EvidenceAdmissionRejection> {
            Ok(Evidence {
                id: acquired.id.clone(),
                source: acquired.source.clone(),
                observation: acquired.observation.clone(),
                facts: acquired.facts.clone(),
                metadata: EvidenceMetadata {
                    temporal: Some(TemporalValidity {
                        effective_from_unix_seconds: Some(100),
                        effective_until_unix_seconds: Some(300),
                    }),
                    scope: None,
                    provenance_class: Some("primary".into()),
                },
            })
        }
    }

    fn missing_support_input() -> HarnessInput {
        HarnessInput {
            task: "is enabled".into(),
            evidence: vec![],
            hypotheses: vec![Proposition {
                key: "feature.enabled".into(),
                value: "true".into(),
            }],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: EvidenceAuthorityPolicy {
                ranks: BTreeMap::from([("primary".into(), 20)]),
            },
        }
    }

    fn candidate() -> ReasoningCandidate {
        ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: "c1".into(),
                statement: "enabled".into(),
                proposed_state: EpistemicState::Supported,
                proposition: Some(Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                }),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        }
    }

    #[test]
    fn admitted_evidence_is_reverified_before_resolution_succeeds() {
        let resolver = OneEvidenceResolver {
            raw: AcquiredEvidence {
                id: "resolved-e1".into(),
                source: "fixture source".into(),
                observation: "feature enabled".into(),
                facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
                acquisition_metadata: Default::default(),
            },
        };
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = default_grounded_resolution_runtime(&PrimaryAdmission, &resolvers, &[]);
        let policy = GroundedResolutionPolicy {
            budget: ResolutionBudget {
                required_authority_class: Some("primary".into()),
                ..ResolutionBudget::default()
            },
            ..Default::default()
        };
        let result = runtime
            .run(missing_support_input(), candidate(), &policy)
            .unwrap();
        assert_eq!(result.initial_verdict, Verdict::Unknown);
        assert_eq!(result.final_verdict, Verdict::Accept);
        assert_eq!(
            result.terminal_status,
            ResolutionTerminalStatus::ResolvedSupported
        );
        assert_eq!(result.attempts.len(), 1);
        assert_eq!(
            result.finalization.status,
            FinalizationStatus::GroundedAnswer
        );
    }

    struct ExactHarnessTargetResolver {
        target: Proposition,
    }

    impl ResolutionResolver for ExactHarnessTargetResolver {
        fn name(&self) -> &'static str {
            "exact_harness_target"
        }

        fn class(&self) -> ResolverClass {
            ResolverClass::EvidenceAcquisition
        }

        fn resolve(
            &self,
            request: &ResolutionRequest,
            _attempt_index: usize,
        ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
            let Some(proposition) = request.target.proposition() else {
                return Ok(ResolutionResolverOutput {
                    contribution: ResolutionResolverContribution::NoResult,
                    cost: ResolutionCost::default(),
                });
            };
            if proposition != &self.target {
                return Ok(ResolutionResolverOutput {
                    contribution: ResolutionResolverContribution::NoResult,
                    cost: ResolutionCost::default(),
                });
            }
            Ok(ResolutionResolverOutput {
                contribution: ResolutionResolverContribution::AcquiredEvidence {
                    evidence: vec![AcquiredEvidence {
                        id: "exact-target-evidence".into(),
                        source: "fixture source".into(),
                        observation: format!("{}={}", proposition.key, proposition.value),
                        facts: BTreeMap::from([(
                            proposition.key.clone(),
                            proposition.value.clone(),
                        )]),
                        acquisition_metadata: Default::default(),
                    }],
                },
                cost: ResolutionCost::default(),
            })
        }
    }

    fn unrelated_candidate() -> ReasoningCandidate {
        ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: "noise".into(),
                statement: "unrelated unresolved claim".into(),
                proposed_state: EpistemicState::Assumed,
                proposition: Some(Proposition {
                    key: "unrelated.detail".into(),
                    value: "maybe".into(),
                }),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        }
    }

    #[test]
    fn harness_owned_target_precedes_unrelated_candidate_resolution() {
        let target = Proposition {
            key: "service.failover_region".into(),
            value: "eu-west-1".into(),
        };
        let input = HarnessInput {
            task: "resolve failover region".into(),
            hypotheses: vec![target.clone()],
            ..Default::default()
        };
        let resolver = ExactHarnessTargetResolver {
            target: target.clone(),
        };
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = default_grounded_resolution_runtime(&PrimaryAdmission, &resolvers, &[]);
        let policy = GroundedResolutionPolicy {
            budget: ResolutionBudget {
                max_attempts: 1,
                ..ResolutionBudget::default()
            },
            ..Default::default()
        };

        let result = runtime.run(input, unrelated_candidate(), &policy).unwrap();

        assert_eq!(result.attempts.len(), 1);
        assert_eq!(
            result.attempts[0].request.target.proposition(),
            Some(&target)
        );
        assert_eq!(
            result.attempts[0].status,
            ResolutionAttemptStatus::AppliedEvidence
        );
        assert!(result.final_artifact.claims.iter().any(|claim| {
            claim.proposition.as_ref() == Some(&target)
                && claim.state == EpistemicState::Supported
                && claim.evidence_ids == vec!["exact-target-evidence"]
        }));
        assert_eq!(result.final_verdict, Verdict::Unknown);
        assert_eq!(result.terminal_status, ResolutionTerminalStatus::Exhausted);
    }

    #[test]
    fn harness_owned_requirement_target_closes_even_when_candidate_omits_it() {
        let target = Proposition {
            key: "incident.root_cause".into(),
            value: "database".into(),
        };
        let input = HarnessInput {
            task: "resolve incident cause".into(),
            evidence_requirements: vec![EvidenceRequirement {
                proposition: target.clone(),
                as_of_unix_seconds: None,
                scope: None,
                minimum_authority_class: None,
            }],
            ..Default::default()
        };
        let resolver = ExactHarnessTargetResolver {
            target: target.clone(),
        };
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = default_grounded_resolution_runtime(&PrimaryAdmission, &resolvers, &[]);
        let policy = GroundedResolutionPolicy {
            budget: ResolutionBudget {
                max_attempts: 1,
                ..ResolutionBudget::default()
            },
            ..Default::default()
        };

        let result = runtime.run(input, unrelated_candidate(), &policy).unwrap();

        assert_eq!(
            result.attempts[0].request.target.proposition(),
            Some(&target)
        );
        assert!(matches!(
            &result.attempts[0].request.target,
            ResolutionTarget::EvidenceQualification { requirement }
                if requirement.proposition == target
        ));
        assert!(result.final_artifact.hypotheses.contains(&target));
        assert!(result.final_artifact.claims.iter().any(|claim| {
            claim.proposition.as_ref() == Some(&target) && claim.state == EpistemicState::Supported
        }));
    }

    #[test]
    fn already_supported_harness_target_is_not_requested_again() {
        let target = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let input = HarnessInput {
            task: "feature status".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "config".into(),
                observation: "enabled".into(),
                facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
                metadata: EvidenceMetadata::default(),
            }],
            hypotheses: vec![target.clone()],
            ..Default::default()
        };
        let outcome = StandardGroundingPipeline
            .run(input, unrelated_candidate(), &[])
            .unwrap();
        assert_eq!(outcome.verdict, Verdict::Unknown);

        let requests =
            DefaultResolutionPlanner.plan(&outcome, &GroundedResolutionPolicy::default());

        assert!(!requests.iter().any(|request| {
            request
                .target
                .proposition()
                .is_some_and(|proposition| proposition == &target)
        }));
        assert!(requests.iter().any(|request| {
            request.target.proposition().is_some_and(|proposition| {
                proposition.key == "unrelated.detail" && proposition.value == "maybe"
            })
        }));
    }

    #[test]
    fn acquired_evidence_is_untrusted_by_default() {
        let resolver = OneEvidenceResolver {
            raw: AcquiredEvidence {
                id: "resolved-e1".into(),
                source: "fixture source".into(),
                observation: "feature enabled".into(),
                facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
                acquisition_metadata: Default::default(),
            },
        };
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime =
            default_grounded_resolution_runtime(&RejectAllEvidenceAdmission, &resolvers, &[]);
        let result = runtime
            .run(
                missing_support_input(),
                candidate(),
                &GroundedResolutionPolicy::default(),
            )
            .unwrap();
        assert_eq!(result.final_verdict, Verdict::Unknown);
        assert_eq!(result.terminal_status, ResolutionTerminalStatus::Exhausted);
        assert!(
            result
                .attempts
                .iter()
                .all(|attempt| attempt.status == ResolutionAttemptStatus::RejectedUntrustedEvidence)
        );
    }

    struct RevisionResolver;

    impl ResolutionResolver for RevisionResolver {
        fn name(&self) -> &'static str {
            "fixture_reviser"
        }

        fn class(&self) -> ResolverClass {
            ResolverClass::CandidateRevision
        }

        fn resolve(
            &self,
            _request: &ResolutionRequest,
            _attempt_index: usize,
        ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
            Ok(ResolutionResolverOutput {
                contribution: ResolutionResolverContribution::CandidateRevision {
                    candidate: ReasoningCandidate {
                        claims: vec![CandidateClaim {
                            id: "revised".into(),
                            statement: "feature disabled".into(),
                            proposed_state: EpistemicState::Supported,
                            proposition: Some(Proposition {
                                key: "feature.enabled".into(),
                                value: "false".into(),
                            }),
                            evidence_ids: vec!["e1".into()],
                        }],
                        inferences: vec![],
                    },
                },
                cost: ResolutionCost::default(),
            })
        }
    }

    #[test]
    fn revised_candidate_reenters_the_full_untrusted_pipeline() {
        let input = HarnessInput {
            task: "feature status".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "config".into(),
                observation: "disabled".into(),
                facts: BTreeMap::from([("feature.enabled".into(), "false".into())]),
                metadata: EvidenceMetadata::default(),
            }],
            hypotheses: vec![],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: Default::default(),
        };
        let bad_candidate = ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: "bad".into(),
                statement: "feature enabled".into(),
                proposed_state: EpistemicState::Supported,
                proposition: Some(Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                }),
                evidence_ids: vec!["e1".into()],
            }],
            inferences: vec![],
        };
        let reviser = RevisionResolver;
        let resolvers: [&dyn ResolutionResolver; 1] = [&reviser];
        let runtime =
            default_grounded_resolution_runtime(&RejectAllEvidenceAdmission, &resolvers, &[]);
        let mut allowed = BTreeSet::new();
        allowed.insert(ResolverClass::CandidateRevision);
        let policy = GroundedResolutionPolicy {
            budget: ResolutionBudget {
                allowed_resolver_classes: allowed,
                ..Default::default()
            },
            revise_refuted: true,
            ..Default::default()
        };
        let result = runtime.run(input, bad_candidate, &policy).unwrap();
        assert_eq!(result.initial_verdict, Verdict::Reject);
        assert_eq!(result.final_verdict, Verdict::Accept);
        assert_eq!(
            result.terminal_status,
            ResolutionTerminalStatus::ResolvedSupported
        );
        assert!(
            result
                .final_artifact
                .verification_receipts
                .iter()
                .any(|receipt| receipt
                    .proposition
                    .as_ref()
                    .is_some_and(|p| p.value == "false"))
        );
    }

    struct FixtureTrustedVerifier;

    impl TrustedResolutionVerifier for FixtureTrustedVerifier {
        fn name(&self) -> &'static str {
            "fixture_oracle"
        }

        fn verify(
            &self,
            request: &ResolutionRequest,
            _artifact: &crate::ReasoningArtifact,
            _attempt_index: usize,
        ) -> Result<TrustedVerifierResolutionOutput, ResolutionAdapterError> {
            let proposition = request.target.proposition().unwrap().clone();
            Ok(TrustedVerifierResolutionOutput {
                receipts: vec![VerificationReceipt {
                    id: "fixture-resolution-receipt".into(),
                    verifier: "fixture_oracle".into(),
                    claim_statement: None,
                    proposition: Some(proposition),
                    claim_id: None,
                    conclusion: crate::VerificationConclusion::Supported,
                    evidence_ids: vec!["oracle-input".into()],
                }],
                cost: ResolutionCost::default(),
            })
        }
    }

    #[test]
    fn trusted_verifier_uses_a_separate_authority_boundary() {
        let verifier = FixtureTrustedVerifier;
        let trusted: [&dyn TrustedResolutionVerifier; 1] = [&verifier];
        let runtime =
            default_grounded_resolution_runtime(&RejectAllEvidenceAdmission, &[], &trusted);
        let mut allowed = BTreeSet::new();
        allowed.insert(ResolverClass::DeterministicVerifier);
        let policy = GroundedResolutionPolicy {
            budget: ResolutionBudget {
                allowed_resolver_classes: allowed,
                ..Default::default()
            },
            proposition_resolver_class: ResolverClass::DeterministicVerifier,
            ..Default::default()
        };
        let mut input = missing_support_input();
        input.evidence.push(Evidence {
            id: "oracle-input".into(),
            source: "fixture oracle input".into(),
            observation: "opaque input interpreted by external deterministic verifier".into(),
            facts: BTreeMap::new(),
            metadata: EvidenceMetadata::default(),
        });
        let result = runtime.run(input, candidate(), &policy).unwrap();
        assert_eq!(result.final_verdict, Verdict::Accept);
        assert_eq!(
            result.attempts[0].status,
            ResolutionAttemptStatus::AppliedVerification
        );
        assert_eq!(result.attempts[0].verification_receipts, 1);
    }

    #[test]
    fn budget_exhaustion_is_not_evidence() {
        let resolver = OneEvidenceResolver {
            raw: AcquiredEvidence {
                id: "resolved-e1".into(),
                source: "fixture source".into(),
                observation: "feature enabled".into(),
                facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
                acquisition_metadata: Default::default(),
            },
        };
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = default_grounded_resolution_runtime(&PrimaryAdmission, &resolvers, &[]);
        let policy = GroundedResolutionPolicy {
            budget: ResolutionBudget {
                max_added_tokens: Some(5),
                ..ResolutionBudget::default()
            },
            ..Default::default()
        };
        let result = runtime
            .run(missing_support_input(), candidate(), &policy)
            .unwrap();
        assert_eq!(result.final_verdict, Verdict::Unknown);
        assert_eq!(result.terminal_status, ResolutionTerminalStatus::Exhausted);
        assert_eq!(
            result.attempts[0].status,
            ResolutionAttemptStatus::BudgetExceeded
        );
        assert!(result.final_artifact.evidence.is_empty());
    }

    struct ExtraClaimRenderer;

    impl FinalAnswerRenderer for ExtraClaimRenderer {
        fn render(
            &self,
            artifact: &crate::ReasoningArtifact,
            verdict: Verdict,
        ) -> crate::FinalAnswerCandidate {
            let mut candidate = CanonicalFinalAnswerRenderer.render(artifact, verdict);
            candidate.text = "feature enabled in r1".into();
            candidate.factual_claims.push(crate::FinalAnswerClaim {
                proposition: Proposition {
                    key: "deployment.region".into(),
                    value: "r1".into(),
                },
                mode: crate::FinalClaimMode::Grounded,
            });
            candidate
        }
    }

    struct FinalizationResolver;

    impl ResolutionResolver for FinalizationResolver {
        fn name(&self) -> &'static str {
            "finalization_acquirer"
        }

        fn class(&self) -> ResolverClass {
            ResolverClass::EvidenceAcquisition
        }

        fn resolve(
            &self,
            request: &ResolutionRequest,
            _attempt_index: usize,
        ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
            let proposition = request.target.proposition().unwrap();
            Ok(ResolutionResolverOutput {
                contribution: ResolutionResolverContribution::AcquiredEvidence {
                    evidence: vec![AcquiredEvidence {
                        id: "region-evidence".into(),
                        source: "fixture source".into(),
                        observation: "region r1".into(),
                        facts: BTreeMap::from([(
                            proposition.key.clone(),
                            proposition.value.clone(),
                        )]),
                        acquisition_metadata: Default::default(),
                    }],
                },
                cost: ResolutionCost::default(),
            })
        }
    }

    #[test]
    fn final_renderer_new_fact_is_routed_back_through_verification() {
        let input = HarnessInput {
            task: "feature status".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "config".into(),
                observation: "enabled".into(),
                facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
                metadata: EvidenceMetadata::default(),
            }],
            hypotheses: vec![],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: Default::default(),
        };
        let base_candidate = ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: "c1".into(),
                statement: "enabled".into(),
                proposed_state: EpistemicState::Supported,
                proposition: Some(Proposition {
                    key: "feature.enabled".into(),
                    value: "true".into(),
                }),
                evidence_ids: vec!["e1".into()],
            }],
            inferences: vec![],
        };
        let resolver = FinalizationResolver;
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = GroundedResolutionRuntime {
            pipeline: &StandardGroundingPipeline,
            planner: &DefaultResolutionPlanner,
            evidence_admission: &PrimaryAdmission,
            resolvers: &resolvers,
            trusted_verifiers: &[],
            renderer: &ExtraClaimRenderer,
        };
        let result = runtime
            .run(input, base_candidate, &GroundedResolutionPolicy::default())
            .unwrap();
        // The renderer-introduced proposition is converted into a new hypothesis,
        // receives admitted evidence, and must pass the ordinary verifier before output.
        assert_eq!(
            result.finalization.status,
            FinalizationStatus::GroundedAnswer
        );
        assert!(result.finalization.text.is_some());
        assert_eq!(result.usage.attempts, 1);
        assert!(result.final_artifact.claims.iter().any(|claim| {
            claim.proposition.as_ref().is_some_and(|proposition| {
                proposition.key == "deployment.region" && proposition.value == "r1"
            }) && claim.state == EpistemicState::Supported
        }));
    }

    #[test]
    fn per_request_attempt_budget_is_reported_separately() {
        struct NoResultResolver;
        impl ResolutionResolver for NoResultResolver {
            fn name(&self) -> &'static str {
                "no_result"
            }
            fn class(&self) -> ResolverClass {
                ResolverClass::EvidenceAcquisition
            }
            fn resolve(
                &self,
                _request: &ResolutionRequest,
                _attempt_index: usize,
            ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
                Ok(ResolutionResolverOutput {
                    contribution: ResolutionResolverContribution::NoResult,
                    cost: ResolutionCost {
                        added_tokens: 2,
                        elapsed_ms: 3,
                    },
                })
            }
        }
        struct BudgetPlanner;
        impl ResolutionPlanner for BudgetPlanner {
            fn plan(
                &self,
                outcome: &HarnessOutcome,
                _policy: &GroundedResolutionPolicy,
            ) -> Vec<ResolutionRequest> {
                if outcome.verdict != Verdict::Unknown {
                    return vec![];
                }
                vec![ResolutionRequest {
                    id: "budgeted-request".into(),
                    reason: ResolutionReason::MissingSupport,
                    target: ResolutionTarget::Proposition {
                        proposition: Proposition {
                            key: "feature.enabled".into(),
                            value: "true".into(),
                        },
                    },
                    resolver_class: ResolverClass::EvidenceAcquisition,
                    budget: ResolutionRequestBudget {
                        max_attempts: Some(1),
                        max_added_tokens: Some(5),
                        max_elapsed_ms: Some(5),
                    },
                }]
            }
        }
        let resolver = NoResultResolver;
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = GroundedResolutionRuntime {
            pipeline: &StandardGroundingPipeline,
            planner: &BudgetPlanner,
            evidence_admission: &RejectAllEvidenceAdmission,
            resolvers: &resolvers,
            trusted_verifiers: &[],
            renderer: &CanonicalFinalAnswerRenderer,
        };
        let result = runtime
            .run(
                missing_support_input(),
                candidate(),
                &GroundedResolutionPolicy::default(),
            )
            .unwrap();
        assert_eq!(result.terminal_status, ResolutionTerminalStatus::Exhausted);
        assert_eq!(result.usage.attempts, 1);
        assert_eq!(result.request_usage["budgeted-request"].attempts, 1);
        assert_eq!(result.request_usage["budgeted-request"].added_tokens, 2);
        assert_eq!(result.request_usage["budgeted-request"].elapsed_ms, 3);
    }

    struct MutatingAdmission;

    impl EvidenceAdmissionPolicy for MutatingAdmission {
        fn admit(
            &self,
            _resolver_name: &str,
            _request: &ResolutionRequest,
            acquired: &AcquiredEvidence,
        ) -> Result<Evidence, EvidenceAdmissionRejection> {
            let mut facts = acquired.facts.clone();
            facts.insert("feature.enabled".into(), "false".into());
            Ok(Evidence {
                id: acquired.id.clone(),
                source: acquired.source.clone(),
                observation: acquired.observation.clone(),
                facts,
                metadata: EvidenceMetadata::default(),
            })
        }
    }

    #[test]
    fn admission_policy_may_add_metadata_but_cannot_rewrite_acquired_facts() {
        let resolver = OneEvidenceResolver {
            raw: AcquiredEvidence {
                id: "resolved-e1".into(),
                source: "fixture source".into(),
                observation: "feature enabled".into(),
                facts: BTreeMap::from([("feature.enabled".into(), "true".into())]),
                acquisition_metadata: Default::default(),
            },
        };
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = default_grounded_resolution_runtime(&MutatingAdmission, &resolvers, &[]);
        let error = runtime
            .run(
                missing_support_input(),
                candidate(),
                &GroundedResolutionPolicy::default(),
            )
            .unwrap_err();
        assert!(matches!(error, ResolutionError::AdmissionChangedContent));
    }

    struct HumanResolver;
    impl ResolutionResolver for HumanResolver {
        fn name(&self) -> &'static str {
            "human_router"
        }
        fn class(&self) -> ResolverClass {
            ResolverClass::HumanReview
        }
        fn resolve(
            &self,
            _request: &ResolutionRequest,
            _attempt_index: usize,
        ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
            Ok(ResolutionResolverOutput {
                contribution: ResolutionResolverContribution::HumanReviewRequired,
                cost: ResolutionCost::default(),
            })
        }
    }

    struct HumanPlanner;
    impl ResolutionPlanner for HumanPlanner {
        fn plan(
            &self,
            outcome: &HarnessOutcome,
            _policy: &GroundedResolutionPolicy,
        ) -> Vec<ResolutionRequest> {
            if outcome.verdict != Verdict::Unknown {
                return vec![];
            }
            vec![ResolutionRequest {
                id: "human-review".into(),
                reason: ResolutionReason::ExplicitRequest,
                target: ResolutionTarget::HumanReview {
                    claim_id: Some("c1".into()),
                },
                resolver_class: ResolverClass::HumanReview,
                budget: ResolutionRequestBudget::default(),
            }]
        }
    }

    #[test]
    fn human_review_is_an_explicit_terminal_state() {
        let resolver = HumanResolver;
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = GroundedResolutionRuntime {
            pipeline: &StandardGroundingPipeline,
            planner: &HumanPlanner,
            evidence_admission: &RejectAllEvidenceAdmission,
            resolvers: &resolvers,
            trusted_verifiers: &[],
            renderer: &CanonicalFinalAnswerRenderer,
        };
        let policy = GroundedResolutionPolicy {
            budget: ResolutionBudget {
                allowed_resolver_classes: BTreeSet::from([ResolverClass::HumanReview]),
                ..Default::default()
            },
            allow_human_review: true,
            ..Default::default()
        };
        let result = runtime
            .run(missing_support_input(), candidate(), &policy)
            .unwrap();
        assert_eq!(
            result.terminal_status,
            ResolutionTerminalStatus::HumanReviewRequired
        );
    }

    #[test]
    fn missing_allowed_resolver_terminates_unavailable_without_fabricating_support() {
        let runtime = default_grounded_resolution_runtime(&RejectAllEvidenceAdmission, &[], &[]);
        let result = runtime
            .run(
                missing_support_input(),
                candidate(),
                &GroundedResolutionPolicy::default(),
            )
            .unwrap();
        assert_eq!(
            result.terminal_status,
            ResolutionTerminalStatus::Unavailable
        );
        assert_eq!(result.final_verdict, Verdict::Unknown);
        assert!(result.attempts.is_empty());
    }
}
