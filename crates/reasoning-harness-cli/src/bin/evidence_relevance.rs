use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    EvidenceNegativeTargetConfirmationV2, EvidencePositiveTargetConfirmation,
    EvidenceRelevanceAssessment, EvidenceRelevanceBinding, EvidenceRelevanceBindingProposal,
    EvidenceRelevanceCandidate, EvidenceRelevanceDisposition, EvidenceRelevanceSignalKind,
    EvidenceRelevanceTargetPolicy, ModelAdapter, ModelError, ModelErrorKind, ModelRequest,
    ModelUsage, build_evidence_negative_target_confirmation_v2_request,
    build_evidence_positive_target_confirmation_request,
    build_evidence_relevance_binding_proposal_request, build_json_object_fallback_request,
    materialize_evidence_relevance_v5, parse_evidence_negative_target_confirmation_v2,
    parse_evidence_positive_target_confirmation, parse_evidence_relevance_binding_proposal,
};
use reasoning_harness_providers::{GoogleAdapter, GroqAdapter, MistralAdapter, NvidiaAdapter};
use serde::{Deserialize, Serialize};

const CONFIGURATION_ID: &str = "evidence-relevance-live-calibration-v9";
const EXPECTED_SUITE_ID: &str = "evidence-relevance-calibration-v9";
const EXPECTED_STATUS: &str = "fresh_unobserved_calibration";
const EXPECTED_RELATIVE_DIR: &str = "fixtures/evidence-relevance-calibration-v9";
const CONFIRMATION_STAGE_MAX_MODEL_CALLS: u32 = 1;
const EXPECTED_CASES: usize = 47;

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
    #[serde(default)]
    expected_negative_target_confirmation: Option<EvidenceNegativeTargetConfirmationV2>,
    #[serde(default)]
    expected_positive_target_confirmation: Option<EvidencePositiveTargetConfirmation>,
    expected_disposition: EvidenceRelevanceDisposition,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_negative_target_confirmation: Option<EvidenceNegativeTargetConfirmationV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_negative_target_confirmation: Option<EvidenceNegativeTargetConfirmationV2>,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative_target_confirmation_match: Option<bool>,
    negative_target_confirmation_invoked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_positive_target_confirmation: Option<EvidencePositiveTargetConfirmation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_positive_target_confirmation: Option<EvidencePositiveTargetConfirmation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    positive_target_confirmation_match: Option<bool>,
    positive_target_confirmation_invoked: bool,
    expected_disposition: EvidenceRelevanceDisposition,
    #[serde(skip_serializing_if = "Option::is_none")]
    materialized_disposition: Option<EvidenceRelevanceDisposition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disposition_match: Option<bool>,
    lexical_baseline: EvidenceRelevanceDisposition,
    lexical_baseline_match: bool,
    deterministic_safety_override: bool,
    used_json_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    provider_attempts_complete: bool,
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
    negative_target_confirmation_expected_cases: usize,
    negative_target_confirmation_invocations: usize,
    negative_target_confirmation_exact_matches: usize,
    negative_target_confirmation_exact_accuracy: Option<f64>,
    positive_target_confirmation_expected_cases: usize,
    positive_target_confirmation_invocations: usize,
    positive_target_confirmation_exact_matches: usize,
    positive_target_confirmation_exact_accuracy: Option<f64>,
    false_safe_negative_confirmations: usize,
    false_positive_target_local_confirmations: usize,
    materialized_exact_matches: usize,
    materialized_exact_accuracy: Option<f64>,
    correctness_wrong_target_relevance_retention: usize,
    false_relevance_rejections: usize,
    relevant_left_ambiguous: usize,
    utility_misses: usize,
    ambiguous_dispositions: usize,
    deterministic_safety_overrides: usize,
    lexical_exact_matches: usize,
    lexical_exact_accuracy: Option<f64>,
    lexical_wrong_target_relevance_retention: usize,
    lexical_expected_relevant_misses: usize,
    model_calls: u64,
    provider_attempts: u64,
    provider_attempts_incomplete_observations: usize,
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
    used_json_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
}

#[derive(Debug)]
struct NegativeConfirmationCallOutcome {
    confirmation: EvidenceNegativeTargetConfirmationV2,
    model_calls: u32,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
}

#[derive(Debug)]
struct PositiveConfirmationCallOutcome {
    confirmation: EvidencePositiveTargetConfirmation,
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
    used_json_fallback: bool,
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
        let expects_negative = expected.target_binding != EvidenceRelevanceBinding::Exact;
        let expects_positive = expected.target_binding == EvidenceRelevanceBinding::Exact
            && expected.relation_binding == EvidenceRelevanceBinding::Exact;

        if expects_negative != case.expected_negative_target_confirmation.is_some() {
            return Err(format!(
                "non-exact target case {} must freeze exactly one expected negative confirmation",
                case.id
            ));
        }
        if expects_positive != case.expected_positive_target_confirmation.is_some() {
            return Err(format!(
                "exact/exact case {} must freeze exactly one expected positive confirmation",
                case.id
            ));
        }
        if case.expected_negative_target_confirmation.is_some()
            && case.expected_positive_target_confirmation.is_some()
        {
            return Err(format!(
                "case {} must not carry both negative and positive confirmation expectations",
                case.id
            ));
        }

        let assessment = materialize_evidence_relevance_v5(
            &case.policy,
            &case.candidate,
            Some(&expected),
            case.expected_negative_target_confirmation,
            case.expected_positive_target_confirmation,
        )
        .map_err(|error| format!("invalid calibration policy {}: {error}", case.id))?;
        if assessment.disposition != case.expected_disposition {
            return Err(format!(
                "deterministic expected proposal mismatch for {}: expected {:?}, materialized {:?}",
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
            metrics: empty_metrics(selected.len()),
            observations: Vec::new(),
        });
    }

    if args.max_consecutive_operational_failures == 0 {
        return Err("--max-consecutive-operational-failures must be at least 1".into());
    }

    let generator = Generator::from_provider(args.provider, &args.model)?;
    let provider = args.provider.name().to_owned();
    let mut observations = Vec::with_capacity(selected.len());
    let mut consecutive_operational_failures = 0usize;
    let mut operational_abort: Option<OperationalAbort> = None;

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
        let case_seed = args.seed.and_then(|base| base.checked_add(index as u64));
        let request = build_evidence_relevance_binding_proposal_request(
            &case.policy,
            &case.candidate,
            case_seed,
        )
        .map_err(|error| format!("build relevance request {}: {error}", case.id))?;
        let lexical_baseline = simple_lexical_baseline(&case.policy, &case.candidate);

        let started = Instant::now();
        let result = call_model_for_proposal(
            generator.adapter(),
            request,
            case.policy.assessment_budget.max_model_attempts,
            Duration::from_millis(case.policy.assessment_budget.max_elapsed_ms),
        )
        .await;
        let observation = match result {
            Ok(call) => {
                complete_observed_case(
                    generator.adapter(),
                    case,
                    case_seed,
                    lexical_baseline,
                    started,
                    call,
                )
                .await?
            }
            Err(failure) => failure_observation(
                case,
                lexical_baseline,
                ObservationFailure {
                    latency_ms: started.elapsed().as_millis(),
                    used_json_fallback: failure.used_json_fallback,
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
            observation.used_json_fallback,
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

        consecutive_operational_failures = next_operational_failure_streak(
            consecutive_operational_failures,
            observations
                .last()
                .and_then(|observation| observation.failure_class.as_deref()),
        );

        if consecutive_operational_failures >= args.max_consecutive_operational_failures
            && index + 1 < selected.len()
        {
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
        metrics: summarize_metrics(&observations),
        observations,
    })
}

async fn complete_observed_case(
    adapter: &dyn ModelAdapter,
    case: &CalibrationCase,
    case_seed: Option<u64>,
    lexical_baseline: EvidenceRelevanceDisposition,
    started: Instant,
    call: CallOutcome,
) -> Result<CaseObservation, String> {
    let observed = call.proposal;
    let used_json_fallback = call.used_json_fallback;
    let mut model_calls = call.model_calls;
    let mut provider_attempts = call.provider_attempts;
    let mut usage = call.usage;
    let mut provider_model = call.provider_model;
    let mut finish_reason = call.finish_reason;
    let mut negative_confirmation = None;
    let mut positive_confirmation = None;

    let confirmation_kind = if observed.target_binding != EvidenceRelevanceBinding::Exact {
        Some("negative")
    } else if observed.relation_binding == EvidenceRelevanceBinding::Exact {
        Some("positive")
    } else {
        None
    };

    if let Some(kind) = confirmation_kind {
        let remaining = Duration::from_millis(case.policy.assessment_budget.max_elapsed_ms)
            .saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Ok(failure_observation(
                case,
                lexical_baseline,
                ObservationFailure {
                    latency_ms: started.elapsed().as_millis(),
                    used_json_fallback,
                    model_calls,
                    provider_attempts,
                    provider_attempts_complete: true,
                    usage,
                    provider_model,
                    finish_reason,
                    class: "assessment_timeout".into(),
                    message: format!("{kind} target confirmation had no remaining case budget"),
                },
            ));
        }

        if kind == "negative" {
            let request = build_evidence_negative_target_confirmation_v2_request(
                &case.policy,
                &case.candidate,
                case_seed.map(|seed| seed ^ 0x51d1_57c7),
            )
            .map_err(|error| {
                format!(
                    "build negative-target confirmation request {}: {error}",
                    case.id
                )
            })?;
            match call_model_for_negative_confirmation(adapter, request, remaining).await {
                Ok(confirmation_call) => {
                    negative_confirmation = Some(confirmation_call.confirmation);
                    model_calls = model_calls.saturating_add(confirmation_call.model_calls);
                    provider_attempts =
                        provider_attempts.saturating_add(confirmation_call.provider_attempts);
                    usage.add_summary(&confirmation_call.usage);
                    if confirmation_call.provider_model.is_some() {
                        provider_model = confirmation_call.provider_model;
                    }
                    if confirmation_call.finish_reason.is_some() {
                        finish_reason = confirmation_call.finish_reason;
                    }
                }
                Err(mut failure) => {
                    failure.used_json_fallback |= used_json_fallback;
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
                            used_json_fallback: failure.used_json_fallback,
                            model_calls: failure.model_calls,
                            provider_attempts: failure.provider_attempts,
                            provider_attempts_complete: failure.provider_attempts_complete,
                            usage: failure.usage,
                            provider_model: failure.provider_model,
                            finish_reason: failure.finish_reason,
                            class: failure.class,
                            message: format!(
                                "negative-target confirmation failed: {}",
                                failure.message
                            ),
                        },
                    ));
                }
            }
        } else {
            let request = build_evidence_positive_target_confirmation_request(
                &case.policy,
                &case.candidate,
                case_seed.map(|seed| seed ^ 0xa93c_2b41),
            )
            .map_err(|error| {
                format!(
                    "build positive-target confirmation request {}: {error}",
                    case.id
                )
            })?;
            match call_model_for_positive_confirmation(adapter, request, remaining).await {
                Ok(confirmation_call) => {
                    positive_confirmation = Some(confirmation_call.confirmation);
                    model_calls = model_calls.saturating_add(confirmation_call.model_calls);
                    provider_attempts =
                        provider_attempts.saturating_add(confirmation_call.provider_attempts);
                    usage.add_summary(&confirmation_call.usage);
                    if confirmation_call.provider_model.is_some() {
                        provider_model = confirmation_call.provider_model;
                    }
                    if confirmation_call.finish_reason.is_some() {
                        finish_reason = confirmation_call.finish_reason;
                    }
                }
                Err(mut failure) => {
                    failure.used_json_fallback |= used_json_fallback;
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
                            used_json_fallback: failure.used_json_fallback,
                            model_calls: failure.model_calls,
                            provider_attempts: failure.provider_attempts,
                            provider_attempts_complete: failure.provider_attempts_complete,
                            usage: failure.usage,
                            provider_model: failure.provider_model,
                            finish_reason: failure.finish_reason,
                            class: failure.class,
                            message: format!(
                                "positive-target confirmation failed: {}",
                                failure.message
                            ),
                        },
                    ));
                }
            }
        }
    }

    match materialize_evidence_relevance_v5(
        &case.policy,
        &case.candidate,
        Some(&observed),
        negative_confirmation,
        positive_confirmation,
    ) {
        Ok(assessment) => Ok(success_observation(
            case,
            lexical_baseline,
            observed,
            negative_confirmation,
            positive_confirmation,
            assessment,
            started.elapsed().as_millis(),
            used_json_fallback,
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
                used_json_fallback,
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
    used_json_fallback: bool,
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
    observed_negative_target_confirmation: Option<EvidenceNegativeTargetConfirmationV2>,
    observed_positive_target_confirmation: Option<EvidencePositiveTargetConfirmation>,
    assessment: EvidenceRelevanceAssessment,
    latency_ms: u128,
    used_json_fallback: bool,
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
        expected_negative_target_confirmation: case.expected_negative_target_confirmation,
        observed_negative_target_confirmation,
        negative_target_confirmation_match: case
            .expected_negative_target_confirmation
            .map(|expected| observed_negative_target_confirmation == Some(expected)),
        negative_target_confirmation_invoked: observed_negative_target_confirmation.is_some(),
        expected_positive_target_confirmation: case.expected_positive_target_confirmation,
        observed_positive_target_confirmation,
        positive_target_confirmation_match: case
            .expected_positive_target_confirmation
            .map(|expected| observed_positive_target_confirmation == Some(expected)),
        positive_target_confirmation_invoked: observed_positive_target_confirmation.is_some(),
        expected_disposition: case.expected_disposition,
        materialized_disposition: Some(materialized),
        disposition_match: Some(materialized == case.expected_disposition),
        lexical_baseline,
        lexical_baseline_match: lexical_baseline == case.expected_disposition,
        deterministic_safety_override: matches!(
            (observed.target_binding, observed.relation_binding),
            (
                EvidenceRelevanceBinding::Exact,
                EvidenceRelevanceBinding::Exact
            )
        ) && materialized != EvidenceRelevanceDisposition::Relevant,
        used_json_fallback,
        model_calls,
        provider_attempts,
        provider_attempts_complete: true,
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
    let negative_target_confirmation_invoked = failure
        .message
        .starts_with("negative-target confirmation failed:");
    let positive_target_confirmation_invoked = failure
        .message
        .starts_with("positive-target confirmation failed:");
    CaseObservation {
        id: case.id.clone(),
        family: case.family.clone(),
        expected_proposal: case.expected_proposal,
        observed_proposal: None,
        proposal_match: None,
        expected_negative_target_confirmation: case.expected_negative_target_confirmation,
        observed_negative_target_confirmation: None,
        negative_target_confirmation_match: None,
        negative_target_confirmation_invoked,
        expected_positive_target_confirmation: case.expected_positive_target_confirmation,
        observed_positive_target_confirmation: None,
        positive_target_confirmation_match: None,
        positive_target_confirmation_invoked,
        expected_disposition: case.expected_disposition,
        materialized_disposition: None,
        disposition_match: None,
        lexical_baseline,
        lexical_baseline_match: lexical_baseline == case.expected_disposition,
        deterministic_safety_override: false,
        used_json_fallback: failure.used_json_fallback,
        model_calls: failure.model_calls,
        provider_attempts: failure.provider_attempts,
        provider_attempts_complete: failure.provider_attempts_complete,
        latency_ms: failure.latency_ms,
        usage: failure.usage,
        provider_model: failure.provider_model,
        finish_reason: failure.finish_reason,
        assessment: None,
        failure_class: Some(failure.class),
        failure: Some(failure.message),
    }
}

async fn call_model_for_proposal(
    adapter: &dyn ModelAdapter,
    request: ModelRequest,
    max_model_calls: u32,
    max_elapsed: Duration,
) -> Result<CallOutcome, CallFailure> {
    let deadline = tokio::time::Instant::now() + max_elapsed;
    let mut model_calls = 0u32;
    let mut provider_attempts = 0u32;
    let mut usage = UsageSummary::default();
    let mut last_model = None;
    let mut last_finish_reason = None;

    if max_model_calls == 0 {
        return Err(CallFailure {
            class: "attempt_budget".into(),
            message: "model-call budget is zero".into(),
            used_json_fallback: false,
            model_calls,
            provider_attempts,
            provider_attempts_complete: true,
            usage,
            provider_model: last_model,
            finish_reason: last_finish_reason,
        });
    }

    model_calls += 1;
    let primary = tokio::time::timeout_at(deadline, adapter.generate(request.clone())).await;
    let primary = match primary {
        Ok(result) => result,
        Err(_) => {
            return Err(CallFailure {
                class: "assessment_timeout".into(),
                message: "evidence relevance assessment exceeded elapsed-time budget".into(),
                used_json_fallback: false,
                model_calls,
                provider_attempts,
                provider_attempts_complete: false,
                usage,
                provider_model: last_model,
                finish_reason: last_finish_reason,
            });
        }
    };

    match primary {
        Ok(response) => {
            provider_attempts = provider_attempts.saturating_add(response.provider_attempts);
            usage.add(&response.usage);
            last_model = Some(response.model.clone());
            last_finish_reason = response.finish_reason.clone();
            match parse_evidence_relevance_binding_proposal(&response.text) {
                Ok(proposal) => Ok(CallOutcome {
                    proposal,
                    used_json_fallback: false,
                    model_calls,
                    provider_attempts,
                    usage,
                    provider_model: last_model,
                    finish_reason: last_finish_reason,
                }),
                Err(primary_parse_error) => {
                    let Some(fallback) = build_json_object_fallback_request(&request) else {
                        return Err(CallFailure {
                            class: "protocol".into(),
                            message: format!(
                                "primary structured proposal parse failed and no fallback exists: {primary_parse_error}"
                            ),
                            used_json_fallback: false,
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
                        deadline,
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
        Err(error) if error.kind == ModelErrorKind::UnsupportedCapability => {
            provider_attempts = provider_attempts.saturating_add(error.provider_attempts);
            let Some(fallback) = build_json_object_fallback_request(&request) else {
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
            call_fallback(
                adapter,
                fallback,
                deadline,
                max_model_calls,
                model_calls,
                provider_attempts,
                usage,
                last_model,
                last_finish_reason,
                "primary JSON-Schema capability unsupported".into(),
            )
            .await
        }
        Err(error) => {
            provider_attempts = provider_attempts.saturating_add(error.provider_attempts);
            Err(model_failure(
                error,
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
    deadline: tokio::time::Instant,
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
                "{primary_context}; JSON-object fallback blocked by model-call budget"
            ),
            used_json_fallback: false,
            model_calls: prior_model_calls,
            provider_attempts: prior_provider_attempts,
            provider_attempts_complete: true,
            usage,
            provider_model: prior_model,
            finish_reason: prior_finish_reason,
        });
    }

    let model_calls = prior_model_calls + 1;
    let result = tokio::time::timeout_at(deadline, adapter.generate(fallback)).await;
    let result = match result {
        Ok(result) => result,
        Err(_) => {
            return Err(CallFailure {
                class: "assessment_timeout".into(),
                message: format!(
                    "{primary_context}; JSON-object fallback exceeded elapsed-time budget"
                ),
                used_json_fallback: true,
                model_calls,
                provider_attempts: prior_provider_attempts,
                provider_attempts_complete: false,
                usage,
                provider_model: prior_model,
                finish_reason: prior_finish_reason,
            });
        }
    };

    match result {
        Ok(response) => {
            let provider_attempts =
                prior_provider_attempts.saturating_add(response.provider_attempts);
            usage.add(&response.usage);
            let model = Some(response.model.clone());
            let finish_reason = response.finish_reason.clone();
            match parse_evidence_relevance_binding_proposal(&response.text) {
                Ok(proposal) => Ok(CallOutcome {
                    proposal,
                    used_json_fallback: true,
                    model_calls,
                    provider_attempts,
                    usage,
                    provider_model: model,
                    finish_reason,
                }),
                Err(error) => Err(CallFailure {
                    class: "protocol".into(),
                    message: format!(
                        "{primary_context}; JSON-object fallback proposal parse failed: {error}"
                    ),
                    used_json_fallback: true,
                    model_calls,
                    provider_attempts,
                    provider_attempts_complete: true,
                    usage,
                    provider_model: model,
                    finish_reason,
                }),
            }
        }
        Err(error) => {
            let provider_attempts = prior_provider_attempts.saturating_add(error.provider_attempts);
            let mut failure = model_failure(
                error,
                true,
                model_calls,
                provider_attempts,
                usage,
                prior_model,
                prior_finish_reason,
            );
            failure.message = format!("{primary_context}; fallback failed: {}", failure.message);
            Err(failure)
        }
    }
}

async fn call_model_for_negative_confirmation(
    adapter: &dyn ModelAdapter,
    request: ModelRequest,
    max_elapsed: Duration,
) -> Result<NegativeConfirmationCallOutcome, CallFailure> {
    if CONFIRMATION_STAGE_MAX_MODEL_CALLS == 0 {
        return Err(CallFailure {
            class: "attempt_budget".into(),
            message: "negative-target confirmation model-call budget is zero".into(),
            used_json_fallback: false,
            model_calls: 0,
            provider_attempts: 0,
            provider_attempts_complete: true,
            usage: UsageSummary::default(),
            provider_model: None,
            finish_reason: None,
        });
    }
    let result = tokio::time::timeout(max_elapsed, adapter.generate(request)).await;
    let response = match result {
        Err(_) => {
            return Err(CallFailure {
                class: "assessment_timeout".into(),
                message: "negative-target confirmation exceeded remaining case budget".into(),
                used_json_fallback: false,
                model_calls: 1,
                provider_attempts: 0,
                provider_attempts_complete: false,
                usage: UsageSummary::default(),
                provider_model: None,
                finish_reason: None,
            });
        }
        Ok(Err(error)) => {
            let provider_attempts = error.provider_attempts;
            return Err(model_failure(
                error,
                false,
                1,
                provider_attempts,
                UsageSummary::default(),
                None,
                None,
            ));
        }
        Ok(Ok(response)) => response,
    };
    let mut usage = UsageSummary::default();
    usage.add(&response.usage);
    let confirmation =
        parse_evidence_negative_target_confirmation_v2(&response.text).map_err(|error| {
            CallFailure {
                class: "protocol".into(),
                message: format!("negative-target enum confirmation parse failed: {error}"),
                used_json_fallback: false,
                model_calls: 1,
                provider_attempts: response.provider_attempts,
                provider_attempts_complete: true,
                usage: usage.clone(),
                provider_model: Some(response.model.clone()),
                finish_reason: response.finish_reason.clone(),
            }
        })?;
    Ok(NegativeConfirmationCallOutcome {
        confirmation,
        model_calls: 1,
        provider_attempts: response.provider_attempts,
        usage,
        provider_model: Some(response.model),
        finish_reason: response.finish_reason,
    })
}

async fn call_model_for_positive_confirmation(
    adapter: &dyn ModelAdapter,
    request: ModelRequest,
    max_elapsed: Duration,
) -> Result<PositiveConfirmationCallOutcome, CallFailure> {
    if CONFIRMATION_STAGE_MAX_MODEL_CALLS == 0 {
        return Err(CallFailure {
            class: "attempt_budget".into(),
            message: "positive-target confirmation model-call budget is zero".into(),
            used_json_fallback: false,
            model_calls: 0,
            provider_attempts: 0,
            provider_attempts_complete: true,
            usage: UsageSummary::default(),
            provider_model: None,
            finish_reason: None,
        });
    }
    let result = tokio::time::timeout(max_elapsed, adapter.generate(request)).await;
    let response = match result {
        Err(_) => {
            return Err(CallFailure {
                class: "assessment_timeout".into(),
                message: "positive-target confirmation exceeded remaining case budget".into(),
                used_json_fallback: false,
                model_calls: 1,
                provider_attempts: 0,
                provider_attempts_complete: false,
                usage: UsageSummary::default(),
                provider_model: None,
                finish_reason: None,
            });
        }
        Ok(Err(error)) => {
            let provider_attempts = error.provider_attempts;
            return Err(model_failure(
                error,
                false,
                1,
                provider_attempts,
                UsageSummary::default(),
                None,
                None,
            ));
        }
        Ok(Ok(response)) => response,
    };
    let mut usage = UsageSummary::default();
    usage.add(&response.usage);
    let confirmation =
        parse_evidence_positive_target_confirmation(&response.text).map_err(|error| {
            CallFailure {
                class: "protocol".into(),
                message: format!("positive-target enum confirmation parse failed: {error}"),
                used_json_fallback: false,
                model_calls: 1,
                provider_attempts: response.provider_attempts,
                provider_attempts_complete: true,
                usage: usage.clone(),
                provider_model: Some(response.model.clone()),
                finish_reason: response.finish_reason.clone(),
            }
        })?;
    Ok(PositiveConfirmationCallOutcome {
        confirmation,
        model_calls: 1,
        provider_attempts: response.provider_attempts,
        usage,
        provider_model: Some(response.model),
        finish_reason: response.finish_reason,
    })
}

fn model_failure(
    error: ModelError,
    used_json_fallback: bool,
    model_calls: u32,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
) -> CallFailure {
    CallFailure {
        class: model_error_class(error.kind).into(),
        message: error.message,
        used_json_fallback,
        model_calls,
        provider_attempts,
        provider_attempts_complete: true,
        usage,
        provider_model,
        finish_reason,
    }
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
    let negative_target_confirmation_expected_cases = observations
        .iter()
        .filter(|case| case.expected_negative_target_confirmation.is_some())
        .count();
    let negative_target_confirmation_invocations = observations
        .iter()
        .filter(|case| case.negative_target_confirmation_invoked)
        .count();
    let negative_target_confirmation_exact_matches = observations
        .iter()
        .filter(|case| case.negative_target_confirmation_match == Some(true))
        .count();
    let positive_target_confirmation_expected_cases = observations
        .iter()
        .filter(|case| case.expected_positive_target_confirmation.is_some())
        .count();
    let positive_target_confirmation_invocations = observations
        .iter()
        .filter(|case| case.positive_target_confirmation_invoked)
        .count();
    let positive_target_confirmation_exact_matches = observations
        .iter()
        .filter(|case| case.positive_target_confirmation_match == Some(true))
        .count();
    let false_safe_negative_confirmations = observations
        .iter()
        .filter(|case| {
            let observed_safe = matches!(
                case.observed_negative_target_confirmation,
                Some(EvidenceNegativeTargetConfirmationV2::ConfirmedDistinctEntity)
                    | Some(EvidenceNegativeTargetConfirmationV2::ConfirmedLocalTargetAbsent)
            );
            let expected_safe = matches!(
                case.expected_negative_target_confirmation,
                Some(EvidenceNegativeTargetConfirmationV2::ConfirmedDistinctEntity)
                    | Some(EvidenceNegativeTargetConfirmationV2::ConfirmedLocalTargetAbsent)
            );
            observed_safe && !expected_safe
        })
        .count();
    let false_positive_target_local_confirmations = observations
        .iter()
        .filter(|case| {
            case.observed_positive_target_confirmation
                == Some(EvidencePositiveTargetConfirmation::ConfirmedTargetLocalBinding)
                && case.expected_positive_target_confirmation
                    != Some(EvidencePositiveTargetConfirmation::ConfirmedTargetLocalBinding)
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
    let deterministic_safety_overrides = observations
        .iter()
        .filter(|case| case.deterministic_safety_override)
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
        negative_target_confirmation_expected_cases,
        negative_target_confirmation_invocations,
        negative_target_confirmation_exact_matches,
        negative_target_confirmation_exact_accuracy: accuracy(
            negative_target_confirmation_exact_matches,
            negative_target_confirmation_expected_cases,
        ),
        positive_target_confirmation_expected_cases,
        positive_target_confirmation_invocations,
        positive_target_confirmation_exact_matches,
        positive_target_confirmation_exact_accuracy: accuracy(
            positive_target_confirmation_exact_matches,
            positive_target_confirmation_expected_cases,
        ),
        false_safe_negative_confirmations,
        false_positive_target_local_confirmations,
        materialized_exact_matches,
        materialized_exact_accuracy: accuracy(materialized_exact_matches, successful),
        correctness_wrong_target_relevance_retention,
        false_relevance_rejections,
        relevant_left_ambiguous,
        utility_misses,
        ambiguous_dispositions,
        deterministic_safety_overrides,
        lexical_exact_matches,
        lexical_exact_accuracy: accuracy(lexical_exact_matches, observations.len()),
        lexical_wrong_target_relevance_retention,
        lexical_expected_relevant_misses,
        model_calls,
        provider_attempts,
        provider_attempts_incomplete_observations,
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
        negative_target_confirmation_expected_cases: 0,
        negative_target_confirmation_invocations: 0,
        negative_target_confirmation_exact_matches: 0,
        negative_target_confirmation_exact_accuracy: None,
        positive_target_confirmation_expected_cases: 0,
        positive_target_confirmation_invocations: 0,
        positive_target_confirmation_exact_matches: 0,
        positive_target_confirmation_exact_accuracy: None,
        false_safe_negative_confirmations: 0,
        false_positive_target_local_confirmations: 0,
        materialized_exact_matches: 0,
        materialized_exact_accuracy: None,
        correctness_wrong_target_relevance_retention: 0,
        false_relevance_rejections: 0,
        relevant_left_ambiguous: 0,
        utility_misses: 0,
        ambiguous_dispositions: 0,
        deterministic_safety_overrides: 0,
        lexical_exact_matches: 0,
        lexical_exact_accuracy: None,
        lexical_wrong_target_relevance_retention: 0,
        lexical_expected_relevant_misses: 0,
        model_calls: 0,
        provider_attempts: 0,
        provider_attempts_incomplete_observations: 0,
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
    fn expected_proposals_materialize_all_v9_cases() {
        let manifest = load();
        assert_eq!(manifest.cases.len(), EXPECTED_CASES);
        for case in manifest.cases {
            let proposal = case.expected_proposal;
            let assessment = materialize_evidence_relevance_v5(
                &case.policy,
                &case.candidate,
                Some(&proposal),
                case.expected_negative_target_confirmation,
                case.expected_positive_target_confirmation,
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
        build_evidence_relevance_binding_proposal_request(&case.policy, &case.candidate, Some(462))
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
        let failure =
            call_model_for_proposal(&adapter, fixture_request(), 1, Duration::from_secs(1))
                .await
                .unwrap_err();
        assert_eq!(failure.class, "attempt_budget");
        assert_eq!(failure.model_calls, 1);
        assert!(!failure.used_json_fallback);
    }

    #[tokio::test]
    async fn assessment_elapsed_budget_fails_typed_and_closed() {
        let adapter = SequenceAdapter::new(
            vec![model_response(
                r#"{"target_binding":"exact","relation_binding":"exact"}"#,
            )],
            50,
        );
        let failure =
            call_model_for_proposal(&adapter, fixture_request(), 2, Duration::from_millis(5))
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
    fn latency_percentiles_use_nearest_rank() {
        let values = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100];
        assert_eq!(percentile_latency(&values, 50), Some(50));
        assert_eq!(percentile_latency(&values, 95), Some(100));
        assert_eq!(percentile_latency(&[42], 95), Some(42));
        assert_eq!(percentile_latency(&[], 95), None);
    }

    fn negative_confirmation_request() -> ModelRequest {
        let manifest = load();
        let case = manifest
            .cases
            .iter()
            .find(|case| case.expected_negative_target_confirmation.is_some())
            .expect("negative target case");
        build_evidence_negative_target_confirmation_v2_request(
            &case.policy,
            &case.candidate,
            Some(463),
        )
        .expect("negative confirmation request")
    }

    #[tokio::test]
    async fn negative_confirmation_parser_returns_typed_state() {
        let adapter = SequenceAdapter::new(vec![model_response("confirmed_distinct_entity")], 0);
        let result = call_model_for_negative_confirmation(
            &adapter,
            negative_confirmation_request(),
            Duration::from_secs(1),
        )
        .await
        .expect("confirmation result");
        assert_eq!(
            result.confirmation,
            EvidenceNegativeTargetConfirmationV2::ConfirmedDistinctEntity
        );
        assert_eq!(result.model_calls, 1);
        assert_eq!(result.provider_attempts, 1);
    }

    #[tokio::test]
    async fn negative_confirmation_timeout_is_operational_failure() {
        let adapter = SequenceAdapter::new(vec![model_response("not_confirmed")], 50);
        let failure = call_model_for_negative_confirmation(
            &adapter,
            negative_confirmation_request(),
            Duration::from_millis(5),
        )
        .await
        .unwrap_err();
        assert_eq!(failure.class, "assessment_timeout");
        assert!(!failure.provider_attempts_complete);
    }

    fn positive_confirmation_request() -> ModelRequest {
        let manifest = load();
        let case = manifest
            .cases
            .iter()
            .find(|case| case.expected_positive_target_confirmation.is_some())
            .expect("positive target case");
        build_evidence_positive_target_confirmation_request(
            &case.policy,
            &case.candidate,
            Some(464),
        )
        .expect("positive confirmation request")
    }

    #[tokio::test]
    async fn positive_confirmation_parser_returns_typed_state() {
        let adapter =
            SequenceAdapter::new(vec![model_response("confirmed_target_local_binding")], 0);
        let result = call_model_for_positive_confirmation(
            &adapter,
            positive_confirmation_request(),
            Duration::from_secs(1),
        )
        .await
        .expect("positive confirmation result");
        assert_eq!(
            result.confirmation,
            EvidencePositiveTargetConfirmation::ConfirmedTargetLocalBinding
        );
        assert_eq!(result.model_calls, 1);
        assert_eq!(result.provider_attempts, 1);
    }

    #[tokio::test]
    async fn confirmation_text_parser_does_not_retry_or_repair_malformed_output() {
        let adapter = SequenceAdapter::new(
            vec![model_response(
                "confirmed_target_local_binding because it matches",
            )],
            0,
        );
        let failure = call_model_for_positive_confirmation(
            &adapter,
            positive_confirmation_request(),
            Duration::from_secs(1),
        )
        .await
        .unwrap_err();
        assert_eq!(failure.class, "protocol");
        assert_eq!(failure.model_calls, 1);
        assert_eq!(failure.provider_attempts, 1);
        assert!(!failure.used_json_fallback);
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
        let result =
            call_model_for_proposal(&adapter, fixture_request(), 2, Duration::from_secs(1))
                .await
                .expect("fallback result");
        assert_eq!(result.model_calls, 2);
        assert_eq!(result.provider_attempts, 2);
        assert!(result.used_json_fallback);
    }
}
