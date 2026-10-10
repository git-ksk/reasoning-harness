use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use reasoning_harness_core::{
    AcquiredEvidence, AcquiredEvidenceMetadata, ResolutionAdapterError, ResolutionAdapterErrorKind,
    ResolutionCost, ResolutionRequest, ResolutionResolver, ResolutionResolverContribution,
    ResolutionResolverOutput, ResolverClass,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::config_identity::stable_config_id;

pub const MCP_READONLY_RESOLVER_ID: &str = "mcp_readonly_v1";
pub const MCP_PROTOCOL_VERSION: &str = "2026-07-28";
pub const DEFAULT_MCP_RESOLVER_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_MCP_RESOLVER_MAX_RESPONSE_BYTES: usize = 1024 * 1024;

const MCP_CLIENT_NAME: &str = "reasoning-harness";
const MCP_PROVENANCE_META_KEY: &str = "git-ksk/reasoning-harness/provenance";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct McpReadOnlyResolverConfig {
    /// Harness-owned logical identity for the server configuration. This is provenance only.
    pub server_id: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    /// Explicit tool allowlist. The selected tool must be a member.
    pub allowed_tools: BTreeSet<String>,
    pub tool: String,
    /// v0.3.0 supports MCP only as evidence acquisition.
    pub resolver_class: ResolverClass,
    /// Harness-owned fixed tool arguments.
    pub fixed_arguments: BTreeMap<String, Value>,
    /// Optional tool argument that receives stable request/attempt provenance.
    pub provenance_argument: Option<String>,
    /// Harness-owned source identity applied to every acquired result from this tool.
    pub source: String,
    pub timeout_ms: u64,
    pub max_response_bytes: usize,
}

impl McpReadOnlyResolverConfig {
    pub fn with_defaults(
        server_id: impl Into<String>,
        program: PathBuf,
        tool: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        let tool = tool.into();
        Self {
            server_id: server_id.into(),
            program,
            args: vec![],
            allowed_tools: BTreeSet::from([tool.clone()]),
            tool,
            resolver_class: ResolverClass::EvidenceAcquisition,
            fixed_arguments: BTreeMap::new(),
            provenance_argument: None,
            source: source.into(),
            timeout_ms: DEFAULT_MCP_RESOLVER_TIMEOUT_MS,
            max_response_bytes: DEFAULT_MCP_RESOLVER_MAX_RESPONSE_BYTES,
        }
    }
}

#[derive(Debug)]
pub struct McpReadOnlyResolver {
    config: McpReadOnlyResolverConfig,
    config_id: String,
}

impl McpReadOnlyResolver {
    pub fn new(config: McpReadOnlyResolverConfig) -> Self {
        let config_id = stable_config_id(MCP_READONLY_RESOLVER_ID, &config);
        Self { config, config_id }
    }
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(default)]
    result: Option<McpCallToolResult>,
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

#[derive(Debug)]
enum LineReadError {
    Io,
    TooLarge,
    Eof,
}

fn read_one_bounded_line(
    stdout: impl std::io::Read,
    max_bytes: usize,
) -> Result<Vec<u8>, LineReadError> {
    let mut reader = BufReader::new(stdout);
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf().map_err(|_| LineReadError::Io)?;
        if available.is_empty() {
            return if bytes.is_empty() {
                Err(LineReadError::Eof)
            } else {
                Ok(bytes)
            };
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let take = newline.map_or(available.len(), |index| index + 1);
        if bytes.len().saturating_add(take) > max_bytes {
            return Err(LineReadError::TooLarge);
        }
        bytes.extend_from_slice(&available[..take]);
        reader.consume(take);
        if newline.is_some() {
            while bytes
                .last()
                .is_some_and(|byte| *byte == b'\n' || *byte == b'\r')
            {
                bytes.pop();
            }
            return Ok(bytes);
        }
    }
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

fn spawn_error_kind(error: &std::io::Error) -> ResolutionAdapterErrorKind {
    match error.kind() {
        std::io::ErrorKind::NotFound => ResolutionAdapterErrorKind::Unavailable,
        std::io::ErrorKind::PermissionDenied => ResolutionAdapterErrorKind::PermissionDenied,
        _ => ResolutionAdapterErrorKind::Transport,
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

fn opaque_observation(result: &McpCallToolResult) -> String {
    if let Some(structured) = &result.structured_content {
        serde_json::to_string(structured).unwrap_or_else(|_| "mcp structured result".into())
    } else {
        serde_json::to_string(&result.content).unwrap_or_else(|_| "mcp tool result".into())
    }
}

fn contribution_from_result(
    config: &McpReadOnlyResolverConfig,
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
                "mcp:{}:{}:{}:{}",
                config.server_id, config.tool, request.id, attempt_index
            ),
            source: config.source.clone(),
            observation,
            facts,
            acquisition_metadata,
        }],
    }
}

impl ResolutionResolver for McpReadOnlyResolver {
    fn name(&self) -> &'static str {
        MCP_READONLY_RESOLVER_ID
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
        if self.config.server_id.trim().is_empty()
            || self.config.tool.trim().is_empty()
            || self.config.source.trim().is_empty()
            || self.config.resolver_class != ResolverClass::EvidenceAcquisition
            || !self.config.allowed_tools.contains(&self.config.tool)
            || self.config.timeout_ms == 0
            || self.config.max_response_bytes == 0
        {
            return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
        }

        let provenance = json!({
            "request_id": request.id,
            "attempt_index": attempt_index,
            "resolver": MCP_READONLY_RESOLVER_ID,
            "server_id": self.config.server_id,
            "tool": self.config.tool,
        });
        let mut arguments = self.config.fixed_arguments.clone();
        if let Some(argument) = self.config.provenance_argument.as_deref() {
            if argument.trim().is_empty() || arguments.contains_key(argument) {
                return Err(error(ResolutionAdapterErrorKind::PolicyDenied, started));
            }
            arguments.insert(argument.into(), provenance.clone());
        }
        let request_id = format!("reasoning-harness:{}:{}", request.id, attempt_index);
        let payload = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "tools/call",
            "params": {
                "name": self.config.tool,
                "arguments": arguments,
                "_meta": {
                    "io.modelcontextprotocol/protocolVersion": MCP_PROTOCOL_VERSION,
                    "io.modelcontextprotocol/clientCapabilities": {},
                    "io.modelcontextprotocol/clientInfo": {
                        "name": MCP_CLIENT_NAME,
                        "version": env!("CARGO_PKG_VERSION")
                    },
                    MCP_PROVENANCE_META_KEY: provenance
                }
            }
        });
        let mut payload = serde_json::to_vec(&payload)
            .map_err(|_| error(ResolutionAdapterErrorKind::Protocol, started))?;
        payload.push(b'\n');

        let mut child = Command::new(&self.config.program)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|spawn| error(spawn_error_kind(&spawn), started))?;
        let write_result = child
            .stdin
            .as_mut()
            .ok_or_else(|| error(ResolutionAdapterErrorKind::Transport, started))?
            .write_all(&payload);
        drop(child.stdin.take());
        if write_result.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error(ResolutionAdapterErrorKind::Transport, started));
        }

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| error(ResolutionAdapterErrorKind::Transport, started))?;
        let max_response_bytes = self.config.max_response_bytes;
        let (tx, rx) = mpsc::sync_channel(1);
        let reader = thread::spawn(move || {
            let _ = tx.send(read_one_bounded_line(stdout, max_response_bytes));
        });
        let received = rx.recv_timeout(Duration::from_millis(self.config.timeout_ms));
        match received {
            Err(mpsc::RecvTimeoutError::Timeout) => {
                let _ = child.kill();
                let _ = child.wait();
                drop(reader);
                Err(error(ResolutionAdapterErrorKind::Timeout, started))
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                Err(error(ResolutionAdapterErrorKind::Transport, started))
            }
            Ok(line) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = reader.join();
                let line = match line {
                    Ok(line) => line,
                    Err(LineReadError::TooLarge) => {
                        return Err(error(ResolutionAdapterErrorKind::Protocol, started));
                    }
                    Err(LineReadError::Io) | Err(LineReadError::Eof) => {
                        return Err(error(ResolutionAdapterErrorKind::Transport, started));
                    }
                };
                let response: JsonRpcResponse = serde_json::from_slice(&line)
                    .map_err(|_| error(ResolutionAdapterErrorKind::Protocol, started))?;
                if response.jsonrpc != "2.0" || response.id != Value::String(request_id) {
                    return Err(error(ResolutionAdapterErrorKind::Protocol, started));
                }
                match (response.result, response.error) {
                    (Some(result), None) if result.is_error.unwrap_or(false) => {
                        Err(error(ResolutionAdapterErrorKind::ToolExecution, started))
                    }
                    (Some(result), None) => Ok(ResolutionResolverOutput {
                        contribution: contribution_from_result(
                            &self.config,
                            request,
                            attempt_index,
                            result,
                        ),
                        cost: measured_cost(started),
                    }),
                    (None, Some(rpc_error)) => Err(error(rpc_error_kind(&rpc_error), started)),
                    _ => Err(error(ResolutionAdapterErrorKind::Protocol, started)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::{fs, os::unix::fs::PermissionsExt};

    use reasoning_harness_core::{
        EvidenceAdmissionPolicy, EvidenceAuthorityPolicy, EvidenceRequirement,
        GroundedResolutionPolicy, GroundedResolutionRuntime, HarnessInput, HarnessOutcome,
        Proposition, ResolutionPlanner, ResolutionReason, ResolutionRequestBudget,
        ResolutionTarget, StandardGroundingPipeline, Verdict,
    };

    use super::*;
    use crate::{
        ExternalEvidenceAdmissionConfig, ExternalEvidenceAdmissionPolicy,
        ExternalEvidenceSourcePolicy,
    };

    fn request() -> ResolutionRequest {
        ResolutionRequest {
            id: "resolution:service.region".into(),
            reason: reasoning_harness_core::ResolutionReason::MissingSupport,
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

    #[cfg(unix)]
    fn script(body: &str, name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "reason-mcp-readonly-{}-{name}.sh",
            std::process::id()
        ));
        fs::write(&path, body).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(unix)]
    #[test]
    fn modern_stdio_call_carries_protocol_and_stable_provenance() {
        let path = script(
            r#"#!/bin/sh
read request
printf '%s' "$request" | grep -q '"method":"tools/call"' || exit 2
printf '%s' "$request" | grep -q '"io.modelcontextprotocol/protocolVersion":"2026-07-28"' || exit 3
printf '%s' "$request" | grep -q '"git-ksk/reasoning-harness/provenance"' || exit 4
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0","result":{"content":[{"type":"text","text":"opaque lookup"}]}}'
"#,
            "modern",
        );
        let resolver = McpReadOnlyResolver::new(McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            path.clone(),
            "lookup",
            "mcp:fixture:lookup",
        ));
        let output = resolver.resolve(&request(), 0).unwrap();
        fs::remove_file(path).ok();
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                assert_eq!(evidence.len(), 1);
                assert_eq!(evidence[0].source, "mcp:fixture:lookup");
                assert!(evidence[0].facts.is_empty());
            }
            other => panic!("expected acquired evidence, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn acquisition_envelope_is_raw_data_and_reverification_still_owns_support() {
        const REQUEST_ID: &str = "mcp-admission-request";
        let path = script(
            r#"#!/bin/sh
read request
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:mcp-admission-request:0","result":{"content":[{"type":"text","text":"service.region=eu-west-1"}],"structuredContent":{"reasoning_harness":{"observation":"service.region=eu-west-1","facts":{"service.region":"eu-west-1"},"acquisition_metadata":{"observed_at_unix_seconds":980,"retrieved_at_unix_seconds":990,"claimed_authority_class":"primary"}}}}}'
"#,
            "admission",
        );
        let resolver = McpReadOnlyResolver::new(McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            path.clone(),
            "lookup",
            "mcp:fixture:lookup",
        ));
        let admission = ExternalEvidenceAdmissionPolicy::new(ExternalEvidenceAdmissionConfig {
            resolver_name: MCP_READONLY_RESOLVER_ID,
            evaluation_time_unix_seconds: 1_000,
            authority_policy: EvidenceAuthorityPolicy {
                ranks: BTreeMap::from([("primary".into(), 10)]),
            },
            minimum_authority_class: Some("primary".into()),
            required_scope: None,
            sources: BTreeMap::from([(
                "mcp:fixture:lookup".into(),
                ExternalEvidenceSourcePolicy {
                    authority_class: "primary".into(),
                    max_age_seconds: 60,
                    scope: None,
                },
            )]),
        });
        let proposition = Proposition {
            key: "service.region".into(),
            value: "eu-west-1".into(),
        };
        let requirement = EvidenceRequirement {
            proposition: proposition.clone(),
            as_of_unix_seconds: Some(1_000),
            scope: None,
            minimum_authority_class: Some("primary".into()),
        };
        struct FixedPlanner {
            requirement: EvidenceRequirement,
        }
        impl ResolutionPlanner for FixedPlanner {
            fn plan(
                &self,
                outcome: &HarnessOutcome,
                _policy: &GroundedResolutionPolicy,
            ) -> Vec<ResolutionRequest> {
                if outcome.verdict != Verdict::Unknown {
                    return vec![];
                }
                vec![ResolutionRequest {
                    id: REQUEST_ID.into(),
                    reason: ResolutionReason::EvidenceQualification,
                    target: ResolutionTarget::EvidenceQualification {
                        requirement: self.requirement.clone(),
                    },
                    resolver_class: ResolverClass::EvidenceAcquisition,
                    budget: ResolutionRequestBudget::default(),
                }]
            }
        }
        let input = HarnessInput {
            task: "determine region".into(),
            evidence: vec![],
            hypotheses: vec![proposition],
            assumptions: vec![],
            evidence_requirements: vec![requirement.clone()],
            authority_policy: admission.authority_policy().clone(),
        };
        let planner = FixedPlanner { requirement };
        let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
        let runtime = GroundedResolutionRuntime {
            pipeline: &StandardGroundingPipeline,
            planner: &planner,
            evidence_admission: &admission,
            resolvers: &resolvers,
            trusted_verifiers: &[],
            renderer: &reasoning_harness_core::CanonicalFinalAnswerRenderer,
        };
        let mut policy = GroundedResolutionPolicy::default();
        policy.budget.required_authority_class = Some("primary".into());
        let outcome = runtime
            .run(
                input,
                reasoning_harness_core::ReasoningCandidate::default(),
                &policy,
            )
            .unwrap();
        fs::remove_file(path).ok();
        assert_eq!(outcome.initial_verdict, Verdict::Unknown);
        assert_eq!(outcome.final_verdict, Verdict::Accept);
        assert_eq!(outcome.attempts.len(), 1);
        assert_eq!(outcome.attempts[0].adapter_name, MCP_READONLY_RESOLVER_ID);
        assert!(outcome.attempts[0].adapter_config_id.is_some());
        assert_eq!(
            outcome.attempts[0].admission_policy_id,
            admission.identity().map(str::to_string)
        );
        assert!(!outcome.final_artifact.verification_receipts.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn opaque_or_tool_error_results_never_self_promote() {
        let opaque_path = script(
            r#"#!/bin/sh
read request
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0","result":{"content":[{"type":"text","text":"probably eu-west-1"}]}}'
"#,
            "opaque",
        );
        let opaque = McpReadOnlyResolver::new(McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            opaque_path.clone(),
            "lookup",
            "mcp:fixture:lookup",
        ));
        let output = opaque.resolve(&request(), 0).unwrap();
        fs::remove_file(opaque_path).ok();
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                assert!(evidence[0].facts.is_empty());
                assert_eq!(
                    evidence[0].acquisition_metadata,
                    AcquiredEvidenceMetadata::default()
                );
            }
            other => panic!("expected acquired evidence, got {other:?}"),
        }

        let error_path = script(
            r#"#!/bin/sh
read request
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0","result":{"content":[{"type":"text","text":"backend denied"}],"isError":true}}'
"#,
            "tool-error",
        );
        let failed = McpReadOnlyResolver::new(McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            error_path.clone(),
            "lookup",
            "mcp:fixture:lookup",
        ));
        let failure = failed.resolve(&request(), 0).unwrap_err();
        fs::remove_file(error_path).ok();
        assert_eq!(failure.kind, ResolutionAdapterErrorKind::ToolExecution);
    }

    #[cfg(unix)]
    #[test]
    fn allowlist_timeout_and_protocol_errors_fail_closed() {
        let path = script("#!/bin/sh\nread request\nsleep 1\n", "timeout");
        let mut config = McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            path.clone(),
            "lookup",
            "mcp:fixture:lookup",
        );
        config.timeout_ms = 30;
        let resolver = McpReadOnlyResolver::new(config);
        assert_eq!(
            resolver.resolve(&request(), 0).unwrap_err().kind,
            ResolutionAdapterErrorKind::Timeout
        );
        fs::remove_file(path).ok();

        let mut denied = McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            PathBuf::from("unused"),
            "lookup",
            "mcp:fixture:lookup",
        );
        denied.allowed_tools.clear();
        assert_eq!(
            McpReadOnlyResolver::new(denied)
                .resolve(&request(), 0)
                .unwrap_err()
                .kind,
            ResolutionAdapterErrorKind::PolicyDenied
        );
    }
}
