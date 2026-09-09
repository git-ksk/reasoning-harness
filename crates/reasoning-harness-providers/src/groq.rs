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

const DEFAULT_BASE_URL: &str = "https://api.groq.com/openai/v1/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(180);
const MAX_RATE_LIMIT_RETRIES: usize = 5;
const MAX_STRUCTURED_OUTPUT_RETRIES: usize = 2;
const INITIAL_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(5);
const MAX_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(60);
const MAX_RATE_LIMIT_RESET_DELAY: Duration = Duration::from_secs(180);
const MIN_REQUEST_INTERVAL_ENV: &str = "REASON_GROQ_MIN_REQUEST_INTERVAL_MS";
const TOKENS_PER_MINUTE_ENV: &str = "REASON_GROQ_TOKENS_PER_MINUTE";
const RATE_LIMIT_TELEMETRY_ENV: &str = "REASON_GROQ_RATE_LIMIT_TELEMETRY";

/// GroqCloud adapter using the provider's OpenAI-compatible Chat Completions API.
///
/// Model output remains an untrusted candidate. This adapter never participates in
/// deterministic verification, evidence admission, or final verdict selection.
pub struct GroqAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
    min_request_interval: Duration,
    tokens_per_minute: Option<u64>,
    pacing: tokio::sync::Mutex<PacingState>,
}

#[derive(Debug, Default)]
struct PacingState {
    last_request_started: Option<Instant>,
    next_token_slot: Option<Instant>,
}

impl GroqAdapter {
    pub fn from_env(model: impl Into<String>) -> Result<Self, ModelError> {
        let api_key = env::var("GROQ_API_KEY")
            .map_err(|_| ModelError::new(ModelErrorKind::Credentials, "GROQ_API_KEY is not set"))?;
        let min_request_interval = env_u64(MIN_REQUEST_INTERVAL_ENV)?
            .map(Duration::from_millis)
            .unwrap_or(Duration::ZERO);
        let tokens_per_minute = env_u64(TOKENS_PER_MINUTE_ENV)?.filter(|value| *value > 0);
        Self::with_base_url_timeout_and_pacing(
            api_key,
            model,
            DEFAULT_BASE_URL,
            DEFAULT_TIMEOUT,
            min_request_interval,
            tokens_per_minute,
        )
    }

    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, ModelError> {
        Self::with_base_url_timeout_and_pacing(
            api_key,
            model,
            DEFAULT_BASE_URL,
            DEFAULT_TIMEOUT,
            Duration::ZERO,
            None,
        )
    }

    #[cfg(test)]
    fn with_base_url_and_timeout(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: &str,
        timeout: Duration,
    ) -> Result<Self, ModelError> {
        Self::with_base_url_timeout_and_pacing(
            api_key,
            model,
            base_url,
            timeout,
            Duration::ZERO,
            None,
        )
    }

    #[cfg(test)]
    fn with_test_pacing(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: &str,
        min_request_interval: Duration,
        tokens_per_minute: Option<u64>,
    ) -> Result<Self, ModelError> {
        Self::with_base_url_timeout_and_pacing(
            api_key,
            model,
            base_url,
            Duration::from_secs(3),
            min_request_interval,
            tokens_per_minute,
        )
    }

    fn with_base_url_timeout_and_pacing(
        api_key: impl Into<String>,
        model: impl Into<String>,
        base_url: &str,
        timeout: Duration,
        min_request_interval: Duration,
        tokens_per_minute: Option<u64>,
    ) -> Result<Self, ModelError> {
        let api_key = api_key.into();
        let model = model.into();
        if api_key.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Credentials,
                "Groq API key must not be empty",
            ));
        }
        if model.trim().is_empty() {
            return Err(ModelError::new(
                ModelErrorKind::Protocol,
                "Groq model identifier must not be empty",
            ));
        }
        let base_url = Url::parse(base_url).map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Groq API base URL: {error}"),
            )
        })?;
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Transport,
                    format!("failed to build Groq HTTP client: {error}"),
                )
            })?;
        Ok(Self {
            client,
            api_key,
            base_url,
            model,
            min_request_interval,
            tokens_per_minute,
            pacing: tokio::sync::Mutex::new(PacingState::default()),
        })
    }

    async fn wait_for_request_slot(&self) {
        if self.min_request_interval.is_zero() && self.tokens_per_minute.is_none() {
            return;
        }
        let mut pacing = self.pacing.lock().await;
        let now = Instant::now();
        let request_slot = pacing
            .last_request_started
            .and_then(|last| last.checked_add(self.min_request_interval));
        let target = [request_slot, pacing.next_token_slot]
            .into_iter()
            .flatten()
            .max();
        if let Some(target) = target {
            if target > now {
                tokio::time::sleep(target - now).await;
            }
        }
        pacing.last_request_started = Some(Instant::now());
    }

    async fn record_token_usage(&self, total_tokens: Option<u64>) {
        let (Some(tokens_per_minute), Some(total_tokens)) = (self.tokens_per_minute, total_tokens)
        else {
            return;
        };
        if tokens_per_minute == 0 || total_tokens == 0 {
            return;
        }
        let delay_secs = (total_tokens as f64 * 60.0) / tokens_per_minute as f64;
        let delay = Duration::from_secs_f64(delay_secs.max(0.001));
        let mut pacing = self.pacing.lock().await;
        let base = pacing.last_request_started.unwrap_or_else(Instant::now);
        let next = base.checked_add(delay).unwrap_or_else(Instant::now);
        pacing.next_token_slot = Some(match pacing.next_token_slot {
            Some(existing) => existing.max(next),
            None => next,
        });
    }

    async fn generate_inner(&self, request: ModelRequest) -> Result<ModelResponse, ModelError> {
        let endpoint = self.base_url.join("chat/completions").map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("failed to construct Groq endpoint: {error}"),
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

        let best_effort_structured_json = matches!(
            &request.output_format,
            ModelOutputFormat::JsonSchema { .. } | ModelOutputFormat::JsonObject
        );
        let reasoning = reasoning_controls(&self.model, request.reasoning_preference);
        let body = ChatRequest {
            model: &self.model,
            messages,
            response_format: response_format(request.output_format),
            max_completion_tokens: request.max_tokens,
            seed: request.random_seed,
            reasoning_effort: reasoning.effort,
            include_reasoning: reasoning.include_reasoning,
            temperature: 0.0,
            stream: false,
        };

        let mut rate_limit_retries = 0usize;
        let mut structured_output_retries = 0usize;
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

            let status = response.status();
            log_rate_limit_telemetry(status, response.headers(), rate_limit_retries);

            if status == StatusCode::TOO_MANY_REQUESTS
                && rate_limit_retries < MAX_RATE_LIMIT_RETRIES
            {
                let delay = rate_limit_delay(response.headers(), rate_limit_retries);
                rate_limit_retries += 1;
                tokio::time::sleep(delay).await;
                continue;
            }

            if status == StatusCode::BAD_REQUEST && best_effort_structured_json {
                let error_body = response.text().await.unwrap_or_default();
                if is_retryable_structured_output_error(&error_body) {
                    if structured_output_retries < MAX_STRUCTURED_OUTPUT_RETRIES {
                        log_structured_output_retry(structured_output_retries);
                        structured_output_retries += 1;
                        continue;
                    }
                    let detail = provider_error_detail(&error_body);
                    return Err(ModelError::new(
                        ModelErrorKind::UnsupportedCapability,
                        format!(
                            "Groq structured JSON generation remained unavailable after {structured_output_retries} structured-output retries{detail}"
                        ),
                    )
                    .with_provider_attempts(provider_attempts(
                        rate_limit_retries,
                        structured_output_retries,
                    )));
                }
                return Err(http_error(
                    status,
                    &error_body,
                    rate_limit_retries,
                    structured_output_retries,
                    String::new(),
                ));
            }

            break response;
        };

        let status = response.status();
        let limit_detail = if status == StatusCode::TOO_MANY_REQUESTS {
            rate_limit_header_detail(response.headers())
        } else {
            String::new()
        };
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(http_error(
                status,
                &error_body,
                rate_limit_retries,
                structured_output_retries,
                limit_detail,
            ));
        }

        let response: ChatResponse = response.json().await.map_err(|error| {
            ModelError::new(
                ModelErrorKind::Protocol,
                format!("invalid Groq Chat Completions response: {error}"),
            )
            .with_provider_attempts(provider_attempts(
                rate_limit_retries,
                structured_output_retries,
            ))
        })?;
        let total_tokens = response.usage.as_ref().and_then(|usage| usage.total_tokens);
        self.record_token_usage(total_tokens).await;
        let choice = response.choices.into_iter().next().ok_or_else(|| {
            ModelError::new(
                ModelErrorKind::Protocol,
                "Groq response contained no choices",
            )
            .with_provider_attempts(provider_attempts(
                rate_limit_retries,
                structured_output_retries,
            ))
        })?;
        let usage = response.usage.unwrap_or_default();
        let finish_reason = choice.finish_reason;
        let reasoning_present = choice
            .message
            .reasoning
            .as_ref()
            .is_some_and(|value| !reasoning_value_is_empty(value));
        let text = choice
            .message
            .content
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| {
                ModelError::new(
                    ModelErrorKind::Protocol,
                    format!(
                        "Groq response contained no model text output; finish_reason={}; completion_tokens={}; total_tokens={}; reasoning_present={reasoning_present}",
                        bounded_finish_reason(finish_reason.as_deref()),
                        optional_token_count(usage.completion_tokens),
                        optional_token_count(usage.total_tokens),
                    ),
                )
                .with_provider_attempts(provider_attempts(
                    rate_limit_retries,
                    structured_output_retries,
                ))
            })?;

        Ok(ModelResponse {
            text,
            model: response.model.unwrap_or_else(|| self.model.clone()),
            usage: ModelUsage {
                input_tokens: usage.prompt_tokens,
                output_tokens: usage.completion_tokens,
                total_tokens: usage.total_tokens,
            },
            provider_attempts: provider_attempts(rate_limit_retries, structured_output_retries),
            finish_reason,
        })
    }
}

impl ModelAdapter for GroqAdapter {
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
    max_completion_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_effort: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_reasoning: Option<bool>,
    temperature: f32,
    stream: bool,
}

#[derive(Debug, Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseFormat {
    JsonObject,
    JsonSchema { json_schema: JsonSchema },
}

#[derive(Debug, Serialize)]
struct JsonSchema {
    name: String,
    schema: Value,
    strict: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ReasoningControls {
    effort: Option<&'static str>,
    include_reasoning: Option<bool>,
}

fn reasoning_controls(
    model: &str,
    preference: Option<ModelReasoningPreference>,
) -> ReasoningControls {
    if preference == Some(ModelReasoningPreference::Minimize)
        && matches!(model, "openai/gpt-oss-20b" | "openai/gpt-oss-120b")
    {
        return ReasoningControls {
            effort: Some("low"),
            include_reasoning: Some(false),
        };
    }
    ReasoningControls::default()
}

fn reasoning_value_is_empty(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(text) => text.trim().is_empty(),
        Value::Array(values) => values.is_empty(),
        Value::Object(values) => values.is_empty(),
        _ => false,
    }
}

fn bounded_finish_reason(reason: Option<&str>) -> String {
    reason
        .map(|reason| truncate_diagnostic(reason, 64))
        .unwrap_or_else(|| "unknown".into())
}

fn optional_token_count(value: Option<u64>) -> String {
    value.map_or_else(|| "unknown".into(), |value| value.to_string())
}

fn response_format(format: ModelOutputFormat) -> Option<ResponseFormat> {
    match format {
        ModelOutputFormat::Text => None,
        ModelOutputFormat::JsonObject => Some(ResponseFormat::JsonObject),
        ModelOutputFormat::JsonSchema { name, schema } => Some(ResponseFormat::JsonSchema {
            json_schema: JsonSchema {
                name,
                schema,
                // v4/Harness schemas intentionally permit optional fields and are validated
                // again by the Harness-owned parser. Groq strict mode requires every object
                // to be closed and every property required, so use provider best-effort schema
                // mode without changing the Harness schema or semantic contract.
                strict: false,
            },
        }),
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
    content: Option<String>,
    #[serde(default)]
    reasoning: Option<Value>,
}

#[derive(Debug, Default, Deserialize)]
struct Usage {
    prompt_tokens: Option<u64>,
    completion_tokens: Option<u64>,
    total_tokens: Option<u64>,
}

fn env_u64(name: &str) -> Result<Option<u64>, ModelError> {
    let Ok(value) = env::var(name) else {
        return Ok(None);
    };
    let parsed = value.trim().parse::<u64>().map_err(|_| {
        ModelError::new(
            ModelErrorKind::Protocol,
            format!("{name} must be a non-negative integer"),
        )
    })?;
    Ok(Some(parsed))
}

fn provider_attempts(rate_limit_retries: usize, structured_output_retries: usize) -> u32 {
    u32::try_from(
        rate_limit_retries
            .saturating_add(structured_output_retries)
            .saturating_add(1),
    )
    .unwrap_or(u32::MAX)
}

fn is_retryable_structured_output_error(body: &str) -> bool {
    let normalized = body.to_ascii_lowercase();
    normalized.contains("failed to validate json")
        || normalized.contains("failed to generate json")
        || normalized.contains("generated json does not match the expected schema")
}

fn log_structured_output_retry(retry_index: usize) {
    if rate_limit_telemetry_enabled() {
        eprintln!(
            "[groq-structured-output] status=400 retry_index={retry_index} reason=schema_validation"
        );
    }
}

fn http_error(
    status: StatusCode,
    body: &str,
    rate_limit_retries: usize,
    structured_output_retries: usize,
    limit_detail: String,
) -> ModelError {
    let kind = classify_http_error(status, body);
    let detail = provider_error_detail(body);
    ModelError::new(
        kind,
        format!(
            "Groq API returned HTTP {status} after {rate_limit_retries} rate-limit retries and {structured_output_retries} structured-output retries{detail}{limit_detail}"
        ),
    )
    .with_provider_attempts(provider_attempts(
        rate_limit_retries,
        structured_output_retries,
    ))
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
        ModelErrorKind::Timeout => "Groq API request timed out".to_string(),
        ModelErrorKind::ProviderUnavailable => "Groq API connection unavailable".to_string(),
        _ => format!("Groq API request failed: {error}"),
    };
    ModelError::new(kind, detail)
}

fn classify_http_error(status: StatusCode, body: &str) -> ModelErrorKind {
    let normalized = body.to_ascii_lowercase();
    if status == StatusCode::TOO_MANY_REQUESTS {
        if [
            "quota",
            "credit",
            "balance",
            "daily limit",
            "tokens per day",
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

fn rate_limit_delay(headers: &reqwest::header::HeaderMap, retry_index: usize) -> Duration {
    if let Some(value) = header_str(headers, RETRY_AFTER.as_str()) {
        if let Some(delay) = parse_reset_duration(value) {
            return delay.min(MAX_RATE_LIMIT_RESET_DELAY);
        }
    }
    for name in ["x-ratelimit-reset-tokens", "x-ratelimit-reset-requests"] {
        if let Some(delay) = header_str(headers, name).and_then(parse_reset_duration) {
            return delay.min(MAX_RATE_LIMIT_RESET_DELAY);
        }
    }
    let multiplier = 1u32.checked_shl(retry_index as u32).unwrap_or(u32::MAX);
    INITIAL_RATE_LIMIT_BACKOFF
        .checked_mul(multiplier)
        .unwrap_or(MAX_RATE_LIMIT_BACKOFF)
        .min(MAX_RATE_LIMIT_BACKOFF)
}

fn parse_reset_duration(value: &str) -> Option<Duration> {
    let value = value.trim();
    if let Some(ms) = value.strip_suffix("ms") {
        let ms = ms.trim().parse::<f64>().ok()?;
        return Some(Duration::from_secs_f64((ms / 1000.0).max(0.001)));
    }
    if let Some(minutes_at) = value.find('m') {
        let minutes = value[..minutes_at].trim().parse::<f64>().ok()?;
        let rest = &value[minutes_at + 1..];
        let seconds = rest
            .strip_suffix('s')
            .unwrap_or(rest)
            .trim()
            .parse::<f64>()
            .ok()?;
        return Some(Duration::from_secs_f64(
            (minutes * 60.0 + seconds).max(0.001),
        ));
    }
    if let Some(seconds) = value.strip_suffix('s') {
        let seconds = seconds.trim().parse::<f64>().ok()?;
        return Some(Duration::from_secs_f64(seconds.max(0.001)));
    }
    if let Ok(raw) = value.parse::<u64>() {
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

fn header_str<'a>(headers: &'a reqwest::header::HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

fn rate_limit_header_summary(headers: &reqwest::header::HeaderMap) -> String {
    [
        "retry-after",
        "x-ratelimit-limit-requests",
        "x-ratelimit-remaining-requests",
        "x-ratelimit-reset-requests",
        "x-ratelimit-limit-tokens",
        "x-ratelimit-remaining-tokens",
        "x-ratelimit-reset-tokens",
    ]
    .iter()
    .filter_map(|name| {
        header_str(headers, name).map(|value| format!("{name}={}", truncate_diagnostic(value, 96)))
    })
    .collect::<Vec<_>>()
    .join(",")
}

fn rate_limit_header_detail(headers: &reqwest::header::HeaderMap) -> String {
    let summary = rate_limit_header_summary(headers);
    if summary.is_empty() {
        String::new()
    } else {
        format!("; rate_limit_headers={summary}")
    }
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
            "[groq-rate-limit] status={} retry_index={} {}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_best_effort_json_schema_without_rewriting_harness_schema() {
        let format = response_format(ModelOutputFormat::JsonSchema {
            name: "candidate".into(),
            schema: json!({"type": "object", "properties": {}}),
        });
        let value = serde_json::to_value(format).unwrap();
        assert_eq!(value["type"], "json_schema");
        assert_eq!(value["json_schema"]["name"], "candidate");
        assert_eq!(value["json_schema"]["strict"], false);
        assert!(value.get("verdict").is_none());
    }

    #[test]
    fn serializes_openai_compatible_request_without_model_specific_controls() {
        let body = ChatRequest {
            model: "qwen/qwen3.8-27b",
            messages: vec![Message {
                role: "user",
                content: "return json".into(),
            }],
            response_format: response_format(ModelOutputFormat::JsonObject),
            max_completion_tokens: Some(1024),
            seed: Some(42),
            reasoning_effort: None,
            include_reasoning: None,
            temperature: 0.0,
            stream: false,
        };
        let value = serde_json::to_value(body).unwrap();
        assert_eq!(value["model"], "qwen/qwen3.8-27b");
        assert_eq!(value["seed"], 42);
        assert_eq!(value["max_completion_tokens"], 1024);
        assert!(value.get("reasoning_effort").is_none());
    }

    #[test]
    fn minimizes_reasoning_only_for_groq_gpt_oss_models() {
        for model in ["openai/gpt-oss-20b", "openai/gpt-oss-120b"] {
            let controls = reasoning_controls(model, Some(ModelReasoningPreference::Minimize));
            assert_eq!(controls.effort, Some("low"));
            assert_eq!(controls.include_reasoning, Some(false));
        }
        assert_eq!(
            reasoning_controls("qwen/qwen3.8-27b", Some(ModelReasoningPreference::Minimize)),
            ReasoningControls::default()
        );
        assert_eq!(
            reasoning_controls("openai/gpt-oss-120b", None),
            ReasoningControls::default()
        );
    }

    #[tokio::test]
    async fn empty_content_reports_bounded_terminal_metadata_without_reasoning_text() {
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
            let body = r#"{"model":"openai/gpt-oss-120b","choices":[{"finish_reason":"length","message":{"content":null,"reasoning":"PRIVATE_REASONING_MUST_NOT_LEAK"}}],"usage":{"prompt_tokens":44,"completion_tokens":256,"total_tokens":300}}"#;
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });
        let adapter = GroqAdapter::with_base_url_and_timeout(
            "test-key",
            "openai/gpt-oss-120b",
            &format!("http://{address}/v1/"),
            Duration::from_secs(2),
        )
        .unwrap();
        let error = adapter
            .generate(ModelRequest {
                task: "return one json object".into(),
                system: None,
                output_format: ModelOutputFormat::Text,
                max_tokens: Some(256),
                random_seed: Some(7),
                reasoning_preference: Some(ModelReasoningPreference::Minimize),
            })
            .await
            .unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::Protocol);
        assert!(error.message.contains("finish_reason=length"));
        assert!(error.message.contains("completion_tokens=256"));
        assert!(error.message.contains("total_tokens=300"));
        assert!(error.message.contains("reasoning_present=true"));
        assert!(!error.message.contains("PRIVATE_REASONING_MUST_NOT_LEAK"));
        server.join().unwrap();
    }

    #[test]
    fn parses_groq_reset_durations() {
        assert_eq!(
            parse_reset_duration("7.66s"),
            Some(Duration::from_secs_f64(7.66))
        );
        assert_eq!(
            parse_reset_duration("2m59.56s"),
            Some(Duration::from_secs_f64(179.56))
        );
        assert_eq!(
            parse_reset_duration("1500ms"),
            Some(Duration::from_secs_f64(1.5))
        );
    }

    #[test]
    fn classifies_rate_limit_quota_and_credentials() {
        assert_eq!(
            classify_http_error(StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded"),
            ModelErrorKind::RateLimit
        );
        assert_eq!(
            classify_http_error(StatusCode::TOO_MANY_REQUESTS, "daily limit quota exhausted"),
            ModelErrorKind::Quota
        );
        assert_eq!(
            classify_http_error(StatusCode::UNAUTHORIZED, ""),
            ModelErrorKind::Credentials
        );
    }

    #[test]
    fn extracts_safe_rate_limit_headers_for_diagnostics() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("x-ratelimit-limit-requests", "1000".parse().unwrap());
        headers.insert("x-ratelimit-remaining-tokens", "7990".parse().unwrap());
        headers.insert("x-ratelimit-reset-tokens", "7.66s".parse().unwrap());
        let detail = rate_limit_header_detail(&headers);
        assert!(detail.contains("x-ratelimit-limit-requests=1000"));
        assert!(detail.contains("x-ratelimit-remaining-tokens=7990"));
        assert!(!detail.contains("Authorization"));
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
                        "Retry-After: 0.01s\r\nX-RateLimit-Limit-Tokens: 8000\r\n",
                        r#"{"error":{"message":"rate limit exceeded"}}"#,
                    )
                } else {
                    (
                        "200 OK",
                        "X-RateLimit-Limit-Tokens: 8000\r\nX-RateLimit-Remaining-Tokens: 7900\r\n",
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
        let adapter = GroqAdapter::with_base_url_and_timeout(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
            Duration::from_secs(2),
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
        assert_eq!(response.provider_attempts, 2);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn retries_best_effort_schema_validation_failure_then_returns_success() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for attempt in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                let (status, body) = if attempt < 2 {
                    (
                        "400 Bad Request",
                        r#"{"error":{"message":"Failed to validate JSON. Please adjust your prompt. See 'failed_generation' for more details."}}"#,
                    )
                } else {
                    (
                        "200 OK",
                        r#"{"model":"test-model","choices":[{"finish_reason":"stop","message":{"content":"{}"}}],"usage":{"prompt_tokens":3,"completion_tokens":1,"total_tokens":4}}"#,
                    )
                };
                write!(
                    stream,
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });
        let adapter = GroqAdapter::with_base_url_and_timeout(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
            Duration::from_secs(2),
        )
        .unwrap();
        let response = adapter
            .generate(ModelRequest {
                task: "return json".into(),
                system: None,
                output_format: ModelOutputFormat::JsonSchema {
                    name: "test".into(),
                    schema: json!({"type":"object"}),
                },
                max_tokens: Some(8),
                random_seed: Some(1),
                reasoning_preference: None,
            })
            .await
            .unwrap();
        assert_eq!(response.text, "{}");
        assert_eq!(response.provider_attempts, 3);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn exhausted_structured_output_retries_surface_typed_capability_failure() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for _ in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                let body = r#"{"error":{"message":"Failed to generate JSON. Please adjust your prompt."}}"#;
                write!(
                    stream,
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });
        let adapter = GroqAdapter::with_base_url_and_timeout(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
            Duration::from_secs(2),
        )
        .unwrap();
        let error = adapter
            .generate(ModelRequest {
                task: "return json".into(),
                system: None,
                output_format: ModelOutputFormat::JsonSchema {
                    name: "test".into(),
                    schema: json!({"type":"object"}),
                },
                max_tokens: Some(8),
                random_seed: Some(1),
                reasoning_preference: None,
            })
            .await
            .unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::UnsupportedCapability);
        assert_eq!(error.provider_attempts, 3);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn exhausted_json_object_generation_retries_surface_typed_capability_failure() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            thread,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for _ in 0..3 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 8192];
                let _ = stream.read(&mut buffer);
                let body = r#"{"error":{"message":"Failed to generate JSON. Please adjust your prompt."}}"#;
                write!(
                    stream,
                    "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
            }
        });
        let adapter = GroqAdapter::with_base_url_and_timeout(
            "test-key",
            "test-model",
            &format!("http://{address}/v1/"),
            Duration::from_secs(2),
        )
        .unwrap();
        let error = adapter
            .generate(ModelRequest {
                task: "return json".into(),
                system: None,
                output_format: ModelOutputFormat::JsonObject,
                max_tokens: Some(8),
                random_seed: Some(1),
                reasoning_preference: None,
            })
            .await
            .unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::UnsupportedCapability);
        assert_eq!(error.provider_attempts, 3);
        server.join().unwrap();
    }

    #[test]
    fn structured_output_retry_matches_only_validation_errors() {
        assert!(is_retryable_structured_output_error(
            r#"{"error":{"message":"Failed to validate JSON. Please adjust your prompt."}}"#
        ));
        assert!(is_retryable_structured_output_error(
            r#"{"error":{"message":"Generated JSON does not match the expected schema."}}"#
        ));
        assert!(is_retryable_structured_output_error(
            r#"{"error":{"message":"Failed to generate JSON. Please adjust your prompt."}}"#
        ));
        assert!(!is_retryable_structured_output_error(
            r#"{"error":{"message":"unsupported parameter"}}"#
        ));
    }

    #[tokio::test]
    async fn token_pacing_uses_actual_previous_usage() {
        let adapter = GroqAdapter::with_test_pacing(
            "test-key",
            "test-model",
            "http://127.0.0.1:1/v1/",
            Duration::ZERO,
            Some(60_000),
        )
        .unwrap();
        {
            let mut pacing = adapter.pacing.lock().await;
            pacing.last_request_started = Some(Instant::now());
        }
        adapter.record_token_usage(Some(100)).await;
        let pacing = adapter.pacing.lock().await;
        let delay = pacing
            .next_token_slot
            .unwrap()
            .saturating_duration_since(pacing.last_request_started.unwrap());
        assert!(delay >= Duration::from_millis(99));
        assert!(delay <= Duration::from_millis(110));
    }

    #[test]
    fn rejects_empty_credentials_without_echoing_them() {
        let error = GroqAdapter::new("", "openai/gpt-oss-120b").err().unwrap();
        assert_eq!(error.kind, ModelErrorKind::Credentials);
        assert!(!error.to_string().contains("Bearer"));
    }
}
