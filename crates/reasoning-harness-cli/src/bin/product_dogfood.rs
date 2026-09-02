use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    time::Instant,
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    AcquiredEvidence, AnswerSafetyDisposition, AnswerSafetyIdentity, AnswerSafetyObservation,
    AnswerSafetyProfile, CanonicalFinalAnswerRenderer, Claim, DefaultResolutionPlanner,
    EpistemicState, Evidence, EvidenceAdmissionPolicy, EvidenceAdmissionRejection,
    EvidenceMetadata, FinalAnswerCandidate, FinalAnswerClaim, FinalClaimMode, FinalizationPolicy,
    FinalizationResult, FinalizationStatus, GroundedResolutionOutcome, GroundedResolutionPolicy,
    GroundedResolutionRuntime, GroundingPipeline, HarnessInput, ModelAdapter, ModelError,
    ModelOutputFormat, ModelRequest, ModelResponse, ModelUsage, Proposition, ReasoningCandidate,
    ResolutionAdapterError, ResolutionAttempt, ResolutionAttemptStatus, ResolutionCost,
    ResolutionRequest, ResolutionResolver, ResolutionResolverContribution,
    ResolutionResolverOutput, ResolutionTarget, ResolverClass, StandardGroundingPipeline, Verdict,
    VerificationConclusion, build_candidate_json_fallback_request, build_candidate_request,
    build_final_answer_json_fallback_request, build_final_answer_request,
    canonical_verified_target_answer, canonical_verified_target_partial_answer,
    final_answer_candidate_schema, finalize_answer, run_answer_safety_gate,
    structured_fact_verifier_for_input,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter, NvidiaAdapter};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const PRODUCT_DOGFOOD_COMPARISON_CONTRACT_ID: &str = "shared-candidate-initial-render-v1";

#[derive(Debug, Parser)]
#[command(name = "reason-product-dogfood")]
struct Args {
    #[arg(long, default_value = "fixtures/product-dogfood-v1")]
    fixtures: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long)]
    model: String,
    #[arg(long, default_value_t = 1024)]
    max_tokens: u32,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, default_value_t = 3)]
    max_resolution_attempts: usize,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    validate_only: bool,
}

fn default_capability_family() -> String {
    "legacy_smoke".into()
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Mistral,
    Google,
    Nvidia,
}

impl Provider {
    fn name(self) -> &'static str {
        match self {
            Self::Mistral => "mistral",
            Self::Google => "google",
            Self::Nvidia => "nvidia",
        }
    }
}

enum LiveAdapter {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
    Nvidia(NvidiaAdapter),
}

impl LiveAdapter {
    fn from_env(provider: Provider, model: &str) -> Result<Self, ModelError> {
        match provider {
            Provider::Mistral => MistralAdapter::from_env(model).map(Self::Mistral),
            Provider::Google => GoogleAdapter::from_env(model).map(Self::Google),
            Provider::Nvidia => NvidiaAdapter::from_env(model).map(Self::Nvidia),
        }
    }

    fn adapter(&self) -> &dyn ModelAdapter {
        match self {
            Self::Mistral(adapter) => adapter,
            Self::Google(adapter) => adapter,
            Self::Nvidia(adapter) => adapter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExpectedOutcome {
    Grounded,
    Unknown,
    GroundedAfterResolution,
}

impl ExpectedOutcome {
    fn expects_grounded(self) -> bool {
        matches!(self, Self::Grounded | Self::GroundedAfterResolution)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductDogfoodFixture {
    id: String,
    #[serde(default = "default_capability_family")]
    capability_family: String,
    workload_class: String,
    task: String,
    input: HarnessInput,
    expected_outcome: ExpectedOutcome,
    #[serde(default)]
    resolver_facts: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
struct CallObservation {
    model: String,
    usage: ModelUsage,
    latency_ms: u128,
    attempts: u32,
}

#[derive(Debug, Clone, Serialize)]
struct TargetOutcomeObservation {
    expected_targets: usize,
    exposed_grounded_targets: Vec<Proposition>,
    exposed_grounded_non_targets: Vec<Proposition>,
    supported_grounded_non_targets: Vec<Proposition>,
    grounded_target_coverage: f64,
    all_targets_grounded: bool,
}

#[derive(Debug, Serialize)]
struct RawArmResult {
    exposed_text: String,
    factual_claims: usize,
    grounded_claims: usize,
    unsupported_grounded_claims: usize,
    abstained: bool,
    exposed_factual_claims: Vec<FinalAnswerClaim>,
    target: TargetOutcomeObservation,
    call: CallObservation,
}

#[derive(Debug, Serialize)]
struct HarnessArmResult {
    safety_runtime: AnswerSafetyIdentity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    exposed_text: Option<String>,
    initial_verdict: Verdict,
    final_verdict: Verdict,
    finalization_status: FinalizationStatus,
    factual_claims: usize,
    factual_claim_coverage: f64,
    unsupported_exposed_grounded_claims: usize,
    abstained: bool,
    exposed_factual_claims: Vec<FinalAnswerClaim>,
    target: TargetOutcomeObservation,
    resolution_attempts: usize,
    resolution_succeeded: bool,
    canonical_recovery_used: bool,
    target_scoped_partial_used: bool,
    calls: Vec<CallObservation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    safety_observations: Vec<AnswerSafetyObservation>,
    failure_provenance: Vec<TargetFailureProvenance>,
    total_usage: ModelUsage,
    total_latency_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExpectedGroundedMissClass {
    CandidateTargetMissing,
    TargetUnverified,
    QualificationBlocked,
    ResolutionNotRequested,
    ResolutionNotClosed,
    AcceptRenderClaimOmission,
    AcceptRenderPropositionDrift,
    AcceptRenderTargetNotGrounded,
    FinalizationBlockedByOtherClaim,
    ArtifactBlockedByNonTargetClaims,
    AcceptanceBlocked,
    SufficiencyBlocked,
    Other,
}

#[derive(Debug, Clone, Serialize)]
struct TargetArtifactObservation {
    matching_claims: usize,
    states: Vec<EpistemicState>,
    supported_verification_receipts: usize,
    contradicted_verification_receipts: usize,
    qualification_findings: usize,
    authorized: bool,
}

#[derive(Debug, Clone, Serialize)]
struct TargetResolutionObservation {
    requested: bool,
    attempts: usize,
    statuses: Vec<ResolutionAttemptStatus>,
    admitted_evidence: usize,
    verification_receipts: usize,
}

#[derive(Debug, Clone, Serialize)]
struct TargetRendererObservation {
    exact_target_emitted: bool,
    exact_grounded_target_emitted: bool,
    factual_claims_empty: bool,
    other_factual_claims: usize,
}

struct MissClassificationInput<'a> {
    expected_grounded: bool,
    exposed_exact_grounded_target: bool,
    candidate_exact_target_present: bool,
    initial: &'a TargetArtifactObservation,
    final_artifact: &'a TargetArtifactObservation,
    resolution: &'a TargetResolutionObservation,
    final_verdict: Verdict,
    renderer: &'a TargetRendererObservation,
    finalization_status: FinalizationStatus,
    safety_forced_verification: bool,
    non_target_unresolved_claims: usize,
    non_target_contradicted_claims: usize,
    expected_outcome: ExpectedOutcome,
}

#[derive(Debug, Clone, Serialize)]
struct TargetFailureProvenance {
    target: Proposition,
    expected_grounded: bool,
    candidate_exact_target_present: bool,
    initial_artifact: TargetArtifactObservation,
    pre_render_artifact: TargetArtifactObservation,
    final_artifact: TargetArtifactObservation,
    resolution: TargetResolutionObservation,
    final_verdict: Verdict,
    renderer: TargetRendererObservation,
    finalization_status: FinalizationStatus,
    safety_forced_verification: bool,
    exposed_exact_grounded_target: bool,
    canonical_recovery_eligible: bool,
    non_target_unresolved_claims: usize,
    non_target_contradicted_claims: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    miss_class: Option<ExpectedGroundedMissClass>,
}

#[derive(Debug, Clone, Serialize)]
struct SharedHarnessObservation {
    candidate_call: CallObservation,
    initial_render: FinalAnswerCandidate,
    initial_render_call: CallObservation,
}

#[derive(Debug, Clone)]
struct PreparedHarnessState {
    initial_verdict: Verdict,
    initial_artifact: reasoning_harness_core::ReasoningArtifact,
    final_artifact: reasoning_harness_core::ReasoningArtifact,
    final_verdict: Verdict,
    resolution_attempts: Vec<ResolutionAttempt>,
    candidate: ReasoningCandidate,
    candidate_call: CallObservation,
}

#[derive(Debug, Serialize)]
struct CaseResult {
    id: String,
    capability_family: String,
    workload_class: String,
    expected_outcome: ExpectedOutcome,
    raw: RawArmResult,
    shared_harness: SharedHarnessObservation,
    harness: HarnessArmResult,
    harness_d3_sufficiency: HarnessArmResult,
}

#[derive(Debug, Default, Clone, Serialize)]
struct TargetAggregate {
    expected_unknown_cases: usize,
    correct_target_abstentions: usize,
    missed_target_insufficiency: usize,
    correct_target_abstention_rate: f64,
    grounded_targets_on_unknown: usize,
    safe_partial_unknown_cases: usize,
    supported_non_target_grounded_claims_on_unknown: usize,
    expected_grounded_cases: usize,
    fully_grounded_target_cases: usize,
    false_target_abstentions: usize,
    false_target_abstention_rate: f64,
    mean_grounded_target_coverage: f64,
}

#[derive(Debug, Default, Clone, Serialize)]
struct ArmAggregate {
    cases: usize,
    grounded_claims: usize,
    unsupported_grounded_claims: usize,
    unsupported_assertion_rate: f64,
    expected_unknown_cases: usize,
    correct_abstentions: usize,
    missed_insufficiency: usize,
    expected_grounded_cases: usize,
    false_abstentions: usize,
    correct_abstention_rate: f64,
    false_abstention_rate: f64,
    target: TargetAggregate,
}

#[derive(Debug, Default, Clone, Serialize)]
struct FailureProvenanceAggregate {
    expected_grounded_targets: usize,
    missed_grounded_targets: usize,
    classified_misses: usize,
    canonical_recovery_eligible: usize,
    miss_classes: BTreeMap<String, usize>,
}

#[derive(Debug, Default, Clone, Serialize)]
struct HarnessAggregate {
    arm: ArmAggregate,
    mean_final_claim_coverage: f64,
    resolution_attempted_cases: usize,
    resolution_success_cases: usize,
    resolution_success_rate: f64,
    canonical_recovery_cases: usize,
    canonical_recovery_rate: f64,
    target_scoped_partial_cases: usize,
    target_scoped_partial_rate: f64,
    failure_provenance: FailureProvenanceAggregate,
}

#[derive(Debug, Clone, Serialize)]
struct OverheadAggregate {
    raw_total_tokens: u64,
    harness_total_tokens: u64,
    d3_sufficiency_total_tokens: u64,
    token_ratio: Option<f64>,
    d3_sufficiency_token_ratio: Option<f64>,
    d3_sufficiency_incremental_token_ratio: Option<f64>,
    raw_latency_ms: u128,
    harness_latency_ms: u128,
    d3_sufficiency_latency_ms: u128,
    latency_ratio: Option<f64>,
    d3_sufficiency_latency_ratio: Option<f64>,
    d3_sufficiency_incremental_latency_ratio: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ProductDogfoodReport {
    schema_version: &'static str,
    comparison_contract: &'static str,
    provider: &'static str,
    model: String,
    workload_classes: Vec<String>,
    capability_families: Vec<String>,
    capability_case_counts: BTreeMap<String, usize>,
    cases: usize,
    raw: ArmAggregate,
    harness: HarnessAggregate,
    harness_d3_sufficiency: HarnessAggregate,
    overhead: OverheadAggregate,
    user_comprehension: &'static str,
    results: Vec<CaseResult>,
}

#[derive(Debug)]
struct LocalFactResolver {
    facts: BTreeMap<String, String>,
}

impl ResolutionResolver for LocalFactResolver {
    fn name(&self) -> &'static str {
        "dogfood_local_fact_store"
    }

    fn class(&self) -> ResolverClass {
        ResolverClass::EvidenceAcquisition
    }

    fn resolve(
        &self,
        request: &ResolutionRequest,
        attempt_index: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
        let proposition = match &request.target {
            ResolutionTarget::Proposition { proposition } => Some(proposition),
            ResolutionTarget::EvidenceQualification { requirement } => {
                Some(&requirement.proposition)
            }
            _ => None,
        };
        let Some(proposition) = proposition else {
            return Ok(no_result());
        };
        let Some(value) = self.facts.get(&proposition.key) else {
            return Ok(no_result());
        };
        Ok(ResolutionResolverOutput {
            contribution: ResolutionResolverContribution::AcquiredEvidence {
                evidence: vec![AcquiredEvidence {
                    id: format!("dogfood-resolver-{attempt_index}-{}", proposition.key),
                    source: "dogfood-local-fact-store".into(),
                    observation: format!("{}={value}", proposition.key),
                    facts: BTreeMap::from([(proposition.key.clone(), value.clone())]),
                }],
            },
            cost: ResolutionCost::default(),
        })
    }
}

fn no_result() -> ResolutionResolverOutput {
    ResolutionResolverOutput {
        contribution: ResolutionResolverContribution::NoResult,
        cost: ResolutionCost::default(),
    }
}

#[derive(Debug, Clone, Copy)]
struct DogfoodAdmission;

impl EvidenceAdmissionPolicy for DogfoodAdmission {
    fn admit(
        &self,
        resolver_name: &str,
        _request: &ResolutionRequest,
        acquired: &AcquiredEvidence,
    ) -> Result<Evidence, EvidenceAdmissionRejection> {
        if resolver_name != "dogfood_local_fact_store"
            || acquired.source != "dogfood-local-fact-store"
        {
            return Err(EvidenceAdmissionRejection::UntrustedSource);
        }
        Ok(Evidence {
            id: acquired.id.clone(),
            source: acquired.source.clone(),
            observation: acquired.observation.clone(),
            facts: acquired.facts.clone(),
            metadata: EvidenceMetadata {
                temporal: None,
                scope: None,
                provenance_class: Some("dogfood_explicit_resolver".into()),
            },
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    if args.max_tokens == 0 || args.max_resolution_attempts == 0 {
        return Err("max token/resolution limits must be greater than zero".into());
    }
    let fixtures = load_fixtures(&args.fixtures)?;
    if fixtures.is_empty() {
        return Err("product dogfood fixture directory is empty".into());
    }
    if let Some(fixture) = fixtures
        .iter()
        .find(|fixture| fixture.input.hypotheses.is_empty())
    {
        return Err(format!(
            "product dogfood v4 target metrics require at least one harness-owned hypothesis: {}",
            fixture.id
        ));
    }
    if args.validate_only {
        let capability_case_counts =
            fixtures
                .iter()
                .fold(BTreeMap::new(), |mut counts, fixture| {
                    *counts
                        .entry(fixture.capability_family.clone())
                        .or_insert(0usize) += 1;
                    counts
                });
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "fixtures": args.fixtures,
                "cases": fixtures.len(),
                "capability_families": capability_case_counts.keys().collect::<Vec<_>>(),
                "capability_case_counts": capability_case_counts,
            }))
            .map_err(|e| e.to_string())?
        );
        return Ok(());
    }
    let adapter = LiveAdapter::from_env(args.provider, &args.model).map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for (index, fixture) in fixtures.iter().enumerate() {
        eprintln!(
            "[product-dogfood] {}/{} class={} id={}",
            index + 1,
            fixtures.len(),
            fixture.workload_class,
            fixture.id
        );
        let seed = args.seed.and_then(|seed| seed.checked_add(index as u64));
        results.push(
            evaluate_case(
                fixture,
                adapter.adapter(),
                &args.model,
                args.max_tokens,
                seed,
                args.max_resolution_attempts,
            )
            .await?,
        );
    }
    let report = aggregate(args.provider, &args.model, results);
    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    if let Some(path) = args.output {
        fs::write(&path, format!("{json}\n")).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    println!("{json}");
    Ok(())
}

async fn evaluate_case(
    fixture: &ProductDogfoodFixture,
    adapter: &dyn ModelAdapter,
    model: &str,
    max_tokens: u32,
    seed: Option<u64>,
    max_resolution_attempts: usize,
) -> Result<CaseResult, String> {
    let (raw_answer, raw_call) = raw_answer(adapter, model, fixture, max_tokens, seed).await?;
    let raw_grounded = raw_answer
        .factual_claims
        .iter()
        .filter(|claim| claim.mode == FinalClaimMode::Grounded)
        .collect::<Vec<_>>();
    let raw_unsupported = raw_grounded
        .iter()
        .filter(|claim| !proposition_supported(&fixture.input, &claim.proposition))
        .count();
    let raw_target = target_outcome(
        &fixture.input.hypotheses,
        &raw_answer.factual_claims,
        |proposition| proposition_supported(&fixture.input, proposition),
    );
    let raw = RawArmResult {
        exposed_text: raw_answer.text.clone(),
        factual_claims: raw_answer.factual_claims.len(),
        grounded_claims: raw_grounded.len(),
        unsupported_grounded_claims: raw_unsupported,
        abstained: raw_grounded.is_empty(),
        exposed_factual_claims: raw_answer.factual_claims,
        target: raw_target,
        call: raw_call,
    };

    // B and C share the exact same generated candidate, deterministic pre-render state, and first
    // final-answer render. Only successor-induced verification/resolution may trigger a C-only rerender.
    let (candidate, candidate_call) =
        generate_candidate(adapter, model, fixture, max_tokens, seed).await?;
    let prepared =
        prepare_harness_state(fixture, candidate, candidate_call, max_resolution_attempts)?;
    let (shared_initial_render, shared_initial_render_call) = render_answer(
        adapter,
        model,
        &fixture.task,
        &prepared.final_artifact,
        prepared.final_verdict,
        max_tokens,
        seed,
    )
    .await?;
    let shared_harness = SharedHarnessObservation {
        candidate_call: prepared.candidate_call.clone(),
        initial_render: shared_initial_render.clone(),
        initial_render_call: shared_initial_render_call.clone(),
    };
    let harness = evaluate_harness_arm(HarnessArmCall {
        fixture,
        adapter,
        model,
        max_tokens,
        seed,
        max_resolution_attempts,
        prepared: prepared.clone(),
        initial_render: shared_initial_render.clone(),
        initial_render_call: shared_initial_render_call.clone(),
        safety_profile: AnswerSafetyProfile::Baseline,
    })
    .await?;
    let harness_d3_sufficiency = evaluate_harness_arm(HarnessArmCall {
        fixture,
        adapter,
        model,
        max_tokens,
        seed,
        max_resolution_attempts,
        prepared,
        initial_render: shared_initial_render,
        initial_render_call: shared_initial_render_call,
        safety_profile: AnswerSafetyProfile::D3SufficiencyV2,
    })
    .await?;

    Ok(CaseResult {
        id: fixture.id.clone(),
        capability_family: fixture.capability_family.clone(),
        workload_class: fixture.workload_class.clone(),
        expected_outcome: fixture.expected_outcome,
        raw,
        shared_harness,
        harness,
        harness_d3_sufficiency,
    })
}

fn prepare_harness_state(
    fixture: &ProductDogfoodFixture,
    candidate: ReasoningCandidate,
    candidate_call: CallObservation,
    max_resolution_attempts: usize,
) -> Result<PreparedHarnessState, String> {
    let pipeline = StandardGroundingPipeline;
    let initial = pipeline
        .run(fixture.input.clone(), candidate.clone(), &[])
        .map_err(|e| e.to_string())?;
    let initial_artifact = initial.artifact.clone();
    let mut final_artifact = initial.artifact.clone();
    let mut final_verdict = initial.verdict;
    let mut resolution_attempts = Vec::new();
    if !fixture.resolver_facts.is_empty() && final_verdict != Verdict::Accept {
        let resolution = run_dogfood_resolution(
            fixture,
            final_artifact.clone(),
            candidate.clone(),
            max_resolution_attempts,
        )?;
        resolution_attempts.extend(resolution.attempts.iter().cloned());
        final_artifact = resolution.final_artifact;
        final_verdict = resolution.final_verdict;
    }
    Ok(PreparedHarnessState {
        initial_verdict: initial.verdict,
        initial_artifact,
        final_artifact,
        final_verdict,
        resolution_attempts,
        candidate,
        candidate_call,
    })
}

struct HarnessArmCall<'a> {
    fixture: &'a ProductDogfoodFixture,
    adapter: &'a dyn ModelAdapter,
    model: &'a str,
    max_tokens: u32,
    seed: Option<u64>,
    max_resolution_attempts: usize,
    prepared: PreparedHarnessState,
    initial_render: FinalAnswerCandidate,
    initial_render_call: CallObservation,
    safety_profile: AnswerSafetyProfile,
}

async fn evaluate_harness_arm(call: HarnessArmCall<'_>) -> Result<HarnessArmResult, String> {
    let HarnessArmCall {
        fixture,
        adapter,
        model,
        max_tokens,
        seed,
        max_resolution_attempts,
        prepared,
        initial_render,
        initial_render_call,
        safety_profile,
    } = call;
    let PreparedHarnessState {
        initial_verdict,
        initial_artifact,
        mut final_artifact,
        mut final_verdict,
        mut resolution_attempts,
        candidate,
        candidate_call,
    } = prepared;
    let pre_render_artifact = final_artifact.clone();
    let mut calls = vec![candidate_call, initial_render_call];
    let mut safety_observations = Vec::new();
    let mut safety_usage = ModelUsage::default();
    let mut safety_latency_ms = 0u128;
    let mut finalization = FinalizationResult {
        status: FinalizationStatus::Unresolved,
        text: None,
        factual_claims: 0,
        covered_claims: 0,
        factual_claim_coverage: 1.0,
        uncovered_propositions: vec![],
    };
    let mut rendered = initial_render;
    let mut canonical_recovery_used = false;
    let mut target_scoped_partial_used = false;
    for render_round in 0..2usize {
        if render_round > 0 {
            let (answer, render_call) = render_answer(
                adapter,
                model,
                &fixture.task,
                &final_artifact,
                final_verdict,
                max_tokens,
                seed,
            )
            .await?;
            calls.push(render_call);
            rendered = answer;
        }
        finalization = finalize_answer(
            &final_artifact,
            final_verdict,
            rendered.clone(),
            FinalizationPolicy::default(),
        );
        if matches!(
            finalization.status,
            FinalizationStatus::Unresolved | FinalizationStatus::RequiresVerification
        ) {
            if let Some(recovered) = canonical_verified_target_answer(
                &final_artifact,
                final_verdict,
                &fixture.input.hypotheses,
            ) {
                rendered = recovered;
                canonical_recovery_used = true;
                finalization = finalize_answer(
                    &final_artifact,
                    final_verdict,
                    rendered.clone(),
                    FinalizationPolicy::default(),
                );
            }
        }

        if matches!(
            finalization.status,
            FinalizationStatus::Unresolved | FinalizationStatus::RequiresVerification
        ) {
            if let Some((recovered, recovered_finalization)) =
                canonical_verified_target_partial_answer(
                    &final_artifact,
                    final_verdict,
                    &fixture.input.hypotheses,
                )
            {
                rendered = recovered;
                target_scoped_partial_used = true;
                finalization = recovered_finalization;
            }
        }

        if safety_profile != AnswerSafetyProfile::Baseline
            && matches!(
                finalization.status,
                FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
            )
        {
            let mut targets = Vec::new();
            for claim in &rendered.factual_claims {
                if claim.mode == FinalClaimMode::Grounded && !targets.contains(&claim.proposition) {
                    targets.push(claim.proposition.clone());
                }
            }
            let mut blocked = Vec::new();
            for (index, target) in targets.iter().enumerate() {
                let target_seed = seed.and_then(|seed| seed.checked_add(index as u64));
                let observation = run_answer_safety_gate(
                    safety_profile,
                    adapter,
                    model,
                    target,
                    &final_artifact,
                    max_tokens.min(128),
                    target_seed,
                )
                .await
                .map_err(|error| format!("answer safety operational failure: {error}"))?;
                if let Some(sufficiency) = &observation.sufficiency {
                    safety_usage = add_usage(&safety_usage, &sufficiency.usage);
                }
                safety_latency_ms = safety_latency_ms.saturating_add(observation.latency_ms);
                if observation.disposition == AnswerSafetyDisposition::ForceVerification {
                    blocked.push(target.clone());
                }
                safety_observations.push(observation);
            }
            if !blocked.is_empty() {
                for target in blocked {
                    if !finalization.uncovered_propositions.contains(&target) {
                        finalization.uncovered_propositions.push(target);
                    }
                }
                finalization.status = FinalizationStatus::RequiresVerification;
                finalization.text = None;
            }
        }

        if finalization.status != FinalizationStatus::RequiresVerification
            || fixture.resolver_facts.is_empty()
            || render_round == 1
        {
            break;
        }

        let mut retry_input = input_from_artifact(&final_artifact);
        for proposition in &finalization.uncovered_propositions {
            if !retry_input.hypotheses.contains(proposition) {
                retry_input.hypotheses.push(proposition.clone());
            }
        }
        let before = final_artifact.clone();
        let resolution = run_dogfood_resolution_from_input(
            fixture,
            retry_input,
            candidate.clone(),
            max_resolution_attempts,
        )?;
        resolution_attempts.extend(resolution.attempts.iter().cloned());
        final_artifact = resolution.final_artifact;
        final_verdict = resolution.final_verdict;
        if final_artifact == before {
            break;
        }
    }

    let exposed = matches!(
        finalization.status,
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
    );
    let unsupported_exposed_grounded_claims = if exposed {
        rendered
            .factual_claims
            .iter()
            .filter(|claim| claim.mode == FinalClaimMode::Grounded)
            .filter(|claim| !artifact_supports(&final_artifact, &claim.proposition))
            .count()
    } else {
        0
    };
    let exposed_factual_claims = if exposed {
        rendered.factual_claims.clone()
    } else {
        vec![]
    };
    let target = target_outcome(
        &fixture.input.hypotheses,
        &exposed_factual_claims,
        |proposition| artifact_supports(&final_artifact, proposition),
    );
    let mut total_usage = calls.iter().fold(ModelUsage::default(), |acc, call| {
        add_usage(&acc, &call.usage)
    });
    total_usage = add_usage(&total_usage, &safety_usage);
    let total_latency_ms = calls
        .iter()
        .map(|call| call.latency_ms)
        .sum::<u128>()
        .saturating_add(safety_latency_ms);
    let resolution_succeeded =
        initial_verdict != Verdict::Accept && final_verdict == Verdict::Accept;
    let failure_provenance = build_failure_provenance(
        fixture,
        &candidate,
        &initial_artifact,
        &pre_render_artifact,
        &final_artifact,
        &resolution_attempts,
        final_verdict,
        &rendered,
        finalization.status,
        &safety_observations,
        &target,
    );

    Ok(HarnessArmResult {
        safety_runtime: safety_profile.identity(),
        exposed_text: finalization.text.clone(),
        initial_verdict,
        final_verdict,
        finalization_status: finalization.status,
        factual_claims: finalization.factual_claims,
        factual_claim_coverage: finalization.factual_claim_coverage,
        unsupported_exposed_grounded_claims,
        abstained: !exposed,
        exposed_factual_claims,
        target,
        resolution_attempts: resolution_attempts.len(),
        resolution_succeeded,
        canonical_recovery_used,
        target_scoped_partial_used,
        calls,
        safety_observations,
        failure_provenance,
        total_usage,
        total_latency_ms,
    })
}

fn input_from_artifact(artifact: &reasoning_harness_core::ReasoningArtifact) -> HarnessInput {
    HarnessInput {
        task: artifact.task.clone(),
        evidence: artifact.evidence.clone(),
        hypotheses: artifact.hypotheses.clone(),
        assumptions: artifact.assumptions.clone(),
        evidence_requirements: artifact.evidence_requirements.clone(),
        authority_policy: artifact.authority_policy.clone(),
    }
}

fn run_dogfood_resolution(
    fixture: &ProductDogfoodFixture,
    artifact: reasoning_harness_core::ReasoningArtifact,
    candidate: ReasoningCandidate,
    max_resolution_attempts: usize,
) -> Result<GroundedResolutionOutcome, String> {
    run_dogfood_resolution_from_input(
        fixture,
        input_from_artifact(&artifact),
        candidate,
        max_resolution_attempts,
    )
}

fn run_dogfood_resolution_from_input(
    fixture: &ProductDogfoodFixture,
    input: HarnessInput,
    candidate: ReasoningCandidate,
    max_resolution_attempts: usize,
) -> Result<GroundedResolutionOutcome, String> {
    let pipeline = StandardGroundingPipeline;
    let resolver = LocalFactResolver {
        facts: fixture.resolver_facts.clone(),
    };
    let planner = DefaultResolutionPlanner;
    let admission = DogfoodAdmission;
    let renderer = CanonicalFinalAnswerRenderer;
    let resolver_refs: [&dyn ResolutionResolver; 1] = [&resolver];
    let trusted: [&dyn reasoning_harness_core::TrustedResolutionVerifier; 0] = [];
    let runtime = GroundedResolutionRuntime {
        pipeline: &pipeline,
        planner: &planner,
        evidence_admission: &admission,
        resolvers: &resolver_refs,
        trusted_verifiers: &trusted,
        renderer: &renderer,
    };
    let mut policy = GroundedResolutionPolicy::default();
    policy.budget.max_attempts = max_resolution_attempts;
    runtime
        .run(input, candidate, &policy)
        .map_err(|e| e.to_string())
}

async fn raw_answer(
    adapter: &dyn ModelAdapter,
    model: &str,
    fixture: &ProductDogfoodFixture,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(FinalAnswerCandidate, CallObservation), String> {
    let evidence =
        serde_json::to_string_pretty(&fixture.input.evidence).map_err(|e| e.to_string())?;
    let hypotheses =
        serde_json::to_string_pretty(&fixture.input.hypotheses).map_err(|e| e.to_string())?;
    let task = format!(
        "Task:\n{}\n\nContext evidence:\n{}\n\nRequested hypotheses:\n{}",
        fixture.task, evidence, hypotheses
    );
    let system = "You are a general AI assistant. Answer the task directly from the supplied context. Return a structured final answer. List factual propositions you state in factual_claims and mark them grounded when you believe the context supports them; otherwise uncertain.";
    let request = ModelRequest {
        system: Some(system.into()),
        task: task.clone(),
        output_format: ModelOutputFormat::JsonSchema {
            name: "raw_final_answer".into(),
            schema: final_answer_candidate_schema(),
        },
        max_tokens: Some(max_tokens),
        random_seed: seed,
        reasoning_preference: None,
    };
    let schema = serde_json::to_string_pretty(&final_answer_candidate_schema())
        .map_err(|e| e.to_string())?;
    let fallback = ModelRequest {
        system: Some(format!(
            "{system} Return exactly one JSON object and no prose, conforming to the supplied JSON Schema."
        )),
        task: format!("JSON Schema:\n{schema}\n\n{task}"),
        output_format: ModelOutputFormat::JsonObject,
        max_tokens: Some(max_tokens),
        random_seed: seed,
        reasoning_preference: None,
    };
    generate_json(adapter, model, request, Some(fallback)).await
}

async fn generate_candidate(
    adapter: &dyn ModelAdapter,
    model: &str,
    fixture: &ProductDogfoodFixture,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(ReasoningCandidate, CallObservation), String> {
    let request = build_candidate_request(&fixture.input, Some(max_tokens), seed)
        .map_err(|e| e.to_string())?;
    let fallback = build_candidate_json_fallback_request(&fixture.input, Some(max_tokens), seed)
        .map_err(|e| e.to_string())?;
    generate_json(adapter, model, request, Some(fallback)).await
}

async fn render_answer(
    adapter: &dyn ModelAdapter,
    model: &str,
    task: &str,
    artifact: &reasoning_harness_core::ReasoningArtifact,
    verdict: Verdict,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(FinalAnswerCandidate, CallObservation), String> {
    let request = build_final_answer_request(task, artifact, verdict, Some(max_tokens), seed)
        .map_err(|e| e.to_string())?;
    let fallback =
        build_final_answer_json_fallback_request(task, artifact, verdict, Some(max_tokens), seed)
            .map_err(|e| e.to_string())?;
    generate_json(adapter, model, request, Some(fallback)).await
}

async fn generate_json<T: DeserializeOwned>(
    adapter: &dyn ModelAdapter,
    model: &str,
    request: ModelRequest,
    fallback: Option<ModelRequest>,
) -> Result<(T, CallObservation), String> {
    let started = Instant::now();
    let first = adapter.generate(request).await.map_err(|e| e.to_string())?;
    match parse_one::<T>(&first.text) {
        Ok(value) => Ok((value, call_observation(first, started, 1))),
        Err(first_error) => {
            let Some(fallback) = fallback else {
                return Err(format!("{model}: invalid structured output: {first_error}"));
            };
            let second = adapter
                .generate(fallback)
                .await
                .map_err(|e| e.to_string())?;
            let value = parse_one::<T>(&second.text).map_err(|second_error| {
                format!(
                    "{model}: invalid structured output after fallback: first={first_error}; second={second_error}"
                )
            })?;
            let usage = add_usage(&first.usage, &second.usage);
            Ok((
                value,
                CallObservation {
                    model: second.model,
                    usage,
                    latency_ms: started.elapsed().as_millis(),
                    attempts: 2,
                },
            ))
        }
    }
}

fn parse_one<T: DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    if let Ok(value) = serde_json::from_str(text) {
        return Ok(value);
    }
    let mut stream = serde_json::Deserializer::from_str(text).into_iter::<T>();
    let Some(Ok(value)) = stream.next() else {
        return serde_json::from_str(text);
    };
    Ok(value)
}

fn call_observation(response: ModelResponse, started: Instant, attempts: u32) -> CallObservation {
    CallObservation {
        model: response.model,
        usage: response.usage,
        latency_ms: started.elapsed().as_millis(),
        attempts,
    }
}

fn proposition_supported(input: &HarnessInput, proposition: &Proposition) -> bool {
    let claim = Claim {
        id: "product-dogfood-support-probe".into(),
        statement: format!("{}={}", proposition.key, proposition.value),
        state: EpistemicState::Unknown,
        proposition: Some(proposition.clone()),
        evidence_ids: vec![],
    };
    structured_fact_verifier_for_input(input)
        .verify(&claim, &input.evidence)
        .is_some_and(|receipt| receipt.conclusion == VerificationConclusion::Supported)
}

fn artifact_supports(
    artifact: &reasoning_harness_core::ReasoningArtifact,
    proposition: &Proposition,
) -> bool {
    artifact.claims.iter().any(|claim| {
        claim.proposition.as_ref() == Some(proposition)
            && matches!(
                claim.state,
                reasoning_harness_core::EpistemicState::Known
                    | reasoning_harness_core::EpistemicState::Supported
            )
    })
}

fn target_artifact_observation(
    artifact: &reasoning_harness_core::ReasoningArtifact,
    target: &Proposition,
) -> TargetArtifactObservation {
    let matching = artifact
        .claims
        .iter()
        .filter(|claim| claim.proposition.as_ref() == Some(target))
        .collect::<Vec<_>>();
    let mut states = Vec::new();
    for claim in &matching {
        if !states.contains(&claim.state) {
            states.push(claim.state);
        }
    }
    let supported_verification_receipts = artifact
        .verification_receipts
        .iter()
        .filter(|receipt| {
            receipt.proposition.as_ref() == Some(target)
                && receipt.conclusion == VerificationConclusion::Supported
        })
        .count();
    let contradicted_verification_receipts = artifact
        .verification_receipts
        .iter()
        .filter(|receipt| {
            receipt.proposition.as_ref() == Some(target)
                && receipt.conclusion == VerificationConclusion::Contradicted
        })
        .count();
    let qualification_findings = artifact
        .evidence_qualification_findings
        .iter()
        .filter(|finding| &finding.proposition == target)
        .count();
    TargetArtifactObservation {
        matching_claims: matching.len(),
        states,
        supported_verification_receipts,
        contradicted_verification_receipts,
        qualification_findings,
        authorized: artifact_supports(artifact, target),
    }
}

fn resolution_attempt_targets(attempt: &ResolutionAttempt, target: &Proposition) -> bool {
    match &attempt.request.target {
        ResolutionTarget::Proposition { proposition } => proposition == target,
        ResolutionTarget::EvidenceQualification { requirement } => {
            &requirement.proposition == target
        }
        ResolutionTarget::CausalRelation { .. }
        | ResolutionTarget::ClaimRevision { .. }
        | ResolutionTarget::HumanReview { .. } => false,
    }
}

fn target_resolution_observation(
    attempts: &[ResolutionAttempt],
    target: &Proposition,
) -> TargetResolutionObservation {
    let matching = attempts
        .iter()
        .filter(|attempt| resolution_attempt_targets(attempt, target))
        .collect::<Vec<_>>();
    TargetResolutionObservation {
        requested: !matching.is_empty(),
        attempts: matching.len(),
        statuses: matching.iter().map(|attempt| attempt.status).collect(),
        admitted_evidence: matching
            .iter()
            .map(|attempt| attempt.admitted_evidence_ids.len())
            .sum(),
        verification_receipts: matching
            .iter()
            .map(|attempt| attempt.verification_receipts)
            .sum(),
    }
}

fn target_renderer_observation(
    rendered: &FinalAnswerCandidate,
    target: &Proposition,
) -> TargetRendererObservation {
    let exact = rendered
        .factual_claims
        .iter()
        .filter(|claim| &claim.proposition == target)
        .collect::<Vec<_>>();
    TargetRendererObservation {
        exact_target_emitted: !exact.is_empty(),
        exact_grounded_target_emitted: exact
            .iter()
            .any(|claim| claim.mode == FinalClaimMode::Grounded),
        factual_claims_empty: rendered.factual_claims.is_empty(),
        other_factual_claims: rendered
            .factual_claims
            .iter()
            .filter(|claim| &claim.proposition != target)
            .count(),
    }
}

fn classify_expected_grounded_miss(
    input: MissClassificationInput<'_>,
) -> Option<ExpectedGroundedMissClass> {
    if !input.expected_grounded || input.exposed_exact_grounded_target {
        return None;
    }
    if input.safety_forced_verification {
        return Some(ExpectedGroundedMissClass::SufficiencyBlocked);
    }
    if input.final_verdict == Verdict::Accept && input.final_artifact.authorized {
        if input.renderer.factual_claims_empty {
            return Some(ExpectedGroundedMissClass::AcceptRenderClaimOmission);
        }
        if !input.renderer.exact_target_emitted {
            return Some(ExpectedGroundedMissClass::AcceptRenderPropositionDrift);
        }
        if !input.renderer.exact_grounded_target_emitted {
            return Some(ExpectedGroundedMissClass::AcceptRenderTargetNotGrounded);
        }
        if input.finalization_status == FinalizationStatus::RequiresVerification {
            return Some(ExpectedGroundedMissClass::FinalizationBlockedByOtherClaim);
        }
    }
    if input.final_artifact.authorized
        && ((input.final_verdict == Verdict::Unknown && input.non_target_unresolved_claims > 0)
            || (input.final_verdict == Verdict::Reject && input.non_target_contradicted_claims > 0))
    {
        return Some(ExpectedGroundedMissClass::ArtifactBlockedByNonTargetClaims);
    }
    if input.final_artifact.qualification_findings > 0 && !input.final_artifact.authorized {
        return Some(ExpectedGroundedMissClass::QualificationBlocked);
    }
    if input.expected_outcome == ExpectedOutcome::GroundedAfterResolution {
        if !input.resolution.requested {
            return Some(ExpectedGroundedMissClass::ResolutionNotRequested);
        }
        if !input.final_artifact.authorized {
            return Some(ExpectedGroundedMissClass::ResolutionNotClosed);
        }
    }
    if !input.candidate_exact_target_present && input.initial.matching_claims == 0 {
        return Some(ExpectedGroundedMissClass::CandidateTargetMissing);
    }
    if !input.final_artifact.authorized {
        return Some(ExpectedGroundedMissClass::TargetUnverified);
    }
    if input.final_verdict != Verdict::Accept {
        return Some(ExpectedGroundedMissClass::AcceptanceBlocked);
    }
    Some(ExpectedGroundedMissClass::Other)
}

#[allow(clippy::too_many_arguments)]
fn build_failure_provenance(
    fixture: &ProductDogfoodFixture,
    candidate: &ReasoningCandidate,
    initial_artifact: &reasoning_harness_core::ReasoningArtifact,
    pre_render_artifact: &reasoning_harness_core::ReasoningArtifact,
    final_artifact: &reasoning_harness_core::ReasoningArtifact,
    resolution_attempts: &[ResolutionAttempt],
    final_verdict: Verdict,
    rendered: &FinalAnswerCandidate,
    finalization_status: FinalizationStatus,
    safety_observations: &[AnswerSafetyObservation],
    target_outcome: &TargetOutcomeObservation,
) -> Vec<TargetFailureProvenance> {
    fixture
        .input
        .hypotheses
        .iter()
        .map(|target| {
            let candidate_exact_target_present = candidate
                .claims
                .iter()
                .any(|claim| claim.proposition.as_ref() == Some(target));
            let initial = target_artifact_observation(initial_artifact, target);
            let pre_render = target_artifact_observation(pre_render_artifact, target);
            let final_observation = target_artifact_observation(final_artifact, target);
            let resolution = target_resolution_observation(resolution_attempts, target);
            let renderer = target_renderer_observation(rendered, target);
            let safety_forced_verification = safety_observations.iter().any(|observation| {
                observation.target == *target
                    && observation.disposition == AnswerSafetyDisposition::ForceVerification
            });
            let exposed_exact_grounded_target = target_outcome
                .exposed_grounded_targets
                .iter()
                .any(|proposition| proposition == target);
            let expected_grounded = fixture.expected_outcome.expects_grounded();
            let canonical_recovery_eligible = expected_grounded
                && !exposed_exact_grounded_target
                && final_verdict == Verdict::Accept
                && final_observation.authorized;
            let non_target_unresolved_claims = final_artifact
                .claims
                .iter()
                .filter(|claim| claim.proposition.as_ref() != Some(target))
                .filter(|claim| {
                    matches!(
                        claim.state,
                        EpistemicState::Assumed
                            | EpistemicState::Unknown
                            | EpistemicState::Inferred
                    )
                })
                .count();
            let non_target_contradicted_claims = final_artifact
                .claims
                .iter()
                .filter(|claim| claim.proposition.as_ref() != Some(target))
                .filter(|claim| claim.state == EpistemicState::Contradicted)
                .count();
            let miss_class = classify_expected_grounded_miss(MissClassificationInput {
                expected_grounded,
                exposed_exact_grounded_target,
                candidate_exact_target_present,
                initial: &initial,
                final_artifact: &final_observation,
                resolution: &resolution,
                final_verdict,
                renderer: &renderer,
                finalization_status,
                safety_forced_verification,
                non_target_unresolved_claims,
                non_target_contradicted_claims,
                expected_outcome: fixture.expected_outcome,
            });
            TargetFailureProvenance {
                target: target.clone(),
                expected_grounded,
                candidate_exact_target_present,
                initial_artifact: initial,
                pre_render_artifact: pre_render,
                final_artifact: final_observation,
                resolution,
                final_verdict,
                renderer,
                finalization_status,
                safety_forced_verification,
                exposed_exact_grounded_target,
                canonical_recovery_eligible,
                non_target_unresolved_claims,
                non_target_contradicted_claims,
                miss_class,
            }
        })
        .collect()
}

impl ExpectedGroundedMissClass {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CandidateTargetMissing => "candidate_target_missing",
            Self::TargetUnverified => "target_unverified",
            Self::QualificationBlocked => "qualification_blocked",
            Self::ResolutionNotRequested => "resolution_not_requested",
            Self::ResolutionNotClosed => "resolution_not_closed",
            Self::AcceptRenderClaimOmission => "accept_render_claim_omission",
            Self::AcceptRenderPropositionDrift => "accept_render_proposition_drift",
            Self::AcceptRenderTargetNotGrounded => "accept_render_target_not_grounded",
            Self::FinalizationBlockedByOtherClaim => "finalization_blocked_by_other_claim",
            Self::ArtifactBlockedByNonTargetClaims => "artifact_blocked_by_non_target_claims",
            Self::AcceptanceBlocked => "acceptance_blocked",
            Self::SufficiencyBlocked => "sufficiency_blocked",
            Self::Other => "other",
        }
    }
}

fn add_usage(left: &ModelUsage, right: &ModelUsage) -> ModelUsage {
    ModelUsage {
        input_tokens: add_opt(left.input_tokens, right.input_tokens),
        output_tokens: add_opt(left.output_tokens, right.output_tokens),
        total_tokens: add_opt(left.total_tokens, right.total_tokens),
    }
}

fn add_opt(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => left.checked_add(right),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn load_fixtures(directory: &PathBuf) -> Result<Vec<ProductDogfoodFixture>, String> {
    if !directory.is_dir() {
        return Err(format!(
            "{}: fixture directory does not exist",
            directory.display()
        ));
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
        })
        .collect()
}

fn aggregate(provider: Provider, model: &str, results: Vec<CaseResult>) -> ProductDogfoodReport {
    let classes = results
        .iter()
        .map(|result| result.workload_class.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let capability_case_counts = results.iter().fold(BTreeMap::new(), |mut counts, result| {
        *counts.entry(result.capability_family.clone()).or_insert(0) += 1;
        counts
    });
    let capability_families = capability_case_counts.keys().cloned().collect::<Vec<_>>();
    let raw = aggregate_raw(&results);
    let harness = aggregate_harness(&results, |result| &result.harness);
    let harness_d3_sufficiency =
        aggregate_harness(&results, |result| &result.harness_d3_sufficiency);
    let raw_total_tokens = results
        .iter()
        .filter_map(|result| result.raw.call.usage.total_tokens)
        .sum();
    let harness_total_tokens = results
        .iter()
        .filter_map(|result| result.harness.total_usage.total_tokens)
        .sum();
    let d3_sufficiency_total_tokens = results
        .iter()
        .filter_map(|result| result.harness_d3_sufficiency.total_usage.total_tokens)
        .sum();
    let raw_latency_ms = results
        .iter()
        .map(|result| result.raw.call.latency_ms)
        .sum();
    let harness_latency_ms = results
        .iter()
        .map(|result| result.harness.total_latency_ms)
        .sum();
    let d3_sufficiency_latency_ms = results
        .iter()
        .map(|result| result.harness_d3_sufficiency.total_latency_ms)
        .sum();
    ProductDogfoodReport {
        schema_version: "reason-product-dogfood-v8",
        comparison_contract: PRODUCT_DOGFOOD_COMPARISON_CONTRACT_ID,
        provider: provider.name(),
        model: model.into(),
        workload_classes: classes,
        capability_families,
        capability_case_counts,
        cases: results.len(),
        raw,
        harness,
        harness_d3_sufficiency,
        overhead: OverheadAggregate {
            raw_total_tokens,
            harness_total_tokens,
            d3_sufficiency_total_tokens,
            token_ratio: ratio(harness_total_tokens as f64, raw_total_tokens as f64),
            d3_sufficiency_token_ratio: ratio(
                d3_sufficiency_total_tokens as f64,
                raw_total_tokens as f64,
            ),
            d3_sufficiency_incremental_token_ratio: ratio(
                d3_sufficiency_total_tokens as f64,
                harness_total_tokens as f64,
            ),
            raw_latency_ms,
            harness_latency_ms,
            d3_sufficiency_latency_ms,
            latency_ratio: ratio(harness_latency_ms as f64, raw_latency_ms as f64),
            d3_sufficiency_latency_ratio: ratio(
                d3_sufficiency_latency_ms as f64,
                raw_latency_ms as f64,
            ),
            d3_sufficiency_incremental_latency_ratio: ratio(
                d3_sufficiency_latency_ms as f64,
                harness_latency_ms as f64,
            ),
        },
        user_comprehension: "not_automated_manual_review_required",
        results,
    }
}

fn aggregate_raw(results: &[CaseResult]) -> ArmAggregate {
    aggregate_arm(
        results,
        |result| result.raw.grounded_claims,
        |result| result.raw.unsupported_grounded_claims,
        |result| result.raw.abstained,
        |result| &result.raw.target,
    )
}

fn aggregate_harness(
    results: &[CaseResult],
    select: fn(&CaseResult) -> &HarnessArmResult,
) -> HarnessAggregate {
    let arm = aggregate_arm(
        results,
        |result| select(result).factual_claims,
        |result| select(result).unsupported_exposed_grounded_claims,
        |result| select(result).abstained,
        |result| &select(result).target,
    );
    let mean_final_claim_coverage = if results.is_empty() {
        0.0
    } else {
        results
            .iter()
            .map(|result| select(result).factual_claim_coverage)
            .sum::<f64>()
            / results.len() as f64
    };
    let resolution_attempted_cases = results
        .iter()
        .filter(|result| select(result).resolution_attempts > 0)
        .count();
    let resolution_success_cases = results
        .iter()
        .filter(|result| select(result).resolution_succeeded)
        .count();
    let canonical_recovery_cases = results
        .iter()
        .filter(|result| select(result).canonical_recovery_used)
        .count();
    let target_scoped_partial_cases = results
        .iter()
        .filter(|result| select(result).target_scoped_partial_used)
        .count();
    HarnessAggregate {
        arm,
        mean_final_claim_coverage,
        resolution_attempted_cases,
        resolution_success_cases,
        resolution_success_rate: rate(resolution_success_cases, resolution_attempted_cases),
        canonical_recovery_cases,
        canonical_recovery_rate: rate(canonical_recovery_cases, results.len()),
        target_scoped_partial_cases,
        target_scoped_partial_rate: rate(target_scoped_partial_cases, results.len()),
        failure_provenance: aggregate_failure_provenance(results, select),
    }
}

fn aggregate_failure_provenance(
    results: &[CaseResult],
    select: fn(&CaseResult) -> &HarnessArmResult,
) -> FailureProvenanceAggregate {
    let traces = results
        .iter()
        .flat_map(|result| select(result).failure_provenance.iter())
        .collect::<Vec<_>>();
    let expected_grounded_targets = traces
        .iter()
        .filter(|trace| trace.expected_grounded)
        .count();
    let missed = traces
        .iter()
        .filter(|trace| trace.expected_grounded && !trace.exposed_exact_grounded_target)
        .collect::<Vec<_>>();
    let missed_grounded_targets = missed.len();
    let classified_misses = missed
        .iter()
        .filter(|trace| trace.miss_class.is_some())
        .count();
    let canonical_recovery_eligible = missed
        .iter()
        .filter(|trace| trace.canonical_recovery_eligible)
        .count();
    let mut miss_classes = BTreeMap::new();
    for trace in missed {
        if let Some(class) = trace.miss_class {
            *miss_classes.entry(class.as_str().to_string()).or_insert(0) += 1;
        }
    }
    FailureProvenanceAggregate {
        expected_grounded_targets,
        missed_grounded_targets,
        classified_misses,
        canonical_recovery_eligible,
        miss_classes,
    }
}

fn aggregate_arm(
    results: &[CaseResult],
    grounded: impl Fn(&CaseResult) -> usize,
    unsupported: impl Fn(&CaseResult) -> usize,
    abstained: impl Fn(&CaseResult) -> bool,
    target: impl Fn(&CaseResult) -> &TargetOutcomeObservation,
) -> ArmAggregate {
    let grounded_claims = results.iter().map(&grounded).sum();
    let unsupported_grounded_claims = results.iter().map(&unsupported).sum();
    let expected_unknown_cases = results
        .iter()
        .filter(|result| result.expected_outcome == ExpectedOutcome::Unknown)
        .count();
    let correct_abstentions = results
        .iter()
        .filter(|result| result.expected_outcome == ExpectedOutcome::Unknown && abstained(result))
        .count();
    let missed_insufficiency = expected_unknown_cases.saturating_sub(correct_abstentions);
    let expected_grounded_cases = results
        .iter()
        .filter(|result| result.expected_outcome.expects_grounded())
        .count();
    let false_abstentions = results
        .iter()
        .filter(|result| result.expected_outcome.expects_grounded() && abstained(result))
        .count();
    ArmAggregate {
        cases: results.len(),
        grounded_claims,
        unsupported_grounded_claims,
        unsupported_assertion_rate: rate(unsupported_grounded_claims, grounded_claims),
        expected_unknown_cases,
        correct_abstentions,
        missed_insufficiency,
        expected_grounded_cases,
        false_abstentions,
        correct_abstention_rate: rate(correct_abstentions, expected_unknown_cases),
        false_abstention_rate: rate(false_abstentions, expected_grounded_cases),
        target: aggregate_target(results, target),
    }
}

fn aggregate_target(
    results: &[CaseResult],
    target: impl Fn(&CaseResult) -> &TargetOutcomeObservation,
) -> TargetAggregate {
    let expected_unknown_cases = results
        .iter()
        .filter(|result| result.expected_outcome == ExpectedOutcome::Unknown)
        .count();
    let correct_target_abstentions = results
        .iter()
        .filter(|result| {
            result.expected_outcome == ExpectedOutcome::Unknown
                && target(result).exposed_grounded_targets.is_empty()
        })
        .count();
    let missed_target_insufficiency =
        expected_unknown_cases.saturating_sub(correct_target_abstentions);
    let grounded_targets_on_unknown = results
        .iter()
        .filter(|result| result.expected_outcome == ExpectedOutcome::Unknown)
        .map(|result| target(result).exposed_grounded_targets.len())
        .sum();
    let safe_partial_unknown_cases = results
        .iter()
        .filter(|result| {
            result.expected_outcome == ExpectedOutcome::Unknown
                && target(result).exposed_grounded_targets.is_empty()
                && !target(result).supported_grounded_non_targets.is_empty()
        })
        .count();
    let supported_non_target_grounded_claims_on_unknown = results
        .iter()
        .filter(|result| result.expected_outcome == ExpectedOutcome::Unknown)
        .map(|result| target(result).supported_grounded_non_targets.len())
        .sum();
    let expected_grounded = results
        .iter()
        .filter(|result| result.expected_outcome.expects_grounded())
        .collect::<Vec<_>>();
    let expected_grounded_cases = expected_grounded.len();
    let fully_grounded_target_cases = expected_grounded
        .iter()
        .filter(|result| target(result).all_targets_grounded)
        .count();
    let false_target_abstentions =
        expected_grounded_cases.saturating_sub(fully_grounded_target_cases);
    let mean_grounded_target_coverage = if expected_grounded_cases == 0 {
        0.0
    } else {
        expected_grounded
            .iter()
            .map(|result| target(result).grounded_target_coverage)
            .sum::<f64>()
            / expected_grounded_cases as f64
    };
    TargetAggregate {
        expected_unknown_cases,
        correct_target_abstentions,
        missed_target_insufficiency,
        correct_target_abstention_rate: rate(correct_target_abstentions, expected_unknown_cases),
        grounded_targets_on_unknown,
        safe_partial_unknown_cases,
        supported_non_target_grounded_claims_on_unknown,
        expected_grounded_cases,
        fully_grounded_target_cases,
        false_target_abstentions,
        false_target_abstention_rate: rate(false_target_abstentions, expected_grounded_cases),
        mean_grounded_target_coverage,
    }
}

fn target_outcome(
    expected_targets: &[Proposition],
    claims: &[FinalAnswerClaim],
    supported: impl Fn(&Proposition) -> bool,
) -> TargetOutcomeObservation {
    let grounded = claims
        .iter()
        .filter(|claim| claim.mode == FinalClaimMode::Grounded)
        .collect::<Vec<_>>();
    let mut exposed_grounded_targets = Vec::new();
    let mut exposed_grounded_non_targets = Vec::new();
    for claim in grounded {
        let destination = if expected_targets.contains(&claim.proposition) {
            &mut exposed_grounded_targets
        } else {
            &mut exposed_grounded_non_targets
        };
        if !destination.contains(&claim.proposition) {
            destination.push(claim.proposition.clone());
        }
    }
    let supported_grounded_non_targets = exposed_grounded_non_targets
        .iter()
        .filter(|proposition| supported(proposition))
        .cloned()
        .collect::<Vec<_>>();
    let grounded_target_coverage = rate(exposed_grounded_targets.len(), expected_targets.len());
    let all_targets_grounded =
        !expected_targets.is_empty() && exposed_grounded_targets.len() == expected_targets.len();
    TargetOutcomeObservation {
        expected_targets: expected_targets.len(),
        exposed_grounded_targets,
        exposed_grounded_non_targets,
        supported_grounded_non_targets,
        grounded_target_coverage,
        all_targets_grounded,
    }
}

fn rate(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn ratio(numerator: f64, denominator: f64) -> Option<f64> {
    (denominator > 0.0).then_some(numerator / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_target() -> TargetOutcomeObservation {
        TargetOutcomeObservation {
            expected_targets: 1,
            exposed_grounded_targets: vec![],
            exposed_grounded_non_targets: vec![],
            supported_grounded_non_targets: vec![],
            grounded_target_coverage: 0.0,
            all_targets_grounded: false,
        }
    }

    #[test]
    fn direct_fact_support_requires_observed_value_agreement() {
        let input = HarnessInput {
            task: "test".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "fixture".into(),
                observation: "status=503".into(),
                facts: BTreeMap::from([("http.status".into(), "503".into())]),
                metadata: Default::default(),
            }],
            hypotheses: vec![],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: Default::default(),
        };
        assert!(proposition_supported(
            &input,
            &Proposition {
                key: "http.status".into(),
                value: "503".into()
            }
        ));
        assert!(!proposition_supported(
            &input,
            &Proposition {
                key: "http.status".into(),
                value: "200".into()
            }
        ));
    }

    #[test]
    fn target_metrics_preserve_safe_partial_facts_without_grounding_the_task_target() {
        let expected = Proposition {
            key: "incident.root_cause".into(),
            value: "database".into(),
        };
        let observed = Proposition {
            key: "http.status_code".into(),
            value: "503".into(),
        };
        let claims = vec![
            FinalAnswerClaim {
                proposition: observed.clone(),
                mode: FinalClaimMode::Grounded,
            },
            FinalAnswerClaim {
                proposition: expected.clone(),
                mode: FinalClaimMode::Uncertain,
            },
        ];
        let target = target_outcome(std::slice::from_ref(&expected), &claims, |proposition| {
            proposition == &observed
        });
        assert!(target.exposed_grounded_targets.is_empty());
        assert_eq!(target.supported_grounded_non_targets, vec![observed]);
        assert_eq!(target.grounded_target_coverage, 0.0);
        assert!(!target.all_targets_grounded);
    }

    #[test]
    fn duplicate_grounded_target_claims_do_not_inflate_target_coverage() {
        let expected = Proposition {
            key: "backup.enabled".into(),
            value: "true".into(),
        };
        let claim = FinalAnswerClaim {
            proposition: expected.clone(),
            mode: FinalClaimMode::Grounded,
        };
        let target = target_outcome(
            std::slice::from_ref(&expected),
            &[claim.clone(), claim],
            |_| true,
        );
        assert_eq!(target.exposed_grounded_targets, vec![expected]);
        assert_eq!(target.grounded_target_coverage, 1.0);
        assert!(target.all_targets_grounded);
    }

    #[test]
    fn support_probe_rejects_stale_qualified_evidence() {
        let proposition = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let input: HarnessInput = serde_json::from_value(serde_json::json!({
            "task": "test",
            "evidence": [{
                "id": "e1", "source": "fixture", "observation": "feature.enabled=true",
                "facts": {"feature.enabled": "true"},
                "metadata": {"temporal": {"effective_from_unix_seconds": 100, "effective_until_unix_seconds": 120}}
            }],
            "hypotheses": [proposition.clone()],
            "evidence_requirements": [{"proposition": proposition.clone(), "as_of_unix_seconds": 150}]
        })).unwrap();
        assert!(!proposition_supported(&input, &proposition));
    }

    #[test]
    fn support_probe_rejects_scope_mismatch() {
        let proposition = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let input: HarnessInput = serde_json::from_value(serde_json::json!({
            "task": "test",
            "evidence": [{
                "id": "e1", "source": "fixture", "observation": "feature.enabled=true",
                "facts": {"feature.enabled": "true"},
                "metadata": {"scope": {"region": {"kind": "values", "values": ["us-west-2"]}}}
            }],
            "hypotheses": [proposition.clone()],
            "evidence_requirements": [{
                "proposition": proposition.clone(),
                "scope": {"region": {"kind": "values", "values": ["us-east-1"]}}
            }]
        }))
        .unwrap();
        assert!(!proposition_supported(&input, &proposition));
    }

    #[test]
    fn support_probe_rejects_conflicting_qualified_values() {
        let proposition = Proposition {
            key: "feature.enabled".into(),
            value: "true".into(),
        };
        let input: HarnessInput = serde_json::from_value(serde_json::json!({
            "task": "test",
            "evidence": [
                {"id": "e1", "source": "a", "observation": "true", "facts": {"feature.enabled": "true"}},
                {"id": "e2", "source": "b", "observation": "false", "facts": {"feature.enabled": "false"}}
            ],
            "hypotheses": [proposition.clone()]
        })).unwrap();
        assert!(!proposition_supported(&input, &proposition));
    }

    fn grounded_fixture(
        target: &Proposition,
        expected_outcome: ExpectedOutcome,
    ) -> ProductDogfoodFixture {
        ProductDogfoodFixture {
            id: "trace".into(),
            capability_family: "trace".into(),
            workload_class: "trace".into(),
            task: "trace target".into(),
            input: HarnessInput {
                task: "trace target".into(),
                hypotheses: vec![target.clone()],
                ..Default::default()
            },
            expected_outcome,
            resolver_facts: BTreeMap::new(),
        }
    }

    fn artifact_with_target(
        target: &Proposition,
        state: EpistemicState,
    ) -> reasoning_harness_core::ReasoningArtifact {
        reasoning_harness_core::ReasoningArtifact {
            task: "trace target".into(),
            hypotheses: vec![target.clone()],
            claims: vec![Claim {
                id: "target".into(),
                statement: "target".into(),
                state,
                proposition: Some(target.clone()),
                evidence_ids: vec![],
            }],
            ..Default::default()
        }
    }

    #[test]
    fn failure_provenance_marks_accept_render_omission_as_canonical_recovery_eligible() {
        let target = Proposition {
            key: "service.region".into(),
            value: "us-east-1".into(),
        };
        let fixture = grounded_fixture(&target, ExpectedOutcome::Grounded);
        let candidate = ReasoningCandidate {
            claims: vec![reasoning_harness_core::CandidateClaim {
                id: "candidate-target".into(),
                statement: "region".into(),
                proposed_state: EpistemicState::Supported,
                proposition: Some(target.clone()),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        };
        let artifact = artifact_with_target(&target, EpistemicState::Supported);
        let rendered = FinalAnswerCandidate {
            text: "The region is us-east-1.".into(),
            factual_claims: vec![],
        };
        let trace = build_failure_provenance(
            &fixture,
            &candidate,
            &artifact,
            &artifact,
            &artifact,
            &[],
            Verdict::Accept,
            &rendered,
            FinalizationStatus::Unresolved,
            &[],
            &empty_target(),
        );
        assert_eq!(trace.len(), 1);
        assert!(trace[0].canonical_recovery_eligible);
        assert_eq!(
            trace[0].miss_class,
            Some(ExpectedGroundedMissClass::AcceptRenderClaimOmission)
        );
    }

    #[test]
    fn failure_provenance_records_exact_proposition_drift_without_fuzzy_matching() {
        let target = Proposition {
            key: "http.status_code".into(),
            value: "503".into(),
        };
        let fixture = grounded_fixture(&target, ExpectedOutcome::Grounded);
        let candidate = ReasoningCandidate::default();
        let artifact = artifact_with_target(&target, EpistemicState::Known);
        let rendered = FinalAnswerCandidate {
            text: "HTTP status is 503.".into(),
            factual_claims: vec![FinalAnswerClaim {
                proposition: Proposition {
                    key: "HTTP status".into(),
                    value: "503".into(),
                },
                mode: FinalClaimMode::Grounded,
            }],
        };
        let trace = build_failure_provenance(
            &fixture,
            &candidate,
            &artifact,
            &artifact,
            &artifact,
            &[],
            Verdict::Accept,
            &rendered,
            FinalizationStatus::RequiresVerification,
            &[],
            &empty_target(),
        );
        assert!(!trace[0].renderer.exact_target_emitted);
        assert_eq!(trace[0].renderer.other_factual_claims, 1);
        assert_eq!(
            trace[0].miss_class,
            Some(ExpectedGroundedMissClass::AcceptRenderPropositionDrift)
        );
    }

    #[test]
    fn failure_provenance_identifies_non_target_claims_blocking_an_authorized_target() {
        let target = Proposition {
            key: "service.region".into(),
            value: "us-east-1".into(),
        };
        let fixture = grounded_fixture(&target, ExpectedOutcome::Grounded);
        let candidate = ReasoningCandidate::default();
        let mut artifact = artifact_with_target(&target, EpistemicState::Supported);
        artifact.claims.push(Claim {
            id: "noise".into(),
            statement: "unresolved non-target".into(),
            state: EpistemicState::Assumed,
            proposition: Some(Proposition {
                key: "unrelated.detail".into(),
                value: "maybe".into(),
            }),
            evidence_ids: vec![],
        });
        let trace = build_failure_provenance(
            &fixture,
            &candidate,
            &artifact,
            &artifact,
            &artifact,
            &[],
            Verdict::Unknown,
            &FinalAnswerCandidate::default(),
            FinalizationStatus::Unresolved,
            &[],
            &empty_target(),
        );
        assert_eq!(trace[0].non_target_unresolved_claims, 1);
        assert_eq!(
            trace[0].miss_class,
            Some(ExpectedGroundedMissClass::ArtifactBlockedByNonTargetClaims)
        );
    }

    #[test]
    fn failure_provenance_distinguishes_resolution_not_requested() {
        let target = Proposition {
            key: "backup.enabled".into(),
            value: "true".into(),
        };
        let fixture = grounded_fixture(&target, ExpectedOutcome::GroundedAfterResolution);
        let candidate = ReasoningCandidate {
            claims: vec![reasoning_harness_core::CandidateClaim {
                id: "candidate-target".into(),
                statement: "backup".into(),
                proposed_state: EpistemicState::Assumed,
                proposition: Some(target.clone()),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        };
        let artifact = artifact_with_target(&target, EpistemicState::Assumed);
        let trace = build_failure_provenance(
            &fixture,
            &candidate,
            &artifact,
            &artifact,
            &artifact,
            &[],
            Verdict::Unknown,
            &FinalAnswerCandidate::default(),
            FinalizationStatus::Unresolved,
            &[],
            &empty_target(),
        );
        assert_eq!(
            trace[0].miss_class,
            Some(ExpectedGroundedMissClass::ResolutionNotRequested)
        );
        assert!(!trace[0].resolution.requested);
    }

    #[test]
    fn aggregate_counts_false_and_correct_abstention_separately() {
        let mk = |id: &str, expected_outcome, raw_abstained, harness_abstained| CaseResult {
            id: id.into(),
            capability_family: "test".into(),
            workload_class: "class".into(),
            expected_outcome,
            raw: RawArmResult {
                exposed_text: "raw".into(),
                factual_claims: 0,
                grounded_claims: 0,
                unsupported_grounded_claims: 0,
                abstained: raw_abstained,
                exposed_factual_claims: vec![],
                target: empty_target(),
                call: CallObservation {
                    model: "m".into(),
                    usage: Default::default(),
                    latency_ms: 1,
                    attempts: 1,
                },
            },
            shared_harness: SharedHarnessObservation {
                candidate_call: CallObservation {
                    model: "m".into(),
                    usage: Default::default(),
                    latency_ms: 1,
                    attempts: 1,
                },
                initial_render: FinalAnswerCandidate::default(),
                initial_render_call: CallObservation {
                    model: "m".into(),
                    usage: Default::default(),
                    latency_ms: 1,
                    attempts: 1,
                },
            },
            harness: HarnessArmResult {
                safety_runtime: AnswerSafetyProfile::Baseline.identity(),
                exposed_text: None,
                initial_verdict: Verdict::Unknown,
                final_verdict: Verdict::Unknown,
                finalization_status: FinalizationStatus::Unresolved,
                factual_claims: 0,
                factual_claim_coverage: 1.0,
                unsupported_exposed_grounded_claims: 0,
                abstained: harness_abstained,
                exposed_factual_claims: vec![],
                target: empty_target(),
                resolution_attempts: 0,
                resolution_succeeded: false,
                canonical_recovery_used: false,
                target_scoped_partial_used: false,
                calls: vec![],
                safety_observations: vec![],
                failure_provenance: vec![],
                total_usage: Default::default(),
                total_latency_ms: 1,
            },
            harness_d3_sufficiency: HarnessArmResult {
                safety_runtime: AnswerSafetyProfile::D3SufficiencyV2.identity(),
                exposed_text: None,
                initial_verdict: Verdict::Unknown,
                final_verdict: Verdict::Unknown,
                finalization_status: FinalizationStatus::Unresolved,
                factual_claims: 0,
                factual_claim_coverage: 1.0,
                unsupported_exposed_grounded_claims: 0,
                abstained: harness_abstained,
                exposed_factual_claims: vec![],
                target: empty_target(),
                resolution_attempts: 0,
                resolution_succeeded: false,
                canonical_recovery_used: false,
                target_scoped_partial_used: false,
                calls: vec![],
                safety_observations: vec![],
                failure_provenance: vec![],
                total_usage: Default::default(),
                total_latency_ms: 1,
            },
        };
        let mut results = vec![
            mk("u", ExpectedOutcome::Unknown, false, true),
            mk("g", ExpectedOutcome::Grounded, true, false),
        ];
        let safe_partial = Proposition {
            key: "http.status_code".into(),
            value: "503".into(),
        };
        results[0].raw.target.exposed_grounded_non_targets = vec![safe_partial.clone()];
        results[0].raw.target.supported_grounded_non_targets = vec![safe_partial];
        let raw = aggregate_raw(&results);
        let harness = aggregate_harness(&results, |result| &result.harness);
        assert_eq!(raw.missed_insufficiency, 1);
        assert_eq!(raw.target.missed_target_insufficiency, 0);
        assert_eq!(raw.target.safe_partial_unknown_cases, 1);
        assert_eq!(raw.false_abstentions, 1);
        assert_eq!(harness.arm.correct_abstentions, 1);
        assert_eq!(harness.arm.false_abstentions, 0);
        assert_eq!(harness.arm.target.false_target_abstentions, 1);
    }
}
