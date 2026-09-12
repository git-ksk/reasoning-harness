use std::{
    collections::BTreeMap,
    process::Command,
    time::{Duration, Instant},
};

use reasoning_harness_core::{
    AcquiredEvidence, AcquiredEvidenceMetadata, ResolutionAdapterError, ResolutionAdapterErrorKind,
    ResolutionCost, ResolutionRequest, ResolutionResolver, ResolutionResolverContribution,
    ResolutionResolverOutput, ResolverClass,
};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    config_identity::stable_config_id,
    mcp_readonly::{MCP_PROTOCOL_VERSION, McpReadOnlyResolverConfig},
    subprocess_deadline::run_until_line,
    subprocess_environment::isolate_subprocess_environment,
};

pub const MCP_READONLY_V2_RESOLVER_ID: &str = "mcp_readonly_v2";

const MCP_CLIENT_NAME: &str = "reasoning-harness";
const MCP_PROVENANCE_META_KEY: &str = "git-ksk/reasoning-harness/provenance";

#[derive(Debug)]
/// v0.4 operational successor that preserves the v1 stateless/read-only semantics while
/// applying the shared whole-invocation subprocess deadline. The frozen `mcp_readonly_v1`
/// implementation remains untouched for historical replay/evaluation compatibility.
pub struct McpReadOnlyResolverV2 {
    config: McpReadOnlyResolverConfig,
    config_id: String,
}

impl McpReadOnlyResolverV2 {
    pub fn new(config: McpReadOnlyResolverConfig) -> Self {
        let config_id = stable_config_id(MCP_READONLY_V2_RESOLVER_ID, &config);
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

impl ResolutionResolver for McpReadOnlyResolverV2 {
    fn name(&self) -> &'static str {
        MCP_READONLY_V2_RESOLVER_ID
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
            "resolver": MCP_READONLY_V2_RESOLVER_ID,
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

        let mut command = Command::new(&self.config.program);
        command.args(&self.config.args);
        isolate_subprocess_environment(&mut command);
        let line = run_until_line(
            &mut command,
            payload,
            started,
            Duration::from_millis(self.config.timeout_ms),
            self.config.max_response_bytes,
            None,
        )
        .map_err(|kind| error(kind, started))?;
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

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf};

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
            "reason-mcp-readonly-v2-{}-{name}.sh",
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
[ -z "${REASON_SUBPROCESS_SENTINEL_SECRET+x}" ] || exit 90
[ -n "${PATH:-}" ] || exit 91
read request
printf '%s' "$request" | grep -q '"method":"tools/call"' || exit 2
printf '%s' "$request" | grep -q '"io.modelcontextprotocol/protocolVersion":"2026-07-28"' || exit 3
printf '%s' "$request" | grep -q '"git-ksk/reasoning-harness/provenance"' || exit 4
printf '%s\n' '{"jsonrpc":"2.0","id":"reasoning-harness:resolution:service.region:0","result":{"content":[{"type":"text","text":"opaque lookup"}]}}'
"#,
            "modern",
        );
        let resolver = McpReadOnlyResolverV2::new(McpReadOnlyResolverConfig::with_defaults(
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
        let resolver = McpReadOnlyResolverV2::new(McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            path.clone(),
            "lookup",
            "mcp:fixture:lookup",
        ));
        let admission = ExternalEvidenceAdmissionPolicy::new(ExternalEvidenceAdmissionConfig {
            resolver_name: MCP_READONLY_V2_RESOLVER_ID,
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
        assert_eq!(
            outcome.attempts[0].adapter_name,
            MCP_READONLY_V2_RESOLVER_ID
        );
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
        let opaque = McpReadOnlyResolverV2::new(McpReadOnlyResolverConfig::with_defaults(
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
        let failed = McpReadOnlyResolverV2::new(McpReadOnlyResolverConfig::with_defaults(
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
    fn large_request_write_to_non_reader_respects_whole_invocation_timeout() {
        let path = script("#!/bin/sh\nsleep 1\n", "blocked-write");
        let mut config = McpReadOnlyResolverConfig::with_defaults(
            "fixture-server",
            path.clone(),
            "lookup",
            "mcp:fixture:lookup",
        );
        config.timeout_ms = 40;
        config
            .fixed_arguments
            .insert("large".into(), Value::String("x".repeat(2 * 1024 * 1024)));
        let resolver = McpReadOnlyResolverV2::new(config);
        let wall = Instant::now();
        let error = resolver.resolve(&request(), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Timeout);
        assert!(wall.elapsed() < Duration::from_millis(500));
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
        let resolver = McpReadOnlyResolverV2::new(config);
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
            McpReadOnlyResolverV2::new(denied)
                .resolve(&request(), 0)
                .unwrap_err()
                .kind,
            ResolutionAdapterErrorKind::PolicyDenied
        );
    }
}
