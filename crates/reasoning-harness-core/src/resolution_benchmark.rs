use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

use crate::{
    AcquiredEvidence, BenchmarkFixture, CanonicalFinalAnswerRenderer, Evidence,
    EvidenceAdmissionPolicy, EvidenceAdmissionRejection, EvidenceMetadata, FinalizationStatus,
    GroundedResolutionOutcome, GroundedResolutionPolicy, GroundedResolutionRuntime,
    ResolutionAdapterError, ResolutionAdapterErrorKind, ResolutionCost, ResolutionPlanner,
    ResolutionRequest, ResolutionResolver, ResolutionResolverContribution,
    ResolutionResolverOutput, ResolutionTerminalStatus, ResolverClass, StandardGroundingPipeline,
    Verdict, evaluate_benchmark_fixture,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionBenchmarkFixture {
    pub id: String,
    pub base_case_id: String,
    pub base_fixture_path: String,
    pub request: ResolutionRequest,
    pub policy: GroundedResolutionPolicy,
    #[serde(default)]
    pub authority_policy: crate::EvidenceAuthorityPolicy,
    #[serde(default)]
    pub resolver_steps: Vec<ResolutionFixtureStep>,
    pub expected_terminal_status: ResolutionTerminalStatus,
    pub expected_final_verdict: Verdict,
    pub expected_finalization_status: FinalizationStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionFixtureStep {
    pub result: ResolutionFixtureStepResult,
    #[serde(default)]
    pub cost: ResolutionCost,
    #[serde(default)]
    pub trusted_metadata: BTreeMap<String, EvidenceMetadata>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ResolutionFixtureStepResult {
    Evidence { evidence: Vec<AcquiredEvidence> },
    NoResult,
    HumanReviewRequired,
    AdapterError { error: ResolutionAdapterErrorKind },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ResolutionBenchmarkCaseResult {
    pub scenario_id: String,
    pub base_case_id: String,
    pub direct_one_shot_verdict: Option<Verdict>,
    pub diagnose_only_verdict: Option<Verdict>,
    pub bounded_initial_verdict: Verdict,
    pub bounded_final_verdict: Verdict,
    pub terminal_status: ResolutionTerminalStatus,
    pub finalization_status: FinalizationStatus,
    pub recovered_supported: bool,
    pub unsafe_final_answer: bool,
    pub blocked_unverified_finalization: bool,
    pub factual_claim_coverage: f64,
    pub attempts: usize,
    pub added_tokens: u64,
    pub elapsed_ms: u64,
    pub expectations_met: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct ResolutionBenchmarkAggregate {
    pub cases: usize,
    pub passed_cases: usize,
    pub initially_unknown_cases: usize,
    pub recovered_supported_cases: usize,
    pub recovery_rate: f64,
    pub resolved_qualified_cases: usize,
    pub resolved_refuted_cases: usize,
    pub exhausted_cases: usize,
    pub unavailable_cases: usize,
    pub human_review_required_cases: usize,
    pub unsafe_final_answers: usize,
    pub blocked_unverified_finalizations: usize,
    pub mean_factual_claim_coverage: f64,
    pub total_attempts: usize,
    pub mean_attempts: f64,
    pub added_tokens: u64,
    pub elapsed_ms: u64,
}

struct FixedResolutionPlanner {
    request: ResolutionRequest,
}

impl ResolutionPlanner for FixedResolutionPlanner {
    fn plan(
        &self,
        outcome: &crate::HarnessOutcome,
        _policy: &GroundedResolutionPolicy,
    ) -> Vec<ResolutionRequest> {
        if outcome.verdict == Verdict::Accept {
            vec![]
        } else {
            vec![self.request.clone()]
        }
    }
}

struct FixtureSequenceResolver {
    class: ResolverClass,
    steps: Vec<ResolutionFixtureStep>,
    cursor: AtomicUsize,
}

impl ResolutionResolver for FixtureSequenceResolver {
    fn name(&self) -> &'static str {
        "fixture_resolution_resolver"
    }

    fn class(&self) -> ResolverClass {
        self.class
    }

    fn resolve(
        &self,
        _request: &ResolutionRequest,
        _attempt_index: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
        let cursor = self.cursor.fetch_add(1, Ordering::Relaxed);
        let step = self
            .steps
            .get(cursor)
            .cloned()
            .unwrap_or(ResolutionFixtureStep {
                result: ResolutionFixtureStepResult::NoResult,
                cost: ResolutionCost::default(),
                trusted_metadata: BTreeMap::new(),
            });
        match step.result {
            ResolutionFixtureStepResult::Evidence { evidence } => Ok(ResolutionResolverOutput {
                contribution: ResolutionResolverContribution::AcquiredEvidence { evidence },
                cost: step.cost,
            }),
            ResolutionFixtureStepResult::NoResult => Ok(ResolutionResolverOutput {
                contribution: ResolutionResolverContribution::NoResult,
                cost: step.cost,
            }),
            ResolutionFixtureStepResult::HumanReviewRequired => Ok(ResolutionResolverOutput {
                contribution: ResolutionResolverContribution::HumanReviewRequired,
                cost: step.cost,
            }),
            ResolutionFixtureStepResult::AdapterError { error } => Err(ResolutionAdapterError {
                kind: error,
                cost: step.cost,
            }),
        }
    }
}

struct FixtureAdmission {
    metadata: BTreeMap<String, EvidenceMetadata>,
}

impl EvidenceAdmissionPolicy for FixtureAdmission {
    fn admit(
        &self,
        resolver_name: &str,
        _request: &ResolutionRequest,
        acquired: &AcquiredEvidence,
    ) -> Result<Evidence, EvidenceAdmissionRejection> {
        if resolver_name != "fixture_resolution_resolver" {
            return Err(EvidenceAdmissionRejection::UntrustedSource);
        }
        let metadata = self
            .metadata
            .get(&acquired.id)
            .cloned()
            .ok_or(EvidenceAdmissionRejection::MissingTrustedMetadata)?;
        Ok(Evidence {
            id: acquired.id.clone(),
            source: acquired.source.clone(),
            observation: acquired.observation.clone(),
            facts: acquired.facts.clone(),
            metadata,
        })
    }
}

pub fn evaluate_resolution_fixture(
    fixture: &ResolutionBenchmarkFixture,
    base_fixture: &BenchmarkFixture,
) -> Result<ResolutionBenchmarkCaseResult, crate::ResolutionError> {
    let ordinary =
        evaluate_benchmark_fixture(base_fixture, base_fixture.recorded_candidate.clone());
    let mut input = base_fixture.input.clone();
    if !fixture.authority_policy.ranks.is_empty() {
        input.authority_policy = fixture.authority_policy.clone();
    }

    let metadata = fixture
        .resolver_steps
        .iter()
        .flat_map(|step| step.trusted_metadata.clone())
        .collect::<BTreeMap<_, _>>();
    let admission = FixtureAdmission { metadata };
    let resolver = FixtureSequenceResolver {
        class: fixture.request.resolver_class,
        steps: fixture.resolver_steps.clone(),
        cursor: AtomicUsize::new(0),
    };
    let planner = FixedResolutionPlanner {
        request: fixture.request.clone(),
    };
    let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
    let runtime = GroundedResolutionRuntime {
        pipeline: &StandardGroundingPipeline,
        planner: &planner,
        evidence_admission: &admission,
        resolvers: &resolvers,
        trusted_verifiers: &[],
        renderer: &CanonicalFinalAnswerRenderer,
    };
    let bounded = runtime.run(
        input,
        base_fixture.recorded_candidate.clone(),
        &fixture.policy,
    )?;
    Ok(case_result(fixture, ordinary, bounded))
}

fn case_result(
    fixture: &ResolutionBenchmarkFixture,
    ordinary: crate::BenchmarkCaseResult,
    bounded: GroundedResolutionOutcome,
) -> ResolutionBenchmarkCaseResult {
    let recovered_supported = bounded.initial_verdict == Verdict::Unknown
        && bounded.final_verdict == Verdict::Accept
        && bounded.terminal_status == ResolutionTerminalStatus::ResolvedSupported;
    let blocked_unverified_finalization =
        bounded.finalization.status == FinalizationStatus::RequiresVerification;
    let unsafe_final_answer =
        bounded.finalization.text.is_some() && bounded.finalization.factual_claim_coverage < 1.0;
    let expectations_met = bounded.terminal_status == fixture.expected_terminal_status
        && bounded.final_verdict == fixture.expected_final_verdict
        && bounded.finalization.status == fixture.expected_finalization_status;
    ResolutionBenchmarkCaseResult {
        scenario_id: fixture.id.clone(),
        base_case_id: fixture.base_case_id.clone(),
        direct_one_shot_verdict: ordinary.baseline.verdict,
        diagnose_only_verdict: ordinary.harness.verdict,
        bounded_initial_verdict: bounded.initial_verdict,
        bounded_final_verdict: bounded.final_verdict,
        terminal_status: bounded.terminal_status,
        finalization_status: bounded.finalization.status,
        recovered_supported,
        unsafe_final_answer,
        blocked_unverified_finalization,
        factual_claim_coverage: bounded.finalization.factual_claim_coverage,
        attempts: bounded.usage.attempts,
        added_tokens: bounded.usage.added_tokens,
        elapsed_ms: bounded.usage.elapsed_ms,
        expectations_met,
    }
}

pub fn aggregate_resolution_benchmark(
    results: &[ResolutionBenchmarkCaseResult],
) -> ResolutionBenchmarkAggregate {
    let initially_unknown_cases = results
        .iter()
        .filter(|result| result.bounded_initial_verdict == Verdict::Unknown)
        .count();
    let recovered_supported_cases = results
        .iter()
        .filter(|result| result.recovered_supported)
        .count();
    let total_attempts = results.iter().map(|result| result.attempts).sum();
    ResolutionBenchmarkAggregate {
        cases: results.len(),
        passed_cases: results
            .iter()
            .filter(|result| result.expectations_met)
            .count(),
        initially_unknown_cases,
        recovered_supported_cases,
        recovery_rate: rate(recovered_supported_cases, initially_unknown_cases),
        resolved_qualified_cases: terminal_count(
            results,
            ResolutionTerminalStatus::ResolvedQualified,
        ),
        resolved_refuted_cases: terminal_count(results, ResolutionTerminalStatus::ResolvedRefuted),
        exhausted_cases: terminal_count(results, ResolutionTerminalStatus::Exhausted),
        unavailable_cases: terminal_count(results, ResolutionTerminalStatus::Unavailable),
        human_review_required_cases: terminal_count(
            results,
            ResolutionTerminalStatus::HumanReviewRequired,
        ),
        unsafe_final_answers: results
            .iter()
            .filter(|result| result.unsafe_final_answer)
            .count(),
        blocked_unverified_finalizations: results
            .iter()
            .filter(|result| result.blocked_unverified_finalization)
            .count(),
        mean_factual_claim_coverage: mean(
            results.iter().map(|result| result.factual_claim_coverage),
            results.len(),
        ),
        total_attempts,
        mean_attempts: mean(
            results.iter().map(|result| result.attempts as f64),
            results.len(),
        ),
        added_tokens: results.iter().map(|result| result.added_tokens).sum(),
        elapsed_ms: results.iter().map(|result| result.elapsed_ms).sum(),
    }
}

fn terminal_count(
    results: &[ResolutionBenchmarkCaseResult],
    terminal: ResolutionTerminalStatus,
) -> usize {
    results
        .iter()
        .filter(|result| result.terminal_status == terminal)
        .count()
}

fn mean(values: impl Iterator<Item = f64>, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        values.sum::<f64>() / count as f64
    }
}

fn rate(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}
