use std::{env, error::Error as _, fs, path::PathBuf, time::Duration};

use reqwest::{Certificate, Client, ClientBuilder, redirect};

pub const CUSTOM_CA_BUNDLE_ENV: &str = "REASON_CA_BUNDLE";
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
