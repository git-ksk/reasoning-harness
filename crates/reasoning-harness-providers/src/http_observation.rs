//! Opt-in native evaluation transport observations. No filesystem or credentials.
use std::{future::Future, sync::Arc};

use reasoning_harness_core::{ModelError, ModelErrorKind, ModelUsage};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpObservationContext {
    pub correlation_id: String,
    pub logical_attempt: u32,
    pub request_sha256: String,
}

tokio::task_local! {
    static CONTEXT: HttpObservationContext;
}

pub async fn with_http_observation_context<T>(
    context: HttpObservationContext,
    future: impl Future<Output = T>,
) -> T {
    CONTEXT.scope(context, future).await
}

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn observation_error() -> ModelError {
    ModelError::new(
        ModelErrorKind::Transport,
        "evaluation HTTP journal halted; no further request permitted",
    )
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HttpObservation {
    Request {
        request_id: String,
        context: HttpObservationContext,
        provider: &'static str,
        model: String,
        wire_request_sha256: String,
    },
    Response {
        request_id: String,
        http_status: u16,
        response_sha256: String,
        usage: ModelUsage,
        failure_class: Option<&'static str>,
    },
    Aborted {
        request_id: String,
    },
    Completed {
        request_id: String,
    },
}

pub trait HttpObservationSink: Send + Sync {
    /// Must durably persist before returning success. Errors must latch stop.
    fn record(&self, event: HttpObservation) -> Result<(), ModelError>;
}

pub(crate) struct PendingHttpObservation {
    sink: Arc<dyn HttpObservationSink>,
    request_id: String,
    finished: bool,
}

impl PendingHttpObservation {
    pub fn begin(
        sink: Arc<dyn HttpObservationSink>,
        model: &str,
        body: &[u8],
    ) -> Result<Self, ModelError> {
        let context = CONTEXT
            .try_with(Clone::clone)
            .map_err(|_| observation_error())?;
        let request_id = format!(
            "http-{}-{}",
            context.correlation_id, context.logical_attempt
        );
        sink.record(HttpObservation::Request {
            request_id: request_id.clone(),
            context,
            provider: "mistral",
            model: model.to_owned(),
            wire_request_sha256: sha256(body),
        })?;
        Ok(Self {
            sink,
            request_id,
            finished: false,
        })
    }

    pub fn finish(
        &mut self,
        status: u16,
        body: &[u8],
        usage: ModelUsage,
        failure: Option<&'static str>,
    ) -> Result<(), ModelError> {
        self.sink.record(HttpObservation::Response {
            request_id: self.request_id.clone(),
            http_status: status,
            response_sha256: sha256(body),
            usage,
            failure_class: failure,
        })
    }

    pub fn complete(&mut self) -> Result<(), ModelError> {
        self.sink.record(HttpObservation::Completed {
            request_id: self.request_id.clone(),
        })?;
        self.finished = true;
        Ok(())
    }
}

impl Drop for PendingHttpObservation {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.sink.record(HttpObservation::Aborted {
                request_id: self.request_id.clone(),
            });
        }
    }
}
