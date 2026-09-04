use std::{
    collections::BTreeMap,
    env, fs,
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use reasoning_harness_core::{HarnessInput, ReasoningArtifact, ReasoningCandidate};
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const MCP_PROTOCOL_VERSION: &str = "2026-07-28";
const MCP_PRODUCT_SERVER_ID: &str = "reasoning_harness_mcp_v1";
const CLI_OUTPUT_CONTRACT: &str = "reason-cli-output-v1";
const DEFAULT_MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
const DEFAULT_MAX_RESPONSE_BYTES: usize = 8 * 1024 * 1024;
const DEFAULT_NATIVE_TIMEOUT_MS: u64 = 30_000;
const STDERR_LIMIT: u64 = 64 * 1024;
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Parser)]
#[command(
    name = "reason-mcp",
    about = "Optional MCP product adapter for the native reason runtime"
)]
struct Args {
    /// Path to the supported native `reason` executable. Defaults to a sibling binary.
    #[arg(long)]
    reason_command: Option<PathBuf>,
    #[arg(long, default_value_t = DEFAULT_MAX_REQUEST_BYTES)]
    max_request_bytes: usize,
    #[arg(long, default_value_t = DEFAULT_MAX_RESPONSE_BYTES)]
    max_response_bytes: usize,
    #[arg(long, default_value_t = DEFAULT_NATIVE_TIMEOUT_MS)]
    native_timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CallToolParams {
    name: String,
    #[serde(default)]
    arguments: Value,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
enum AskProvider {
    Mistral,
    Google,
    Nvidia,
}

impl AskProvider {
    const fn cli_value(self) -> &'static str {
        match self {
            Self::Mistral => "mistral",
            Self::Google => "google",
            Self::Nvidia => "nvidia",
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct AskToolInput {
    task: String,
    provider: AskProvider,
    model: String,
    #[serde(default)]
    context: Vec<String>,
    #[serde(default)]
    facts: BTreeMap<String, String>,
    #[serde(default)]
    hypotheses: BTreeMap<String, String>,
    #[serde(default)]
    resolver_facts: BTreeMap<String, String>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    max_resolution_attempts: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RunToolInput {
    input: HarnessInput,
    candidate: ReasoningCandidate,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct VerifyToolInput {
    artifact: ReasoningArtifact,
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum SchemaKind {
    Artifact,
    Candidate,
    Config,
    SemanticCheck,
}

impl SchemaKind {
    const fn cli_value(self) -> &'static str {
        match self {
            Self::Artifact => "artifact",
            Self::Candidate => "candidate",
            Self::Config => "config",
            Self::SemanticCheck => "semantic-check",
        }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SchemaToolInput {
    kind: SchemaKind,
}

#[derive(Debug)]
struct ServerConfig {
    reason_command: PathBuf,
    max_request_bytes: usize,
    max_response_bytes: usize,
    native_timeout: Duration,
}

#[derive(Debug)]
enum NativeFailure {
    Spawn,
    Timeout,
    OutputTooLarge,
    Protocol,
    Io,
}

impl NativeFailure {
    const fn class(&self) -> &'static str {
        match self {
            Self::Spawn => "native_spawn",
            Self::Timeout => "timeout",
            Self::OutputTooLarge => "native_output_too_large",
            Self::Protocol => "native_protocol",
            Self::Io => "native_io",
        }
    }
}

#[derive(Debug)]
struct NativeInvocation {
    envelope: Value,
    is_error: bool,
}

#[derive(Debug)]
struct TempInvocationDir {
    path: PathBuf,
}

impl TempInvocationDir {
    fn new() -> Result<Self, NativeFailure> {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| NativeFailure::Io)?
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "reason-mcp-{}-{nanos}-{counter}",
            std::process::id()
        ));
        fs::create_dir(&path).map_err(|_| NativeFailure::Io)?;
        Ok(Self { path })
    }

    fn write_json(&self, name: &str, value: &impl Serialize) -> Result<PathBuf, NativeFailure> {
        let path = self.path.join(name);
        let bytes = serde_json::to_vec(value).map_err(|_| NativeFailure::Protocol)?;
        fs::write(&path, bytes).map_err(|_| NativeFailure::Io)?;
        Ok(path)
    }

    fn write_text(&self, name: &str, value: &str) -> Result<PathBuf, NativeFailure> {
        let path = self.path.join(name);
        fs::write(&path, value.as_bytes()).map_err(|_| NativeFailure::Io)?;
        Ok(path)
    }
}

impl Drop for TempInvocationDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn sibling_reason_command() -> Result<PathBuf, String> {
    let current = env::current_exe().map_err(|error| error.to_string())?;
    let parent = current
        .parent()
        .ok_or_else(|| "reason-mcp executable has no parent directory".to_string())?;
    #[cfg(windows)]
    let name = "reason.exe";
    #[cfg(not(windows))]
    let name = "reason";
    Ok(parent.join(name))
}

fn tool_list() -> Value {
    json!({
        "tools": [
            {
                "name": "reason_ask",
                "title": "Ask Reasoning Harness",
                "description": "Execute the supported native natural-language runtime including bounded resolution, finalization, and answer safety. This closed surface accepts no arbitrary config, raw CLI arguments, or direct verification receipts.",
                "inputSchema": serde_json::to_value(schema_for!(AskToolInput)).expect("ask schema serializes"),
                "annotations": {"readOnlyHint": true}
            },
            {
                "name": "reason_run",
                "title": "Run Reasoning Harness",
                "description": "Execute the supported structured native reason runtime. The caller supplies HarnessInput and ReasoningCandidate; this tool does not accept trusted receipts, provider generation, arbitrary config, or raw CLI arguments.",
                "inputSchema": serde_json::to_value(schema_for!(RunToolInput)).expect("run schema serializes"),
                "annotations": {"readOnlyHint": true}
            },
            {
                "name": "reason_verify",
                "title": "Verify Reasoning Artifact",
                "description": "Validate a reasoning-artifact-v1 through the supported native reason verify product command.",
                "inputSchema": serde_json::to_value(schema_for!(VerifyToolInput)).expect("verify schema serializes"),
                "annotations": {"readOnlyHint": true}
            },
            {
                "name": "reason_schema",
                "title": "Get Reasoning Harness Schema",
                "description": "Return an existing native Reasoning Harness product schema without redefining the contract in MCP.",
                "inputSchema": serde_json::to_value(schema_for!(SchemaToolInput)).expect("schema tool schema serializes"),
                "annotations": {"readOnlyHint": true}
            }
        ],
        "ttlMs": 0,
        "cacheScope": "private",
        "_meta": {
            "io.modelcontextprotocol/protocolVersion": MCP_PROTOCOL_VERSION,
            "git-ksk/reasoning-harness/server": MCP_PRODUCT_SERVER_ID,
            "git-ksk/reasoning-harness/scope": "one MCP tool invocation is one native runtime invocation; it does not certify the caller agent loop"
        }
    })
}

fn server_discover() -> Value {
    json!({
        "supportedVersions": [MCP_PROTOCOL_VERSION],
        "capabilities": {"tools": {}},
        "instructions": "Reasoning Harness exposes a thin read-only product adapter over selected native reason operations.",
        "ttlMs": 0,
        "cacheScope": "private"
    })
}

fn stamp_server_info(mut result: Value) -> Value {
    if let Some(object) = result.as_object_mut() {
        let meta = object.entry("_meta").or_insert_with(|| json!({}));
        if let Some(meta) = meta.as_object_mut() {
            meta.insert(
                "io.modelcontextprotocol/serverInfo".into(),
                json!({"name":"reasoning-harness","version":env!("CARGO_PKG_VERSION")}),
            );
        }
    }
    result
}

fn rpc_result(id: Value, result: Value) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(stamp_server_info(result)),
        error: None,
    }
}

fn rpc_error(id: Value, code: i64, message: impl Into<String>, data: Option<Value>) -> RpcResponse {
    RpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(RpcError {
            code,
            message: message.into(),
            data,
        }),
    }
}

fn tool_result(envelope: Value, is_error: bool) -> Value {
    let text = serde_json::to_string(&envelope).unwrap_or_else(|_| "reason native output".into());
    json!({
        "content": [{"type":"text","text":text}],
        "structuredContent": envelope,
        "isError": is_error,
        "_meta": {
            "io.modelcontextprotocol/protocolVersion": MCP_PROTOCOL_VERSION,
            "git-ksk/reasoning-harness/server": MCP_PRODUCT_SERVER_ID,
            "git-ksk/reasoning-harness/native_contract": CLI_OUTPUT_CONTRACT,
            "git-ksk/reasoning-harness/scope": "this result applies only to this native Reasoning Harness invocation"
        }
    })
}

fn operational_tool_error(class: &str) -> Value {
    tool_result(
        json!({
            "schema_version": "reason-mcp-operational-failure-v1",
            "status": "failed",
            "failure": {"failure_class": class},
            "server": MCP_PRODUCT_SERVER_ID
        }),
        true,
    )
}

fn read_bounded<R: Read>(reader: R, limit: usize) -> Result<Vec<u8>, NativeFailure> {
    let take = u64::try_from(limit)
        .ok()
        .and_then(|value| value.checked_add(1))
        .ok_or(NativeFailure::OutputTooLarge)?;
    let mut bytes = Vec::new();
    reader
        .take(take)
        .read_to_end(&mut bytes)
        .map_err(|_| NativeFailure::Io)?;
    if bytes.len() > limit {
        return Err(NativeFailure::OutputTooLarge);
    }
    Ok(bytes)
}

fn invoke_native(
    config: &ServerConfig,
    args: &[String],
    expected_command: &str,
) -> Result<NativeInvocation, NativeFailure> {
    let mut child = Command::new(&config.reason_command)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| NativeFailure::Spawn)?;
    let stdout = child.stdout.take().ok_or(NativeFailure::Io)?;
    let stderr = child.stderr.take().ok_or(NativeFailure::Io)?;
    let response_limit = config.max_response_bytes;
    let stdout_reader = thread::spawn(move || read_bounded(stdout, response_limit));
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.take(STDERR_LIMIT).read_to_end(&mut bytes);
        bytes
    });

    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < config.native_timeout => {
                thread::sleep(PROCESS_POLL_INTERVAL)
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(NativeFailure::Timeout);
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(NativeFailure::Io);
            }
        }
    };
    let stdout = stdout_reader.join().map_err(|_| NativeFailure::Io)??;
    let _ = stderr_reader.join();
    let envelope: Value = serde_json::from_slice(&stdout).map_err(|_| NativeFailure::Protocol)?;
    if envelope.get("schema_version").and_then(Value::as_str) != Some(CLI_OUTPUT_CONTRACT)
        || envelope.get("command").and_then(Value::as_str) != Some(expected_command)
    {
        return Err(NativeFailure::Protocol);
    }
    let is_error = !status.success();
    Ok(NativeInvocation { envelope, is_error })
}

fn key_value_arg(key: &str, value: &str) -> Result<String, NativeFailure> {
    if key.trim().is_empty() || key.contains('=') || key.contains('\n') || key.contains('\r') {
        return Err(NativeFailure::Protocol);
    }
    Ok(format!("{key}={value}"))
}

fn build_ask_args(
    temp: &TempInvocationDir,
    input: &AskToolInput,
) -> Result<Vec<String>, NativeFailure> {
    if input.task.trim().is_empty() || input.model.trim().is_empty() {
        return Err(NativeFailure::Protocol);
    }
    if input.max_tokens == Some(0) || input.max_resolution_attempts == Some(0) {
        return Err(NativeFailure::Protocol);
    }

    let mut args = vec![
        "--provider".into(),
        input.provider.cli_value().into(),
        "--model".into(),
        input.model.clone(),
    ];
    if let Some(max_tokens) = input.max_tokens {
        args.extend(["--max-tokens".into(), max_tokens.to_string()]);
    }
    if let Some(seed) = input.seed {
        args.extend(["--seed".into(), seed.to_string()]);
    }
    if let Some(max_attempts) = input.max_resolution_attempts {
        args.extend(["--max-resolution-attempts".into(), max_attempts.to_string()]);
    }
    for (index, context) in input.context.iter().enumerate() {
        let path = temp.write_text(&format!("context-{index}.txt"), context)?;
        args.extend(["--file".into(), path.to_string_lossy().into_owned()]);
    }
    for (key, value) in &input.facts {
        args.extend(["--fact".into(), key_value_arg(key, value)?]);
    }
    for (key, value) in &input.hypotheses {
        args.extend(["--hypothesis".into(), key_value_arg(key, value)?]);
    }
    for (key, value) in &input.resolver_facts {
        args.extend(["--resolver-fact".into(), key_value_arg(key, value)?]);
    }
    args.extend([
        "--no-config".into(),
        "--format".into(),
        "json".into(),
        "--".into(),
    ]);
    args.push(input.task.clone());
    Ok(args)
}

fn call_ask(config: &ServerConfig, value: Value) -> Result<NativeInvocation, NativeFailure> {
    let input: AskToolInput = serde_json::from_value(value).map_err(|_| NativeFailure::Protocol)?;
    let temp = TempInvocationDir::new()?;
    let args = build_ask_args(&temp, &input)?;
    invoke_native(config, &args, "ask")
}

fn call_run(config: &ServerConfig, value: Value) -> Result<NativeInvocation, NativeFailure> {
    let input: RunToolInput = serde_json::from_value(value).map_err(|_| NativeFailure::Protocol)?;
    let temp = TempInvocationDir::new()?;
    let input_path = temp.write_json("input.json", &input.input)?;
    let candidate_path = temp.write_json("candidate.json", &input.candidate)?;
    invoke_native(
        config,
        &[
            "run".into(),
            "--input".into(),
            input_path.to_string_lossy().into_owned(),
            "--candidate".into(),
            candidate_path.to_string_lossy().into_owned(),
            "--no-config".into(),
            "--format".into(),
            "json".into(),
        ],
        "run",
    )
}

fn call_verify(config: &ServerConfig, value: Value) -> Result<NativeInvocation, NativeFailure> {
    let input: VerifyToolInput =
        serde_json::from_value(value).map_err(|_| NativeFailure::Protocol)?;
    let temp = TempInvocationDir::new()?;
    let artifact_path = temp.write_json("artifact.json", &input.artifact)?;
    invoke_native(
        config,
        &[
            "verify".into(),
            artifact_path.to_string_lossy().into_owned(),
            "--format".into(),
            "json".into(),
        ],
        "verify",
    )
}

fn call_schema(config: &ServerConfig, value: Value) -> Result<NativeInvocation, NativeFailure> {
    let input: SchemaToolInput =
        serde_json::from_value(value).map_err(|_| NativeFailure::Protocol)?;
    invoke_native(
        config,
        &["schema".into(), input.kind.cli_value().into()],
        "schema",
    )
}

fn handle_call(config: &ServerConfig, params: Value) -> Result<Value, NativeFailure> {
    let call: CallToolParams =
        serde_json::from_value(params).map_err(|_| NativeFailure::Protocol)?;
    let invocation = match call.name.as_str() {
        "reason_ask" => call_ask(config, call.arguments)?,
        "reason_run" => call_run(config, call.arguments)?,
        "reason_verify" => call_verify(config, call.arguments)?,
        "reason_schema" => call_schema(config, call.arguments)?,
        _ => return Err(NativeFailure::Protocol),
    };
    Ok(tool_result(invocation.envelope, invocation.is_error))
}

fn handle_request(config: &ServerConfig, request: RpcRequest) -> RpcResponse {
    if request.jsonrpc != "2.0" {
        return rpc_error(request.id, -32600, "invalid JSON-RPC version", None);
    }
    match request.method.as_str() {
        "server/discover" => rpc_result(request.id, server_discover()),
        "tools/list" => rpc_result(request.id, tool_list()),
        "tools/call" => {
            let Some(params) = request.params else {
                return rpc_error(request.id, -32602, "tools/call requires params", None);
            };
            match handle_call(config, params) {
                Ok(result) => rpc_result(request.id, result),
                Err(NativeFailure::Protocol) => rpc_error(
                    request.id,
                    -32602,
                    "invalid tool name, arguments, or native product output",
                    Some(json!({"reasoning_harness":{"operational_kind":"protocol"}})),
                ),
                Err(failure) => rpc_result(request.id, operational_tool_error(failure.class())),
            }
        }
        "ping" => rpc_result(request.id, json!({})),
        _ => rpc_error(request.id, -32601, "method not found", None),
    }
}

fn serve(config: ServerConfig) -> Result<(), String> {
    if config.max_request_bytes == 0
        || config.max_response_bytes == 0
        || config.native_timeout.is_zero()
    {
        return Err("request/response limits and native timeout must be greater than zero".into());
    }
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    loop {
        let mut line = Vec::new();
        let read = reader
            .read_until(b'\n', &mut line)
            .map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        while line
            .last()
            .is_some_and(|byte| *byte == b'\n' || *byte == b'\r')
        {
            line.pop();
        }
        if line.is_empty() {
            continue;
        }
        let response = if line.len() > config.max_request_bytes {
            rpc_error(
                Value::Null,
                -32600,
                "request exceeds configured byte limit",
                None,
            )
        } else {
            match serde_json::from_slice::<RpcRequest>(&line) {
                Ok(request) => handle_request(&config, request),
                Err(_) => rpc_error(Value::Null, -32700, "parse error", None),
            }
        };
        let bytes = serde_json::to_vec(&response).map_err(|error| error.to_string())?;
        if bytes.len() > config.max_response_bytes {
            let fallback = serde_json::to_vec(&rpc_error(
                Value::Null,
                -32603,
                "MCP response exceeds configured byte limit",
                None,
            ))
            .map_err(|error| error.to_string())?;
            writer
                .write_all(&fallback)
                .map_err(|error| error.to_string())?;
        } else {
            writer
                .write_all(&bytes)
                .map_err(|error| error.to_string())?;
        }
        writer.write_all(b"\n").map_err(|error| error.to_string())?;
        writer.flush().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let reason_command = args
        .reason_command
        .map_or_else(sibling_reason_command, Ok)?;
    serve(ServerConfig {
        reason_command,
        max_request_bytes: args.max_request_bytes,
        max_response_bytes: args.max_response_bytes,
        native_timeout: Duration::from_millis(args.native_timeout_ms),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_list_exposes_only_selected_native_surfaces() {
        let list = tool_list();
        let names = list["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            vec!["reason_ask", "reason_run", "reason_verify", "reason_schema"]
        );
        let run_schema = serde_json::to_string(&list["tools"][1]["inputSchema"]).unwrap();
        assert!(!run_schema.contains("verification_receipts"));
        assert!(!run_schema.contains("provider"));
        assert!(!run_schema.contains("api_key"));
    }

    #[test]
    fn schemas_reuse_native_types_and_are_closed_at_mcp_wrapper() {
        let ask = serde_json::to_value(schema_for!(AskToolInput)).unwrap();
        let run = serde_json::to_value(schema_for!(RunToolInput)).unwrap();
        let verify = serde_json::to_value(schema_for!(VerifyToolInput)).unwrap();
        assert_eq!(ask["additionalProperties"], false);
        assert_eq!(run["additionalProperties"], false);
        assert_eq!(verify["additionalProperties"], false);
        assert!(run.to_string().contains("HarnessInput"));
        assert!(run.to_string().contains("ReasoningCandidate"));
        assert!(verify.to_string().contains("ReasoningArtifact"));
        let ask_text = ask.to_string();
        assert!(!ask_text.contains("receipts"));
        assert!(!ask_text.contains("config"));
        assert!(!ask_text.contains("cli_args"));
    }

    #[test]
    fn ask_args_are_literal_closed_and_task_is_after_option_terminator() {
        let temp = TempInvocationDir::new().unwrap();
        let input = AskToolInput {
            task: "--not-an-option".into(),
            provider: AskProvider::Google,
            model: "model-x".into(),
            context: vec!["untrusted context".into()],
            facts: BTreeMap::from([("status.code".into(), "200".into())]),
            hypotheses: BTreeMap::from([("service.ready".into(), "true".into())]),
            resolver_facts: BTreeMap::new(),
            max_tokens: Some(256),
            seed: Some(7),
            max_resolution_attempts: Some(2),
        };
        let args = build_ask_args(&temp, &input).unwrap();
        let terminator = args.iter().position(|arg| arg == "--").unwrap();
        assert_eq!(&args[terminator + 1], "--not-an-option");
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--fact", "status.code=200"])
        );
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--hypothesis", "service.ready=true"])
        );
        assert!(
            !args
                .iter()
                .any(|arg| arg == "--config" || arg == "--receipts")
        );
    }

    #[test]
    fn modern_discovery_and_cache_hints_are_explicit() {
        let discover = stamp_server_info(server_discover());
        assert_eq!(discover["supportedVersions"][0], MCP_PROTOCOL_VERSION);
        assert_eq!(discover["ttlMs"], 0);
        assert_eq!(discover["cacheScope"], "private");
        assert_eq!(
            discover["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            "reasoning-harness"
        );
        let list = tool_list();
        assert_eq!(list["ttlMs"], 0);
        assert_eq!(list["cacheScope"], "private");
    }

    #[test]
    fn native_ask_envelope_is_passed_through_with_finalization_and_identity() {
        let native = json!({
            "schema_version": CLI_OUTPUT_CONTRACT,
            "command": "ask",
            "result": {
                "configuration": {"mode":"natural_language_provider","model":"model-x"},
                "safety_runtime": {"profile":"verified_target_v1","configuration_id":"verified-target-answer-gate-v1"},
                "finalization": {"status":"grounded_answer","factual_claims":1,"covered_claims":1,"factual_claim_coverage":1.0,"uncovered_propositions":[]}
            }
        });
        let result = tool_result(native.clone(), false);
        assert_eq!(result["structuredContent"], native);
        assert_eq!(
            result["structuredContent"]["result"]["finalization"]["status"],
            "grounded_answer"
        );
        assert_eq!(
            result["structuredContent"]["result"]["safety_runtime"]["configuration_id"],
            "verified-target-answer-gate-v1"
        );
    }

    #[test]
    fn scope_metadata_never_claims_to_verify_the_caller_loop() {
        let list = tool_list();
        let scope = list["_meta"]["git-ksk/reasoning-harness/scope"]
            .as_str()
            .unwrap();
        assert!(scope.contains("does not certify the caller agent loop"));
    }
}
