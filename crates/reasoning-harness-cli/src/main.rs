use std::{fs, path::PathBuf, process::ExitCode, time::Instant};

use clap::{Parser, Subcommand, ValueEnum};
use reasoning_harness_core::{
    HarnessInput, ModelAdapter, ModelUsage, ReasoningArtifact, ReasoningCandidate,
    StrictAcceptancePolicy, build_candidate_request, evaluate, run_harness, validate_artifact,
};
use reasoning_harness_providers::MistralAdapter;
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
    /// Compute reproducible metrics for a finalized ReasoningArtifact JSON file.
    Eval {
        artifact: PathBuf,
        #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
}

#[derive(Debug, Serialize)]
struct VerifyOutput<'a> {
    valid: bool,
    diagnostics: &'a [reasoning_harness_core::Diagnostic],
}

#[derive(Debug, Serialize)]
struct GenerationObservation {
    provider: &'static str,
    model: String,
    usage: ModelUsage,
    latency_ms: u128,
}

#[derive(Debug, Serialize)]
struct RunOutput {
    candidate: ReasoningCandidate,
    outcome: reasoning_harness_core::HarnessOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation: Option<GenerationObservation>,
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
            provider,
            model,
            max_tokens,
            seed,
            format,
        } => {
            let input: HarnessInput = read_json(&input)?;
            let (candidate, generation) = match (candidate, provider) {
                (Some(path), None) => (read_json(&path)?, None),
                (None, Some(Provider::Mistral)) => {
                    let adapter =
                        MistralAdapter::from_env(model).map_err(|error| error.to_string())?;
                    let request = build_candidate_request(&input, Some(max_tokens), seed)
                        .map_err(|error| error.to_string())?;
                    let started = Instant::now();
                    let response = adapter
                        .generate(request)
                        .await
                        .map_err(|error| error.to_string())?;
                    let latency_ms = started.elapsed().as_millis();
                    let candidate: ReasoningCandidate = serde_json::from_str(&response.text)
                        .map_err(|error| {
                            format!("provider returned invalid candidate JSON: {error}")
                        })?;
                    let observation = GenerationObservation {
                        provider: "mistral",
                        model: response.model,
                        usage: response.usage,
                        latency_ms,
                    };
                    (candidate, Some(observation))
                }
                (Some(_), Some(_)) => {
                    return Err("choose either --candidate or --provider, not both".into());
                }
                (None, None) => {
                    return Err("reason run requires either --candidate or --provider".into());
                }
            };

            let outcome = run_harness(input, candidate.clone(), &[], &StrictAcceptancePolicy)
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
        Command::Eval { artifact, format } => {
            let artifact: ReasoningArtifact = read_json(&artifact)?;
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
