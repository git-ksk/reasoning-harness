use std::{
    env,
    time::{Duration, SystemTime},
};

use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelRequest, ModelResponse,
    ModelUsage,
};
use reqwest::{Client, StatusCode, Url, header::RETRY_AFTER};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "https://api.mistral.ai/v1/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_RATE_LIMIT_RETRIES: usize = 5;
const INITIAL_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(5);
const MAX_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(60);
const MAX_SUCCESS_PACING_DELAY: Duration = Duration::from_secs(90);
const RATE_LIMIT_TELEMETRY_ENV: &str = "REASON_MISTRAL_RATE_LIMIT_TELEMETRY";

pub struct MistralAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
}

impl MistralAdapter {
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
        let client = Client::builder()
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
        })
    }

    async fn generate_inner(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
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

        let mut rate_limit_retries = 0usize;
        let response = loop {
            let response = self
                .client
                .post(endpoint.clone())
                .bearer_auth(&self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(classify_transport_error)?;

            log_rate_limit_telemetry(response.status(), response.headers(), rate_limit_retries);

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
        let limit_detail = if status == StatusCode::TOO_MANY_REQUESTS {
            rate_limit_header_detail(response.headers())
        } else {
            String::new()
        };
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            let kind = classify_http_error(status, &body);
            let detail = provider_error_detail(&body);
            return Err(ModelError::new(
                kind,
                format!(
                    "Mistral returned HTTP {status} after {rate_limit_retries} rate-limit retries{detail}{limit_detail}"
                ),
            ));
        }

        let pacing_delay = success_pacing_delay(response.headers());
        let response: ChatResponse = response.json().await.map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Mistral response: {error}"),
            )
        })?;
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            ModelError::new(
                ModelErrorKind::Protocol,
                "Mistral response contained no choices",
            )
        })?;

        let model_response = ModelResponse {
            text: choice.message.content.into_text()?,
            model: response.model,
            usage: ModelUsage {
                input_tokens: response
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.prompt_tokens),
                output_tokens: response
                    .usage
                    .as_ref()
                    .and_then(|usage| usage.completion_tokens),
                total_tokens: response.usage.and_then(|usage| usage.total_tokens),
            },
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
            tokio::time::sleep(delay).await;
        }

        Ok(model_response)
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
        if ["quota", "billing", "credit", "insufficient balance"]
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
    if let Some(delay) = retry_after_delay(headers) {
        return delay;
    }
    if let Some(delay) = rate_limit_reset_delay(headers) {
        return delay.min(MAX_SUCCESS_PACING_DELAY);
    }

    fallback_rate_limit_delay(retry_index)
}

fn retry_after_delay(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?;
    parse_reset_duration(value)
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
        return Some(Duration::from_secs_f64((milliseconds / 1000.0).max(0.001)));
    }
    if let Some(seconds) = value.strip_suffix('s') {
        let seconds = seconds.trim().parse::<f64>().ok()?;
        return Some(Duration::from_secs_f64(seconds.max(1.0)));
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

fn rate_limit_header_detail(headers: &reqwest::header::HeaderMap) -> String {
    let summary = rate_limit_header_summary(headers);
    if summary.is_empty() {
        String::new()
    } else {
        format!("; rate_limit_headers={summary}")
    }
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
    fn rate_limit_delay_prefers_retry_after_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(RETRY_AFTER, reqwest::header::HeaderValue::from_static("7"));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(7));
    }

    #[test]
    fn rate_limit_delay_uses_bounded_exponential_fallback() {
        let headers = reqwest::header::HeaderMap::new();
        assert_eq!(rate_limit_delay(&headers, 0), Duration::from_secs(5));
        assert_eq!(rate_limit_delay(&headers, 1), Duration::from_secs(10));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(20));
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
                Duration::from_secs(5),
                Duration::from_secs(10),
                Duration::from_secs(20),
                Duration::from_secs(40),
                Duration::from_secs(60),
            ]
        );
        assert!(delays.iter().take(4).sum::<Duration>() > Duration::from_secs(60));
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
        server.join().unwrap();
    }
}
