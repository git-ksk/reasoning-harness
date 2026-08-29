use thiserror::Error;

use crate::{ReasoningArtifact, validate_artifact};

pub trait Pass {
    fn name(&self) -> &'static str;
    fn apply(&self, artifact: ReasoningArtifact) -> Result<ReasoningArtifact, HarnessError>;
}

#[derive(Debug, Error)]
pub enum HarnessError {
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
    for pass in passes {
        artifact = pass.apply(artifact)?;
        let report = validate_artifact(&artifact);
        if !report.is_ok() {
            return Err(HarnessError::InvalidState {
                pass: pass.name(),
                diagnostics: report
                    .diagnostics
                    .into_iter()
                    .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
                    .collect(),
            });
        }
    }
    Ok(artifact)
}
