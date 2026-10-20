//! Preserve attempt usage independently of whether a response decodes successfully.
use super::{GenerationFailure, GenerationObservation, add_usage};
use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelExecutionBudget, ModelExecutionTelemetrySnapshot, ModelRequest,
    ModelResponse, ModelUsage,
};
use std::{future::Future, pin::Pin, sync::Mutex};

struct Observed {
    attempts: u32,
    usage: ModelUsage,
}

pub(super) struct RecordingAdapter<'a> {
    inner: &'a dyn ModelAdapter,
    observed: Mutex<Observed>,
}

impl<'a> RecordingAdapter<'a> {
    pub(super) fn new(inner: &'a dyn ModelAdapter) -> Self {
        Self {
            inner,
            observed: Mutex::new(Observed {
                attempts: 0,
                usage: ModelUsage {
                    input_tokens: Some(0),
                    output_tokens: Some(0),
                    total_tokens: Some(0),
                },
            }),
        }
    }

    pub(super) fn finish<T>(
        &self,
        result: Result<(T, GenerationObservation), GenerationFailure>,
    ) -> Result<(T, GenerationObservation), GenerationFailure> {
        let observed = self.observed.lock().unwrap();
        match result {
            Ok((value, mut observation)) => {
                observation.provider_attempts = observed.attempts;
                observation.usage = observed.usage.clone();
                Ok((value, observation))
            }
            Err(mut failure) => {
                failure.provider_attempts = observed.attempts;
                failure.usage = (observed.attempts > 0
                    && [
                        observed.usage.input_tokens,
                        observed.usage.output_tokens,
                        observed.usage.total_tokens,
                    ]
                    .iter()
                    .any(Option::is_some))
                .then(|| Box::new(observed.usage.clone()));
                Err(failure)
            }
        }
    }
}

impl ModelAdapter for RecordingAdapter<'_> {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>> {
        Box::pin(async move {
            let result = self.inner.generate(request).await;
            let mut observed = self.observed.lock().unwrap();
            match &result {
                Ok(response) => {
                    observed.attempts =
                        observed.attempts.saturating_add(response.provider_attempts);
                    observed.usage = add_usage(&observed.usage, &response.usage);
                }
                Err(error) => {
                    observed.attempts = observed.attempts.saturating_add(error.provider_attempts);
                    // A failed HTTP attempt has no reported usage. A local
                    // zero-attempt capability refusal consumes no tokens.
                    if error.provider_attempts > 0 {
                        observed.usage = ModelUsage::default();
                    }
                }
            }
            result
        })
    }
    fn configure_execution_budget(&self, budget: Option<ModelExecutionBudget>) {
        self.inner.configure_execution_budget(budget);
    }
    fn execution_telemetry_snapshot(&self) -> Option<ModelExecutionTelemetrySnapshot> {
        self.inner.execution_telemetry_snapshot()
    }
}
