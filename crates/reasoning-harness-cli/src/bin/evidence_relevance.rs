use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    EvidenceLocalBlockingReason, EvidenceLocalQualificationV6, EvidenceRelevanceAssessment,
    EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate, EvidenceRelevanceDisposition,
    EvidenceRelevanceSignalKind, EvidenceRelevanceTargetPolicy, ModelAdapter, ModelError,
    ModelErrorKind, ModelExecutionBudget, ModelExecutionTelemetrySnapshot, ModelRequest,
    ModelResponse, ModelUsage, build_evidence_local_qualification_v8_request,
    build_evidence_relevance_binding_proposal_v5_request, build_strict_json_text_fallback_request,
    derive_effective_evidence_local_qualification_v1, materialize_evidence_relevance_v14,
    parse_evidence_local_qualification_v8, parse_evidence_relevance_binding_proposal,
};
use reasoning_harness_providers::{GoogleAdapter, GroqAdapter, MistralAdapter, NvidiaAdapter};
use serde::{Deserialize, Serialize};

const CONFIGURATION_ID: &str = "evidence-relevance-live-calibration-v19";
const EXPECTED_SUITE_ID: &str = "evidence-relevance-calibration-v19";
const EXPECTED_STATUS: &str = "fresh_unobserved_calibration";
const EXPECTED_ANNOTATION_PROTOCOL_ID: &str = "evidence-relevance-effective-qualification-v19";
const EXPECTED_FIXED_CORE_ID: &str = "evidence-relevance-fixed-core-v1";
const EXPECTED_RELATIVE_DIR: &str = "fixtures/evidence-relevance-calibration-v19";
const QUALIFICATION_STAGE_MAX_MODEL_CALLS: u32 = 2;
const GROQ_STRICT_JSON_TEXT_MAX_TOKENS: u32 = 512;
const EXPECTED_CASES: usize = 48;
const PROVIDER_WAIT_BUDGET_MS: u64 = 45_000;
const MAX_SINGLE_PROVIDER_WAIT_MS: u64 = 30_000;
const ABSOLUTE_CASE_BUDGET_MS: u64 = 120_000;

#[derive(Debug, Parser)]
#[command(
    name = "reason-evidence-relevance-study",
    about = "Fresh model-backed calibration for evidence-target semantic relevance"
)]
struct Args {
    target: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long)]
    model: String,
    #[arg(long = "fixture")]
    fixture_ids: Vec<String>,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, default_value_t = 1000)]
    inter_case_delay_ms: u64,
    #[arg(long)]
    checkpoint: Option<PathBuf>,
    #[arg(long, default_value_t = 2)]
    max_consecutive_operational_failures: usize,
    #[arg(long, default_value_t = 2)]
    max_consecutive_capacity_failures: usize,
    #[arg(long, default_value_t = false)]
    continue_after_operational_failures: bool,
    #[arg(long, default_value_t = false)]
    validate_only: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Mistral,
    Google,
    Nvidia,
    Groq,
}

impl Provider {
    fn name(self) -> &'static str {
        match self {
            Self::Mistral => "mistral",
            Self::Google => "google",
            Self::Nvidia => "nvidia",
            Self::Groq => "groq",
        }
    }
}

enum Generator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
    Nvidia(NvidiaAdapter),
    Groq(GroqAdapter),
}

impl Generator {
    fn from_provider(provider: Provider, model: &str) -> Result<Self, String> {
        match provider {
            Provider::Mistral => MistralAdapter::from_env(model)
                .map(Self::Mistral)
                .map_err(|error| error.to_string()),
            Provider::Google => GoogleAdapter::from_env(model)
                .map(Self::Google)
                .map_err(|error| error.to_string()),
            Provider::Nvidia => NvidiaAdapter::from_env(model)
                .map(Self::Nvidia)
                .map_err(|error| error.to_string()),
            Provider::Groq => GroqAdapter::from_env(model)
                .map(Self::Groq)
                .map_err(|error| error.to_string()),
        }
    }

    fn adapter(&self) -> &dyn ModelAdapter {
        match self {
            Self::Mistral(adapter) => adapter,
            Self::Google(adapter) => adapter,
            Self::Nvidia(adapter) => adapter,
            Self::Groq(adapter) => adapter,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CalibrationManifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    annotation_protocol_id: String,
    fixed_core_id: String,
    cases: Vec<CalibrationCase>,
}

#[derive(Debug, Deserialize)]
struct CalibrationCase {
    id: String,
    family: String,
    #[serde(rename = "task")]
    _task: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_proposal: EvidenceRelevanceBindingProposal,
    expected_local_qualification: EvidenceLocalQualificationV6,
    expected_disposition: EvidenceRelevanceDisposition,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
struct ExecutionSummary {
    provider_attempts_started: u64,
    provider_attempts_completed: u64,
    active_ms: u64,
    wait_ms: u64,
    pacing_wait_ms: u64,
    retry_wait_ms: u64,
}

impl From<ModelExecutionTelemetrySnapshot> for ExecutionSummary {
    fn from(value: ModelExecutionTelemetrySnapshot) -> Self {
        Self {
            provider_attempts_started: value.provider_attempts_started,
            provider_attempts_completed: value.provider_attempts_completed,
            active_ms: value.active_ms,
            wait_ms: value.wait_ms,
            pacing_wait_ms: value.pacing_wait_ms,
            retry_wait_ms: value.retry_wait_ms,
        }
    }
}

#[derive(Debug)]
struct CaseExecutionBudget {
    remaining_active_ms: u64,
    remaining_wait_ms: u64,
    max_single_wait_ms: u64,
    absolute_deadline: tokio::time::Instant,
}

impl CaseExecutionBudget {
    fn new(active_ms: u64) -> Self {
        Self {
            remaining_active_ms: active_ms,
            remaining_wait_ms: PROVIDER_WAIT_BUDGET_MS,
            max_single_wait_ms: MAX_SINGLE_PROVIDER_WAIT_MS,
            absolute_deadline: tokio::time::Instant::now()
                + Duration::from_millis(ABSOLUTE_CASE_BUDGET_MS),
        }
    }

    fn provider_budget(&self) -> ModelExecutionBudget {
        ModelExecutionBudget {
            max_active_ms: self.remaining_active_ms,
            max_wait_ms: self.remaining_wait_ms,
            max_single_wait_ms: self.max_single_wait_ms.min(self.remaining_wait_ms),
        }
    }

    fn consume(&mut self, execution: ExecutionSummary) {
        self.remaining_active_ms = self.remaining_active_ms.saturating_sub(execution.active_ms);
        self.remaining_wait_ms = self.remaining_wait_ms.saturating_sub(execution.wait_ms);
    }
}

#[derive(Debug)]
struct AdapterCallSuccess {
    response: ModelResponse,
}

#[derive(Debug)]
struct AdapterCallFailure {
    error: Option<ModelError>,
    execution: ExecutionSummary,
    absolute_timeout: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
struct UsageSummary {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
}

impl UsageSummary {
    fn add(&mut self, usage: &ModelUsage) {
        self.input_tokens += usage.input_tokens.unwrap_or(0);
        self.output_tokens += usage.output_tokens.unwrap_or(0);
        self.total_tokens += usage.total_tokens.unwrap_or(0);
    }

    fn add_summary(&mut self, other: &Self) {
        self.input_tokens = self.input_tokens.saturating_add(other.input_tokens);
        self.output_tokens = self.output_tokens.saturating_add(other.output_tokens);
        self.total_tokens = self.total_tokens.saturating_add(other.total_tokens);
    }
}

#[derive(Debug, Clone, Serialize)]
struct CaseObservation {
    id: String,
    family: String,
    expected_proposal: EvidenceRelevanceBindingProposal,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_proposal: Option<EvidenceRelevanceBindingProposal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proposal_match: Option<bool>,
    expected_local_qualification: EvidenceLocalQualificationV6,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_local_qualification: Option<EvidenceLocalQualificationV6>,
    #[serde(skip_serializing_if = "Option::is_none")]
    local_qualification_match: Option<bool>,
    local_qualification_invoked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    effective_local_qualification: Option<EvidenceLocalQualificationV6>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effective_local_qualification_match: Option<bool>,
    effective_local_qualification_derived: bool,
    expected_disposition: EvidenceRelevanceDisposition,
    #[serde(skip_serializing_if = "Option::is_none")]
    materialized_disposition: Option<EvidenceRelevanceDisposition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disposition_match: Option<bool>,
    lexical_baseline: EvidenceRelevanceDisposition,
    lexical_baseline_match: bool,
    deterministic_guard_override: bool,
    used_structured_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    provider_attempts_complete: bool,
    execution: ExecutionSummary,
    latency_ms: u128,
    usage: UsageSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assessment: Option<EvidenceRelevanceAssessment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CalibrationMetrics {
    cases: usize,
    successful_provider_cases: usize,
    failed_provider_cases: usize,
    proposal_exact_matches: usize,
    proposal_exact_accuracy: Option<f64>,
    local_qualification_expected_cases: usize,
    local_qualification_invocations: usize,
    local_qualification_exact_matches: usize,
    local_qualification_exact_accuracy: Option<f64>,
    qualification_scope_risk_misses: usize,
    qualification_spurious_scope_risks: usize,
    qualification_identity_scope_misses: usize,
    qualification_relation_scope_misses: usize,
    effective_local_qualification_expected_cases: usize,
    effective_local_qualification_derivations: usize,
    effective_local_qualification_exact_matches: usize,
    effective_local_qualification_exact_accuracy: Option<f64>,
    effective_qualification_scope_risk_misses: usize,
    effective_qualification_spurious_scope_risks: usize,
    effective_qualification_identity_scope_misses: usize,
    effective_qualification_relation_scope_misses: usize,
    materialized_exact_matches: usize,
    materialized_exact_accuracy: Option<f64>,
    correctness_wrong_target_relevance_retention: usize,
    false_relevance_rejections: usize,
    relevant_left_ambiguous: usize,
    utility_misses: usize,
    ambiguous_dispositions: usize,
    deterministic_guard_overrides: usize,
    lexical_exact_matches: usize,
    lexical_exact_accuracy: Option<f64>,
    lexical_wrong_target_relevance_retention: usize,
    lexical_expected_relevant_misses: usize,
    model_calls: u64,
    provider_attempts: u64,
    provider_attempts_started: u64,
    provider_attempts_completed: u64,
    provider_attempts_incomplete_observations: usize,
    active_execution_ms: u64,
    provider_wait_ms: u64,
    pacing_wait_ms: u64,
    retry_wait_ms: u64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    total_latency_ms: u128,
    mean_latency_ms: Option<f64>,
    latency_p50_ms: Option<u128>,
    latency_p95_ms: Option<u128>,
    latency_max_ms: Option<u128>,
    successful_latency_p50_ms: Option<u128>,
    successful_latency_p95_ms: Option<u128>,
    successful_latency_max_ms: Option<u128>,
    failure_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct StudyOutput {
    configuration_id: &'static str,
    suite_id: String,
    issue: u64,
    source_rule: String,
    annotation_protocol_id: String,
    fixed_core_id: String,
    corpus_status_at_observation: String,
    candidate_commit: String,
    provider: String,
    model: String,
    seed: Option<u64>,
    planned_cases: usize,
    completed_cases: usize,
    canonical_full_calibration: bool,
    scorability: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    operational_abort: Option<OperationalAbort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_arm_latch: Option<ProviderArmLatch>,
    metrics: CalibrationMetrics,
    observations: Vec<CaseObservation>,
}

#[derive(Debug, Clone, Serialize)]
struct OperationalAbort {
    reason: &'static str,
    after_case_id: String,
    next_case_id: String,
    consecutive_operational_failures: usize,
    max_consecutive_operational_failures: usize,
    remaining_cases: usize,
}

#[derive(Debug, Clone, Serialize)]
struct ProviderArmLatch {
    reason: &'static str,
    triggered_after_case_id: String,
    triggering_failure_class: String,
    threshold: usize,
    suppressed_cases: usize,
}

#[derive(Debug, Serialize)]
struct Checkpoint<'a> {
    checkpoint_version: &'static str,
    run_status: &'static str,
    configuration_id: &'static str,
    suite_id: &'a str,
    provider: &'a str,
    model: &'a str,
    expected_cases: usize,
    completed_cases: usize,
    scorability: &'static str,
    observations: &'a [CaseObservation],
}

#[derive(Debug)]
struct CallOutcome {
    proposal: EvidenceRelevanceBindingProposal,
    used_structured_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
}

#[derive(Debug)]
struct QualificationCallOutcome {
    qualification: EvidenceLocalQualificationV6,
    used_structured_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
}

#[derive(Debug)]
struct CallFailure {
    class: String,
    message: String,
    used_structured_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    provider_attempts_complete: bool,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(output) => match serde_json::to_string_pretty(&output) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("serialize evidence-relevance study: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("evidence-relevance study failed: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<StudyOutput, String> {
    let args = Args::parse();
    let manifest = load_manifest(&args.target)?;
    let selected = select_cases(&manifest, &args.fixture_ids)?;

    for case in &selected {
        let expected = case.expected_proposal;
        let assessment = materialize_evidence_relevance_v14(
            &case.policy,
            &case.candidate,
            Some(&expected),
            Some(&case.expected_local_qualification),
        )
        .map_err(|error| format!("invalid calibration policy {}: {error}", case.id))?;
        if assessment.disposition != case.expected_disposition {
            return Err(format!(
                "deterministic expected proposal/qualification mismatch for {}: expected {:?}, materialized {:?}",
                case.id, case.expected_disposition, assessment.disposition
            ));
        }
    }

    if args.validate_only {
        return Ok(StudyOutput {
            configuration_id: CONFIGURATION_ID,
            suite_id: manifest.suite_id.clone(),
            issue: manifest.issue,
            source_rule: manifest.source_rule.clone(),
            annotation_protocol_id: manifest.annotation_protocol_id.clone(),
            fixed_core_id: manifest.fixed_core_id.clone(),
            corpus_status_at_observation: manifest.status.clone(),
            candidate_commit: git_head().unwrap_or_else(|_| "unknown".into()),
            provider: args.provider.name().into(),
            model: args.model,
            seed: args.seed,
            planned_cases: selected.len(),
            completed_cases: 0,
            canonical_full_calibration: false,
            scorability: "validate_only_non_scorable",
            operational_abort: None,
            provider_arm_latch: None,
            metrics: empty_metrics(selected.len()),
            observations: Vec::new(),
        });
    }

    if args.max_consecutive_operational_failures == 0 {
        return Err("--max-consecutive-operational-failures must be at least 1".into());
    }
    if args.max_consecutive_capacity_failures == 0 {
        return Err("--max-consecutive-capacity-failures must be at least 1".into());
    }

    let generator = Generator::from_provider(args.provider, &args.model)?;
    let provider = args.provider.name().to_owned();
    let mut observations = Vec::with_capacity(selected.len());
    let mut consecutive_operational_failures = 0usize;
    let mut consecutive_capacity_failures = 0usize;
    let mut operational_abort: Option<OperationalAbort> = None;
    let mut provider_arm_latch: Option<ProviderArmLatch> = None;

    if let Some(path) = args.checkpoint.as_deref() {
        write_checkpoint(
            path,
            &manifest.suite_id,
            &provider,
            &args.model,
            selected.len(),
            &observations,
            "in_progress",
        )?;
    }

    for (index, case) in selected.iter().enumerate() {
        if provider_arm_latch.is_some() {
            let lexical_baseline = simple_lexical_baseline(&case.policy, &case.candidate);
            let latch_reason = provider_arm_latch
                .as_ref()
                .map(|latch| latch.reason)
                .unwrap_or("provider_arm_latched");
            observations.push(failure_observation(
                case,
                lexical_baseline,
                ObservationFailure {
                    latency_ms: 0,
                    used_structured_fallback: false,
                    model_calls: 0,
                    provider_attempts: 0,
                    provider_attempts_complete: true,
                            usage: UsageSummary::default(),
                    provider_model: Some(args.model.clone()),
                    finish_reason: None,
                    class: "provider_arm_latched".into(),
                    message: format!(
                        "provider arm suppressed after prior operational latch; reason={latch_reason}"
                    ),
                },
            ));
            if let Some(latch) = provider_arm_latch.as_mut() {
                latch.suppressed_cases = latch.suppressed_cases.saturating_add(1);
            }
            if let Some(path) = args.checkpoint.as_deref() {
                write_checkpoint(
                    path,
                    &manifest.suite_id,
                    &provider,
                    &args.model,
                    selected.len(),
                    &observations,
                    "in_progress",
                )?;
            }
            continue;
        }

        let case_seed = args.seed.and_then(|base| base.checked_add(index as u64));
        let request = build_evidence_relevance_binding_proposal_v5_request(
            &case.policy,
            &case.candidate,
            case_seed,
        )
        .map_err(|error| format!("build relevance request {}: {error}", case.id))?;
        let lexical_baseline = simple_lexical_baseline(&case.policy, &case.candidate);

        let started = Instant::now();
        let adapter = generator.adapter();
        let execution_before = adapter.execution_telemetry_snapshot();
        let mut case_budget =
            CaseExecutionBudget::new(case.policy.assessment_budget.max_elapsed_ms);
        let result = call_model_for_proposal(
            adapter,
            args.provider,
            request,
            case.policy.assessment_budget.max_model_attempts,
            &mut case_budget,
        )
        .await;
        let mut observation = match result {
            Ok(call) => {
                complete_observed_case(
                    generator.adapter(),
                    args.provider,
                    case,
                    case_seed,
                    lexical_baseline,
                    started,
                    call,
                    &mut case_budget,
                )
                .await?
            }
            Err(failure) => failure_observation(
                case,
                lexical_baseline,
                ObservationFailure {
                    latency_ms: started.elapsed().as_millis(),
                    used_structured_fallback: failure.used_structured_fallback,
                    model_calls: failure.model_calls,
                    provider_attempts: failure.provider_attempts,
                    provider_attempts_complete: failure.provider_attempts_complete,
                    usage: failure.usage,
                    provider_model: failure.provider_model,
                    finish_reason: failure.finish_reason,
                    class: failure.class,
                    message: failure.message,
                },
            ),
        };

        let execution_after = adapter.execution_telemetry_snapshot();
        observation.execution = match (execution_before, execution_after) {
            (Some(before), Some(after)) => ExecutionSummary::from(after.saturating_delta(before)),
            _ => ExecutionSummary {
                provider_attempts_started: u64::from(observation.provider_attempts),
                provider_attempts_completed: if observation.provider_attempts_complete {
                    u64::from(observation.provider_attempts)
                } else {
                    0
                },
                active_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                ..ExecutionSummary::default()
            },
        };

        eprintln!(
            "[evidence-relevance-study] case={} status={} proposal={:?} materialized={:?} lexical={:?} fallback={}",
            case.id,
            if observation.failure.is_none() {
                "ok"
            } else {
                "failed"
            },
            observation.observed_proposal,
            observation.materialized_disposition,
            observation.lexical_baseline,
            observation.used_structured_fallback,
        );
        observations.push(observation);

        if let Some(path) = args.checkpoint.as_deref() {
            write_checkpoint(
                path,
                &manifest.suite_id,
                &provider,
                &args.model,
                selected.len(),
                &observations,
                "in_progress",
            )?;
        }

        let latest_failure_class = observations
            .last()
            .and_then(|observation| observation.failure_class.as_deref());
        consecutive_operational_failures =
            next_operational_failure_streak(consecutive_operational_failures, latest_failure_class);
        consecutive_capacity_failures =
            next_capacity_failure_streak(consecutive_capacity_failures, latest_failure_class);

        if provider_arm_latch.is_none() {
            if latest_failure_class == Some("quota") {
                provider_arm_latch = Some(ProviderArmLatch {
                    reason: "confirmed_quota_failure",
                    triggered_after_case_id: case.id.clone(),
                    triggering_failure_class: "quota".into(),
                    threshold: 1,
                    suppressed_cases: 0,
                });
                eprintln!(
                    "[evidence-relevance-study] provider arm latched after typed quota failure; remaining external calls will be suppressed"
                );
            } else if consecutive_capacity_failures >= args.max_consecutive_capacity_failures
                && index + 1 < selected.len()
            {
                provider_arm_latch = Some(ProviderArmLatch {
                    reason: "correlated_capacity_failure_budget_exhausted",
                    triggered_after_case_id: case.id.clone(),
                    triggering_failure_class: latest_failure_class.unwrap_or("unknown").to_owned(),
                    threshold: args.max_consecutive_capacity_failures,
                    suppressed_cases: 0,
                });
                eprintln!(
                    "[evidence-relevance-study] provider arm latched after {consecutive_capacity_failures} consecutive capacity failures; remaining external calls will be suppressed"
                );
            }
        }

        if should_abort_after_operational_failure(
            args.continue_after_operational_failures,
            consecutive_operational_failures,
            args.max_consecutive_operational_failures,
            index + 1 < selected.len(),
        ) {
            operational_abort = Some(OperationalAbort {
                reason: "consecutive_operational_failure_budget_exhausted",
                after_case_id: case.id.clone(),
                next_case_id: selected[index + 1].id.clone(),
                consecutive_operational_failures,
                max_consecutive_operational_failures: args.max_consecutive_operational_failures,
                remaining_cases: selected.len().saturating_sub(index + 1),
            });
            eprintln!(
                "[evidence-relevance-study] aborting after {} consecutive operational provider failures; next_case={}",
                consecutive_operational_failures,
                selected[index + 1].id
            );
            break;
        }

        if index + 1 < selected.len() && args.inter_case_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(args.inter_case_delay_ms)).await;
        }
    }

    if let Some(path) = args.checkpoint.as_deref() {
        write_checkpoint(
            path,
            &manifest.suite_id,
            &provider,
            &args.model,
            selected.len(),
            &observations,
            if operational_abort.is_some() {
                "aborted_operational_failure_budget"
            } else {
                "completed"
            },
        )?;
    }

    let canonical_full_calibration =
        args.fixture_ids.is_empty() && selected.len() == manifest.cases.len();
    let operationally_complete = operational_abort.is_none()
        && observations.len() == selected.len()
        && observations.iter().all(|case| case.failure.is_none());
    let scorability = if canonical_full_calibration && operationally_complete {
        "complete_calibration_observation"
    } else {
        "non_canonical_or_operationally_incomplete"
    };

    Ok(StudyOutput {
        configuration_id: CONFIGURATION_ID,
        suite_id: manifest.suite_id.clone(),
        issue: manifest.issue,
        source_rule: manifest.source_rule.clone(),
        annotation_protocol_id: manifest.annotation_protocol_id.clone(),
        fixed_core_id: manifest.fixed_core_id.clone(),
        corpus_status_at_observation: manifest.status.clone(),
        candidate_commit: git_head().unwrap_or_else(|_| "unknown".into()),
        provider,
        model: args.model,
        seed: args.seed,
        planned_cases: selected.len(),
        completed_cases: observations.len(),
        canonical_full_calibration,
        scorability,
        operational_abort,
        provider_arm_latch,
        metrics: summarize_metrics(&observations),
        observations,
    })
}

#[allow(clippy::too_many_arguments)]
async fn complete_observed_case(
    adapter: &dyn ModelAdapter,
    provider: Provider,
    case: &CalibrationCase,
    case_seed: Option<u64>,
    lexical_baseline: EvidenceRelevanceDisposition,
    started: Instant,
    call: CallOutcome,
    budget: &mut CaseExecutionBudget,
) -> Result<CaseObservation, String> {
    let observed = call.proposal;
    let mut used_structured_fallback = call.used_structured_fallback;
    let mut model_calls = call.model_calls;
    let mut provider_attempts = call.provider_attempts;
    let mut usage = call.usage;
    let mut provider_model = call.provider_model;
    let mut finish_reason = call.finish_reason;

    let request = build_evidence_local_qualification_v8_request(
        &case.policy,
        &case.candidate,
        case_seed.map(|seed| seed ^ 0x6a11_f1ed),
    )
    .map_err(|error| format!("build local qualification request {}: {error}", case.id))?;

    let qualification_call =
        match call_model_for_local_qualification(adapter, provider, request, budget).await {
            Ok(call) => call,
            Err(mut failure) => {
                failure.used_structured_fallback |= used_structured_fallback;
                failure.model_calls = failure.model_calls.saturating_add(model_calls);
                failure.provider_attempts =
                    failure.provider_attempts.saturating_add(provider_attempts);
                failure.usage.add_summary(&usage);
                if failure.provider_model.is_none() {
                    failure.provider_model = provider_model;
                }
                if failure.finish_reason.is_none() {
                    failure.finish_reason = finish_reason;
                }
                return Ok(failure_observation(
                    case,
                    lexical_baseline,
                    ObservationFailure {
                        latency_ms: started.elapsed().as_millis(),
                        used_structured_fallback: failure.used_structured_fallback,
                        model_calls: failure.model_calls,
                        provider_attempts: failure.provider_attempts,
                        provider_attempts_complete: failure.provider_attempts_complete,
                        usage: failure.usage,
                        provider_model: failure.provider_model,
                        finish_reason: failure.finish_reason,
                        class: failure.class,
                        message: format!("local qualification failed: {}", failure.message),
                    },
                ));
            }
        };

    let observed_qualification = qualification_call.qualification;
    used_structured_fallback |= qualification_call.used_structured_fallback;
    model_calls = model_calls.saturating_add(qualification_call.model_calls);
    provider_attempts = provider_attempts.saturating_add(qualification_call.provider_attempts);
    usage.add_summary(&qualification_call.usage);
    if qualification_call.provider_model.is_some() {
        provider_model = qualification_call.provider_model;
    }
    if qualification_call.finish_reason.is_some() {
        finish_reason = qualification_call.finish_reason;
    }

    let effective_local_qualification = match derive_effective_evidence_local_qualification_v1(
        &case.policy,
        &case.candidate,
        Some(&observed),
        Some(&observed_qualification),
    ) {
        Ok(effective) => effective,
        Err(error) => {
            return Ok(failure_observation(
                case,
                lexical_baseline,
                ObservationFailure {
                    latency_ms: started.elapsed().as_millis(),
                    used_structured_fallback,
                    model_calls,
                    provider_attempts,
                    provider_attempts_complete: true,
                    usage,
                    provider_model,
                    finish_reason,
                    class: "effective_qualification".into(),
                    message: error.to_string(),
                },
            ));
        }
    };

    match materialize_evidence_relevance_v14(
        &case.policy,
        &case.candidate,
        Some(&observed),
        Some(&observed_qualification),
    ) {
        Ok(assessment) => Ok(success_observation(
            case,
            lexical_baseline,
            observed,
            observed_qualification,
            effective_local_qualification,
            assessment,
            started.elapsed().as_millis(),
            used_structured_fallback,
            model_calls,
            provider_attempts,
            usage,
            provider_model,
            finish_reason,
        )),
        Err(error) => Ok(failure_observation(
            case,
            lexical_baseline,
            ObservationFailure {
                latency_ms: started.elapsed().as_millis(),
                used_structured_fallback,
                model_calls,
                provider_attempts,
                provider_attempts_complete: true,
                usage,
                provider_model,
                finish_reason,
                class: "materialization".into(),
                message: error.to_string(),
            },
        )),
    }
}

struct ObservationFailure {
    latency_ms: u128,
    used_structured_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    provider_attempts_complete: bool,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
    class: String,
    message: String,
}

#[allow(clippy::too_many_arguments)]
fn success_observation(
    case: &CalibrationCase,
    lexical_baseline: EvidenceRelevanceDisposition,
    observed: EvidenceRelevanceBindingProposal,
    observed_local_qualification: EvidenceLocalQualificationV6,
    effective_local_qualification: EvidenceLocalQualificationV6,
    assessment: EvidenceRelevanceAssessment,
    latency_ms: u128,
    used_structured_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
) -> CaseObservation {
    let materialized = assessment.disposition;
    CaseObservation {
        id: case.id.clone(),
        family: case.family.clone(),
        expected_proposal: case.expected_proposal,
        observed_proposal: Some(observed),
        proposal_match: Some(observed == case.expected_proposal),
        expected_local_qualification: case.expected_local_qualification,
        observed_local_qualification: Some(observed_local_qualification),
        local_qualification_match: Some(
            observed_local_qualification == case.expected_local_qualification,
        ),
        local_qualification_invoked: true,
        effective_local_qualification: Some(effective_local_qualification),
        effective_local_qualification_match: Some(
            effective_local_qualification == case.expected_local_qualification,
        ),
        effective_local_qualification_derived: true,
        expected_disposition: case.expected_disposition,
        materialized_disposition: Some(materialized),
        disposition_match: Some(materialized == case.expected_disposition),
        lexical_baseline,
        lexical_baseline_match: lexical_baseline == case.expected_disposition,
        deterministic_guard_override: materialized == EvidenceRelevanceDisposition::Ambiguous
            && (observed.target_binding != case.expected_proposal.target_binding
                || observed.relation_binding != case.expected_proposal.relation_binding
                || effective_local_qualification != case.expected_local_qualification),
        used_structured_fallback,
        model_calls,
        provider_attempts,
        provider_attempts_complete: true,
        execution: ExecutionSummary::default(),
        latency_ms,
        usage,
        provider_model,
        finish_reason,
        assessment: Some(assessment),
        failure_class: None,
        failure: None,
    }
}

fn failure_observation(
    case: &CalibrationCase,
    lexical_baseline: EvidenceRelevanceDisposition,
    failure: ObservationFailure,
) -> CaseObservation {
    let local_qualification_invoked = failure.message.starts_with("local qualification failed:");
    CaseObservation {
        id: case.id.clone(),
        family: case.family.clone(),
        expected_proposal: case.expected_proposal,
        observed_proposal: None,
        proposal_match: None,
        expected_local_qualification: case.expected_local_qualification,
        observed_local_qualification: None,
        local_qualification_match: None,
        local_qualification_invoked,
        effective_local_qualification: None,
        effective_local_qualification_match: None,
        effective_local_qualification_derived: false,
        expected_disposition: case.expected_disposition,
        materialized_disposition: None,
        disposition_match: None,
        lexical_baseline,
        lexical_baseline_match: lexical_baseline == case.expected_disposition,
        deterministic_guard_override: false,
        used_structured_fallback: failure.used_structured_fallback,
        model_calls: failure.model_calls,
        provider_attempts: failure.provider_attempts,
        provider_attempts_complete: failure.provider_attempts_complete,
        execution: ExecutionSummary::default(),
        latency_ms: failure.latency_ms,
        usage: failure.usage,
        provider_model: failure.provider_model,
        finish_reason: failure.finish_reason,
        assessment: None,
        failure_class: Some(failure.class),
        failure: Some(failure.message),
    }
}

async fn execute_adapter_call(
    adapter: &dyn ModelAdapter,
    request: ModelRequest,
    budget: &mut CaseExecutionBudget,
) -> Result<AdapterCallSuccess, AdapterCallFailure> {
    if budget.remaining_active_ms == 0 {
        return Err(AdapterCallFailure {
            error: Some(ModelError::new(
                ModelErrorKind::Timeout,
                "semantic active execution budget exhausted before provider call",
            )),
            execution: ExecutionSummary::default(),
            absolute_timeout: false,
        });
    }

    let before = adapter.execution_telemetry_snapshot();
    adapter.configure_execution_budget(Some(budget.provider_budget()));
    let started = Instant::now();

    let provider_future = adapter.generate(request);
    let no_provider_telemetry = before.is_none();
    let call_deadline = if no_provider_telemetry {
        let active_deadline =
            tokio::time::Instant::now() + Duration::from_millis(budget.remaining_active_ms);
        active_deadline.min(budget.absolute_deadline)
    } else {
        budget.absolute_deadline
    };

    let result = tokio::time::timeout_at(call_deadline, provider_future).await;
    adapter.configure_execution_budget(None);
    let after = adapter.execution_telemetry_snapshot();

    let call_elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let active_before_ms = budget.remaining_active_ms;
    let mut execution = match (before, after) {
        (Some(before), Some(after)) => ExecutionSummary::from(after.saturating_delta(before)),
        _ => {
            let attempts = match &result {
                Ok(Ok(response)) => u64::from(response.provider_attempts),
                Ok(Err(error)) => u64::from(error.provider_attempts),
                Err(_) => 0,
            };
            ExecutionSummary {
                provider_attempts_started: attempts,
                provider_attempts_completed: attempts,
                active_ms: call_elapsed_ms,
                ..ExecutionSummary::default()
            }
        }
    };
    let inferred_active_ms = call_elapsed_ms.saturating_sub(execution.wait_ms);
    execution.active_ms = execution.active_ms.max(inferred_active_ms);
    let active_budget_exceeded = execution.active_ms > active_before_ms;
    budget.consume(execution);

    if active_budget_exceeded && result.is_ok() {
        return Err(AdapterCallFailure {
            error: Some(
                ModelError::new(
                    ModelErrorKind::Timeout,
                    "provider active execution exceeded semantic execution budget",
                )
                .with_provider_attempts(
                    u32::try_from(execution.provider_attempts_started).unwrap_or(u32::MAX),
                ),
            ),
            execution,
            absolute_timeout: false,
        });
    }

    match result {
        Ok(Ok(response)) => Ok(AdapterCallSuccess { response }),
        Ok(Err(error)) => Err(AdapterCallFailure {
            error: Some(error),
            execution,
            absolute_timeout: false,
        }),
        Err(_) => Err(AdapterCallFailure {
            error: None,
            execution,
            absolute_timeout: call_deadline == budget.absolute_deadline,
        }),
    }
}

#[allow(clippy::too_many_arguments)]
fn adapter_call_timeout_failure(
    failure: AdapterCallFailure,
    context: &str,
    used_structured_fallback: bool,
    model_calls: u32,
    prior_provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
) -> CallFailure {
    let started = u32::try_from(failure.execution.provider_attempts_started).unwrap_or(u32::MAX);
    let class = if failure.absolute_timeout {
        "case_absolute_timeout"
    } else {
        "assessment_timeout"
    };
    CallFailure {
        class: class.into(),
        message: format!("{context}; {class}"),
        used_structured_fallback,
        model_calls,
        provider_attempts: prior_provider_attempts.saturating_add(started),
        provider_attempts_complete: false,
        usage,
        provider_model,
        finish_reason,
    }
}

fn adapter_call_model_failure(
    failure: AdapterCallFailure,
    used_structured_fallback: bool,
    model_calls: u32,
    prior_provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
) -> CallFailure {
    match failure.error {
        Some(error) => {
            let started =
                u32::try_from(failure.execution.provider_attempts_started).unwrap_or(u32::MAX);
            let completed =
                u32::try_from(failure.execution.provider_attempts_completed).unwrap_or(u32::MAX);
            let attempts = started.max(error.provider_attempts);
            let mut result = model_failure(
                error,
                used_structured_fallback,
                model_calls,
                prior_provider_attempts.saturating_add(attempts),
                usage,
                provider_model,
                finish_reason,
            );
            result.provider_attempts_complete = started == completed;
            result
        }
        None => adapter_call_timeout_failure(
            failure,
            "provider call exceeded bounded execution budget",
            used_structured_fallback,
            model_calls,
            prior_provider_attempts,
            usage,
            provider_model,
            finish_reason,
        ),
    }
}

async fn call_model_for_proposal(
    adapter: &dyn ModelAdapter,
    provider: Provider,
    request: ModelRequest,
    max_model_calls: u32,
    budget: &mut CaseExecutionBudget,
) -> Result<CallOutcome, CallFailure> {
    let mut model_calls = 0u32;
    let mut provider_attempts = 0u32;
    let mut usage = UsageSummary::default();
    let mut last_model = None;
    let mut last_finish_reason = None;

    if max_model_calls == 0 {
        return Err(CallFailure {
            class: "attempt_budget".into(),
            message: "model-call budget is zero".into(),
            used_structured_fallback: false,
            model_calls,
            provider_attempts,
            provider_attempts_complete: true,
            usage,
            provider_model: last_model,
            finish_reason: last_finish_reason,
        });
    }

    let groq_text_primary = matches!(provider, Provider::Groq);
    let primary_request = if groq_text_primary {
        let Some(mut text_request) = build_strict_json_text_fallback_request(&request) else {
            return Err(CallFailure {
                class: "protocol".into(),
                message: "Groq strict-JSON Text primary transport unavailable".into(),
                used_structured_fallback: false,
                model_calls,
                provider_attempts,
                provider_attempts_complete: true,
                usage,
                provider_model: last_model,
                finish_reason: last_finish_reason,
            });
        };
        text_request.max_tokens = Some(
            text_request
                .max_tokens
                .unwrap_or_default()
                .max(GROQ_STRICT_JSON_TEXT_MAX_TOKENS),
        );
        text_request
    } else {
        request.clone()
    };

    model_calls += 1;
    let primary = execute_adapter_call(adapter, primary_request, budget).await;

    match primary {
        Ok(call) => {
            let response = call.response;
            provider_attempts = provider_attempts.saturating_add(response.provider_attempts);
            usage.add(&response.usage);
            last_model = Some(response.model.clone());
            last_finish_reason = response.finish_reason.clone();
            match parse_evidence_relevance_binding_proposal(&response.text) {
                Ok(proposal) => Ok(CallOutcome {
                    proposal,
                    used_structured_fallback: false,
                    model_calls,
                    provider_attempts,
                    usage,
                    provider_model: last_model,
                    finish_reason: last_finish_reason,
                }),
                Err(primary_parse_error) => {
                    if groq_text_primary {
                        return Err(CallFailure {
                            class: "protocol".into(),
                            message: format!(
                                "Groq strict-JSON Text primary proposal parse failed: {primary_parse_error}"
                            ),
                            used_structured_fallback: false,
                            model_calls,
                            provider_attempts,
                            provider_attempts_complete: true,
                            usage,
                            provider_model: last_model,
                            finish_reason: last_finish_reason,
                        });
                    }
                    let Some(fallback) = build_strict_json_text_fallback_request(&request) else {
                        return Err(CallFailure {
                            class: "protocol".into(),
                            message: format!(
                                "primary structured proposal parse failed and no fallback exists: {primary_parse_error}"
                            ),
                            used_structured_fallback: false,
                            model_calls,
                            provider_attempts,
                            provider_attempts_complete: true,
                            usage,
                            provider_model: last_model,
                            finish_reason: last_finish_reason,
                        });
                    };
                    call_fallback(
                        adapter,
                        fallback,
                        budget,
                        max_model_calls,
                        model_calls,
                        provider_attempts,
                        usage,
                        last_model,
                        last_finish_reason,
                        format!("primary structured proposal parse failed: {primary_parse_error}"),
                    )
                    .await
                }
            }
        }
        Err(failure) => {
            if let Some(error) = failure.error.as_ref()
                && error.kind == ModelErrorKind::UnsupportedCapability
                && !groq_text_primary
            {
                let error = failure.error.expect("checked error");
                let attempts = error.provider_attempts;
                provider_attempts = provider_attempts.saturating_add(attempts);
                let Some(fallback) = build_strict_json_text_fallback_request(&request) else {
                    return Err(model_failure(
                        error,
                        false,
                        model_calls,
                        provider_attempts,
                        usage,
                        last_model,
                        last_finish_reason,
                    ));
                };
                return call_fallback(
                    adapter,
                    fallback,
                    budget,
                    max_model_calls,
                    model_calls,
                    provider_attempts,
                    usage,
                    last_model,
                    last_finish_reason,
                    "primary JSON-Schema capability unsupported".into(),
                )
                .await;
            }

            Err(adapter_call_model_failure(
                failure,
                false,
                model_calls,
                provider_attempts,
                usage,
                last_model,
                last_finish_reason,
            ))
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn call_fallback(
    adapter: &dyn ModelAdapter,
    fallback: ModelRequest,
    budget: &mut CaseExecutionBudget,
    max_model_calls: u32,
    prior_model_calls: u32,
    prior_provider_attempts: u32,
    mut usage: UsageSummary,
    prior_model: Option<String>,
    prior_finish_reason: Option<String>,
    primary_context: String,
) -> Result<CallOutcome, CallFailure> {
    if prior_model_calls >= max_model_calls {
        return Err(CallFailure {
            class: "attempt_budget".into(),
            message: format!(
                "{primary_context}; strict-JSON text fallback blocked by model-call budget"
            ),
            used_structured_fallback: false,
            model_calls: prior_model_calls,
            provider_attempts: prior_provider_attempts,
            provider_attempts_complete: true,
            usage,
            provider_model: prior_model,
            finish_reason: prior_finish_reason,
        });
    }

    let model_calls = prior_model_calls + 1;
    match execute_adapter_call(adapter, fallback, budget).await {
        Ok(call) => {
            let response = call.response;
            let provider_attempts =
                prior_provider_attempts.saturating_add(response.provider_attempts);
            usage.add(&response.usage);
            let model = Some(response.model.clone());
            let finish_reason = response.finish_reason.clone();
            match parse_evidence_relevance_binding_proposal(&response.text) {
                Ok(proposal) => Ok(CallOutcome {
                    proposal,
                    used_structured_fallback: true,
                    model_calls,
                    provider_attempts,
                    usage,
                    provider_model: model,
                    finish_reason,
                }),
                Err(error) => Err(CallFailure {
                    class: "protocol".into(),
                    message: format!(
                        "{primary_context}; strict-JSON text fallback proposal parse failed: {error}"
                    ),
                    used_structured_fallback: true,
                    model_calls,
                    provider_attempts,
                    provider_attempts_complete: true,
                    usage,
                    provider_model: model,
                    finish_reason,
                }),
            }
        }
        Err(failure) => {
            let mut result = adapter_call_model_failure(
                failure,
                true,
                model_calls,
                prior_provider_attempts,
                usage,
                prior_model,
                prior_finish_reason,
            );
            result.message = format!("{primary_context}; fallback failed: {}", result.message);
            Err(result)
        }
    }
}

async fn call_model_for_local_qualification(
    adapter: &dyn ModelAdapter,
    provider: Provider,
    request: ModelRequest,
    budget: &mut CaseExecutionBudget,
) -> Result<QualificationCallOutcome, CallFailure> {
    if QUALIFICATION_STAGE_MAX_MODEL_CALLS == 0 {
        return Err(CallFailure {
            class: "attempt_budget".into(),
            message: "local qualification model-call budget is zero".into(),
            used_structured_fallback: false,
            model_calls: 0,
            provider_attempts: 0,
            provider_attempts_complete: true,
            usage: UsageSummary::default(),
            provider_model: None,
            finish_reason: None,
        });
    }

    let groq_text_primary = matches!(provider, Provider::Groq);
    let primary_request = if groq_text_primary {
        let Some(mut text_request) = build_strict_json_text_fallback_request(&request) else {
            return Err(CallFailure {
                class: "protocol".into(),
                message: "Groq strict-JSON Text primary local qualification transport unavailable"
                    .into(),
                used_structured_fallback: false,
                model_calls: 0,
                provider_attempts: 0,
                provider_attempts_complete: true,
                usage: UsageSummary::default(),
                provider_model: None,
                finish_reason: None,
            });
        };
        text_request.max_tokens = Some(
            text_request
                .max_tokens
                .unwrap_or_default()
                .max(GROQ_STRICT_JSON_TEXT_MAX_TOKENS),
        );
        text_request
    } else {
        request.clone()
    };

    match execute_adapter_call(adapter, primary_request, budget).await {
        Ok(call) => {
            let response = call.response;
            let mut usage = UsageSummary::default();
            usage.add(&response.usage);
            let attempts = response.provider_attempts;
            let model = Some(response.model.clone());
            let finish = response.finish_reason.clone();
            match parse_evidence_local_qualification_v8(&response.text) {
                Ok(qualification) => Ok(QualificationCallOutcome {
                    qualification,
                    used_structured_fallback: false,
                    model_calls: 1,
                    provider_attempts: attempts,
                    usage,
                    provider_model: model,
                    finish_reason: finish,
                }),
                Err(error) => {
                    if groq_text_primary {
                        return Err(CallFailure {
                            class: "protocol".into(),
                            message: format!(
                                "Groq strict-JSON Text primary local qualification parse failed: {error}"
                            ),
                            used_structured_fallback: false,
                            model_calls: 1,
                            provider_attempts: attempts,
                            provider_attempts_complete: true,
                            usage,
                            provider_model: model,
                            finish_reason: finish,
                        });
                    }
                    call_local_qualification_fallback(
                        adapter,
                        &request,
                        budget,
                        1,
                        attempts,
                        usage,
                        model,
                        finish,
                        format!("primary structured local qualification parse failed: {error}"),
                    )
                    .await
                }
            }
        }
        Err(failure) => {
            if let Some(error) = failure.error.as_ref()
                && error.kind == ModelErrorKind::UnsupportedCapability
                && !groq_text_primary
            {
                let error = failure.error.expect("checked error");
                return call_local_qualification_fallback(
                    adapter,
                    &request,
                    budget,
                    1,
                    error.provider_attempts,
                    UsageSummary::default(),
                    None,
                    None,
                    "primary local qualification JSON-Schema capability unsupported".into(),
                )
                .await;
            }
            Err(adapter_call_model_failure(
                failure,
                false,
                1,
                0,
                UsageSummary::default(),
                None,
                None,
            ))
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn call_local_qualification_fallback(
    adapter: &dyn ModelAdapter,
    request: &ModelRequest,
    budget: &mut CaseExecutionBudget,
    prior_model_calls: u32,
    prior_provider_attempts: u32,
    mut usage: UsageSummary,
    prior_model: Option<String>,
    prior_finish_reason: Option<String>,
    primary_context: String,
) -> Result<QualificationCallOutcome, CallFailure> {
    if prior_model_calls >= QUALIFICATION_STAGE_MAX_MODEL_CALLS {
        return Err(CallFailure {
            class: "attempt_budget".into(),
            message: format!(
                "{primary_context}; strict-JSON text fallback blocked by model-call budget"
            ),
            used_structured_fallback: false,
            model_calls: prior_model_calls,
            provider_attempts: prior_provider_attempts,
            provider_attempts_complete: true,
            usage,
            provider_model: prior_model,
            finish_reason: prior_finish_reason,
        });
    }

    let Some(fallback) = build_strict_json_text_fallback_request(request) else {
        return Err(CallFailure {
            class: "protocol".into(),
            message: format!("{primary_context}; structured fallback unavailable"),
            used_structured_fallback: false,
            model_calls: prior_model_calls,
            provider_attempts: prior_provider_attempts,
            provider_attempts_complete: true,
            usage,
            provider_model: prior_model,
            finish_reason: prior_finish_reason,
        });
    };

    let model_calls = prior_model_calls + 1;
    match execute_adapter_call(adapter, fallback, budget).await {
        Ok(call) => {
            let response = call.response;
            let attempts = prior_provider_attempts.saturating_add(response.provider_attempts);
            usage.add(&response.usage);
            let model = Some(response.model.clone());
            let finish_reason = response.finish_reason.clone();
            match parse_evidence_local_qualification_v8(&response.text) {
                Ok(qualification) => Ok(QualificationCallOutcome {
                    qualification,
                    used_structured_fallback: true,
                    model_calls,
                    provider_attempts: attempts,
                    usage,
                    provider_model: model,
                    finish_reason,
                }),
                Err(error) => Err(CallFailure {
                    class: "protocol".into(),
                    message: format!(
                        "{primary_context}; strict-JSON text fallback local qualification parse failed: {error}"
                    ),
                    used_structured_fallback: true,
                    model_calls,
                    provider_attempts: attempts,
                    provider_attempts_complete: true,
                    usage,
                    provider_model: model,
                    finish_reason,
                }),
            }
        }
        Err(failure) => {
            let mut result = adapter_call_model_failure(
                failure,
                true,
                model_calls,
                prior_provider_attempts,
                usage,
                prior_model,
                prior_finish_reason,
            );
            result.message = format!("{primary_context}; fallback failed: {}", result.message);
            Err(result)
        }
    }
}

fn model_failure(
    error: ModelError,
    used_structured_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
) -> CallFailure {
    let kind = error.kind;
    let message = sanitize_provider_failure_message(kind, &error.message);
    CallFailure {
        class: model_error_class(kind).into(),
        message,
        used_structured_fallback,
        model_calls,
        provider_attempts,
        provider_attempts_complete: true,
        usage,
        provider_model,
        finish_reason,
    }
}

fn sanitize_provider_failure_message(kind: ModelErrorKind, message: &str) -> String {
    match kind {
        ModelErrorKind::Quota => {
            let normalized = message.to_ascii_lowercase();
            let scope = if normalized.contains("tokens per day")
                || normalized.contains("tpd")
                || normalized.contains("perday")
                || normalized.contains("daily")
            {
                "daily"
            } else {
                "unspecified"
            };
            format!("provider quota failure; scope={scope}")
        }
        ModelErrorKind::RateLimit => "provider rate-limit failure".into(),
        ModelErrorKind::Credentials => "provider credential failure".into(),
        _ => redact_provider_message_tokens(message),
    }
}

fn redact_provider_message_tokens(message: &str) -> String {
    message
        .split_whitespace()
        .map(|token| {
            let lower = token.to_ascii_lowercase();
            if lower.contains("http://")
                || lower.contains("https://")
                || lower.contains("org_")
                || lower.contains("organization_id")
                || lower.contains("project_id")
                || lower.contains("account_id")
            {
                "<redacted>"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn model_error_class(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Credentials => "credentials",
        ModelErrorKind::Transport => "transport",
        ModelErrorKind::Provider => "provider",
        ModelErrorKind::RateLimit => "rate_limit",
        ModelErrorKind::Quota => "quota",
        ModelErrorKind::ProviderUnavailable => "provider_unavailable",
        ModelErrorKind::Timeout => "timeout",
        ModelErrorKind::Protocol => "protocol",
        ModelErrorKind::UnsupportedCapability => "unsupported_capability",
    }
}

fn should_abort_after_operational_failure(
    continue_after_operational_failures: bool,
    consecutive_operational_failures: usize,
    max_consecutive_operational_failures: usize,
    has_remaining_cases: bool,
) -> bool {
    !continue_after_operational_failures
        && has_remaining_cases
        && consecutive_operational_failures >= max_consecutive_operational_failures
}

fn next_capacity_failure_streak(current: usize, failure_class: Option<&str>) -> usize {
    if failure_class.is_some_and(is_capacity_failure_class) {
        current.saturating_add(1)
    } else {
        0
    }
}

fn is_capacity_failure_class(class: &str) -> bool {
    matches!(
        class,
        "assessment_timeout"
            | "case_absolute_timeout"
            | "provider_unavailable"
            | "rate_limit"
            | "timeout"
            | "transport"
    )
}

fn next_operational_failure_streak(current: usize, failure_class: Option<&str>) -> usize {
    if failure_class.is_some_and(is_operational_provider_failure_class) {
        current.saturating_add(1)
    } else {
        0
    }
}

fn is_operational_provider_failure_class(class: &str) -> bool {
    matches!(
        class,
        "assessment_timeout"
            | "case_absolute_timeout"
            | "provider_arm_latched"
            | "credentials"
            | "provider"
            | "provider_unavailable"
            | "protocol"
            | "quota"
            | "rate_limit"
            | "timeout"
            | "transport"
    )
}

fn repository_root() -> Result<PathBuf, String> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("canonicalize repository root: {error}"))
}

fn load_manifest(target: &Path) -> Result<CalibrationManifest, String> {
    let expected = repository_root()?
        .join(EXPECTED_RELATIVE_DIR)
        .canonicalize()
        .map_err(|error| format!("canonicalize expected calibration directory: {error}"))?;
    let target = target
        .canonicalize()
        .map_err(|error| format!("canonicalize target directory: {error}"))?;
    if target != expected {
        return Err(format!(
            "evidence-relevance study accepts only this checkout's {EXPECTED_RELATIVE_DIR}"
        ));
    }

    let raw = fs::read_to_string(target.join("manifest.json"))
        .map_err(|error| format!("read calibration manifest: {error}"))?;
    let manifest: CalibrationManifest = serde_json::from_str(&raw)
        .map_err(|error| format!("parse calibration manifest: {error}"))?;

    if manifest.suite_id != EXPECTED_SUITE_ID {
        return Err(format!(
            "unexpected calibration suite id {:?}",
            manifest.suite_id
        ));
    }
    if manifest.issue != 462 {
        return Err(format!("unexpected issue binding {}", manifest.issue));
    }
    if manifest.status != EXPECTED_STATUS {
        return Err(format!(
            "unexpected calibration status {:?}",
            manifest.status
        ));
    }
    if manifest.annotation_protocol_id != EXPECTED_ANNOTATION_PROTOCOL_ID {
        return Err(format!(
            "unexpected annotation protocol id {:?}",
            manifest.annotation_protocol_id
        ));
    }
    if manifest.fixed_core_id != EXPECTED_FIXED_CORE_ID {
        return Err(format!(
            "unexpected fixed core id {:?}",
            manifest.fixed_core_id
        ));
    }
    if manifest.cases.len() != EXPECTED_CASES {
        return Err(format!(
            "expected {EXPECTED_CASES} calibration cases, got {}",
            manifest.cases.len()
        ));
    }
    if manifest
        .source_rule
        .to_ascii_lowercase()
        .contains("production_incident_included")
    {
        return Err("production incident must remain excluded from calibration tuning".into());
    }
    let mut ids = std::collections::BTreeSet::new();
    for case in &manifest.cases {
        if case.id.trim().is_empty()
            || case.family.trim().is_empty()
            || case._task.trim().is_empty()
        {
            return Err("calibration case id/family/task must not be empty".into());
        }
        if !ids.insert(case.id.as_str()) {
            return Err(format!("duplicate calibration case id {}", case.id));
        }
    }

    Ok(manifest)
}

fn select_cases<'a>(
    manifest: &'a CalibrationManifest,
    fixture_ids: &[String],
) -> Result<Vec<&'a CalibrationCase>, String> {
    if fixture_ids.is_empty() {
        return Ok(manifest.cases.iter().collect());
    }

    let mut selected = Vec::new();
    for id in fixture_ids {
        let case = manifest
            .cases
            .iter()
            .find(|case| &case.id == id)
            .ok_or_else(|| format!("unknown fixture id {id}"))?;
        if selected
            .iter()
            .any(|existing: &&CalibrationCase| existing.id == case.id)
        {
            return Err(format!("duplicate fixture id {id}"));
        }
        selected.push(case);
    }
    Ok(selected)
}

fn normalized_tokens(value: &str) -> Vec<String> {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

fn simple_lexical_baseline(
    policy: &EvidenceRelevanceTargetPolicy,
    candidate: &EvidenceRelevanceCandidate,
) -> EvidenceRelevanceDisposition {
    let Some(entity) = &policy.entity else {
        return EvidenceRelevanceDisposition::Ambiguous;
    };

    let names = std::iter::once(&entity.canonical_name)
        .chain(entity.aliases.iter())
        .map(|name| normalized_tokens(name))
        .filter(|tokens| !tokens.is_empty())
        .collect::<Vec<_>>();

    for signal in &candidate.signals {
        if matches!(
            signal.kind,
            EvidenceRelevanceSignalKind::CanonicalUrl
                | EvidenceRelevanceSignalKind::NavigationOrFooter
        ) {
            continue;
        }
        let signal_tokens = normalized_tokens(&signal.text);
        for name_tokens in &names {
            if name_tokens
                .iter()
                .all(|token| signal_tokens.iter().any(|candidate| candidate == token))
            {
                return EvidenceRelevanceDisposition::Relevant;
            }
        }
    }

    EvidenceRelevanceDisposition::Ambiguous
}

fn summarize_metrics(observations: &[CaseObservation]) -> CalibrationMetrics {
    let successful = observations
        .iter()
        .filter(|case| case.failure.is_none())
        .count();
    let failed = observations.len().saturating_sub(successful);
    let proposal_exact_matches = observations
        .iter()
        .filter(|case| case.proposal_match == Some(true))
        .count();
    let materialized_exact_matches = observations
        .iter()
        .filter(|case| case.disposition_match == Some(true))
        .count();
    let local_qualification_expected_cases = observations.len();
    let local_qualification_invocations = observations
        .iter()
        .filter(|case| case.local_qualification_invoked)
        .count();
    let local_qualification_exact_matches = observations
        .iter()
        .filter(|case| case.local_qualification_match == Some(true))
        .count();
    let qualification_scope_risk_misses = observations
        .iter()
        .filter(|case| {
            case.expected_local_qualification.scope_risk != EvidenceLocalBlockingReason::None
                && case
                    .observed_local_qualification
                    .as_ref()
                    .is_some_and(|observed| {
                        observed.scope_risk != case.expected_local_qualification.scope_risk
                    })
        })
        .count();
    let qualification_spurious_scope_risks = observations
        .iter()
        .filter(|case| {
            case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None
                && case
                    .observed_local_qualification
                    .as_ref()
                    .is_some_and(|observed| {
                        observed.scope_risk != EvidenceLocalBlockingReason::None
                    })
        })
        .count();
    let qualification_identity_scope_misses = observations
        .iter()
        .filter(|case| {
            case.observed_local_qualification
                .as_ref()
                .is_some_and(|observed| {
                    observed.identity_scope != case.expected_local_qualification.identity_scope
                })
        })
        .count();
    let qualification_relation_scope_misses = observations
        .iter()
        .filter(|case| {
            case.observed_local_qualification
                .as_ref()
                .is_some_and(|observed| {
                    observed.relation_scope != case.expected_local_qualification.relation_scope
                })
        })
        .count();
    let effective_local_qualification_expected_cases = observations.len();
    let effective_local_qualification_derivations = observations
        .iter()
        .filter(|case| case.effective_local_qualification_derived)
        .count();
    let effective_local_qualification_exact_matches = observations
        .iter()
        .filter(|case| case.effective_local_qualification_match == Some(true))
        .count();
    let effective_qualification_scope_risk_misses = observations
        .iter()
        .filter(|case| {
            case.expected_local_qualification.scope_risk != EvidenceLocalBlockingReason::None
                && case
                    .effective_local_qualification
                    .as_ref()
                    .is_some_and(|effective| {
                        effective.scope_risk != case.expected_local_qualification.scope_risk
                    })
        })
        .count();
    let effective_qualification_spurious_scope_risks = observations
        .iter()
        .filter(|case| {
            case.expected_local_qualification.scope_risk == EvidenceLocalBlockingReason::None
                && case
                    .effective_local_qualification
                    .as_ref()
                    .is_some_and(|effective| {
                        effective.scope_risk != EvidenceLocalBlockingReason::None
                    })
        })
        .count();
    let effective_qualification_identity_scope_misses = observations
        .iter()
        .filter(|case| {
            case.effective_local_qualification
                .as_ref()
                .is_some_and(|effective| {
                    effective.identity_scope != case.expected_local_qualification.identity_scope
                })
        })
        .count();
    let effective_qualification_relation_scope_misses = observations
        .iter()
        .filter(|case| {
            case.effective_local_qualification
                .as_ref()
                .is_some_and(|effective| {
                    effective.relation_scope != case.expected_local_qualification.relation_scope
                })
        })
        .count();

    let correctness_wrong_target_relevance_retention = observations
        .iter()
        .filter(|case| {
            case.materialized_disposition == Some(EvidenceRelevanceDisposition::Relevant)
                && case.expected_disposition != EvidenceRelevanceDisposition::Relevant
        })
        .count();
    let false_relevance_rejections = observations
        .iter()
        .filter(|case| {
            case.expected_disposition == EvidenceRelevanceDisposition::Relevant
                && case.materialized_disposition == Some(EvidenceRelevanceDisposition::Irrelevant)
        })
        .count();
    let relevant_left_ambiguous = observations
        .iter()
        .filter(|case| {
            case.expected_disposition == EvidenceRelevanceDisposition::Relevant
                && case.materialized_disposition == Some(EvidenceRelevanceDisposition::Ambiguous)
        })
        .count();
    let utility_misses = observations
        .iter()
        .filter(|case| {
            case.failure.is_none()
                && case.materialized_disposition.is_some()
                && case.materialized_disposition != Some(case.expected_disposition)
                && !(case.materialized_disposition == Some(EvidenceRelevanceDisposition::Relevant)
                    && case.expected_disposition != EvidenceRelevanceDisposition::Relevant)
        })
        .count();
    let ambiguous_dispositions = observations
        .iter()
        .filter(|case| {
            case.materialized_disposition == Some(EvidenceRelevanceDisposition::Ambiguous)
        })
        .count();
    let deterministic_guard_overrides = observations
        .iter()
        .filter(|case| case.deterministic_guard_override)
        .count();
    let lexical_exact_matches = observations
        .iter()
        .filter(|case| case.lexical_baseline_match)
        .count();
    let lexical_wrong_target_relevance_retention = observations
        .iter()
        .filter(|case| {
            case.lexical_baseline == EvidenceRelevanceDisposition::Relevant
                && case.expected_disposition != EvidenceRelevanceDisposition::Relevant
        })
        .count();
    let lexical_expected_relevant_misses = observations
        .iter()
        .filter(|case| {
            case.expected_disposition == EvidenceRelevanceDisposition::Relevant
                && case.lexical_baseline != EvidenceRelevanceDisposition::Relevant
        })
        .count();

    let model_calls = observations
        .iter()
        .map(|case| case.model_calls as u64)
        .sum();
    let provider_attempts = observations
        .iter()
        .map(|case| case.provider_attempts as u64)
        .sum();
    let provider_attempts_started = observations
        .iter()
        .map(|case| case.execution.provider_attempts_started)
        .sum();
    let provider_attempts_completed = observations
        .iter()
        .map(|case| case.execution.provider_attempts_completed)
        .sum();
    let active_execution_ms = observations
        .iter()
        .map(|case| case.execution.active_ms)
        .sum();
    let provider_wait_ms = observations.iter().map(|case| case.execution.wait_ms).sum();
    let pacing_wait_ms = observations
        .iter()
        .map(|case| case.execution.pacing_wait_ms)
        .sum();
    let retry_wait_ms = observations
        .iter()
        .map(|case| case.execution.retry_wait_ms)
        .sum();
    let input_tokens = observations
        .iter()
        .map(|case| case.usage.input_tokens)
        .sum();
    let output_tokens = observations
        .iter()
        .map(|case| case.usage.output_tokens)
        .sum();
    let total_tokens = observations
        .iter()
        .map(|case| case.usage.total_tokens)
        .sum();
    let total_latency_ms = observations.iter().map(|case| case.latency_ms).sum();
    let mean_latency_ms =
        (!observations.is_empty()).then(|| total_latency_ms as f64 / observations.len() as f64);
    let latencies = observations
        .iter()
        .map(|case| case.latency_ms)
        .collect::<Vec<_>>();
    let successful_latencies = observations
        .iter()
        .filter(|case| case.failure.is_none())
        .map(|case| case.latency_ms)
        .collect::<Vec<_>>();
    let latency_p50_ms = percentile_latency(&latencies, 50);
    let latency_p95_ms = percentile_latency(&latencies, 95);
    let latency_max_ms = latencies.iter().copied().max();
    let successful_latency_p50_ms = percentile_latency(&successful_latencies, 50);
    let successful_latency_p95_ms = percentile_latency(&successful_latencies, 95);
    let successful_latency_max_ms = successful_latencies.iter().copied().max();
    let provider_attempts_incomplete_observations = observations
        .iter()
        .filter(|case| !case.provider_attempts_complete)
        .count();

    let mut failure_counts = BTreeMap::new();
    for failure in observations
        .iter()
        .filter_map(|case| case.failure_class.as_ref())
    {
        *failure_counts.entry(failure.clone()).or_insert(0) += 1;
    }

    CalibrationMetrics {
        cases: observations.len(),
        successful_provider_cases: successful,
        failed_provider_cases: failed,
        proposal_exact_matches,
        proposal_exact_accuracy: accuracy(proposal_exact_matches, successful),
        local_qualification_expected_cases,
        local_qualification_invocations,
        local_qualification_exact_matches,
        local_qualification_exact_accuracy: accuracy(
            local_qualification_exact_matches,
            local_qualification_expected_cases,
        ),
        qualification_scope_risk_misses,
        qualification_spurious_scope_risks,
        qualification_identity_scope_misses,
        qualification_relation_scope_misses,
        effective_local_qualification_expected_cases,
        effective_local_qualification_derivations,
        effective_local_qualification_exact_matches,
        effective_local_qualification_exact_accuracy: accuracy(
            effective_local_qualification_exact_matches,
            effective_local_qualification_expected_cases,
        ),
        effective_qualification_scope_risk_misses,
        effective_qualification_spurious_scope_risks,
        effective_qualification_identity_scope_misses,
        effective_qualification_relation_scope_misses,
        materialized_exact_matches,
        materialized_exact_accuracy: accuracy(materialized_exact_matches, successful),
        correctness_wrong_target_relevance_retention,
        false_relevance_rejections,
        relevant_left_ambiguous,
        utility_misses,
        ambiguous_dispositions,
        deterministic_guard_overrides,
        lexical_exact_matches,
        lexical_exact_accuracy: accuracy(lexical_exact_matches, observations.len()),
        lexical_wrong_target_relevance_retention,
        lexical_expected_relevant_misses,
        model_calls,
        provider_attempts,
        provider_attempts_started,
        provider_attempts_completed,
        provider_attempts_incomplete_observations,
        active_execution_ms,
        provider_wait_ms,
        pacing_wait_ms,
        retry_wait_ms,
        input_tokens,
        output_tokens,
        total_tokens,
        total_latency_ms,
        mean_latency_ms,
        latency_p50_ms,
        latency_p95_ms,
        latency_max_ms,
        successful_latency_p50_ms,
        successful_latency_p95_ms,
        successful_latency_max_ms,
        failure_counts,
    }
}

fn empty_metrics(cases: usize) -> CalibrationMetrics {
    CalibrationMetrics {
        cases,
        successful_provider_cases: 0,
        failed_provider_cases: 0,
        proposal_exact_matches: 0,
        proposal_exact_accuracy: None,
        local_qualification_expected_cases: 0,
        local_qualification_invocations: 0,
        local_qualification_exact_matches: 0,
        local_qualification_exact_accuracy: None,
        qualification_scope_risk_misses: 0,
        qualification_spurious_scope_risks: 0,
        qualification_identity_scope_misses: 0,
        qualification_relation_scope_misses: 0,
        effective_local_qualification_expected_cases: 0,
        effective_local_qualification_derivations: 0,
        effective_local_qualification_exact_matches: 0,
        effective_local_qualification_exact_accuracy: None,
        effective_qualification_scope_risk_misses: 0,
        effective_qualification_spurious_scope_risks: 0,
        effective_qualification_identity_scope_misses: 0,
        effective_qualification_relation_scope_misses: 0,
        materialized_exact_matches: 0,
        materialized_exact_accuracy: None,
        correctness_wrong_target_relevance_retention: 0,
        false_relevance_rejections: 0,
        relevant_left_ambiguous: 0,
        utility_misses: 0,
        ambiguous_dispositions: 0,
        deterministic_guard_overrides: 0,
        lexical_exact_matches: 0,
        lexical_exact_accuracy: None,
        lexical_wrong_target_relevance_retention: 0,
        lexical_expected_relevant_misses: 0,
        model_calls: 0,
        provider_attempts: 0,
        provider_attempts_started: 0,
        provider_attempts_completed: 0,
        provider_attempts_incomplete_observations: 0,
        active_execution_ms: 0,
        provider_wait_ms: 0,
        pacing_wait_ms: 0,
        retry_wait_ms: 0,
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
        total_latency_ms: 0,
        mean_latency_ms: None,
        latency_p50_ms: None,
        latency_p95_ms: None,
        latency_max_ms: None,
        successful_latency_p50_ms: None,
        successful_latency_p95_ms: None,
        successful_latency_max_ms: None,
        failure_counts: BTreeMap::new(),
    }
}

fn percentile_latency(values: &[u128], percentile: usize) -> Option<u128> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let percentile = percentile.clamp(1, 100);
    let rank = percentile.saturating_mul(sorted.len()).saturating_add(99) / 100;
    sorted.get(rank.saturating_sub(1)).copied()
}

fn accuracy(matches: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| matches as f64 / denominator as f64)
}

fn write_checkpoint(
    path: &Path,
    suite_id: &str,
    provider: &str,
    model: &str,
    expected_cases: usize,
    observations: &[CaseObservation],
    run_status: &'static str,
) -> Result<(), String> {
    let checkpoint = Checkpoint {
        checkpoint_version: "evidence-relevance-checkpoint-v1",
        run_status,
        configuration_id: CONFIGURATION_ID,
        suite_id,
        provider,
        model,
        expected_cases,
        completed_cases: observations.len(),
        scorability: if run_status == "completed"
            && observations.len() == expected_cases
            && observations.iter().all(|case| case.failure.is_none())
        {
            "complete_calibration_observation"
        } else if run_status == "completed" {
            "operationally_incomplete_non_scorable"
        } else {
            "non_scorable_in_progress"
        },
        observations,
    };
    let bytes = serde_json::to_vec_pretty(&checkpoint)
        .map_err(|error| format!("serialize checkpoint: {error}"))?;
    let temp = path.with_extension("tmp");
    fs::write(&temp, bytes).map_err(|error| format!("write checkpoint temp: {error}"))?;
    fs::rename(&temp, path).map_err(|error| format!("replace checkpoint: {error}"))?;
    Ok(())
}

fn git_head() -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("run git rev-parse: {error}"))?;
    if !output.status.success() {
        return Err("git rev-parse HEAD failed".into());
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("decode git HEAD: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use reasoning_harness_core::{
        EvidenceLocalBlockingReason, EvidenceLocalIdentityScope, EvidenceLocalRelationScope,
        EvidenceRelevanceBinding,
    };

    fn load() -> CalibrationManifest {
        let path = repository_root()
            .expect("repo root")
            .join(EXPECTED_RELATIVE_DIR);
        load_manifest(&path).expect("load calibration")
    }

    #[test]
    fn lexical_baseline_exposes_known_limitations() {
        let manifest = load();
        let exact = manifest
            .cases
            .iter()
            .filter(|case| {
                simple_lexical_baseline(&case.policy, &case.candidate) == case.expected_disposition
            })
            .count();
        assert!(exact < manifest.cases.len());

        let wrong_target = manifest
            .cases
            .iter()
            .filter(|case| {
                simple_lexical_baseline(&case.policy, &case.candidate)
                    == EvidenceRelevanceDisposition::Relevant
                    && case.expected_disposition != EvidenceRelevanceDisposition::Relevant
            })
            .count();
        assert!(wrong_target > 0);
    }

    #[test]
    fn expected_primary_and_effective_qualification_materialize_all_v19_cases() {
        let manifest = load();
        assert_eq!(manifest.cases.len(), EXPECTED_CASES);
        for case in manifest.cases {
            let proposal = case.expected_proposal;
            let assessment = materialize_evidence_relevance_v14(
                &case.policy,
                &case.candidate,
                Some(&proposal),
                Some(&case.expected_local_qualification),
            )
            .unwrap();
            assert_eq!(
                assessment.disposition, case.expected_disposition,
                "{}",
                case.id
            );
        }
    }
    struct SequenceAdapter {
        responses: std::sync::Mutex<
            std::collections::VecDeque<Result<reasoning_harness_core::ModelResponse, ModelError>>,
        >,
        delay_ms: u64,
    }

    impl SequenceAdapter {
        fn new(
            responses: Vec<Result<reasoning_harness_core::ModelResponse, ModelError>>,
            delay_ms: u64,
        ) -> Self {
            Self {
                responses: std::sync::Mutex::new(responses.into()),
                delay_ms,
            }
        }
    }

    impl ModelAdapter for SequenceAdapter {
        fn generate<'a>(
            &'a self,
            _request: ModelRequest,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<reasoning_harness_core::ModelResponse, ModelError>,
                    > + Send
                    + 'a,
            >,
        > {
            Box::pin(async move {
                if self.delay_ms > 0 {
                    tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
                }
                self.responses
                    .lock()
                    .unwrap()
                    .pop_front()
                    .expect("test response")
            })
        }
    }

    fn model_response(text: &str) -> Result<reasoning_harness_core::ModelResponse, ModelError> {
        Ok(reasoning_harness_core::ModelResponse {
            text: text.into(),
            model: "fixture-model".into(),
            usage: ModelUsage {
                input_tokens: Some(10),
                output_tokens: Some(2),
                total_tokens: Some(12),
            },
            provider_attempts: 1,
            finish_reason: Some("stop".into()),
        })
    }

    fn fixture_request() -> ModelRequest {
        let manifest = load();
        let case = &manifest.cases[0];
        build_evidence_relevance_binding_proposal_v5_request(
            &case.policy,
            &case.candidate,
            Some(462),
        )
        .expect("request")
    }

    #[tokio::test]
    async fn fallback_respects_model_call_budget() {
        let adapter = SequenceAdapter::new(
            vec![
                model_response("not json"),
                model_response(r#"{"target_binding":"exact","relation_binding":"exact"}"#),
            ],
            0,
        );
        let failure = call_model_for_proposal(
            &adapter,
            Provider::Mistral,
            fixture_request(),
            1,
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .unwrap_err();
        assert_eq!(failure.class, "attempt_budget");
        assert_eq!(failure.model_calls, 1);
        assert!(!failure.used_structured_fallback);
    }

    #[tokio::test]
    async fn assessment_elapsed_budget_fails_typed_and_closed() {
        let adapter = SequenceAdapter::new(
            vec![model_response(
                r#"{"target_binding":"exact","relation_binding":"exact"}"#,
            )],
            50,
        );
        let failure = call_model_for_proposal(
            &adapter,
            Provider::Mistral,
            fixture_request(),
            2,
            &mut CaseExecutionBudget::new(5),
        )
        .await
        .unwrap_err();
        assert_eq!(failure.class, "assessment_timeout");
        assert_eq!(failure.model_calls, 1);
        assert!(!failure.provider_attempts_complete);
    }

    #[test]
    fn operational_failure_streak_counts_provider_failures_and_resets_on_response_failure() {
        assert_eq!(
            next_operational_failure_streak(0, Some("assessment_timeout")),
            1
        );
        assert_eq!(
            next_operational_failure_streak(1, Some("provider_unavailable")),
            2
        );
        assert_eq!(next_operational_failure_streak(1, Some("protocol")), 2);
        assert_eq!(next_operational_failure_streak(2, None), 0);
        assert!(is_operational_provider_failure_class("rate_limit"));
        assert!(is_operational_provider_failure_class("quota"));
        assert!(!is_operational_provider_failure_class("materialization"));
    }

    #[test]
    fn groq_strict_json_transport_budget_exceeds_semantic_schema_budget() {
        assert_eq!(fixture_request().max_tokens, Some(192));
        assert_eq!(local_qualification_request().max_tokens, Some(192));
        assert_eq!(GROQ_STRICT_JSON_TEXT_MAX_TOKENS, 512);
    }

    #[test]
    fn full_diagnostic_mode_can_disable_operational_circuit_break() {
        let args = Args::try_parse_from([
            "reason-evidence-relevance-study",
            "fixtures/evidence-relevance-calibration-v19",
            "--provider",
            "groq",
            "--model",
            "openai/gpt-oss-120b",
            "--continue-after-operational-failures",
        ])
        .expect("parse args");
        assert!(args.continue_after_operational_failures);
        assert_eq!(args.max_consecutive_operational_failures, 2);
        assert_eq!(args.max_consecutive_capacity_failures, 2);
    }

    #[test]
    fn full_diagnostic_mode_never_opens_operational_circuit() {
        assert!(!should_abort_after_operational_failure(true, 2, 2, true));
        assert!(!should_abort_after_operational_failure(true, 99, 2, true));
        assert!(should_abort_after_operational_failure(false, 2, 2, true));
        assert!(!should_abort_after_operational_failure(false, 1, 2, true));
        assert!(!should_abort_after_operational_failure(false, 2, 2, false));
    }

    #[test]
    fn capacity_failure_streak_is_separate_from_protocol_and_quota() {
        assert_eq!(next_capacity_failure_streak(0, Some("rate_limit")), 1);
        assert_eq!(
            next_capacity_failure_streak(1, Some("provider_unavailable")),
            2
        );
        assert_eq!(next_capacity_failure_streak(2, Some("quota")), 0);
        assert_eq!(next_capacity_failure_streak(2, Some("protocol")), 0);
        assert!(is_capacity_failure_class("assessment_timeout"));
        assert!(is_capacity_failure_class("case_absolute_timeout"));
        assert!(!is_capacity_failure_class("quota"));
    }

    #[test]
    fn persisted_provider_failures_redact_quota_identity_and_urls() {
        let raw = "Rate limit reached in organization org_01secret on tokens per day (TPD). Upgrade at https://console.example/settings/billing";
        let quota = sanitize_provider_failure_message(ModelErrorKind::Quota, raw);
        assert_eq!(quota, "provider quota failure; scope=daily");
        assert!(!quota.contains("org_01secret"));
        assert!(!quota.contains("https://"));

        let transport = redact_provider_message_tokens(
            "request failed at https://api.example for org_01secret",
        );
        assert!(!transport.contains("https://api.example"));
        assert!(!transport.contains("org_01secret"));
        assert!(transport.contains("<redacted>"));
    }

    #[test]
    fn latency_percentiles_use_nearest_rank() {
        let values = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        assert_eq!(percentile_latency(&values, 50), Some(50));
        assert_eq!(percentile_latency(&values, 95), Some(100));
        assert_eq!(percentile_latency(&[42], 95), Some(42));
        assert_eq!(percentile_latency(&[], 95), None);
    }

    fn local_qualification_request() -> ModelRequest {
        let manifest = load();
        let case = &manifest.cases[0];
        build_evidence_local_qualification_v8_request(&case.policy, &case.candidate, Some(463))
            .expect("local qualification request")
    }

    fn qualification_json() -> &'static str {
        r#"{"identity_scope":"exact_target","relation_scope":"requested_relation","scope_risk":"none"}"#
    }

    #[test]
    fn local_qualification_request_uses_bounded_json_schema_transport() {
        let request = local_qualification_request();
        assert!(matches!(
            request.output_format,
            reasoning_harness_core::ModelOutputFormat::JsonSchema { .. }
        ));
        assert_eq!(request.max_tokens, Some(192));
    }

    #[tokio::test]
    async fn local_qualification_parser_returns_typed_state() {
        let adapter = SequenceAdapter::new(vec![model_response(qualification_json())], 0);
        let result = call_model_for_local_qualification(
            &adapter,
            Provider::Mistral,
            local_qualification_request(),
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .expect("qualification result");
        assert_eq!(
            result.qualification.identity_scope,
            EvidenceLocalIdentityScope::ExactTarget
        );
        assert_eq!(
            result.qualification.relation_scope,
            EvidenceLocalRelationScope::RequestedRelation
        );
        assert_eq!(
            result.qualification.scope_risk,
            EvidenceLocalBlockingReason::None
        );
        assert_eq!(result.model_calls, 1);
        assert_eq!(result.provider_attempts, 1);
        assert!(!result.used_structured_fallback);
    }

    #[tokio::test]
    async fn local_qualification_uses_one_transport_fallback_on_malformed_primary() {
        let adapter = SequenceAdapter::new(
            vec![
                model_response("not json"),
                model_response(qualification_json()),
            ],
            0,
        );
        let result = call_model_for_local_qualification(
            &adapter,
            Provider::Mistral,
            local_qualification_request(),
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .expect("qualification fallback result");
        assert_eq!(result.model_calls, 2);
        assert_eq!(result.provider_attempts, 2);
        assert!(result.used_structured_fallback);
    }

    #[tokio::test]
    async fn local_qualification_timeout_is_operational_failure() {
        let adapter = SequenceAdapter::new(vec![model_response(qualification_json())], 50);
        let failure = call_model_for_local_qualification(
            &adapter,
            Provider::Mistral,
            local_qualification_request(),
            &mut CaseExecutionBudget::new(5),
        )
        .await
        .unwrap_err();
        assert_eq!(failure.class, "assessment_timeout");
        assert!(!failure.provider_attempts_complete);
    }

    #[tokio::test]
    async fn local_qualification_unsupported_schema_uses_bounded_strict_json_text_fallback() {
        let adapter = SequenceAdapter::new(
            vec![
                Err(ModelError::new(
                    ModelErrorKind::UnsupportedCapability,
                    "schema unsupported",
                )),
                model_response(qualification_json()),
            ],
            0,
        );
        let result = call_model_for_local_qualification(
            &adapter,
            Provider::Mistral,
            local_qualification_request(),
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .expect("qualification fallback result");
        assert_eq!(result.model_calls, 2);
        assert_eq!(result.provider_attempts, 2);
        assert!(result.used_structured_fallback);
    }

    #[tokio::test]
    async fn local_qualification_malformed_primary_and_fallback_fail_closed_without_third_call() {
        let adapter = SequenceAdapter::new(
            vec![model_response("not json"), model_response("still not json")],
            0,
        );
        let failure = call_model_for_local_qualification(
            &adapter,
            Provider::Mistral,
            local_qualification_request(),
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .unwrap_err();
        assert_eq!(failure.class, "protocol");
        assert_eq!(failure.model_calls, 2);
        assert_eq!(failure.provider_attempts, 2);
        assert!(failure.used_structured_fallback);
    }

    #[tokio::test]
    async fn local_qualification_still_runs_when_primary_relation_is_different() {
        let manifest = load();
        let case = manifest
            .cases
            .iter()
            .find(|case| case.expected_disposition == EvidenceRelevanceDisposition::Relevant)
            .expect("positive case");
        let adapter = SequenceAdapter::new(vec![model_response(qualification_json())], 0);
        let call = CallOutcome {
            proposal: EvidenceRelevanceBindingProposal {
                target_binding: EvidenceRelevanceBinding::Exact,
                relation_binding: EvidenceRelevanceBinding::Different,
            },
            used_structured_fallback: false,
            model_calls: 1,
            provider_attempts: 1,
            usage: UsageSummary::default(),
            provider_model: Some("fixture-model".into()),
            finish_reason: Some("stop".into()),
        };
        let observation = complete_observed_case(
            &adapter,
            Provider::Mistral,
            case,
            Some(462),
            EvidenceRelevanceDisposition::Ambiguous,
            Instant::now(),
            call,
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .expect("observation");
        assert!(observation.local_qualification_invoked);
        assert_eq!(
            observation.materialized_disposition,
            Some(EvidenceRelevanceDisposition::Ambiguous)
        );
    }

    #[tokio::test]
    async fn relation_difference_requires_matching_local_confirmation() {
        let manifest = load();
        let case = manifest
            .cases
            .iter()
            .find(|case| case.id == "13_same_service_different_feature")
            .expect("relation-different case");
        let adapter = SequenceAdapter::new(
            vec![model_response(
                r#"{"identity_scope":"exact_target","relation_scope":"different_relation","scope_risk":"none"}"#,
            )],
            0,
        );
        let call = CallOutcome {
            proposal: case.expected_proposal,
            used_structured_fallback: false,
            model_calls: 1,
            provider_attempts: 1,
            usage: UsageSummary::default(),
            provider_model: Some("fixture-model".into()),
            finish_reason: Some("stop".into()),
        };
        let observation = complete_observed_case(
            &adapter,
            Provider::Mistral,
            case,
            Some(462),
            EvidenceRelevanceDisposition::Ambiguous,
            Instant::now(),
            call,
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .expect("observation");
        assert!(observation.local_qualification_invoked);
        assert_eq!(
            observation.materialized_disposition,
            Some(EvidenceRelevanceDisposition::Irrelevant)
        );
    }

    #[tokio::test]
    async fn bounded_fallback_records_two_model_calls() {
        let adapter = SequenceAdapter::new(
            vec![
                model_response("not json"),
                model_response(r#"{"target_binding":"exact","relation_binding":"exact"}"#),
            ],
            0,
        );
        let result = call_model_for_proposal(
            &adapter,
            Provider::Mistral,
            fixture_request(),
            2,
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .expect("fallback result");
        assert_eq!(result.model_calls, 2);
        assert_eq!(result.provider_attempts, 2);
        assert!(result.used_structured_fallback);
    }

    #[test]
    fn effective_metrics_are_separate_from_raw_verifier_metrics() {
        let manifest = load();
        let case = manifest
            .cases
            .iter()
            .find(|case| {
                case.expected_disposition == EvidenceRelevanceDisposition::Relevant
                    && case.expected_local_qualification.scope_risk
                        == EvidenceLocalBlockingReason::None
            })
            .expect("clean positive case");
        let raw = EvidenceLocalQualificationV6 {
            scope_risk: EvidenceLocalBlockingReason::ContextGap,
            ..case.expected_local_qualification
        };
        let effective = derive_effective_evidence_local_qualification_v1(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&raw),
        )
        .expect("effective qualification");
        let assessment = materialize_evidence_relevance_v14(
            &case.policy,
            &case.candidate,
            Some(&case.expected_proposal),
            Some(&raw),
        )
        .expect("materialization");
        let observation = success_observation(
            case,
            EvidenceRelevanceDisposition::Ambiguous,
            case.expected_proposal,
            raw,
            effective,
            assessment,
            1,
            false,
            2,
            2,
            UsageSummary::default(),
            Some("fixture-model".into()),
            Some("stop".into()),
        );
        let metrics = summarize_metrics(&[observation]);

        assert_eq!(metrics.local_qualification_exact_matches, 0);
        assert_eq!(metrics.qualification_spurious_scope_risks, 1);
        assert_eq!(metrics.effective_local_qualification_derivations, 1);
        assert_eq!(metrics.effective_local_qualification_exact_matches, 1);
        assert_eq!(metrics.effective_qualification_spurious_scope_risks, 0);
        assert_eq!(metrics.materialized_exact_matches, 1);
    }

    #[tokio::test]
    async fn groq_uses_strict_json_text_as_primary_without_second_fallback() {
        let adapter = SequenceAdapter::new(vec![model_response("not json")], 0);
        let failure = call_model_for_proposal(
            &adapter,
            Provider::Groq,
            fixture_request(),
            2,
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .unwrap_err();
        assert_eq!(failure.class, "protocol");
        assert_eq!(failure.model_calls, 1);
        assert_eq!(failure.provider_attempts, 1);
        assert!(!failure.used_structured_fallback);
    }

    #[tokio::test]
    async fn groq_local_qualification_uses_strict_json_text_as_primary_without_second_fallback() {
        let adapter = SequenceAdapter::new(vec![model_response("not json")], 0);
        let failure = call_model_for_local_qualification(
            &adapter,
            Provider::Groq,
            local_qualification_request(),
            &mut CaseExecutionBudget::new(1_000),
        )
        .await
        .unwrap_err();
        assert_eq!(failure.class, "protocol");
        assert_eq!(failure.model_calls, 1);
        assert_eq!(failure.provider_attempts, 1);
        assert!(!failure.used_structured_fallback);
    }
}
