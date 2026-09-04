use std::collections::BTreeMap;

pub const EXTERNAL_EVIDENCE_ADMISSION_ID: &str = "external_evidence_admission_v1";

use reasoning_harness_core::{
    AcquiredEvidence, ApplicabilityScope, Evidence, EvidenceAdmissionPolicy,
    EvidenceAdmissionRejection, EvidenceAuthorityPolicy, EvidenceMetadata, ResolutionRequest,
    ResolutionTarget, ScopeCoverage, TemporalValidity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalEvidenceSourcePolicy {
    /// Harness-owned authority class assigned to this exact source identity.
    pub authority_class: String,
    /// Maximum age, relative to the Harness-owned evaluation time, for admitted observations.
    pub max_age_seconds: u64,
    /// Maximum applicability scope this source is configured to claim.
    pub scope: Option<ApplicabilityScope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalEvidenceAdmissionConfig {
    pub resolver_name: &'static str,
    pub evaluation_time_unix_seconds: i64,
    pub authority_policy: EvidenceAuthorityPolicy,
    pub minimum_authority_class: Option<String>,
    pub required_scope: Option<ApplicabilityScope>,
    pub sources: BTreeMap<String, ExternalEvidenceSourcePolicy>,
}

#[derive(Debug, Clone)]
pub struct ExternalEvidenceAdmissionPolicy {
    config: ExternalEvidenceAdmissionConfig,
}

impl ExternalEvidenceAdmissionPolicy {
    pub fn new(config: ExternalEvidenceAdmissionConfig) -> Self {
        Self { config }
    }

    pub fn authority_policy(&self) -> &EvidenceAuthorityPolicy {
        &self.config.authority_policy
    }

    pub fn minimum_authority_class(&self) -> Option<&str> {
        self.config.minimum_authority_class.as_deref()
    }

    pub fn required_scope(&self) -> Option<&ApplicabilityScope> {
        self.config.required_scope.as_ref()
    }

    pub fn evaluation_time_unix_seconds(&self) -> i64 {
        self.config.evaluation_time_unix_seconds
    }
}

impl EvidenceAdmissionPolicy for ExternalEvidenceAdmissionPolicy {
    fn admit(
        &self,
        resolver_name: &str,
        request: &ResolutionRequest,
        acquired: &AcquiredEvidence,
    ) -> Result<Evidence, EvidenceAdmissionRejection> {
        if resolver_name != self.config.resolver_name {
            return Err(EvidenceAdmissionRejection::UntrustedSource);
        }
        let source_policy = self
            .config
            .sources
            .get(&acquired.source)
            .ok_or(EvidenceAdmissionRejection::UntrustedSource)?;

        let observed_at = acquired
            .acquisition_metadata
            .observed_at_unix_seconds
            .ok_or(EvidenceAdmissionRejection::MissingObservationTime)?;
        let retrieved_at = acquired
            .acquisition_metadata
            .retrieved_at_unix_seconds
            .ok_or(EvidenceAdmissionRejection::MissingRetrievalTime)?;
        if retrieved_at < observed_at || retrieved_at > self.config.evaluation_time_unix_seconds {
            return Err(EvidenceAdmissionRejection::InvalidEvidence);
        }
        let claimed_authority = acquired
            .acquisition_metadata
            .claimed_authority_class
            .as_deref()
            .filter(|class| !class.trim().is_empty())
            .ok_or(EvidenceAdmissionRejection::MissingAuthorityClaim)?;
        if claimed_authority != source_policy.authority_class {
            return Err(EvidenceAdmissionRejection::AuthorityClaimMismatch);
        }

        let source_rank = self
            .config
            .authority_policy
            .ranks
            .get(&source_policy.authority_class)
            .copied()
            .ok_or(EvidenceAdmissionRejection::UnknownAuthorityClass)?;
        if let Some(minimum_class) = strongest_required_authority(request, &self.config)? {
            let minimum_rank = self
                .config
                .authority_policy
                .ranks
                .get(minimum_class)
                .copied()
                .ok_or(EvidenceAdmissionRejection::UnknownAuthorityClass)?;
            if source_rank < minimum_rank {
                return Err(EvidenceAdmissionRejection::InsufficientAuthority);
            }
        }

        let evaluation_time =
            request_evaluation_time(request).unwrap_or(self.config.evaluation_time_unix_seconds);
        if observed_at > evaluation_time {
            return Err(EvidenceAdmissionRejection::NotYetValid);
        }
        let max_age = i64::try_from(source_policy.max_age_seconds)
            .map_err(|_| EvidenceAdmissionRejection::InvalidEvidence)?;
        if evaluation_time.saturating_sub(observed_at) > max_age {
            return Err(EvidenceAdmissionRejection::Stale);
        }

        let required_scope =
            request_required_scope(request).or(self.config.required_scope.as_ref());
        let acquired_scope = acquired.acquisition_metadata.scope.as_ref();
        if (source_policy.scope.is_some() || required_scope.is_some()) && acquired_scope.is_none() {
            return Err(EvidenceAdmissionRejection::MissingScopeMetadata);
        }
        if let (Some(scope), Some(allowed)) = (acquired_scope, source_policy.scope.as_ref()) {
            ensure_scope_within_allowed(scope, allowed)?;
        }
        if let (Some(scope), Some(required)) = (acquired_scope, required_scope) {
            ensure_scope_covers_required(scope, required)?;
        }

        let effective_until = observed_at
            .checked_add(max_age)
            .ok_or(EvidenceAdmissionRejection::InvalidEvidence)?;
        Ok(Evidence {
            id: acquired.id.clone(),
            source: acquired.source.clone(),
            observation: acquired.observation.clone(),
            facts: acquired.facts.clone(),
            metadata: EvidenceMetadata {
                temporal: Some(TemporalValidity {
                    effective_from_unix_seconds: Some(observed_at),
                    effective_until_unix_seconds: Some(effective_until),
                }),
                scope: acquired_scope.cloned(),
                // Authority comes from Harness configuration, never from the resolver claim.
                provenance_class: Some(source_policy.authority_class.clone()),
            },
        })
    }
}

fn request_evaluation_time(request: &ResolutionRequest) -> Option<i64> {
    match &request.target {
        ResolutionTarget::EvidenceQualification { requirement } => requirement.as_of_unix_seconds,
        _ => None,
    }
}

fn request_required_scope(request: &ResolutionRequest) -> Option<&ApplicabilityScope> {
    match &request.target {
        ResolutionTarget::EvidenceQualification { requirement } => requirement.scope.as_ref(),
        _ => None,
    }
}

fn strongest_required_authority<'a>(
    request: &'a ResolutionRequest,
    config: &'a ExternalEvidenceAdmissionConfig,
) -> Result<Option<&'a str>, EvidenceAdmissionRejection> {
    let request_class = match &request.target {
        ResolutionTarget::EvidenceQualification { requirement } => {
            requirement.minimum_authority_class.as_deref()
        }
        _ => None,
    };
    let configured = config.minimum_authority_class.as_deref();
    match (configured, request_class) {
        (None, None) => Ok(None),
        (Some(class), None) | (None, Some(class)) => Ok(Some(class)),
        (Some(left), Some(right)) => {
            let left_rank = config
                .authority_policy
                .ranks
                .get(left)
                .ok_or(EvidenceAdmissionRejection::UnknownAuthorityClass)?;
            let right_rank = config
                .authority_policy
                .ranks
                .get(right)
                .ok_or(EvidenceAdmissionRejection::UnknownAuthorityClass)?;
            Ok(Some(if left_rank >= right_rank { left } else { right }))
        }
    }
}

fn ensure_scope_within_allowed(
    evidence: &ApplicabilityScope,
    allowed: &ApplicabilityScope,
) -> Result<(), EvidenceAdmissionRejection> {
    for (dimension, evidence_coverage) in evidence {
        let allowed_coverage = allowed
            .get(dimension)
            .ok_or(EvidenceAdmissionRejection::ScopeMismatch)?;
        match (evidence_coverage, allowed_coverage) {
            (_, ScopeCoverage::Any) => {}
            (ScopeCoverage::Any, ScopeCoverage::Values { .. }) => {
                return Err(EvidenceAdmissionRejection::ScopeExpansion);
            }
            (
                ScopeCoverage::Values {
                    values: evidence_values,
                },
                ScopeCoverage::Values {
                    values: allowed_values,
                },
            ) if evidence_values.is_subset(allowed_values) => {}
            (ScopeCoverage::Values { .. }, ScopeCoverage::Values { .. }) => {
                return Err(EvidenceAdmissionRejection::ScopeExpansion);
            }
        }
    }
    Ok(())
}

fn ensure_scope_covers_required(
    evidence: &ApplicabilityScope,
    required: &ApplicabilityScope,
) -> Result<(), EvidenceAdmissionRejection> {
    for (dimension, required_coverage) in required {
        let evidence_coverage = evidence
            .get(dimension)
            .ok_or(EvidenceAdmissionRejection::ScopeMismatch)?;
        match (evidence_coverage, required_coverage) {
            (ScopeCoverage::Any, _) => {}
            (ScopeCoverage::Values { .. }, ScopeCoverage::Any) => {
                return Err(EvidenceAdmissionRejection::ScopeExpansion);
            }
            (
                ScopeCoverage::Values {
                    values: evidence_values,
                },
                ScopeCoverage::Values {
                    values: required_values,
                },
            ) if required_values.is_subset(evidence_values) => {}
            (
                ScopeCoverage::Values {
                    values: evidence_values,
                },
                ScopeCoverage::Values {
                    values: required_values,
                },
            ) if required_values.is_disjoint(evidence_values) => {
                return Err(EvidenceAdmissionRejection::ScopeMismatch);
            }
            (ScopeCoverage::Values { .. }, ScopeCoverage::Values { .. }) => {
                return Err(EvidenceAdmissionRejection::ScopeExpansion);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use reasoning_harness_core::{
        AcquiredEvidenceMetadata, EvidenceRequirement, Proposition, ResolutionReason,
        ResolutionRequestBudget, ResolverClass,
    };

    use super::*;

    fn values(items: &[&str]) -> ScopeCoverage {
        ScopeCoverage::Values {
            values: items.iter().map(|item| (*item).to_string()).collect(),
        }
    }

    fn scope(region: &[&str]) -> ApplicabilityScope {
        BTreeMap::from([("region".into(), values(region))])
    }

    fn policy() -> ExternalEvidenceAdmissionPolicy {
        ExternalEvidenceAdmissionPolicy::new(ExternalEvidenceAdmissionConfig {
            resolver_name: "external_command_v1",
            evaluation_time_unix_seconds: 1_000,
            authority_policy: EvidenceAuthorityPolicy {
                ranks: BTreeMap::from([("secondary".into(), 10), ("primary".into(), 20)]),
            },
            minimum_authority_class: Some("primary".into()),
            required_scope: Some(scope(&["eu-west-1"])),
            sources: BTreeMap::from([(
                "api:trusted".into(),
                ExternalEvidenceSourcePolicy {
                    authority_class: "primary".into(),
                    max_age_seconds: 60,
                    scope: Some(scope(&["eu-west-1", "eu-west-2"])),
                },
            )]),
        })
    }

    fn request() -> ResolutionRequest {
        ResolutionRequest {
            id: "resolution:test".into(),
            reason: ResolutionReason::EvidenceQualification,
            target: ResolutionTarget::EvidenceQualification {
                requirement: EvidenceRequirement {
                    proposition: Proposition {
                        key: "service.region".into(),
                        value: "eu-west-1".into(),
                    },
                    as_of_unix_seconds: Some(1_000),
                    scope: Some(scope(&["eu-west-1"])),
                    minimum_authority_class: Some("primary".into()),
                },
            },
            resolver_class: ResolverClass::EvidenceAcquisition,
            budget: ResolutionRequestBudget::default(),
        }
    }

    fn acquired() -> AcquiredEvidence {
        AcquiredEvidence {
            id: "e1".into(),
            source: "api:trusted".into(),
            observation: "service.region=eu-west-1".into(),
            facts: BTreeMap::from([("service.region".into(), "eu-west-1".into())]),
            acquisition_metadata: AcquiredEvidenceMetadata {
                observed_at_unix_seconds: Some(980),
                retrieved_at_unix_seconds: Some(990),
                scope: Some(scope(&["eu-west-1"])),
                claimed_authority_class: Some("primary".into()),
            },
        }
    }

    #[test]
    fn configured_source_can_admit_fresh_scoped_evidence_without_self_authority() {
        let evidence = policy()
            .admit("external_command_v1", &request(), &acquired())
            .unwrap();
        assert_eq!(
            evidence.metadata.provenance_class.as_deref(),
            Some("primary")
        );
        assert_eq!(
            evidence
                .metadata
                .temporal
                .unwrap()
                .effective_until_unix_seconds,
            Some(1_040)
        );
    }

    #[test]
    fn stale_wrong_scope_and_authority_claims_fail_closed() {
        let admission = policy();

        let mut stale = acquired();
        stale.acquisition_metadata.observed_at_unix_seconds = Some(900);
        assert_eq!(
            admission.admit("external_command_v1", &request(), &stale),
            Err(EvidenceAdmissionRejection::Stale)
        );

        let mut wrong_scope = acquired();
        wrong_scope.acquisition_metadata.scope = Some(scope(&["us-east-1"]));
        assert!(matches!(
            admission.admit("external_command_v1", &request(), &wrong_scope),
            Err(EvidenceAdmissionRejection::ScopeExpansion
                | EvidenceAdmissionRejection::ScopeMismatch)
        ));

        let mut self_elevated = acquired();
        self_elevated.acquisition_metadata.claimed_authority_class = Some("superuser".into());
        assert_eq!(
            admission.admit("external_command_v1", &request(), &self_elevated),
            Err(EvidenceAdmissionRejection::AuthorityClaimMismatch)
        );
    }

    #[test]
    fn insufficient_configured_authority_and_invalid_retrieval_time_fail_closed() {
        let mut admission = policy();
        admission
            .config
            .sources
            .get_mut("api:trusted")
            .unwrap()
            .authority_class = "secondary".into();
        let mut lower_authority = acquired();
        lower_authority.acquisition_metadata.claimed_authority_class = Some("secondary".into());
        assert_eq!(
            admission.admit("external_command_v1", &request(), &lower_authority),
            Err(EvidenceAdmissionRejection::InsufficientAuthority)
        );

        let admission = policy();
        let mut missing_retrieval = acquired();
        missing_retrieval
            .acquisition_metadata
            .retrieved_at_unix_seconds = None;
        assert_eq!(
            admission.admit("external_command_v1", &request(), &missing_retrieval),
            Err(EvidenceAdmissionRejection::MissingRetrievalTime)
        );

        let mut reversed = acquired();
        reversed.acquisition_metadata.retrieved_at_unix_seconds = Some(970);
        assert_eq!(
            admission.admit("external_command_v1", &request(), &reversed),
            Err(EvidenceAdmissionRejection::InvalidEvidence)
        );
    }

    #[test]
    fn missing_normalized_metadata_is_machine_classified() {
        let admission = policy();
        let mut raw = acquired();
        raw.acquisition_metadata = AcquiredEvidenceMetadata::default();
        assert_eq!(
            admission.admit("external_command_v1", &request(), &raw),
            Err(EvidenceAdmissionRejection::MissingObservationTime)
        );
    }

    #[derive(Debug)]
    struct FixedExternalResolver {
        raw: AcquiredEvidence,
    }

    impl reasoning_harness_core::ResolutionResolver for FixedExternalResolver {
        fn name(&self) -> &'static str {
            "external_command_v1"
        }

        fn class(&self) -> ResolverClass {
            ResolverClass::EvidenceAcquisition
        }

        fn resolve(
            &self,
            _request: &ResolutionRequest,
            _attempt_index: usize,
        ) -> Result<
            reasoning_harness_core::ResolutionResolverOutput,
            reasoning_harness_core::ResolutionAdapterError,
        > {
            Ok(reasoning_harness_core::ResolutionResolverOutput {
                contribution:
                    reasoning_harness_core::ResolutionResolverContribution::AcquiredEvidence {
                        evidence: vec![self.raw.clone()],
                    },
                cost: reasoning_harness_core::ResolutionCost::default(),
            })
        }
    }

    fn resolution_input(
        admission: &ExternalEvidenceAdmissionPolicy,
    ) -> reasoning_harness_core::HarnessInput {
        let requirement = match request().target {
            ResolutionTarget::EvidenceQualification { requirement } => requirement,
            _ => unreachable!(),
        };
        reasoning_harness_core::HarnessInput {
            task: "determine service region".into(),
            evidence: vec![],
            hypotheses: vec![requirement.proposition.clone()],
            assumptions: vec![],
            evidence_requirements: vec![requirement],
            authority_policy: admission.authority_policy().clone(),
        }
    }

    fn run_resolution(
        admission: &ExternalEvidenceAdmissionPolicy,
        raw: AcquiredEvidence,
    ) -> reasoning_harness_core::GroundedResolutionOutcome {
        use reasoning_harness_core::{
            CanonicalFinalAnswerRenderer, DefaultResolutionPlanner, GroundedResolutionPolicy,
            GroundedResolutionRuntime, ResolutionResolver, StandardGroundingPipeline,
        };
        let resolver = FixedExternalResolver { raw };
        let resolver_refs: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = GroundedResolutionRuntime {
            pipeline: &StandardGroundingPipeline,
            planner: &DefaultResolutionPlanner,
            evidence_admission: admission,
            resolvers: &resolver_refs,
            trusted_verifiers: &[],
            renderer: &CanonicalFinalAnswerRenderer,
        };
        let mut policy = GroundedResolutionPolicy::default();
        policy.budget.required_authority_class = Some("primary".into());
        runtime
            .run(
                resolution_input(admission),
                reasoning_harness_core::ReasoningCandidate::default(),
                &policy,
            )
            .unwrap()
    }

    #[test]
    fn admitted_external_evidence_reenters_ordinary_verification_before_accept() {
        use reasoning_harness_core::{FinalizationStatus, ResolutionAttemptStatus, Verdict};
        let admission = policy();
        let outcome = run_resolution(&admission, acquired());
        assert_eq!(outcome.initial_verdict, Verdict::Unknown);
        assert_eq!(outcome.final_verdict, Verdict::Accept);
        assert_eq!(outcome.attempts.len(), 1);
        assert_eq!(
            outcome.attempts[0].status,
            ResolutionAttemptStatus::AppliedEvidence
        );
        assert_eq!(outcome.attempts[0].admission_rejection, None);
        assert_eq!(
            outcome.finalization.status,
            FinalizationStatus::GroundedAnswer
        );
        assert!(
            outcome
                .final_artifact
                .verification_receipts
                .iter()
                .any(|receipt| {
                    receipt.proposition.as_ref().is_some_and(|proposition| {
                        proposition.key == "service.region" && proposition.value == "eu-west-1"
                    })
                })
        );
    }

    #[test]
    fn stale_external_evidence_stays_unknown_with_typed_admission_telemetry() {
        use reasoning_harness_core::{ResolutionAttemptStatus, Verdict};
        let admission = policy();
        let mut raw = acquired();
        raw.acquisition_metadata.observed_at_unix_seconds = Some(900);
        let outcome = run_resolution(&admission, raw);
        assert_eq!(outcome.final_verdict, Verdict::Unknown);
        assert!(!outcome.attempts.is_empty());
        assert!(outcome.attempts.iter().all(|attempt| {
            attempt.status == ResolutionAttemptStatus::RejectedUntrustedEvidence
                && attempt.admission_rejection == Some(EvidenceAdmissionRejection::Stale)
        }));
        assert!(outcome.final_artifact.verification_receipts.is_empty());
    }

    #[test]
    fn scope_helper_does_not_accept_broader_untrusted_claim() {
        let evidence = BTreeMap::from([("region".into(), ScopeCoverage::Any)]);
        let allowed = BTreeMap::from([(
            "region".into(),
            ScopeCoverage::Values {
                values: BTreeSet::from(["eu-west-1".into()]),
            },
        )]);
        assert_eq!(
            ensure_scope_within_allowed(&evidence, &allowed),
            Err(EvidenceAdmissionRejection::ScopeExpansion)
        );
    }
}
