use std::{
    env,
    time::{Duration, Instant, SystemTime},
};

use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelReasoningPreference,
    ModelRequest, ModelResponse, ModelUsage,
};
use reqwest::{Client, StatusCode, Url, header::RETRY_AFTER};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://integrate.api.nvidia.com/v1/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);
const MAX_RATE_LIMIT_RETRIES: usize = 3;
const INITIAL_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(5);
// Conservative client-side pacing for hosted trial endpoints: 37.5 requests/minute.
// This is deliberately not modeled as a provider contract; Retry-After remains authoritative.
const DEFAULT_MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(1_600);

/// NVIDIA Hosted NIM adapter using the provider's OpenAI-compatible Chat Completions API.
///
/// Model output remains an untrusted candidate. This adapter never participates in
/// deterministic verification, adversarial authority, or final verdict selection.
pub struct NvidiaAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
    min_request_interval: Duration,
    last_request_started: tokio::sync::Mutex<Option<Instant>>,
}

impl NvidiaAdapter {
    pub fn from_env(model: impl Into<String>) -> Result<Self, ModelError> {
        let api_key = env::var("NVIDIA_API_KEY").map_err(|_| {
            ModelError::new(ModelErrorKind::Credentials, "NVIDIA_API_KEY is not set")
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
        Self::with_base_url_timeout_and_interval(
            api_key,
            model,
            base_url,
            DEFAULT_TIMEOUT,
            DEFAULT_MIN_REQUEST_INTERVAL,
        )
    }

    #[cfg(test)]
    fn with_base_url_and_timeout(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: &str,
        timeout: Duration,
    ) -> Result<Self, ModelError> {
        Self::with_base_url_timeout_and_interval(api_key, model, base_url, timeout, Duration::ZERO)
    }

    fn with_base_url_timeout_and_interval(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: &str,
        timeout: Duration,
        min_request_interval: Duration,
    ) -> Result<Self, ModelError> {
        let api_key = api_key.into();
        let model = model.into();
        if api_key.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Credentials,
                "NVIDIA API key must not be empty",
            ));
        }
        if model.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Protocol,
                "NVIDIA model identifier must not be empty",
            ));
        }

        let base_url = Url::parse(base_url).map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid NVIDIA API base URL: {error}"),
            )
        })?;
        let client = Client::builder()
            .timeout(timeout)
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
            min_request_interval,
            last_request_started: tokio::sync::Mutex::new(None),
        })
    }

    async fn wait_for_request_slot(&self) {
        if self.min_request_interval.is_zero() {
            return;
        }
        let mut last_started = self.last_request_started.lock().await;
        if let Some(previous) = *last_started {
            let elapsed = previous.elapsed();
            if elapsed < self.min_request_interval {
                tokio::time::sleep(self.min_request_interval - elapsed).await;
            }
        }
        *last_started = Some(Instant::now());
    }

    async fn generate_inner(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let endpoint = self.base_url.join("chat/completions").map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("failed to construct NVIDIA endpoint: {error}"),
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
            seed: request.random_seed,
            chat_template_kwargs: reasoning_controls(request.reasoning_preference),
            temperature: 0.0,
            stream: false,
        };

        let mut rate_limit_retries = 0usize;
        let response = loop {
            self.wait_for_request_slot().await;
            let response = self
                .client
                .post(endpoint.clone())
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(classify_transport_error)?;

            if response.status() != StatusCode::TOO_MANY_REQUESTS
                || rate_limit_retries >= MAX_RATE_LIMIT_RETRIES
            {
                break response;
            }

            let delay = rate_limit_delay(response.headers(), rate_limit_retries);
            rate_limit_retries += 1;
            tokio::time::sleep(delay).await;
        };

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let kind = classify_http_error(status, &body);
            let detail = provider_error_detail(&body);
            return Err(ModelError::new(
                kind,
                format!(
                    "NVIDIA API returned HTTP {status} after {rate_limit_retries} rate-limit retries{detail}"
                ),
            ));
        }

        let response: ChatResponse = response.json().await.map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid NVIDIA Chat Completions response: {error}"),
            )
        })?;
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            ModelError::new(
                ModelErrorKind::Protocol,
                "NVIDIA response contained no choices",
            )
        })?;
        let text = choice.message.content.into_text()?;
        let usage = response.usage.unwrap_or_default();

        Ok(ModelResponse {
            text,
            model: response.model.unwrap_or_else(|| self.model.clone()),
            usage: ModelUsage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            },
            finish_reason: choice.finish_reason,
        })
    }
}

impl ModelAdapter for NvidiaAdapter {
    fn generate<'a>(
        &'a self,
        request: ModelRequest,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ModelResponse, ModelError>> + Send + 'a>,
    > {
        Box::pin(self.generate_inner(request))
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<ChatTemplateKwargs>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Debug, Serialize)]
struct ChatTemplateKwargs {
    enable_thinking: bool,
}

fn reasoning_controls(preference: Option<ModelReasoningPreference>) -> Option<ChatTemplateKwargs> {
    preference.map(|ModelReasoningPreference::Minimize| ChatTemplateKwargs {
        enable_thinking: false,
    })
}

fn response_format(format: ModelOutputFormat) -> Option<ResponseFormat> {
    match format {
        ModelOutputFormat::Text => None,
        ModelOutputFormat::JsonObject | ModelOutputFormat::JsonSchema { .. } => {
            Some(ResponseFormat {
                kind: "json_object",
            })
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    model: Option<String>,
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
        let text = match self {
            Self::Text(text) => text,
            Self::Chunks(chunks) => chunks.into_iter().filter_map(|chunk| chunk.text).collect(),
        };
        if text.trim().is_empty() {
            Err(ModelError::new(
                ModelErrorKind::Protocol,
                "NVIDIA response contained no model text output",
            ))
        } else {
            Ok(text)
        }
    }
}

#[derive(Debug, Deserialize)]
struct ResponseChunk {
    text: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

fn classify_transport_error(error: reqwest::Error) -> ModelError {
    let kind = if error.is_timeout() {
        ModelErrorKind::Timeout
    } else if error.is_connect() {
        ModelErrorKind::ProviderUnavailable
    } else {
        ModelErrorKind::Transport
    };
    let detail = match kind {
        ModelErrorKind::Timeout => "NVIDIA API request timed out".to_string(),
        ModelErrorKind::ProviderUnavailable => "NVIDIA API connection unavailable".to_string(),
        _ => format!("NVIDIA API request failed: {error}"),
    };
    ModelError::new(kind, detail)
}

fn classify_http_error(status: StatusCode, body: &str) -> ModelErrorKind {
    let normalized = body.to_ascii_lowercase();
    if status == StatusCode::TOO_MANY_REQUESTS {
        if ["quota", "credit", "balance", "insufficient"]
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
    } else if status == StatusCode::UNAUTHORIZED {
        ModelErrorKind::Credentials
    } else {
        ModelErrorKind::Provider
    }
}

fn rate_limit_delay(headers: &reqwest::header::HeaderMap, retry_index: usize) -> Duration {
    if let Some(value) = headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
    {
        if let Ok(seconds) = value.parse::<u64>() {
            return Duration::from_secs(seconds.max(1));
        }
        if let Ok(instant) = httpdate::parse_http_date(value) {
            return instant
                .duration_since(SystemTime::now())
                .unwrap_or(Duration::from_secs(1))
                .max(Duration::from_secs(1));
        }
    }

    let multiplier = 1u32.checked_shl(retry_index as u32).unwrap_or(u32::MAX);
    INITIAL_RATE_LIMIT_BACKOFF
        .checked_mul(multiplier)
        .unwrap_or(Duration::MAX)
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
    let normalized = value.replace(['\n', '\r'], " ");
    let mut chars = normalized.chars();
    let prefix = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_provider_neutral_json_mode_without_authority_fields() {
        let format = response_format(ModelOutputFormat::JsonSchema {
            name: "candidate".into(),
            schema: json!({"type": "object"}),
        });
        let value = serde_json::to_value(format).unwrap();
        assert_eq!(value["type"], "json_object");
        assert!(value.get("verdict").is_none());
    }

    #[test]
    fn parses_text_and_usage() {
        let response: ChatResponse = serde_json::from_value(json!({
            "model": "nvidia/nemotron-3.5-lightning-30b-a3b",
            "choices": [{
                "finish_reason": "stop",
                "message": {"content": "{\"claims\":[],\"inferences\":[]}"}
            }],
            "usage": {"prompt_tokens": 11, "completion_tokens": 5, "total_tokens": 16}
        }))
        .unwrap();
        let choice = response.choices.into_iter().next().unwrap();
        assert_eq!(
            choice.message.content.into_text().unwrap(),
            "{\"claims\":[],\"inferences\":[]}"
        );
        assert_eq!(response.usage.unwrap().total_tokens, Some(16));
    }

    #[test]
    fn rejects_empty_model_text() {
        let content = ResponseContent::Text("   ".into());
        let error = content.into_text().unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::Protocol);
    }

    #[test]
    fn classifies_rate_limit_quota_timeout_and_unavailable() {
        assert_eq!(
            classify_http_error(StatusCode::TOO_MANY_REQUESTS, "busy"),
            ModelErrorKind::RateLimit
        );
        assert_eq!(
            classify_http_error(StatusCode::TOO_MANY_REQUESTS, "quota exceeded"),
            ModelErrorKind::Quota
        );
        assert_eq!(
            classify_http_error(StatusCode::PAYMENT_REQUIRED, ""),
            ModelErrorKind::Quota
        );
        assert_eq!(
            classify_http_error(StatusCode::GATEWAY_TIMEOUT, ""),
            ModelErrorKind::Timeout
        );
        assert_eq!(
            classify_http_error(StatusCode::SERVICE_UNAVAILABLE, ""),
            ModelErrorKind::ProviderUnavailable
        );
    }

    #[test]
    fn retry_after_prefers_seconds_and_supports_http_date() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(RETRY_AFTER, reqwest::header::HeaderValue::from_static("7"));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(7));

        let future = SystemTime::now() + Duration::from_secs(3);
        let value = httpdate::fmt_http_date(future);
        headers.insert(
            RETRY_AFTER,
            reqwest::header::HeaderValue::from_str(&value).unwrap(),
        );
        let delay = rate_limit_delay(&headers, 0);
        assert!(delay >= Duration::from_secs(1));
        assert!(delay <= Duration::from_secs(4));
    }

    #[test]
    fn serializes_model_seed_and_json_mode_without_model_specific_branches() {
        let body = ChatRequest {
            model: "deepseek-ai/deepseek-v4-flash-0731",
            messages: vec![Message {
                role: "user",
                content: "return json".into(),
            }],
            response_format: response_format(ModelOutputFormat::JsonObject),
            max_tokens: Some(4096),
            seed: Some(42),
            chat_template_kwargs: None,
            temperature: 0.0,
            stream: false,
        };
        let value = serde_json::to_value(body).unwrap();
        assert_eq!(value["model"], "deepseek-ai/deepseek-v4-flash-0731");
        assert_eq!(value["seed"], 42);
        assert_eq!(value["response_format"]["type"], "json_object");
        assert!(value.get("reasoning_effort").is_none());
        assert!(value.get("chat_template_kwargs").is_none());
    }

    #[test]
    fn maps_minimized_reasoning_to_disable_thinking_without_changing_output_contract() {
        let controls = reasoning_controls(Some(ModelReasoningPreference::Minimize)).unwrap();
        let value = serde_json::to_value(controls).unwrap();
        assert_eq!(value["enable_thinking"], false);
        assert!(value.get("reasoning_budget").is_none());
    }

    #[tokio::test]
    async fn classifies_request_timeout_without_contacting_nvidia() {
        use std::{io::Read, net::TcpListener, thread};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer);
            thread::sleep(Duration::from_millis(150));
        });
        let adapter = NvidiaAdapter::with_base_url_and_timeout(
            "test-key",
            "nvidia/nemotron-3.5-lightning-30b-a3b",
            &format!("http://{address}/v1/"),
            Duration::from_millis(20),
        )
        .unwrap();
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
        assert_eq!(error.kind, ModelErrorKind::Timeout);
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
                        r#"{"model":"test-model","choices":[{"finish_reason":"stop","message":{"content":"ok"}}]}"#,
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
        let adapter = NvidiaAdapter::with_base_url_and_timeout(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
            Duration::from_secs(3),
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
        server.join().unwrap();
    }

    #[tokio::test]
    async fn classifies_malformed_success_response_as_protocol_failure() {
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
            let body = "{not-json";
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });
        let adapter = NvidiaAdapter::with_base_url_and_timeout(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
            Duration::from_secs(1),
        )
        .unwrap();
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
        assert_eq!(error.kind, ModelErrorKind::Protocol);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn classifies_connection_refusal_as_provider_unavailable() {
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let adapter = NvidiaAdapter::with_base_url_and_timeout(
            "test-key",
            "nvidia/nemotron-3.5-lightning-30b-a3b",
            &format!("http://{address}/v1/"),
            Duration::from_secs(1),
        )
        .unwrap();
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
        assert_eq!(error.kind, ModelErrorKind::ProviderUnavailable);
    }

    #[test]
    fn rejects_empty_credentials_without_echoing_them() {
        let error = NvidiaAdapter::new("", "nvidia/nemotron-3.5-lightning-30b-a3b")
            .err()
            .unwrap();
        assert_eq!(error.kind, ModelErrorKind::Credentials);
        assert!(!error.to_string().contains("Bearer"));
    }
}
