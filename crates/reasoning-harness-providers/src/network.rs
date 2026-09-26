use std::{
    env,
    error::Error as _,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use reasoning_harness_core::{
    ModelError, ModelErrorKind, ModelExecutionBudget, ModelExecutionTelemetrySnapshot,
};
use reqwest::{Certificate, Client, ClientBuilder, redirect};

pub const CUSTOM_CA_BUNDLE_ENV: &str = "REASON_CA_BUNDLE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderWaitKind {
    Pacing,
    Retry,
}

#[derive(Debug, Default)]
pub struct ExecutionTelemetryCounters {
    provider_attempts_started: AtomicU64,
    provider_attempts_completed: AtomicU64,
    active_ms: AtomicU64,
    wait_ms: AtomicU64,
    pacing_wait_ms: AtomicU64,
    retry_wait_ms: AtomicU64,
}

impl ExecutionTelemetryCounters {
    pub fn snapshot(&self) -> ModelExecutionTelemetrySnapshot {
        ModelExecutionTelemetrySnapshot {
            provider_attempts_started: self.provider_attempts_started.load(Ordering::Relaxed),
            provider_attempts_completed: self.provider_attempts_completed.load(Ordering::Relaxed),
            active_ms: self.active_ms.load(Ordering::Relaxed),
            wait_ms: self.wait_ms.load(Ordering::Relaxed),
            pacing_wait_ms: self.pacing_wait_ms.load(Ordering::Relaxed),
            retry_wait_ms: self.retry_wait_ms.load(Ordering::Relaxed),
        }
    }

    pub fn attempt_started(&self) {
        self.provider_attempts_started
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn active_elapsed(&self, active: Duration) {
        self.active_ms
            .fetch_add(duration_ms_u64(active), Ordering::Relaxed);
    }

    pub fn attempt_completed(&self, active: Duration) {
        self.provider_attempts_completed
            .fetch_add(1, Ordering::Relaxed);
        self.active_elapsed(active);
    }

    pub fn wait_completed(&self, kind: ProviderWaitKind, waited: Duration) {
        let millis = duration_ms_u64(waited);
        self.wait_ms.fetch_add(millis, Ordering::Relaxed);
        match kind {
            ProviderWaitKind::Pacing => {
                self.pacing_wait_ms.fetch_add(millis, Ordering::Relaxed);
            }
            ProviderWaitKind::Retry => {
                self.retry_wait_ms.fetch_add(millis, Ordering::Relaxed);
            }
        }
    }
}

pub fn bounded_wait(
    requested: Duration,
    used_wait: Duration,
    budget: Option<ModelExecutionBudget>,
) -> Result<Duration, ModelError> {
    let Some(budget) = budget else {
        return Ok(requested);
    };

    let requested_ms = duration_ms_u64(requested);
    if requested_ms > budget.max_single_wait_ms {
        return Err(ModelError::new(
            ModelErrorKind::RateLimit,
            format!(
                "provider wait exceeds configured single-wait budget: requested_ms={requested_ms} max_single_wait_ms={}",
                budget.max_single_wait_ms
            ),
        ));
    }

    let used_ms = duration_ms_u64(used_wait);
    if used_ms.saturating_add(requested_ms) > budget.max_wait_ms {
        return Err(ModelError::new(
            ModelErrorKind::RateLimit,
            format!(
                "provider cumulative wait budget exhausted: used_ms={used_ms} requested_ms={requested_ms} max_wait_ms={}",
                budget.max_wait_ms
            ),
        ));
    }

    Ok(requested)
}

pub fn remaining_active_budget(
    used_active: Duration,
    budget: Option<ModelExecutionBudget>,
) -> Result<Option<Duration>, ModelError> {
    let Some(budget) = budget else {
        return Ok(None);
    };

    let used_ms = duration_ms_u64(used_active);
    if used_ms >= budget.max_active_ms {
        return Err(ModelError::new(
            ModelErrorKind::Timeout,
            format!(
                "provider active execution budget exhausted: used_ms={used_ms} max_active_ms={}",
                budget.max_active_ms
            ),
        ));
    }

    Ok(Some(Duration::from_millis(
        budget.max_active_ms.saturating_sub(used_ms),
    )))
}

pub fn duration_ms_u64(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

const MAX_CUSTOM_CA_BUNDLE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkFailureKind {
    Timeout,
    Dns,
    Proxy,
    TlsCertificate,
    Connectivity,
    Transport,
    CustomCa,
}

impl NetworkFailureKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::Dns => "dns",
            Self::Proxy => "proxy",
            Self::TlsCertificate => "tls_certificate",
            Self::Connectivity => "connectivity",
            Self::Transport => "transport",
            Self::CustomCa => "custom_ca",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkConfigError {
    message: String,
}

impl NetworkConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NetworkEnvironmentStatus {
    pub http_proxy: bool,
    pub https_proxy: bool,
    pub all_proxy: bool,
    pub no_proxy: bool,
    pub custom_ca_bundle: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct NetworkProbeFailure {
    pub kind: NetworkFailureKind,
}

pub fn environment_status() -> NetworkEnvironmentStatus {
    NetworkEnvironmentStatus {
        http_proxy: env_present("HTTP_PROXY", "http_proxy"),
        https_proxy: env_present("HTTPS_PROXY", "https_proxy"),
        all_proxy: env_present("ALL_PROXY", "all_proxy"),
        no_proxy: env_present("NO_PROXY", "no_proxy"),
        custom_ca_bundle: env::var_os(CUSTOM_CA_BUNDLE_ENV).is_some(),
    }
}

pub fn client_builder() -> Result<ClientBuilder, NetworkConfigError> {
    apply_custom_ca(Client::builder())
}

pub fn validate_custom_ca() -> Result<&'static str, NetworkConfigError> {
    if env::var_os(CUSTOM_CA_BUNDLE_ENV).is_none() {
        return Ok("not_configured");
    }
    let _builder = apply_custom_ca(Client::builder())?;
    Ok("ready")
}

pub async fn probe_https(url: &str, timeout: Duration) -> Result<u16, NetworkProbeFailure> {
    let client = client_builder()
        .map_err(|_| NetworkProbeFailure {
            kind: NetworkFailureKind::CustomCa,
        })?
        .timeout(timeout)
        .redirect(redirect::Policy::none())
        .build()
        .map_err(|_| NetworkProbeFailure {
            kind: NetworkFailureKind::Transport,
        })?;
    client
        .head(url)
        .send()
        .await
        .map(|response| response.status().as_u16())
        .map_err(|error| NetworkProbeFailure {
            kind: classify_reqwest_error(&error),
        })
}

pub fn classify_reqwest_error(error: &reqwest::Error) -> NetworkFailureKind {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(current) = source {
        text.push(' ');
        text.push_str(&current.to_string());
        source = current.source();
    }
    classify_network_error_text(error.is_timeout(), error.is_connect(), &text)
}

fn apply_custom_ca(builder: ClientBuilder) -> Result<ClientBuilder, NetworkConfigError> {
    let Some(raw_path) = env::var_os(CUSTOM_CA_BUNDLE_ENV) else {
        return Ok(builder);
    };
    let path = PathBuf::from(raw_path);
    if path.as_os_str().is_empty() {
        return Err(NetworkConfigError::new(
            "REASON_CA_BUNDLE is set but the path is empty",
        ));
    }
    let metadata = fs::metadata(&path).map_err(|_| {
        NetworkConfigError::new("REASON_CA_BUNDLE cannot be read as a regular file")
    })?;
    if !metadata.is_file() {
        return Err(NetworkConfigError::new(
            "REASON_CA_BUNDLE must point to a regular PEM file",
        ));
    }
    if metadata.len() == 0 || metadata.len() > MAX_CUSTOM_CA_BUNDLE_BYTES {
        return Err(NetworkConfigError::new(
            "REASON_CA_BUNDLE must be a non-empty PEM bundle no larger than 1 MiB",
        ));
    }
    let bytes = fs::read(&path)
        .map_err(|_| NetworkConfigError::new("REASON_CA_BUNDLE could not be read"))?;
    let certificates = Certificate::from_pem_bundle(&bytes).map_err(|_| {
        NetworkConfigError::new("REASON_CA_BUNDLE is not a valid PEM certificate bundle")
    })?;
    if certificates.is_empty() {
        return Err(NetworkConfigError::new(
            "REASON_CA_BUNDLE contains no certificates",
        ));
    }
    Ok(certificates
        .into_iter()
        .fold(builder, |builder, certificate| {
            builder.add_root_certificate(certificate)
        }))
}

fn env_present(upper: &str, lower: &str) -> bool {
    env::var_os(upper).is_some() || env::var_os(lower).is_some()
}

fn classify_network_error_text(
    is_timeout: bool,
    is_connect: bool,
    text: &str,
) -> NetworkFailureKind {
    if is_timeout {
        return NetworkFailureKind::Timeout;
    }
    let normalized = text.to_ascii_lowercase();
    if [
        "certificate",
        "unknown issuer",
        "invalid peer certificate",
        "webpki",
        "rustls",
        "tls handshake",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
    {
        return NetworkFailureKind::TlsCertificate;
    }
    if [
        "failed to lookup address",
        "name or service not known",
        "temporary failure in name resolution",
        "nodename nor servname",
        "no such host",
        "dns error",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
    {
        return NetworkFailureKind::Dns;
    }
    if [
        "proxy",
        "proxy connect",
        "tunnel",
        "407 proxy authentication",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
    {
        return NetworkFailureKind::Proxy;
    }
    if is_connect {
        NetworkFailureKind::Connectivity
    } else {
        NetworkFailureKind::Transport
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_wait_budget_caps_single_and_cumulative_waits() {
        let budget = ModelExecutionBudget {
            max_active_ms: 60_000,
            max_wait_ms: 45_000,
            max_single_wait_ms: 30_000,
        };
        assert_eq!(
            bounded_wait(Duration::from_secs(20), Duration::ZERO, Some(budget)).unwrap(),
            Duration::from_secs(20)
        );

        let single = bounded_wait(Duration::from_secs(31), Duration::ZERO, Some(budget))
            .expect_err("single wait must be capped");
        assert_eq!(single.kind, ModelErrorKind::RateLimit);

        let cumulative = bounded_wait(
            Duration::from_secs(26),
            Duration::from_secs(20),
            Some(budget),
        )
        .expect_err("cumulative wait must be capped");
        assert_eq!(cumulative.kind, ModelErrorKind::RateLimit);
    }

    #[test]
    fn provider_active_budget_is_separate_from_wait_budget() {
        let budget = ModelExecutionBudget {
            max_active_ms: 60_000,
            max_wait_ms: 45_000,
            max_single_wait_ms: 30_000,
        };
        assert_eq!(
            remaining_active_budget(Duration::from_secs(10), Some(budget)).unwrap(),
            Some(Duration::from_secs(50))
        );
        let exhausted = remaining_active_budget(Duration::from_secs(60), Some(budget))
            .expect_err("active budget must be finite");
        assert_eq!(exhausted.kind, ModelErrorKind::Timeout);
    }

    #[test]
    fn execution_telemetry_separates_attempts_active_and_wait() {
        let counters = ExecutionTelemetryCounters::default();
        counters.attempt_started();
        counters.attempt_completed(Duration::from_millis(7));
        counters.wait_completed(ProviderWaitKind::Pacing, Duration::from_millis(11));
        counters.wait_completed(ProviderWaitKind::Retry, Duration::from_millis(13));
        let snapshot = counters.snapshot();
        assert_eq!(snapshot.provider_attempts_started, 1);
        assert_eq!(snapshot.provider_attempts_completed, 1);
        assert_eq!(snapshot.active_ms, 7);
        assert_eq!(snapshot.wait_ms, 24);
        assert_eq!(snapshot.pacing_wait_ms, 11);
        assert_eq!(snapshot.retry_wait_ms, 13);
    }

    #[test]
    fn network_failure_classifier_separates_dns_proxy_tls_and_connectivity() {
        assert_eq!(
            classify_network_error_text(false, true, "dns error: failed to lookup address"),
            NetworkFailureKind::Dns
        );
        assert_eq!(
            classify_network_error_text(
                false,
                true,
                "proxy connect failed: 407 proxy authentication"
            ),
            NetworkFailureKind::Proxy
        );
        assert_eq!(
            classify_network_error_text(false, true, "invalid peer certificate: UnknownIssuer"),
            NetworkFailureKind::TlsCertificate
        );
        assert_eq!(
            classify_network_error_text(false, true, "connection refused"),
            NetworkFailureKind::Connectivity
        );
        assert_eq!(
            classify_network_error_text(true, true, "deadline elapsed"),
            NetworkFailureKind::Timeout
        );
    }
}
