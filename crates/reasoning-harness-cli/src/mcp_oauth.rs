use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    process::Command,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use keyring::{Entry, Error as KeyringError};
use rand::RngExt as _;
use reqwest::{Client, redirect};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use super::{CliError, McpRemoteOAuthFileConfig, OutputFormat, print_product_json};

const MCP_OAUTH_SERVICE: &str = "io.github.git-ksk.reason-cli.mcp-oauth.v1";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);
const CALLBACK_PATH: &str = "/oauth/callback";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredOAuthToken {
    schema_version: String,
    server_name: String,
    issuer: String,
    client_id: String,
    resource: String,
    token_endpoint: String,
    access_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    token_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expires_at_unix_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    expires_in: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct McpOAuthStatus {
    pub server_name: String,
    pub configured: bool,
    pub stored: bool,
    pub usable: bool,
    pub issuer: String,
    pub client_id: String,
    pub expires_at_unix_seconds: Option<u64>,
    pub refresh_available: bool,
}

#[derive(Debug, Serialize)]
struct LoginOutput {
    operation: &'static str,
    server_name: String,
    stored: bool,
    issuer: String,
    client_id: String,
    refresh_available: bool,
    expires_at_unix_seconds: Option<u64>,
}

#[derive(Debug, Serialize)]
struct LogoutOutput {
    operation: &'static str,
    server_name: String,
    removed: bool,
}

pub(crate) fn status(
    name: &str,
    resource: &str,
    oauth: &McpRemoteOAuthFileConfig,
) -> Result<McpOAuthStatus, CliError> {
    validate_oauth_config(oauth)?;
    let stored = load_token(name)?;
    let usable = stored.as_ref().is_some_and(|token| {
        token_matches_config(token, name, resource, oauth)
            && (token_is_fresh(token) || token.refresh_token.is_some())
    });
    Ok(McpOAuthStatus {
        server_name: name.into(),
        configured: true,
        stored: stored.is_some(),
        usable,
        issuer: oauth.issuer.clone(),
        client_id: oauth.client_id.clone(),
        expires_at_unix_seconds: stored
            .as_ref()
            .filter(|token| token_matches_config(token, name, resource, oauth))
            .and_then(|token| token.expires_at_unix_seconds),
        refresh_available: stored
            .as_ref()
            .filter(|token| token_matches_config(token, name, resource, oauth))
            .and_then(|token| token.refresh_token.as_ref())
            .is_some(),
    })
}

pub(crate) fn login(
    name: &str,
    endpoint: &str,
    oauth: &McpRemoteOAuthFileConfig,
    no_browser: bool,
    replace: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    validate_oauth_config(oauth)?;
    validate_resource_endpoint(endpoint)?;
    if load_token(name)?.is_some() && !replace {
        return Err(CliError::new(
            "mcp_oauth_exists",
            format!(
                "stored OAuth credentials already exist for {name:?}; use `reason mcp login {name} --replace` to reauthenticate"
            ),
        ));
    }

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| {
        CliError::new(
            "mcp_oauth_callback",
            format!("cannot bind loopback OAuth callback listener: {error}"),
        )
    })?;
    listener.set_nonblocking(true).map_err(|error| {
        CliError::new(
            "mcp_oauth_callback",
            format!("cannot configure OAuth callback listener: {error}"),
        )
    })?;
    let port = listener
        .local_addr()
        .map_err(|error| {
            CliError::new(
                "mcp_oauth_callback",
                format!("cannot inspect OAuth callback listener: {error}"),
            )
        })?
        .port();
    let redirect_uri = format!("http://127.0.0.1:{port}{CALLBACK_PATH}");
    let verifier = random_urlsafe(48);
    let state = random_urlsafe(32);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let authorization_url =
        build_authorization_url(oauth, endpoint, &redirect_uri, &state, &challenge)?;

    match format {
        OutputFormat::Human => {
            println!("Open this URL to authorize {name}:");
            println!("{authorization_url}");
        }
        OutputFormat::Json => {
            eprintln!("OAuth authorization URL: {authorization_url}");
        }
    }
    if !no_browser {
        let _ = open_browser(&authorization_url);
    }

    let callback = wait_for_callback(&listener, &state, &oauth.issuer)?;
    let token = exchange_code(name, endpoint, oauth, &redirect_uri, &verifier, &callback)?;
    save_token(name, &token)?;
    let output = LoginOutput {
        operation: "login",
        server_name: name.into(),
        stored: true,
        issuer: oauth.issuer.clone(),
        client_id: oauth.client_id.clone(),
        refresh_available: token.refresh_token.is_some(),
        expires_at_unix_seconds: token.expires_at_unix_seconds,
    };
    match format {
        OutputFormat::Json => print_product_json("mcp", &output).map_err(CliError::from),
        OutputFormat::Human => {
            println!("OAuth login complete for {name}; credential stored in the native OS store.");
            Ok(())
        }
    }
}

pub(crate) fn logout(name: &str, format: OutputFormat) -> Result<(), CliError> {
    let removed = delete_token(name)?;
    let output = LogoutOutput {
        operation: "logout",
        server_name: name.into(),
        removed,
    };
    match format {
        OutputFormat::Json => print_product_json("mcp", &output).map_err(CliError::from),
        OutputFormat::Human => {
            if removed {
                println!("Removed stored OAuth credential for {name}.");
            } else {
                println!("No stored OAuth credential existed for {name}.");
            }
            Ok(())
        }
    }
}

pub(crate) fn access_token(
    name: &str,
    endpoint: &str,
    oauth: &McpRemoteOAuthFileConfig,
) -> Result<String, CliError> {
    validate_oauth_config(oauth)?;
    let mut token = load_token(name)?.ok_or_else(|| {
        CliError::new(
            "mcp_authentication",
            format!("no OAuth credential is stored for {name:?}; run `reason mcp login {name}`"),
        )
    })?;
    if !token_matches_config(&token, name, endpoint, oauth) {
        return Err(CliError::new(
            "mcp_authentication",
            format!(
                "stored OAuth credential for {name:?} is bound to different issuer/client metadata; run `reason mcp login {name} --replace`"
            ),
        ));
    }
    if token_is_fresh(&token) {
        return Ok(token.access_token);
    }
    let Some(refresh_token) = token.refresh_token.clone() else {
        return Err(CliError::new(
            "mcp_authentication",
            format!(
                "OAuth credential for {name:?} expired and has no refresh token; run `reason mcp login {name} --replace`"
            ),
        ));
    };
    token = refresh(name, endpoint, oauth, &refresh_token)?;
    save_token(name, &token)?;
    Ok(token.access_token)
}

pub(crate) fn validate_oauth_config(oauth: &McpRemoteOAuthFileConfig) -> Result<(), CliError> {
    for (label, value) in [
        ("issuer", oauth.issuer.as_str()),
        (
            "authorization_endpoint",
            oauth.authorization_endpoint.as_str(),
        ),
        ("token_endpoint", oauth.token_endpoint.as_str()),
        ("client_id", oauth.client_id.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(CliError::new(
                "mcp_oauth_configuration",
                format!("remote MCP OAuth {label} must be non-empty"),
            ));
        }
    }
    let issuer = Url::parse(&oauth.issuer).map_err(|_| {
        CliError::new(
            "mcp_oauth_configuration",
            "OAuth issuer must be a valid URL",
        )
    })?;
    let authorization = Url::parse(&oauth.authorization_endpoint).map_err(|_| {
        CliError::new(
            "mcp_oauth_configuration",
            "OAuth authorization_endpoint must be a valid URL",
        )
    })?;
    let token = Url::parse(&oauth.token_endpoint).map_err(|_| {
        CliError::new(
            "mcp_oauth_configuration",
            "OAuth token_endpoint must be a valid URL",
        )
    })?;
    for (label, url) in [
        ("issuer", issuer),
        ("authorization_endpoint", authorization),
        ("token_endpoint", token),
    ] {
        if (url.scheme() != "https" && !loopback_http(&url))
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(CliError::new(
                "mcp_oauth_configuration",
                format!(
                    "OAuth {label} must use a credential-free HTTPS URL without a fragment (loopback HTTP is allowed only for local testing)"
                ),
            ));
        }
    }
    if oauth.scopes.iter().any(|scope| scope.trim().is_empty()) {
        return Err(CliError::new(
            "mcp_oauth_configuration",
            "OAuth scopes must not contain empty values",
        ));
    }
    Ok(())
}

fn validate_resource_endpoint(endpoint: &str) -> Result<(), CliError> {
    let url = Url::parse(endpoint).map_err(|_| {
        CliError::new(
            "mcp_configuration",
            "remote MCP endpoint must be a valid URL",
        )
    })?;
    if (url.scheme() != "https" && !loopback_http(&url))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(CliError::new(
            "mcp_configuration",
            "remote MCP endpoint must use a credential-free HTTPS URL without a fragment (loopback HTTP is allowed only for local testing)",
        ));
    }
    Ok(())
}

fn loopback_http(url: &Url) -> bool {
    url.scheme() == "http"
        && matches!(
            url.host_str(),
            Some("127.0.0.1" | "localhost" | "::1" | "[::1]")
        )
}

fn build_authorization_url(
    oauth: &McpRemoteOAuthFileConfig,
    resource: &str,
    redirect_uri: &str,
    state: &str,
    challenge: &str,
) -> Result<String, CliError> {
    let mut url = Url::parse(&oauth.authorization_endpoint).map_err(|_| {
        CliError::new(
            "mcp_oauth_configuration",
            "OAuth authorization_endpoint must be a valid URL",
        )
    })?;
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("response_type", "code");
        query.append_pair("client_id", &oauth.client_id);
        query.append_pair("redirect_uri", redirect_uri);
        query.append_pair("code_challenge", challenge);
        query.append_pair("code_challenge_method", "S256");
        query.append_pair("state", state);
        query.append_pair("resource", resource);
        if !oauth.scopes.is_empty() {
            query.append_pair("scope", &oauth.scopes.join(" "));
        }
    }
    Ok(url.into())
}

#[derive(Debug)]
struct CallbackResult {
    code: String,
}

fn wait_for_callback(
    listener: &TcpListener,
    expected_state: &str,
    expected_issuer: &str,
) -> Result<CallbackResult, CliError> {
    let started = Instant::now();
    while started.elapsed() < LOGIN_TIMEOUT {
        match listener.accept() {
            Ok((stream, _)) => return handle_callback(stream, expected_state, expected_issuer),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(error) => {
                return Err(CliError::new(
                    "mcp_oauth_callback",
                    format!("OAuth callback listener failed: {error}"),
                ));
            }
        }
    }
    Err(CliError::new(
        "mcp_oauth_timeout",
        "OAuth login timed out waiting for the loopback callback",
    ))
}

fn handle_callback(
    mut stream: TcpStream,
    expected_state: &str,
    expected_issuer: &str,
) -> Result<CallbackResult, CliError> {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    let mut reader = BufReader::new(stream.try_clone().map_err(|error| {
        CliError::new(
            "mcp_oauth_callback",
            format!("cannot read OAuth callback: {error}"),
        )
    })?);
    let mut request = String::new();
    reader.read_line(&mut request).map_err(|error| {
        CliError::new(
            "mcp_oauth_callback",
            format!("cannot read OAuth callback: {error}"),
        )
    })?;
    let target = request
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| CliError::new("mcp_oauth_callback", "invalid OAuth callback request"))?;
    let url = Url::parse(&format!("http://127.0.0.1{target}"))
        .map_err(|_| CliError::new("mcp_oauth_callback", "invalid OAuth callback URL"))?;
    if url.path() != CALLBACK_PATH {
        return Err(CliError::new(
            "mcp_oauth_callback",
            "unexpected OAuth callback path",
        ));
    }
    let query = url
        .query_pairs()
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();
    let unique = |key: &str| -> Result<Option<&str>, CliError> {
        let values = query
            .iter()
            .filter_map(|(candidate, value)| (candidate == key).then_some(value.as_str()))
            .collect::<Vec<_>>();
        if values.len() > 1 {
            return Err(CliError::new(
                "mcp_oauth_callback",
                format!("OAuth callback repeated {key}; refusing ambiguous authorization response"),
            ));
        }
        Ok(values.into_iter().next())
    };
    if let Some(error) = unique("error")? {
        respond(&mut stream, false);
        return Err(CliError::new(
            "mcp_oauth_authorization",
            format!("OAuth authorization failed: {error}"),
        ));
    }
    let state = unique("state")?
        .ok_or_else(|| CliError::new("mcp_oauth_state", "OAuth callback omitted state"))?;
    if state != expected_state {
        respond(&mut stream, false);
        return Err(CliError::new(
            "mcp_oauth_state",
            "OAuth callback state did not match",
        ));
    }
    let issuer = unique("iss")?.ok_or_else(|| {
        CliError::new(
            "mcp_oauth_issuer",
            "OAuth callback omitted RFC 9207 iss; refusing authorization-server mix-up risk",
        )
    })?;
    if issuer != expected_issuer {
        respond(&mut stream, false);
        return Err(CliError::new(
            "mcp_oauth_issuer",
            "OAuth callback issuer did not match the configured issuer",
        ));
    }
    let code = unique("code")?.ok_or_else(|| {
        CliError::new(
            "mcp_oauth_callback",
            "OAuth callback omitted authorization code",
        )
    })?;
    respond(&mut stream, true);
    Ok(CallbackResult {
        code: code.to_string(),
    })
}

fn respond(stream: &mut TcpStream, success: bool) {
    let body = if success {
        "Reason OAuth login completed. You can close this window."
    } else {
        "Reason OAuth login failed. Return to the terminal for details."
    };
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn exchange_code(
    name: &str,
    resource: &str,
    oauth: &McpRemoteOAuthFileConfig,
    redirect_uri: &str,
    verifier: &str,
    callback: &CallbackResult,
) -> Result<StoredOAuthToken, CliError> {
    let form = vec![
        ("grant_type".to_string(), "authorization_code".to_string()),
        ("code".to_string(), callback.code.clone()),
        ("redirect_uri".to_string(), redirect_uri.to_string()),
        ("client_id".to_string(), oauth.client_id.clone()),
        ("code_verifier".to_string(), verifier.to_string()),
        ("resource".to_string(), resource.to_string()),
    ];
    let token = token_request(&oauth.token_endpoint, form, "exchange")?;
    token_from_response(name, resource, oauth, token)
}

fn refresh(
    name: &str,
    resource: &str,
    oauth: &McpRemoteOAuthFileConfig,
    refresh_token: &str,
) -> Result<StoredOAuthToken, CliError> {
    let mut form = vec![
        ("grant_type".to_string(), "refresh_token".to_string()),
        ("refresh_token".to_string(), refresh_token.to_string()),
        ("client_id".to_string(), oauth.client_id.clone()),
        ("resource".to_string(), resource.to_string()),
    ];
    if !oauth.scopes.is_empty() {
        form.push(("scope".to_string(), oauth.scopes.join(" ")));
    }
    let mut refreshed = token_request(&oauth.token_endpoint, form, "refresh")?;
    if refreshed.refresh_token.is_none() {
        refreshed.refresh_token = Some(refresh_token.into());
    }
    token_from_response(name, resource, oauth, refreshed)
}

fn token_request(
    endpoint: &str,
    form: Vec<(String, String)>,
    operation: &'static str,
) -> Result<TokenResponse, CliError> {
    let endpoint = endpoint.to_string();
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| CliError::new("mcp_oauth_transport", "cannot build OAuth runtime"))?;
        runtime.block_on(async move {
            let client = Client::builder()
                .timeout(Duration::from_secs(30))
                .redirect(redirect::Policy::none())
                .build()
                .map_err(|_| {
                    CliError::new("mcp_oauth_transport", "cannot build OAuth HTTP client")
                })?;
            let response = client
                .post(&endpoint)
                .form(&form)
                .send()
                .await
                .map_err(|_| {
                    CliError::new(
                        "mcp_oauth_transport",
                        format!("OAuth token {operation} failed"),
                    )
                })?;
            if !response.status().is_success() {
                return Err(CliError::new(
                    "mcp_oauth_token",
                    format!(
                        "OAuth token endpoint returned HTTP {}",
                        response.status().as_u16()
                    ),
                ));
            }
            response.json::<TokenResponse>().await.map_err(|_| {
                CliError::new("mcp_oauth_token", "OAuth token response was invalid JSON")
            })
        })
    })
    .join()
    .map_err(|_| CliError::new("mcp_oauth_transport", "OAuth worker thread failed"))?
}

fn token_from_response(
    name: &str,
    resource: &str,
    oauth: &McpRemoteOAuthFileConfig,
    token: TokenResponse,
) -> Result<StoredOAuthToken, CliError> {
    if token.access_token.trim().is_empty() || !token.token_type.eq_ignore_ascii_case("bearer") {
        return Err(CliError::new(
            "mcp_oauth_token",
            "OAuth token response must contain a non-empty Bearer access token",
        ));
    }
    let now = now_unix();
    Ok(StoredOAuthToken {
        schema_version: "reason-mcp-oauth-token-v1".into(),
        server_name: name.into(),
        issuer: oauth.issuer.clone(),
        client_id: oauth.client_id.clone(),
        resource: resource.to_string(),
        token_endpoint: oauth.token_endpoint.clone(),
        access_token: token.access_token,
        refresh_token: token.refresh_token.filter(|value| !value.trim().is_empty()),
        token_type: "Bearer".into(),
        scope: token.scope,
        expires_at_unix_seconds: token.expires_in.map(|seconds| now.saturating_add(seconds)),
    })
}

fn token_matches_config(
    token: &StoredOAuthToken,
    name: &str,
    resource: &str,
    oauth: &McpRemoteOAuthFileConfig,
) -> bool {
    token.schema_version == "reason-mcp-oauth-token-v1"
        && token.server_name == name
        && token.issuer == oauth.issuer
        && token.client_id == oauth.client_id
        && token.resource == resource
        && token.token_endpoint == oauth.token_endpoint
        && token.token_type.eq_ignore_ascii_case("bearer")
        && !token.access_token.trim().is_empty()
}

fn token_is_fresh(token: &StoredOAuthToken) -> bool {
    token
        .expires_at_unix_seconds
        .is_none_or(|expires| expires > now_unix().saturating_add(60))
}

fn account(name: &str) -> String {
    format!("mcp:{name}:oauth")
}

fn entry(name: &str) -> Result<Entry, CliError> {
    Entry::new(MCP_OAUTH_SERVICE, &account(name)).map_err(store_error)
}

fn load_token(name: &str) -> Result<Option<StoredOAuthToken>, CliError> {
    let value = match entry(name)?.get_password() {
        Ok(value) => value,
        Err(KeyringError::NoEntry) => return Ok(None),
        Err(error) => return Err(store_error(error)),
    };
    let token = serde_json::from_str(&value).map_err(|_| {
        CliError::new(
            "mcp_oauth_store",
            "stored MCP OAuth credential is malformed; log out and authenticate again",
        )
    })?;
    Ok(Some(token))
}

fn save_token(name: &str, token: &StoredOAuthToken) -> Result<(), CliError> {
    let value = serde_json::to_string(token)
        .map_err(|_| CliError::new("mcp_oauth_store", "cannot serialize MCP OAuth credential"))?;
    entry(name)?.set_password(&value).map_err(store_error)
}

fn delete_token(name: &str) -> Result<bool, CliError> {
    match entry(name)?.delete_credential() {
        Ok(()) => Ok(true),
        Err(KeyringError::NoEntry) => Ok(false),
        Err(error) => Err(store_error(error)),
    }
}

fn store_error(error: KeyringError) -> CliError {
    match error {
        KeyringError::NoDefaultStore
        | KeyringError::NoStorageAccess(_)
        | KeyringError::PlatformFailure(_)
        | KeyringError::NotSupportedByStore(_) => CliError::new(
            "credential_store_unavailable",
            "native OS credential store is unavailable for MCP OAuth",
        ),
        _ => CliError::new(
            "mcp_oauth_store",
            "native OS credential store failed for MCP OAuth",
        ),
    }
}

fn random_urlsafe(bytes: usize) -> String {
    let mut raw = vec![0u8; bytes];
    rand::rng().fill(&mut raw[..]);
    URL_SAFE_NO_PAD.encode(raw)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or_default()
}

fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().map(|_| ())
    }
    #[cfg(target_os = "windows")]
    {
        return Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .spawn()
            .map(|_| ());
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn().map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oauth() -> McpRemoteOAuthFileConfig {
        McpRemoteOAuthFileConfig {
            issuer: "https://auth.example.test".into(),
            authorization_endpoint: "https://auth.example.test/authorize".into(),
            token_endpoint: "https://auth.example.test/token".into(),
            client_id: "https://client.example.test/reason.json".into(),
            scopes: vec!["mcp:read".into()],
        }
    }

    #[test]
    fn authorization_url_uses_pkce_state_resource_and_scope_without_secret() {
        let url = build_authorization_url(
            &oauth(),
            "https://mcp.example.test/mcp",
            "http://127.0.0.1:1234/oauth/callback",
            "state-value",
            "challenge-value",
        )
        .unwrap();
        assert!(url.contains("code_challenge=challenge-value"));
        assert!(url.contains("state=state-value"));
        assert!(url.contains("resource=https%3A%2F%2Fmcp.example.test%2Fmcp"));
        assert!(url.contains("scope=mcp%3Aread"));
        assert!(!url.contains("access_token"));
        assert!(!url.contains("client_secret"));
    }

    fn callback_result(query: &str) -> Result<CallbackResult, CliError> {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let query = query.to_string();
        let sender = thread::spawn(move || {
            let mut stream = TcpStream::connect(addr).unwrap();
            let request = format!(
                "GET {CALLBACK_PATH}?{query} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
            );
            stream.write_all(request.as_bytes()).unwrap();
        });
        let (stream, _) = listener.accept().unwrap();
        let result = handle_callback(stream, "expected-state", "https://auth.example.test");
        sender.join().unwrap();
        result
    }

    #[test]
    fn callback_requires_exact_state_and_rfc9207_issuer() {
        let ok =
            callback_result("code=abc&state=expected-state&iss=https%3A%2F%2Fauth.example.test")
                .unwrap();
        assert_eq!(ok.code, "abc");

        let wrong_state =
            callback_result("code=abc&state=wrong&iss=https%3A%2F%2Fauth.example.test")
                .unwrap_err();
        assert_eq!(wrong_state.failure_class, "mcp_oauth_state");

        let missing_issuer = callback_result("code=abc&state=expected-state").unwrap_err();
        assert_eq!(missing_issuer.failure_class, "mcp_oauth_issuer");

        let wrong_issuer =
            callback_result("code=abc&state=expected-state&iss=https%3A%2F%2Fevil.example.test")
                .unwrap_err();
        assert_eq!(wrong_issuer.failure_class, "mcp_oauth_issuer");

        let duplicate_state = callback_result(
            "code=abc&state=wrong&state=expected-state&iss=https%3A%2F%2Fauth.example.test",
        )
        .unwrap_err();
        assert_eq!(duplicate_state.failure_class, "mcp_oauth_callback");
    }

    #[test]
    fn authorization_code_exchange_uses_pkce_resource_and_never_requires_client_secret() {
        use std::io::Read as _;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut first = String::new();
            reader.read_line(&mut first).unwrap();
            assert!(first.starts_with("POST /token "));
            let mut content_length = 0usize;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    content_length = value.trim().parse().unwrap();
                }
            }
            let mut body = vec![0u8; content_length];
            reader.read_exact(&mut body).unwrap();
            let body = String::from_utf8(body).unwrap();
            assert!(body.contains("grant_type=authorization_code"));
            assert!(body.contains("code=auth-code"));
            assert!(body.contains("code_verifier=verifier-value"));
            assert!(body.contains("resource=https%3A%2F%2Fmcp.example.test%2Fmcp"));
            assert!(body.contains("client_id=client-id"));
            assert!(!body.contains("client_secret"));
            let response_body = r#"{"access_token":"access-secret","token_type":"Bearer","refresh_token":"refresh-secret","expires_in":3600}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        let mut config = oauth();
        config.client_id = "client-id".into();
        config.token_endpoint = format!("http://{addr}/token");
        let callback = CallbackResult {
            code: "auth-code".into(),
        };
        let token = exchange_code(
            "demo",
            "https://mcp.example.test/mcp",
            &config,
            "http://127.0.0.1:1234/oauth/callback",
            "verifier-value",
            &callback,
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(token.access_token, "access-secret");
        assert_eq!(token.refresh_token.as_deref(), Some("refresh-secret"));
        assert_eq!(token.issuer, "https://auth.example.test");
        assert_eq!(token.client_id, "client-id");
        assert_eq!(token.resource, "https://mcp.example.test/mcp");
        assert_eq!(token.token_endpoint, config.token_endpoint);
    }

    #[test]
    fn stored_token_is_issuer_and_client_bound() {
        let config = oauth();
        let token = StoredOAuthToken {
            schema_version: "reason-mcp-oauth-token-v1".into(),
            server_name: "demo".into(),
            issuer: config.issuer.clone(),
            client_id: config.client_id.clone(),
            resource: "https://mcp.example.test/mcp".into(),
            token_endpoint: config.token_endpoint.clone(),
            access_token: "secret".into(),
            refresh_token: Some("refresh".into()),
            token_type: "Bearer".into(),
            scope: None,
            expires_at_unix_seconds: None,
        };
        assert!(token_matches_config(
            &token,
            "demo",
            "https://mcp.example.test/mcp",
            &config
        ));
        let mut changed = config.clone();
        changed.issuer = "https://other.example.test".into();
        assert!(!token_matches_config(
            &token,
            "demo",
            "https://mcp.example.test/mcp",
            &changed
        ));
        assert!(!token_matches_config(
            &token,
            "demo",
            "https://evil.example.test/mcp",
            &config
        ));
        let mut changed_token_endpoint = config.clone();
        changed_token_endpoint.token_endpoint = "https://evil.example.test/token".into();
        assert!(!token_matches_config(
            &token,
            "demo",
            "https://mcp.example.test/mcp",
            &changed_token_endpoint
        ));
    }

    #[test]
    #[ignore = "mutates the native OS credential store; run only in isolated CI"]
    fn native_os_store_round_trip() {
        let name = format!("ci-oauth-{}", std::process::id());
        let config = oauth();
        let token = StoredOAuthToken {
            schema_version: "reason-mcp-oauth-token-v1".into(),
            server_name: name.clone(),
            issuer: config.issuer.clone(),
            client_id: config.client_id.clone(),
            resource: "https://mcp.example.test/mcp".into(),
            token_endpoint: config.token_endpoint.clone(),
            access_token: "ci-access-token".into(),
            refresh_token: Some("ci-refresh-token".into()),
            token_type: "Bearer".into(),
            scope: Some("mcp:read".into()),
            expires_at_unix_seconds: Some(now_unix().saturating_add(3600)),
        };
        let _ = delete_token(&name);
        save_token(&name, &token).unwrap();
        let loaded = load_token(&name).unwrap().unwrap();
        assert!(token_matches_config(
            &loaded,
            &name,
            "https://mcp.example.test/mcp",
            &config
        ));
        assert_eq!(loaded.access_token, "ci-access-token");
        assert!(delete_token(&name).unwrap());
        assert!(load_token(&name).unwrap().is_none());
    }

    #[test]
    fn oauth_requires_https_except_loopback() {
        let mut config = oauth();
        config.token_endpoint = "http://example.test/token".into();
        assert!(validate_oauth_config(&config).is_err());
        config.token_endpoint = "http://127.0.0.1:8080/token".into();
        assert!(validate_oauth_config(&config).is_ok());
    }
}
