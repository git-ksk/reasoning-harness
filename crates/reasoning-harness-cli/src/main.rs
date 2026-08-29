use std::{fs, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand, ValueEnum};
use reasoning_harness_core::{
    ReasoningArtifact, StrictAcceptancePolicy, evaluate, run_harness, validate_artifact,
};
use serde::Serialize;

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

#[derive(Debug, Subcommand)]
enum Command {
    /// Execute the deterministic harness over a candidate ReasoningArtifact.
    Run {
        artifact: PathBuf,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Deterministically validate a ReasoningArtifact JSON file.
    Verify {
        artifact: PathBuf,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Compute reproducible harness metrics for a ReasoningArtifact JSON file.
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

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Run { artifact, format } => {
            let artifact = read_artifact(&artifact)?;
            let outcome = run_harness(artifact, &[], &StrictAcceptancePolicy)
                .map_err(|error| error.to_string())?;
            match format {
                OutputFormat::Human => println!("{:?}", outcome.verdict),
                OutputFormat::Json => print_json(&outcome)?,
            }
            Ok(())
        }
        Command::Verify { artifact, format } => {
            let artifact = read_artifact(&artifact)?;
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
            let artifact = read_artifact(&artifact)?;
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

fn read_artifact(path: &PathBuf) -> Result<ReasoningArtifact, String> {
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
