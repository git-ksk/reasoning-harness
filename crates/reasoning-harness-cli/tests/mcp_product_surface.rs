use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::{Value, json};

fn repo_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn native_reason(args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_reason"))
        .args(args)
        .output()
        .unwrap();
    assert!(output.status.success(), "native reason failed");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn mcp_session(reason_command: &Path, requests: &[Value]) -> Vec<Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_reason-mcp"))
        .arg("--reason-command")
        .arg(reason_command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        for request in requests {
            serde_json::to_writer(&mut *stdin, request).unwrap();
            stdin.write_all(b"\n").unwrap();
        }
    }
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "reason-mcp failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn mcp_run_is_byte_semantically_equal_to_supported_native_run_contract() {
    let input_path = repo_path("examples/input.json");
    let candidate_path = repo_path("examples/candidate.json");
    let input = read_json(&input_path);
    let candidate = read_json(&candidate_path);
    let direct = native_reason(&[
        "run",
        "--input",
        input_path.to_str().unwrap(),
        "--candidate",
        candidate_path.to_str().unwrap(),
        "--no-config",
        "--format",
        "json",
    ]);
    let responses = mcp_session(
        Path::new(env!("CARGO_BIN_EXE_reason")),
        &[
            json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
            json!({
                "jsonrpc":"2.0","id":2,"method":"tools/call",
                "params":{"name":"reason_run","arguments":{"input":input,"candidate":candidate}}
            }),
        ],
    );
    assert_eq!(responses.len(), 2);
    let tools = responses[0]["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 4);
    let result = &responses[1]["result"];
    assert_eq!(result["isError"], false);
    assert_eq!(result["structuredContent"], direct);
    assert_eq!(direct["schema_version"], "reason-cli-output-v1");
    assert_eq!(direct["command"], "run");
    assert!(matches!(
        direct["result"]["outcome"]["verdict"].as_str(),
        Some("accept" | "reject" | "unknown")
    ));
    assert_eq!(
        result["_meta"]["git-ksk/reasoning-harness/native_contract"],
        "reason-cli-output-v1"
    );
}

#[test]
fn mcp_verify_and_schema_preserve_native_product_contracts() {
    let artifact_path = repo_path("examples/artifact.json");
    let artifact = read_json(&artifact_path);
    let direct_verify = native_reason(&[
        "verify",
        artifact_path.to_str().unwrap(),
        "--format",
        "json",
    ]);
    let direct_schema = native_reason(&["schema", "artifact"]);
    let responses = mcp_session(
        Path::new(env!("CARGO_BIN_EXE_reason")),
        &[
            json!({
                "jsonrpc":"2.0","id":"verify","method":"tools/call",
                "params":{"name":"reason_verify","arguments":{"artifact":artifact}}
            }),
            json!({
                "jsonrpc":"2.0","id":"schema","method":"tools/call",
                "params":{"name":"reason_schema","arguments":{"kind":"artifact"}}
            }),
        ],
    );
    assert_eq!(responses[0]["result"]["structuredContent"], direct_verify);
    assert_eq!(responses[1]["result"]["structuredContent"], direct_schema);
    assert_eq!(
        responses[0]["result"]["structuredContent"]["result"]["valid"],
        true
    );
    assert_eq!(
        responses[1]["result"]["structuredContent"]["result"]["contract_id"],
        "reasoning-artifact-v1"
    );
}

#[test]
fn mcp_run_has_no_receipt_config_provider_or_raw_cli_bypass() {
    let input = read_json(&repo_path("examples/input.json"));
    let candidate = read_json(&repo_path("examples/candidate.json"));
    let responses = mcp_session(
        Path::new(env!("CARGO_BIN_EXE_reason")),
        &[json!({
            "jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"reason_run","arguments":{
                "input":input,
                "candidate":candidate,
                "receipts":[],
                "config":"attacker.json",
                "provider":"attacker",
                "args":["--receipts","attacker.json"]
            }}
        })],
    );
    assert_eq!(responses[0]["error"]["code"], -32602);
    assert_eq!(
        responses[0]["error"]["data"]["reasoning_harness"]["operational_kind"],
        "protocol"
    );
}

#[test]
fn mcp_discovery_is_modern_stateless_and_ask_has_no_raw_bypass() {
    let impossible = repo_path("target/definitely-not-a-reason-binary");
    let responses = mcp_session(
        &impossible,
        &[
            json!({"jsonrpc":"2.0","id":"discover","method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}),
            json!({"jsonrpc":"2.0","id":"list","method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}),
            json!({
                "jsonrpc":"2.0","id":"ask","method":"tools/call",
                "params":{"name":"reason_ask","arguments":{
                    "task":"check status",
                    "provider":"google",
                    "model":"model-x",
                    "receipts":[],
                    "config":"attacker.json",
                    "cli_args":["--receipts","attacker.json"]
                }}
            }),
        ],
    );
    assert_eq!(responses[0]["result"]["supportedVersions"][0], "2026-07-28");
    assert_eq!(responses[0]["result"]["ttlMs"], 0);
    assert_eq!(responses[0]["result"]["cacheScope"], "private");
    assert_eq!(
        responses[0]["result"]["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
        "reasoning-harness"
    );
    assert_eq!(responses[1]["result"]["tools"].as_array().unwrap().len(), 4);
    assert_eq!(responses[1]["result"]["ttlMs"], 0);
    assert_eq!(responses[2]["error"]["code"], -32602);
}

#[test]
fn native_spawn_failure_is_tool_operational_failure_not_semantic_unknown() {
    let input = read_json(&repo_path("examples/input.json"));
    let candidate = read_json(&repo_path("examples/candidate.json"));
    let impossible = repo_path("target/definitely-not-a-reason-binary");
    let responses = mcp_session(
        &impossible,
        &[json!({
            "jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"reason_run","arguments":{"input":input,"candidate":candidate}}
        })],
    );
    assert_eq!(responses[0]["result"]["isError"], true);
    assert_eq!(
        responses[0]["result"]["structuredContent"]["schema_version"],
        "reason-mcp-operational-failure-v1"
    );
    assert_eq!(
        responses[0]["result"]["structuredContent"]["failure"]["failure_class"],
        "native_spawn"
    );
    assert!(
        responses[0]["result"]["structuredContent"]
            .get("verdict")
            .is_none()
    );
}
