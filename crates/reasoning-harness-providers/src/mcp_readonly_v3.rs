use std::{
    collections::{BTreeMap, BTreeSet},
    process::Command,
    time::{Duration, Instant},
};

use reasoning_harness_core::{
    AcquiredEvidence, AcquiredEvidenceMetadata, ResolutionAdapterError, ResolutionAdapterErrorKind,
    ResolutionCost, ResolutionRequest, ResolutionResolver, ResolutionResolverContribution,
    ResolutionResolverOutput, ResolverClass,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    SubprocessCancellation,
    config_identity::stable_config_id,
    mcp_readonly::{MCP_PROTOCOL_VERSION, McpReadOnlyResolverConfig},
    subprocess_deadline::DeadlineLineSession,
    subprocess_environment::isolate_subprocess_environment,
};

pub const MCP_READONLY_V3_RESOLVER_ID: &str = "mcp_readonly_v3";
pub const MCP_READONLY_V3_DOWNLEVEL_PROTOCOL_VERSION: &str = "2025-11-25";
pub const DEFAULT_MCP_READONLY_V3_MAX_TOOL_LIST_PAGES: usize = 8;

const MCP_CLIENT_NAME: &str = "reasoning-harness";
const MCP_PROVENANCE_META_KEY: &str = "git-ksk/reasoning-harness/provenance";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct McpReadOnlyResolverV3Config {
    pub base: McpReadOnlyResolverConfig,
    pub requested_protocol_version: String,
    pub supported_protocol_versions: BTreeSet<String>,
    pub require_read_only_hint: bool,
    pub max_tool_list_pages: usize,
}

impl McpReadOnlyResolverV3Config {
    pub fn with_defaults(base: McpReadOnlyResolverConfig) -> Self {
        Self {
            base,
            requested_protocol_version: MCP_PROTOCOL_VERSION.into(),
            supported_protocol_versions: BTreeSet::from([
                MCP_PROTOCOL_VERSION.into(),
                MCP_READONLY_V3_DOWNLEVEL_PROTOCOL_VERSION.into(),
            ]),
            require_read_only_hint: true,
            max_tool_list_pages: DEFAULT_MCP_READONLY_V3_MAX_TOOL_LIST_PAGES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct McpReadOnlyReadiness {
    pub server_id: String,
    pub selected_tool: String,
    pub negotiated_protocol_version: String,
    pub read_only_hint: bool,
}

#[derive(Debug)]
pub struct McpReadOnlyResolverV3 {
    config: McpReadOnlyResolverV3Config,
    config_id: String,
    cancellation: Option<SubprocessCancellation>,
}

impl McpReadOnlyResolverV3 {
    pub fn new(config: McpReadOnlyResolverV3Config) -> Self {
        let config_id = stable_config_id(MCP_READONLY_V3_RESOLVER_ID, &config);
        Self {
            config,
            config_id,
            cancellation: None,
        }
    }

    pub fn with_cancellation(mut self, cancellation: SubprocessCancellation) -> Self {
        self.cancellation = Some(cancellation);
        self
    }

    /// Verifies MCP session negotiation and the selected tool's server-declared read-only proof
    /// without invoking the tool. This is a product-readiness check only; it creates no evidence
    /// and grants no authority.
    pub fn probe_readiness(&self) -> Result<McpReadOnlyReadiness, ResolutionAdapterError> {
        let started = Instant::now();
        let base = &self.config.base;
        if base.server_id.trim().is_empty()
            || base.tool.trim().is_empty()
            || base.source.trim().is_empty()
            || base.resolver_class != ResolverClass::EvidenceAcquisition
            || !base.allowed_tools.contains(&base.tool)
            || base.timeout_ms == 0
            || base.max_response_bytes == 0
            || self.config.requested_protocol_version.trim().is_empty()
            || self.config.supported_protocol_versions.is_empty()
            || !self
                .config
                .supported_protocol_versions
                .contains(&self.config.requested_protocol_version)
            || self
                .config
                .supported_protocol_versions
                .iter()
                .any(|version| version.trim().is_empty())
            || self.config.max_tool_list_pages == 0
        {
            return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
        }

        let timeout = Duration::from_millis(base.timeout_ms);
        let mut command = Command::new(&base.program);
        command.args(&base.args);
        isolate_subprocess_environment(&mut command);
        let mut session = DeadlineLineSession::spawn(
            &mut command,
            started,
            timeout,
            base.max_response_bytes,
            self.cancellation.clone(),
        )
        .map_err(|kind| error(kind, started))?;

        let id_prefix = "reasoning-harness:mcp-readiness";
        let initialize_id = format!("{id_prefix}:initialize");
        write_json(
            &mut session,
            &json!({
                "jsonrpc": "2.0",
                "id": initialize_id,
                "method": "initialize",
                "params": {
                    "protocolVersion": self.config.requested_protocol_version,
                    "capabilities": {},
                    "clientInfo": {
                        "name": MCP_CLIENT_NAME,
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            }),
        )
        .map_err(|kind| error(session_io_kind(kind), started))?;
        let initialize_response = read_rpc(&mut session, &initialize_id)
            .map_err(|kind| error(session_io_kind(kind), started))?;
        if let Some(rpc_error) = initialize_response.error {
            let kind = rpc_error_kind(&rpc_error);
            return Err(error(
                if kind == ResolutionAdapterErrorKind::Protocol {
                    ResolutionAdapterErrorKind::Negotiation
                } else {
                    kind
                },
                started,
            ));
        }
        let initialize_result: McpInitializeResult = serde_json::from_value(
            initialize_response
                .result
                .ok_or_else(|| error(ResolutionAdapterErrorKind::Negotiation, started))?,
        )
        .map_err(|_| error(ResolutionAdapterErrorKind::Negotiation, started))?;
        if !self
            .config
            .supported_protocol_versions
            .contains(&initialize_result.protocol_version)
        {
            return Err(error(ResolutionAdapterErrorKind::Negotiation, started));
        }
        let negotiated_protocol = initialize_result.protocol_version;

        write_json(
            &mut session,
            &json!({
                "jsonrpc": "2.0",
                "method": "notifications/initialized"
            }),
        )
        .map_err(|kind| error(session_io_kind(kind), started))?;

        let mut cursor = None::<String>;
        let mut seen_cursors = BTreeSet::new();
        let mut pagination_incomplete = false;
        for page in 0..self.config.max_tool_list_pages {
            let list_id = format!("{id_prefix}:tools-list:{page}");
            let params = match cursor.as_deref() {
                Some(cursor) => json!({ "cursor": cursor }),
                None => json!({}),
            };
            write_json(
                &mut session,
                &json!({
                    "jsonrpc": "2.0",
                    "id": list_id,
                    "method": "tools/list",
                    "params": params
                }),
            )
            .map_err(|kind| error(session_io_kind(kind), started))?;
            let response = read_rpc(&mut session, &list_id)
                .map_err(|kind| error(session_io_kind(kind), started))?;
            if let Some(rpc_error) = response.error {
                let kind = rpc_error_kind(&rpc_error);
                return Err(error(
                    if kind == ResolutionAdapterErrorKind::Protocol {
                        ResolutionAdapterErrorKind::Session
                    } else {
                        kind
                    },
                    started,
                ));
            }
            let result: McpToolsListResult = serde_json::from_value(
                response
                    .result
                    .ok_or_else(|| error(ResolutionAdapterErrorKind::Protocol, started))?,
            )
            .map_err(|_| error(ResolutionAdapterErrorKind::Protocol, started))?;
            if let Some(tool) = result.tools.iter().find(|tool| tool.name == base.tool) {
                let read_only_hint = tool.annotations.as_ref().and_then(|a| a.read_only_hint);
                if self.config.require_read_only_hint && read_only_hint != Some(true) {
                    return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
                }
                return Ok(McpReadOnlyReadiness {
                    server_id: base.server_id.clone(),
                    selected_tool: base.tool.clone(),
                    negotiated_protocol_version: negotiated_protocol,
                    read_only_hint: read_only_hint == Some(true),
                });
            }
            let Some(next_cursor) = result.next_cursor.filter(|cursor| !cursor.is_empty()) else {
                break;
            };
            if !seen_cursors.insert(next_cursor.clone()) {
                return Err(error(ResolutionAdapterErrorKind::Session, started));
            }
            if page + 1 == self.config.max_tool_list_pages {
                pagination_incomplete = true;
                break;
            }
            cursor = Some(next_cursor);
        }
        Err(error(
            if pagination_incomplete {
                ResolutionAdapterErrorKind::Session
            } else {
                ResolutionAdapterErrorKind::PolicyDenied
            },
            started,
        ))
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<McpRpcError>,
}

#[derive(Debug, Deserialize)]
struct McpRpcError {
    #[allow(dead_code)]
    code: i64,
    #[allow(dead_code)]
    message: String,
    #[serde(default)]
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct McpInitializeResult {
    protocol_version: String,
}

#[derive(Debug, Deserialize)]
struct McpToolsListResult {
    tools: Vec<McpToolDescriptor>,
    #[serde(default, rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McpToolDescriptor {
    name: String,
    #[serde(default)]
    annotations: Option<McpToolAnnotations>,
}

#[derive(Debug, Deserialize)]
struct McpToolAnnotations {
    #[serde(default, rename = "readOnlyHint")]
    read_only_hint: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct McpCallToolResult {
    content: Vec<Value>,
    #[serde(default, rename = "structuredContent")]
    structured_content: Option<Value>,
    #[serde(default, rename = "isError")]
    is_error: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct McpAcquisitionEnvelope {
    reasoning_harness: McpAcquisitionPayload,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct McpAcquisitionPayload {
    observation: String,
    #[serde(default)]
    facts: BTreeMap<String, String>,
    #[serde(default)]
    acquisition_metadata: AcquiredEvidenceMetadata,
}

fn measured_cost(started: Instant) -> ResolutionCost {
    ResolutionCost {
        elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        calls: 1,
        ..ResolutionCost::default()
    }
}

fn error(kind: ResolutionAdapterErrorKind, started: Instant) -> ResolutionAdapterError {
    ResolutionAdapterError {
        kind,
        cost: measured_cost(started),
    }
}

fn session_io_kind(kind: ResolutionAdapterErrorKind) -> ResolutionAdapterErrorKind {
    match kind {
        ResolutionAdapterErrorKind::Timeout
        | ResolutionAdapterErrorKind::Unavailable
        | ResolutionAdapterErrorKind::PermissionDenied
        | ResolutionAdapterErrorKind::PolicyDenied
        | ResolutionAdapterErrorKind::Protocol => kind,
        _ => ResolutionAdapterErrorKind::Session,
    }
}

fn rpc_error_kind(error: &McpRpcError) -> ResolutionAdapterErrorKind {
    let kind = error
        .data
        .as_ref()
        .and_then(|data| data.pointer("/reasoning_harness/operational_kind"))
        .and_then(Value::as_str);
    match kind {
        Some("authentication") => ResolutionAdapterErrorKind::Authentication,
        Some("permission_denied") => ResolutionAdapterErrorKind::PermissionDenied,
        Some("timeout") => ResolutionAdapterErrorKind::Timeout,
        Some("transport") => ResolutionAdapterErrorKind::Transport,
        Some("policy_denied") => ResolutionAdapterErrorKind::PolicyDenied,
        Some("tool_execution") => ResolutionAdapterErrorKind::ToolExecution,
        _ => ResolutionAdapterErrorKind::Protocol,
    }
}

fn parse_rpc_response(
    line: &[u8],
    expected_id: &str,
) -> Result<JsonRpcResponse, ResolutionAdapterErrorKind> {
    let response: JsonRpcResponse =
        serde_json::from_slice(line).map_err(|_| ResolutionAdapterErrorKind::Protocol)?;
    if response.jsonrpc != "2.0" || response.id != Value::String(expected_id.into()) {
        return Err(ResolutionAdapterErrorKind::Protocol);
    }
    if response.result.is_some() == response.error.is_some() {
        return Err(ResolutionAdapterErrorKind::Protocol);
    }
    Ok(response)
}

fn write_json(
    session: &mut DeadlineLineSession,
    value: &Value,
) -> Result<(), ResolutionAdapterErrorKind> {
    let payload = serde_json::to_vec(value).map_err(|_| ResolutionAdapterErrorKind::Protocol)?;
    session.write_line(payload)
}

fn read_rpc(
    session: &mut DeadlineLineSession,
    expected_id: &str,
) -> Result<JsonRpcResponse, ResolutionAdapterErrorKind> {
    let line = session.read_line()?;
    parse_rpc_response(&line, expected_id)
}

fn opaque_observation(result: &McpCallToolResult) -> String {
    if let Some(structured) = &result.structured_content {
        serde_json::to_string(structured).unwrap_or_else(|_| "mcp structured result".into())
    } else {
        serde_json::to_string(&result.content).unwrap_or_else(|_| "mcp tool result".into())
    }
}

fn contribution_from_result(
    config: &McpReadOnlyResolverV3Config,
    request: &ResolutionRequest,
    attempt_index: usize,
    negotiated_protocol: &str,
    result: McpCallToolResult,
) -> ResolutionResolverContribution {
    let payload = result
        .structured_content
        .as_ref()
        .and_then(|value| serde_json::from_value::<McpAcquisitionEnvelope>(value.clone()).ok())
        .map(|envelope| envelope.reasoning_harness);
    let (observation, facts, acquisition_metadata) = if let Some(payload) = payload {
        (
            payload.observation,
            payload.facts,
            payload.acquisition_metadata,
        )
    } else {
        (
            opaque_observation(&result),
            BTreeMap::new(),
            AcquiredEvidenceMetadata::default(),
        )
    };
    ResolutionResolverContribution::AcquiredEvidence {
        evidence: vec![AcquiredEvidence {
            id: format!(
                "mcp:{}:{}:{}:{}:{}",
                config.base.server_id,
                config.base.tool,
                negotiated_protocol,
                request.id,
                attempt_index
            ),
            source: config.base.source.clone(),
            observation,
            facts,
            acquisition_metadata,
        }],
    }
}

impl ResolutionResolver for McpReadOnlyResolverV3 {
    fn name(&self) -> &'static str {
        MCP_READONLY_V3_RESOLVER_ID
    }

    fn class(&self) -> ResolverClass {
        self.config.base.resolver_class
    }

    fn config_id(&self) -> Option<&str> {
        Some(&self.config_id)
    }

    fn resolve(
        &self,
        request: &ResolutionRequest,
        attempt_index: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
        let started = Instant::now();
        let base = &self.config.base;
        if base.server_id.trim().is_empty()
            || base.tool.trim().is_empty()
            || base.source.trim().is_empty()
            || base.resolver_class != ResolverClass::EvidenceAcquisition
            || !base.allowed_tools.contains(&base.tool)
            || base.timeout_ms == 0
            || base.max_response_bytes == 0
            || self.config.requested_protocol_version.trim().is_empty()
            || self.config.supported_protocol_versions.is_empty()
            || !self
                .config
                .supported_protocol_versions
                .contains(&self.config.requested_protocol_version)
            || self
                .config
                .supported_protocol_versions
                .iter()
                .any(|version| version.trim().is_empty())
            || self.config.max_tool_list_pages == 0
        {
            return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
        }

        let provenance_base = json!({
            "request_id": request.id,
            "attempt_index": attempt_index,
            "resolver": MCP_READONLY_V3_RESOLVER_ID,
            "config_id": self.config_id,
            "server_id": base.server_id,
            "tool": base.tool,
        });
        let mut arguments = base.fixed_arguments.clone();
        if let Some(argument) = base.provenance_argument.as_deref() {
            if argument.trim().is_empty() || arguments.contains_key(argument) {
                return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
            }
            arguments.insert(argument.into(), provenance_base.clone());
        }

        let timeout = Duration::from_millis(base.timeout_ms);
        let mut command = Command::new(&base.program);
        command.args(&base.args);
        isolate_subprocess_environment(&mut command);
        let mut session = DeadlineLineSession::spawn(
            &mut command,
            started,
            timeout,
            base.max_response_bytes,
            self.cancellation.clone(),
        )
        .map_err(|kind| error(kind, started))?;

        let id_prefix = format!("reasoning-harness:{}:{}", request.id, attempt_index);
        let initialize_id = format!("{id_prefix}:initialize");
        let initialize = json!({
            "jsonrpc": "2.0",
            "id": initialize_id,
            "method": "initialize",
            "params": {
                "protocolVersion": self.config.requested_protocol_version,
                "capabilities": {},
                "clientInfo": {
                    "name": MCP_CLIENT_NAME,
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });
        write_json(&mut session, &initialize)
            .map_err(|kind| error(session_io_kind(kind), started))?;
        let initialize_response = read_rpc(&mut session, &initialize_id)
            .map_err(|kind| error(session_io_kind(kind), started))?;
        if let Some(rpc_error) = initialize_response.error {
            let kind = rpc_error_kind(&rpc_error);
            return Err(error(
                if kind == ResolutionAdapterErrorKind::Protocol {
                    ResolutionAdapterErrorKind::Negotiation
                } else {
                    kind
                },
                started,
            ));
        }
        let initialize_result: McpInitializeResult = serde_json::from_value(
            initialize_response
                .result
                .ok_or_else(|| error(ResolutionAdapterErrorKind::Negotiation, started))?,
        )
        .map_err(|_| error(ResolutionAdapterErrorKind::Negotiation, started))?;
        if !self
            .config
            .supported_protocol_versions
            .contains(&initialize_result.protocol_version)
        {
            return Err(error(ResolutionAdapterErrorKind::Negotiation, started));
        }
        let negotiated_protocol = initialize_result.protocol_version;

        let initialized = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        write_json(&mut session, &initialized)
            .map_err(|kind| error(session_io_kind(kind), started))?;

        let mut cursor = None::<String>;
        let mut seen_cursors = BTreeSet::new();
        let mut selected_tool_verified = false;
        let mut pagination_incomplete = false;
        for page in 0..self.config.max_tool_list_pages {
            let list_id = format!("{id_prefix}:tools-list:{page}");
            let params = match cursor.as_deref() {
                Some(cursor) => json!({ "cursor": cursor }),
                None => json!({}),
            };
            write_json(
                &mut session,
                &json!({
                    "jsonrpc": "2.0",
                    "id": list_id,
                    "method": "tools/list",
                    "params": params
                }),
            )
            .map_err(|kind| error(session_io_kind(kind), started))?;
            let response = read_rpc(&mut session, &list_id)
                .map_err(|kind| error(session_io_kind(kind), started))?;
            if let Some(rpc_error) = response.error {
                let kind = rpc_error_kind(&rpc_error);
                return Err(error(
                    if kind == ResolutionAdapterErrorKind::Protocol {
                        ResolutionAdapterErrorKind::Session
                    } else {
                        kind
                    },
                    started,
                ));
            }
            let result: McpToolsListResult = serde_json::from_value(
                response
                    .result
                    .ok_or_else(|| error(ResolutionAdapterErrorKind::Protocol, started))?,
            )
            .map_err(|_| error(ResolutionAdapterErrorKind::Protocol, started))?;
            if let Some(tool) = result.tools.iter().find(|tool| tool.name == base.tool) {
                if self.config.require_read_only_hint
                    && tool.annotations.as_ref().and_then(|a| a.read_only_hint) != Some(true)
                {
                    return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
                }
                selected_tool_verified = true;
                break;
            }
            let Some(next_cursor) = result.next_cursor.filter(|cursor| !cursor.is_empty()) else {
                break;
            };
            if !seen_cursors.insert(next_cursor.clone()) {
                return Err(error(ResolutionAdapterErrorKind::Session, started));
            }
            if page + 1 == self.config.max_tool_list_pages {
                pagination_incomplete = true;
                break;
            }
            cursor = Some(next_cursor);
        }
        if !selected_tool_verified {
            return Err(error(
                if pagination_incomplete {
                    ResolutionAdapterErrorKind::Session
                } else {
                    ResolutionAdapterErrorKind::PolicyDenied
                },
                started,
            ));
        }

        let tool_call_id = format!("{id_prefix}:tools-call");
        let provenance = json!({
            "request_id": request.id,
            "attempt_index": attempt_index,
            "resolver": MCP_READONLY_V3_RESOLVER_ID,
            "config_id": self.config_id,
            "server_id": base.server_id,
            "tool": base.tool,
            "requested_protocol_version": self.config.requested_protocol_version,
            "negotiated_protocol_version": negotiated_protocol,
        });
        write_json(
            &mut session,
            &json!({
                "jsonrpc": "2.0",
                "id": tool_call_id,
                "method": "tools/call",
                "params": {
                    "name": base.tool,
                    "arguments": arguments,
                    "_meta": {
                        "io.modelcontextprotocol/protocolVersion": negotiated_protocol,
                        "io.modelcontextprotocol/clientCapabilities": {},
                        "io.modelcontextprotocol/clientInfo": {
                            "name": MCP_CLIENT_NAME,
                            "version": env!("CARGO_PKG_VERSION")
                        },
                        MCP_PROVENANCE_META_KEY: provenance
                    }
                }
            }),
        )
        .map_err(|kind| error(session_io_kind(kind), started))?;
        let response = read_rpc(&mut session, &tool_call_id)
            .map_err(|kind| error(session_io_kind(kind), started))?;
        match (response.result, response.error) {
            (Some(result), None) => {
                let result: McpCallToolResult = serde_json::from_value(result)
                    .map_err(|_| error(ResolutionAdapterErrorKind::Protocol, started))?;
                if result.is_error.unwrap_or(false) {
                    return Err(error(ResolutionAdapterErrorKind::ToolExecution, started));
                }
                Ok(ResolutionResolverOutput {
                    contribution: contribution_from_result(
                        &self.config,
                        request,
                        attempt_index,
                        &negotiated_protocol,
                        result,
                    ),
                    cost: measured_cost(started),
                })
            }
            (None, Some(rpc_error)) => {
                let kind = rpc_error_kind(&rpc_error);
                Err(error(
                    if kind == ResolutionAdapterErrorKind::Protocol {
                        ResolutionAdapterErrorKind::ToolExecution
                    } else {
                        kind
                    },
                    started,
                ))
            }
            _ => Err(error(ResolutionAdapterErrorKind::Protocol, started)),
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

    use reasoning_harness_core::{
        Proposition, ResolutionReason, ResolutionRequestBudget, ResolutionTarget,
    };

    use super::*;

    fn request() -> ResolutionRequest {
        ResolutionRequest {
            id: "resolution:service.region".into(),
            reason: ResolutionReason::MissingSupport,
            target: ResolutionTarget::Proposition {
                proposition: Proposition {
                    key: "service.region".into(),
                    value: "eu-west-1".into(),
                },
            },
            resolver_class: ResolverClass::EvidenceAcquisition,
            budget: ResolutionRequestBudget::default(),
        }
    }

    fn script(body: &str, name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "reason-mcp-readonly-v3-{}-{name}.sh",
            std::process::id()
        ));
        fs::write(&path, body).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    fn config(path: PathBuf) -> McpReadOnlyResolverV3Config {
        McpReadOnlyResolverV3Config::with_defaults(McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            path,
            "lookup",
            "mcp:fixture:lookup",
        ))
    }

    #[test]
    fn readiness_probe_negotiates_and_verifies_read_only_without_calling_tool() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s' "$initialize" | grep -q '"method":"initialize"' || exit 2
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-readiness:initialize","result":{"protocolVersion":"2025-11-25","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
printf '%s' "$initialized" | grep -q '"method":"notifications/initialized"' || exit 3
read list
printf '%s' "$list" | grep -q '"method":"tools/list"' || exit 4
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-readiness:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":true}}]}}'
# A readiness probe must end here; any tools/call would make the test hang/fail.
"#,
            "readiness",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let readiness = resolver.probe_readiness().unwrap();
        fs::remove_file(path).ok();
        assert_eq!(readiness.server_id, "fixture-server");
        assert_eq!(readiness.selected_tool, "lookup");
        assert_eq!(readiness.negotiated_protocol_version, "2025-11-25");
        assert!(readiness.read_only_hint);
    }

    #[test]
    fn readiness_probe_rejects_selected_tool_without_read_only_proof() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-readiness:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-readiness:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":false}}]}}'
"#,
            "readiness-write-hint",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let failure = resolver.probe_readiness().unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::PolicyDenied);
    }

    #[test]
    fn negotiated_session_accepts_allowlisted_downgrade_and_keeps_generic_content_opaque() {
        let path = script(
            r#"#!/bin/sh
[ -z "${REASON_SUBPROCESS_SENTINEL_SECRET+x}" ] || exit 90
[ -n "${PATH:-}" ] || exit 91
read initialize
printf '%s' "$initialize" | grep -q '"method":"initialize"' || exit 2
printf '%s' "$initialize" | grep -q '"protocolVersion":"2026-07-28"' || exit 3
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2025-11-25","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
printf '%s' "$initialized" | grep -q '"method":"notifications/initialized"' || exit 4
read list
printf '%s' "$list" | grep -q '"method":"tools/list"' || exit 5
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-list:0","result":{"tools":[{"name":"lookup","description":"read","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":true}}]}}'
read call
printf '%s' "$call" | grep -q '"method":"tools/call"' || exit 6
printf '%s' "$call" | grep -q '"io.modelcontextprotocol/protocolVersion":"2025-11-25"' || exit 7
printf '%s' "$call" | grep -q '"mcp_readonly_v3"' || exit 8
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-call","result":{"content":[{"type":"text","text":"probably eu-west-1"}],"resultType":"file"}}'
"#,
            "negotiated",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let output = resolver.resolve(&request(), 0).unwrap();
        fs::remove_file(path).ok();
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                assert_eq!(evidence.len(), 1);
                assert!(evidence[0].id.contains("2025-11-25"));
                assert!(evidence[0].facts.is_empty());
                assert_eq!(
                    evidence[0].acquisition_metadata,
                    AcquiredEvidenceMetadata::default()
                );
            }
            other => panic!("expected acquired evidence, got {other:?}"),
        }
    }

    #[test]
    fn explicit_harness_envelope_is_preserved_as_untrusted_acquisition_data() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":true}}]}}'
read call
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-call","result":{"content":[],"structuredContent":{"reasoning_harness":{"observation":"service.region=eu-west-1","facts":{"service.region":"eu-west-1"},"acquisition_metadata":{"claimed_authority_class":"primary"}}}}}'
"#,
            "envelope",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let output = resolver.resolve(&request(), 0).unwrap();
        fs::remove_file(path).ok();
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                assert_eq!(evidence[0].facts["service.region"], "eu-west-1");
                assert_eq!(
                    evidence[0]
                        .acquisition_metadata
                        .claimed_authority_class
                        .as_deref(),
                    Some("primary")
                );
            }
            other => panic!("expected acquired evidence, got {other:?}"),
        }
    }

    #[test]
    fn unsupported_negotiated_revision_is_typed_negotiation_failure() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
"#,
            "downgrade-denied",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let failure = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::Negotiation);
    }

    #[test]
    fn selected_tool_requires_server_read_only_proof() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":false}}]}}'
"#,
            "write-hint",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let failure = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::PolicyDenied);
    }

    #[test]
    fn session_break_after_initialize_is_typed_session_failure() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
exit 0
"#,
            "session-break",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let failure = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::Session);
    }

    #[test]
    fn malformed_rpc_remains_protocol_failure() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' 'not-json'
"#,
            "protocol",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let failure = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::Protocol);
    }

    #[test]
    fn tool_is_error_remains_tool_execution_failure() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":true}}]}}'
read call
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-call","result":{"content":[{"type":"text","text":"failed"}],"isError":true}}'
"#,
            "tool-error",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let failure = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::ToolExecution);
    }

    #[test]
    fn tool_json_rpc_error_is_typed_tool_execution_failure() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object"},"annotations":{"readOnlyHint":true}}]}}'
read call
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-call","error":{"code":-32602,"message":"invalid tool arguments"}}'
"#,
            "tool-rpc-error",
        );
        let resolver = McpReadOnlyResolverV3::new(config(path.clone()));
        let failure = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::ToolExecution);
    }

    #[test]
    fn whole_session_lifecycle_uses_one_absolute_deadline() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
sleep 1
"#,
            "deadline",
        );
        let mut cfg = config(path.clone());
        cfg.base.timeout_ms = 40;
        let resolver = McpReadOnlyResolverV3::new(cfg);
        let wall = Instant::now();
        let failure = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::Timeout);
        assert!(wall.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn tool_list_page_budget_exhaustion_is_typed_session_failure() {
        let path = script(
            r#"#!/bin/sh
read initialize
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:initialize","result":{"protocolVersion":"2026-07-28","capabilities":{},"serverInfo":{"name":"fixture","version":"1"}}}'
read initialized
read list
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0:tools-list:0","result":{"tools":[],"nextCursor":"page-2"}}'
"#,
            "page-budget",
        );
        let mut cfg = config(path.clone());
        cfg.max_tool_list_pages = 1;
        let resolver = McpReadOnlyResolverV3::new(cfg);
        let failure = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::Session);
    }

    #[test]
    #[ignore = "requires Docker and GITHUB_PERSONAL_ACCESS_TOKEN"]
    fn official_github_mcp_pinned_image_negotiates_and_generic_result_stays_opaque() {
        if std::env::var_os("GITHUB_PERSONAL_ACCESS_TOKEN").is_none() {
            panic!("GITHUB_PERSONAL_ACCESS_TOKEN is required for the ignored acceptance probe");
        }
        let mut base = McpReadOnlyResolverConfig::with_defaults(
            "github-official-v1.12.0",
            PathBuf::from("docker"),
            "get_file_contents",
            "github:official-mcp:generic-readme",
        );
        base.args = vec![
            "run".into(),
            "-i".into(),
            "--rm".into(),
            "-e".into(),
            "GITHUB_PERSONAL_ACCESS_TOKEN".into(),
            "-e".into(),
            "GITHUB_READ_ONLY=1".into(),
            "-e".into(),
            "GITHUB_TOOLS=get_file_contents".into(),
            "ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33".into(),
        ];
        base.fixed_arguments = BTreeMap::from([
            ("owner".into(), Value::String("github".into())),
            ("repo".into(), Value::String("github-mcp-server".into())),
            ("path".into(), Value::String("README.md".into())),
            ("ref".into(), Value::String("refs/heads/main".into())),
        ]);
        base.timeout_ms = 30_000;
        let resolver = McpReadOnlyResolverV3::new(McpReadOnlyResolverV3Config::with_defaults(base));
        let output = resolver
            .resolve(&request(), 0)
            .expect("official GitHub MCP v3 acceptance");
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                assert_eq!(evidence.len(), 1);
                assert!(evidence[0].facts.is_empty());
                assert_eq!(evidence[0].source, "github:official-mcp:generic-readme");
            }
            other => panic!("expected opaque acquired evidence, got {other:?}"),
        }
    }

    #[test]
    fn config_identity_binds_negotiation_policy() {
        let path = PathBuf::from("fixture");
        let first = McpReadOnlyResolverV3::new(config(path.clone()));
        let mut changed = config(path);
        changed
            .supported_protocol_versions
            .remove(MCP_READONLY_V3_DOWNLEVEL_PROTOCOL_VERSION);
        let second = McpReadOnlyResolverV3::new(changed);
        assert_ne!(first.config_id(), second.config_id());
    }
}
