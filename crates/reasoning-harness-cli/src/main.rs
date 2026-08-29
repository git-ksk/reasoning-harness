use std::{fs, path::PathBuf, process::ExitCode};

use clap::{Parser, Subcommand};
use reasoning_harness_core::{ReasoningArtifact, evaluate, validate_artifact};

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

#[derive(Debug, Subcommand)]
enum Command {
    /// Deterministically validate a ReasoningArtifact JSON file.
    Verify { artifact: PathBuf },
    /// Compute reproducible harness metrics for a ReasoningArtifact JSON file.
    Eval { artifact: PathBuf },
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
        Command::Verify { artifact } => {
            let artifact = read_artifact(&artifact)?;
            let report = validate_artifact(&artifact);
            if report.is_ok() {
                println!("valid");
                Ok(())
            } else {
                for diagnostic in report.diagnostics {
                    eprintln!("{}: {}", diagnostic.code, diagnostic.message);
                }
                Err("artifact validation failed".into())
            }
        }
        Command::Eval { artifact } => {
            let artifact = read_artifact(&artifact)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&evaluate(&artifact))
                    .map_err(|error| error.to_string())?
            );
            Ok(())
        }
    }
}

fn read_artifact(path: &PathBuf) -> Result<ReasoningArtifact, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}
