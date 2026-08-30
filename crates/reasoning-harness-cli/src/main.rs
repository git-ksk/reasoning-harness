use std::{
    collections::{BTreeMap, VecDeque},
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::Arc,
    time::Instant,
};

use clap::{Parser, Subcommand, ValueEnum};
use reasoning_harness_core::{
    AdversarialDiscoveryPass, AssumptionDiscoveryPass, BenchmarkAggregate, BenchmarkCaseResult,
    BenchmarkComparison, BenchmarkFixture, ClaimCorpusSummary, CorpusManifest,
    DiagnosticObservation, DiagnosticTrial, EvidenceQualificationPass, HarnessInput, ModelAdapter,
    ModelError, ModelErrorKind, ModelUsage, ReasoningArtifact, ReasoningCandidate,
    RepeatedDiagnosticReport, StrictAcceptancePolicy, StructuredFactConflictDetector,
    TrustedVerificationPass, VerificationPass, VerificationReceipt, aggregate_benchmark,
    aggregate_claim_corpus, aggregate_repeated_diagnostics, build_candidate_json_fallback_request,
    build_candidate_request, evaluate, evaluate_benchmark_fixture_with_diagnostics,
    frameworks::five_whys::FiveWhysRestatementPass, run_harness,
    structured_fact_verifier_for_input, validate_artifact,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter, NvidiaAdapter};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Debug, Parser)]
#[command(
    name = "reason",
    version,
    about = "Native reasoning correctness harness CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum OutputFormat {
    #[default]
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Mistral,
    Google,
    Nvidia,
    #[value(hide = true)]
    Gemma,
}

enum LiveGenerator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
    Nvidia(NvidiaAdapter),
}

impl LiveGenerator {
    fn from_provider(provider: Provider, model: &str) -> Result<Self, String> {
        match provider {
            Provider::Mistral => MistralAdapter::from_env(model)
                .map(Self::Mistral)
                .map_err(|error| error.to_string()),
            Provider::Google | Provider::Gemma => GoogleAdapter::from_env(model)
                .map(Self::Google)
                .map_err(|error| error.to_string()),
            Provider::Nvidia => NvidiaAdapter::from_env(model)
                .map(Self::Nvidia)
                .map_err(|error| error.to_string()),
        }
    }

    async fn generate(
        &self,
        input: &HarnessInput,
        max_tokens: u32,
        seed: Option<u64>,
        requested_model: &str,
    ) -> Result<(ReasoningCandidate, GenerationObservation), GenerationFailure> {
        match self {
            Self::Mistral(adapter) => {
                generate_with_adapter(adapter, "mistral", requested_model, input, max_tokens, seed)
                    .await
            }
            Self::Google(adapter) => {
                generate_with_adapter(adapter, "google", requested_model, input, max_tokens, seed)
                    .await
            }
            Self::Nvidia(adapter) => {
                generate_with_adapter(adapter, "nvidia", requested_model, input, max_tokens, seed)
                    .await
            }
        }
    }
}

async fn generate_with_adapter<A: ModelAdapter>(
    adapter: &A,
    provider: &'static str,
    requested_model: &str,
    input: &HarnessInput,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(ReasoningCandidate, GenerationObservation), GenerationFailure> {
    let started = Instant::now();
    let request = build_candidate_request(input, Some(max_tokens), seed).map_err(|error| {
        generation_failure(
            provider,
            requested_model,
            started,
            ModelError::new(ModelErrorKind::Protocol, error.to_string()),
        )
    })?;
    let first = adapter
        .generate(request)
        .await
        .map_err(|error| generation_failure(provider, requested_model, started, error))?;
    let (candidate, response, provider_attempts, usage) = match parse_candidate_json(&first.text) {
        Ok((candidate, ignored_trailing_text)) => {
            if ignored_trailing_text {
                eprintln!(
                    "{provider} candidate normalization: ignored non-JSON trailing text after one complete candidate object"
                );
            }
            let usage = first.usage.clone();
            (candidate, first, 1, usage)
        }
        Err(first_error) => {
            let fallback = build_candidate_json_fallback_request(input, Some(max_tokens), seed)
                .map_err(|error| {
                    generation_failure(
                        provider,
                        requested_model,
                        started,
                        ModelError::new(ModelErrorKind::Protocol, error.to_string()),
                    )
                })?;
            let second = adapter.generate(fallback).await.map_err(|error| {
                let kind = error.kind;
                generation_failure(
                    provider,
                    requested_model,
                    started,
                    ModelError::new(
                        kind,
                        format!(
                            "{provider} structured-output fallback failed after invalid first candidate (finish_reason={}, bytes={}): {error}",
                            first.finish_reason.as_deref().unwrap_or("unknown"),
                            first.text.len(),
                        ),
                    ),
                )
            })?;
            let (candidate, ignored_trailing_text) = parse_candidate_json(&second.text)
                .map_err(|second_error| {
                    generation_failure(
                        provider,
                        requested_model,
                        started,
                        ModelError::new(
                            ModelErrorKind::Protocol,
                            format!(
                                "provider returned invalid candidate JSON after structured-output fallback: first_error={first_error}; first_finish_reason={}; first_bytes={}; second_error={second_error}; second_finish_reason={}; second_bytes={}",
                                first.finish_reason.as_deref().unwrap_or("unknown"),
                                first.text.len(),
                                second.finish_reason.as_deref().unwrap_or("unknown"),
                                second.text.len(),
                            ),
                        ),
                    )
                })?;
            if ignored_trailing_text {
                eprintln!(
                    "{provider} fallback candidate normalization: ignored non-JSON trailing text after one complete candidate object"
                );
            }
            let usage = add_usage(&first.usage, &second.usage);
            (candidate, second, 2, usage)
        }
    };
    let latency_ms = started.elapsed().as_millis();
    Ok((
        candidate,
        GenerationObservation {
            provider,
            model: response.model,
            usage,
            latency_ms,
            provider_attempts,
            cost_usd: None,
        },
    ))
}

#[derive(Debug, Clone, Serialize)]
struct GenerationFailure {
    provider: &'static str,
    model: String,
    latency_ms: u128,
    failure_class: &'static str,
    message: String,
}

fn generation_failure(
    provider: &'static str,
    requested_model: &str,
    started: Instant,
    error: ModelError,
) -> GenerationFailure {
    GenerationFailure {
        provider,
        model: requested_model.to_string(),
        latency_ms: started.elapsed().as_millis(),
        failure_class: model_error_class(error.kind),
        message: error.to_string(),
    }
}

fn model_error_class(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Credentials => "credentials",
        ModelErrorKind::Transport => "transport",
        ModelErrorKind::Provider => "provider_error",
        ModelErrorKind::RateLimit => "rate_limit",
        ModelErrorKind::Quota => "quota",
        ModelErrorKind::ProviderUnavailable => "provider_unavailable",
        ModelErrorKind::Timeout => "timeout",
        ModelErrorKind::Protocol => "protocol",
        ModelErrorKind::UnsupportedCapability => "unsupported_capability",
    }
}

fn format_generation_failure(failure: &GenerationFailure) -> String {
    format!(
        "provider={} model={} failure_class={} latency_ms={}: {}",
        failure.provider, failure.model, failure.failure_class, failure.latency_ms, failure.message
    )
}

fn parse_candidate_json(text: &str) -> Result<(ReasoningCandidate, bool), serde_json::Error> {
    match serde_json::from_str::<ReasoningCandidate>(text) {
        Ok(candidate) => Ok((candidate, false)),
        Err(strict_error) => {
            let mut stream =
                serde_json::Deserializer::from_str(text).into_iter::<ReasoningCandidate>();
            let Some(Ok(candidate)) = stream.next() else {
                return Err(strict_error);
            };
            let remainder = &text[stream.byte_offset()..];
            let mut trailing_values =
                serde_json::Deserializer::from_str(remainder).into_iter::<serde_json::Value>();
            match trailing_values.next() {
                Some(Ok(_)) => Err(strict_error),
                Some(Err(_)) => Ok((candidate, true)),
                None => Err(strict_error),
            }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate or load a candidate, then execute the harness-owned correctness process.
    Run {
        /// Harness-owned task and evidence JSON.
        #[arg(long)]
        input: PathBuf,
        /// Offline candidate JSON. Mutually exclusive with --provider.
        #[arg(long)]
        candidate: Option<PathBuf>,
        /// Trusted verification receipts JSON array. Never sent to the model.
        #[arg(long)]
        receipts: Option<PathBuf>,
        /// Live candidate generator. Mutually exclusive with --candidate.
        #[arg(long, value_enum)]
        provider: Option<Provider>,
        /// Provider model identifier used for live candidate generation.
        #[arg(long, default_value = "ministral-8b-latest")]
        model: String,
        /// Maximum candidate-generation tokens.
        #[arg(long, default_value_t = 1024)]
        max_tokens: u32,
        /// Optional provider random seed for repeatable research runs.
        #[arg(long)]
        seed: Option<u64>,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Deterministically validate a finalized ReasoningArtifact JSON file.
    Verify {
        artifact: PathBuf,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Evaluate one artifact or a directory of benchmark fixtures.
    Eval {
        target: PathBuf,
        /// Optional live provider. Without this, fixture suites use recorded candidates.
        #[arg(long, value_enum)]
        provider: Option<Provider>,
        #[arg(long, default_value = "ministral-8b-latest")]
        model: String,
        #[arg(long, default_value_t = 1024)]
        max_tokens: u32,
        /// Base seed. Trial N uses base_seed + N for reproducible multi-trial runs.
        #[arg(long)]
        seed: Option<u64>,
        /// Number of live generations per fixture.
        #[arg(long, default_value_t = 1)]
        trials: usize,
        /// Maximum number of live fixture generations in flight.
        #[arg(long, default_value_t = 1)]
        concurrency: usize,
        /// Optional input-token price in USD per million tokens.
        #[arg(long)]
        input_cost_per_million: Option<f64>,
        /// Optional output-token price in USD per million tokens.
        #[arg(long)]
        output_cost_per_million: Option<f64>,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}

#[derive(Debug, Serialize)]
struct VerifyOutput<'a> {
    valid: bool,
    diagnostics: &'a [reasoning_harness_core::Diagnostic],
}

#[derive(Debug, Clone, Serialize)]
struct GenerationObservation {
    provider: &'static str,
    model: String,
    usage: ModelUsage,
    latency_ms: u128,
    provider_attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_usd: Option<f64>,
}

fn add_usage(left: &ModelUsage, right: &ModelUsage) -> ModelUsage {
    ModelUsage {
        input_tokens: add_optional(left.input_tokens, right.input_tokens),
        output_tokens: add_optional(left.output_tokens, right.output_tokens),
        total_tokens: add_optional(left.total_tokens, right.total_tokens),
    }
}

fn add_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => left.checked_add(right),
        _ => None,
    }
}

#[derive(Debug, Serialize)]
struct RunOutput {
    candidate: ReasoningCandidate,
    outcome: reasoning_harness_core::HarnessOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation: Option<GenerationObservation>,
}

#[derive(Debug, Clone, Serialize)]
struct ObservedBenchmarkCase {
    fixture_id: String,
    trial: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<BenchmarkCaseResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<DiagnosticObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation: Option<GenerationObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<GenerationFailure>,
}

#[derive(Debug, Serialize)]
struct OperationalSummary {
    attempted_runs: usize,
    generated_runs: usize,
    failed_runs: usize,
    failure_classes: BTreeMap<&'static str, usize>,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    total_latency_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    total_cost_usd: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ScalarDistribution {
    count: usize,
    mean: f64,
    min: f64,
    max: f64,
    stddev: f64,
}

#[derive(Debug, Serialize)]
struct BenchmarkArmStability {
    verdict_accuracy: ScalarDistribution,
    accept_recall: ScalarDistribution,
    reject_recall: ScalarDistribution,
    unknown_recall: ScalarDistribution,
    unsafe_accept_cases: ScalarDistribution,
    deterministic_verifier_failure_rate: ScalarDistribution,
    contradiction_detection_rate: ScalarDistribution,
    counterexample_detection_rate: ScalarDistribution,
}

#[derive(Debug, Serialize)]
struct BenchmarkStabilityComparison {
    baseline: BenchmarkArmStability,
    harness: BenchmarkArmStability,
}

#[derive(Debug, Serialize)]
struct TrialSummary {
    trial_index: usize,
    expected_runs: usize,
    correctness_cases: usize,
    operationally_complete: bool,
    operational: OperationalSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    comparison: Option<BenchmarkComparison>,
}

#[derive(Debug, Serialize)]
struct OperationalDistributions {
    #[serde(skip_serializing_if = "Option::is_none")]
    successful_request_total_tokens: Option<ScalarDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    successful_request_latency_ms: Option<ScalarDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    complete_trial_total_tokens: Option<ScalarDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    complete_trial_total_latency_ms: Option<ScalarDistribution>,
}

#[derive(Debug, Serialize)]
struct TrialStabilitySummary {
    requested_trials: usize,
    complete_trials: usize,
    incomplete_trials: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    correctness: Option<BenchmarkStabilityComparison>,
    diagnostics: RepeatedDiagnosticReport,
    operational: OperationalDistributions,
    per_trial: Vec<TrialSummary>,
}

#[derive(Debug, Serialize)]
struct BenchmarkCorpusOutput {
    corpus_version: String,
    score_compatibility_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stratification: Option<ClaimCorpusSummary>,
}

#[derive(Debug, Serialize)]
struct BenchmarkOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    corpus: Option<BenchmarkCorpusOutput>,
    comparison: BenchmarkComparison,
    operational: OperationalSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    stability: Option<TrialStabilitySummary>,
    cases: Vec<ObservedBenchmarkCase>,
}

#[derive(Debug, Clone)]
struct BenchmarkRunConfig<'a> {
    provider: Option<Provider>,
    model: &'a str,
    max_tokens: u32,
    seed: Option<u64>,
    trials: usize,
    concurrency: usize,
    input_cost_per_million: Option<f64>,
    output_cost_per_million: Option<f64>,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Run {
            input,
            candidate,
            receipts,
            provider,
            model,
            max_tokens,
            seed,
            format,
        } => {
            let input: HarnessInput = read_json(&input)?;
            let (candidate, generation) = match (candidate, provider) {
                (Some(path), None) => (read_json(&path)?, None),
                (None, Some(provider)) => {
                    let generator = LiveGenerator::from_provider(provider, &model)?;
                    let (candidate, observation) = generator
                        .generate(&input, max_tokens, seed, &model)
                        .await
                        .map_err(|failure| format_generation_failure(&failure))?;
                    (candidate, Some(observation))
                }
                (Some(_), Some(_)) => {
                    return Err("choose either --candidate or --provider, not both".into());
                }
                (None, None) => {
                    return Err("reason run requires either --candidate or --provider".into());
                }
            };

            let receipts: Vec<VerificationReceipt> = match receipts {
                Some(path) => read_json(&path)?,
                None => Vec::new(),
            };
            let structured_verifier = structured_fact_verifier_for_input(&input);
            let passes: Vec<Box<dyn reasoning_harness_core::Pass>> = vec![
                Box::new(AdversarialDiscoveryPass::new(vec![Box::new(
                    StructuredFactConflictDetector,
                )])),
                Box::new(EvidenceQualificationPass),
                Box::new(VerificationPass::new(vec![structured_verifier])),
                Box::new(TrustedVerificationPass::new(receipts)),
                Box::new(FiveWhysRestatementPass),
                Box::new(AssumptionDiscoveryPass),
            ];
            let outcome = run_harness(input, candidate.clone(), &passes, &StrictAcceptancePolicy)
                .map_err(|error| error.to_string())?;
            let output = RunOutput {
                candidate,
                outcome,
                generation,
            };
            match format {
                OutputFormat::Human => {
                    println!("verdict: {:?}", output.outcome.verdict);
                    if let Some(generation) = &output.generation {
                        println!(
                            "generation: provider={} model={} latency_ms={}",
                            generation.provider, generation.model, generation.latency_ms
                        );
                    }
                }
                OutputFormat::Json => print_json(&output)?,
            }
            Ok(())
        }
        Command::Verify { artifact, format } => {
            let artifact: ReasoningArtifact = read_json(&artifact)?;
            let report = validate_artifact(&artifact);
            match format {
                OutputFormat::Human if report.is_ok() => println!("valid"),
                OutputFormat::Human => {
                    for diagnostic in &report.diagnostics {
                        eprintln!("{}: {}", diagnostic.code, diagnostic.message);
                    }
                }
                OutputFormat::Json => print_json(&VerifyOutput {
                    valid: report.is_ok(),
                    diagnostics: &report.diagnostics,
                })?,
            }
            if report.is_ok() {
                Ok(())
            } else {
                Err("artifact validation failed".into())
            }
        }
        Command::Eval {
            target,
            provider,
            model,
            max_tokens,
            seed,
            trials,
            concurrency,
            input_cost_per_million,
            output_cost_per_million,
            format,
        } => {
            if target.is_dir() {
                let config = BenchmarkRunConfig {
                    provider,
                    model: &model,
                    max_tokens,
                    seed,
                    trials,
                    concurrency,
                    input_cost_per_million,
                    output_cost_per_million,
                };
                let output = run_fixture_suite(&target, &config).await?;
                match format {
                    OutputFormat::Human => print_benchmark_human(&output),
                    OutputFormat::Json => print_json(&output)?,
                }
                Ok(())
            } else {
                if provider.is_some() || trials != 1 || concurrency != 1 {
                    return Err(
                        "live provider and multi-trial options require a fixture directory".into(),
                    );
                }
                let artifact: ReasoningArtifact = read_json(&target)?;
                let metrics = evaluate(&artifact);
                match format {
                    OutputFormat::Human => println!(
                        "valid={} evidence_coverage={:.3} unknown_rate={:.3} accepted_without_evidence={}",
                        metrics.valid,
                        metrics.evidence_coverage,
                        metrics.explicit_unknown_rate,
                        metrics.accepted_without_evidence
                    ),
                    OutputFormat::Json => print_json(&metrics)?,
                }
                if metrics.valid {
                    Ok(())
                } else {
                    Err("artifact validation failed during eval".into())
                }
            }
        }
    }
}

async fn run_fixture_suite(
    directory: &PathBuf,
    config: &BenchmarkRunConfig<'_>,
) -> Result<BenchmarkOutput, String> {
    if config.trials == 0 {
        return Err("--trials must be at least 1".into());
    }
    if !(1..=10).contains(&config.concurrency) {
        return Err("--concurrency must be between 1 and 10".into());
    }
    if config.provider.is_none() && config.trials != 1 {
        return Err("offline recorded fixtures support exactly one trial".into());
    }
    if config.provider.is_none() && config.concurrency != 1 {
        return Err("--concurrency requires a live provider".into());
    }
    if config.input_cost_per_million.is_some() ^ config.output_cost_per_million.is_some() {
        return Err("provide both input and output token prices, or neither".into());
    }
    for rate in [
        config.input_cost_per_million,
        config.output_cost_per_million,
    ]
    .into_iter()
    .flatten()
    {
        if !rate.is_finite() || rate < 0.0 {
            return Err("token prices must be finite non-negative values".into());
        }
    }

    let fixtures = load_fixtures(directory)?;
    let total_runs = fixtures
        .len()
        .checked_mul(config.trials)
        .ok_or("fixture/trial count overflowed usize")?;

    let observed = if let Some(provider) = config.provider {
        let generator = Arc::new(LiveGenerator::from_provider(provider, config.model)?);
        let mut completed_runs = 0usize;
        let mut started_runs = 0usize;
        let mut ordered: Vec<Option<ObservedBenchmarkCase>> =
            (0..total_runs).map(|_| None).collect();

        // A trial is one full-corpus pass. Keep concurrency inside that pass so
        // provider timing/failures from a later trial do not overlap an earlier trial.
        // `sequence` preserves the historical fixture-major/trial-minor JSON order.
        for trial in 0..config.trials {
            let mut pending = VecDeque::new();
            for (fixture_index, fixture) in fixtures.iter().cloned().enumerate() {
                let sequence = fixture_index
                    .checked_mul(config.trials)
                    .and_then(|value| value.checked_add(trial))
                    .ok_or("fixture/trial sequence overflowed usize")?;
                pending.push_back((sequence, fixture));
            }
            let mut tasks = tokio::task::JoinSet::new();

            loop {
                while tasks.len() < config.concurrency {
                    let Some((sequence, fixture)) = pending.pop_front() else {
                        break;
                    };
                    let generator = Arc::clone(&generator);
                    let model = config.model.to_string();
                    let max_tokens = config.max_tokens;
                    let input_cost = config.input_cost_per_million;
                    let output_cost = config.output_cost_per_million;
                    let trial_seed = match config.seed {
                        Some(value) => Some(
                            value
                                .checked_add(trial as u64)
                                .ok_or("trial seed overflowed u64")?,
                        ),
                        None => None,
                    };
                    started_runs += 1;
                    eprintln!(
                        "[benchmark] provider={} model={} start={}/{} fixture={} trial={} in_flight={}",
                        provider_name(provider),
                        model,
                        started_runs,
                        total_runs,
                        fixture.id,
                        trial + 1,
                        tasks.len() + 1
                    );
                    tasks.spawn(async move {
                        let generated = generator
                            .generate(&fixture.input, max_tokens, trial_seed, &model)
                            .await;
                        let case = match generated {
                            Ok((candidate, mut observation)) => {
                                observation.cost_usd =
                                    estimate_cost(&observation.usage, input_cost, output_cost);
                                let evaluation = evaluate_benchmark_fixture_with_diagnostics(
                                    &fixture, candidate,
                                );
                                ObservedBenchmarkCase {
                                    fixture_id: fixture.id.clone(),
                                    trial,
                                    result: Some(evaluation.result),
                                    diagnostics: evaluation.diagnostics,
                                    generation: Some(observation),
                                    failure: None,
                                }
                            }
                            Err(failure) => ObservedBenchmarkCase {
                                fixture_id: fixture.id.clone(),
                                trial,
                                result: None,
                                diagnostics: None,
                                generation: None,
                                failure: Some(failure),
                            },
                        };
                        (sequence, case)
                    });
                }

                if tasks.is_empty() {
                    break;
                }
                let joined = tasks
                    .join_next()
                    .await
                    .ok_or("live benchmark task set ended unexpectedly")?
                    .map_err(|error| format!("live benchmark worker failed: {error}"))?;
                let (sequence, case) = joined;
                completed_runs += 1;
                eprintln!(
                    "[benchmark] provider={} model={} completed={}/{} fixture={} trial={} status={}",
                    provider_name(provider),
                    config.model,
                    completed_runs,
                    total_runs,
                    case.fixture_id,
                    case.trial + 1,
                    if case.failure.is_some() {
                        "failed"
                    } else {
                        "ok"
                    }
                );
                ordered[sequence] = Some(case);
            }
        }

        ordered
            .into_iter()
            .enumerate()
            .map(|(index, case)| {
                case.ok_or_else(|| format!("missing benchmark result for sequence {index}"))
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let mut observed = Vec::with_capacity(total_runs);
        for fixture in fixtures {
            let evaluation = evaluate_benchmark_fixture_with_diagnostics(
                &fixture,
                fixture.recorded_candidate.clone(),
            );
            observed.push(ObservedBenchmarkCase {
                fixture_id: fixture.id.clone(),
                trial: 0,
                result: Some(evaluation.result),
                diagnostics: evaluation.diagnostics,
                generation: None,
                failure: None,
            });
        }
        observed
    };

    let results: Vec<BenchmarkCaseResult> = observed
        .iter()
        .filter_map(|case| case.result.clone())
        .collect();
    let comparison = aggregate_benchmark(&results);
    let corpus = load_corpus_manifest(directory)?
        .map(|manifest| {
            let stratification = if config.provider.is_none() {
                Some(
                    aggregate_claim_corpus(&manifest, &results)
                        .map_err(|error| error.to_string())?,
                )
            } else {
                None
            };
            Ok::<_, String>(BenchmarkCorpusOutput {
                corpus_version: manifest.corpus_version,
                score_compatibility_id: manifest.score_compatibility_id,
                stratification,
            })
        })
        .transpose()?;
    let operational = operational_summary(&observed);
    let stability = if config.provider.is_some() {
        Some(trial_stability_summary(
            &observed,
            fixtures_per_trial(total_runs, config.trials),
            config.trials,
        )?)
    } else {
        None
    };
    Ok(BenchmarkOutput {
        provider: config.provider.map(provider_name),
        model: config.provider.map(|_| config.model.to_string()),
        corpus,
        comparison,
        operational,
        stability,
        cases: observed,
    })
}

fn fixtures_per_trial(total_runs: usize, trials: usize) -> usize {
    debug_assert!(trials > 0);
    total_runs / trials
}

fn load_corpus_manifest(directory: &Path) -> Result<Option<CorpusManifest>, String> {
    let path = directory.join("corpus/v1.json");
    if !path.is_file() {
        return Ok(None);
    }
    read_json(&path).map(Some)
}

fn load_fixtures(directory: &PathBuf) -> Result<Vec<BenchmarkFixture>, String> {
    let entries =
        fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("{}: {error}", directory.display()))?;
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    paths.sort();
    if paths.is_empty() {
        return Err(format!("{} contains no JSON fixtures", directory.display()));
    }
    paths.iter().map(read_json).collect()
}

fn estimate_cost(
    usage: &ModelUsage,
    input_cost_per_million: Option<f64>,
    output_cost_per_million: Option<f64>,
) -> Option<f64> {
    let input_rate = input_cost_per_million?;
    let output_rate = output_cost_per_million?;
    let input_tokens = usage.input_tokens? as f64;
    let output_tokens = usage.output_tokens? as f64;
    Some((input_tokens * input_rate + output_tokens * output_rate) / 1_000_000.0)
}

fn operational_summary(cases: &[ObservedBenchmarkCase]) -> OperationalSummary {
    let generations = cases.iter().filter_map(|case| case.generation.as_ref());
    let attempted_runs = cases.len();
    let failed_runs = cases.iter().filter(|case| case.failure.is_some()).count();
    let mut failure_classes = BTreeMap::new();
    for failure in cases.iter().filter_map(|case| case.failure.as_ref()) {
        *failure_classes.entry(failure.failure_class).or_insert(0) += 1;
    }
    let mut generated_runs = 0;
    let mut input_tokens = 0;
    let mut output_tokens = 0;
    let mut total_tokens = 0;
    let mut total_latency_ms = 0;
    let mut total_cost_usd = Some(0.0);

    for generation in generations {
        generated_runs += 1;
        input_tokens += generation.usage.input_tokens.unwrap_or(0);
        output_tokens += generation.usage.output_tokens.unwrap_or(0);
        total_tokens += generation.usage.total_tokens.unwrap_or_else(|| {
            generation.usage.input_tokens.unwrap_or(0) + generation.usage.output_tokens.unwrap_or(0)
        });
        total_latency_ms += generation.latency_ms;
        match (total_cost_usd, generation.cost_usd) {
            (Some(total), Some(cost)) => total_cost_usd = Some(total + cost),
            _ => total_cost_usd = None,
        }
    }

    if generated_runs == 0 {
        total_cost_usd = None;
    }

    OperationalSummary {
        attempted_runs,
        generated_runs,
        failed_runs,
        failure_classes,
        input_tokens,
        output_tokens,
        total_tokens,
        total_latency_ms,
        total_cost_usd,
    }
}

fn trial_stability_summary(
    cases: &[ObservedBenchmarkCase],
    expected_runs_per_trial: usize,
    requested_trials: usize,
) -> Result<TrialStabilitySummary, String> {
    let mut per_trial = Vec::with_capacity(requested_trials);
    for trial_index in 0..requested_trials {
        let trial_cases: Vec<&ObservedBenchmarkCase> = cases
            .iter()
            .filter(|case| case.trial == trial_index)
            .collect();
        let trial_results: Vec<BenchmarkCaseResult> = trial_cases
            .iter()
            .filter_map(|case| case.result.clone())
            .collect();
        let correctness_cases = trial_results.len();
        let trial_owned: Vec<ObservedBenchmarkCase> =
            trial_cases.iter().map(|case| (*case).clone()).collect();
        let operational = operational_summary(&trial_owned);
        let operationally_complete = operational.attempted_runs == expected_runs_per_trial
            && operational.generated_runs == expected_runs_per_trial
            && operational.failed_runs == 0;
        let comparison = if trial_results.is_empty() {
            None
        } else {
            Some(aggregate_benchmark(&trial_results))
        };
        per_trial.push(TrialSummary {
            trial_index,
            expected_runs: expected_runs_per_trial,
            correctness_cases,
            operationally_complete,
            operational,
            comparison,
        });
    }

    let complete_trials = per_trial
        .iter()
        .filter(|trial| trial.operationally_complete)
        .count();
    let complete_comparisons: Vec<&BenchmarkComparison> = per_trial
        .iter()
        .filter(|trial| trial.operationally_complete)
        .filter_map(|trial| trial.comparison.as_ref())
        .collect();
    let correctness = if complete_comparisons.is_empty() {
        None
    } else {
        Some(BenchmarkStabilityComparison {
            baseline: benchmark_arm_stability(&complete_comparisons, |comparison| {
                &comparison.baseline
            }),
            harness: benchmark_arm_stability(&complete_comparisons, |comparison| {
                &comparison.harness
            }),
        })
    };

    let diagnostic_trials = per_trial
        .iter()
        .map(|trial| DiagnosticTrial {
            trial_index: trial.trial_index,
            operationally_complete: trial.operationally_complete,
            operational_failures: trial.operational.failed_runs,
            observations: cases
                .iter()
                .filter(|case| case.trial == trial.trial_index)
                .filter_map(|case| case.diagnostics.clone())
                .collect(),
        })
        .collect::<Vec<_>>();
    let diagnostics = aggregate_repeated_diagnostics(&diagnostic_trials)
        .map_err(|error| format!("diagnostic stability aggregation failed: {error}"))?;

    Ok(TrialStabilitySummary {
        requested_trials,
        complete_trials,
        incomplete_trials: requested_trials.saturating_sub(complete_trials),
        correctness,
        diagnostics,
        operational: operational_distributions(cases, &per_trial),
        per_trial,
    })
}

fn benchmark_arm_stability(
    comparisons: &[&BenchmarkComparison],
    select: impl Fn(&BenchmarkComparison) -> &BenchmarkAggregate,
) -> BenchmarkArmStability {
    let metrics: Vec<&BenchmarkAggregate> = comparisons
        .iter()
        .map(|comparison| select(comparison))
        .collect();
    BenchmarkArmStability {
        verdict_accuracy: required_distribution(
            metrics.iter().map(|metric| metric.verdict_accuracy),
        ),
        accept_recall: required_distribution(metrics.iter().map(|metric| metric.accept_recall)),
        reject_recall: required_distribution(metrics.iter().map(|metric| metric.reject_recall)),
        unknown_recall: required_distribution(metrics.iter().map(|metric| metric.unknown_recall)),
        unsafe_accept_cases: required_distribution(
            metrics
                .iter()
                .map(|metric| metric.unsafe_accept_cases as f64),
        ),
        deterministic_verifier_failure_rate: required_distribution(
            metrics
                .iter()
                .map(|metric| metric.deterministic_verifier_failure_rate),
        ),
        contradiction_detection_rate: required_distribution(
            metrics
                .iter()
                .map(|metric| metric.contradiction_detection_rate),
        ),
        counterexample_detection_rate: required_distribution(
            metrics
                .iter()
                .map(|metric| metric.counterexample_detection_rate),
        ),
    }
}

fn operational_distributions(
    cases: &[ObservedBenchmarkCase],
    per_trial: &[TrialSummary],
) -> OperationalDistributions {
    let successful_request_total_tokens = scalar_distribution(cases.iter().filter_map(|case| {
        case.generation
            .as_ref()
            .and_then(|generation| observed_total_tokens(&generation.usage))
            .map(|tokens| tokens as f64)
    }));
    let successful_request_latency_ms = scalar_distribution(cases.iter().filter_map(|case| {
        case.generation
            .as_ref()
            .map(|generation| generation.latency_ms as f64)
    }));

    let mut complete_trial_tokens = Vec::new();
    let mut complete_trial_latency = Vec::new();
    for trial in per_trial
        .iter()
        .filter(|trial| trial.operationally_complete)
    {
        let trial_cases: Vec<&ObservedBenchmarkCase> = cases
            .iter()
            .filter(|case| case.trial == trial.trial_index)
            .collect();
        let tokens: Option<Vec<u64>> = trial_cases
            .iter()
            .map(|case| {
                case.generation
                    .as_ref()
                    .and_then(|generation| observed_total_tokens(&generation.usage))
            })
            .collect();
        if let Some(tokens) = tokens {
            complete_trial_tokens.push(tokens.into_iter().sum::<u64>() as f64);
        }
        complete_trial_latency.push(
            trial_cases
                .iter()
                .filter_map(|case| case.generation.as_ref())
                .map(|generation| generation.latency_ms)
                .sum::<u128>() as f64,
        );
    }

    OperationalDistributions {
        successful_request_total_tokens,
        successful_request_latency_ms,
        complete_trial_total_tokens: scalar_distribution(complete_trial_tokens),
        complete_trial_total_latency_ms: scalar_distribution(complete_trial_latency),
    }
}

fn observed_total_tokens(usage: &ModelUsage) -> Option<u64> {
    usage.total_tokens.or_else(|| {
        usage
            .input_tokens
            .zip(usage.output_tokens)
            .and_then(|(input, output)| input.checked_add(output))
    })
}

fn required_distribution(values: impl IntoIterator<Item = f64>) -> ScalarDistribution {
    scalar_distribution(values).expect("complete trial stability always has at least one value")
}

fn scalar_distribution(values: impl IntoIterator<Item = f64>) -> Option<ScalarDistribution> {
    let values: Vec<f64> = values
        .into_iter()
        .filter(|value| value.is_finite())
        .collect();
    if values.is_empty() {
        return None;
    }
    let count = values.len();
    let mean = values.iter().sum::<f64>() / count as f64;
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / count as f64;
    Some(ScalarDistribution {
        count,
        mean,
        min,
        max,
        stddev: variance.sqrt(),
    })
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Mistral => "mistral",
        Provider::Google | Provider::Gemma => "google",
        Provider::Nvidia => "nvidia",
    }
}

fn print_benchmark_human(output: &BenchmarkOutput) {
    println!(
        "attempted_runs: {} generated_runs: {} failed_runs: {}",
        output.operational.attempted_runs,
        output.operational.generated_runs,
        output.operational.failed_runs
    );
    println!("cases: {}", output.comparison.harness.cases);
    println!(
        "verdict_accuracy: baseline={:.3} harness={:.3}",
        output.comparison.baseline.verdict_accuracy, output.comparison.harness.verdict_accuracy
    );
    println!(
        "unsupported_accepted_claims: baseline={} harness={}",
        output.comparison.baseline.unsupported_accepted_claims,
        output.comparison.harness.unsupported_accepted_claims
    );
    println!(
        "accept_recall: baseline={:.3} harness={:.3}",
        output.comparison.baseline.accept_recall, output.comparison.harness.accept_recall
    );
    println!(
        "reject_recall: baseline={:.3} harness={:.3}",
        output.comparison.baseline.reject_recall, output.comparison.harness.reject_recall
    );
    println!(
        "unknown_recall: baseline={:.3} harness={:.3}",
        output.comparison.baseline.unknown_recall, output.comparison.harness.unknown_recall
    );
    println!(
        "hidden_assumption_exposure: baseline={:.3} harness={:.3}",
        output.comparison.baseline.hidden_assumption_exposure_rate,
        output.comparison.harness.hidden_assumption_exposure_rate
    );
    println!(
        "contradiction_detection: baseline={:.3} harness={:.3}",
        output.comparison.baseline.contradiction_detection_rate,
        output.comparison.harness.contradiction_detection_rate
    );
    println!(
        "causal_edge_quality: baseline={:.3} harness={:.3}",
        output.comparison.baseline.causal_edge_quality,
        output.comparison.harness.causal_edge_quality
    );
    if let Some(corpus) = &output.corpus {
        println!(
            "corpus: version={} compatibility={}",
            corpus.corpus_version, corpus.score_compatibility_id
        );
        if let Some(stratification) = &corpus.stratification {
            for slice in &stratification.by_category {
                println!(
                    "corpus_category: {} cases={} harness_accuracy={:.3}",
                    slice.label, slice.cases, slice.comparison.harness.verdict_accuracy
                );
            }
            for slice in &stratification.by_difficulty {
                println!(
                    "corpus_difficulty: {} cases={} harness_accuracy={:.3}",
                    slice.label, slice.cases, slice.comparison.harness.verdict_accuracy
                );
            }
        }
    }
    if let Some(stability) = &output.stability {
        println!(
            "trials: requested={} complete={} incomplete={}",
            stability.requested_trials, stability.complete_trials, stability.incomplete_trials
        );
        if let Some(correctness) = &stability.correctness {
            let accuracy = &correctness.harness.verdict_accuracy;
            println!(
                "harness_accuracy_stability: mean={:.3} min={:.3} max={:.3} stddev={:.3} n={}",
                accuracy.mean, accuracy.min, accuracy.max, accuracy.stddev, accuracy.count
            );
        }
        println!(
            "diagnostic_stability: complete_trials={} incomplete_trials={} fixtures={} operational_failures={}",
            stability.diagnostics.complete_trials,
            stability.diagnostics.incomplete_trials,
            stability.diagnostics.fixtures.len(),
            stability.diagnostics.operational_failures
        );
    }
}

fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn print_json(value: &impl Serialize) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[cfg(test)]
mod candidate_json_tests {
    use super::*;

    #[test]
    fn accepts_one_complete_candidate_with_non_json_trailing_text() {
        let text = r#"{"claims":[],"inferences":[]}
<|channel|>done"#;
        let (candidate, normalized) = parse_candidate_json(text).unwrap();
        assert!(candidate.claims.is_empty());
        assert!(normalized);
    }

    #[test]
    fn rejects_multiple_json_candidate_values() {
        let text = r#"{"claims":[],"inferences":[]}
{"claims":[],"inferences":[]}"#;
        assert!(parse_candidate_json(text).is_err());
    }

    #[test]
    fn rejects_incomplete_candidate_json() {
        let text = r#"{"claims":[{"#;
        assert!(parse_candidate_json(text).is_err());
    }

    #[test]
    fn parses_nvidia_single_model_selection() {
        let cli = Cli::try_parse_from([
            "reason",
            "eval",
            "fixtures",
            "--provider",
            "nvidia",
            "--model",
            "google/gemma-4-31b-it",
        ])
        .unwrap();
        match cli.command {
            Command::Eval {
                provider, model, ..
            } => {
                assert!(matches!(provider, Some(Provider::Nvidia)));
                assert_eq!(model, "google/gemma-4-31b-it");
            }
            _ => panic!("expected eval command"),
        }
    }

    #[test]
    fn parses_live_concurrency() {
        let cli = Cli::try_parse_from([
            "reason",
            "eval",
            "fixtures",
            "--provider",
            "nvidia",
            "--model",
            "nvidia/nemotron-3.5-lightning-30b-a3b",
            "--concurrency",
            "3",
        ])
        .unwrap();
        match cli.command {
            Command::Eval { concurrency, .. } => assert_eq!(concurrency, 3),
            _ => panic!("expected eval command"),
        }
    }

    #[test]
    fn operational_summary_preserves_failed_generation_class() {
        let cases = vec![ObservedBenchmarkCase {
            fixture_id: "fixture-a".into(),
            trial: 0,
            result: None,
            diagnostics: None,
            generation: None,
            failure: Some(GenerationFailure {
                provider: "nvidia",
                model: "test-model".into(),
                latency_ms: 12,
                failure_class: "rate_limit",
                message: "NVIDIA API returned HTTP 429".into(),
            }),
        }];
        let summary = operational_summary(&cases);
        assert_eq!(summary.attempted_runs, 1);
        assert_eq!(summary.generated_runs, 0);
        assert_eq!(summary.failed_runs, 1);
        assert_eq!(summary.failure_classes.get("rate_limit"), Some(&1));
    }

    #[test]
    fn scalar_distribution_uses_population_standard_deviation() {
        let distribution = scalar_distribution([0.8, 1.0]).unwrap();
        assert_eq!(distribution.count, 2);
        assert!((distribution.mean - 0.9).abs() < 1e-12);
        assert!((distribution.min - 0.8).abs() < 1e-12);
        assert!((distribution.max - 1.0).abs() < 1e-12);
        assert!((distribution.stddev - 0.1).abs() < 1e-12);
    }

    #[test]
    fn stability_excludes_incomplete_trials_from_correctness_distribution() {
        use reasoning_harness_core::{BenchmarkArmResult, Verdict};

        fn arm(correct: bool) -> BenchmarkArmResult {
            BenchmarkArmResult {
                verdict: Some(if correct {
                    Verdict::Accept
                } else {
                    Verdict::Unknown
                }),
                claims: 1,
                claims_with_evidence: 1,
                inference_edges: 0,
                verdict_correct: correct,
                evidence_coverage: 1.0,
                unsupported_accepted_claims: 0,
                unsafe_accept: false,
                hidden_assumptions_exposed: 0,
                contradiction_claims_detected: 0,
                counterexamples_detected: 0,
                hard_adversarial_findings: 0,
                soft_adversarial_findings: 0,
                bad_inference_edges_retained: 0,
                deterministic_failure: false,
                deterministic_failure_reason: None,
            }
        }

        fn result(id: &str, correct: bool) -> BenchmarkCaseResult {
            BenchmarkCaseResult {
                fixture_id: id.into(),
                expected_verdict: Verdict::Accept,
                expected_hidden_assumptions: 0,
                expected_contradiction: false,
                expected_counterexample: false,
                baseline: arm(correct),
                harness: arm(correct),
            }
        }

        fn generation(tokens: u64, latency_ms: u128) -> GenerationObservation {
            GenerationObservation {
                provider: "test",
                model: "test-model".into(),
                usage: ModelUsage {
                    input_tokens: Some(tokens / 2),
                    output_tokens: Some(tokens - tokens / 2),
                    total_tokens: Some(tokens),
                },
                latency_ms,
                provider_attempts: 1,
                cost_usd: None,
            }
        }

        let cases = vec![
            ObservedBenchmarkCase {
                fixture_id: "a".into(),
                trial: 0,
                result: Some(result("a", true)),
                diagnostics: Some(DiagnosticObservation {
                    fixture_id: "a".into(),
                    signals: vec![],
                }),
                generation: Some(generation(10, 100)),
                failure: None,
            },
            ObservedBenchmarkCase {
                fixture_id: "b".into(),
                trial: 0,
                result: Some(result("b", true)),
                diagnostics: Some(DiagnosticObservation {
                    fixture_id: "b".into(),
                    signals: vec![],
                }),
                generation: Some(generation(20, 200)),
                failure: None,
            },
            ObservedBenchmarkCase {
                fixture_id: "a".into(),
                trial: 1,
                result: Some(result("a", false)),
                diagnostics: Some(DiagnosticObservation {
                    fixture_id: "a".into(),
                    signals: vec![],
                }),
                generation: Some(generation(30, 300)),
                failure: None,
            },
            ObservedBenchmarkCase {
                fixture_id: "b".into(),
                trial: 1,
                result: None,
                diagnostics: None,
                generation: None,
                failure: Some(GenerationFailure {
                    provider: "test",
                    model: "test-model".into(),
                    latency_ms: 400,
                    failure_class: "timeout",
                    message: "timed out".into(),
                }),
            },
        ];

        let stability = trial_stability_summary(&cases, 2, 2).unwrap();
        assert_eq!(stability.complete_trials, 1);
        assert_eq!(stability.incomplete_trials, 1);
        assert!(stability.per_trial[0].operationally_complete);
        assert!(!stability.per_trial[1].operationally_complete);
        assert_eq!(stability.per_trial[1].correctness_cases, 1);
        assert_eq!(stability.per_trial[1].operational.failed_runs, 1);
        assert_eq!(
            stability.per_trial[1]
                .operational
                .failure_classes
                .get("timeout"),
            Some(&1)
        );

        let diagnostics = &stability.diagnostics;
        assert_eq!(diagnostics.requested_trials, 2);
        assert_eq!(diagnostics.complete_trials, 1);
        assert_eq!(diagnostics.incomplete_trials, 1);
        assert_eq!(diagnostics.operational_failures, 1);
        assert_eq!(diagnostics.excluded_incomplete_trial_observations, 1);
        assert_eq!(diagnostics.fixtures.len(), 2);

        let accuracy = &stability.correctness.unwrap().harness.verdict_accuracy;
        assert_eq!(accuracy.count, 1);
        assert!((accuracy.mean - 1.0).abs() < 1e-12);
        assert!((accuracy.stddev - 0.0).abs() < 1e-12);

        let request_tokens = stability
            .operational
            .successful_request_total_tokens
            .unwrap();
        assert_eq!(request_tokens.count, 3);
        let trial_tokens = stability.operational.complete_trial_total_tokens.unwrap();
        assert_eq!(trial_tokens.count, 1);
        assert!((trial_tokens.mean - 30.0).abs() < 1e-12);
    }

    #[test]
    fn live_corpus_identity_serializes_without_pooled_stratification() {
        let corpus = BenchmarkCorpusOutput {
            corpus_version: "1.0.0".into(),
            score_compatibility_id: "corpus-v1".into(),
            stratification: None,
        };
        let json = serde_json::to_value(corpus).unwrap();
        assert_eq!(json["corpus_version"], "1.0.0");
        assert_eq!(json["score_compatibility_id"], "corpus-v1");
        assert!(json.get("stratification").is_none());
    }

    #[test]
    fn maps_operational_failure_classes_stably() {
        assert_eq!(model_error_class(ModelErrorKind::RateLimit), "rate_limit");
        assert_eq!(model_error_class(ModelErrorKind::Quota), "quota");
        assert_eq!(
            model_error_class(ModelErrorKind::ProviderUnavailable),
            "provider_unavailable"
        );
        assert_eq!(model_error_class(ModelErrorKind::Timeout), "timeout");
        assert_eq!(model_error_class(ModelErrorKind::Protocol), "protocol");
    }
}
