//! Offline controls for already-admitted investigation snapshots.
use super::*;
use reasoning_harness_core::{AcquiredEvidenceMetadata, ResolutionAttemptStatus};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Resolver {
    evidence: Vec<AcquiredEvidence>,
    calls: AtomicUsize,
}
impl Resolver {
    fn new(evidence: Vec<AcquiredEvidence>) -> Self {
        Self {
            evidence,
            calls: AtomicUsize::new(0),
        }
    }
}
impl ResolutionResolver for Resolver {
    fn name(&self) -> &'static str {
        "fixture_resolver"
    }
    fn class(&self) -> ResolverClass {
        ResolverClass::EvidenceAcquisition
    }
    fn config_id(&self) -> Option<&str> {
        Some("fixture-config")
    }
    fn resolve(
        &self,
        _: &ResolutionRequest,
        _: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(ResolutionResolverOutput {
            contribution: ResolutionResolverContribution::AcquiredEvidence {
                evidence: self.evidence.clone(),
            },
            cost: ResolutionCost {
                calls: 1,
                added_tokens: 7,
                elapsed_ms: 3,
                cost_microusd: Some(4),
            },
        })
    }
}
fn raw() -> AcquiredEvidence {
    AcquiredEvidence {
        id: "snapshot-1".into(),
        source: "source-1".into(),
        observation: "value=4".into(),
        facts: BTreeMap::from([("unit.value".into(), "4".into())]),
        acquisition_metadata: AcquiredEvidenceMetadata {
            observed_at_unix_seconds: Some(1000),
            retrieved_at_unix_seconds: Some(1000),
            scope: None,
            claimed_authority_class: Some("primary".into()),
        },
    }
}
fn config() -> ExternalEvidenceAdmissionConfig {
    ExternalEvidenceAdmissionConfig {
        resolver_name: "fixture_resolver",
        evaluation_time_unix_seconds: 1010,
        authority_policy: EvidenceAuthorityPolicy {
            ranks: BTreeMap::from([("primary".into(), 20)]),
        },
        minimum_authority_class: Some("primary".into()),
        required_scope: None,
        sources: BTreeMap::from([(
            "source-1".into(),
            ExternalEvidenceSourcePolicy {
                authority_class: "primary".into(),
                max_age_seconds: 60,
                scope: None,
            },
        )]),
    }
}
fn request(target: &str) -> ResolutionRequest {
    ResolutionRequest {
        id: format!("investigation:{target}:fixture"),
        reason: ResolutionReason::Investigation,
        target: ResolutionTarget::InvestigationQuestion {
            target_id: target.into(),
            question: "What is the value?".into(),
            expected_fact_key: Some("unit.value".into()),
        },
        resolver_class: ResolverClass::EvidenceAcquisition,
        budget: ResolutionRequestBudget {
            max_attempts: Some(1),
            ..Default::default()
        },
    }
}
fn first_admission() -> (HarnessInput, ReasoningCandidate, GroundedResolutionOutcome) {
    let input = HarnessInput {
        task: "Determine value".into(),
        ..Default::default()
    };
    let candidate = ReasoningCandidate::default();
    let resolver = Resolver::new(vec![raw()]);
    let admission = ExternalEvidenceAdmissionPolicy::new(config());
    let first = run_investigation_resolution(
        input,
        candidate.clone(),
        &resolver,
        Some(&admission),
        request("first"),
    )
    .unwrap();
    assert_eq!(first.attempts.len(), 1);
    assert_eq!(
        first.attempts[0].status,
        ResolutionAttemptStatus::AppliedEvidence
    );
    assert_eq!(first.attempts[0].admitted_evidence_ids, vec!["snapshot-1"]);
    (input_from_artifact(&first.final_artifact), candidate, first)
}
fn repeat(
    input: HarnessInput,
    candidate: ReasoningCandidate,
    evidence: Vec<AcquiredEvidence>,
    admission: Option<&ExternalEvidenceAdmissionPolicy>,
) -> GroundedResolutionOutcome {
    let resolver = Resolver::new(evidence);
    let result =
        run_investigation_resolution(input, candidate, &resolver, admission, request("second"))
            .unwrap();
    assert_eq!(resolver.calls.load(Ordering::SeqCst), 1);
    result
}
#[test]
fn first_admission_then_identical_snapshot_is_no_result_with_actual_cost() {
    let (input, candidate, first) = first_admission();
    let policy = ExternalEvidenceAdmissionPolicy::new(config());
    let result = repeat(input, candidate, vec![raw()], Some(&policy));
    assert_eq!(result.attempts.len(), 1);
    let attempt = &result.attempts[0];
    assert_eq!(attempt.status, ResolutionAttemptStatus::NoResult);
    assert!(attempt.admitted_evidence_ids.is_empty());
    assert_eq!(attempt.adapter_name, "fixture_resolver");
    assert_eq!(attempt.adapter_config_id.as_deref(), Some("fixture-config"));
    assert_eq!(attempt.cost.calls, 1);
    assert_eq!(attempt.cost.added_tokens, 7);
    assert_eq!(attempt.cost.elapsed_ms, 3);
    assert_eq!(attempt.cost.cost_microusd, Some(4));
    assert_eq!(result.usage.calls, 1);
    assert_eq!(result.usage.added_tokens, 7);
    assert_eq!(
        result.final_artifact.evidence,
        first.final_artifact.evidence
    );
    assert_eq!(result.final_artifact.claims, first.final_artifact.claims);
    assert_eq!(
        result.final_artifact.verification_receipts,
        first.final_artifact.verification_receipts
    );
}
#[test]
fn changed_raw_or_no_admission_stays_on_original_rejection_path() {
    let (input, candidate, _) = first_admission();
    let admission = ExternalEvidenceAdmissionPolicy::new(config());
    let mut variants = Vec::new();
    let mut value = raw();
    value.source = "unknown-source".into();
    variants.push(value);
    let mut value = raw();
    value.observation = "changed".into();
    variants.push(value);
    let mut value = raw();
    value.facts.insert("unit.value".into(), "5".into());
    variants.push(value);
    let mut value = raw();
    value.acquisition_metadata.observed_at_unix_seconds = Some(1001);
    variants.push(value);
    let mut value = raw();
    value.acquisition_metadata.claimed_authority_class = Some("secondary".into());
    variants.push(value);
    let mut value = raw();
    value.acquisition_metadata.observed_at_unix_seconds = None;
    variants.push(value);
    let mut value = raw();
    value.acquisition_metadata.scope = Some(BTreeMap::from([(
        "region".into(),
        ScopeCoverage::Values {
            values: BTreeSet::from(["other".into()]),
        },
    )]));
    variants.push(value);
    for value in variants {
        let result = repeat(
            input.clone(),
            candidate.clone(),
            vec![value],
            Some(&admission),
        );
        assert_eq!(
            result.attempts[0].status,
            ResolutionAttemptStatus::MalformedOutput
        );
        assert!(result.attempts[0].admitted_evidence_ids.is_empty());
        assert_eq!(result.final_artifact.evidence, input.evidence);
    }
    let result = repeat(input.clone(), candidate.clone(), vec![raw()], None);
    assert_eq!(
        result.attempts[0].status,
        ResolutionAttemptStatus::MalformedOutput
    );
    let mut expired = config();
    expired.evaluation_time_unix_seconds = 1100;
    let result = repeat(
        input,
        candidate,
        vec![raw()],
        Some(&ExternalEvidenceAdmissionPolicy::new(expired)),
    );
    assert_eq!(
        result.attempts[0].status,
        ResolutionAttemptStatus::MalformedOutput
    );
}
#[test]
fn newer_retrieval_does_not_refresh_original_observation_clock() {
    let (input, candidate, first) = first_admission();
    let mut acquired = raw();
    acquired.acquisition_metadata.retrieved_at_unix_seconds = Some(1009);
    let policy = ExternalEvidenceAdmissionPolicy::new(config());
    let result = repeat(input, candidate, vec![acquired], Some(&policy));
    assert_eq!(result.attempts[0].status, ResolutionAttemptStatus::NoResult);
    assert_eq!(
        result.final_artifact.evidence,
        first.final_artifact.evidence
    );
    let temporal = result.final_artifact.evidence[0]
        .metadata
        .temporal
        .as_ref()
        .unwrap();
    assert_eq!(temporal.effective_from_unix_seconds, Some(1000));
    assert_eq!(temporal.effective_until_unix_seconds, Some(1060));
}
#[test]
fn mixed_or_duplicate_batch_and_ambiguous_existing_ids_are_not_suppressed() {
    let (input, candidate, _) = first_admission();
    let policy = ExternalEvidenceAdmissionPolicy::new(config());
    let mut fresh = raw();
    fresh.id = "snapshot-2".into();
    for batch in [
        vec![raw(), fresh.clone()],
        vec![fresh, raw()],
        vec![raw(), raw()],
    ] {
        let result = repeat(input.clone(), candidate.clone(), batch, Some(&policy));
        assert_eq!(
            result.attempts[0].status,
            ResolutionAttemptStatus::MalformedOutput
        );
        assert_eq!(result.final_artifact.evidence, input.evidence);
        assert!(result.attempts[0].admitted_evidence_ids.is_empty());
    }
    let resolver = Resolver::new(vec![raw()]);
    let duplicated = vec![input.evidence[0].clone(), input.evidence[0].clone()];
    let wrapper = AcquisitionOnlyResolver {
        inner: &resolver,
        existing_evidence: &duplicated,
        admission: Some(&policy),
    };
    assert!(matches!(
        wrapper.resolve(&request("other"), 0).unwrap().contribution,
        ResolutionResolverContribution::AcquiredEvidence { .. }
    ));
}
#[test]
fn empty_and_new_batches_keep_existing_behavior() {
    let (input, candidate, _) = first_admission();
    let policy = ExternalEvidenceAdmissionPolicy::new(config());
    let mut fresh = raw();
    fresh.id = "snapshot-2".into();
    let result = repeat(input.clone(), candidate.clone(), vec![fresh], Some(&policy));
    assert_eq!(
        result.attempts[0].status,
        ResolutionAttemptStatus::AppliedEvidence
    );
    assert_eq!(result.attempts[0].admitted_evidence_ids, vec!["snapshot-2"]);
    let resolver = Resolver::new(vec![]);
    let wrapper = AcquisitionOnlyResolver {
        inner: &resolver,
        existing_evidence: &input.evidence,
        admission: Some(&policy),
    };
    assert!(
        matches!(wrapper.resolve(&request("other"),0).unwrap().contribution,ResolutionResolverContribution::AcquiredEvidence { evidence } if evidence.is_empty())
    );
}
