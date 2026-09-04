use std::{env, time::Duration};

use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelRequest, ModelResponse,
    ModelUsage,
};
use reqwest::{Client, StatusCode, Url, header::RETRY_AFTER};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);
const MAX_RATE_LIMIT_RETRIES: usize = 3;
const MAX_PROVIDER_ATTEMPTS: u32 = 4;
const MAX_TRANSIENT_RETRIES: usize = 2;
const MAX_EMPTY_TEXT_RETRIES: usize = 1;
const INITIAL_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(10);
const INITIAL_TRANSIENT_BACKOFF: Duration = Duration::from_millis(500);
const GOOGLE_RECOMMENDED_TEMPERATURE: f32 = 1.0;

/// Google Gemini API / AI Studio adapter for Google-hosted text models.
///
/// This adapter is intentionally limited to untrusted candidate generation. It never
/// participates in harness verification or verdict authority.
pub struct GoogleAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
}

impl GoogleAdapter {
    pub fn from_env(model: impl Into<String>) -> Result<Self, ModelError> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
            ModelError::new(ModelErrorKind::Credentials, "GEMINI_API_KEY is not set")
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
                "Gemini API key must not be empty",
            ));
        }
        if model.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Protocol,
                "Google model identifier must not be empty",
            ));
        }

        let base_url = Url::parse(base_url).map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Gemini API base URL: {error}"),
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
        let endpoint = self.base_url.join("interactions").map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("failed to construct Gemini Interactions endpoint: {error}"),
            )
        })?;

        let body = InteractionRequest {
            model: &self.model,
            input: request.task,
            system_instruction: request.system,
            response_format: response_format(request.output_format),
            generation_config: GenerationConfig {
                max_output_tokens: request.max_tokens,
                seed: request.random_seed,
                temperature: GOOGLE_RECOMMENDED_TEMPERATURE,
            },
            store: false,
        };

        let mut provider_attempts = 0u32;
        let mut rate_limit_retries = 0usize;
        let mut transient_retries = 0usize;
        let mut empty_text_retries = 0usize;

        loop {
            provider_attempts = provider_attempts.saturating_add(1);
            let response = self
                .client
                .post(endpoint.clone())
                .header("x-goog-api-key", &self.api_key)
                .json(&body)
                .send()
                .await
                .map_err(|error| {
                    let kind = if error.is_timeout() {
                        ModelErrorKind::Timeout
                    } else {
                        ModelErrorKind::Transport
                    };
                    let detail = if error.is_timeout() {
                        "Gemini API request timed out".to_string()
                    } else {
                        format!("Gemini API request failed: {error}")
                    };
                    ModelError::new(kind, detail).with_provider_attempts(provider_attempts)
                })?;

            let status = response.status();
            if !status.is_success() {
                let rate_limit_delay = rate_limit_delay(response.headers(), rate_limit_retries);
                let body = response.text().await.unwrap_or_default();
                let kind = classify_http_error(status, &body);

                if status == StatusCode::TOO_MANY_REQUESTS
                    && kind == ModelErrorKind::RateLimit
                    && rate_limit_retries < MAX_RATE_LIMIT_RETRIES
                    && provider_attempts < MAX_PROVIDER_ATTEMPTS
                {
                    rate_limit_retries += 1;
                    tokio::time::sleep(rate_limit_delay).await;
                    continue;
                }

                if is_transient_http_status(status)
                    && transient_retries < MAX_TRANSIENT_RETRIES
                    && provider_attempts < MAX_PROVIDER_ATTEMPTS
                {
                    let delay = transient_retry_delay(transient_retries);
                    transient_retries += 1;
                    tokio::time::sleep(delay).await;
                    continue;
                }

                let detail = google_error_detail(&body);
                return Err(ModelError::new(
                    kind,
                    format!(
                        "Gemini API returned HTTP {status} after {provider_attempts} provider attempts{detail}"
                    ),
                )
                .with_provider_attempts(provider_attempts));
            }

            let response: InteractionResponse = response.json().await.map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Protocol,
                    format!("invalid Gemini Interactions response: {error}"),
                )
                .with_provider_attempts(provider_attempts)
            })?;
            let text = match response.text() {
                Ok(text) => text,
                Err(error)
                    if error.kind == ModelErrorKind::Protocol
                        && empty_text_retries < MAX_EMPTY_TEXT_RETRIES
                        && provider_attempts < MAX_PROVIDER_ATTEMPTS =>
                {
                    empty_text_retries += 1;
                    tokio::time::sleep(transient_retry_delay(0)).await;
                    continue;
                }
                Err(error) => return Err(error.with_provider_attempts(provider_attempts)),
            };
            let usage = response.usage.unwrap_or_default();

            return Ok(ModelResponse {
                text,
                model: response.model.unwrap_or_else(|| self.model.clone()),
                usage: ModelUsage {
                    input_tokens: usage.total_input_tokens,
                    output_tokens: usage.total_output_tokens,
                    total_tokens: usage.total_tokens,
                },
                provider_attempts,
                finish_reason: response.status,
            });
        }
    }
}

impl ModelAdapter for GoogleAdapter {
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
struct InteractionRequest<'a> {
    model: &'a str,
    input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<String>,
    response_format: ResponseFormat,
    generation_config: GenerationConfig,
    store: bool,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
    mime_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<Value>,
}

fn classify_http_error(status: StatusCode, body: &str) -> ModelErrorKind {
    if status == StatusCode::TOO_MANY_REQUESTS {
        let normalized = body.to_ascii_lowercase();
        if [
            "quota",
            "billing",
            "credit",
            "resource_exhausted",
            "free_tier_requests",
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
    } else if status == StatusCode::UNAUTHORIZED {
        ModelErrorKind::Credentials
    } else {
        ModelErrorKind::Provider
    }
}

fn is_transient_http_status(status: StatusCode) -> bool {
    matches!(status.as_u16(), 500 | 502 | 503 | 504)
}

fn transient_retry_delay(retry_index: usize) -> Duration {
    let multiplier = 1u32.checked_shl(retry_index as u32).unwrap_or(u32::MAX);
    INITIAL_TRANSIENT_BACKOFF
        .checked_mul(multiplier)
        .unwrap_or(Duration::from_secs(2))
        .min(Duration::from_secs(2))
}

fn google_error_detail(body: &str) -> String {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return String::new();
    };
    let Some(error) = value.get("error") else {
        return String::new();
    };
    let provider_status = error.get("status").and_then(Value::as_str);
    let message = error.get("message").and_then(Value::as_str);
    match (provider_status, message) {
        (Some(status), Some(message)) => format!(
            "; provider_status={status}; message={}",
            truncate_diagnostic(message, 512)
        ),
        (Some(status), None) => format!("; provider_status={status}"),
        (None, Some(message)) => format!("; message={}", truncate_diagnostic(message, 512)),
        (None, None) => String::new(),
    }
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

fn rate_limit_delay(headers: &reqwest::header::HeaderMap, retry_index: usize) -> Duration {
    if let Some(seconds) = headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
    {
        return Duration::from_secs(seconds.max(1));
    }

    let multiplier = 1u32.checked_shl(retry_index as u32).unwrap_or(u32::MAX);
    INITIAL_RATE_LIMIT_BACKOFF
        .checked_mul(multiplier)
        .unwrap_or(Duration::MAX)
}

fn response_format(format: ModelOutputFormat) -> ResponseFormat {
    match format {
        ModelOutputFormat::Text => ResponseFormat {
            kind: "text",
            mime_type: "text/plain",
            schema: None,
        },
        ModelOutputFormat::JsonObject => ResponseFormat {
            kind: "text",
            mime_type: "application/json",
            schema: Some(json!({"type": "object"})),
        },
        ModelOutputFormat::JsonSchema { schema, .. } => ResponseFormat {
            kind: "text",
            mime_type: "application/json",
            schema: Some(schema),
        },
    }
}

#[derive(Debug, Deserialize)]
struct InteractionResponse {
    model: Option<String>,
    status: Option<String>,
    #[serde(default)]
    steps: Vec<InteractionStep>,
    usage: Option<Usage>,
}

impl InteractionResponse {
    fn text(&self) -> Result<String, ModelError> {
        let text = self
            .steps
            .iter()
            .filter(|step| step.kind.as_deref() == Some("model_output"))
            .flat_map(|step| step.content.iter())
            .filter(|content| content.kind.as_deref() == Some("text"))
            .filter_map(|content| content.text.as_deref())
            .collect::<String>();
        if text.is_empty() {
            Err(ModelError::new(
                ModelErrorKind::Protocol,
                "Gemini Interactions response contained no model text output",
            ))
        } else {
            Ok(text)
        }
    }
}

#[derive(Debug, Deserialize)]
struct InteractionStep {
    #[serde(rename = "type")]
    kind: Option<String>,
    #[serde(default)]
    content: Vec<InteractionContent>,
}

#[derive(Debug, Deserialize)]
struct InteractionContent {
    #[serde(rename = "type")]
    kind: Option<String>,
    text: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct Usage {
    total_input_tokens: Option<u64>,
    total_output_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_interactions_json_schema_without_authority_fields() {
        let value = serde_json::to_value(response_format(ModelOutputFormat::JsonSchema {
            name: "candidate".into(),
            schema: json!({"type": "object", "properties": {"claims": {"type": "array"}}}),
        }))
        .unwrap();

        assert_eq!(value["type"], "text");
        assert_eq!(value["mime_type"], "application/json");
        assert_eq!(value["schema"]["type"], "object");
        assert!(value.get("verdict").is_none());
    }

    #[test]
    fn parses_model_output_text_and_usage() {
        let response: InteractionResponse = serde_json::from_value(json!({
            "model": "gemma-4-26b-a4b-it",
            "status": "completed",
            "steps": [{
                "type": "model_output",
                "content": [{"type": "text", "text": "{\"claims\":[]}"}]
            }],
            "usage": {
                "total_input_tokens": 10,
                "total_output_tokens": 4,
                "total_tokens": 14
            }
        }))
        .unwrap();
        assert_eq!(response.text().unwrap(), "{\"claims\":[]}");
        assert_eq!(response.usage.unwrap().total_tokens, Some(14));
    }

    #[test]
    fn uses_google_recommended_sampling_temperature() {
        let value = serde_json::to_value(GenerationConfig {
            max_output_tokens: Some(4096),
            seed: Some(7),
            temperature: GOOGLE_RECOMMENDED_TEMPERATURE,
        })
        .unwrap();
        assert_eq!(value["temperature"], 1.0);
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
        assert_eq!(rate_limit_delay(&headers, 0), Duration::from_secs(10));
        assert_eq!(rate_limit_delay(&headers, 1), Duration::from_secs(20));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(40));
    }

    #[test]
    fn extracts_bounded_google_provider_error_detail() {
        let detail = google_error_detail(
            r#"{"error":{"code":400,"status":"INVALID_ARGUMENT","message":"bad request\nwithout secrets"}}"#,
        );
        assert!(detail.contains("provider_status=INVALID_ARGUMENT"));
        assert!(detail.contains("message=bad request without secrets"));
        assert!(!detail.contains('\n'));
    }

    #[test]
    fn classifies_google_quota_rate_limit_and_availability_errors() {
        assert_eq!(
            classify_http_error(
                StatusCode::TOO_MANY_REQUESTS,
                r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"Quota exceeded for free_tier_requests"}}"#,
            ),
            ModelErrorKind::Quota
        );
        assert_eq!(
            classify_http_error(StatusCode::TOO_MANY_REQUESTS, "slow down"),
            ModelErrorKind::RateLimit
        );
        assert_eq!(
            classify_http_error(StatusCode::SERVICE_UNAVAILABLE, "temporary"),
            ModelErrorKind::ProviderUnavailable
        );
        assert_eq!(
            classify_http_error(StatusCode::GATEWAY_TIMEOUT, "timeout"),
            ModelErrorKind::Timeout
        );
        assert_eq!(
            classify_http_error(StatusCode::UNAUTHORIZED, "bad key"),
            ModelErrorKind::Credentials
        );
    }

    #[test]
    fn rejects_empty_credentials_without_echoing_them() {
        let error = GoogleAdapter::new("", "gemma-4-26b-a4b-it").err().unwrap();
        assert_eq!(error.kind, ModelErrorKind::Credentials);
        assert!(!error.to_string().contains("x-goog-api-key"));
    }

    fn test_request() -> ModelRequest {
        ModelRequest {
            task: "test".into(),
            system: None,
            output_format: ModelOutputFormat::Text,
            max_tokens: Some(8),
            random_seed: None,
            reasoning_preference: None,
        }
    }

    fn success_body(text: &str) -> String {
        serde_json::json!({
            "model": "test-model",
            "status": "completed",
            "steps": [{
                "type": "model_output",
                "content": [{"type": "text", "text": text}]
            }],
            "usage": {
                "total_input_tokens": 2,
                "total_output_tokens": 1,
                "total_tokens": 3
            }
        })
        .to_string()
    }

    fn empty_text_body() -> String {
        serde_json::json!({
            "model": "test-model",
            "status": "completed",
            "steps": [],
            "usage": {
                "total_input_tokens": 2,
                "total_output_tokens": 0,
                "total_tokens": 2
            }
        })
        .to_string()
    }

    fn spawn_sequence_server(
        responses: Vec<(&'static str, String, &'static str)>,
    ) -> (String, std::thread::JoinHandle<()>) {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for (status, body, extra_headers) in responses {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{extra_headers}\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });
        (format!("http://{address}/v1beta/"), server)
    }

    #[tokio::test]
    async fn retries_http_500_then_returns_success_with_attempt_count() {
        let (base_url, server) = spawn_sequence_server(vec![
            (
                "500 Internal Server Error",
                r#"{"error":{"message":"high demand"}}"#.into(),
                "",
            ),
            ("200 OK", success_body("ok"), ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let response = adapter.generate(test_request()).await.unwrap();
        assert_eq!(response.text, "ok");
        assert_eq!(response.provider_attempts, 2);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn repeated_transient_5xx_fails_after_bounded_retries() {
        let busy = r#"{"error":{"message":"high demand"}}"#.to_string();
        let (base_url, server) = spawn_sequence_server(vec![
            ("503 Service Unavailable", busy.clone(), ""),
            ("503 Service Unavailable", busy.clone(), ""),
            ("503 Service Unavailable", busy, ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let error = adapter.generate(test_request()).await.unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::ProviderUnavailable);
        assert_eq!(error.provider_attempts, 3);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn retries_one_empty_model_text_then_returns_success() {
        let (base_url, server) = spawn_sequence_server(vec![
            ("200 OK", empty_text_body(), ""),
            ("200 OK", success_body("ok"), ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let response = adapter.generate(test_request()).await.unwrap();
        assert_eq!(response.text, "ok");
        assert_eq!(response.provider_attempts, 2);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn repeated_empty_model_text_remains_protocol_failure() {
        let (base_url, server) = spawn_sequence_server(vec![
            ("200 OK", empty_text_body(), ""),
            ("200 OK", empty_text_body(), ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let error = adapter.generate(test_request()).await.unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::Protocol);
        assert_eq!(error.provider_attempts, 2);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn deterministic_http_failures_and_quota_do_not_retry() {
        for (status, body, expected) in [
            (
                "400 Bad Request",
                r#"{"error":{"status":"INVALID_ARGUMENT","message":"bad request"}}"#,
                ModelErrorKind::Provider,
            ),
            (
                "401 Unauthorized",
                r#"{"error":{"status":"UNAUTHENTICATED","message":"bad key"}}"#,
                ModelErrorKind::Credentials,
            ),
            (
                "429 Too Many Requests",
                r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"Quota exceeded for free_tier_requests"}}"#,
                ModelErrorKind::Quota,
            ),
        ] {
            let (base_url, server) = spawn_sequence_server(vec![(status, body.to_string(), "")]);
            let adapter =
                GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
            let error = adapter.generate(test_request()).await.unwrap_err();
            assert_eq!(error.kind, expected);
            assert_eq!(error.provider_attempts, 1);
            server.join().unwrap();
        }
    }

    #[tokio::test]
    async fn rate_limit_retry_remains_compatible_and_observable() {
        let (base_url, server) = spawn_sequence_server(vec![
            (
                "429 Too Many Requests",
                r#"{"error":{"message":"slow down"}}"#.into(),
                "Retry-After: 1\r\n",
            ),
            ("200 OK", success_body("ok"), ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let response = adapter.generate(test_request()).await.unwrap();
        assert_eq!(response.text, "ok");
        assert_eq!(response.provider_attempts, 2);
        server.join().unwrap();
    }
}
