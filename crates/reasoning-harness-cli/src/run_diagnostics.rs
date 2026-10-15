//! Failure-only observations. These never become an answer or a session checkpoint.
use crate::usage_budget::UsageReport;
use serde::Serialize;
use std::sync::{Arc, Mutex};

pub(super) type SharedRunDiagnostics = Arc<Mutex<RunDiagnostics>>;

#[derive(Debug, Default)]
pub(super) struct RunDiagnostics {
    pub usage: Option<UsageReport>,
    pub trace_written: Option<bool>,
    pub pending_adapter_generations: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct PartialRunObservation {
    pub complete: bool,
    pub usage_is_lower_bound: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_written: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pending_adapter_generations: Option<usize>,
}

impl RunDiagnostics {
    pub fn partial(&self, failure_class: &str) -> Option<Box<PartialRunObservation>> {
        if self.usage.is_none() && self.trace_written.is_none() {
            return None;
        }
        let lower_bound = failure_class == "cancelled"
            || self
                .pending_adapter_generations
                .is_some_and(|count| count > 0);
        let mut usage = self.usage.clone();
        if lower_bound && let Some(report) = usage.as_mut() {
            // A dropped in-flight request may have been billed. Do not report zero
            // or a previous completed request's tokens as the whole run's total.
            report.run.input_tokens = None;
            report.run.output_tokens = None;
            report.run.total_tokens = None;
            if let Some(session) = report.session.as_mut() {
                session.input_tokens = None;
                session.output_tokens = None;
                session.total_tokens = None;
            }
            report.estimated_model_cost_usd = None;
        }
        Some(Box::new(PartialRunObservation {
            complete: false,
            usage_is_lower_bound: lower_bound,
            usage,
            trace_written: self.trace_written,
            pending_adapter_generations: self.pending_adapter_generations,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use reasoning_harness_core::ModelResponse;
    use std::{
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicUsize, Ordering},
    };

    struct FakeModel {
        calls: Arc<AtomicUsize>,
        usage: ModelUsage,
        failure: bool,
        pending: bool,
    }
    impl ModelAdapter for FakeModel {
        fn generate<'a>(
            &'a self,
            _: ModelRequest,
        ) -> Pin<Box<dyn Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async move {
                if self.pending {
                    return std::future::pending().await;
                }
                if self.failure {
                    return Err(
                        ModelError::new(ModelErrorKind::Quota, "daily quota reached")
                            .with_provider_attempts(2),
                    );
                }
                Ok(ModelResponse {
                    text: r#"{"claims":[{"id":"c1","statement":"The value is unverified.","proposed_state":"unknown","proposition":{"key":"test.value","value":"yes"},"evidence_ids":[]}],"inferences":[]}"#.into(),
                    model: "diagnostic-test".into(), usage: self.usage.clone(), provider_attempts: 1, finish_reason: Some("stop".into()),
                })
            })
        }
    }
    fn trace_path(label: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "reason-failure-diagnostics-{label}-{}-{}.json",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    fn args(path: Option<PathBuf>) -> NaturalArgs {
        NaturalArgs {
            task: Some("Is the test value known?".into()),
            provider: Some(Provider::Groq),
            model: Some("diagnostic-test".into()),
            no_config: true,
            format: Some(OutputFormat::Json),
            diagnostic_trace: path,
            max_model_calls: Some(1),
            ..NaturalArgs::default()
        }
    }
    fn model(
        calls: Arc<AtomicUsize>,
        usage: ModelUsage,
        failure: bool,
        pending: bool,
    ) -> LiveGenerator {
        LiveGenerator::Test(DiagnosticTestAdapter(Box::new(FakeModel {
            calls,
            usage,
            failure,
            pending,
        })))
    }
    async fn execute(
        args: NaturalArgs,
        generator: LiveGenerator,
    ) -> Result<NaturalOutput, CliError> {
        let context = SharedRunDiagnostics::default();
        execute_natural_inner(
            args,
            None,
            SubprocessCancellation::default(),
            context.clone(),
            |_, _| Ok(generator),
        )
        .await
        .map_err(|mut error| {
            error.partial_diagnostics = context.lock().unwrap().partial(error.failure_class);
            error
        })
    }
    fn known_usage() -> ModelUsage {
        ModelUsage {
            input_tokens: Some(20),
            output_tokens: Some(10),
            total_tokens: Some(30),
        }
    }
    #[tokio::test]
    async fn budget_stop_preserves_the_actual_response_usage_and_partial_trace() {
        let path = trace_path("budget");
        let calls = Arc::new(AtomicUsize::new(0));
        let error = execute(
            args(Some(path.clone())),
            model(calls.clone(), known_usage(), false, false),
        )
        .await
        .unwrap_err();
        assert_eq!(error.failure_class, "usage_budget_exceeded");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "budget must still stop a second generation"
        );
        let envelope =
            serde_json::to_value(ProductFailureOutput::from_error("ask", &error)).unwrap();
        assert_eq!(envelope["status"], "failed");
        assert!(envelope.get("final_outcome").is_none());
        assert_eq!(envelope["partial_diagnostics"]["complete"], false);
        assert_eq!(
            envelope["partial_diagnostics"]["usage"]["run"]["model_calls"],
            1
        );
        assert_eq!(
            envelope["partial_diagnostics"]["usage"]["run"]["total_tokens"],
            30
        );
        assert_eq!(envelope["partial_diagnostics"]["trace_written"], true);
        let trace: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(trace["completion_status"], "partial");
        assert!(
            trace["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "model_response")
        );
        assert!(
            !trace["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "finalization_state")
        );
        fs::remove_file(path).unwrap();
    }
    #[tokio::test]
    async fn missing_provider_usage_remains_unmeasurable_and_is_not_filled_with_zero() {
        let path = trace_path("missing");
        let calls = Arc::new(AtomicUsize::new(0));
        let mut input = args(Some(path.clone()));
        input.max_total_tokens = Some(1000);
        let error = execute(
            input,
            model(calls.clone(), ModelUsage::default(), false, false),
        )
        .await
        .unwrap_err();
        assert_eq!(error.failure_class, "usage_budget_unmeasurable");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let partial = error.partial_diagnostics.unwrap();
        let report = partial.usage.unwrap();
        assert_eq!(report.run.model_calls, 1);
        assert_eq!(report.run.total_tokens, None);
        assert_eq!(partial.trace_written, Some(true));
        fs::remove_file(path).unwrap();
    }
    #[tokio::test]
    async fn initial_provider_failure_retains_attempts_without_a_candidate_or_false_zero_tokens() {
        let calls = Arc::new(AtomicUsize::new(0));
        let error = execute(args(None), model(calls.clone(), known_usage(), true, false))
            .await
            .unwrap_err();
        assert_eq!(error.failure_class, "quota");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let report = error.partial_diagnostics.unwrap().usage.unwrap();
        assert_eq!(report.run.provider_attempts, 2);
        assert_eq!(report.run.total_tokens, None);
    }
    #[tokio::test]
    async fn cancelling_inflight_generation_flushes_trace_and_marks_usage_as_a_lower_bound() {
        let path = trace_path("cancel");
        let calls = Arc::new(AtomicUsize::new(0));
        let context = SharedRunDiagnostics::default();
        let cancellation = progress::CancellationRun::test_instance();
        let generator = model(calls.clone(), known_usage(), false, true);
        let operation = await_cancellable_operation(
            &cancellation,
            execute_natural_inner(
                args(Some(path.clone())),
                None,
                cancellation.subprocess_token(),
                context.clone(),
                |_, _| Ok(generator),
            ),
        );
        let cancel = async {
            while calls.load(Ordering::SeqCst) == 0 {
                tokio::task::yield_now().await;
            }
            cancellation.cancel_for_test();
        };
        let (result, ()) = tokio::join!(operation, cancel);
        let error = result.unwrap_err();
        assert_eq!(error.failure_class, "cancelled");
        let partial = context
            .lock()
            .unwrap()
            .partial(error.failure_class)
            .unwrap();
        assert!(partial.usage_is_lower_bound);
        assert_eq!(partial.pending_adapter_generations, Some(1));
        assert_eq!(partial.usage.unwrap().run.total_tokens, None);
        assert_eq!(partial.trace_written, Some(true));
        let trace: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(trace["completion_status"], "partial");
        assert_eq!(
            trace["events"].as_array().unwrap().len(),
            1,
            "no invented response for the cancelled request"
        );
        fs::remove_file(path).unwrap();
    }
    #[tokio::test]
    async fn trace_write_failure_does_not_replace_the_original_budget_error() {
        let parent = trace_path("not-a-directory");
        fs::write(&parent, "blocking file").unwrap();
        let calls = Arc::new(AtomicUsize::new(0));
        let error = execute(
            args(Some(parent.join("trace.json"))),
            model(calls, known_usage(), false, false),
        )
        .await
        .unwrap_err();
        assert_eq!(error.failure_class, "usage_budget_exceeded");
        assert_eq!(
            error.partial_diagnostics.unwrap().trace_written,
            Some(false)
        );
        fs::remove_file(parent).unwrap();
    }
    #[test]
    fn completed_trace_flushes_once_with_the_same_event_contract() {
        let path = trace_path("complete");
        let context = SharedRunDiagnostics::default();
        {
            let mut trace = DiagnosticTraceRecorder::new("groq", "test", None);
            trace.preserve_on_exit(path.clone(), context.clone());
            trace.mark_completed();
        }
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(value["completion_status"], "completed");
        assert_eq!(
            value["contract"],
            diagnostic_trace::NATURAL_DIAGNOSTIC_TRACE_CONTRACT_ID
        );
        assert_eq!(context.lock().unwrap().trace_written, Some(true));
        fs::remove_file(path).unwrap();
    }
}
