use std::{
    collections::{BTreeMap, VecDeque},
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::Arc,
    time::Instant,
};

use clap::{Parser, Subcommand, ValueEnum};
use reasoning_harness_core::{
    AdversarialDiscoveryPass, AssumptionDiscoveryPass, BenchmarkAggregate, BenchmarkCaseResult,
    BenchmarkComparison, BenchmarkFixture, CalibrationLabel, ClaimCorpusSummary, CorpusManifest,
    DiagnosticObservation, DiagnosticTrial, EvidenceQualificationPass, HarnessInput, ModelAdapter,
    ModelBackedSoftJudgeError, ModelError, ModelErrorKind, ModelUsage,
    REASONING_ARTIFACT_CONTRACT_ID, REASONING_CANDIDATE_CONTRACT_ID, ReasoningArtifact,
    ReasoningCandidate, RepeatedDiagnosticReport, ResolutionBenchmarkAggregate,
    ResolutionBenchmarkCaseResult, ResolutionBenchmarkFixture, SemanticDiagnosticKind,
    SoftJudgeCalibrationFixture, SoftJudgeCalibrationReport, SoftJudgeDecision,
    SoftJudgeFallbackReason, SoftJudgeIdentity, SoftJudgeObservation, StrictAcceptancePolicy,
    StructuredFactConflictDetector, TrustedVerificationPass, VerificationPass, VerificationReceipt,
    aggregate_benchmark, aggregate_claim_corpus, aggregate_repeated_diagnostics,
    aggregate_resolution_benchmark, aggregate_soft_judge_calibration,
    build_candidate_json_fallback_request, build_candidate_request, evaluate,
    evaluate_benchmark_fixture_with_diagnostics, evaluate_resolution_fixture,
    frameworks::five_whys::FiveWhysRestatementPass, reasoning_artifact_schema,
    reasoning_candidate_schema, run_harness, run_model_backed_soft_judge,
    structured_fact_verifier_for_input, validate_artifact,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter, NvidiaAdapter};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

const CLI_OUTPUT_SCHEMA_VERSION: &str = "reason-cli-output-v1";
const CLI_CONFIG_CONTRACT_ID: &str = "reason-config-v1";
const DEFAULT_MAX_TOKENS: u32 = 1024;

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

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
enum OutputFormat {
    #[default]
    Human,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SchemaKind {
    Artifact,
    Candidate,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Mistral,
    Google,
    Nvidia,
    #[value(hide = true)]
    Gemma,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
struct RunFileConfig {
    provider: Option<Provider>,
    model: Option<String>,
    max_tokens: Option<u32>,
    format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CliFileConfig {
    schema_version: String,
    #[serde(default)]
    run: RunFileConfig,
}

impl Default for CliFileConfig {
    fn default() -> Self {
        Self {
            schema_version: CLI_CONFIG_CONTRACT_ID.into(),
            run: RunFileConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct LoadedCliConfig {
    config: CliFileConfig,
    sources: Vec<&'static str>,
}

#[derive(Debug)]
struct ResolvedRunConfig {
    provider: Option<Provider>,
    model: Option<String>,
    max_tokens: u32,
    format: OutputFormat,
    config_sources: Vec<&'static str>,
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

    fn adapter(&self) -> &dyn ModelAdapter {
        match self {
            Self::Mistral(adapter) => adapter,
            Self::Google(adapter) => adapter,
            Self::Nvidia(adapter) => adapter,
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
    /// PRODUCT: Generate or load a candidate, then execute the harness-owned correctness process.
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
        /// Provider model identifier used for live candidate generation. Required for live mode unless configured.
        #[arg(long)]
        model: Option<String>,
        /// Maximum candidate-generation tokens.
        #[arg(long)]
        max_tokens: Option<u32>,
        /// Optional provider random seed for repeatable research runs.
        #[arg(long)]
        seed: Option<u64>,
        /// Highest-precedence non-secret config file layered over project/user config.
        #[arg(long, value_name = "PATH", conflicts_with = "no_config")]
        config: Option<PathBuf>,
        /// Ignore user/project config for a hermetic invocation.
        #[arg(long)]
        no_config: bool,
        #[arg(long, value_enum)]
        format: Option<OutputFormat>,
    },
    /// PRODUCT: Deterministically validate a finalized ReasoningArtifact JSON file.
    Verify {
        artifact: PathBuf,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// PRODUCT: Print a versioned JSON Schema for a supported wire contract.
    Schema {
        #[arg(value_enum)]
        kind: SchemaKind,
    },
    /// RESEARCH/EVAL: Evaluate one artifact or a directory of benchmark fixtures.
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
    /// RESEARCH/EVAL: Evaluate the deterministic bounded-resolution research scenarios.
    EvalResolution {
        /// Directory containing ResolutionBenchmarkFixture JSON scenarios.
        target: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// RESEARCH/EVAL: Evaluate recorded or live soft semantic-judge calibration.
    EvalJudges {
        /// Directory containing SoftJudgeCalibrationFixture JSON cases.
        target: PathBuf,
        /// Optional live provider. Without this, committed recorded observations are used.
        #[arg(long, value_enum)]
        provider: Option<Provider>,
        /// Provider model identifier used for live semantic judging.
        #[arg(long, default_value = "ministral-8b-latest")]
        model: String,
        /// Maximum tokens for one soft-judge generation.
        #[arg(long, default_value_t = 256)]
        max_tokens: u32,
        /// Base random seed. Trial N uses base_seed + N.
        #[arg(long)]
        seed: Option<u64>,
        /// Number of live calibration trials. Recorded mode requires exactly one.
        #[arg(long, default_value_t = 1)]
        trials: usize,
        /// Maximum number of live semantic-judge fixture calls in flight within one trial.
        #[arg(long, default_value_t = 1)]
        concurrency: usize,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}

#[derive(Debug, Serialize)]
struct CliContractVersions {
    artifact: &'static str,
    candidate: &'static str,
    config: &'static str,
}

#[derive(Debug, Serialize)]
struct CliEnvelope<T> {
    schema_version: &'static str,
    command: &'static str,
    cli_version: &'static str,
    contracts: CliContractVersions,
    result: T,
}

#[derive(Debug, Serialize)]
struct SchemaOutput {
    contract_id: &'static str,
    schema: serde_json::Value,
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
struct RunConfigurationObservation {
    mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    output_format: OutputFormat,
    config_sources: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct RunOutput {
    configuration: RunConfigurationObservation,
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

#[derive(Debug, Serialize)]
struct ResolutionBenchmarkOutput {
    aggregate: ResolutionBenchmarkAggregate,
    cases: Vec<ResolutionBenchmarkCaseResult>,
}

#[derive(Debug, Clone, Serialize)]
struct LiveSoftJudgeFailure {
    fixture_id: String,
    trial: usize,
    failure_class: &'static str,
    latency_ms: u128,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct LiveSoftJudgeCase {
    fixture_id: String,
    trial: usize,
    kind: SemanticDiagnosticKind,
    label: CalibrationLabel,
    #[serde(skip_serializing_if = "Option::is_none")]
    observation: Option<SoftJudgeObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_attempts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_reason: Option<SoftJudgeFallbackReason>,
    latency_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<LiveSoftJudgeFailure>,
}

#[derive(Debug, Serialize)]
struct LiveSoftJudgeOperationalSummary {
    attempted_runs: usize,
    successful_runs: usize,
    failed_runs: usize,
    failure_classes: BTreeMap<&'static str, usize>,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    successful_provider_attempts: u64,
    fallback_runs: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_rate: Option<f64>,
    fallback_reason_counts: BTreeMap<SoftJudgeFallbackReason, usize>,
    total_latency_ms: u128,
}

#[derive(Debug, Serialize)]
struct LiveSoftJudgeFamilySummary {
    kind: SemanticDiagnosticKind,
    successful_runs: usize,
    findings: usize,
    no_findings: usize,
    abstentions: usize,
}

#[derive(Debug, Serialize)]
struct LiveSoftJudgeTrialSummary {
    trial_index: usize,
    expected_cases: usize,
    successful_cases: usize,
    operationally_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    report: Option<SoftJudgeCalibrationReport>,
}

#[derive(Debug, Serialize)]
struct LiveSoftJudgeStability {
    requested_trials: usize,
    complete_trials: usize,
    incomplete_trials: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    precision: Option<ScalarDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recall: Option<ScalarDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision_coverage: Option<ScalarDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambiguous_abstention_rate: Option<ScalarDistribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    abstentions: Option<ScalarDistribution>,
}

#[derive(Debug, Serialize)]
struct LiveSoftJudgeOutput {
    provider: &'static str,
    model: String,
    corpus: String,
    operational: LiveSoftJudgeOperationalSummary,
    stability: LiveSoftJudgeStability,
    families: Vec<LiveSoftJudgeFamilySummary>,
    per_trial: Vec<LiveSoftJudgeTrialSummary>,
    cases: Vec<LiveSoftJudgeCase>,
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
            config,
            no_config,
            format,
        } => {
            ensure_single_stdin(
                std::iter::once(&input)
                    .chain(candidate.iter())
                    .chain(receipts.iter()),
            )?;
            if config.as_ref().is_some_and(|path| is_stdin(path)) {
                return Err("--config must be a file path; stdin config is not supported".into());
            }
            let loaded_config = if no_config {
                LoadedCliConfig::default()
            } else {
                load_cli_config(config.as_ref())?
            };
            let resolved = resolve_run_config(
                candidate.is_some(),
                provider,
                model,
                max_tokens,
                format,
                loaded_config,
            )?;
            let input: HarnessInput = read_json(&input)?;
            let (candidate, generation) = match (candidate, resolved.provider) {
                (Some(path), None) => (read_json(&path)?, None),
                (None, Some(provider)) => {
                    let model = resolved
                        .model
                        .as_deref()
                        .expect("live run config validates model presence");
                    let generator = LiveGenerator::from_provider(provider, model)?;
                    let (candidate, observation) = generator
                        .generate(&input, resolved.max_tokens, seed, model)
                        .await
                        .map_err(|failure| format_generation_failure(&failure))?;
                    (candidate, Some(observation))
                }
                (Some(_), Some(_)) => {
                    return Err("choose either --candidate or --provider, not both".into());
                }
                (None, None) => {
                    return Err(
                        "reason run requires either --candidate or a live --provider (CLI/config)"
                            .into(),
                    );
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
            let live = generation.is_some();
            let output = RunOutput {
                configuration: RunConfigurationObservation {
                    mode: if live {
                        "live_provider"
                    } else {
                        "recorded_candidate"
                    },
                    provider: resolved.provider.map(provider_name),
                    model: live.then(|| resolved.model.clone()).flatten(),
                    max_tokens: live.then_some(resolved.max_tokens),
                    output_format: resolved.format,
                    config_sources: resolved.config_sources,
                },
                candidate,
                outcome,
                generation,
            };
            match resolved.format {
                OutputFormat::Human => {
                    println!("verdict: {:?}", output.outcome.verdict);
                    if let Some(generation) = &output.generation {
                        println!(
                            "generation: provider={} model={} latency_ms={}",
                            generation.provider, generation.model, generation.latency_ms
                        );
                    }
                }
                OutputFormat::Json => print_product_json("run", &output)?,
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
                OutputFormat::Json => print_product_json(
                    "verify",
                    &VerifyOutput {
                        valid: report.is_ok(),
                        diagnostics: &report.diagnostics,
                    },
                )?,
            }
            if report.is_ok() {
                Ok(())
            } else {
                Err("artifact validation failed".into())
            }
        }
        Command::Schema { kind } => {
            let output = match kind {
                SchemaKind::Artifact => SchemaOutput {
                    contract_id: REASONING_ARTIFACT_CONTRACT_ID,
                    schema: reasoning_artifact_schema(),
                },
                SchemaKind::Candidate => SchemaOutput {
                    contract_id: REASONING_CANDIDATE_CONTRACT_ID,
                    schema: reasoning_candidate_schema(),
                },
                SchemaKind::Config => SchemaOutput {
                    contract_id: CLI_CONFIG_CONTRACT_ID,
                    schema: serde_json::to_value(schema_for!(CliFileConfig))
                        .map_err(|error| error.to_string())?,
                },
            };
            print_product_json("schema", &output)
        }
        Command::EvalResolution { target, format } => {
            if !target.is_dir() {
                return Err("eval-resolution requires a fixture directory".into());
            }
            let output = run_resolution_fixture_suite(&target)?;
            match format {
                OutputFormat::Human => print_resolution_benchmark_human(&output),
                OutputFormat::Json => print_json(&output)?,
            }
            Ok(())
        }
        Command::EvalJudges {
            target,
            provider,
            model,
            max_tokens,
            seed,
            trials,
            concurrency,
            format,
        } => {
            if !target.is_dir() {
                return Err("eval-judges requires a fixture directory".into());
            }
            if trials == 0 {
                return Err("--trials must be at least 1".into());
            }
            if !(1..=10).contains(&concurrency) {
                return Err("--concurrency must be between 1 and 10".into());
            }
            match provider {
                None => {
                    if trials != 1 {
                        return Err("recorded eval-judges supports exactly one trial".into());
                    }
                    if concurrency != 1 {
                        return Err("--concurrency requires a live eval-judges provider".into());
                    }
                    let report = run_soft_judge_calibration_suite(&target)?;
                    match format {
                        OutputFormat::Human => print_soft_judge_calibration_human(&report),
                        OutputFormat::Json => print_json(&report)?,
                    }
                }
                Some(provider) => {
                    let output = run_live_soft_judge_calibration_suite(
                        &target,
                        provider,
                        &model,
                        max_tokens,
                        seed,
                        trials,
                        concurrency,
                    )
                    .await?;
                    match format {
                        OutputFormat::Human => print_live_soft_judge_human(&output),
                        OutputFormat::Json => print_json(&output)?,
                    }
                }
            }
            Ok(())
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

fn run_resolution_fixture_suite(directory: &PathBuf) -> Result<ResolutionBenchmarkOutput, String> {
    let fixture_root = directory
        .parent()
        .ok_or_else(|| format!("{} has no fixture root", directory.display()))?;
    let manifest: CorpusManifest = read_json(&fixture_root.join("corpus/v1.json"))?;
    let entries =
        fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let mut paths = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    if paths.is_empty() {
        return Err(format!(
            "{} contains no resolution fixtures",
            directory.display()
        ));
    }

    let mut cases = Vec::with_capacity(paths.len());
    for path in paths {
        let fixture: ResolutionBenchmarkFixture = read_json(&path)?;
        let metadata = manifest
            .cases
            .iter()
            .find(|case| case.case_id == fixture.base_case_id)
            .ok_or_else(|| {
                format!(
                    "resolution scenario {} references unknown corpus case {}",
                    fixture.id, fixture.base_case_id
                )
            })?;
        if metadata.fixture_path != fixture.base_fixture_path {
            return Err(format!(
                "resolution scenario {} base path {} does not match corpus path {}",
                fixture.id, fixture.base_fixture_path, metadata.fixture_path
            ));
        }
        let base: BenchmarkFixture = read_json(&fixture_root.join(&fixture.base_fixture_path))?;
        if base.id != metadata.fixture_id {
            return Err(format!(
                "resolution scenario {} base fixture id {} does not match corpus fixture id {}",
                fixture.id, base.id, metadata.fixture_id
            ));
        }
        cases
            .push(evaluate_resolution_fixture(&fixture, &base).map_err(|error| error.to_string())?);
    }
    let aggregate = aggregate_resolution_benchmark(&cases);
    Ok(ResolutionBenchmarkOutput { aggregate, cases })
}

fn load_soft_judge_fixtures(
    directory: &PathBuf,
) -> Result<Vec<SoftJudgeCalibrationFixture>, String> {
    let entries =
        fs::read_dir(directory).map_err(|error| format!("{}: {error}", directory.display()))?;
    let mut paths = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    if paths.is_empty() {
        return Err(format!(
            "{} contains no semantic-judge calibration fixtures",
            directory.display()
        ));
    }
    paths.iter().map(read_json).collect()
}

fn run_soft_judge_calibration_suite(
    directory: &PathBuf,
) -> Result<SoftJudgeCalibrationReport, String> {
    let fixtures = load_soft_judge_fixtures(directory)?;
    aggregate_soft_judge_calibration(&fixtures).map_err(|error| error.to_string())
}

async fn run_live_soft_judge_calibration_suite(
    directory: &PathBuf,
    provider: Provider,
    model: &str,
    max_tokens: u32,
    seed: Option<u64>,
    trials: usize,
    concurrency: usize,
) -> Result<LiveSoftJudgeOutput, String> {
    let fixtures = load_soft_judge_fixtures(directory)?;
    let generator = Arc::new(LiveGenerator::from_provider(provider, model)?);
    let identity = SoftJudgeIdentity {
        judge_id: format!("live:{}:{model}", provider_name(provider)),
        model_id: model.to_string(),
        configuration_id: "soft-semantic-v3".into(),
    };
    let total_runs = fixtures
        .len()
        .checked_mul(trials)
        .ok_or("semantic-judge fixture/trial count overflowed usize")?;
    let mut completed_runs = 0usize;
    let mut started_runs = 0usize;
    let mut cases = Vec::with_capacity(total_runs);
    let mut per_trial = Vec::with_capacity(trials);

    // Keep trials sequential so each stability sample remains one full-corpus pass.
    // Concurrency overlaps only independent fixtures within the active trial.
    for trial_index in 0..trials {
        let trial_seed = seed
            .map(|base| {
                base.checked_add(trial_index as u64)
                    .ok_or_else(|| "soft-judge trial seed overflow".to_string())
            })
            .transpose()?;
        let mut evaluated = fixtures.clone();
        for fixture in &mut evaluated {
            fixture.recorded_observations.clear();
        }
        let mut pending = fixtures
            .iter()
            .cloned()
            .enumerate()
            .collect::<VecDeque<_>>();
        let mut tasks = tokio::task::JoinSet::new();
        let mut ordered_cases: Vec<Option<LiveSoftJudgeCase>> =
            (0..fixtures.len()).map(|_| None).collect();
        let mut ordered_observations: Vec<Option<SoftJudgeObservation>> =
            (0..fixtures.len()).map(|_| None).collect();
        let mut successful_cases = 0usize;
        let mut trial_failed = false;

        loop {
            while tasks.len() < concurrency {
                let Some((fixture_index, fixture)) = pending.pop_front() else {
                    break;
                };
                let generator = Arc::clone(&generator);
                let identity = identity.clone();
                let provider_label = provider_name(provider);
                let model_label = model.to_string();
                started_runs += 1;
                eprintln!(
                    "[semantic-judge] provider={} model={} start={}/{} fixture={} trial={} in_flight={}",
                    provider_label,
                    model_label,
                    started_runs,
                    total_runs,
                    fixture.id,
                    trial_index + 1,
                    tasks.len() + 1
                );
                tasks.spawn(async move {
                    let started = Instant::now();
                    let result = run_model_backed_soft_judge(
                        generator.adapter(),
                        identity,
                        &fixture.request,
                        max_tokens,
                        trial_seed,
                    )
                    .await;
                    let latency_ms = started.elapsed().as_millis();
                    let (case, observation) = match result {
                        Ok(result) => {
                            let observation = result.observation.clone();
                            (
                                LiveSoftJudgeCase {
                                    fixture_id: fixture.id.clone(),
                                    trial: trial_index,
                                    kind: fixture.request.kind,
                                    label: fixture.label,
                                    observation: Some(result.observation),
                                    provider_model: Some(result.model),
                                    usage: Some(result.usage),
                                    provider_attempts: Some(result.provider_attempts),
                                    fallback_reason: Some(result.fallback_reason),
                                    latency_ms,
                                    failure: None,
                                },
                                Some(observation),
                            )
                        }
                        Err(error) => {
                            let failure_class = soft_judge_failure_class(&error);
                            let message = error.to_string();
                            (
                                LiveSoftJudgeCase {
                                    fixture_id: fixture.id.clone(),
                                    trial: trial_index,
                                    kind: fixture.request.kind,
                                    label: fixture.label,
                                    observation: None,
                                    provider_model: None,
                                    usage: None,
                                    provider_attempts: None,
                                    fallback_reason: None,
                                    latency_ms,
                                    failure: Some(LiveSoftJudgeFailure {
                                        fixture_id: fixture.id.clone(),
                                        trial: trial_index,
                                        failure_class,
                                        latency_ms,
                                        message,
                                    }),
                                },
                                None,
                            )
                        }
                    };
                    (fixture_index, case, observation)
                });
            }

            if tasks.is_empty() {
                break;
            }
            let joined = tasks
                .join_next()
                .await
                .ok_or("semantic-judge task set ended unexpectedly")?
                .map_err(|error| format!("semantic-judge worker failed: {error}"))?;
            let (fixture_index, case, observation) = joined;
            completed_runs += 1;
            if case.failure.is_some() {
                trial_failed = true;
            } else {
                successful_cases += 1;
            }
            eprintln!(
                "[semantic-judge] provider={} model={} completed={}/{} fixture={} trial={} status={}",
                provider_name(provider),
                model,
                completed_runs,
                total_runs,
                case.fixture_id,
                trial_index + 1,
                if case.failure.is_some() {
                    "failed"
                } else {
                    "ok"
                }
            );
            ordered_cases[fixture_index] = Some(case);
            ordered_observations[fixture_index] = observation;
        }

        for (fixture_index, observation) in ordered_observations.into_iter().enumerate() {
            if let Some(observation) = observation {
                evaluated[fixture_index]
                    .recorded_observations
                    .push(observation);
            }
        }
        cases.extend(
            ordered_cases
                .into_iter()
                .enumerate()
                .map(|(index, case)| {
                    case.ok_or_else(|| {
                        format!("missing semantic-judge result for fixture index {index}")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        );

        let operationally_complete = !trial_failed && successful_cases == fixtures.len();
        let report = if operationally_complete {
            Some(
                aggregate_soft_judge_calibration(&evaluated)
                    .map_err(|error| format!("live soft-judge calibration failed: {error}"))?,
            )
        } else {
            None
        };
        per_trial.push(LiveSoftJudgeTrialSummary {
            trial_index,
            expected_cases: fixtures.len(),
            successful_cases,
            operationally_complete,
            report,
        });
    }

    let operational = live_soft_judge_operational_summary(&cases);
    let stability = live_soft_judge_stability(&per_trial, trials);
    let families = live_soft_judge_family_summary(&cases, &per_trial);
    let corpus = directory
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("semantic-judges")
        .to_string();
    Ok(LiveSoftJudgeOutput {
        provider: provider_name(provider),
        model: model.to_string(),
        corpus,
        operational,
        stability,
        families,
        per_trial,
        cases,
    })
}

fn soft_judge_failure_class(error: &ModelBackedSoftJudgeError) -> &'static str {
    if let Some(kind) = error.model_error_kind() {
        return model_error_class(kind);
    }
    match error {
        ModelBackedSoftJudgeError::InvalidStructuredOutput(_) => "protocol",
        ModelBackedSoftJudgeError::SoftJudge(_) => "soft_judge_protocol",
        ModelBackedSoftJudgeError::Model(_) => unreachable!("model errors have a classified kind"),
    }
}

fn live_soft_judge_operational_summary(
    cases: &[LiveSoftJudgeCase],
) -> LiveSoftJudgeOperationalSummary {
    let mut failure_classes = BTreeMap::new();
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;
    let mut total_tokens = 0u64;
    let mut successful_provider_attempts = 0u64;
    let mut fallback_runs = 0usize;
    let mut fallback_reason_counts = BTreeMap::new();
    let mut successful_runs = 0usize;
    let total_latency_ms = cases.iter().map(|case| case.latency_ms).sum();

    for case in cases {
        if let Some(failure) = &case.failure {
            *failure_classes.entry(failure.failure_class).or_insert(0) += 1;
            continue;
        }
        successful_runs += 1;
        if let Some(usage) = &case.usage {
            input_tokens += usage.input_tokens.unwrap_or(0);
            output_tokens += usage.output_tokens.unwrap_or(0);
            total_tokens += observed_total_tokens(usage).unwrap_or(0);
        }
        let provider_attempts = case.provider_attempts.unwrap_or(0);
        successful_provider_attempts += provider_attempts as u64;
        let fallback_reason = case
            .fallback_reason
            .unwrap_or(SoftJudgeFallbackReason::NotNeeded);
        *fallback_reason_counts.entry(fallback_reason).or_insert(0) += 1;
        if fallback_reason != SoftJudgeFallbackReason::NotNeeded {
            fallback_runs += 1;
        }
    }

    LiveSoftJudgeOperationalSummary {
        attempted_runs: cases.len(),
        successful_runs,
        failed_runs: cases.len().saturating_sub(successful_runs),
        failure_classes,
        input_tokens,
        output_tokens,
        total_tokens,
        successful_provider_attempts,
        fallback_runs,
        fallback_rate: (successful_runs > 0).then(|| fallback_runs as f64 / successful_runs as f64),
        fallback_reason_counts,
        total_latency_ms,
    }
}

fn live_soft_judge_family_summary(
    cases: &[LiveSoftJudgeCase],
    per_trial: &[LiveSoftJudgeTrialSummary],
) -> Vec<LiveSoftJudgeFamilySummary> {
    let complete_trials = per_trial
        .iter()
        .filter(|trial| trial.operationally_complete)
        .map(|trial| trial.trial_index)
        .collect::<std::collections::BTreeSet<_>>();
    let mut summaries = BTreeMap::<SemanticDiagnosticKind, LiveSoftJudgeFamilySummary>::new();
    for case in cases
        .iter()
        .filter(|case| complete_trials.contains(&case.trial))
        .filter(|case| case.failure.is_none())
    {
        let Some(observation) = &case.observation else {
            continue;
        };
        let summary = summaries
            .entry(case.kind)
            .or_insert(LiveSoftJudgeFamilySummary {
                kind: case.kind,
                successful_runs: 0,
                findings: 0,
                no_findings: 0,
                abstentions: 0,
            });
        summary.successful_runs += 1;
        match observation.decision {
            SoftJudgeDecision::Finding => summary.findings += 1,
            SoftJudgeDecision::NoFinding => summary.no_findings += 1,
            SoftJudgeDecision::Abstain => summary.abstentions += 1,
        }
    }
    summaries.into_values().collect()
}

fn live_soft_judge_stability(
    per_trial: &[LiveSoftJudgeTrialSummary],
    requested_trials: usize,
) -> LiveSoftJudgeStability {
    let complete = per_trial
        .iter()
        .filter(|trial| trial.operationally_complete)
        .filter_map(|trial| trial.report.as_ref())
        .filter_map(|report| report.judges.first())
        .collect::<Vec<_>>();
    let complete_trials = complete.len();
    LiveSoftJudgeStability {
        requested_trials,
        complete_trials,
        incomplete_trials: requested_trials.saturating_sub(complete_trials),
        precision: scalar_distribution(complete.iter().filter_map(|metrics| metrics.precision)),
        recall: scalar_distribution(complete.iter().filter_map(|metrics| metrics.recall)),
        decision_coverage: scalar_distribution(
            complete.iter().map(|metrics| metrics.decision_coverage),
        ),
        ambiguous_abstention_rate: scalar_distribution(
            complete
                .iter()
                .filter_map(|metrics| metrics.ambiguous_abstention_rate),
        ),
        abstentions: scalar_distribution(complete.iter().map(|metrics| metrics.abstentions as f64)),
    }
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
    if !output.operational.failure_classes.is_empty() {
        println!("failure_classes: {:?}", output.operational.failure_classes);
        for case in &output.cases {
            if let Some(failure) = &case.failure {
                println!(
                    "failed_run: fixture={} trial={} class={} latency_ms={} message={}",
                    case.fixture_id,
                    case.trial,
                    failure.failure_class,
                    failure.latency_ms,
                    failure.message
                );
            }
        }
    }
}

fn print_resolution_benchmark_human(output: &ResolutionBenchmarkOutput) {
    println!(
        "resolution_cases: {} passed={} initially_unknown={} recovered_supported={} recovery_rate={:.3}",
        output.aggregate.cases,
        output.aggregate.passed_cases,
        output.aggregate.initially_unknown_cases,
        output.aggregate.recovered_supported_cases,
        output.aggregate.recovery_rate
    );
    println!(
        "terminal: refuted={} exhausted={} unavailable={} human_review={} unsafe_final_answers={} blocked_unverified_finalizations={}",
        output.aggregate.resolved_refuted_cases,
        output.aggregate.exhausted_cases,
        output.aggregate.unavailable_cases,
        output.aggregate.human_review_required_cases,
        output.aggregate.unsafe_final_answers,
        output.aggregate.blocked_unverified_finalizations
    );
    println!(
        "final_claim_coverage={:.3} attempts={} mean_attempts={:.3} added_tokens={} elapsed_ms={}",
        output.aggregate.mean_factual_claim_coverage,
        output.aggregate.total_attempts,
        output.aggregate.mean_attempts,
        output.aggregate.added_tokens,
        output.aggregate.elapsed_ms
    );
}

fn print_soft_judge_calibration_human(report: &SoftJudgeCalibrationReport) {
    println!(
        "soft_judge_cases: {} judges={}",
        report.cases,
        report.judges.len()
    );
    for metrics in &report.judges {
        println!(
            "judge: id={} model={} config={} precision={} recall={} coverage={:.3} ambiguous_abstention={} abstentions={}",
            metrics.judge.judge_id,
            metrics.judge.model_id,
            metrics.judge.configuration_id,
            format_optional_metric(metrics.precision),
            format_optional_metric(metrics.recall),
            metrics.decision_coverage,
            format_optional_metric(metrics.ambiguous_abstention_rate),
            metrics.abstentions
        );
    }
    println!(
        "agreement: comparable_pairs={} agree={} disagree={} abstain_votes={} observed={} krippendorff_alpha={}",
        report.agreement.comparable_pairs,
        report.agreement.agreeing_pairs,
        report.agreement.disagreeing_pairs,
        report.agreement.abstain_votes,
        format_optional_metric(report.agreement.observed_pairwise_agreement),
        format_optional_metric(report.agreement.krippendorff_alpha_nominal)
    );
}

fn print_live_soft_judge_human(output: &LiveSoftJudgeOutput) {
    println!(
        "live_soft_judge: provider={} model={} corpus={} attempted={} successful={} failed={}",
        output.provider,
        output.model,
        output.corpus,
        output.operational.attempted_runs,
        output.operational.successful_runs,
        output.operational.failed_runs
    );
    println!(
        "trials: requested={} complete={} incomplete={}",
        output.stability.requested_trials,
        output.stability.complete_trials,
        output.stability.incomplete_trials
    );
    if let Some(precision) = &output.stability.precision {
        println!(
            "precision_stability: mean={:.3} min={:.3} max={:.3} stddev={:.3} n={}",
            precision.mean, precision.min, precision.max, precision.stddev, precision.count
        );
    }
    if let Some(recall) = &output.stability.recall {
        println!(
            "recall_stability: mean={:.3} min={:.3} max={:.3} stddev={:.3} n={}",
            recall.mean, recall.min, recall.max, recall.stddev, recall.count
        );
    }
    if let Some(coverage) = &output.stability.decision_coverage {
        println!(
            "coverage_stability: mean={:.3} min={:.3} max={:.3} stddev={:.3} n={}",
            coverage.mean, coverage.min, coverage.max, coverage.stddev, coverage.count
        );
    }
    if let Some(ambiguous) = &output.stability.ambiguous_abstention_rate {
        println!(
            "ambiguous_abstention_stability: mean={:.3} min={:.3} max={:.3} stddev={:.3} n={}",
            ambiguous.mean, ambiguous.min, ambiguous.max, ambiguous.stddev, ambiguous.count
        );
    }
    println!(
        "tokens: input={} output={} total={} successful_provider_attempts={} fallback_runs={} fallback_rate={} latency_ms={}",
        output.operational.input_tokens,
        output.operational.output_tokens,
        output.operational.total_tokens,
        output.operational.successful_provider_attempts,
        output.operational.fallback_runs,
        format_optional_metric(output.operational.fallback_rate),
        output.operational.total_latency_ms
    );
    if !output.operational.fallback_reason_counts.is_empty() {
        println!(
            "fallback_reasons: {}",
            output
                .operational
                .fallback_reason_counts
                .iter()
                .map(|(reason, count)| format!("{reason:?}={count}"))
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    for family in &output.families {
        println!(
            "family: kind={:?} successful={} finding={} no_finding={} abstain={}",
            family.kind,
            family.successful_runs,
            family.findings,
            family.no_findings,
            family.abstentions
        );
    }
    if !output.operational.failure_classes.is_empty() {
        println!("failure_classes: {:?}", output.operational.failure_classes);
        for case in &output.cases {
            if let Some(failure) = &case.failure {
                println!(
                    "failed_run: fixture={} trial={} kind={:?} label={:?} class={} latency_ms={} message={}",
                    case.fixture_id,
                    case.trial,
                    case.kind,
                    case.label,
                    failure.failure_class,
                    failure.latency_ms,
                    failure.message
                );
            }
        }
    }
}

fn format_optional_metric(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".into(), |value| format!("{value:.3}"))
}

fn load_cli_config(explicit: Option<&PathBuf>) -> Result<LoadedCliConfig, String> {
    let mut loaded = LoadedCliConfig::default();

    if let Some(path) = user_config_path().filter(|path| path.is_file()) {
        merge_config_file(&mut loaded, &path, "user")?;
    }

    if let Some(path) = project_config_path().filter(|path| path.is_file()) {
        merge_config_file(&mut loaded, &path, "project")?;
    }

    if let Some(path) = explicit {
        if !path.is_file() {
            return Err(format!(
                "{}: explicit config file does not exist",
                path.display()
            ));
        }
        merge_config_file(&mut loaded, path, "explicit")?;
    }

    Ok(loaded)
}

fn merge_config_file(
    loaded: &mut LoadedCliConfig,
    path: &Path,
    source: &'static str,
) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let overlay: CliFileConfig =
        serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    if overlay.schema_version != CLI_CONFIG_CONTRACT_ID {
        return Err(format!(
            "{}: unsupported config schema_version {:?}; expected {:?}",
            path.display(),
            overlay.schema_version,
            CLI_CONFIG_CONTRACT_ID
        ));
    }
    merge_cli_config(&mut loaded.config, overlay);
    loaded.sources.push(source);
    Ok(())
}

fn merge_cli_config(base: &mut CliFileConfig, overlay: CliFileConfig) {
    if overlay.run.provider.is_some() {
        base.run.provider = overlay.run.provider;
    }
    if overlay.run.model.is_some() {
        base.run.model = overlay.run.model;
    }
    if overlay.run.max_tokens.is_some() {
        base.run.max_tokens = overlay.run.max_tokens;
    }
    if overlay.run.format.is_some() {
        base.run.format = overlay.run.format;
    }
}

fn resolve_run_config(
    has_candidate: bool,
    cli_provider: Option<Provider>,
    cli_model: Option<String>,
    cli_max_tokens: Option<u32>,
    cli_format: Option<OutputFormat>,
    loaded: LoadedCliConfig,
) -> Result<ResolvedRunConfig, String> {
    if has_candidate && cli_provider.is_some() {
        return Err("choose either --candidate or --provider, not both".into());
    }

    if let Some(cli_provider) = cli_provider
        && loaded
            .config
            .run
            .provider
            .is_some_and(|configured| configured != cli_provider)
        && cli_model.is_none()
        && loaded.config.run.model.is_some()
    {
        return Err(
            "--provider overrides the configured provider; supply --model explicitly to avoid reusing a model configured for a different provider"
                .into(),
        );
    }

    let provider = if has_candidate {
        None
    } else {
        cli_provider.or(loaded.config.run.provider)
    };
    let model = cli_model.or(loaded.config.run.model);
    let max_tokens = cli_max_tokens
        .or(loaded.config.run.max_tokens)
        .unwrap_or(DEFAULT_MAX_TOKENS);
    if max_tokens == 0 {
        return Err("--max-tokens / run.max_tokens must be at least 1".into());
    }
    if provider.is_some() && model.as_deref().is_none_or(str::is_empty) {
        return Err(
            "live provider mode requires --model or a non-empty configured run.model".into(),
        );
    }

    Ok(ResolvedRunConfig {
        provider,
        model,
        max_tokens,
        format: cli_format.or(loaded.config.run.format).unwrap_or_default(),
        config_sources: loaded.sources,
    })
}

fn user_config_path() -> Option<PathBuf> {
    if let Some(home) = env::var_os("REASON_HOME") {
        return Some(PathBuf::from(home).join("config.json"));
    }
    if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("reason").join("config.json"));
    }
    if cfg!(windows)
        && let Some(app_data) = env::var_os("APPDATA")
    {
        return Some(PathBuf::from(app_data).join("reason").join("config.json"));
    }
    env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("reason")
            .join("config.json")
    })
}

fn project_config_path() -> Option<PathBuf> {
    env::current_dir()
        .ok()
        .map(|directory| directory.join(".reason").join("config.json"))
}

fn ensure_single_stdin<'a>(paths: impl IntoIterator<Item = &'a PathBuf>) -> Result<(), String> {
    let stdin_count = paths.into_iter().filter(|path| is_stdin(path)).count();
    if stdin_count <= 1 {
        Ok(())
    } else {
        Err("only one input source may use '-' (stdin) per command".into())
    }
}

fn is_stdin(path: &Path) -> bool {
    path == Path::new("-")
}

fn read_json<T: DeserializeOwned>(path: &PathBuf) -> Result<T, String> {
    let (bytes, label) = if is_stdin(path) {
        let mut bytes = Vec::new();
        io::stdin()
            .read_to_end(&mut bytes)
            .map_err(|error| format!("<stdin>: {error}"))?;
        (bytes, "<stdin>".to_string())
    } else {
        (
            fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?,
            path.display().to_string(),
        )
    };
    serde_json::from_slice(&bytes).map_err(|error| format!("{label}: {error}"))
}

fn print_product_json(command: &'static str, result: &impl Serialize) -> Result<(), String> {
    print_json(&CliEnvelope {
        schema_version: CLI_OUTPUT_SCHEMA_VERSION,
        command,
        cli_version: env!("CARGO_PKG_VERSION"),
        contracts: CliContractVersions {
            artifact: REASONING_ARTIFACT_CONTRACT_ID,
            candidate: REASONING_CANDIDATE_CONTRACT_ID,
            config: CLI_CONFIG_CONTRACT_ID,
        },
        result,
    })
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
    fn product_json_envelope_has_stable_contract_ids() {
        let value = serde_json::to_value(CliEnvelope {
            schema_version: CLI_OUTPUT_SCHEMA_VERSION,
            command: "verify",
            cli_version: env!("CARGO_PKG_VERSION"),
            contracts: CliContractVersions {
                artifact: REASONING_ARTIFACT_CONTRACT_ID,
                candidate: REASONING_CANDIDATE_CONTRACT_ID,
                config: CLI_CONFIG_CONTRACT_ID,
            },
            result: VerifyOutput {
                valid: true,
                diagnostics: &[],
            },
        })
        .unwrap();
        assert_eq!(value["schema_version"], CLI_OUTPUT_SCHEMA_VERSION);
        assert_eq!(value["command"], "verify");
        assert_eq!(
            value["contracts"]["artifact"],
            REASONING_ARTIFACT_CONTRACT_ID
        );
        assert_eq!(
            value["contracts"]["candidate"],
            REASONING_CANDIDATE_CONTRACT_ID
        );
        assert_eq!(value["contracts"]["config"], CLI_CONFIG_CONTRACT_ID);
        assert_eq!(value["result"]["valid"], true);
    }

    #[test]
    fn stdin_source_is_limited_to_one_per_command() {
        let stdin = PathBuf::from("-");
        let file = PathBuf::from("input.json");
        assert!(ensure_single_stdin([&stdin, &file]).is_ok());
        assert!(ensure_single_stdin([&stdin, &stdin]).is_err());
    }

    #[test]
    fn parses_schema_product_command() {
        let cli = Cli::try_parse_from(["reason", "schema", "artifact"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Schema {
                kind: SchemaKind::Artifact
            }
        ));
    }

    #[test]
    fn config_merge_is_fieldwise_and_higher_precedence_wins() {
        let mut base = CliFileConfig {
            schema_version: CLI_CONFIG_CONTRACT_ID.into(),
            run: RunFileConfig {
                provider: Some(Provider::Mistral),
                model: Some("base-model".into()),
                max_tokens: Some(128),
                format: None,
            },
        };
        merge_cli_config(
            &mut base,
            CliFileConfig {
                schema_version: CLI_CONFIG_CONTRACT_ID.into(),
                run: RunFileConfig {
                    provider: None,
                    model: Some("override-model".into()),
                    max_tokens: None,
                    format: Some(OutputFormat::Json),
                },
            },
        );
        assert_eq!(base.run.provider, Some(Provider::Mistral));
        assert_eq!(base.run.model.as_deref(), Some("override-model"));
        assert_eq!(base.run.max_tokens, Some(128));
        assert_eq!(base.run.format, Some(OutputFormat::Json));
    }

    #[test]
    fn recorded_candidate_ignores_configured_live_provider() {
        let resolved = resolve_run_config(
            true,
            None,
            None,
            None,
            Some(OutputFormat::Json),
            LoadedCliConfig {
                config: CliFileConfig {
                    schema_version: CLI_CONFIG_CONTRACT_ID.into(),
                    run: RunFileConfig {
                        provider: Some(Provider::Google),
                        model: Some("gemini-test".into()),
                        max_tokens: Some(256),
                        format: None,
                    },
                },
                sources: vec!["user"],
            },
        )
        .unwrap();
        assert_eq!(resolved.provider, None);
        assert_eq!(resolved.format, OutputFormat::Json);
        assert_eq!(resolved.config_sources, vec!["user"]);
    }

    #[test]
    fn live_provider_requires_explicit_or_configured_model() {
        let error = resolve_run_config(
            false,
            Some(Provider::Google),
            None,
            None,
            None,
            LoadedCliConfig::default(),
        )
        .unwrap_err();
        assert!(error.contains("requires --model"));
    }

    #[test]
    fn provider_override_requires_model_when_config_provider_changes() {
        let error = resolve_run_config(
            false,
            Some(Provider::Google),
            None,
            None,
            None,
            LoadedCliConfig {
                config: CliFileConfig {
                    schema_version: CLI_CONFIG_CONTRACT_ID.into(),
                    run: RunFileConfig {
                        provider: Some(Provider::Mistral),
                        model: Some("ministral-8b-latest".into()),
                        max_tokens: None,
                        format: None,
                    },
                },
                sources: vec!["project"],
            },
        )
        .unwrap_err();
        assert!(error.contains("supply --model explicitly"));
    }

    #[test]
    fn config_schema_rejects_secret_like_unknown_fields() {
        let text = r#"{
          "schema_version": "reason-config-v1",
          "run": {"model": "m", "api_key": "secret"}
        }"#;
        assert!(serde_json::from_str::<CliFileConfig>(text).is_err());
    }

    #[test]
    fn parses_config_schema_product_command() {
        let cli = Cli::try_parse_from(["reason", "schema", "config"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Schema {
                kind: SchemaKind::Config
            }
        ));
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
    fn parses_live_soft_judge_concurrency() {
        let cli = Cli::try_parse_from([
            "reason",
            "eval-judges",
            "fixtures/semantic-judges-holdout-v2",
            "--provider",
            "nvidia",
            "--model",
            "nvidia/nemotron-3.5-lightning-30b-a3b",
            "--concurrency",
            "4",
        ])
        .unwrap();
        match cli.command {
            Command::EvalJudges { concurrency, .. } => assert_eq!(concurrency, 4),
            _ => panic!("expected eval-judges command"),
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

    fn soft_judge_case(
        fixture_id: &str,
        trial: usize,
        decision: SoftJudgeDecision,
        provider_attempts: u32,
    ) -> LiveSoftJudgeCase {
        LiveSoftJudgeCase {
            fixture_id: fixture_id.into(),
            trial,
            kind: SemanticDiagnosticKind::Contradiction,
            label: CalibrationLabel::Negative,
            observation: Some(SoftJudgeObservation {
                judge: SoftJudgeIdentity {
                    judge_id: "judge".into(),
                    model_id: "model".into(),
                    configuration_id: "config".into(),
                },
                request_id: fixture_id.into(),
                decision,
                finding: None,
            }),
            provider_model: Some("model".into()),
            usage: Some(ModelUsage {
                input_tokens: Some(10),
                output_tokens: Some(2),
                total_tokens: Some(12),
            }),
            provider_attempts: Some(provider_attempts),
            fallback_reason: Some(if provider_attempts > 1 {
                SoftJudgeFallbackReason::InvalidPrimaryStructuredOutput
            } else {
                SoftJudgeFallbackReason::NotNeeded
            }),
            latency_ms: 5,
            failure: None,
        }
    }

    #[test]
    fn live_soft_judge_operational_summary_reports_fallback_rate() {
        let cases = vec![
            soft_judge_case("a", 0, SoftJudgeDecision::NoFinding, 1),
            soft_judge_case("b", 0, SoftJudgeDecision::NoFinding, 2),
        ];
        let summary = live_soft_judge_operational_summary(&cases);
        assert_eq!(summary.successful_runs, 2);
        assert_eq!(summary.successful_provider_attempts, 3);
        assert_eq!(summary.fallback_runs, 1);
        assert_eq!(summary.fallback_rate, Some(0.5));
        assert_eq!(
            summary
                .fallback_reason_counts
                .get(&SoftJudgeFallbackReason::NotNeeded),
            Some(&1)
        );
        assert_eq!(
            summary
                .fallback_reason_counts
                .get(&SoftJudgeFallbackReason::InvalidPrimaryStructuredOutput),
            Some(&1)
        );
    }

    #[test]
    fn live_soft_judge_family_summary_excludes_incomplete_trials() {
        let cases = vec![
            soft_judge_case("complete", 0, SoftJudgeDecision::NoFinding, 1),
            soft_judge_case("incomplete", 1, SoftJudgeDecision::Abstain, 1),
        ];
        let per_trial = vec![
            LiveSoftJudgeTrialSummary {
                trial_index: 0,
                expected_cases: 1,
                successful_cases: 1,
                operationally_complete: true,
                report: None,
            },
            LiveSoftJudgeTrialSummary {
                trial_index: 1,
                expected_cases: 2,
                successful_cases: 1,
                operationally_complete: false,
                report: None,
            },
        ];
        let families = live_soft_judge_family_summary(&cases, &per_trial);
        assert_eq!(families.len(), 1);
        assert_eq!(families[0].successful_runs, 1);
        assert_eq!(families[0].no_findings, 1);
        assert_eq!(families[0].abstentions, 0);
    }

    #[test]
    fn live_soft_judge_stability_excludes_incomplete_trials() {
        let fixture_dir =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/semantic-judges");
        let report = run_soft_judge_calibration_suite(&fixture_dir).unwrap();
        let per_trial = vec![
            LiveSoftJudgeTrialSummary {
                trial_index: 0,
                expected_cases: 9,
                successful_cases: 9,
                operationally_complete: true,
                report: Some(report.clone()),
            },
            LiveSoftJudgeTrialSummary {
                trial_index: 1,
                expected_cases: 9,
                successful_cases: 8,
                operationally_complete: false,
                report: None,
            },
            LiveSoftJudgeTrialSummary {
                trial_index: 2,
                expected_cases: 9,
                successful_cases: 9,
                operationally_complete: true,
                report: Some(report),
            },
        ];
        let stability = live_soft_judge_stability(&per_trial, 3);
        assert_eq!(stability.complete_trials, 2);
        assert_eq!(stability.incomplete_trials, 1);
        assert_eq!(stability.precision.as_ref().unwrap().count, 2);
        assert_eq!(stability.recall.as_ref().unwrap().count, 2);
        assert_eq!(stability.decision_coverage.as_ref().unwrap().count, 2);
        assert_eq!(
            stability.ambiguous_abstention_rate.as_ref().unwrap().count,
            2
        );
        assert_eq!(stability.abstentions.as_ref().unwrap().count, 2);
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
