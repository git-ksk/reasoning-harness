use std::{collections::BTreeMap, fs, path::PathBuf, process::ExitCode, time::Instant};

use clap::{Parser, Subcommand, ValueEnum};
use reasoning_harness_core::{
    AdversarialDiscoveryPass, BenchmarkCaseResult, BenchmarkComparison, BenchmarkFixture,
    HarnessInput, ModelAdapter, ModelError, ModelErrorKind, ModelUsage, ReasoningArtifact,
    ReasoningCandidate, StrictAcceptancePolicy, StructuredFactConflictDetector,
    StructuredFactVerifier, TrustedVerificationPass, VerificationPass, VerificationReceipt,
    aggregate_benchmark, build_candidate_json_fallback_request, build_candidate_request, evaluate,
    evaluate_benchmark_fixture, frameworks::five_whys::FiveWhysRestatementPass, run_harness,
    validate_artifact,
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

#[derive(Debug, Serialize)]
struct ObservedBenchmarkCase {
    fixture_id: String,
    trial: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<BenchmarkCaseResult>,
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
struct BenchmarkOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    comparison: BenchmarkComparison,
    operational: OperationalSummary,
    cases: Vec<ObservedBenchmarkCase>,
}

#[derive(Debug, Clone)]
struct BenchmarkRunConfig<'a> {
    provider: Option<Provider>,
    model: &'a str,
    max_tokens: u32,
    seed: Option<u64>,
    trials: usize,
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
            let passes: Vec<Box<dyn reasoning_harness_core::Pass>> = vec![
                Box::new(AdversarialDiscoveryPass::new(vec![Box::new(
                    StructuredFactConflictDetector,
                )])),
                Box::new(VerificationPass::new(vec![Box::new(
                    StructuredFactVerifier,
                )])),
                Box::new(TrustedVerificationPass::new(receipts)),
                Box::new(FiveWhysRestatementPass),
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
                if provider.is_some() || trials != 1 {
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
    if config.provider.is_none() && config.trials != 1 {
        return Err("offline recorded fixtures support exactly one trial".into());
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
    let generator = config
        .provider
        .map(|provider| LiveGenerator::from_provider(provider, config.model))
        .transpose()?;
    let total_runs = fixtures
        .len()
        .checked_mul(config.trials)
        .ok_or("fixture/trial count overflowed usize")?;
    let mut completed_runs = 0usize;
    let mut observed = Vec::new();
    for fixture in fixtures {
        for trial in 0..config.trials {
            if let Some(provider) = config.provider {
                eprintln!(
                    "[benchmark] provider={} model={} [{}/{}] fixture={} trial={}",
                    provider_name(provider),
                    config.model,
                    completed_runs + 1,
                    total_runs,
                    fixture.id,
                    trial + 1
                );
            }
            let trial_seed = match config.seed {
                Some(value) => Some(
                    value
                        .checked_add(trial as u64)
                        .ok_or("trial seed overflowed u64")?,
                ),
                None => None,
            };
            let (candidate, mut generation) = if let Some(generator) = &generator {
                let generated = generator
                    .generate(&fixture.input, config.max_tokens, trial_seed, config.model)
                    .await;
                match generated {
                    Ok((candidate, observation)) => (candidate, Some(observation)),
                    Err(failure) => {
                        observed.push(ObservedBenchmarkCase {
                            fixture_id: fixture.id.clone(),
                            trial,
                            result: None,
                            generation: None,
                            failure: Some(failure),
                        });
                        completed_runs += 1;
                        continue;
                    }
                }
            } else {
                (fixture.recorded_candidate.clone(), None)
            };

            if let Some(observation) = generation.as_mut() {
                observation.cost_usd = estimate_cost(
                    &observation.usage,
                    config.input_cost_per_million,
                    config.output_cost_per_million,
                );
            }
            observed.push(ObservedBenchmarkCase {
                fixture_id: fixture.id.clone(),
                trial,
                result: Some(evaluate_benchmark_fixture(&fixture, candidate)),
                generation,
                failure: None,
            });
            completed_runs += 1;
        }
    }

    let results: Vec<BenchmarkCaseResult> = observed
        .iter()
        .filter_map(|case| case.result.clone())
        .collect();
    let comparison = aggregate_benchmark(&results);
    let operational = operational_summary(&observed);
    Ok(BenchmarkOutput {
        provider: config.provider.map(provider_name),
        model: config.provider.map(|_| config.model.to_string()),
        comparison,
        operational,
        cases: observed,
    })
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
    fn operational_summary_preserves_failed_generation_class() {
        let cases = vec![ObservedBenchmarkCase {
            fixture_id: "fixture-a".into(),
            trial: 0,
            result: None,
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
