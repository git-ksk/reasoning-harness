use std::{collections::BTreeSet, fs, path::PathBuf};

use clap::Subcommand;
use reasoning_harness_core::ResolutionAdapterErrorKind;
use reasoning_harness_providers::McpReadOnlyResolverV3;
use serde::Serialize;

use super::{
    CLI_CONFIG_CONTRACT_ID, CliError, CliFileConfig, LoadedCliConfig,
    McpReadOnlyResolverFileConfig, OutputFormat, model_catalog, print_product_json,
    resolve_mcp_readonly_config, user_config_path,
};

const SURFACE_ID: &str = "reason-mcp-management-v1";

#[derive(Debug, Subcommand)]
pub(crate) enum McpCommand {
    /// Add one active local read-only MCP acquisition source to the user config.
    Add {
        /// Local source name. Stored as the MCP server_id.
        name: String,
        /// Local MCP stdio executable.
        #[arg(long, value_name = "PROGRAM")]
        program: String,
        /// Literal executable argument. Repeatable; no shell parsing is performed.
        #[arg(long = "arg", value_name = "ARG", allow_hyphen_values = true)]
        args: Vec<String>,
        /// Selected read-only MCP tool.
        #[arg(long, value_name = "TOOL")]
        tool: String,
        /// Additional explicitly allowlisted tool name. Repeatable. The selected tool is always included.
        #[arg(long = "allow-tool", value_name = "TOOL")]
        allow_tools: Vec<String>,
        /// Harness-owned provenance source label. Defaults to mcp:<name>:<tool>.
        #[arg(long, value_name = "SOURCE")]
        source: Option<String>,
        /// Whole-session timeout in milliseconds.
        #[arg(long, value_name = "MILLISECONDS")]
        timeout_ms: Option<u64>,
        /// Maximum bytes accepted for one MCP response line.
        #[arg(long, value_name = "BYTES")]
        max_response_bytes: Option<usize>,
        /// Explicitly replace the currently configured user MCP source.
        #[arg(long)]
        replace: bool,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// List the user-configured local read-only MCP acquisition source.
    List {
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Inspect one user-configured MCP source without exposing argument values or secrets.
    Inspect {
        name: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Verify MCP negotiation and server read-only proof without invoking the selected tool.
    Test {
        name: String,
        #[arg(long, value_enum, default_value_t)]
        format: OutputFormat,
    },
    /// Remove one user-configured MCP source.
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
            | Self::List { format }
            | Self::Inspect { format, .. }
            | Self::Test { format, .. }
            | Self::Remove { format, .. } => *format,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct McpSummary {
    name: String,
    transport: &'static str,
    program: String,
    argument_count: usize,
    selected_tool: String,
    allowed_tools: Vec<String>,
    source: String,
    read_only: bool,
    resolver_class: &'static str,
    requested_protocol_version: String,
    supported_protocol_versions: Vec<String>,
    max_tool_list_pages: usize,
    timeout_ms: u64,
    max_response_bytes: usize,
    fixed_argument_keys: Vec<String>,
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
    program: String,
    selected_tool: String,
    negotiated_protocol_version: String,
    read_only_hint: bool,
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
        } => add(
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
        McpCommand::List { format } => list(format),
        McpCommand::Inspect { name, format } => inspect(&name, format),
        McpCommand::Test { name, format } => test(&name, format),
        McpCommand::Remove { name, format } => remove(&name, format),
    }
}

#[allow(clippy::too_many_arguments)]
fn add(
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
    if timeout_ms == Some(0) || max_response_bytes == Some(0) {
        return Err(CliError::new(
            "mcp_configuration",
            "--timeout-ms and --max-response-bytes must be at least 1",
        ));
    }
    let mut allowed_tools = BTreeSet::from([tool.clone()]);
    for allowed in allow_tools {
        allowed_tools.insert(required("allow-tool", allowed)?);
    }
    let source = match source {
        Some(source) => required("source", source)?,
        None => format!("mcp:{name}:{tool}"),
    };
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
    // Validate the exact runtime policy before touching disk.
    let summary = resolved_summary(&configured)?;
    let (path, mut root, existing) = user_json()?;
    if let Some(existing) = existing
        && !replace
    {
        return Err(CliError::new(
            "mcp_configuration",
            format!(
                "user config already contains MCP source {:?}; use --replace to replace it explicitly",
                existing.server_id
            ),
        ));
    }
    set_mcp_json(&mut root, Some(&configured))?;
    validate_root(&root)?;
    model_catalog::write_user_config_value(&path, &root)?;
    emit_mutation("add", path, name, Some(summary), format)
}

fn list(format: OutputFormat) -> Result<(), CliError> {
    let (path, _root, configured) = user_json()?;
    let sources = configured
        .as_ref()
        .map(resolved_summary)
        .transpose()?
        .into_iter()
        .collect::<Vec<_>>();
    match format {
        OutputFormat::Json => print_product_json(
            "mcp",
            &ListOutput {
                management_surface: SURFACE_ID,
                config_contract: CLI_CONFIG_CONTRACT_ID,
                config_path: path.display().to_string(),
                sources,
            },
        )
        .map_err(|error| CliError::new("serialization", error)),
        OutputFormat::Human => {
            if sources.is_empty() {
                println!("No user MCP acquisition source configured.");
            } else {
                for source in sources {
                    println!(
                        "{}  transport={}  program={}  tool={}  read_only={}  allowed_tools={}",
                        source.name,
                        source.transport,
                        source.program,
                        source.selected_tool,
                        source.read_only,
                        source.allowed_tools.join(",")
                    );
                }
            }
            Ok(())
        }
    }
}

fn inspect(name: &str, format: OutputFormat) -> Result<(), CliError> {
    validate_name(name)?;
    let (path, _root, configured) = user_json()?;
    let configured = require_named(configured.as_ref(), name)?;
    let source = resolved_summary(configured)?;
    match format {
        OutputFormat::Json => print_product_json(
            "mcp",
            &InspectOutput {
                management_surface: SURFACE_ID,
                config_contract: CLI_CONFIG_CONTRACT_ID,
                config_path: path.display().to_string(),
                source,
            },
        )
        .map_err(|error| CliError::new("serialization", error)),
        OutputFormat::Human => {
            println!("name: {}", source.name);
            println!("transport: {}", source.transport);
            println!("program: {}", source.program);
            println!("argument_count: {}", source.argument_count);
            println!("selected_tool: {}", source.selected_tool);
            println!("allowed_tools: {}", source.allowed_tools.join(", "));
            println!("source: {}", source.source);
            println!("read_only: {}", source.read_only);
            println!("resolver_class: {}", source.resolver_class);
            println!(
                "requested_protocol_version: {}",
                source.requested_protocol_version
            );
            println!(
                "supported_protocol_versions: {}",
                source.supported_protocol_versions.join(", ")
            );
            println!("max_tool_list_pages: {}", source.max_tool_list_pages);
            println!("timeout_ms: {}", source.timeout_ms);
            println!("max_response_bytes: {}", source.max_response_bytes);
            if !source.fixed_argument_keys.is_empty() {
                println!(
                    "fixed_argument_keys: {} (values hidden)",
                    source.fixed_argument_keys.join(", ")
                );
            }
            Ok(())
        }
    }
}

fn test(name: &str, format: OutputFormat) -> Result<(), CliError> {
    validate_name(name)?;
    let (_path, _root, configured) = user_json()?;
    let configured = require_named(configured.as_ref(), name)?;
    let runtime = resolve_runtime(configured)?;
    let program = runtime.base.program.display().to_string();
    let resolver = McpReadOnlyResolverV3::new(runtime);
    let readiness = resolver.probe_readiness().map_err(readiness_error)?;
    let output = TestOutput {
        management_surface: SURFACE_ID,
        status: "ready",
        name: readiness.server_id,
        transport: "stdio",
        program,
        selected_tool: readiness.selected_tool,
        negotiated_protocol_version: readiness.negotiated_protocol_version,
        read_only_hint: readiness.read_only_hint,
    };
    match format {
        OutputFormat::Json => print_product_json("mcp", &output)
            .map_err(|error| CliError::new("serialization", error)),
        OutputFormat::Human => {
            println!(
                "ready: name={} protocol={} tool={} read_only_hint=true",
                output.name, output.negotiated_protocol_version, output.selected_tool
            );
            Ok(())
        }
    }
}

fn remove(name: &str, format: OutputFormat) -> Result<(), CliError> {
    validate_name(name)?;
    let (path, mut root, configured) = user_json()?;
    let configured = require_named(configured.as_ref(), name)?;
    let removed_name = configured.server_id.clone();
    set_mcp_json(&mut root, None)?;
    validate_root(&root)?;
    model_catalog::write_user_config_value(&path, &root)?;
    emit_mutation("remove", path, removed_name, None, format)
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
        .map_err(|error| CliError::new("serialization", error)),
        OutputFormat::Human => {
            println!("{operation}: {name}");
            println!("config: {}", path.display());
            if let Some(source) = source {
                println!(
                    "transport={} program={} tool={} read_only=true",
                    source.transport, source.program, source.selected_tool
                );
            }
            Ok(())
        }
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

fn user_json() -> Result<
    (
        PathBuf,
        serde_json::Value,
        Option<McpReadOnlyResolverFileConfig>,
    ),
    CliError,
> {
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
    Ok((path, root, parsed.resolution.mcp_readonly))
}

fn set_mcp_json(
    root: &mut serde_json::Value,
    configured: Option<&McpReadOnlyResolverFileConfig>,
) -> Result<(), CliError> {
    let object = root
        .as_object_mut()
        .ok_or_else(|| CliError::new("configuration", "user config root must be a JSON object"))?;
    let resolution = object
        .entry("resolution")
        .or_insert_with(|| serde_json::json!({}))
        .as_object_mut()
        .ok_or_else(|| {
            CliError::new("configuration", "user config resolution must be an object")
        })?;
    match configured {
        Some(configured) => {
            resolution.insert(
                "mcp_readonly".into(),
                serde_json::to_value(configured).map_err(|error| {
                    CliError::new("configuration", format!("serialize MCP config: {error}"))
                })?,
            );
        }
        None => {
            resolution.remove("mcp_readonly");
        }
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
    if let Some(configured) = parsed.resolution.mcp_readonly.as_ref() {
        resolve_runtime(configured)?;
    }
    Ok(())
}

fn require_named<'a>(
    configured: Option<&'a McpReadOnlyResolverFileConfig>,
    name: &str,
) -> Result<&'a McpReadOnlyResolverFileConfig, CliError> {
    let configured = configured.ok_or_else(|| {
        CliError::new(
            "mcp_not_found",
            "no user MCP acquisition source is configured; use `reason mcp add` first",
        )
    })?;
    if configured.server_id != name {
        return Err(CliError::new(
            "mcp_not_found",
            format!(
                "MCP source {name:?} is not configured; current user source is {:?}",
                configured.server_id
            ),
        ));
    }
    Ok(configured)
}

fn resolve_runtime(
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

fn resolved_summary(configured: &McpReadOnlyResolverFileConfig) -> Result<McpSummary, CliError> {
    let runtime = resolve_runtime(configured)?;
    Ok(McpSummary {
        name: runtime.base.server_id,
        transport: "stdio",
        program: runtime.base.program.display().to_string(),
        argument_count: runtime.base.args.len(),
        selected_tool: runtime.base.tool,
        allowed_tools: runtime.base.allowed_tools.into_iter().collect(),
        source: runtime.base.source,
        read_only: true,
        resolver_class: "evidence_acquisition",
        requested_protocol_version: runtime.requested_protocol_version,
        supported_protocol_versions: runtime.supported_protocol_versions.into_iter().collect(),
        max_tool_list_pages: runtime.max_tool_list_pages,
        timeout_ms: runtime.base.timeout_ms,
        max_response_bytes: runtime.base.max_response_bytes,
        fixed_argument_keys: configured.fixed_arguments.keys().cloned().collect(),
    })
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
            "MCP readiness requires authentication; remote/OAuth lifecycle is handled separately",
        ),
        ResolutionAdapterErrorKind::PermissionDenied => (
            "mcp_permission",
            "MCP readiness was denied by the server or local process permissions",
        ),
        ResolutionAdapterErrorKind::Timeout => ("mcp_timeout", "MCP readiness timed out"),
        ResolutionAdapterErrorKind::Unavailable => (
            "mcp_unavailable",
            "MCP readiness could not reach or start the configured server",
        ),
        ResolutionAdapterErrorKind::Transport => (
            "mcp_transport",
            "MCP readiness failed while starting or communicating with the server",
        ),
        ResolutionAdapterErrorKind::Session => (
            "mcp_session",
            "MCP readiness failed during the MCP session or bounded tool discovery",
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
