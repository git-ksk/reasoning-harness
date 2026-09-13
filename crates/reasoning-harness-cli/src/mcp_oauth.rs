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
use reasoning_harness_providers::{McpRemoteScopeChallenge, network};
use reqwest::{Client, StatusCode, header, redirect};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;

use super::{CliError, McpRemoteOAuthFileConfig, OutputFormat, print_product_json};

const MCP_OAUTH_SERVICE: &str = "io.github.git-ksk.reason-cli.mcp-oauth.v1";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);
const CALLBACK_PATH: &str = "/oauth/callback";
const MAX_DISCOVERY_METADATA_BYTES: usize = 262_144;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum McpOAuthCredentialCleanup {
    Removed,
    Absent,
    BindingMismatch,
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
    let granted_scope = token.scope.clone();
    token = refresh(
        name,
        endpoint,
        oauth,
        &refresh_token,
        granted_scope.as_deref(),
    )?;
    save_token(name, &token)?;
    Ok(token.access_token)
}

#[derive(Debug, Deserialize)]
struct ProtectedResourceMetadata {
    resource: String,
    authorization_servers: Vec<String>,
    #[serde(default)]
    scopes_supported: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AuthorizationServerMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    code_challenge_methods_supported: Vec<String>,
}

#[derive(Debug)]
struct ResourceDiscovery {
    metadata: ProtectedResourceMetadata,
    challenge_scope: Option<Vec<String>>,
}

#[derive(Debug)]
struct BearerResourceChallenge {
    resource_metadata: String,
    scope: Option<Vec<String>>,
}

pub(crate) fn discover_oauth_config(
    endpoint: &str,
    client_id: String,
    requested_scopes: Vec<String>,
    issuer_override: Option<String>,
    authorization_endpoint_override: Option<String>,
    token_endpoint_override: Option<String>,
) -> Result<McpRemoteOAuthFileConfig, CliError> {
    validate_resource_endpoint(endpoint)?;
    if client_id.trim().is_empty() || requested_scopes.iter().any(|scope| scope.trim().is_empty()) {
        return Err(CliError::new(
            "mcp_oauth_configuration",
            "OAuth client_id and every requested scope must be non-empty",
        ));
    }
    let override_count = [
        issuer_override.as_ref(),
        authorization_endpoint_override.as_ref(),
        token_endpoint_override.as_ref(),
    ]
    .into_iter()
    .flatten()
    .count();
    if override_count != 0 && override_count != 3 {
        return Err(CliError::new(
            "mcp_oauth_configuration",
            "advanced OAuth metadata overrides require --issuer, --authorization-endpoint, and --token-endpoint together",
        ));
    }

    let discovery = discover_protected_resource_metadata(endpoint)?;
    validate_protected_resource_metadata(endpoint, &discovery.metadata)?;
    let issuer = select_authorization_server(
        &discovery.metadata.authorization_servers,
        issuer_override.as_deref(),
    )?;
    let metadata = discover_authorization_server_metadata(&issuer)?;
    validate_authorization_server_metadata(&issuer, &metadata)?;

    if let (Some(override_issuer), Some(override_authorization), Some(override_token)) = (
        issuer_override.as_deref(),
        authorization_endpoint_override.as_deref(),
        token_endpoint_override.as_deref(),
    ) && (!urls_equal(override_issuer, &metadata.issuer)
        || !urls_equal(override_authorization, &metadata.authorization_endpoint)
        || !urls_equal(override_token, &metadata.token_endpoint))
    {
        return Err(CliError::new(
            "mcp_oauth_discovery",
            "advanced OAuth metadata overrides did not match discovered authorization-server metadata",
        ));
    }

    let scopes = if !requested_scopes.is_empty() {
        requested_scopes
    } else if let Some(challenge) = discovery.challenge_scope {
        challenge
    } else {
        discovery.metadata.scopes_supported
    };
    let oauth = McpRemoteOAuthFileConfig {
        issuer: metadata.issuer,
        authorization_endpoint: metadata.authorization_endpoint,
        token_endpoint: metadata.token_endpoint,
        client_id,
        scopes,
    };
    validate_oauth_config(&oauth)?;
    Ok(oauth)
}

fn discover_protected_resource_metadata(endpoint: &str) -> Result<ResourceDiscovery, CliError> {
    if let Some(challenge) = discover_resource_metadata_challenge(endpoint)? {
        let metadata =
            fetch_discovery_json::<ProtectedResourceMetadata>(&challenge.resource_metadata, false)?
                .ok_or_else(|| {
                    CliError::new(
                        "mcp_oauth_discovery",
                        "WWW-Authenticate resource_metadata URL returned no metadata",
                    )
                })?;
        return Ok(ResourceDiscovery {
            metadata,
            challenge_scope: challenge.scope,
        });
    }

    let endpoint_url = Url::parse(endpoint).map_err(|_| {
        CliError::new(
            "mcp_oauth_discovery",
            "remote MCP endpoint must be a valid URL for OAuth discovery",
        )
    })?;
    for candidate in protected_resource_metadata_urls(&endpoint_url)? {
        if let Some(metadata) =
            fetch_discovery_json::<ProtectedResourceMetadata>(candidate.as_str(), true)?
        {
            return Ok(ResourceDiscovery {
                metadata,
                challenge_scope: None,
            });
        }
    }
    Err(CliError::new(
        "mcp_oauth_discovery",
        "remote MCP server exposed neither a usable WWW-Authenticate resource_metadata URL nor OAuth Protected Resource Metadata well-known endpoint",
    ))
}

fn discover_resource_metadata_challenge(
    endpoint: &str,
) -> Result<Option<BearerResourceChallenge>, CliError> {
    let endpoint = endpoint.to_string();
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| CliError::new("mcp_oauth_transport", "cannot build OAuth discovery runtime"))?;
        runtime.block_on(async move {
            let response = discovery_client()?
                .post(&endpoint)
                .header("MCP-Protocol-Version", "2026-07-28")
                .header("Mcp-Method", "tools/list")
                .header(header::ACCEPT, "application/json, text/event-stream")
                .json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "reasoning-harness:oauth-discovery",
                    "method": "tools/list",
                    "params": {"_meta": {
                        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
                        "io.modelcontextprotocol/clientInfo": {"name": "reasoning-harness", "version": env!("CARGO_PKG_VERSION")},
                        "io.modelcontextprotocol/clientCapabilities": {}
                    }}
                }))
                .send()
                .await
                .map_err(|_| CliError::new("mcp_oauth_transport", "remote MCP OAuth discovery probe failed"))?;
            if response.status() != StatusCode::UNAUTHORIZED {
                return Ok(None);
            }
            for value in response.headers().get_all(header::WWW_AUTHENTICATE) {
                let Ok(value) = value.to_str() else { continue };
                if let Some(challenge) = parse_bearer_challenge(value)? {
                    validate_discovery_url(&challenge.resource_metadata, "resource_metadata")?;
                    return Ok(Some(challenge));
                }
            }
            Ok(None)
        })
    })
    .join()
    .map_err(|_| CliError::new("mcp_oauth_transport", "OAuth discovery worker thread failed"))?
}

fn parse_bearer_challenge(value: &str) -> Result<Option<BearerResourceChallenge>, CliError> {
    let trimmed = value.trim();
    if !trimmed
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("bearer "))
    {
        return Ok(None);
    }
    let mut resource_metadata = None;
    let mut scope = None;
    for part in split_auth_params(&trimmed[7..]) {
        let Some((name, raw)) = part.split_once('=') else {
            continue;
        };
        let raw = raw.trim();
        let decoded = if raw.starts_with('"') {
            if raw.len() < 2 || !raw.ends_with('"') {
                return Err(CliError::new(
                    "mcp_oauth_discovery",
                    "malformed quoted WWW-Authenticate parameter",
                ));
            }
            unescape_quoted(&raw[1..raw.len() - 1])?
        } else {
            raw.to_string()
        };
        match name.trim() {
            "resource_metadata" => resource_metadata = Some(decoded),
            "scope" => {
                let values = decoded
                    .split_ascii_whitespace()
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                if !values.is_empty() {
                    scope = Some(values);
                }
            }
            _ => {}
        }
    }
    Ok(resource_metadata.map(|metadata| BearerResourceChallenge {
        resource_metadata: metadata,
        scope,
    }))
}

fn unescape_quoted(value: &str) -> Result<String, CliError> {
    let mut output = String::with_capacity(value.len());
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            output.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    if escaped {
        return Err(CliError::new(
            "mcp_oauth_discovery",
            "malformed quoted WWW-Authenticate escape",
        ));
    }
    Ok(output)
}

fn split_auth_params(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' && quoted {
            escaped = true;
        } else if character == '"' {
            quoted = !quoted;
        } else if character == ',' && !quoted {
            parts.push(input[start..index].trim());
            start = index + 1;
        }
    }
    parts.push(input[start..].trim());
    parts
}

fn protected_resource_metadata_urls(endpoint: &Url) -> Result<Vec<Url>, CliError> {
    let endpoint_path = endpoint.path().trim_start_matches('/');
    let mut path_specific = endpoint.clone();
    path_specific.set_query(None);
    path_specific.set_fragment(None);
    let path = if endpoint_path.is_empty() {
        "/.well-known/oauth-protected-resource".to_string()
    } else {
        format!("/.well-known/oauth-protected-resource/{endpoint_path}")
    };
    path_specific.set_path(&path);
    let mut urls = vec![path_specific.clone()];
    if !endpoint_path.is_empty() {
        path_specific.set_path("/.well-known/oauth-protected-resource");
        urls.push(path_specific);
    }
    for url in &urls {
        validate_discovery_url(url.as_str(), "protected-resource metadata")?;
    }
    Ok(urls)
}

fn discover_authorization_server_metadata(
    issuer: &str,
) -> Result<AuthorizationServerMetadata, CliError> {
    let issuer_url = Url::parse(issuer).map_err(|_| {
        CliError::new(
            "mcp_oauth_discovery",
            "authorization server issuer is not a valid URL",
        )
    })?;
    validate_discovery_url(issuer, "authorization server issuer")?;
    for candidate in authorization_server_metadata_urls(&issuer_url)? {
        if let Some(metadata) =
            fetch_discovery_json::<AuthorizationServerMetadata>(candidate.as_str(), true)?
        {
            return Ok(metadata);
        }
    }
    Err(CliError::new(
        "mcp_oauth_discovery",
        "authorization server exposed neither RFC 8414 nor OpenID Connect discovery metadata",
    ))
}

fn authorization_server_metadata_urls(issuer: &Url) -> Result<Vec<Url>, CliError> {
    let mut base = issuer.clone();
    base.set_query(None);
    base.set_fragment(None);
    let issuer_path = issuer.path().trim_matches('/');
    let paths = if issuer_path.is_empty() {
        vec![
            "/.well-known/oauth-authorization-server".to_string(),
            "/.well-known/openid-configuration".to_string(),
        ]
    } else {
        vec![
            format!("/.well-known/oauth-authorization-server/{issuer_path}"),
            format!("/.well-known/openid-configuration/{issuer_path}"),
            format!("/{issuer_path}/.well-known/openid-configuration"),
        ]
    };
    let mut urls = Vec::with_capacity(paths.len());
    for path in paths {
        let mut url = base.clone();
        url.set_path(&path);
        validate_discovery_url(url.as_str(), "authorization-server metadata")?;
        urls.push(url);
    }
    Ok(urls)
}

fn fetch_discovery_json<T: for<'de> Deserialize<'de> + Send + 'static>(
    url: &str,
    allow_not_found: bool,
) -> Result<Option<T>, CliError> {
    validate_discovery_url(url, "OAuth metadata")?;
    let url = url.to_string();
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| CliError::new("mcp_oauth_transport", "cannot build OAuth discovery runtime"))?;
        runtime.block_on(async move {
            let response = discovery_client()?
                .get(&url)
                .header(header::ACCEPT, "application/json")
                .send()
                .await
                .map_err(|_| CliError::new("mcp_oauth_transport", "OAuth metadata request failed"))?;
            if allow_not_found && response.status() == StatusCode::NOT_FOUND {
                return Ok(None);
            }
            if response.status().is_redirection() {
                return Err(CliError::new(
                    "mcp_oauth_discovery",
                    "OAuth metadata endpoint attempted an HTTP redirect; refusing unvalidated redirect",
                ));
            }
            if !response.status().is_success() {
                return Err(CliError::new(
                    "mcp_oauth_discovery",
                    format!("OAuth metadata endpoint returned HTTP {}", response.status().as_u16()),
                ));
            }
            let mut response = response;
            let mut body = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|_| {
                CliError::new("mcp_oauth_transport", "OAuth metadata response read failed")
            })? {
                if body.len().saturating_add(chunk.len()) > MAX_DISCOVERY_METADATA_BYTES {
                    return Err(CliError::new(
                        "mcp_oauth_discovery",
                        "OAuth metadata response exceeded the bounded discovery size limit",
                    ));
                }
                body.extend_from_slice(&chunk);
            }
            serde_json::from_slice::<T>(&body).map(Some).map_err(|_| {
                CliError::new("mcp_oauth_discovery", "OAuth metadata response was invalid or incomplete JSON")
            })
        })
    })
    .join()
    .map_err(|_| CliError::new("mcp_oauth_transport", "OAuth discovery worker thread failed"))?
}

fn discovery_client() -> Result<Client, CliError> {
    network::client_builder()
        .map_err(|error| CliError::new("network_custom_ca", error.message().to_string()))?
        .timeout(Duration::from_secs(30))
        .redirect(redirect::Policy::none())
        .build()
        .map_err(|_| {
            CliError::new(
                "mcp_oauth_transport",
                "cannot build OAuth discovery HTTP client",
            )
        })
}

fn validate_protected_resource_metadata(
    endpoint: &str,
    metadata: &ProtectedResourceMetadata,
) -> Result<(), CliError> {
    if metadata.authorization_servers.is_empty() {
        return Err(CliError::new(
            "mcp_oauth_discovery",
            "Protected Resource Metadata omitted authorization_servers",
        ));
    }
    if !urls_equal(endpoint, &metadata.resource) {
        return Err(CliError::new(
            "mcp_oauth_discovery",
            "Protected Resource Metadata resource did not match the configured MCP endpoint",
        ));
    }
    for issuer in &metadata.authorization_servers {
        validate_discovery_url(issuer, "authorization server issuer")?;
    }
    Ok(())
}

fn select_authorization_server(
    authorization_servers: &[String],
    issuer_override: Option<&str>,
) -> Result<String, CliError> {
    if let Some(issuer) = issuer_override {
        validate_discovery_url(issuer, "authorization server issuer override")?;
        if let Some(found) = authorization_servers
            .iter()
            .find(|candidate| urls_equal(candidate, issuer))
        {
            return Ok(found.clone());
        }
        return Err(CliError::new(
            "mcp_oauth_discovery",
            "advanced --issuer override was not advertised by Protected Resource Metadata",
        ));
    }
    if authorization_servers.len() != 1 {
        return Err(CliError::new(
            "mcp_oauth_discovery",
            "Protected Resource Metadata advertised multiple authorization servers; use the advanced --issuer override to choose one advertised issuer explicitly",
        ));
    }
    Ok(authorization_servers[0].clone())
}

fn validate_authorization_server_metadata(
    expected_issuer: &str,
    metadata: &AuthorizationServerMetadata,
) -> Result<(), CliError> {
    if !urls_equal(expected_issuer, &metadata.issuer) {
        return Err(CliError::new(
            "mcp_oauth_discovery",
            "authorization-server metadata issuer did not match the issuer advertised by the protected resource",
        ));
    }
    validate_discovery_url(&metadata.issuer, "authorization server issuer")?;
    validate_discovery_url(&metadata.authorization_endpoint, "authorization endpoint")?;
    validate_discovery_url(&metadata.token_endpoint, "token endpoint")?;
    if !metadata
        .code_challenge_methods_supported
        .iter()
        .any(|method| method == "S256")
    {
        return Err(CliError::new(
            "mcp_oauth_discovery",
            "authorization-server metadata does not advertise PKCE S256 support",
        ));
    }
    Ok(())
}

fn validate_discovery_url(value: &str, label: &str) -> Result<(), CliError> {
    let url = Url::parse(value).map_err(|_| {
        CliError::new(
            "mcp_oauth_discovery",
            format!("{label} must be a valid URL"),
        )
    })?;
    if (url.scheme() != "https" && !loopback_http(&url))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(CliError::new(
            "mcp_oauth_discovery",
            format!(
                "{label} must use a credential-free HTTPS URL without a fragment (loopback HTTP is allowed only for deterministic local tests)"
            ),
        ));
    }
    Ok(())
}

fn urls_equal(left: &str, right: &str) -> bool {
    match (Url::parse(left), Url::parse(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

pub(crate) fn insufficient_scope_error(
    name: &str,
    challenge: &McpRemoteScopeChallenge,
) -> CliError {
    let scopes = challenge.required_scopes.join(", ");
    let recovery = if safe_cli_atom(name)
        && challenge
            .required_scopes
            .iter()
            .all(|scope| safe_cli_atom(scope))
    {
        let scope_args = challenge
            .required_scopes
            .iter()
            .map(|scope| format!(" --scope {scope}"))
            .collect::<String>();
        format!("reason mcp login {name} --replace{scope_args}")
    } else {
        "reason mcp login <name> --replace, with one --scope argument for each required scope"
            .to_string()
    };
    CliError::new(
        "mcp_insufficient_scope",
        format!(
            "remote MCP source {name:?} requires additional OAuth scope(s) [{scopes}] for resource {}; authorization was not broadened automatically. Reauthorize explicitly with: {recovery}",
            challenge.resource
        ),
    )
}

fn safe_cli_atom(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':' | b'/' | b'~')
        })
}

pub(crate) fn validate_scope_values(scopes: &[String]) -> Result<(), CliError> {
    if scopes.iter().any(|scope| {
        scope.is_empty()
            || scope.len() > 128
            || !scope.bytes().all(|byte| {
                byte == 0x21 || (0x23..=0x5b).contains(&byte) || (0x5d..=0x7e).contains(&byte)
            })
    }) {
        return Err(CliError::new(
            "mcp_oauth_configuration",
            "OAuth scopes must be individual RFC 6749 scope-token values with bounded printable characters",
        ));
    }
    Ok(())
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
    validate_scope_values(&oauth.scopes)?;
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
    granted_scope: Option<&str>,
) -> Result<StoredOAuthToken, CliError> {
    let mut form = vec![
        ("grant_type".to_string(), "refresh_token".to_string()),
        ("refresh_token".to_string(), refresh_token.to_string()),
        ("client_id".to_string(), oauth.client_id.clone()),
        ("resource".to_string(), resource.to_string()),
    ];
    if let Some(scope) = granted_scope.filter(|scope| !scope.trim().is_empty()) {
        form.push(("scope".to_string(), scope.to_string()));
    } else if !oauth.scopes.is_empty() {
        form.push(("scope".to_string(), oauth.scopes.join(" ")));
    }
    let mut refreshed = token_request(&oauth.token_endpoint, form, "refresh")?;
    if refreshed.refresh_token.is_none() {
        refreshed.refresh_token = Some(refresh_token.into());
    }
    if refreshed.scope.is_none() {
        refreshed.scope = granted_scope
            .filter(|scope| !scope.trim().is_empty())
            .map(str::to_string);
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
            let client = network::client_builder()
                .map_err(|error| CliError::new("network_custom_ca", error.message().to_string()))?
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
        scope: token
            .scope
            .or_else(|| (!oauth.scopes.is_empty()).then(|| oauth.scopes.join(" "))),
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

pub(crate) fn delete_matching_stored_credential(
    name: &str,
    resource: &str,
    oauth: &McpRemoteOAuthFileConfig,
) -> Result<McpOAuthCredentialCleanup, CliError> {
    validate_oauth_config(oauth)?;
    let Some(token) = load_token(name)? else {
        return Ok(McpOAuthCredentialCleanup::Absent);
    };
    if !token_matches_config(&token, name, resource, oauth) {
        return Ok(McpOAuthCredentialCleanup::BindingMismatch);
    }
    Ok(if delete_token(name)? {
        McpOAuthCredentialCleanup::Removed
    } else {
        McpOAuthCredentialCleanup::Absent
    })
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

    fn read_http_request(stream: &mut TcpStream) -> String {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut first = String::new();
        reader.read_line(&mut first).unwrap();
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
        if content_length > 0 {
            let mut body = vec![0u8; content_length];
            std::io::Read::read_exact(&mut reader, &mut body).unwrap();
        }
        first
    }

    fn write_http_response(stream: &mut TcpStream, status: &str, headers: &str, body: &str) {
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\n{headers}Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
    }

    #[test]
    fn oauth_discovery_falls_back_to_root_prm_and_oidc_path_insertion() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let endpoint = format!("http://{addr}/mcp");
        let issuer = format!("http://{addr}/tenant");
        let endpoint_server = endpoint.clone();
        let issuer_server = issuer.clone();
        let server = thread::spawn(move || {
            for index in 0..5 {
                let (mut stream, _) = listener.accept().unwrap();
                let first = read_http_request(&mut stream);
                match index {
                    0 => {
                        assert!(first.starts_with("POST /mcp "));
                        write_http_response(&mut stream, "200 OK", "", "{}");
                    }
                    1 => {
                        assert!(
                            first.starts_with("GET /.well-known/oauth-protected-resource/mcp ")
                        );
                        write_http_response(&mut stream, "404 Not Found", "", "");
                    }
                    2 => {
                        assert!(first.starts_with("GET /.well-known/oauth-protected-resource "));
                        let body = serde_json::json!({
                            "resource": endpoint_server,
                            "authorization_servers": [issuer_server],
                            "scopes_supported": ["mcp:read"]
                        })
                        .to_string();
                        write_http_response(&mut stream, "200 OK", "", &body);
                    }
                    3 => {
                        assert!(
                            first
                                .starts_with("GET /.well-known/oauth-authorization-server/tenant ")
                        );
                        write_http_response(&mut stream, "404 Not Found", "", "");
                    }
                    _ => {
                        assert!(first.starts_with("GET /.well-known/openid-configuration/tenant "));
                        let body = serde_json::json!({
                            "issuer": issuer_server,
                            "authorization_endpoint": format!("{issuer_server}/authorize"),
                            "token_endpoint": format!("{issuer_server}/token"),
                            "code_challenge_methods_supported": ["S256"]
                        })
                        .to_string();
                        write_http_response(&mut stream, "200 OK", "", &body);
                    }
                }
            }
        });
        let discovered = discover_oauth_config(
            &endpoint,
            "https://client.example.test/reason.json".into(),
            Vec::new(),
            None,
            None,
            None,
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(discovered.issuer, issuer);
        assert_eq!(discovered.scopes, vec!["mcp:read"]);
        assert!(discovered.authorization_endpoint.ends_with("/authorize"));
        assert!(discovered.token_endpoint.ends_with("/token"));
    }

    #[test]
    fn oauth_discovery_rejects_redirect_and_malformed_metadata() {
        for (status, headers, body) in [
            (
                "302 Found",
                "Location: http://127.0.0.1:9/elsewhere\r\n",
                "",
            ),
            ("200 OK", "", "not-json"),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let addr = listener.local_addr().unwrap();
            let server = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let first = read_http_request(&mut stream);
                assert!(first.starts_with("GET /metadata "));
                write_http_response(&mut stream, status, headers, body);
            });
            let error = fetch_discovery_json::<ProtectedResourceMetadata>(
                &format!("http://{addr}/metadata"),
                false,
            )
            .unwrap_err();
            server.join().unwrap();
            assert_eq!(error.failure_class, "mcp_oauth_discovery");
        }
    }

    #[test]
    fn oauth_discovery_rejects_resource_issuer_and_insecure_metadata_mismatch() {
        let prm = ProtectedResourceMetadata {
            resource: "https://other.example.test/mcp".into(),
            authorization_servers: vec!["https://auth.example.test".into()],
            scopes_supported: vec![],
        };
        assert_eq!(
            validate_protected_resource_metadata("https://mcp.example.test/mcp", &prm)
                .unwrap_err()
                .failure_class,
            "mcp_oauth_discovery"
        );
        let metadata = AuthorizationServerMetadata {
            issuer: "https://other-auth.example.test".into(),
            authorization_endpoint: "https://other-auth.example.test/authorize".into(),
            token_endpoint: "https://other-auth.example.test/token".into(),
            code_challenge_methods_supported: vec!["S256".into()],
        };
        assert_eq!(
            validate_authorization_server_metadata("https://auth.example.test", &metadata)
                .unwrap_err()
                .failure_class,
            "mcp_oauth_discovery"
        );
        assert_eq!(
            validate_discovery_url(
                "http://auth.example.test/.well-known/oauth-authorization-server",
                "metadata"
            )
            .unwrap_err()
            .failure_class,
            "mcp_oauth_discovery"
        );
    }

    #[test]
    fn bearer_challenge_parser_extracts_resource_metadata_and_scope_without_secrets() {
        let challenge = parse_bearer_challenge(
            r#"Bearer realm="mcp", resource_metadata="https://mcp.example.test/.well-known/oauth-protected-resource", scope="files:read user:profile""#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(
            challenge.resource_metadata,
            "https://mcp.example.test/.well-known/oauth-protected-resource"
        );
        assert_eq!(challenge.scope.unwrap(), vec!["files:read", "user:profile"]);
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
    fn refresh_preserves_explicit_step_up_scope_when_server_omits_scope() {
        use std::io::Read as _;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
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
            assert!(body.contains("grant_type=refresh_token"));
            assert!(
                body.contains("scope=files%3Aread+profile%3Aread")
                    || body.contains("scope=files%3Aread%20profile%3Aread")
            );
            let response_body =
                r#"{"access_token":"new-access","token_type":"Bearer","expires_in":3600}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        let mut config = oauth();
        config.token_endpoint = format!("http://{addr}/token");
        let refreshed = refresh(
            "demo",
            "https://mcp.example.test/mcp",
            &config,
            "refresh-secret",
            Some("files:read profile:read"),
        )
        .unwrap();
        server.join().unwrap();
        assert_eq!(refreshed.scope.as_deref(), Some("files:read profile:read"));
        assert_eq!(refreshed.refresh_token.as_deref(), Some("refresh-secret"));
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
    #[ignore = "mutates the native OS credential store; run only in isolated CI"]
    fn native_os_store_scoped_delete_does_not_cross_accounts() {
        let suffix = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let first_name = format!("ci-oauth-first-{suffix}");
        let second_name = format!("ci-oauth-second-{suffix}");
        let config = oauth();
        let token = |name: &str, access_token: &str| StoredOAuthToken {
            schema_version: "reason-mcp-oauth-token-v1".into(),
            server_name: name.into(),
            issuer: config.issuer.clone(),
            client_id: config.client_id.clone(),
            resource: "https://mcp.example.test/mcp".into(),
            token_endpoint: config.token_endpoint.clone(),
            access_token: access_token.into(),
            refresh_token: None,
            token_type: "Bearer".into(),
            scope: Some("mcp:read".into()),
            expires_at_unix_seconds: Some(now_unix().saturating_add(3600)),
        };
        let _ = delete_token(&first_name);
        let _ = delete_token(&second_name);
        save_token(&first_name, &token(&first_name, "first-secret")).unwrap();
        save_token(&second_name, &token(&second_name, "second-secret")).unwrap();

        assert_eq!(
            delete_matching_stored_credential(
                &first_name,
                "https://mcp.example.test/mcp",
                &config,
            )
            .unwrap(),
            McpOAuthCredentialCleanup::Removed
        );
        assert!(load_token(&first_name).unwrap().is_none());
        let second = load_token(&second_name)
            .unwrap()
            .expect("second credential");
        assert_eq!(second.server_name, second_name);
        assert!(second.access_token == "second-secret");

        let mut mismatched = token(&first_name, "mismatch-secret");
        mismatched.resource = "https://other.example.test/mcp".into();
        save_token(&first_name, &mismatched).unwrap();
        assert_eq!(
            delete_matching_stored_credential(
                &first_name,
                "https://mcp.example.test/mcp",
                &config,
            )
            .unwrap(),
            McpOAuthCredentialCleanup::BindingMismatch
        );
        assert!(load_token(&first_name).unwrap().unwrap().access_token == "mismatch-secret");

        assert!(delete_token(&first_name).unwrap());
        assert!(delete_token(&second_name).unwrap());
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
