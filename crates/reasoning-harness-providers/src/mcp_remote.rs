use std::{
    collections::{BTreeMap, BTreeSet},
    thread,
    time::{Duration, Instant},
};

use reasoning_harness_core::{
    AcquiredEvidence, AcquiredEvidenceMetadata, ResolutionAdapterError, ResolutionAdapterErrorKind,
    ResolutionCost, ResolutionRequest, ResolutionResolver, ResolutionResolverContribution,
    ResolutionResolverOutput, ResolverClass,
};
use reqwest::{Client, StatusCode, header, redirect};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{SubprocessCancellation, config_identity::stable_config_id};

pub const MCP_REMOTE_READONLY_RESOLVER_ID: &str = "mcp_remote_readonly_v1";
pub const MCP_REMOTE_PROTOCOL_VERSION: &str = "2026-07-28";
pub const DEFAULT_MCP_REMOTE_TIMEOUT_MS: u64 = 10_000;
pub const DEFAULT_MCP_REMOTE_MAX_RESPONSE_BYTES: usize = 262_144;
pub const DEFAULT_MCP_REMOTE_MAX_TOOL_LIST_PAGES: usize = 8;

const MCP_CLIENT_NAME: &str = "reasoning-harness";
const MCP_PROVENANCE_META_KEY: &str = "git-ksk/reasoning-harness/provenance";
const REMOTE_CANCELLATION_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct McpRemoteReadOnlyResolverConfig {
    pub server_id: String,
    pub endpoint: String,
    pub allowed_tools: BTreeSet<String>,
    pub tool: String,
    pub resolver_class: ResolverClass,
    pub fixed_arguments: BTreeMap<String, Value>,
    pub provenance_argument: Option<String>,
    pub source: String,
    pub protocol_version: String,
    pub require_read_only_hint: bool,
    pub max_tool_list_pages: usize,
    pub timeout_ms: u64,
    pub max_response_bytes: usize,
}

impl McpRemoteReadOnlyResolverConfig {
    pub fn with_defaults(
        server_id: impl Into<String>,
        endpoint: impl Into<String>,
        tool: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        let tool = tool.into();
        Self {
            server_id: server_id.into(),
            endpoint: endpoint.into(),
            allowed_tools: BTreeSet::from([tool.clone()]),
            tool,
            resolver_class: ResolverClass::EvidenceAcquisition,
            fixed_arguments: BTreeMap::new(),
            provenance_argument: None,
            source: source.into(),
            protocol_version: MCP_REMOTE_PROTOCOL_VERSION.into(),
            require_read_only_hint: true,
            max_tool_list_pages: DEFAULT_MCP_REMOTE_MAX_TOOL_LIST_PAGES,
            timeout_ms: DEFAULT_MCP_REMOTE_TIMEOUT_MS,
            max_response_bytes: DEFAULT_MCP_REMOTE_MAX_RESPONSE_BYTES,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct McpRemoteReadiness {
    pub server_id: String,
    pub selected_tool: String,
    pub protocol_version: String,
    pub read_only_hint: bool,
}

pub struct McpRemoteReadOnlyResolver {
    config: McpRemoteReadOnlyResolverConfig,
    config_id: String,
    access_token: Option<String>,
    cancellation: Option<SubprocessCancellation>,
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
    #[serde(default, rename = "inputSchema")]
    input_schema: Option<Value>,
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

impl McpRemoteReadOnlyResolver {
    pub fn new(
        config: McpRemoteReadOnlyResolverConfig,
        access_token: Option<String>,
    ) -> Result<Self, ResolutionAdapterErrorKind> {
        validate_config(&config)?;
        let config_id = stable_config_id(MCP_REMOTE_READONLY_RESOLVER_ID, &config);
        Ok(Self {
            config,
            config_id,
            access_token,
            cancellation: None,
        })
    }

    pub fn with_cancellation(mut self, cancellation: SubprocessCancellation) -> Self {
        self.cancellation = Some(cancellation);
        self
    }

    pub fn probe_readiness(&self) -> Result<McpRemoteReadiness, ResolutionAdapterError> {
        let started = Instant::now();
        self.verify_selected_tool(started)?;
        Ok(McpRemoteReadiness {
            server_id: self.config.server_id.clone(),
            selected_tool: self.config.tool.clone(),
            protocol_version: self.config.protocol_version.clone(),
            read_only_hint: true,
        })
    }

    fn verify_selected_tool(&self, started: Instant) -> Result<(), ResolutionAdapterError> {
        let mut cursor = None::<String>;
        let mut seen = BTreeSet::new();
        let mut page_exhausted = false;
        for page in 0..self.config.max_tool_list_pages {
            let id = format!("reasoning-harness:remote:tools-list:{page}");
            let mut params = serde_json::Map::new();
            if let Some(cursor) = cursor.as_deref() {
                params.insert("cursor".into(), Value::String(cursor.into()));
            }
            params.insert("_meta".into(), client_meta());
            let response = self
                .rpc("tools/list", None, &id, Value::Object(params), started)
                .map_err(|kind| error(kind, started))?;
            let result: McpToolsListResult = serde_json::from_value(
                response
                    .result
                    .ok_or_else(|| error(ResolutionAdapterErrorKind::Protocol, started))?,
            )
            .map_err(|_| error(ResolutionAdapterErrorKind::Protocol, started))?;
            if let Some(tool) = result
                .tools
                .iter()
                .find(|tool| tool.name == self.config.tool)
            {
                if tool
                    .input_schema
                    .as_ref()
                    .is_some_and(contains_custom_mcp_header)
                {
                    return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
                }
                if self.config.require_read_only_hint
                    && tool.annotations.as_ref().and_then(|a| a.read_only_hint) != Some(true)
                {
                    return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
                }
                return Ok(());
            }
            let Some(next) = result.next_cursor.filter(|value| !value.is_empty()) else {
                break;
            };
            if !seen.insert(next.clone()) {
                return Err(error(ResolutionAdapterErrorKind::Session, started));
            }
            if page + 1 == self.config.max_tool_list_pages {
                page_exhausted = true;
                break;
            }
            cursor = Some(next);
        }
        Err(error(
            if page_exhausted {
                ResolutionAdapterErrorKind::Session
            } else {
                ResolutionAdapterErrorKind::PolicyDenied
            },
            started,
        ))
    }

    fn rpc(
        &self,
        method: &str,
        name: Option<&str>,
        id: &str,
        params: Value,
        _started: Instant,
    ) -> Result<JsonRpcResponse, ResolutionAdapterErrorKind> {
        if self
            .cancellation
            .as_ref()
            .is_some_and(SubprocessCancellation::is_cancelled)
        {
            return Err(ResolutionAdapterErrorKind::Transport);
        }
        let endpoint = self.config.endpoint.clone();
        let timeout_ms = self.config.timeout_ms;
        let protocol_version = self.config.protocol_version.clone();
        let access_token = self.access_token.clone();
        let cancellation = self.cancellation.clone();
        let max_response_bytes = self.config.max_response_bytes;
        let method = method.to_string();
        let name = name.map(str::to_string);
        let id = id.to_string();
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| ResolutionAdapterErrorKind::Transport)?;
            runtime.block_on(async move {
                let request = async move {
                    let client = Client::builder()
                        .timeout(Duration::from_millis(timeout_ms))
                        .redirect(redirect::Policy::none())
                        .build()
                        .map_err(|_| ResolutionAdapterErrorKind::Transport)?;
                    let mut request = client
                        .post(&endpoint)
                        .header("MCP-Protocol-Version", &protocol_version)
                        .header("Mcp-Method", &method)
                        .header(header::ACCEPT, "application/json, text/event-stream")
                        .json(&json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "method": method,
                            "params": params
                        }));
                    if let Some(name) = name.as_deref() {
                        request = request.header("Mcp-Name", name);
                    }
                    if let Some(token) = access_token.as_deref() {
                        request = request.bearer_auth(token);
                    }
                    let mut response = request
                        .send()
                        .await
                        .map_err(|_| ResolutionAdapterErrorKind::Transport)?;
                    match response.status() {
                        StatusCode::UNAUTHORIZED => {
                            return Err(ResolutionAdapterErrorKind::Authentication);
                        }
                        StatusCode::FORBIDDEN => {
                            return Err(ResolutionAdapterErrorKind::PermissionDenied);
                        }
                        StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE => {
                            return Err(ResolutionAdapterErrorKind::Unavailable);
                        }
                        status if !status.is_success() => {
                            return Err(ResolutionAdapterErrorKind::Transport);
                        }
                        _ => {}
                    }
                    let content_type = response
                        .headers()
                        .get(header::CONTENT_TYPE)
                        .and_then(|value| value.to_str().ok())
                        .unwrap_or("")
                        .to_ascii_lowercase();
                    let mut body = Vec::new();
                    while let Some(chunk) = response
                        .chunk()
                        .await
                        .map_err(|_| ResolutionAdapterErrorKind::Transport)?
                    {
                        if body.len().saturating_add(chunk.len()) > max_response_bytes {
                            return Err(ResolutionAdapterErrorKind::Protocol);
                        }
                        body.extend_from_slice(&chunk);
                    }
                    let payload = if content_type.contains("text/event-stream") {
                        first_sse_data(&body)?
                    } else {
                        body
                    };
                    let parsed: JsonRpcResponse = serde_json::from_slice(&payload)
                        .map_err(|_| ResolutionAdapterErrorKind::Protocol)?;
                    if parsed.jsonrpc != "2.0" || parsed.id != Value::String(id.clone()) {
                        return Err(ResolutionAdapterErrorKind::Protocol);
                    }
                    if parsed.error.is_some() || parsed.result.is_none() {
                        return Err(ResolutionAdapterErrorKind::Protocol);
                    }
                    Ok(parsed)
                };
                tokio::pin!(request);
                let cancellation_after = cancellation.clone();
                let result = tokio::select! {
                    biased;
                    () = wait_for_remote_cancellation(cancellation) => {
                        Err(ResolutionAdapterErrorKind::Transport)
                    }
                    result = &mut request => result,
                };
                if cancellation_after
                    .as_ref()
                    .is_some_and(SubprocessCancellation::is_cancelled)
                {
                    Err(ResolutionAdapterErrorKind::Transport)
                } else {
                    result
                }
            })
        })
        .join()
        .map_err(|_| ResolutionAdapterErrorKind::Transport)?
    }
}

impl ResolutionResolver for McpRemoteReadOnlyResolver {
    fn name(&self) -> &'static str {
        MCP_REMOTE_READONLY_RESOLVER_ID
    }

    fn class(&self) -> ResolverClass {
        self.config.resolver_class
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
        self.verify_selected_tool(started)?;
        let mut arguments = self.config.fixed_arguments.clone();
        let provenance = json!({
            "request_id": request.id,
            "attempt_index": attempt_index,
            "resolver": MCP_REMOTE_READONLY_RESOLVER_ID,
            "config_id": self.config_id,
            "server_id": self.config.server_id,
            "tool": self.config.tool,
            "protocol_version": self.config.protocol_version,
        });
        if let Some(argument) = self.config.provenance_argument.as_deref() {
            if argument.trim().is_empty() || arguments.contains_key(argument) {
                return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
            }
            arguments.insert(argument.into(), provenance.clone());
        }
        let id = format!(
            "reasoning-harness:remote:{}:{attempt_index}:tools-call",
            request.id
        );
        let result = self
            .rpc(
                "tools/call",
                Some(&self.config.tool),
                &id,
                json!({
                    "name": self.config.tool,
                    "arguments": arguments,
                    "_meta": {
                        "io.modelcontextprotocol/protocolVersion": self.config.protocol_version,
                        "io.modelcontextprotocol/clientInfo": {"name": MCP_CLIENT_NAME, "version": env!("CARGO_PKG_VERSION")},
                        "io.modelcontextprotocol/clientCapabilities": {},
                        MCP_PROVENANCE_META_KEY: provenance
                    }
                }),
                started,
            )
            .map_err(|kind| error(kind, started))?;
        let result: McpCallToolResult = serde_json::from_value(
            result
                .result
                .ok_or_else(|| error(ResolutionAdapterErrorKind::Protocol, started))?,
        )
        .map_err(|_| error(ResolutionAdapterErrorKind::Protocol, started))?;
        if result.is_error.unwrap_or(false) {
            return Err(error(ResolutionAdapterErrorKind::ToolExecution, started));
        }
        Ok(ResolutionResolverOutput {
            contribution: contribution_from_result(&self.config, request, attempt_index, result),
            cost: measured_cost(started),
        })
    }
}

async fn wait_for_remote_cancellation(cancellation: Option<SubprocessCancellation>) {
    let Some(cancellation) = cancellation else {
        std::future::pending::<()>().await;
        return;
    };
    while !cancellation.is_cancelled() {
        tokio::time::sleep(REMOTE_CANCELLATION_POLL_INTERVAL).await;
    }
}

fn validate_config(
    config: &McpRemoteReadOnlyResolverConfig,
) -> Result<(), ResolutionAdapterErrorKind> {
    if config.server_id.trim().is_empty()
        || config.endpoint.trim().is_empty()
        || config.tool.trim().is_empty()
        || config.source.trim().is_empty()
        || config.resolver_class != ResolverClass::EvidenceAcquisition
        || !config.allowed_tools.contains(&config.tool)
        || config.allowed_tools.is_empty()
        || config.protocol_version != MCP_REMOTE_PROTOCOL_VERSION
        || !config.require_read_only_hint
        || config.max_tool_list_pages == 0
        || config.max_tool_list_pages > 32
        || config.timeout_ms == 0
        || config.max_response_bytes == 0
    {
        return Err(ResolutionAdapterErrorKind::PolicyDenied);
    }
    let url = reqwest::Url::parse(&config.endpoint)
        .map_err(|_| ResolutionAdapterErrorKind::PolicyDenied)?;
    if (url.scheme() != "https" && !is_loopback_http(&url))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(ResolutionAdapterErrorKind::PolicyDenied);
    }
    Ok(())
}

fn is_loopback_http(url: &reqwest::Url) -> bool {
    url.scheme() == "http"
        && matches!(
            url.host_str(),
            Some("127.0.0.1" | "localhost" | "[::1]" | "::1")
        )
}

fn client_meta() -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": MCP_REMOTE_PROTOCOL_VERSION,
        "io.modelcontextprotocol/clientInfo": {"name": MCP_CLIENT_NAME, "version": env!("CARGO_PKG_VERSION")},
        "io.modelcontextprotocol/clientCapabilities": {}
    })
}

fn contains_custom_mcp_header(value: &Value) -> bool {
    match value {
        Value::Object(object) => object
            .iter()
            .any(|(key, value)| key == "x-mcp-header" || contains_custom_mcp_header(value)),
        Value::Array(values) => values.iter().any(contains_custom_mcp_header),
        _ => false,
    }
}

fn first_sse_data(body: &[u8]) -> Result<Vec<u8>, ResolutionAdapterErrorKind> {
    let text = std::str::from_utf8(body).map_err(|_| ResolutionAdapterErrorKind::Protocol)?;
    let mut data = Vec::new();
    for line in text.lines() {
        if line.is_empty() {
            if !data.is_empty() {
                break;
            }
            continue;
        }
        if let Some(value) = line.strip_prefix("data:") {
            let value = value.strip_prefix(' ').unwrap_or(value);
            if value == "[DONE]" && data.is_empty() {
                continue;
            }
            data.push(value);
        }
    }
    if data.is_empty() {
        return Err(ResolutionAdapterErrorKind::Protocol);
    }
    Ok(data.join("\n").into_bytes())
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

fn opaque_observation(result: &McpCallToolResult) -> String {
    if let Some(structured) = &result.structured_content {
        serde_json::to_string(structured).unwrap_or_else(|_| "mcp structured result".into())
    } else {
        serde_json::to_string(&result.content).unwrap_or_else(|_| "mcp tool result".into())
    }
}

fn contribution_from_result(
    config: &McpRemoteReadOnlyResolverConfig,
    request: &ResolutionRequest,
    attempt_index: usize,
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
                "mcp-remote:{}:{}:{}:{}:{}",
                config.server_id, config.tool, config.protocol_version, request.id, attempt_index
            ),
            source: config.source.clone(),
            observation,
            facts,
            acquisition_metadata,
        }],
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Read, Write},
        net::{TcpListener, TcpStream},
        sync::mpsc,
        thread,
    };

    use super::*;

    fn read_request(stream: &mut TcpStream, expected_method: &str) {
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut first = String::new();
        reader.read_line(&mut first).unwrap();
        assert!(first.starts_with("POST "));
        let mut headers = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line == "\r\n" {
                break;
            }
            headers.push_str(&line);
        }
        let lower = headers.to_ascii_lowercase();
        assert!(lower.contains(&format!("mcp-method: {expected_method}")));
        assert!(lower.contains("mcp-protocol-version: 2026-07-28"));
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap();
        let mut request_body = vec![0u8; content_length];
        reader.read_exact(&mut request_body).unwrap();
        let request_body = String::from_utf8(request_body).unwrap();
        assert!(request_body.contains("io.modelcontextprotocol/protocolVersion"));
        assert!(request_body.contains("2026-07-28"));
    }

    fn serve_once(body: &'static str, expected_method: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut first = String::new();
            reader.read_line(&mut first).unwrap();
            assert!(first.starts_with("POST "));
            let mut headers = String::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                headers.push_str(&line);
            }
            let lower = headers.to_ascii_lowercase();
            assert!(lower.contains(&format!("mcp-method: {expected_method}")));
            assert!(lower.contains("mcp-protocol-version: 2026-07-28"));
            let content_length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())
                        .flatten()
                })
                .unwrap();
            let mut request_body = vec![0u8; content_length];
            reader.read_exact(&mut request_body).unwrap();
            let request_body = String::from_utf8(request_body).unwrap();
            assert!(request_body.contains("io.modelcontextprotocol/protocolVersion"));
            assert!(request_body.contains("2026-07-28"));
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        format!("http://{addr}/mcp")
    }

    #[test]
    fn readiness_is_stateless_and_requires_read_only_hint() {
        let endpoint = serve_once(
            r#"{"jsonrpc":"2.0","id":"reasoning-harness:remote:tools-list:0","result":{"tools":[{"name":"lookup","annotations":{"readOnlyHint":true}}]}}"#,
            "tools/list",
        );
        let config = McpRemoteReadOnlyResolverConfig::with_defaults(
            "fixture",
            endpoint,
            "lookup",
            "mcp:fixture:lookup",
        );
        let resolver = McpRemoteReadOnlyResolver::new(config, None).unwrap();
        let ready = resolver.probe_readiness().unwrap();
        assert!(ready.read_only_hint);
        assert_eq!(ready.protocol_version, MCP_REMOTE_PROTOCOL_VERSION);
    }

    #[test]
    fn readiness_rejects_write_capable_tool() {
        let endpoint = serve_once(
            r#"{"jsonrpc":"2.0","id":"reasoning-harness:remote:tools-list:0","result":{"tools":[{"name":"lookup","annotations":{"readOnlyHint":false}}]}}"#,
            "tools/list",
        );
        let config = McpRemoteReadOnlyResolverConfig::with_defaults(
            "fixture",
            endpoint,
            "lookup",
            "mcp:fixture:lookup",
        );
        let resolver = McpRemoteReadOnlyResolver::new(config, None).unwrap();
        assert_eq!(
            resolver.probe_readiness().unwrap_err().kind,
            ResolutionAdapterErrorKind::PolicyDenied
        );
    }

    #[test]
    fn readiness_fails_closed_for_unimplemented_x_mcp_header_contract() {
        let endpoint = serve_once(
            r#"{"jsonrpc":"2.0","id":"reasoning-harness:remote:tools-list:0","result":{"tools":[{"name":"lookup","inputSchema":{"type":"object","properties":{"region":{"type":"string","x-mcp-header":"Region"}}},"annotations":{"readOnlyHint":true}}]}}"#,
            "tools/list",
        );
        let config = McpRemoteReadOnlyResolverConfig::with_defaults(
            "fixture",
            endpoint,
            "lookup",
            "mcp:fixture:lookup",
        );
        let resolver = McpRemoteReadOnlyResolver::new(config, None).unwrap();
        assert_eq!(
            resolver.probe_readiness().unwrap_err().kind,
            ResolutionAdapterErrorKind::PolicyDenied
        );
    }

    #[test]
    fn sse_parser_combines_first_event_data_lines() {
        let body = b"event: message\ndata: {\"jsonrpc\":\"2.0\",\ndata: \"id\":1}\n\nevent: message\ndata: ignored\n\n";
        let parsed = first_sse_data(body).unwrap();
        assert_eq!(
            String::from_utf8(parsed).unwrap(),
            "{\"jsonrpc\":\"2.0\",\n\"id\":1}"
        );
    }

    #[test]
    fn remote_acquisition_uses_bearer_without_binding_token_into_config_identity() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let token = "remote-secret-token";
        thread::spawn(move || {
            for (expected_method, body) in [
                (
                    "tools/list",
                    r#"{"jsonrpc":"2.0","id":"reasoning-harness:remote:tools-list:0","result":{"tools":[{"name":"lookup","annotations":{"readOnlyHint":true}}]}}"#,
                ),
                (
                    "tools/call",
                    r#"{"jsonrpc":"2.0","id":"reasoning-harness:remote:resolution:service.region:0:tools-call","result":{"content":[{"type":"text","text":"opaque"}],"structuredContent":{"reasoning_harness":{"observation":"service.region=eu-west-1","facts":{"service.region":"eu-west-1"}}}}}"#,
                ),
            ] {
                let (mut stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut first = String::new();
                reader.read_line(&mut first).unwrap();
                let mut headers = String::new();
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" {
                        break;
                    }
                    headers.push_str(&line);
                }
                let lower = headers.to_ascii_lowercase();
                assert!(lower.contains(&format!("mcp-method: {expected_method}")));
                assert!(lower.contains("authorization: bearer remote-secret-token"));
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        let endpoint = format!("http://{addr}/mcp");
        let config = McpRemoteReadOnlyResolverConfig::with_defaults(
            "fixture",
            endpoint,
            "lookup",
            "mcp:fixture:lookup",
        );
        let first = McpRemoteReadOnlyResolver::new(config.clone(), Some(token.into())).unwrap();
        let second =
            McpRemoteReadOnlyResolver::new(config, Some("different-secret".into())).unwrap();
        assert_eq!(first.config_id(), second.config_id());
        assert!(!first.config_id().unwrap().contains("secret"));
        let request = reasoning_harness_core::ResolutionRequest {
            id: "resolution:service.region".into(),
            reason: reasoning_harness_core::ResolutionReason::MissingSupport,
            target: reasoning_harness_core::ResolutionTarget::Proposition {
                proposition: reasoning_harness_core::Proposition {
                    key: "service.region".into(),
                    value: "eu-west-1".into(),
                },
            },
            resolver_class: ResolverClass::EvidenceAcquisition,
            budget: reasoning_harness_core::ResolutionRequestBudget::default(),
        };
        let output = first.resolve(&request, 0).unwrap();
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                assert_eq!(evidence[0].source, "mcp:fixture:lookup");
                assert_eq!(evidence[0].facts["service.region"], "eu-west-1");
            }
            other => panic!("expected acquired evidence, got {other:?}"),
        }
    }

    #[test]
    fn readiness_cancellation_interrupts_http_before_remote_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (request_started_tx, request_started_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            read_request(&mut stream, "tools/list");
            request_started_tx.send(()).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut byte = [0u8; 1];
            match stream.read(&mut byte) {
                Ok(0) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe
                    ) => {}
                other => panic!("cancelled client connection remained active: {other:?}"),
            }
        });

        let mut config = McpRemoteReadOnlyResolverConfig::with_defaults(
            "fixture",
            format!("http://{addr}/mcp"),
            "lookup",
            "mcp:fixture:lookup",
        );
        config.timeout_ms = 3_000;
        let cancellation = SubprocessCancellation::default();
        let cancel_signal = cancellation.clone();
        let canceller = thread::spawn(move || {
            request_started_rx
                .recv_timeout(Duration::from_secs(1))
                .unwrap();
            cancel_signal.cancel();
        });
        let resolver = McpRemoteReadOnlyResolver::new(config, None)
            .unwrap()
            .with_cancellation(cancellation);
        let started = Instant::now();
        let error = resolver.probe_readiness().unwrap_err();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Transport);
        assert!(started.elapsed() < Duration::from_secs(1));
        canceller.join().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn acquisition_cancellation_interrupts_tools_call_without_evidence() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (call_started_tx, call_started_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut readiness_stream, _) = listener.accept().unwrap();
            read_request(&mut readiness_stream, "tools/list");
            let body = r#"{"jsonrpc":"2.0","id":"reasoning-harness:remote:tools-list:0","result":{"tools":[{"name":"lookup","annotations":{"readOnlyHint":true}}]}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            readiness_stream.write_all(response.as_bytes()).unwrap();
            drop(readiness_stream);

            let (mut call_stream, _) = listener.accept().unwrap();
            read_request(&mut call_stream, "tools/call");
            call_started_tx.send(()).unwrap();
            call_stream
                .set_read_timeout(Some(Duration::from_secs(1)))
                .unwrap();
            let mut byte = [0u8; 1];
            match call_stream.read(&mut byte) {
                Ok(0) => {}
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::BrokenPipe
                    ) => {}
                other => panic!("cancelled tools/call connection remained active: {other:?}"),
            }
        });

        let mut config = McpRemoteReadOnlyResolverConfig::with_defaults(
            "fixture",
            format!("http://{addr}/mcp"),
            "lookup",
            "mcp:fixture:lookup",
        );
        config.timeout_ms = 3_000;
        let cancellation = SubprocessCancellation::default();
        let cancel_signal = cancellation.clone();
        let canceller = thread::spawn(move || {
            call_started_rx
                .recv_timeout(Duration::from_secs(1))
                .unwrap();
            cancel_signal.cancel();
        });
        let resolver = McpRemoteReadOnlyResolver::new(config, None)
            .unwrap()
            .with_cancellation(cancellation);
        let request = reasoning_harness_core::ResolutionRequest {
            id: "resolution:service.region".into(),
            reason: reasoning_harness_core::ResolutionReason::MissingSupport,
            target: reasoning_harness_core::ResolutionTarget::Proposition {
                proposition: reasoning_harness_core::Proposition {
                    key: "service.region".into(),
                    value: "eu-west-1".into(),
                },
            },
            resolver_class: ResolverClass::EvidenceAcquisition,
            budget: reasoning_harness_core::ResolutionRequestBudget::default(),
        };
        let started = Instant::now();
        let error = resolver.resolve(&request, 0).unwrap_err();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Transport);
        assert!(started.elapsed() < Duration::from_secs(1));
        canceller.join().unwrap();
        server.join().unwrap();
    }

    #[test]
    fn non_tls_remote_endpoint_is_rejected_except_loopback() {
        let config = McpRemoteReadOnlyResolverConfig::with_defaults(
            "fixture",
            "http://example.com/mcp",
            "lookup",
            "mcp:fixture:lookup",
        );
        assert!(McpRemoteReadOnlyResolver::new(config, None).is_err());
    }
}
