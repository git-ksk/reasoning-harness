use std::{collections::BTreeSet, fs, path::PathBuf};

use clap::Subcommand;
use reasoning_harness_core::ResolutionAdapterErrorKind;
use reasoning_harness_providers::{
    MCP_REMOTE_PROTOCOL_VERSION, McpReadOnlyResolverV3, McpRemoteReadOnlyResolver,
};
use serde::Serialize;

use super::{
    CLI_CONFIG_CONTRACT_ID, CliError, CliFileConfig, LoadedCliConfig,
    McpReadOnlyResolverFileConfig, McpRemoteOAuthFileConfig, McpRemoteReadOnlyResolverFileConfig,
    OutputFormat, mcp_oauth, model_catalog, print_product_json, progress,
    resolve_mcp_readonly_config, resolve_mcp_remote_readonly_config, user_config_path,
};

const SURFACE_ID: &str = "reason-mcp-management-v2";

#[derive(Debug, Subcommand)]
pub(crate) enum McpCommand {
    /// Add one active local read-only MCP stdio source to user config.
    Add {
        name: String,
        #[arg(long, value_name = "PROGRAM")]
        program: String,
        #[arg(long = "arg", value_name = "ARG", allow_hyphen_values = true)]
        args: Vec<String>,
        #[arg(long, value_name = "TOOL")]
        tool: String,
        #[arg(long = "allow-tool", value_name = "TOOL")]
        allow_tools: Vec<String>,
        #[arg(long, value_name = "SOURCE")]
        source: Option<String>,
        #[arg(long, value_name = "MILLISECONDS")]
        timeout_ms: Option<u64>,
        #[arg(long, value_name = "BYTES")]
        max_response_bytes: Option<usize>,
        #[arg(long)]
        replace: bool,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Add one active remote stateless read-only MCP source with OAuth client metadata.
    AddRemote {
        name: String,
        #[arg(long, value_name = "HTTPS_URL")]
        endpoint: String,
        #[arg(long, value_name = "TOOL")]
        tool: String,
        #[arg(long = "allow-tool", value_name = "TOOL")]
        allow_tools: Vec<String>,
        #[arg(long, value_name = "SOURCE")]
        source: Option<String>,
        #[arg(long, value_name = "URL")]
        issuer: String,
        #[arg(long, value_name = "URL")]
        authorization_endpoint: String,
        #[arg(long, value_name = "URL")]
        token_endpoint: String,
        #[arg(long, value_name = "CLIENT_ID")]
        client_id: String,
        #[arg(long = "scope", value_name = "SCOPE")]
        scopes: Vec<String>,
        #[arg(long, value_name = "MILLISECONDS")]
        timeout_ms: Option<u64>,
        #[arg(long, value_name = "BYTES")]
        max_response_bytes: Option<usize>,
        #[arg(long)]
        replace: bool,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// List the active user-configured MCP acquisition source.
    List {
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Inspect one MCP source without exposing executable arguments or OAuth tokens.
    Inspect {
        name: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Verify read-only MCP readiness without invoking the selected tool.
    Test {
        name: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Start OAuth authorization-code + PKCE login for a remote MCP source.
    Login {
        name: String,
        /// Print the authorization URL without attempting to open a browser.
        #[arg(long)]
        no_browser: bool,
        #[arg(long)]
        replace: bool,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Show remote MCP OAuth credential status without exposing token values.
    Status {
        name: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Delete the remote MCP OAuth credential from the native OS store.
    Logout {
        name: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Remove one MCP source from user config. OAuth credentials are retained until logout.
    Remove {
        name: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

impl McpCommand {
    pub(crate) const fn format(&self) -> OutputFormat {
        match self {
            Self::Add { format, .. }
            | Self::AddRemote { format, .. }
            | Self::List { format }
            | Self::Inspect { format, .. }
            | Self::Test { format, .. }
            | Self::Login { format, .. }
            | Self::Status { format, .. }
            | Self::Logout { format, .. }
            | Self::Remove { format, .. } => *format,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct McpSummary {
    name: String,
    transport: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    program: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    endpoint: Option<String>,
    argument_count: usize,
    selected_tool: String,
    allowed_tools: Vec<String>,
    source: String,
    read_only: bool,
    resolver_class: &'static str,
    protocol_version: String,
    max_tool_list_pages: usize,
    timeout_ms: u64,
    max_response_bytes: usize,
    fixed_argument_keys: Vec<String>,
    oauth_configured: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oauth_client_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ListOutput {
    management_surface: &'static str,
    config_contract: &'static str,
    config_path: String,
    sources: Vec<McpSummary>,
}

#[derive(Debug, Serialize)]
struct InspectOutput {
    management_surface: &'static str,
    config_contract: &'static str,
    config_path: String,
    source: McpSummary,
}

#[derive(Debug, Serialize)]
struct MutationOutput {
    management_surface: &'static str,
    config_contract: &'static str,
    operation: &'static str,
    config_path: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<McpSummary>,
}

#[derive(Debug, Serialize)]
struct TestOutput {
    management_surface: &'static str,
    status: &'static str,
    name: String,
    transport: &'static str,
    selected_tool: String,
    negotiated_protocol_version: String,
    read_only_hint: bool,
}

struct UserMcpConfig {
    path: PathBuf,
    root: serde_json::Value,
    local: Option<McpReadOnlyResolverFileConfig>,
    remote: Option<McpRemoteReadOnlyResolverFileConfig>,
}

pub(crate) fn run(command: McpCommand) -> Result<(), CliError> {
    match command {
        McpCommand::Add {
            name,
            program,
            args,
            tool,
            allow_tools,
            source,
            timeout_ms,
            max_response_bytes,
            replace,
            format,
        } => add_local(
            name,
            program,
            args,
            tool,
            allow_tools,
            source,
            timeout_ms,
            max_response_bytes,
            replace,
            format,
        ),
        McpCommand::AddRemote {
            name,
            endpoint,
            tool,
            allow_tools,
            source,
            issuer,
            authorization_endpoint,
            token_endpoint,
            client_id,
            scopes,
            timeout_ms,
            max_response_bytes,
            replace,
            format,
        } => add_remote(
            name,
            endpoint,
            tool,
            allow_tools,
            source,
            issuer,
            authorization_endpoint,
            token_endpoint,
            client_id,
            scopes,
            timeout_ms,
            max_response_bytes,
            replace,
            format,
        ),
        McpCommand::List { format } => list(format),
        McpCommand::Inspect { name, format } => inspect(&name, format),
        McpCommand::Test { name, format } => test(&name, format),
        McpCommand::Login {
            name,
            no_browser,
            replace,
            format,
        } => login(&name, no_browser, replace, format),
        McpCommand::Status { name, format } => oauth_status(&name, format),
        McpCommand::Logout { name, format } => logout(&name, format),
        McpCommand::Remove { name, format } => remove(&name, format),
    }
}

#[allow(clippy::too_many_arguments)]
fn add_local(
    name: String,
    program: String,
    args: Vec<String>,
    tool: String,
    allow_tools: Vec<String>,
    source: Option<String>,
    timeout_ms: Option<u64>,
    max_response_bytes: Option<usize>,
    replace: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    validate_name(&name)?;
    let program = required("program", program)?;
    reject_secret_like_args(&args)?;
    let tool = required("tool", tool)?;
    validate_limits(timeout_ms, max_response_bytes)?;
    let allowed_tools = allowlist(&tool, allow_tools)?;
    let source = source
        .map(|value| required("source", value))
        .transpose()?
        .unwrap_or_else(|| format!("mcp:{name}:{tool}"));
    let configured = McpReadOnlyResolverFileConfig {
        server_id: name.clone(),
        program,
        args,
        allowed_tools,
        tool,
        read_only: true,
        resolver_class: "evidence_acquisition".into(),
        fixed_arguments: Default::default(),
        provenance_argument: None,
        source,
        requested_protocol_version: None,
        supported_protocol_versions: Default::default(),
        max_tool_list_pages: None,
        timeout_ms,
        max_response_bytes,
        admission: None,
    };
    let summary = local_summary(&configured)?;
    let mut user = user_json()?;
    ensure_replace_policy(&user, replace)?;
    set_local_json(&mut user.root, Some(&configured))?;
    set_remote_json(&mut user.root, None)?;
    validate_root(&user.root)?;
    model_catalog::write_user_config_value(&user.path, &user.root)?;
    emit_mutation("add", user.path, name, Some(summary), format)
}

#[allow(clippy::too_many_arguments)]
fn add_remote(
    name: String,
    endpoint: String,
    tool: String,
    allow_tools: Vec<String>,
    source: Option<String>,
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
    client_id: String,
    scopes: Vec<String>,
    timeout_ms: Option<u64>,
    max_response_bytes: Option<usize>,
    replace: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    validate_name(&name)?;
    let endpoint = required("endpoint", endpoint)?;
    let tool = required("tool", tool)?;
    validate_limits(timeout_ms, max_response_bytes)?;
    let allowed_tools = allowlist(&tool, allow_tools)?;
    let source = source
        .map(|value| required("source", value))
        .transpose()?
        .unwrap_or_else(|| format!("mcp:{name}:{tool}"));
    let oauth = McpRemoteOAuthFileConfig {
        issuer: required("issuer", issuer)?,
        authorization_endpoint: required("authorization-endpoint", authorization_endpoint)?,
        token_endpoint: required("token-endpoint", token_endpoint)?,
        client_id: required("client-id", client_id)?,
        scopes: scopes
            .into_iter()
            .map(|scope| required("scope", scope))
            .collect::<Result<_, _>>()?,
    };
    mcp_oauth::validate_oauth_config(&oauth)?;
    let configured = McpRemoteReadOnlyResolverFileConfig {
        server_id: name.clone(),
        endpoint,
        allowed_tools,
        tool,
        read_only: true,
        resolver_class: "evidence_acquisition".into(),
        fixed_arguments: Default::default(),
        provenance_argument: None,
        source,
        protocol_version: Some(MCP_REMOTE_PROTOCOL_VERSION.into()),
        max_tool_list_pages: None,
        timeout_ms,
        max_response_bytes,
        oauth: Some(oauth),
        admission: None,
    };
    let summary = remote_summary(&configured)?;
    let mut user = user_json()?;
    ensure_replace_policy(&user, replace)?;
    set_local_json(&mut user.root, None)?;
    set_remote_json(&mut user.root, Some(&configured))?;
    validate_root(&user.root)?;
    model_catalog::write_user_config_value(&user.path, &user.root)?;
    emit_mutation("add_remote", user.path, name, Some(summary), format)
}

fn list(format: OutputFormat) -> Result<(), CliError> {
    let user = user_json()?;
    let mut sources = Vec::new();
    if let Some(local) = user.local.as_ref() {
        sources.push(local_summary(local)?);
    }
    if let Some(remote) = user.remote.as_ref() {
        sources.push(remote_summary(remote)?);
    }
    match format {
        OutputFormat::Json => print_product_json(
            "mcp",
            &ListOutput {
                management_surface: SURFACE_ID,
                config_contract: CLI_CONFIG_CONTRACT_ID,
                config_path: user.path.display().to_string(),
                sources,
            },
        )
        .map_err(CliError::from),
        OutputFormat::Human => {
            if sources.is_empty() {
                println!("No user MCP acquisition source configured.");
            }
            for source in sources {
                let target = source
                    .endpoint
                    .as_deref()
                    .or(source.program.as_deref())
                    .unwrap_or("-");
                println!(
                    "{}  transport={}  target={}  tool={}  read_only={}",
                    source.name, source.transport, target, source.selected_tool, source.read_only
                );
            }
            Ok(())
        }
    }
}

fn inspect(name: &str, format: OutputFormat) -> Result<(), CliError> {
    validate_name(name)?;
    let user = user_json()?;
    let source = named_summary(&user, name)?;
    match format {
        OutputFormat::Json => print_product_json(
            "mcp",
            &InspectOutput {
                management_surface: SURFACE_ID,
                config_contract: CLI_CONFIG_CONTRACT_ID,
                config_path: user.path.display().to_string(),
                source,
            },
        )
        .map_err(CliError::from),
        OutputFormat::Human => {
            println!("name: {}", source.name);
            println!("transport: {}", source.transport);
            if let Some(program) = &source.program {
                println!("program: {program}");
                println!("argument_count: {}", source.argument_count);
            }
            if let Some(endpoint) = &source.endpoint {
                println!("endpoint: {endpoint}");
            }
            println!("selected_tool: {}", source.selected_tool);
            println!("allowed_tools: {}", source.allowed_tools.join(", "));
            println!("source: {}", source.source);
            println!("protocol_version: {}", source.protocol_version);
            println!("read_only: true");
            if source.oauth_configured {
                println!(
                    "oauth: configured (credential value stored separately in native OS store)"
                );
                println!(
                    "oauth_issuer: {}",
                    source.oauth_issuer.as_deref().unwrap_or("-")
                );
                println!(
                    "oauth_client_id: {}",
                    source.oauth_client_id.as_deref().unwrap_or("-")
                );
            }
            Ok(())
        }
    }
}

fn test(name: &str, format: OutputFormat) -> Result<(), CliError> {
    validate_name(name)?;
    let user = user_json()?;
    let cancellation = progress::CancellationRun::begin()?;
    let cancellation_token = cancellation.subprocess_token();
    let output = (|| -> Result<TestOutput, CliError> {
        if let Some(local) = user.local.as_ref().filter(|value| value.server_id == name) {
            let runtime = resolve_local_runtime(local)?;
            let readiness = McpReadOnlyResolverV3::new(runtime)
                .with_cancellation(cancellation_token.clone())
                .probe_readiness()
                .map_err(readiness_error)?;
            Ok(TestOutput {
                management_surface: SURFACE_ID,
                status: "ready",
                name: readiness.server_id,
                transport: "stdio",
                selected_tool: readiness.selected_tool,
                negotiated_protocol_version: readiness.negotiated_protocol_version,
                read_only_hint: readiness.read_only_hint,
            })
        } else if let Some(remote) = user.remote.as_ref().filter(|value| value.server_id == name) {
            let resolved = resolve_remote_runtime(remote)?;
            let token = resolved
                .oauth
                .as_ref()
                .map(|oauth| mcp_oauth::access_token(name, &resolved.resolver.endpoint, oauth))
                .transpose()?;
            let readiness = McpRemoteReadOnlyResolver::new(resolved.resolver, token)
                .map_err(|kind| {
                    CliError::new(
                        "mcp_configuration",
                        format!("remote MCP configuration rejected: {kind:?}"),
                    )
                })?
                .with_cancellation(cancellation_token.clone())
                .probe_readiness()
                .map_err(readiness_error)?;
            Ok(TestOutput {
                management_surface: SURFACE_ID,
                status: "ready",
                name: readiness.server_id,
                transport: "streamable_http",
                selected_tool: readiness.selected_tool,
                negotiated_protocol_version: readiness.protocol_version,
                read_only_hint: readiness.read_only_hint,
            })
        } else {
            Err(not_found(name, &user))
        }
    })();
    if cancellation.is_cancelled() {
        return Err(cancellation.error());
    }
    let output = output?;
    match format {
        OutputFormat::Json => print_product_json("mcp", &output).map_err(CliError::from),
        OutputFormat::Human => {
            println!(
                "ready: name={} transport={} protocol={} tool={} read_only_hint=true",
                output.name,
                output.transport,
                output.negotiated_protocol_version,
                output.selected_tool
            );
            Ok(())
        }
    }
}

fn login(
    name: &str,
    no_browser: bool,
    replace: bool,
    format: OutputFormat,
) -> Result<(), CliError> {
    validate_name(name)?;
    let user = user_json()?;
    let remote = require_remote(&user, name)?;
    let oauth = remote.oauth.as_ref().ok_or_else(|| {
        CliError::new(
            "mcp_oauth_configuration",
            "remote MCP source has no OAuth configuration",
        )
    })?;
    mcp_oauth::login(name, &remote.endpoint, oauth, no_browser, replace, format)
}

fn oauth_status(name: &str, format: OutputFormat) -> Result<(), CliError> {
    validate_name(name)?;
    let user = user_json()?;
    let remote = require_remote(&user, name)?;
    let oauth = remote.oauth.as_ref().ok_or_else(|| {
        CliError::new(
            "mcp_oauth_configuration",
            "remote MCP source has no OAuth configuration",
        )
    })?;
    let status = mcp_oauth::status(name, &remote.endpoint, oauth)?;
    match format {
        OutputFormat::Json => print_product_json("mcp", &status).map_err(CliError::from),
        OutputFormat::Human => {
            println!(
                "{}: stored={} usable={} issuer={} client_id={} refresh_available={}",
                status.server_name,
                status.stored,
                status.usable,
                status.issuer,
                status.client_id,
                status.refresh_available
            );
            Ok(())
        }
    }
}

fn logout(name: &str, format: OutputFormat) -> Result<(), CliError> {
    validate_name(name)?;
    mcp_oauth::logout(name, format)
}

fn remove(name: &str, format: OutputFormat) -> Result<(), CliError> {
    validate_name(name)?;
    let mut user = user_json()?;
    let removed = if user
        .local
        .as_ref()
        .is_some_and(|value| value.server_id == name)
    {
        set_local_json(&mut user.root, None)?;
        name.to_string()
    } else if user
        .remote
        .as_ref()
        .is_some_and(|value| value.server_id == name)
    {
        set_remote_json(&mut user.root, None)?;
        name.to_string()
    } else {
        return Err(not_found(name, &user));
    };
    validate_root(&user.root)?;
    model_catalog::write_user_config_value(&user.path, &user.root)?;
    emit_mutation("remove", user.path, removed, None, format)
}

fn emit_mutation(
    operation: &'static str,
    path: PathBuf,
    name: String,
    source: Option<McpSummary>,
    format: OutputFormat,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Json => print_product_json(
            "mcp",
            &MutationOutput {
                management_surface: SURFACE_ID,
                config_contract: CLI_CONFIG_CONTRACT_ID,
                operation,
                config_path: path.display().to_string(),
                name,
                source,
            },
        )
        .map_err(CliError::from),
        OutputFormat::Human => {
            println!("{operation}: {name}");
            println!("config: {}", path.display());
            Ok(())
        }
    }
}

fn user_json() -> Result<UserMcpConfig, CliError> {
    let path = user_config_path().ok_or_else(|| {
        CliError::new(
            "configuration",
            "cannot determine user config path; set REASON_HOME, XDG_CONFIG_HOME, APPDATA, or HOME",
        )
    })?;
    let root = if path.is_file() {
        let bytes = fs::read(&path).map_err(|error| {
            CliError::new("configuration", format!("{}: {error}", path.display()))
        })?;
        serde_json::from_slice(&bytes).map_err(|error| {
            CliError::new("configuration", format!("{}: {error}", path.display()))
        })?
    } else {
        serde_json::json!({"schema_version": CLI_CONFIG_CONTRACT_ID, "run": {}, "resolution": {}})
    };
    let parsed: CliFileConfig = serde_json::from_value(root.clone())
        .map_err(|error| CliError::new("configuration", format!("{}: {error}", path.display())))?;
    if parsed.schema_version != CLI_CONFIG_CONTRACT_ID {
        return Err(CliError::new(
            "configuration",
            format!("{}: unsupported schema_version", path.display()),
        ));
    }
    Ok(UserMcpConfig {
        path,
        root,
        local: parsed.resolution.mcp_readonly,
        remote: parsed.resolution.mcp_remote_readonly,
    })
}

fn ensure_replace_policy(user: &UserMcpConfig, replace: bool) -> Result<(), CliError> {
    if !replace {
        if let Some(name) = user
            .local
            .as_ref()
            .map(|value| &value.server_id)
            .or_else(|| user.remote.as_ref().map(|value| &value.server_id))
        {
            return Err(CliError::new(
                "mcp_configuration",
                format!(
                    "user config already contains MCP source {name:?}; use --replace to replace it explicitly"
                ),
            ));
        }
    }
    Ok(())
}

fn resolution_object(
    root: &mut serde_json::Value,
) -> Result<&mut serde_json::Map<String, serde_json::Value>, CliError> {
    root.as_object_mut()
        .ok_or_else(|| CliError::new("configuration", "user config root must be a JSON object"))?
        .entry("resolution")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| CliError::new("configuration", "user config resolution must be an object"))
}

fn set_local_json(
    root: &mut serde_json::Value,
    configured: Option<&McpReadOnlyResolverFileConfig>,
) -> Result<(), CliError> {
    let resolution = resolution_object(root)?;
    if let Some(value) = configured {
        resolution.insert(
            "mcp_readonly".into(),
            serde_json::to_value(value)
                .map_err(|error| CliError::new("configuration", error.to_string()))?,
        );
    } else {
        resolution.remove("mcp_readonly");
    }
    Ok(())
}

fn set_remote_json(
    root: &mut serde_json::Value,
    configured: Option<&McpRemoteReadOnlyResolverFileConfig>,
) -> Result<(), CliError> {
    let resolution = resolution_object(root)?;
    if let Some(value) = configured {
        resolution.insert(
            "mcp_remote_readonly".into(),
            serde_json::to_value(value)
                .map_err(|error| CliError::new("configuration", error.to_string()))?,
        );
    } else {
        resolution.remove("mcp_remote_readonly");
    }
    Ok(())
}

fn validate_root(root: &serde_json::Value) -> Result<(), CliError> {
    let parsed: CliFileConfig = serde_json::from_value(root.clone()).map_err(|error| {
        CliError::new(
            "mcp_configuration",
            format!("resulting user config is invalid: {error}"),
        )
    })?;
    if parsed.schema_version != CLI_CONFIG_CONTRACT_ID {
        return Err(CliError::new(
            "mcp_configuration",
            "resulting user config has an unsupported schema_version",
        ));
    }
    if parsed.resolution.mcp_readonly.is_some() && parsed.resolution.mcp_remote_readonly.is_some() {
        return Err(CliError::new(
            "mcp_configuration",
            "local and remote MCP acquisition sources are mutually exclusive",
        ));
    }
    if let Some(local) = parsed.resolution.mcp_readonly.as_ref() {
        resolve_local_runtime(local)?;
    }
    if let Some(remote) = parsed.resolution.mcp_remote_readonly.as_ref() {
        resolve_remote_runtime(remote)?;
    }
    Ok(())
}

fn named_summary(user: &UserMcpConfig, name: &str) -> Result<McpSummary, CliError> {
    if let Some(local) = user.local.as_ref().filter(|value| value.server_id == name) {
        local_summary(local)
    } else if let Some(remote) = user.remote.as_ref().filter(|value| value.server_id == name) {
        remote_summary(remote)
    } else {
        Err(not_found(name, user))
    }
}

fn require_remote<'a>(
    user: &'a UserMcpConfig,
    name: &str,
) -> Result<&'a McpRemoteReadOnlyResolverFileConfig, CliError> {
    user.remote
        .as_ref()
        .filter(|value| value.server_id == name)
        .ok_or_else(|| not_found(name, user))
}

fn not_found(name: &str, user: &UserMcpConfig) -> CliError {
    let current = user
        .local
        .as_ref()
        .map(|v| v.server_id.as_str())
        .or_else(|| user.remote.as_ref().map(|v| v.server_id.as_str()));
    match current {
        Some(current) => CliError::new(
            "mcp_not_found",
            format!("MCP source {name:?} is not configured; current user source is {current:?}"),
        ),
        None => CliError::new(
            "mcp_not_found",
            "no user MCP acquisition source is configured; use `reason mcp add` or `reason mcp add-remote` first",
        ),
    }
}

fn resolve_local_runtime(
    configured: &McpReadOnlyResolverFileConfig,
) -> Result<reasoning_harness_providers::McpReadOnlyResolverV3Config, CliError> {
    let mut config = CliFileConfig::default();
    config.resolution.mcp_readonly = Some(configured.clone());
    resolve_mcp_readonly_config(&LoadedCliConfig {
        config,
        sources: vec!["user"],
    })
    .map_err(|error| CliError::new("mcp_configuration", error))?
    .ok_or_else(|| CliError::new("mcp_configuration", "MCP source unexpectedly missing"))
}

fn resolve_remote_runtime(
    configured: &McpRemoteReadOnlyResolverFileConfig,
) -> Result<super::ResolvedMcpRemoteConfig, CliError> {
    let mut config = CliFileConfig::default();
    config.resolution.mcp_remote_readonly = Some(configured.clone());
    resolve_mcp_remote_readonly_config(&LoadedCliConfig {
        config,
        sources: vec!["user"],
    })
    .map_err(|error| CliError::new("mcp_configuration", error))?
    .ok_or_else(|| {
        CliError::new(
            "mcp_configuration",
            "remote MCP source unexpectedly missing",
        )
    })
}

fn local_summary(configured: &McpReadOnlyResolverFileConfig) -> Result<McpSummary, CliError> {
    let runtime = resolve_local_runtime(configured)?;
    Ok(McpSummary {
        name: runtime.base.server_id,
        transport: "stdio",
        program: Some(runtime.base.program.display().to_string()),
        endpoint: None,
        argument_count: runtime.base.args.len(),
        selected_tool: runtime.base.tool,
        allowed_tools: runtime.base.allowed_tools.into_iter().collect(),
        source: runtime.base.source,
        read_only: true,
        resolver_class: "evidence_acquisition",
        protocol_version: runtime.requested_protocol_version,
        max_tool_list_pages: runtime.max_tool_list_pages,
        timeout_ms: runtime.base.timeout_ms,
        max_response_bytes: runtime.base.max_response_bytes,
        fixed_argument_keys: configured.fixed_arguments.keys().cloned().collect(),
        oauth_configured: false,
        oauth_issuer: None,
        oauth_client_id: None,
    })
}

fn remote_summary(
    configured: &McpRemoteReadOnlyResolverFileConfig,
) -> Result<McpSummary, CliError> {
    let runtime = resolve_remote_runtime(configured)?;
    Ok(McpSummary {
        name: runtime.resolver.server_id,
        transport: "streamable_http",
        program: None,
        endpoint: Some(runtime.resolver.endpoint),
        argument_count: 0,
        selected_tool: runtime.resolver.tool,
        allowed_tools: runtime.resolver.allowed_tools.into_iter().collect(),
        source: runtime.resolver.source,
        read_only: true,
        resolver_class: "evidence_acquisition",
        protocol_version: runtime.resolver.protocol_version,
        max_tool_list_pages: runtime.resolver.max_tool_list_pages,
        timeout_ms: runtime.resolver.timeout_ms,
        max_response_bytes: runtime.resolver.max_response_bytes,
        fixed_argument_keys: configured.fixed_arguments.keys().cloned().collect(),
        oauth_configured: runtime.oauth.is_some(),
        oauth_issuer: runtime.oauth.as_ref().map(|o| o.issuer.clone()),
        oauth_client_id: runtime.oauth.as_ref().map(|o| o.client_id.clone()),
    })
}

fn allowlist(tool: &str, extras: Vec<String>) -> Result<BTreeSet<String>, CliError> {
    let mut values = BTreeSet::from([tool.to_string()]);
    for value in extras {
        values.insert(required("allow-tool", value)?);
    }
    Ok(values)
}

fn validate_limits(
    timeout_ms: Option<u64>,
    max_response_bytes: Option<usize>,
) -> Result<(), CliError> {
    if timeout_ms == Some(0) || max_response_bytes == Some(0) {
        Err(CliError::new(
            "mcp_configuration",
            "--timeout-ms and --max-response-bytes must be at least 1",
        ))
    } else {
        Ok(())
    }
}

fn validate_name(name: &str) -> Result<(), CliError> {
    let name = name.trim();
    if name.is_empty()
        || name.len() > 64
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(CliError::new(
            "mcp_configuration",
            "MCP source name must be 1-64 ASCII letters, digits, '.', '_' or '-'",
        ));
    }
    Ok(())
}

fn reject_secret_like_args(args: &[String]) -> Result<(), CliError> {
    const SECRET_MARKERS: &[&str] = &[
        "api-key",
        "api_key",
        "token",
        "secret",
        "password",
        "credential",
        "authorization",
    ];
    if args.iter().any(|arg| {
        let lower = arg.to_ascii_lowercase();
        SECRET_MARKERS.iter().any(|marker| lower.contains(marker))
    }) {
        return Err(CliError::new(
            "mcp_secret_input",
            "MCP executable arguments must be non-secret; credential/token arguments are not persisted by `reason mcp add`",
        ));
    }
    Ok(())
}

fn required(label: &str, value: String) -> Result<String, CliError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(CliError::new(
            "mcp_configuration",
            format!("--{label} must be non-empty"),
        ))
    } else {
        Ok(trimmed.to_string())
    }
}

fn readiness_error(error: reasoning_harness_core::ResolutionAdapterError) -> CliError {
    let (failure_class, message) = match error.kind {
        ResolutionAdapterErrorKind::PolicyDenied => (
            "mcp_policy",
            "MCP readiness rejected: selected tool is missing, not allowlisted, or lacks server readOnlyHint=true",
        ),
        ResolutionAdapterErrorKind::Negotiation => (
            "mcp_negotiation",
            "MCP readiness failed during protocol negotiation",
        ),
        ResolutionAdapterErrorKind::Authentication => (
            "mcp_authentication",
            "MCP readiness requires authentication; run `reason mcp login <name>` for remote OAuth sources",
        ),
        ResolutionAdapterErrorKind::PermissionDenied => (
            "mcp_permission",
            "MCP readiness was denied by server or local process permissions",
        ),
        ResolutionAdapterErrorKind::Timeout => ("mcp_timeout", "MCP readiness timed out"),
        ResolutionAdapterErrorKind::Unavailable => (
            "mcp_unavailable",
            "MCP readiness could not reach or start the configured server",
        ),
        ResolutionAdapterErrorKind::Transport => (
            "mcp_transport",
            "MCP readiness failed while communicating with the server",
        ),
        ResolutionAdapterErrorKind::Session => (
            "mcp_session",
            "MCP readiness failed during bounded tool discovery",
        ),
        ResolutionAdapterErrorKind::Protocol | ResolutionAdapterErrorKind::MalformedOutput => (
            "mcp_protocol",
            "MCP readiness received an invalid protocol response",
        ),
        ResolutionAdapterErrorKind::ToolExecution | ResolutionAdapterErrorKind::Failed => (
            "mcp_operational",
            "MCP readiness failed operationally before the source could be verified",
        ),
    };
    CliError::new(failure_class, message)
}
