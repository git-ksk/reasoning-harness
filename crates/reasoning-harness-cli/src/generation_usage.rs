//! Preserve attempt usage independently of whether a response decodes successfully.
use super::usage_budget::GenerationLimits;
use super::{GenerationFailure, GenerationObservation, add_usage};
use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelExecutionBudget, ModelExecutionTelemetrySnapshot, ModelRequest,
    ModelResponse, ModelUsage,
};
use std::{future::Future, pin::Pin, sync::Mutex};

struct Observed {
    attempts: u32,
    usage: ModelUsage,
    denied: Option<(&'static str, String)>,
}

pub(super) struct RecordingAdapter<'a> {
    inner: &'a dyn ModelAdapter,
    observed: Mutex<Observed>,
    limits: GenerationLimits,
}

impl<'a> RecordingAdapter<'a> {
    pub(super) fn new(inner: &'a dyn ModelAdapter) -> Self {
        Self::with_limits(inner, GenerationLimits::default())
    }

    pub(super) fn with_limits(inner: &'a dyn ModelAdapter, limits: GenerationLimits) -> Self {
        Self {
            inner,
            limits,
            observed: Mutex::new(Observed {
                attempts: 0,
                denied: None,
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
                if let Some((class, message)) = &observed.denied {
                    failure.failure_class = class;
                    failure.message = message.clone();
                }
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
        mut request: ModelRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>> {
        Box::pin(async move {
            {
                let mut observed = self.observed.lock().unwrap();
                let mut denial = None;
                if let Some(limit) = self.limits.output {
                    match observed.usage.output_tokens {
                        None => denial = Some(("usage_budget_unmeasurable", "provider did not report output-token usage required before another request".to_string())),
                        Some(used) if used >= limit => denial = Some(("usage_budget_exceeded", "output-token budget exhausted before another request".to_string())),
                        Some(used) => {
                            let remaining = u32::try_from(limit - used).unwrap_or(u32::MAX);
                            request.max_tokens = Some(request.max_tokens.unwrap_or(remaining).min(remaining));
                        }
                    }
                }
                if denial.is_none()
                    && let Some(limit) = self.limits.total
                {
                    match observed.usage.total_tokens {
                        None => denial = Some(("usage_budget_unmeasurable", "provider did not report total-token usage required before another request".to_string())),
                        Some(used) if used >= limit => denial = Some(("usage_budget_exceeded", "total-token budget exhausted before another request".to_string())),
                        Some(_) => {}
                    }
                }
                if let Some((class, message)) = denial {
                    observed.denied = Some((class, message.clone()));
                    return Err(ModelError {
                        kind: reasoning_harness_core::ModelErrorKind::Protocol,
                        message,
                        provider_attempts: 0,
                    });
                }
            }
            let result = self.inner.generate(request).await;
            let mut observed = self.observed.lock().unwrap();
            match &result {
                Ok(response) => {
                    observed.attempts =
                        observed.attempts.saturating_add(response.provider_attempts);
                    // A provider adapter may retry internally, where individual
                    // attempt usage is not exposed. Do not claim the final
                    // response accounts for all attempts when enforcing a
                    // cumulative token limit.
                    if response.provider_attempts > 1
                        && (self.limits.output.is_some() || self.limits.total.is_some())
                    {
                        observed.usage = ModelUsage::default();
                    } else {
                        observed.usage = add_usage(&observed.usage, &response.usage);
                    }
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
