use std::{
    env,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};

use crate::mistral_retry_budget::{self as retry_budget, Tokens};
use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelExecutionBudget,
    ModelExecutionTelemetrySnapshot, ModelOutputFormat, ModelRequest, ModelResponse, ModelUsage,
};
use reqwest::{Client, StatusCode, Url, header::RETRY_AFTER};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://api.mistral.ai/v1/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);
const MAX_RATE_LIMIT_RETRIES: usize = 3;
const INITIAL_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(30);
const MAX_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(120);
const MAX_RATE_LIMIT_RETRY_WAIT: Duration = Duration::from_secs(210);
const MAX_SUCCESS_PACING_DELAY: Duration = Duration::from_secs(90);
const RATE_LIMIT_TELEMETRY_ENV: &str = "REASON_MISTRAL_RATE_LIMIT_TELEMETRY";

pub struct MistralAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
    single_attempt: bool,
    accounted_retry: bool,
    http_observer: Option<Arc<dyn crate::http_observation::HttpObservationSink>>,
    execution_budget: Mutex<Option<ModelExecutionBudget>>,
    /// Native observed requests defer a successful response's rate-limit
    /// cooldown until the next request. Never hold a completed answer open
    /// merely to wait for a request that may not be needed.
    observed_next_allowed_start: Mutex<Option<Instant>>,
    execution_telemetry: crate::network::ExecutionTelemetryCounters,
}

impl MistralAdapter {
    /// Native evaluation opt-in. Observed calls never perform internal retries.
    pub fn set_http_observer(
        &mut self,
        sink: Arc<dyn crate::http_observation::HttpObservationSink>,
    ) {
        self.http_observer = Some(sink);
    }

    /// Local loopback/provider-free acceptance through the real native CLI path.
    pub fn set_evaluation_loopback_url(&mut self, url: &str) -> Result<(), ModelError> {
        let url = Url::parse(url).map_err(|_| crate::http_observation::observation_error())?;
        if self.http_observer.is_none()
            || url.scheme() != "http"
            || url.host_str() != Some("127.0.0.1")
            || url.port().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(crate::http_observation::observation_error());
        }
        // A loopback control must never send its request through an ambient
        // HTTP proxy. Production clients retain their existing proxy policy.
        self.client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .map_err(|_| crate::http_observation::observation_error())?;
        self.base_url = url;
        Ok(())
    }

    /// Disable adapter-internal HTTP retries for cumulative token-budget enforcement.
    pub fn set_single_attempt(&mut self, enabled: bool) {
        self.single_attempt = enabled;
    }

    /// Explicit development-only opt-in; native observed calls always bypass it.
    pub fn set_accounted_retry(&mut self, enabled: bool) {
        self.accounted_retry = enabled;
    }

    pub fn from_env(model: impl Into<String>) -> Result<Self, ModelError> {
        let api_key = env::var("MISTRAL_API_KEY").map_err(|_| {
            ModelError::new(ModelErrorKind::Credentials, "MISTRAL_API_KEY is not set")
        })?;
        Self::new(api_key, model)
    }

    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, ModelError> {
        Self::with_base_url(api_key, model, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: &str,
    ) -> Result<Self, ModelError> {
        let api_key = api_key.into();
        let model = model.into();
        if api_key.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Credentials,
                "Mistral API key must not be empty",
            ));
        }

        if model.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Protocol,
                "Mistral model identifier must not be empty",
            ));
        }

        let base_url = Url::parse(base_url).map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Mistral base URL: {error}"),
            )
        })?;
        let client = crate::network::client_builder()
            .map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Transport,
                    format!("failed to configure HTTP client: {}", error.message()),
                )
            })?
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .timeout(DEFAULT_TIMEOUT)
            .build()
            .map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Transport,
                    format!("failed to build HTTP client: {error}"),
                )
            })?;

        Ok(Self {
            client,
            api_key,
            base_url,
            model,
            single_attempt: false,
            accounted_retry: false,
            http_observer: None,
            execution_budget: Mutex::new(None),
            observed_next_allowed_start: Mutex::new(None),
            execution_telemetry: crate::network::ExecutionTelemetryCounters::default(),
        })
    }

    async fn generate_inner(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let accounted = self.accounted_retry && self.http_observer.is_none();
        let logical_bytes = if accounted {
            Some(
                crate::http_observation::measure_model_request(&request)?
                    .1
                    .request_utf8_json_bytes,
            )
        } else {
            None
        };
        let evaluation_request = self
            .http_observer
            .as_ref()
            .map(|_| crate::http_observation::measure_model_request(&request))
            .transpose()?;
        let endpoint = self.base_url.join("chat/completions").map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("failed to construct Mistral endpoint: {error}"),
            )
        })?;

        let mut messages = Vec::new();
        if let Some(system) = request.system {
            messages.push(Message {
                role: "system",
                content: system,
            });
        }
        messages.push(Message {
            role: "user",
            content: request.task,
        });

        let body = ChatRequest {
            model: &self.model,
            messages,
            response_format: response_format(request.output_format),
            max_tokens: request.max_tokens,
            random_seed: request.random_seed,
            temperature: 0.0,
        };

        if let Some(observer) = &self.http_observer {
            return self
                .generate_observed(
                    endpoint,
                    &body,
                    observer.clone(),
                    evaluation_request.as_ref().unwrap(),
                )
                .await;
        }

        let accounting_limits = if accounted {
            let Some(limits) = retry_budget::limits() else {
                return Err(local_retry_refusal("missing_accounted_retry_limits"));
            };
            if self.model != "ministral-8b-2512" {
                return Err(local_retry_refusal("unsupported_accounted_retry_model"));
            }
            Some(limits)
        } else {
            None
        };
        let reservation = if accounted {
            let Some(output) = body.max_tokens.filter(|n| (1..=1024).contains(n)) else {
                return Err(local_retry_refusal("invalid_accounted_retry_output_cap"));
            };
            let wire_bytes = serde_json::to_vec(&body)
                .map_err(|_| local_retry_refusal("invalid_accounted_retry_wire"))?
                .len() as u64;
            Some(Tokens {
                input: wire_bytes
                    .max(logical_bytes.unwrap_or_default())
                    .saturating_add(retry_budget::INPUT_OVERHEAD),
                output: u64::from(output),
            })
        } else {
            None
        };
        let mut charged = Tokens::default();
        let mut rate_limit_retries = 0usize;
        let execution_budget = *self
            .execution_budget
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut used_wait = Duration::ZERO;
        let mut used_active = Duration::ZERO;

        let response = loop {
            if let Some((limits, reserve)) = accounting_limits.zip(reservation) {
                if !charged.add(reserve).fits(limits) {
                    retry_budget::publish(charged.usage());
                    return Err(local_retry_refusal("mistral_attempt_reservation_exhausted"));
                }
            }
            let active_remaining =
                crate::network::remaining_active_budget(used_active, execution_budget)?;
            let attempt_number = u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX);
            self.execution_telemetry.attempt_started();
            let attempt_started = Instant::now();
            let send = self
                .client
                .post(endpoint.clone())
                .bearer_auth(&self.api_key)
                .json(&body)
                .send();

            let response = if let Some(remaining) = active_remaining {
                match tokio::time::timeout(remaining, send).await {
                    Ok(result) => {
                        let elapsed = attempt_started.elapsed();
                        used_active = used_active.saturating_add(elapsed);
                        self.execution_telemetry.attempt_completed(elapsed);
                        result
                    }
                    Err(_) => {
                        let elapsed = attempt_started.elapsed();
                        self.execution_telemetry.active_elapsed(elapsed);
                        return Err(ModelError::new(
                            ModelErrorKind::Timeout,
                            "Mistral provider active execution budget exhausted",
                        )
                        .with_provider_attempts(attempt_number));
                    }
                }
            } else {
                let result = send.await;
                let elapsed = attempt_started.elapsed();
                used_active = used_active.saturating_add(elapsed);
                self.execution_telemetry.attempt_completed(elapsed);
                result
            }
            .map_err(|error| {
                classify_transport_error(error).with_provider_attempts(attempt_number)
            })?;

            if response.status() != StatusCode::TOO_MANY_REQUESTS {
                log_rate_limit_telemetry(response.status(), response.headers(), rate_limit_retries);
                break response;
            }

            let headers = response.headers().clone();
            let body = response.text().await.unwrap_or_default();
            let (kind, class) = classify_429(&headers, &body);
            let transient = matches!(class, "temporary_capacity" | "temporary_rate_limit");
            let mut usage_state = "unknown";
            let mut account_decision = Ok(());
            if let Some((limits, reserve)) = accounting_limits.zip(reservation) {
                if transient {
                    let exact = match error_reported_tokens(&body) {
                        Ok(value) => value,
                        Err(()) => {
                            log_429_decision(
                                &headers,
                                attempt_number,
                                class,
                                "invalid_usage",
                                Duration::ZERO,
                                "invalid",
                            );
                            return Err(ModelError::new(
                                ModelErrorKind::Protocol,
                                "Mistral rejected response reported malformed usage",
                            )
                            .with_provider_attempts(attempt_number));
                        }
                    };
                    let debit = exact.unwrap_or(reserve);
                    if !debit.within(reserve) || !charged.add(debit).fits(limits) {
                        log_429_decision(
                            &headers,
                            attempt_number,
                            class,
                            "reservation_violation",
                            Duration::ZERO,
                            "invalid",
                        );
                        return Err(ModelError::new(
                            ModelErrorKind::Protocol,
                            "Mistral reported usage outside conservative reservation",
                        )
                        .with_provider_attempts(attempt_number));
                    }
                    charged = charged.add(debit);
                    usage_state = if exact.is_some() {
                        "reported"
                    } else {
                        "reserved"
                    };
                    log_accounted_attempt(attempt_number, 429, usage_state, debit);
                    if !charged.add(reserve).fits(limits) {
                        account_decision = Err("token_reservation_limit");
                    }
                }
            }
            let decision = retry_429_decision(
                &headers,
                class,
                self.single_attempt && !(accounted && transient),
                rate_limit_retries,
                used_wait,
            )
            .and_then(|delay| account_decision.map(|()| delay))
            .and_then(|delay| {
                crate::network::bounded_wait(delay, used_wait, execution_budget)
                    .map_err(|_| "execution_wait_budget")
            });
            let delay = match decision {
                Ok(delay) => {
                    log_429_decision(&headers, attempt_number, class, "retry", delay, usage_state);
                    delay
                }
                Err(reason) => {
                    log_429_decision(
                        &headers,
                        attempt_number,
                        class,
                        reason,
                        Duration::ZERO,
                        usage_state,
                    );
                    if accounted && transient {
                        retry_budget::publish(charged.usage());
                    }
                    return Err(ModelError::new(
                        kind,
                        format!(
                            "Mistral returned HTTP 429 after {rate_limit_retries} rate-limit retries; class={class}; retry_decision={reason}; usage={usage_state}"
                        ),
                    )
                    .with_provider_attempts(attempt_number));
                }
            };
            rate_limit_retries += 1;
            let wait_started = Instant::now();
            tokio::time::sleep(delay).await;
            if accounted {
                eprintln!(
                    "[mistral-retry-wait] attempt={attempt_number} requested_ms={} elapsed_ms={}",
                    delay.as_millis(),
                    wait_started.elapsed().as_millis()
                );
            }
            used_wait = used_wait.saturating_add(delay);
            self.execution_telemetry
                .wait_completed(crate::network::ProviderWaitKind::Retry, delay);
        };

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let kind = classify_http_error(status, &body);
            let detail = if accounted {
                String::new()
            } else {
                provider_error_detail(&body.replace(&self.api_key, "[redacted]"))
            };
            return Err(ModelError::new(
                kind,
                format!(
                    "Mistral returned HTTP {status} after {rate_limit_retries} rate-limit retries{detail}"
                ),
            )
            .with_provider_attempts(u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX)));
        }

        let pacing_delay = success_pacing_delay(response.headers());
        let response: ChatResponse = response.json().await.map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Mistral response: {error}"),
            )
            .with_provider_attempts(u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX))
        })?;
        let response_usage = ModelUsage {
            input_tokens: response.usage.as_ref().and_then(|u| u.prompt_tokens),
            output_tokens: response.usage.as_ref().and_then(|u| u.completion_tokens),
            total_tokens: response.usage.as_ref().and_then(|u| u.total_tokens),
        };
        if let Some((limits, reserve)) = accounting_limits.zip(reservation) {
            let Some(exact) = Tokens::from_usage(&response_usage) else {
                return Err(ModelError::new(
                    ModelErrorKind::Protocol,
                    "Mistral successful response usage unknown",
                )
                .with_provider_attempts(
                    u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX),
                ));
            };
            if !exact.within(reserve)
                || !charged.add(exact).fits(limits)
                || response.model != self.model
            {
                return Err(ModelError::new(
                    ModelErrorKind::Protocol,
                    "Mistral successful response violated reservation or model pin",
                )
                .with_provider_attempts(
                    u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX),
                ));
            }
            charged = charged.add(exact);
            log_accounted_attempt(
                u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX),
                status.as_u16(),
                "reported",
                exact,
            );
            retry_budget::publish(charged.usage());
        }
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            ModelError::new(
                ModelErrorKind::Protocol,
                "Mistral response contained no choices",
            )
            .with_provider_attempts(u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX))
        })?;

        let model_response = ModelResponse {
            text: choice.message.content.into_text().map_err(|error| {
                error.with_provider_attempts(
                    u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX),
                )
            })?,
            model: response.model,
            usage: if accounted {
                charged.usage()
            } else if rate_limit_retries > 0 {
                ModelUsage::default()
            } else {
                response_usage
            },
            provider_attempts: u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX),
            finish_reason: choice.finish_reason,
        };

        if let Some(delay) = pacing_delay {
            if rate_limit_telemetry_enabled() {
                eprintln!(
                    "[mistral-rate-limit] proactive cooldown_ms={} model={}",
                    delay.as_millis(),
                    self.model
                );
            }
            let delay = crate::network::bounded_wait(delay, used_wait, execution_budget).map_err(
                |error| {
                    error.with_provider_attempts(
                        u32::try_from(rate_limit_retries + 1).unwrap_or(u32::MAX),
                    )
                },
            )?;
            let wait_started = Instant::now();
            tokio::time::sleep(delay).await;
            if accounted {
                eprintln!(
                    "[mistral-pacing-wait] requested_ms={} elapsed_ms={}",
                    delay.as_millis(),
                    wait_started.elapsed().as_millis()
                );
            }
            self.execution_telemetry
                .wait_completed(crate::network::ProviderWaitKind::Pacing, delay);
        }

        Ok(model_response)
    }

    async fn generate_observed(
        &self,
        endpoint: Url,
        body: &ChatRequest<'_>,
        observer: Arc<dyn crate::http_observation::HttpObservationSink>,
        measured: &(String, crate::http_observation::HttpBudgetInput),
    ) -> Result<ModelResponse, ModelError> {
        use crate::http_observation::PendingHttpObservation;
        // Freeze the exact bytes submitted to reqwest, without Authorization.
        let bytes =
            serde_json::to_vec(body).map_err(|_| crate::http_observation::observation_error())?;
        let budget = *self
            .execution_budget
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        // A preceding successful HTTP 200 may require a cooldown before
        // another POST. Wait *before* opening the next observed request,
        // rather than after its predecessor's response while the predecessor
        // is still uncommitted. The single-host native evaluation invokes
        // observed model generations sequentially. An insufficient finite
        // wait budget refuses the next request before any HTTP send.
        let next_allowed = *self
            .observed_next_allowed_start
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if let Some(next_allowed) = next_allowed {
            let requested = next_allowed.saturating_duration_since(Instant::now());
            if !requested.is_zero() {
                let delay = crate::network::bounded_wait(requested, Duration::ZERO, budget)?;
                tokio::time::sleep(delay).await;
                self.execution_telemetry
                    .wait_completed(crate::network::ProviderWaitKind::Pacing, delay);
            }
        }
        let mut pending =
            PendingHttpObservation::begin(observer, &self.model, &bytes, measured).await?;
        let remaining = crate::network::remaining_active_budget(Duration::ZERO, budget)?
            .unwrap_or(DEFAULT_TIMEOUT)
            .min(DEFAULT_TIMEOUT);
        let started = Instant::now();
        self.execution_telemetry.attempt_started();
        let operation = async {
            let mut response = self
                .client
                .post(endpoint)
                .bearer_auth(&self.api_key)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(bytes)
                .send()
                .await
                .map_err(classify_transport_error)?;
            let status = response.status();
            let pacing = success_pacing_delay(response.headers());
            let mut raw = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(classify_transport_error)? {
                if raw.len().saturating_add(chunk.len()) > 4 * 1024 * 1024 {
                    return Err(ModelError::new(
                        ModelErrorKind::Protocol,
                        "Mistral evaluation response exceeds 4 MiB",
                    ));
                }
                raw.extend_from_slice(&chunk);
            }
            let envelope: Option<Value> = serde_json::from_slice(&raw).ok();
            let usage = ModelUsage {
                input_tokens: envelope
                    .as_ref()
                    .and_then(|v| v.pointer("/usage/prompt_tokens"))
                    .and_then(Value::as_u64),
                output_tokens: envelope
                    .as_ref()
                    .and_then(|v| v.pointer("/usage/completion_tokens"))
                    .and_then(Value::as_u64),
                total_tokens: envelope
                    .as_ref()
                    .and_then(|v| v.pointer("/usage/total_tokens"))
                    .and_then(Value::as_u64),
            };
            let parsed = if !status.is_success() {
                Err(ModelError::new(
                    classify_http_error(status, &String::from_utf8_lossy(&raw)),
                    format!("Mistral evaluation returned HTTP {status}"),
                ))
            } else {
                (|| {
                    let response: ChatResponse = serde_json::from_slice(&raw).map_err(|_| {
                        ModelError::new(
                            ModelErrorKind::Protocol,
                            "invalid Mistral evaluation response",
                        )
                    })?;
                    if response.model != self.model {
                        return Err(ModelError::new(
                            ModelErrorKind::Protocol,
                            "Mistral evaluation response model mismatch",
                        ));
                    }
                    let choice = response.choices.into_iter().next().ok_or_else(|| {
                        ModelError::new(
                            ModelErrorKind::Protocol,
                            "Mistral evaluation response contained no choices",
                        )
                    })?;
                    Ok(ModelResponse {
                        text: choice.message.content.into_text()?,
                        model: response.model,
                        usage: usage.clone(),
                        provider_attempts: 1,
                        finish_reason: choice.finish_reason,
                    })
                })()
            };
            let recorded = pending.finish(
                status.as_u16(),
                &raw,
                usage,
                parsed.as_ref().err().map(|_| "provider_error"),
            );
            let model_response = parsed?;
            recorded?;
            Ok((model_response, pacing))
        };
        let result = tokio::time::timeout(remaining, operation).await;
        let active = started.elapsed();
        self.execution_telemetry.attempt_completed(active);
        let (response, pacing) = result
            .map_err(|_| {
                ModelError::new(
                    ModelErrorKind::Timeout,
                    "Mistral evaluation HTTP deadline exhausted",
                )
                .with_provider_attempts(1)
            })?
            .map_err(|e: ModelError| e.with_provider_attempts(1))?;
        if let Some(delay) = pacing {
            // Preserve the same successful-response rate-limit protection,
            // but place it before the NEXT observed generation. Completing
            // the current model response first avoids an HTTP200 -> aborted
            // record solely because no final request was forthcoming.
            let until = Instant::now() + delay;
            let mut next = self
                .observed_next_allowed_start
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if next.is_none_or(|prior| prior < until) {
                *next = Some(until);
            }
        }
        pending.complete()?;
        Ok(response)
    }
}

impl ModelAdapter for MistralAdapter {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>,
    > {
        Box::pin(self.generate_inner(request))
    }

    fn configure_execution_budget(&self, budget: Option<ModelExecutionBudget>) {
        *self
            .execution_budget
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = budget;
    }

    fn execution_telemetry_snapshot(&self) -> Option<ModelExecutionTelemetrySnapshot> {
        Some(self.execution_telemetry.snapshot())
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    response_format: ResponseFormat,
    max_tokens: Option<u32>,
    random_seed: Option<u64>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema { json_schema: JsonSchema },
}

#[derive(Debug, Serialize)]
struct JsonSchema {
    name: String,
    schema: Value,
    strict: bool,
}

fn classify_transport_error(error: reqwest::Error) -> ModelError {
    let kind = if error.is_timeout() {
        ModelErrorKind::Timeout
    } else {
        ModelErrorKind::Transport
    };
    let message = if error.is_timeout() {
        "Mistral request timed out".to_string()
    } else {
        format!("Mistral request failed: {error}")
    };
    ModelError::new(kind, message)
}

fn classify_http_error(status: StatusCode, body: &str) -> ModelErrorKind {
    if status == StatusCode::TOO_MANY_REQUESTS {
        let normalized = body.to_ascii_lowercase();
        if [
            "quota",
            "billing",
            "credit",
            "insufficient balance",
            "monthly",
            "payment required",
            "spending limit",
            "insufficient funds",
            "budget exhausted",
            "budget exceeded",
        ]
        .iter()
        .any(|needle| normalized.contains(needle))
        {
            ModelErrorKind::Quota
        } else {
            ModelErrorKind::RateLimit
        }
    } else if status == StatusCode::PAYMENT_REQUIRED {
        ModelErrorKind::Quota
    } else if status == StatusCode::REQUEST_TIMEOUT || status == StatusCode::GATEWAY_TIMEOUT {
        ModelErrorKind::Timeout
    } else if matches!(status.as_u16(), 502 | 503) || status.is_server_error() {
        ModelErrorKind::ProviderUnavailable
    } else if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        ModelErrorKind::Credentials
    } else if matches!(
        status,
        StatusCode::BAD_REQUEST | StatusCode::NOT_FOUND | StatusCode::UNPROCESSABLE_ENTITY
    ) {
        ModelErrorKind::Protocol
    } else {
        ModelErrorKind::Provider
    }
}

fn classify_429(
    headers: &reqwest::header::HeaderMap,
    body: &str,
) -> (ModelErrorKind, &'static str) {
    let kind = classify_http_error(StatusCode::TOO_MANY_REQUESTS, body);
    if kind == ModelErrorKind::Quota
        || header_u64_any(headers, &["x-ratelimit-remaining-tokens-month"]) == Some(0)
    {
        return (ModelErrorKind::Quota, "quota_or_billing");
    }
    let message = serde_json::from_str::<Value>(body).ok().and_then(|v| {
        v.pointer("/error/message")
            .or_else(|| v.get("message"))
            .and_then(Value::as_str)
            .map(|m| m.trim().trim_end_matches('.').to_ascii_lowercase())
    });
    let class = match message.as_deref() {
        Some("not enough capacity available for this request, please retry later" | "busy") => {
            "temporary_capacity"
        }
        Some("too many requests" | "rate limit exceeded" | "rate limit reached") => {
            "temporary_rate_limit"
        }
        _ => "unclassified_429",
    };
    (kind, class)
}

const RETRY_RESET_HEADERS: &[&str] = &[
    "x-ratelimit-reset-tokens-minute",
    "x-ratelimit-reset-tokens",
    "x-ratelimit-reset-req-minute",
    "x-ratelimit-reset-requests",
    "ratelimit-reset",
];

fn retry_429_decision(
    headers: &reqwest::header::HeaderMap,
    class: &str,
    single_attempt: bool,
    retries: usize,
    used_wait: Duration,
) -> Result<Duration, &'static str> {
    if !matches!(class, "temporary_capacity" | "temporary_rate_limit") {
        return Err("not_confirmed_transient");
    }
    // A rejected HTTP request may have consumed tokens. Never retry while
    // the existing cumulative token guard requires one HTTP attempt.
    if single_attempt {
        return Err("single_attempt_usage_unknown");
    }
    if retries >= MAX_RATE_LIMIT_RETRIES {
        return Err("attempt_limit");
    }
    if (headers.contains_key(RETRY_AFTER) && retry_after_delay(headers).is_none())
        || RETRY_RESET_HEADERS.iter().any(|name| {
            headers.contains_key(*name)
                && header_str(headers, name)
                    .and_then(parse_reset_duration)
                    .is_none()
        })
    {
        return Err("invalid_retry_header");
    }
    let delay = rate_limit_delay(headers, retries);
    if delay > MAX_RATE_LIMIT_BACKOFF || used_wait.saturating_add(delay) > MAX_RATE_LIMIT_RETRY_WAIT
    {
        return Err("retry_wait_limit");
    }
    Ok(delay)
}

fn rate_limit_delay(headers: &reqwest::header::HeaderMap, retry_index: usize) -> Duration {
    retry_after_delay(headers)
        .into_iter()
        .chain(
            RETRY_RESET_HEADERS
                .iter()
                .filter_map(|name| header_str(headers, name).and_then(parse_reset_duration)),
        )
        .max()
        .unwrap_or_else(|| fallback_rate_limit_delay(retry_index))
}

fn retry_after_delay(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();
    // Retry-After delta-seconds are not Unix timestamps.
    if let Ok(seconds) = value.parse::<u64>() {
        return Some(Duration::from_secs(seconds.max(1)));
    }
    let instant = httpdate::parse_http_date(value).ok()?;
    Some(
        instant
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::from_secs(1))
            .max(Duration::from_secs(1)),
    )
}

fn fallback_rate_limit_delay(retry_index: usize) -> Duration {
    let multiplier = 1u32.checked_shl(retry_index as u32).unwrap_or(u32::MAX);
    INITIAL_RATE_LIMIT_BACKOFF
        .checked_mul(multiplier)
        .unwrap_or(MAX_RATE_LIMIT_BACKOFF)
        .min(MAX_RATE_LIMIT_BACKOFF)
}

fn rate_limit_reset_delay(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    const RESET_HEADERS: &[&str] = &[
        "x-ratelimit-reset-tokens-minute",
        "x-ratelimit-reset-tokens",
        "x-ratelimit-reset-req-minute",
        "x-ratelimit-reset-requests",
        "ratelimit-reset",
    ];
    RESET_HEADERS
        .iter()
        .find_map(|name| header_str(headers, name).and_then(parse_reset_duration))
}

fn parse_reset_duration(value: &str) -> Option<Duration> {
    let value = value.trim();
    if let Some(milliseconds) = value.strip_suffix("ms") {
        let milliseconds = milliseconds.trim().parse::<f64>().ok()?;
        if !milliseconds.is_finite() || milliseconds < 0.0 {
            return None;
        }
        return Duration::try_from_secs_f64((milliseconds / 1000.0).max(0.001)).ok();
    }
    if let Some(seconds) = value.strip_suffix('s') {
        let seconds = seconds.trim().parse::<f64>().ok()?;
        if !seconds.is_finite() || seconds < 0.0 {
            return None;
        }
        return Duration::try_from_secs_f64(seconds.max(1.0)).ok();
    }
    if let Ok(raw) = value.parse::<u64>() {
        // Small values are conventional Retry-After/reset durations. Large values are
        // interpreted as Unix timestamps, which some gateways expose for reset headers.
        if raw <= 86_400 {
            return Some(Duration::from_secs(raw.max(1)));
        }
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs();
        return Some(Duration::from_secs(raw.saturating_sub(now).max(1)));
    }
    if let Ok(instant) = httpdate::parse_http_date(value) {
        return Some(
            instant
                .duration_since(SystemTime::now())
                .unwrap_or(Duration::from_secs(1))
                .max(Duration::from_secs(1)),
        );
    }
    None
}

fn success_pacing_delay(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let remaining_tokens = header_u64_any(
        headers,
        &[
            "x-ratelimit-remaining-tokens-minute",
            "x-ratelimit-remaining-tokens",
        ],
    );
    let token_limit = header_u64_any(
        headers,
        &[
            "x-ratelimit-limit-tokens-minute",
            "x-ratelimit-limit-tokens",
        ],
    );
    let query_cost = header_u64_any(headers, &["x-ratelimit-tokens-query-cost"]);
    let remaining_requests = header_u64_any(
        headers,
        &[
            "x-ratelimit-remaining-req-minute",
            "x-ratelimit-remaining-requests",
            "ratelimit-remaining",
        ],
    );
    let request_limit = header_u64_any(
        headers,
        &[
            "x-ratelimit-limit-req-minute",
            "x-ratelimit-limit-requests",
            "ratelimit-limit",
        ],
    );

    let token_pressure = remaining_tokens.is_some_and(|remaining| {
        let cost_guard = query_cost.unwrap_or(0).saturating_mul(2).max(1);
        let percentage_guard = token_limit.map(|limit| limit / 10).unwrap_or(0);
        remaining <= cost_guard.max(percentage_guard)
    });
    let request_pressure = remaining_requests.is_some_and(|remaining| {
        let percentage_guard = request_limit.map(|limit| limit / 10).unwrap_or(0);
        remaining <= 1.max(percentage_guard)
    });

    if token_pressure || request_pressure {
        Some(
            rate_limit_reset_delay(headers)
                .unwrap_or(Duration::from_secs(60))
                .max(Duration::from_secs(1))
                .min(MAX_SUCCESS_PACING_DELAY),
        )
    } else {
        None
    }
}

fn header_u64_any(headers: &reqwest::header::HeaderMap, names: &[&str]) -> Option<u64> {
    names
        .iter()
        .find_map(|name| header_str(headers, name).and_then(|value| value.parse::<u64>().ok()))
}

fn header_str<'a>(headers: &'a reqwest::header::HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn log_429_decision(
    headers: &reqwest::header::HeaderMap,
    attempt: u32,
    class: &str,
    reason: &str,
    delay: Duration,
    usage_state: &str,
) {
    if !rate_limit_telemetry_enabled() {
        return;
    }
    let timestamp_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let summary = rate_limit_header_summary(headers);
    eprintln!(
        "[mistral-rate-limit] status=429 timestamp_ms={timestamp_ms} attempt={attempt} retry_index={} class={class} retry_allowed={} reason={reason} wait_ms={} usage={usage_state} headers={}",
        attempt.saturating_sub(1),
        reason == "retry",
        delay.as_millis(),
        if summary.is_empty() {
            "absent"
        } else {
            &summary
        },
    );
}

fn rate_limit_header_summary(headers: &reqwest::header::HeaderMap) -> String {
    const SAFE_HEADERS: &[&str] = &[
        "retry-after",
        "x-ratelimit-remaining",
        "x-ratelimit-limit-tokens-minute",
        "x-ratelimit-remaining-tokens-minute",
        "x-ratelimit-reset-tokens-minute",
        "x-ratelimit-limit-tokens-month",
        "x-ratelimit-remaining-tokens-month",
        "x-ratelimit-tokens-query-cost",
        "x-ratelimit-limit-req-minute",
        "x-ratelimit-remaining-req-minute",
        "x-ratelimit-reset-req-minute",
        "x-ratelimit-limit-tokens",
        "x-ratelimit-remaining-tokens",
        "x-ratelimit-reset-tokens",
        "x-ratelimit-limit-requests",
        "x-ratelimit-remaining-requests",
        "x-ratelimit-reset-requests",
        "ratelimit-limit",
        "ratelimit-remaining",
        "ratelimit-reset",
    ];

    SAFE_HEADERS
        .iter()
        .filter_map(|name| {
            header_str(headers, name)
                .filter(|value| {
                    value.len() <= 96
                        && (value.parse::<u64>().is_ok() || parse_reset_duration(value).is_some())
                })
                .map(|value| format!("{name}={}", truncate_diagnostic(value, 96)))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn rate_limit_telemetry_enabled() -> bool {
    env::var(RATE_LIMIT_TELEMETRY_ENV).is_ok_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    })
}

fn log_rate_limit_telemetry(
    status: StatusCode,
    headers: &reqwest::header::HeaderMap,
    retry_index: usize,
) {
    if !rate_limit_telemetry_enabled() {
        return;
    }
    let summary = rate_limit_header_summary(headers);
    if !summary.is_empty() {
        eprintln!(
            "[mistral-rate-limit] status={} retry_index={} {}",
            status.as_u16(),
            retry_index,
            summary
        );
    }
}

fn local_retry_refusal(reason: &'static str) -> ModelError {
    retry_budget::refuse(reason);
    ModelError {
        kind: ModelErrorKind::Protocol,
        message: reason.into(),
        provider_attempts: 0,
    }
}

fn error_reported_tokens(body: &str) -> Result<Option<Tokens>, ()> {
    let value: Value = serde_json::from_str(body).map_err(|_| ())?;
    let Some(value) = value.get("usage").filter(|v| !v.is_null()) else {
        return Ok(None);
    };
    let usage: Usage = serde_json::from_value(value.clone()).map_err(|_| ())?;
    Tokens::from_usage(&ModelUsage {
        input_tokens: usage.prompt_tokens,
        output_tokens: usage.completion_tokens,
        total_tokens: usage.total_tokens,
    })
    .map(Some)
    .ok_or(())
}

fn log_accounted_attempt(attempt: u32, status: u16, usage: &str, tokens: Tokens) {
    let timestamp_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    eprintln!(
        "[mistral-accounted-attempt] status={status} attempt={attempt} usage={usage} timestamp_ms={timestamp_ms} charged_input={} charged_output={} charged_total={}",
        tokens.input,
        tokens.output,
        tokens.total()
    );
}

fn provider_error_detail(body: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return String::new();
    };
    let error = value.get("error").unwrap_or(&value);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| error.get("detail").and_then(Value::as_str));
    message
        .map(|message| format!("; message={}", truncate_diagnostic(message, 512)))
        .unwrap_or_default()
}

fn truncate_diagnostic(value: &str, max_chars: usize) -> String {
    let normalized = value.replace(['\r', '\n'], " ");
    let mut chars = normalized.chars();
    let prefix = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

fn response_format(format: ModelOutputFormat) -> ResponseFormat {
    match format {
        ModelOutputFormat::Text => ResponseFormat::Text,
        ModelOutputFormat::JsonObject => ResponseFormat::JsonObject,
        ModelOutputFormat::JsonSchema { name, schema } => ResponseFormat::JsonSchema {
            json_schema: JsonSchema {
                name,
                schema,
                strict: true,
            },
        },
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    model: String,
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    finish_reason: Option<String>,
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: ResponseContent,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponseContent {
    Text(String),
    Chunks(Vec<ResponseChunk>),
}

impl ResponseContent {
    fn into_text(self) -> Result<String, ModelError> {
        match self {
            Self::Text(text) => Ok(text),
            Self::Chunks(chunks) => {
                let text = chunks
                    .into_iter()
                    .filter_map(|chunk| chunk.text)
                    .collect::<String>();
                if text.is_empty() {
                    Err(ModelError::new(
                        ModelErrorKind::Protocol,
                        "Mistral response contained no text content",
                    ))
                } else {
                    Ok(text)
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct ResponseChunk {
    #[allow(dead_code)]
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn observed_http200_finishes_before_cooldown_and_next_request_fails_budget() {
        use crate::http_observation::{
            HttpObservation, HttpObservationContext, HttpObservationSink,
            with_http_observation_context,
        };
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };

        #[derive(Default)]
        struct Journal(Mutex<Vec<&'static str>>);
        impl HttpObservationSink for Journal {
            fn record(&self, event: HttpObservation) -> Result<(), ModelError> {
                let name = match event {
                    HttpObservation::Request { .. } => "request",
                    HttpObservation::Response { .. } => "response",
                    HttpObservation::Aborted { .. } => "aborted",
                    HttpObservation::Completed { .. } => "completed",
                };
                self.0.lock().unwrap().push(name);
                Ok(())
            }
        }

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 8192];
            let _ = stream.read(&mut buffer);
            let body = r#"{"model":"test-model","choices":[{"message":{"content":"done"},"finish_reason":"stop"}],"usage":{"prompt_tokens":4,"completion_tokens":3,"total_tokens":7}}"#;
            write!(stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nx-ratelimit-limit-tokens-minute: 50000\r\nx-ratelimit-remaining-tokens-minute: 4000\r\nx-ratelimit-tokens-query-cost: 1200\r\n\r\n{body}",
                body.len()).unwrap();
        });

        let mut adapter = MistralAdapter::with_base_url(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
        )
        .unwrap();
        let sink = Arc::new(Journal::default());
        adapter.set_http_observer(sink.clone());
        adapter.configure_execution_budget(Some(ModelExecutionBudget {
            max_active_ms: 60_000,
            max_wait_ms: 1_000,
            max_single_wait_ms: 1_000,
        }));
        let request = ModelRequest {
            task: "source-grounded test".into(),
            system: None,
            output_format: ModelOutputFormat::Text,
            max_tokens: Some(8),
            random_seed: None,
            reasoning_preference: None,
        };
        let (request_sha, input) =
            crate::http_observation::measure_model_request(&request).unwrap();
        let first = with_http_observation_context(
            HttpObservationContext {
                correlation_id: "first".into(),
                logical_attempt: 1,
                request_sha256: request_sha.clone(),
                budget_input: Some(input.clone()),
            },
            adapter.generate(request.clone()),
        )
        .await
        .unwrap();
        assert_eq!(first.text, "done");
        assert_eq!(
            *sink.0.lock().unwrap(),
            ["request", "response", "completed"]
        );

        // The next request cannot evade the server's cooldown. Its 60 s
        // observed budget is > 1 s, so refuse BEFORE another HTTP POST or
        // journal reservation. The first answer remains fully committed.
        let second = with_http_observation_context(
            HttpObservationContext {
                correlation_id: "second".into(),
                logical_attempt: 1,
                request_sha256: request_sha,
                budget_input: Some(input),
            },
            adapter.generate(request),
        )
        .await
        .unwrap_err();
        assert_eq!(second.kind, ModelErrorKind::RateLimit);
        assert_eq!(
            *sink.0.lock().unwrap(),
            ["request", "response", "completed"]
        );
        let telemetry = adapter.execution_telemetry_snapshot().unwrap();
        assert_eq!(telemetry.provider_attempts_started, 1);
        assert_eq!(telemetry.provider_attempts_completed, 1);
        assert_eq!(telemetry.pacing_wait_ms, 0);
        server.join().unwrap();
    }

    #[test]
    fn default_request_timeout_supports_long_form_reasoning() {
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(180));
    }

    #[test]
    fn serializes_json_schema_without_provider_authority_fields() {
        let value = serde_json::to_value(response_format(ModelOutputFormat::JsonSchema {
            name: "artifact".into(),
            schema: serde_json::json!({"type": "object"}),
        }))
        .unwrap();

        assert_eq!(value["type"], "json_schema");
        assert_eq!(value["json_schema"]["name"], "artifact");
        assert_eq!(value["json_schema"]["strict"], true);
        assert!(value.get("verdict").is_none());
    }

    #[test]
    fn rejects_empty_credentials_without_echoing_them() {
        let error = MistralAdapter::new("", "ministral-8b-latest")
            .err()
            .unwrap();
        assert_eq!(error.kind, ModelErrorKind::Credentials);
        assert!(!error.to_string().contains("Bearer"));
    }

    #[test]
    fn classifies_429_as_rate_limit_without_quota_signal() {
        assert_eq!(
            classify_http_error(
                StatusCode::TOO_MANY_REQUESTS,
                r#"{"message":"too many requests"}"#
            ),
            ModelErrorKind::RateLimit
        );
    }

    #[test]
    fn classifies_client_request_rejections_as_protocol() {
        for status in [
            StatusCode::BAD_REQUEST,
            StatusCode::NOT_FOUND,
            StatusCode::UNPROCESSABLE_ENTITY,
        ] {
            assert_eq!(
                classify_http_error(status, r#"{"message":"invalid request"}"#),
                ModelErrorKind::Protocol
            );
        }
    }

    #[test]
    fn classifies_429_quota_signal_separately() {
        assert_eq!(
            classify_http_error(
                StatusCode::TOO_MANY_REQUESTS,
                r#"{"message":"quota exhausted"}"#
            ),
            ModelErrorKind::Quota
        );
    }

    #[test]
    fn temporary_429_allowlist_and_monthly_quota() {
        let mut headers = reqwest::header::HeaderMap::new();
        let capacity =
            r#"{"message":"Not enough capacity available for this request, please retry later."}"#;
        assert_eq!(classify_429(&headers, capacity).1, "temporary_capacity");
        assert_eq!(
            classify_429(&headers, r#"{"message":"quota exhausted"}"#).1,
            "quota_or_billing"
        );
        assert_eq!(
            classify_429(&headers, r#"{"message":"unrecognized failure"}"#).1,
            "unclassified_429"
        );
        headers.insert(
            "x-ratelimit-remaining-tokens-month",
            reqwest::header::HeaderValue::from_static("0"),
        );
        assert_eq!(classify_429(&headers, capacity).0, ModelErrorKind::Quota);
    }

    #[test]
    fn retry_decision_rejects_unaccounted_attempts_and_limits() {
        let mut headers = reqwest::header::HeaderMap::new();
        let temporary = "temporary_capacity";
        assert_eq!(
            retry_429_decision(&headers, temporary, false, 0, Duration::ZERO),
            Ok(Duration::from_secs(30))
        );
        for class in ["quota_or_billing", "unclassified_429"] {
            assert_eq!(
                retry_429_decision(&headers, class, false, 0, Duration::ZERO),
                Err("not_confirmed_transient")
            );
        }
        assert_eq!(
            retry_429_decision(&headers, temporary, true, 0, Duration::ZERO),
            Err("single_attempt_usage_unknown")
        );
        assert_eq!(
            retry_429_decision(&headers, temporary, false, 3, Duration::ZERO),
            Err("attempt_limit")
        );
        assert_eq!(
            retry_429_decision(&headers, temporary, false, 2, Duration::from_secs(91)),
            Err("retry_wait_limit")
        );
        headers.insert(
            RETRY_AFTER,
            reqwest::header::HeaderValue::from_static("121"),
        );
        assert_eq!(
            retry_429_decision(&headers, temporary, false, 0, Duration::ZERO),
            Err("retry_wait_limit")
        );
        headers.insert(
            RETRY_AFTER,
            reqwest::header::HeaderValue::from_static("bad"),
        );
        assert_eq!(
            retry_429_decision(&headers, temporary, false, 0, Duration::ZERO),
            Err("invalid_retry_header")
        );
        headers.insert(RETRY_AFTER, reqwest::header::HeaderValue::from_static("30"));
        headers.insert(
            "x-ratelimit-reset-req-minute",
            reqwest::header::HeaderValue::from_static("60s"),
        );
        assert_eq!(
            retry_429_decision(&headers, temporary, false, 0, Duration::ZERO),
            Ok(Duration::from_secs(60))
        );
    }

    #[test]
    fn rate_limit_delay_prefers_retry_after_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(RETRY_AFTER, reqwest::header::HeaderValue::from_static("7"));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(7));
    }

    #[test]
    fn rate_limit_delay_uses_bounded_exponential_fallback() {
        let headers = reqwest::header::HeaderMap::new();
        assert_eq!(rate_limit_delay(&headers, 0), Duration::from_secs(30));
        assert_eq!(rate_limit_delay(&headers, 1), Duration::from_secs(60));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(120));
    }

    #[test]
    fn rate_limit_delay_uses_provider_token_reset_before_fallback() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-ratelimit-reset-tokens-minute",
            reqwest::header::HeaderValue::from_static("42s"),
        );
        assert_eq!(rate_limit_delay(&headers, 4), Duration::from_secs(42));
    }

    #[test]
    fn fallback_backoff_bridges_a_minute_window_without_unbounded_waits() {
        let delays = (0..MAX_RATE_LIMIT_RETRIES)
            .map(fallback_rate_limit_delay)
            .collect::<Vec<_>>();
        assert_eq!(
            delays,
            vec![
                Duration::from_secs(30),
                Duration::from_secs(60),
                Duration::from_secs(120),
            ]
        );
        assert_eq!(delays.iter().sum::<Duration>(), MAX_RATE_LIMIT_RETRY_WAIT);
        assert_eq!(MAX_RATE_LIMIT_RETRIES, 3);
    }

    #[test]
    fn successful_low_token_headroom_requests_a_cooldown() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-ratelimit-limit-tokens-minute",
            reqwest::header::HeaderValue::from_static("50000"),
        );
        headers.insert(
            "x-ratelimit-remaining-tokens-minute",
            reqwest::header::HeaderValue::from_static("4000"),
        );
        headers.insert(
            "x-ratelimit-tokens-query-cost",
            reqwest::header::HeaderValue::from_static("1200"),
        );
        assert_eq!(
            success_pacing_delay(&headers),
            Some(Duration::from_secs(60))
        );
    }

    #[test]
    fn successful_healthy_headroom_does_not_pace() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-ratelimit-limit-tokens-minute",
            reqwest::header::HeaderValue::from_static("50000"),
        );
        headers.insert(
            "x-ratelimit-remaining-tokens-minute",
            reqwest::header::HeaderValue::from_static("42000"),
        );
        headers.insert(
            "x-ratelimit-tokens-query-cost",
            reqwest::header::HeaderValue::from_static("1200"),
        );
        assert_eq!(success_pacing_delay(&headers), None);
    }

    #[test]
    fn telemetry_whitelists_rate_headers_only() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "x-ratelimit-remaining-tokens-minute",
            reqwest::header::HeaderValue::from_static("1234"),
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_static("Bearer secret-value"),
        );
        headers.insert(
            "x-debug-prompt",
            reqwest::header::HeaderValue::from_static("private prompt"),
        );
        let summary = rate_limit_header_summary(&headers);
        assert!(summary.contains("x-ratelimit-remaining-tokens-minute=1234"));
        assert!(!summary.contains("secret-value"));
        assert!(!summary.contains("private prompt"));
    }

    #[test]
    fn provider_error_detail_is_bounded_and_single_line() {
        let body = serde_json::json!({"error": {"message": format!("first\n{}", "x".repeat(600))}})
            .to_string();
        let detail = provider_error_detail(&body);
        assert!(!detail.contains('\n'));
        assert!(detail.ends_with('…'));
        assert!(detail.chars().count() < 530);
    }

    #[tokio::test]
    async fn execution_budget_rejects_long_rate_limit_retry_without_sleep() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 4096];
            let _ = stream.read(&mut buffer);
            let body = r#"{"error":{"message":"busy"}}"#;
            write!(
                stream,
                "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nContent-Length: {}\r\nRetry-After: 31\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });

        let adapter = MistralAdapter::with_base_url(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
        )
        .unwrap();
        adapter.configure_execution_budget(Some(ModelExecutionBudget {
            max_active_ms: 60_000,
            max_wait_ms: 45_000,
            max_single_wait_ms: 30_000,
        }));

        let started = Instant::now();
        let error = adapter
            .generate(ModelRequest {
                task: "test".into(),
                system: None,
                output_format: ModelOutputFormat::Text,
                max_tokens: Some(8),
                random_seed: None,
                reasoning_preference: None,
            })
            .await
            .unwrap_err();

        assert_eq!(error.kind, ModelErrorKind::RateLimit);
        assert!(started.elapsed() < Duration::from_secs(1));
        let telemetry = adapter.execution_telemetry_snapshot().unwrap();
        assert_eq!(telemetry.provider_attempts_started, 1);
        assert_eq!(telemetry.provider_attempts_completed, 1);
        assert_eq!(telemetry.retry_wait_ms, 0);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn retries_http_429_then_returns_success() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for attempt in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 4096];
                let _ = stream.read(&mut buffer);
                let (status, extra_headers, body) = if attempt == 0 {
                    (
                        "429 Too Many Requests",
                        "Retry-After: 1\r\n",
                        r#"{"error":{"message":"busy"}}"#,
                    )
                } else {
                    (
                        "200 OK",
                        "",
                        r#"{"model":"test-model","choices":[{"finish_reason":"stop","message":{"content":"ok"}}],"usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":4}}"#,
                    )
                };
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra_headers}\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });
        let adapter = MistralAdapter::with_base_url(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
        )
        .unwrap();
        let response = adapter
            .generate(ModelRequest {
                task: "test".into(),
                system: None,
                output_format: ModelOutputFormat::Text,
                max_tokens: Some(8),
                random_seed: None,
                reasoning_preference: None,
            })
            .await
            .unwrap();
        assert_eq!(response.text, "ok");
        assert_eq!(response.model, "test-model");
        assert_eq!(response.provider_attempts, 2);
        assert_eq!(response.usage, ModelUsage::default());
        server.join().unwrap();
    }
    fn retry_fixture_request() -> ModelRequest {
        ModelRequest {
            task: "fixture".into(),
            system: None,
            output_format: ModelOutputFormat::Text,
            max_tokens: Some(8),
            random_seed: Some(1),
            reasoning_preference: None,
        }
    }

    #[tokio::test]
    async fn accounted_429_reserves_unknown_usage_and_retries_with_exact_accounting() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for (status, headers, body) in [
                (
                    "429 Too Many Requests",
                    "Retry-After: 1\r\n",
                    r#"{"message":"Not enough capacity available for this request, please retry later."}"#,
                ),
                (
                    "200 OK",
                    "",
                    r#"{"model":"ministral-8b-2512","choices":[{"finish_reason":"stop","message":{"content":"ok"}}],"usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":4}}"#,
                ),
            ] {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{headers}\r\n{body}", body.len()).unwrap();
            }
        });
        let mut adapter = MistralAdapter::with_base_url(
            "test-key",
            "ministral-8b-2512",
            &format!("http://{address}/v1/"),
        )
        .unwrap();
        adapter.set_single_attempt(true);
        adapter.set_accounted_retry(true);
        let (result, account, refusal) = retry_budget::with_limits(
            Some(32),
            Some(4096),
            adapter.generate(retry_fixture_request()),
        )
        .await;
        let response = result.unwrap();
        assert_eq!(response.provider_attempts, 2);
        assert_eq!(response.usage, account.unwrap());
        assert_eq!(response.usage.output_tokens, Some(9));
        assert!(response.usage.input_tokens.unwrap() > 512);
        assert_eq!(refusal, None);
        assert_eq!(
            adapter
                .execution_telemetry_snapshot()
                .unwrap()
                .provider_attempts_started,
            2
        );
        server.join().unwrap();
    }

    #[tokio::test]
    async fn accounted_429_refuses_retry_without_remaining_output_reservation() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 8192];
            let _ = stream.read(&mut buffer);
            let body = r#"{"message":"too many requests"}"#;
            write!(stream, "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nRetry-After: 1\r\n\r\n{body}", body.len()).unwrap();
        });
        let mut adapter = MistralAdapter::with_base_url(
            "test-key",
            "ministral-8b-2512",
            &format!("http://{address}/v1/"),
        )
        .unwrap();
        adapter.set_single_attempt(true);
        adapter.set_accounted_retry(true);
        let (result, account, _) = retry_budget::with_limits(
            Some(8),
            Some(4096),
            adapter.generate(retry_fixture_request()),
        )
        .await;
        let error = result.unwrap_err();
        assert_eq!(error.provider_attempts, 1);
        assert!(error.message.contains("token_reservation_limit"));
        assert_eq!(account.unwrap().output_tokens, Some(8));
        server.join().unwrap();
    }

    #[tokio::test]
    async fn accounted_retry_without_safe_total_reservation_never_sends() {
        let mut adapter = MistralAdapter::with_base_url(
            "test-key",
            "ministral-8b-2512",
            "http://127.0.0.1:1/v1/",
        )
        .unwrap();
        adapter.set_single_attempt(true);
        adapter.set_accounted_retry(true);
        let (result, account, refusal) = retry_budget::with_limits(
            Some(8),
            Some(100),
            adapter.generate(retry_fixture_request()),
        )
        .await;
        assert_eq!(result.unwrap_err().provider_attempts, 0);
        assert_eq!(account.unwrap().total_tokens, Some(0));
        assert_eq!(refusal, Some("mistral_attempt_reservation_exhausted"));
        assert_eq!(
            adapter
                .execution_telemetry_snapshot()
                .unwrap()
                .provider_attempts_started,
            0
        );
        let (result, _, _) =
            retry_budget::with_limits(Some(8), None, adapter.generate(retry_fixture_request()))
                .await;
        assert_eq!(result.unwrap_err().provider_attempts, 0);
    }

    #[test]
    fn rejected_usage_is_exact_only_when_complete_and_consistent() {
        assert_eq!(error_reported_tokens(r#"{"message":"busy"}"#), Ok(None));
        assert_eq!(
            error_reported_tokens(
                r#"{"usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":4}}"#
            ),
            Ok(Some(Tokens {
                input: 3,
                output: 1
            }))
        );
        assert!(
            error_reported_tokens(r#"{"usage":{"prompt_tokens":3,"total_tokens":4}}"#).is_err()
        );
        assert!(
            error_reported_tokens(
                r#"{"usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":8}}"#
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn accounted_retry_does_not_retry_quota_auth_or_unknown() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        for (status, body, kind) in [
            (
                "429 Too Many Requests",
                r#"{"message":"monthly quota exhausted"}"#,
                ModelErrorKind::Quota,
            ),
            (
                "401 Unauthorized",
                r#"{"message":"invalid key"}"#,
                ModelErrorKind::Credentials,
            ),
            (
                "402 Payment Required",
                r#"{"message":"billing unavailable"}"#,
                ModelErrorKind::Quota,
            ),
            (
                "429 Too Many Requests",
                r#"{"message":"unrecognized error"}"#,
                ModelErrorKind::RateLimit,
            ),
            (
                "429 Too Many Requests",
                r#"{"message":"busy","usage":{"prompt_tokens":999999}}"#,
                ModelErrorKind::Protocol,
            ),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                write!(stream, "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nRetry-After: 1\r\n\r\n{body}", body.len()).unwrap();
            });
            let mut adapter = MistralAdapter::with_base_url(
                "test-key",
                "ministral-8b-2512",
                &format!("http://{address}/v1/"),
            )
            .unwrap();
            adapter.set_single_attempt(true);
            adapter.set_accounted_retry(true);
            let (result, account, _) = retry_budget::with_limits(
                Some(32),
                Some(4096),
                adapter.generate(retry_fixture_request()),
            )
            .await;
            let error = result.unwrap_err();
            assert_eq!(error.kind, kind);
            assert_eq!(error.provider_attempts, 1);
            assert!(account.is_none());
            assert_eq!(
                adapter
                    .execution_telemetry_snapshot()
                    .unwrap()
                    .provider_attempts_started,
                1
            );
            server.join().unwrap();
        }
    }

    #[tokio::test]
    async fn accounted_retry_exhaustion_counts_four_http_attempts() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for _ in 0..4 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                let body = r#"{"message":"busy","usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":4}}"#;
                write!(stream, "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nRetry-After: 1\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        let mut adapter = MistralAdapter::with_base_url(
            "test-key",
            "ministral-8b-2512",
            &format!("http://{address}/v1/"),
        )
        .unwrap();
        adapter.set_single_attempt(true);
        adapter.set_accounted_retry(true);
        let (result, account, _) = retry_budget::with_limits(
            Some(32),
            Some(4096),
            adapter.generate(retry_fixture_request()),
        )
        .await;
        let error = result.unwrap_err();
        assert_eq!(error.provider_attempts, 4);
        assert!(error.message.contains("attempt_limit"));
        assert_eq!(account.unwrap().total_tokens, Some(16));
        let telemetry = adapter.execution_telemetry_snapshot().unwrap();
        assert_eq!(telemetry.provider_attempts_started, 4);
        assert_eq!(telemetry.provider_attempts_completed, 4);
        assert_eq!(telemetry.retry_wait_ms, 3000);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn token_guard_single_attempt_stops_rate_limit_retry() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 8192];
            let _ = stream.read(&mut buffer);
            let body = r#"{"error":{"message":"busy"}}"#;
            write!(stream, "HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nContent-Length: {}\r\nRetry-After: 1\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        });
        let mut adapter = MistralAdapter::with_base_url(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
        )
        .unwrap();
        adapter.set_single_attempt(true);
        let error = adapter
            .generate(ModelRequest {
                task: "test".into(),
                system: None,
                output_format: ModelOutputFormat::Text,
                max_tokens: Some(8),
                random_seed: None,
                reasoning_preference: None,
            })
            .await
            .unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::RateLimit);
        assert_eq!(error.provider_attempts, 1);
        assert_eq!(
            adapter
                .execution_telemetry_snapshot()
                .unwrap()
                .provider_attempts_started,
            1
        );
        server.join().unwrap();
    }
    #[tokio::test]
    async fn provider_does_not_follow_redirect_or_resend_post() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        for status in ["307 Temporary Redirect", "308 Permanent Redirect"] {
            for single in [false, true] {
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                let address = listener.local_addr().unwrap();
                let server = thread::spawn(move || {
                    let (mut stream, _) = listener.accept().unwrap();
                    let mut buffer = [0u8; 8192];
                    let _ = stream.read(&mut buffer);
                    write!(stream, "HTTP/1.1 {status}\r\nLocation: http://{address}/redirected\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
                    listener.set_nonblocking(true).unwrap();
                    let deadline = Instant::now() + Duration::from_millis(300);
                    loop {
                        match listener.accept() {
                            Ok((mut redirected, _)) => {
                                let _ = redirected.read(&mut buffer);
                                let _ = redirected.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}");
                                return 2;
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                                if Instant::now() >= deadline {
                                    return 1;
                                }
                                thread::sleep(Duration::from_millis(5));
                            }
                            Err(error) => panic!("listener failed: {error}"),
                        }
                    }
                });
                let mut adapter = MistralAdapter::with_base_url(
                    "test-key",
                    "test-model",
                    &format!("http://{address}/v1/"),
                )
                .unwrap();
                adapter.set_single_attempt(single);
                let error = adapter
                    .generate(ModelRequest {
                        task: "test".into(),
                        system: None,
                        output_format: ModelOutputFormat::Text,
                        max_tokens: Some(8),
                        random_seed: None,
                        reasoning_preference: None,
                    })
                    .await
                    .unwrap_err();
                assert_eq!(error.provider_attempts, 1);
                assert_eq!(
                    server.join().unwrap(),
                    1,
                    "{status} must not resend model POST, single={single}"
                );
            }
        }
    }
    #[tokio::test]
    async fn no_retry_preserves_successful_response_usage() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        for single in [false, true] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                let body = r#"{"model":"test-model","choices":[{"finish_reason":"stop","message":{"content":"ok"}}],"usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":4}}"#;
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
            });
            let mut adapter = MistralAdapter::with_base_url(
                "test-key",
                "test-model",
                &format!("http://{address}/v1/"),
            )
            .unwrap();
            adapter.set_single_attempt(single);
            let response = adapter
                .generate(ModelRequest {
                    task: "test".into(),
                    system: None,
                    output_format: ModelOutputFormat::Text,
                    max_tokens: Some(8),
                    random_seed: None,
                    reasoning_preference: None,
                })
                .await
                .unwrap();
            assert_eq!(response.provider_attempts, 1);
            assert_eq!(response.usage.input_tokens, Some(3));
            assert_eq!(response.usage.output_tokens, Some(1));
            assert_eq!(response.usage.total_tokens, Some(4));
            server.join().unwrap();
        }
    }
}
