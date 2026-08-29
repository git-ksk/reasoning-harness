use serde::Serialize;
use thiserror::Error;

use crate::{
    AcceptancePolicy, HarnessInput, ReasoningArtifact, ReasoningCandidate, ValidationReport,
    Verdict, materialize_candidate, validate_artifact,
};

pub trait Pass {
    fn name(&self) -> &'static str;
    fn apply(&self, artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError>;
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HarnessOutcome {
    pub verdict: Verdict,
    pub artifact: ReasoningArtifact,
}

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("input reasoning state is invalid: {diagnostics:?}")]
    InvalidInput { diagnostics: Vec<String> },
    #[error("pass {pass} failed: {message}")]
    Pass { pass: &'static str, message: String },
    #[error("pass {pass} produced invalid reasoning state: {diagnostics:?}")]
    InvalidState {
        pass: &'static str,
        diagnostics: Vec<String>,
    },
}

pub fn run_passes(
    mut artifact: ReasoningArtifact,
    passes: &[Box<dyn Pass>],
) -> Result<ReasoningArtifact, HarnessError> {
    validate_input(&artifact)?;

    for pass in passes {
        artifact = pass.apply(artifact)?;
        let report = validate_artifact(&artifact);
        if !report.is_ok() {
            return Err(HarnessError::InvalidState {
                pass: pass.name(),
                diagnostics: diagnostics(&report),
            });
        }
    }
    Ok(artifact)
}

pub fn run_harness(
    input: HarnessInput,
    candidate: ReasoningCandidate,
    passes: &[Box<dyn Pass>],
    policy: &dyn AcceptancePolicy,
) -> Result<HarnessOutcome, HarnessError> {
    let artifact = materialize_candidate(input, candidate);
    let artifact = run_passes(artifact, passes)?;
    Ok(HarnessOutcome {
        verdict: policy.decide(&artifact),
        artifact,
    })
}

fn validate_input(artifact: &ReasoningArtifact) -> Result<(), HarnessError> {
    let report = validate_artifact(artifact);
    if report.is_ok() {
        Ok(())
    } else {
        Err(HarnessError::InvalidInput {
            diagnostics: diagnostics(&report),
        })
    }
}

fn diagnostics(report: &ValidationReport) -> Vec<String> {
    report
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
        .collect()
}
