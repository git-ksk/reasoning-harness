//! Provider-free planning through the same natural-input and candidate builder.
use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelRequest, ModelResponse,
};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

#[derive(Clone, Default)]
pub(crate) struct RequestPlanner {
    first: Arc<Mutex<Option<ModelRequest>>>,
    identity: Arc<Mutex<Option<(String, String)>>>,
}

impl ModelAdapter for RequestPlanner {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>> {
        Box::pin(async move {
            let mut first = self.first.lock().expect("request planner lock poisoned");
            if first.is_none() {
                *first = Some(request);
            }
            // No credentials, HTTP, resolver, model response or final answer.
            Err(ModelError::new(
                ModelErrorKind::Credentials,
                "provider-free request planning complete",
            ))
        })
    }
}

pub(crate) async fn write_plan(
    mut args: crate::NaturalArgs,
    destination: &std::path::Path,
) -> Result<(), crate::CliError> {
    let planner = RequestPlanner::default();
    let input = planner.clone();
    let question = args.task.clone().unwrap_or_default();
    args.evaluation_request_plan = None;
    args.diagnostic_trace = None;
    let cancellation = reasoning_harness_providers::SubprocessCancellation::default();
    let result = crate::execute_natural_inner(
        args,
        None,
        cancellation,
        crate::run_diagnostics::SharedRunDiagnostics::default(),
        move |provider, model| {
            *input
                .identity
                .lock()
                .expect("request planner identity poisoned") =
                Some((crate::provider_name(provider).to_owned(), model.to_owned()));
            Ok(crate::LiveGenerator::Planning {
                adapter: input,
                provider: crate::provider_name(provider),
            })
        },
    )
    .await;
    let request = planner
        .first
        .lock()
        .expect("request planner lock poisoned")
        .take();
    let request = match request {
        Some(value) => value,
        None => {
            return Err(result.err().unwrap_or_else(|| {
                crate::CliError::new("configuration", "request plan was not produced")
            }));
        }
    };
    let raw = serde_json::to_value(&request)
        .map_err(|e| crate::CliError::new("configuration", e.to_string()))?;
    let bytes = serde_json::to_vec(&raw)
        .map_err(|e| crate::CliError::new("configuration", e.to_string()))?;
    let (provider, model) = planner
        .identity
        .lock()
        .expect("request planner identity poisoned")
        .take()
        .ok_or_else(|| crate::CliError::new("configuration", "request plan identity missing"))?;
    let record = serde_json::json!({"contract":"reason-native-first-request-plan-v1", "question":question,
        "provider":provider, "model":model,
        "request_sha256":reasoning_harness_providers::http_observation::sha256(&bytes),
        "request":crate::diagnostic_trace::sanitize_value(raw)});
    let data = serde_json::to_vec(&record)
        .map_err(|e| crate::CliError::new("configuration", e.to_string()))?;
    crate::local_privacy::write_new_private_file(destination, &data).map_err(crate::CliError::from)
}
