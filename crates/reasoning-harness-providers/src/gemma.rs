use std::{
    env, fs,
    fs::OpenOptions,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reasoning_harness_core::{
    ModelAdapter, ModelError, ModelErrorKind, ModelOutputFormat, ModelRequest, ModelResponse,
    ModelUsage,
};
use reqwest::{Client, StatusCode, Url, header::RETRY_AFTER};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);
const MAX_RATE_LIMIT_RETRIES: usize = 3;
const MAX_PROVIDER_ATTEMPTS: u32 = 4;
const MAX_TRANSIENT_RETRIES: usize = MAX_PROVIDER_ATTEMPTS as usize - 1;
const MAX_EMPTY_TEXT_RETRIES: usize = MAX_PROVIDER_ATTEMPTS as usize - 1;
const INITIAL_RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(10);
const TRANSIENT_BACKOFF_SCHEDULE: [Duration; 3] = [
    Duration::from_secs(2),
    Duration::from_secs(5),
    Duration::from_secs(10),
];
const GOOGLE_RECOMMENDED_TEMPERATURE: f32 = 1.0;
const GOOGLE_MIN_REQUEST_INTERVAL_MS_ENV: &str = "REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS";
const GOOGLE_SHARED_PACER_PATH_ENV: &str = "REASON_GOOGLE_SHARED_PACER_PATH";
const GOOGLE_ATTEMPT_TELEMETRY_PATH_ENV: &str = "REASON_GOOGLE_ATTEMPT_TELEMETRY_PATH";
const MAX_CONFIGURED_REQUEST_INTERVAL: Duration = Duration::from_secs(60);
const MAX_STRUCTURED_SHORT_RETRY_DELAY: Duration = Duration::from_secs(120);
static GOOGLE_ATTEMPT_TELEMETRY_CALL_SEQUENCE: AtomicU64 = AtomicU64::new(1);

/// Google Gemini API / AI Studio adapter for Google-hosted text models.
///
/// This adapter is intentionally limited to untrusted candidate generation. It never
/// participates in harness verification or verdict authority.
pub struct GoogleAdapter {
    client: Client,
    api_key: String,
    base_url: Url,
    model: String,
    request_pacer: Option<Arc<RequestPacer>>,
    attempt_telemetry_path: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
struct GoogleAttemptTelemetryEvent<'a> {
    schema_version: &'static str,
    event: &'a str,
    call_id: &'a str,
    model: &'a str,
    attempt: u32,
    elapsed_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_code: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_class: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_delay_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quota_window: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_status: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rate_limit_headers: Option<&'a str>,
}

struct GoogleAttemptTelemetryGuard<'a> {
    path: Option<&'a Path>,
    call_id: &'a str,
    model: &'a str,
    attempt: u32,
    started: Instant,
    completed: bool,
}

impl<'a> GoogleAttemptTelemetryGuard<'a> {
    fn start(path: Option<&'a Path>, call_id: &'a str, model: &'a str, attempt: u32) -> Self {
        let guard = Self {
            path,
            call_id,
            model,
            attempt,
            started: Instant::now(),
            completed: false,
        };
        log_google_attempt_telemetry(
            path,
            GoogleAttemptTelemetryEvent {
                schema_version: "reason-google-attempt-telemetry-v1",
                event: "attempt_start",
                call_id,
                model,
                attempt,
                elapsed_ms: 0,
                status_code: None,
                error_class: None,
                retry_delay_ms: None,
                quota_window: None,
                provider_status: None,
                rate_limit_headers: None,
            },
        );
        guard
    }

    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    fn complete(&mut self) {
        self.completed = true;
    }
}

impl Drop for GoogleAttemptTelemetryGuard<'_> {
    fn drop(&mut self) {
        if self.completed {
            return;
        }
        log_google_attempt_telemetry(
            self.path,
            GoogleAttemptTelemetryEvent {
                schema_version: "reason-google-attempt-telemetry-v1",
                event: "cancelled_in_flight",
                call_id: self.call_id,
                model: self.model,
                attempt: self.attempt,
                elapsed_ms: self.elapsed().as_millis(),
                status_code: None,
                error_class: Some("cancelled_before_response"),
                retry_delay_ms: None,
                quota_window: None,
                provider_status: None,
                rate_limit_headers: None,
            },
        );
    }
}

struct RequestPacer {
    min_interval: Duration,
    next_start: tokio::sync::Mutex<tokio::time::Instant>,
    shared_path: Option<PathBuf>,
}

impl RequestPacer {
    fn new(min_interval: Duration, shared_path: Option<PathBuf>) -> Self {
        Self {
            min_interval,
            next_start: tokio::sync::Mutex::new(tokio::time::Instant::now()),
            shared_path,
        }
    }

    async fn wait(&self) -> Result<(), ModelError> {
        if let Some(path) = &self.shared_path {
            return wait_shared_request_slot(path, self.min_interval).await;
        }
        let mut next_start = self.next_start.lock().await;
        let now = tokio::time::Instant::now();
        if *next_start > now {
            tokio::time::sleep_until(*next_start).await;
        }
        *next_start = tokio::time::Instant::now() + self.min_interval;
        Ok(())
    }
}

async fn wait_shared_request_slot(path: &Path, min_interval: Duration) -> Result<(), ModelError> {
    let lock_dir = path.with_extension("lock");
    loop {
        match fs::create_dir(&lock_dir) {
            Ok(()) => break,
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Err(error) => {
                return Err(ModelError::new(
                    ModelErrorKind::Provider,
                    format!("failed to acquire shared Google request pacer: {error}"),
                ));
            }
        }
    }

    let result = (|| -> Result<Duration, ModelError> {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Provider,
                    format!("system clock before Unix epoch: {error}"),
                )
            })?
            .as_millis() as u64;
        let next_ms = match fs::read_to_string(path) {
            Ok(value) => value.trim().parse::<u64>().map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Provider,
                    format!("invalid shared Google pacer state: {error}"),
                )
            })?,
            Err(error) if error.kind() == ErrorKind::NotFound => now_ms,
            Err(error) => {
                return Err(ModelError::new(
                    ModelErrorKind::Provider,
                    format!("failed to read shared Google pacer state: {error}"),
                ));
            }
        };
        let wait_ms = next_ms.saturating_sub(now_ms);
        let base_ms = next_ms.max(now_ms);
        let updated_ms = base_ms.saturating_add(min_interval.as_millis() as u64);
        fs::write(path, format!("{updated_ms}\n")).map_err(|error| {
            ModelError::new(
                ModelErrorKind::Provider,
                format!("failed to write shared Google pacer state: {error}"),
            )
        })?;
        Ok(Duration::from_millis(wait_ms))
    })();
    let _ = fs::remove_dir(&lock_dir);
    let wait = result?;
    if !wait.is_zero() {
        tokio::time::sleep(wait).await;
    }
    Ok(())
}

impl GoogleAdapter {
    pub fn from_env(model: impl Into<String>) -> Result<Self, ModelError> {
        let api_key = env::var("GEMINI_API_KEY").map_err(|_| {
            ModelError::new(ModelErrorKind::Credentials, "GEMINI_API_KEY is not set")
        })?;
        Self::from_api_key_and_env(api_key, model)
    }

    pub fn from_api_key_and_env(
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, ModelError> {
        let mut adapter = Self::new(api_key, model)?;
        if let Some(interval) = configured_request_interval_from_env()? {
            let shared_path = configured_shared_pacer_path_from_env()?;
            adapter.request_pacer = Some(Arc::new(RequestPacer::new(interval, shared_path)));
        }
        adapter.attempt_telemetry_path = configured_attempt_telemetry_path_from_env()?;
        Ok(adapter)
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
        let client = crate::network::client_builder()
            .map_err(|error| {
                ModelError::new(
                    ModelErrorKind::Transport,
                    format!("failed to configure HTTP client: {}", error.message()),
                )
            })?
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
            request_pacer: None,
            attempt_telemetry_path: None,
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
                seed: request.random_seed.map(normalize_google_seed),
                temperature: GOOGLE_RECOMMENDED_TEMPERATURE,
            },
            store: false,
        };

        let call_sequence = GOOGLE_ATTEMPT_TELEMETRY_CALL_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let call_id = format!("g{}-{call_sequence}", std::process::id());
        let mut provider_attempts = 0u32;
        let mut rate_limit_retries = 0usize;
        let mut transient_retries = 0usize;
        let mut empty_text_retries = 0usize;

        loop {
            if let Some(pacer) = &self.request_pacer {
                pacer.wait().await?;
            }
            provider_attempts = provider_attempts.saturating_add(1);
            let mut attempt_telemetry = GoogleAttemptTelemetryGuard::start(
                self.attempt_telemetry_path.as_deref(),
                &call_id,
                &self.model,
                provider_attempts,
            );
            let response = self
                .client
                .post(endpoint.clone())
                .header("x-goog-api-key", &self.api_key)
                .json(&body)
                .send()
                .await;
            let response = match response {
                Ok(response) => response,
                Err(error) => {
                    let kind = if error.is_timeout() {
                        ModelErrorKind::Timeout
                    } else {
                        ModelErrorKind::Transport
                    };
                    attempt_telemetry.complete();
                    log_google_attempt_telemetry(
                        self.attempt_telemetry_path.as_deref(),
                        GoogleAttemptTelemetryEvent {
                            schema_version: "reason-google-attempt-telemetry-v1",
                            event: "transport_error",
                            call_id: &call_id,
                            model: &self.model,
                            attempt: provider_attempts,
                            elapsed_ms: attempt_telemetry.elapsed().as_millis(),
                            status_code: None,
                            error_class: Some(model_error_kind_name(kind)),
                            retry_delay_ms: None,
                            quota_window: None,
                            provider_status: None,
                            rate_limit_headers: None,
                        },
                    );
                    let detail = if error.is_timeout() {
                        "Gemini API request timed out".to_string()
                    } else {
                        format!("Gemini API request failed: {error}")
                    };
                    return Err(
                        ModelError::new(kind, detail).with_provider_attempts(provider_attempts)
                    );
                }
            };

            let status = response.status();
            let rate_limit_headers = google_rate_limit_header_summary(response.headers());
            log_google_attempt_telemetry(
                self.attempt_telemetry_path.as_deref(),
                GoogleAttemptTelemetryEvent {
                    schema_version: "reason-google-attempt-telemetry-v1",
                    event: "http_headers",
                    call_id: &call_id,
                    model: &self.model,
                    attempt: provider_attempts,
                    elapsed_ms: attempt_telemetry.elapsed().as_millis(),
                    status_code: Some(status.as_u16()),
                    error_class: None,
                    retry_delay_ms: None,
                    quota_window: None,
                    provider_status: None,
                    rate_limit_headers: (!rate_limit_headers.is_empty())
                        .then_some(rate_limit_headers.as_str()),
                },
            );
            if !status.is_success() {
                let rate_limit_delay = rate_limit_delay(response.headers(), rate_limit_retries);
                let body = response.text().await.unwrap_or_default();
                let kind = classify_http_error(status, &body);
                let quota_window = structured_google_quota_window(&body).map(quota_window_name);
                let provider_status = google_provider_status(&body);
                let detail = google_error_detail(&body);
                log_google_attempt_telemetry(
                    self.attempt_telemetry_path.as_deref(),
                    GoogleAttemptTelemetryEvent {
                        schema_version: "reason-google-attempt-telemetry-v1",
                        event: "http_response",
                        call_id: &call_id,
                        model: &self.model,
                        attempt: provider_attempts,
                        elapsed_ms: attempt_telemetry.elapsed().as_millis(),
                        status_code: Some(status.as_u16()),
                        error_class: Some(model_error_kind_name(kind)),
                        retry_delay_ms: None,
                        quota_window,
                        provider_status: provider_status.as_deref(),
                        rate_limit_headers: (!rate_limit_headers.is_empty())
                            .then_some(rate_limit_headers.as_str()),
                    },
                );
                attempt_telemetry.complete();

                if status == StatusCode::TOO_MANY_REQUESTS
                    && kind == ModelErrorKind::RateLimit
                    && rate_limit_retries < MAX_RATE_LIMIT_RETRIES
                    && provider_attempts < MAX_PROVIDER_ATTEMPTS
                {
                    rate_limit_retries += 1;
                    log_retry_scheduled_telemetry(
                        &attempt_telemetry,
                        kind,
                        rate_limit_delay,
                        quota_window,
                    );
                    tokio::time::sleep(rate_limit_delay).await;
                    continue;
                }

                if is_transient_http_status(status)
                    && transient_retries < MAX_TRANSIENT_RETRIES
                    && provider_attempts < MAX_PROVIDER_ATTEMPTS
                {
                    let delay = transient_retry_delay(transient_retries);
                    transient_retries += 1;
                    log_retry_scheduled_telemetry(&attempt_telemetry, kind, delay, quota_window);
                    tokio::time::sleep(delay).await;
                    continue;
                }

                return Err(ModelError::new(
                    kind,
                    format!(
                        "Gemini API returned HTTP {status} after {provider_attempts} provider attempts{detail}"
                    ),
                )
                .with_provider_attempts(provider_attempts));
            }

            log_google_attempt_telemetry(
                self.attempt_telemetry_path.as_deref(),
                GoogleAttemptTelemetryEvent {
                    schema_version: "reason-google-attempt-telemetry-v1",
                    event: "http_response",
                    call_id: &call_id,
                    model: &self.model,
                    attempt: provider_attempts,
                    elapsed_ms: attempt_telemetry.elapsed().as_millis(),
                    status_code: Some(status.as_u16()),
                    error_class: None,
                    retry_delay_ms: None,
                    quota_window: None,
                    provider_status: None,
                    rate_limit_headers: (!rate_limit_headers.is_empty())
                        .then_some(rate_limit_headers.as_str()),
                },
            );
            attempt_telemetry.complete();

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
    seed: Option<u32>,
    temperature: f32,
}

fn normalize_google_seed(seed: u64) -> u32 {
    const GOOGLE_SEED_DOMAIN: u64 = i32::MAX as u64 + 1;
    (seed % GOOGLE_SEED_DOMAIN) as u32
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
    mime_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema: Option<Value>,
}

fn configured_request_interval_from_env() -> Result<Option<Duration>, ModelError> {
    let Ok(raw) = env::var(GOOGLE_MIN_REQUEST_INTERVAL_MS_ENV) else {
        return Ok(None);
    };
    parse_request_interval_ms(&raw)
}

fn configured_shared_pacer_path_from_env() -> Result<Option<PathBuf>, ModelError> {
    let Ok(raw) = env::var(GOOGLE_SHARED_PACER_PATH_ENV) else {
        return Ok(None);
    };
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err(ModelError::new(
            ModelErrorKind::Protocol,
            format!("{GOOGLE_SHARED_PACER_PATH_ENV} must be an absolute path"),
        ));
    }
    Ok(Some(path))
}

fn configured_attempt_telemetry_path_from_env() -> Result<Option<PathBuf>, ModelError> {
    let Ok(raw) = env::var(GOOGLE_ATTEMPT_TELEMETRY_PATH_ENV) else {
        return Ok(None);
    };
    if raw.trim().is_empty() {
        return Ok(None);
    }
    let path = PathBuf::from(raw);
    if !path.is_absolute() {
        return Err(ModelError::new(
            ModelErrorKind::Protocol,
            format!("{GOOGLE_ATTEMPT_TELEMETRY_PATH_ENV} must be an absolute path"),
        ));
    }
    Ok(Some(path))
}

fn log_google_attempt_telemetry(path: Option<&Path>, event: GoogleAttemptTelemetryEvent<'_>) {
    let Some(path) = path else {
        return;
    };
    let Ok(line) = serde_json::to_string(&event) else {
        eprintln!("[google-attempt-telemetry-error] failed to serialize event");
        return;
    };
    match OpenOptions::new().create(true).append(true).open(path) {
        Ok(mut file) => {
            if let Err(error) = writeln!(file, "{line}") {
                eprintln!("[google-attempt-telemetry-error] failed to append event: {error}");
            }
        }
        Err(error) => {
            eprintln!("[google-attempt-telemetry-error] failed to open telemetry path: {error}");
        }
    }
}

fn parse_request_interval_ms(raw: &str) -> Result<Option<Duration>, ModelError> {
    let millis = raw.trim().parse::<u64>().map_err(|_| {
        ModelError::new(
            ModelErrorKind::Protocol,
            format!(
                "{GOOGLE_MIN_REQUEST_INTERVAL_MS_ENV} must be an integer number of milliseconds"
            ),
        )
    })?;
    if millis == 0 {
        return Ok(None);
    }
    let interval = Duration::from_millis(millis);
    if interval > MAX_CONFIGURED_REQUEST_INTERVAL {
        return Err(ModelError::new(
            ModelErrorKind::Protocol,
            format!(
                "{GOOGLE_MIN_REQUEST_INTERVAL_MS_ENV} must not exceed {} ms",
                MAX_CONFIGURED_REQUEST_INTERVAL.as_millis()
            ),
        ));
    }
    Ok(Some(interval))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GoogleQuotaWindow {
    ShortWindow,
    Daily,
    Ambiguous,
}

fn parse_google_duration(value: &str) -> Option<Duration> {
    let seconds = value.strip_suffix('s')?.parse::<f64>().ok()?;
    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }
    Some(Duration::from_secs_f64(seconds))
}

fn structured_google_quota_window(body: &str) -> Option<GoogleQuotaWindow> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    let error = value.get("error")?;
    let details = error.get("details").and_then(Value::as_array)?;

    let mut saw_quota_violation = false;
    let mut saw_short_window = false;
    let mut saw_daily = false;
    let mut saw_short_retry_info = false;

    for detail in details {
        let item_type = detail
            .get("@type")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if item_type.ends_with("google.rpc.RetryInfo") {
            saw_short_retry_info = detail
                .get("retryDelay")
                .or_else(|| detail.get("retry_delay"))
                .and_then(Value::as_str)
                .and_then(parse_google_duration)
                .is_some_and(|delay| {
                    delay > Duration::ZERO && delay <= MAX_STRUCTURED_SHORT_RETRY_DELAY
                });
        }

        let Some(violations) = detail.get("violations").and_then(Value::as_array) else {
            continue;
        };
        for violation in violations {
            saw_quota_violation = true;
            let quota_id = violation
                .get("quotaId")
                .or_else(|| violation.get("quota_id"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_lowercase();
            if quota_id.contains("perday") || quota_id.contains("daily") {
                saw_daily = true;
            } else if quota_id.contains("perminute")
                || quota_id.contains("persecond")
                || quota_id.contains("per10minute")
                || quota_id.contains("per10minutes")
            {
                saw_short_window = true;
            }
        }
    }

    // Daily evidence always wins. A bounded structured RetryInfo can identify a short
    // reset only when it is conflict-free; generic quota text remains fail-closed.
    if saw_daily {
        Some(GoogleQuotaWindow::Daily)
    } else if saw_short_window || saw_short_retry_info {
        Some(GoogleQuotaWindow::ShortWindow)
    } else if saw_quota_violation {
        Some(GoogleQuotaWindow::Ambiguous)
    } else {
        None
    }
}

fn quota_window_name(window: GoogleQuotaWindow) -> &'static str {
    match window {
        GoogleQuotaWindow::ShortWindow => "short_window",
        GoogleQuotaWindow::Daily => "daily",
        GoogleQuotaWindow::Ambiguous => "ambiguous",
    }
}

fn model_error_kind_name(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Credentials => "credentials",
        ModelErrorKind::Transport => "transport",
        ModelErrorKind::Provider => "provider",
        ModelErrorKind::RateLimit => "rate_limit",
        ModelErrorKind::Quota => "quota",
        ModelErrorKind::ProviderUnavailable => "provider_unavailable",
        ModelErrorKind::Timeout => "timeout",
        ModelErrorKind::Protocol => "protocol",
        ModelErrorKind::UnsupportedCapability => "unsupported_capability",
    }
}

fn google_rate_limit_header_summary(headers: &reqwest::header::HeaderMap) -> String {
    const SAFE_HEADERS: &[&str] = &[
        "retry-after",
        "ratelimit-limit",
        "ratelimit-remaining",
        "ratelimit-reset",
        "x-ratelimit-limit",
        "x-ratelimit-remaining",
        "x-ratelimit-reset",
        "x-ratelimit-limit-requests",
        "x-ratelimit-remaining-requests",
        "x-ratelimit-reset-requests",
        "x-ratelimit-limit-tokens",
        "x-ratelimit-remaining-tokens",
        "x-ratelimit-reset-tokens",
    ];

    SAFE_HEADERS
        .iter()
        .filter_map(|name| {
            headers
                .get(*name)
                .and_then(|value| value.to_str().ok())
                .map(|value| format!("{name}={}", truncate_diagnostic(value, 96)))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn log_retry_scheduled_telemetry(
    attempt: &GoogleAttemptTelemetryGuard<'_>,
    kind: ModelErrorKind,
    delay: Duration,
    quota_window: Option<&str>,
) {
    log_google_attempt_telemetry(
        attempt.path,
        GoogleAttemptTelemetryEvent {
            schema_version: "reason-google-attempt-telemetry-v1",
            event: "retry_scheduled",
            call_id: attempt.call_id,
            model: attempt.model,
            attempt: attempt.attempt,
            elapsed_ms: attempt.elapsed().as_millis(),
            status_code: None,
            error_class: Some(model_error_kind_name(kind)),
            retry_delay_ms: Some(delay.as_millis()),
            quota_window,
            provider_status: None,
            rate_limit_headers: None,
        },
    );
}

fn classify_http_error(status: StatusCode, body: &str) -> ModelErrorKind {
    if status == StatusCode::TOO_MANY_REQUESTS {
        match structured_google_quota_window(body) {
            Some(GoogleQuotaWindow::ShortWindow) => return ModelErrorKind::RateLimit,
            Some(GoogleQuotaWindow::Daily | GoogleQuotaWindow::Ambiguous) => {
                return ModelErrorKind::Quota;
            }
            None => {}
        }

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

fn transient_retry_base_delay(retry_index: usize) -> Duration {
    TRANSIENT_BACKOFF_SCHEDULE
        .get(retry_index)
        .copied()
        .unwrap_or_else(|| {
            *TRANSIENT_BACKOFF_SCHEDULE
                .last()
                .expect("non-empty schedule")
        })
}

fn retry_delay_with_equal_jitter(base: Duration) -> Duration {
    retry_delay_with_equal_jitter_entropy(base, rand::random::<u64>())
}

fn retry_delay_with_equal_jitter_entropy(base: Duration, entropy: u64) -> Duration {
    let base_ms = u64::try_from(base.as_millis()).unwrap_or(u64::MAX);
    if base_ms <= 1 {
        return base;
    }
    let floor_ms = base_ms / 2;
    let jitter_span_ms = base_ms.saturating_sub(floor_ms);
    let jitter_ms = entropy % jitter_span_ms.saturating_add(1);
    Duration::from_millis(floor_ms.saturating_add(jitter_ms))
}

fn transient_retry_delay(retry_index: usize) -> Duration {
    retry_delay_with_equal_jitter(transient_retry_base_delay(retry_index))
}

fn google_provider_status(body: &str) -> Option<String> {
    serde_json::from_str::<Value>(body)
        .ok()?
        .get("error")?
        .get("status")?
        .as_str()
        .map(|value| truncate_diagnostic(value, 64))
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
    let mut detail = match (provider_status, message) {
        (Some(status), Some(message)) => format!(
            "; provider_status={status}; message={}",
            truncate_diagnostic(message, 512)
        ),
        (Some(status), None) => format!("; provider_status={status}"),
        (None, Some(message)) => format!("; message={}", truncate_diagnostic(message, 512)),
        (None, None) => String::new(),
    };

    let mut quota_ids = Vec::new();
    let mut retry_delay = None;
    if let Some(details) = error.get("details").and_then(Value::as_array) {
        for item in details {
            if let Some(violations) = item.get("violations").and_then(Value::as_array) {
                for violation in violations {
                    if let Some(quota_id) = violation
                        .get("quotaId")
                        .or_else(|| violation.get("quota_id"))
                        .and_then(Value::as_str)
                    {
                        let quota_id = truncate_diagnostic(quota_id, 128);
                        if !quota_ids.contains(&quota_id) && quota_ids.len() < 4 {
                            quota_ids.push(quota_id);
                        }
                    }
                }
            }
            let item_type = item
                .get("@type")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if item_type.ends_with("google.rpc.RetryInfo") {
                retry_delay = item
                    .get("retryDelay")
                    .or_else(|| item.get("retry_delay"))
                    .and_then(Value::as_str)
                    .map(|value| truncate_diagnostic(value, 32));
            }
        }
    }
    if !quota_ids.is_empty() {
        detail.push_str("; quota_ids=");
        detail.push_str(&quota_ids.join(","));
    }
    if let Some(retry_delay) = retry_delay {
        detail.push_str("; retry_delay=");
        detail.push_str(&retry_delay);
    }
    detail
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
    let base = INITIAL_RATE_LIMIT_BACKOFF
        .checked_mul(multiplier)
        .unwrap_or(Duration::MAX);
    retry_delay_with_equal_jitter(base)
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
    fn normalizes_random_seed_to_google_supported_signed_32_bit_domain() {
        assert_eq!(normalize_google_seed(0), 0);
        assert_eq!(normalize_google_seed(i32::MAX as u64), i32::MAX as u32);
        assert_eq!(normalize_google_seed(i32::MAX as u64 + 1), 0);
        assert!(normalize_google_seed(0xa93c_2b41) <= i32::MAX as u32);
        assert!(normalize_google_seed(u64::MAX) <= i32::MAX as u32);
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
    fn parses_opt_in_google_request_pacing_without_changing_default() {
        assert_eq!(parse_request_interval_ms("0").unwrap(), None);
        assert_eq!(
            parse_request_interval_ms("4500").unwrap(),
            Some(Duration::from_millis(4500))
        );
        assert!(parse_request_interval_ms("not-a-number").is_err());
        assert!(parse_request_interval_ms("60001").is_err());
    }

    #[test]
    fn shared_google_pacer_path_requires_absolute_path() {
        let key = GOOGLE_SHARED_PACER_PATH_ENV;
        unsafe { std::env::set_var(key, "relative/pacer") };
        let error = configured_shared_pacer_path_from_env().unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::Protocol);
        unsafe { std::env::remove_var(key) };
    }

    #[tokio::test]
    async fn shared_google_pacer_serializes_request_slots_across_instances() {
        let dir = std::env::temp_dir().join(format!(
            "reason-google-pacer-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("shared.state");
        let interval = Duration::from_millis(80);
        let a = RequestPacer::new(interval, Some(path.clone()));
        let b = RequestPacer::new(interval, Some(path.clone()));
        let start = tokio::time::Instant::now();
        let (ra, rb) = tokio::join!(a.wait(), b.wait());
        ra.unwrap();
        rb.unwrap();
        assert!(start.elapsed() >= Duration::from_millis(70));
        let _ = fs::remove_file(path);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn retryable_google_transients_use_full_provider_attempt_budget() {
        let max_retries = MAX_PROVIDER_ATTEMPTS as usize - 1;
        assert_eq!(MAX_TRANSIENT_RETRIES, max_retries);
        assert_eq!(MAX_EMPTY_TEXT_RETRIES, max_retries);
    }

    #[test]
    fn google_request_timeout_allows_slow_provider_responses() {
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(300));
    }

    #[test]
    fn transient_5xx_backoff_keeps_bounded_base_schedule() {
        assert_eq!(transient_retry_base_delay(0), Duration::from_secs(2));
        assert_eq!(transient_retry_base_delay(1), Duration::from_secs(5));
        assert_eq!(transient_retry_base_delay(2), Duration::from_secs(10));
        assert_eq!(transient_retry_base_delay(3), Duration::from_secs(10));
    }

    #[test]
    fn retry_delay_uses_bounded_equal_jitter() {
        let base = Duration::from_secs(10);
        assert_eq!(
            retry_delay_with_equal_jitter_entropy(base, 0),
            Duration::from_secs(5)
        );
        assert_eq!(
            retry_delay_with_equal_jitter_entropy(base, 5_000),
            Duration::from_secs(10)
        );
        let sampled = retry_delay_with_equal_jitter(base);
        assert!(sampled >= Duration::from_secs(5));
        assert!(sampled <= Duration::from_secs(10));
    }

    #[test]
    fn rate_limit_delay_prefers_retry_after_header() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(RETRY_AFTER, reqwest::header::HeaderValue::from_static("7"));
        assert_eq!(rate_limit_delay(&headers, 2), Duration::from_secs(7));
    }

    #[test]
    fn rate_limit_delay_uses_bounded_jittered_exponential_fallback() {
        let headers = reqwest::header::HeaderMap::new();
        for (retry_index, base_secs) in [(0, 10), (1, 20), (2, 40)] {
            let delay = rate_limit_delay(&headers, retry_index);
            assert!(delay >= Duration::from_secs(base_secs / 2));
            assert!(delay <= Duration::from_secs(base_secs));
        }
    }

    #[test]
    fn extracts_bounded_google_provider_error_detail() {
        let detail = google_error_detail(
            r#"{"error":{"code":400,"status":"INVALID_ARGUMENT","message":"bad request\nwithout secrets"}}"#,
        );
        assert!(detail.contains("provider_status=INVALID_ARGUMENT"));
        assert!(detail.contains("message=bad request without secrets"));
        assert!(!detail.contains('\n'));

        let quota_detail = google_error_detail(
            r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"quota exceeded","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerMinutePerProjectPerModel-FreeTier"}]},{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"10s"}]}}"#,
        );
        assert!(
            quota_detail.contains("quota_ids=GenerateRequestsPerMinutePerProjectPerModel-FreeTier")
        );
        assert!(quota_detail.contains("retry_delay=10s"));
    }

    #[test]
    fn structured_google_quota_window_distinguishes_short_daily_and_ambiguous() {
        let per_minute = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerMinutePerProjectPerModel-FreeTier","quotaMetric":"generativelanguage.googleapis.com/generate_content_free_tier_requests"}]}]}}"#;
        let per_day = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerDayPerProjectPerModel-FreeTier"}]}]}}"#;
        let mixed = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerMinutePerProjectPerModel-FreeTier"},{"quotaId":"GenerateRequestsPerDayPerProjectPerModel-FreeTier"}]}]}}"#;
        let ambiguous = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaMetric":"generativelanguage.googleapis.com/generate_content_free_tier_requests"}]}]}}"#;

        assert_eq!(
            structured_google_quota_window(per_minute),
            Some(GoogleQuotaWindow::ShortWindow)
        );
        assert_eq!(
            structured_google_quota_window(per_day),
            Some(GoogleQuotaWindow::Daily)
        );
        assert_eq!(
            structured_google_quota_window(mixed),
            Some(GoogleQuotaWindow::Daily)
        );
        assert_eq!(
            structured_google_quota_window(ambiguous),
            Some(GoogleQuotaWindow::Ambiguous)
        );
    }

    #[test]
    fn structured_retry_info_without_quota_id_is_short_window_only_when_bounded_and_conflict_free()
    {
        let retry_only = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"51.410081504s"}]}}"#;
        let ambiguous_with_retry = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaMetric":"generativelanguage.googleapis.com/generate_content_free_tier_requests"}]},{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"39.4s"}]}}"#;
        let daily_with_retry = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerDayPerProjectPerModel-FreeTier"}]},{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"51s"}]}}"#;
        let long_retry = r#"{"error":{"details":[{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"3600s"}]}}"#;

        assert_eq!(
            structured_google_quota_window(retry_only),
            Some(GoogleQuotaWindow::ShortWindow)
        );
        assert_eq!(
            structured_google_quota_window(ambiguous_with_retry),
            Some(GoogleQuotaWindow::ShortWindow)
        );
        assert_eq!(
            structured_google_quota_window(daily_with_retry),
            Some(GoogleQuotaWindow::Daily)
        );
        assert_eq!(structured_google_quota_window(long_retry), None);
    }

    #[test]
    fn classifies_quota_id_absent_structured_retry_info_as_rate_limit() {
        let body = r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"You exceeded your current quota","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaMetric":"generativelanguage.googleapis.com/generate_content_free_tier_requests"}]},{"@type":"type.googleapis.com/google.rpc.RetryInfo","retryDelay":"49.751743601s"}]}}"#;
        assert_eq!(
            classify_http_error(StatusCode::TOO_MANY_REQUESTS, body),
            ModelErrorKind::RateLimit
        );
    }

    #[test]
    fn classifies_structured_short_window_google_quota_as_rate_limit_only() {
        let per_minute = r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"quota exceeded","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerMinutePerProjectPerModel-FreeTier","quotaMetric":"generativelanguage.googleapis.com/generate_content_free_tier_requests"}]}]}}"#;
        let per_day = r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"quota exceeded","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerDayPerProjectPerModel-FreeTier"}]}]}}"#;

        assert_eq!(
            classify_http_error(StatusCode::TOO_MANY_REQUESTS, per_minute),
            ModelErrorKind::RateLimit
        );
        assert_eq!(
            classify_http_error(StatusCode::TOO_MANY_REQUESTS, per_day),
            ModelErrorKind::Quota
        );
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
    async fn transient_5xx_can_recover_on_final_allowed_attempt() {
        let busy = r#"{\"error\":{\"message\":\"high demand\"}}"#.to_string();
        let (base_url, server) = spawn_sequence_server(vec![
            ("503 Service Unavailable", busy.clone(), ""),
            ("503 Service Unavailable", busy.clone(), ""),
            ("503 Service Unavailable", busy, ""),
            ("200 OK", success_body("ok"), ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let response = adapter.generate(test_request()).await.unwrap();
        assert_eq!(response.text, "ok");
        assert_eq!(response.provider_attempts, 4);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn repeated_transient_5xx_fails_after_bounded_retries() {
        let busy = r#"{"error":{"message":"high demand"}}"#.to_string();
        let (base_url, server) = spawn_sequence_server(vec![
            ("503 Service Unavailable", busy.clone(), ""),
            ("503 Service Unavailable", busy.clone(), ""),
            ("503 Service Unavailable", busy.clone(), ""),
            ("503 Service Unavailable", busy, ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let error = adapter.generate(test_request()).await.unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::ProviderUnavailable);
        assert_eq!(error.provider_attempts, 4);
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
    async fn empty_model_text_can_recover_on_final_allowed_attempt() {
        let (base_url, server) = spawn_sequence_server(vec![
            ("200 OK", empty_text_body(), ""),
            ("200 OK", empty_text_body(), ""),
            ("200 OK", empty_text_body(), ""),
            ("200 OK", success_body("ok"), ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let response = adapter.generate(test_request()).await.unwrap();
        assert_eq!(response.text, "ok");
        assert_eq!(response.provider_attempts, 4);
        server.join().unwrap();
    }

    #[tokio::test]
    async fn repeated_empty_model_text_remains_protocol_failure() {
        let (base_url, server) = spawn_sequence_server(vec![
            ("200 OK", empty_text_body(), ""),
            ("200 OK", empty_text_body(), ""),
            ("200 OK", empty_text_body(), ""),
            ("200 OK", empty_text_body(), ""),
        ]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let error = adapter.generate(test_request()).await.unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::Protocol);
        assert_eq!(error.provider_attempts, 4);
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
    async fn structured_per_minute_quota_uses_existing_bounded_rate_limit_retry() {
        let short_window = r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"quota exceeded","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerMinutePerProjectPerModel-FreeTier"}]}]}}"#;
        let (base_url, server) = spawn_sequence_server(vec![
            (
                "429 Too Many Requests",
                short_window.into(),
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

    #[tokio::test]
    async fn structured_per_day_quota_remains_non_retryable() {
        let daily = r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"quota exceeded","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerDayPerProjectPerModel-FreeTier"}]}]}}"#;
        let (base_url, server) =
            spawn_sequence_server(vec![("429 Too Many Requests", daily.into(), "")]);
        let adapter = GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        let error = adapter.generate(test_request()).await.unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::Quota);
        assert_eq!(error.provider_attempts, 1);
        server.join().unwrap();
    }

    fn temp_telemetry_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "reason-google-telemetry-{label}-{}-{}.jsonl",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn read_telemetry(path: &Path) -> Vec<Value> {
        fs::read_to_string(path)
            .expect("read telemetry")
            .lines()
            .map(|line| serde_json::from_str(line).expect("parse telemetry line"))
            .collect()
    }

    #[tokio::test]
    async fn attempt_telemetry_records_structured_quota_without_request_content() {
        let daily = r#"{"error":{"status":"RESOURCE_EXHAUSTED","message":"quota exceeded","details":[{"@type":"type.googleapis.com/google.rpc.QuotaFailure","violations":[{"quotaId":"GenerateRequestsPerDayPerProjectPerModel-FreeTier"}]}]}}"#;
        let (base_url, server) =
            spawn_sequence_server(vec![("429 Too Many Requests", daily.into(), "")]);
        let path = temp_telemetry_path("quota");
        let mut adapter =
            GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        adapter.attempt_telemetry_path = Some(path.clone());
        let error = adapter.generate(test_request()).await.unwrap_err();
        assert_eq!(error.kind, ModelErrorKind::Quota);
        server.join().unwrap();

        let events = read_telemetry(&path);
        assert_eq!(events[0]["event"], "attempt_start");
        let response = events
            .iter()
            .find(|event| event["event"] == "http_response")
            .expect("http response event");
        assert_eq!(response["status_code"], 429);
        assert_eq!(response["error_class"], "quota");
        assert_eq!(response["quota_window"], "daily");
        assert_eq!(response["provider_status"], "RESOURCE_EXHAUSTED");
        let raw = fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("test-key"));
        assert!(!raw.contains("Return exactly one"));
        assert!(!raw.contains("quota exceeded"));
        let _ = fs::remove_file(path);
    }

    #[tokio::test]
    async fn attempt_telemetry_records_503_retry_then_success() {
        let busy = r#"{"error":{"status":"UNAVAILABLE","message":"high demand"}}"#;
        let (base_url, server) = spawn_sequence_server(vec![
            ("503 Service Unavailable", busy.into(), ""),
            ("200 OK", success_body("ok"), ""),
        ]);
        let path = temp_telemetry_path("503");
        let mut adapter =
            GoogleAdapter::with_base_url("test-key", "test-model", &base_url).unwrap();
        adapter.attempt_telemetry_path = Some(path.clone());
        let response = adapter.generate(test_request()).await.unwrap();
        assert_eq!(response.provider_attempts, 2);
        server.join().unwrap();

        let events = read_telemetry(&path);
        assert!(events.iter().any(|event| {
            event["event"] == "http_response"
                && event["status_code"] == 503
                && event["error_class"] == "provider_unavailable"
        }));
        assert!(events.iter().any(|event| {
            event["event"] == "retry_scheduled" && event["error_class"] == "provider_unavailable"
        }));
        assert!(
            events
                .iter()
                .any(|event| { event["event"] == "http_response" && event["status_code"] == 200 })
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn attempt_telemetry_guard_records_in_flight_cancellation() {
        let path = temp_telemetry_path("cancel");
        {
            let _guard =
                GoogleAttemptTelemetryGuard::start(Some(&path), "test-call", "test-model", 1);
        }
        let events = read_telemetry(&path);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["event"], "attempt_start");
        assert_eq!(events[1]["event"], "cancelled_in_flight");
        assert_eq!(events[1]["error_class"], "cancelled_before_response");
        let _ = fs::remove_file(path);
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
