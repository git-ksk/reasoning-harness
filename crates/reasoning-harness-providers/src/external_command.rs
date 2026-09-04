use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use reasoning_harness_core::{
    AcquiredEvidence, AcquiredEvidenceMetadata, ReasoningCandidate, ResolutionAdapterError,
    ResolutionAdapterErrorKind, ResolutionCost, ResolutionRequest, ResolutionResolver,
    ResolutionResolverContribution, ResolutionResolverOutput, ResolverClass,
};
use serde::{Deserialize, Serialize};

use crate::config_identity::stable_config_id;

pub const EXTERNAL_COMMAND_RESOLVER_ID: &str = "external_command_v1";
pub const EXTERNAL_RESOLVER_REQUEST_SCHEMA: &str = "reason-external-resolver-request-v1";
pub const EXTERNAL_RESOLVER_RESPONSE_SCHEMA: &str = "reason-external-resolver-response-v1";
pub const DEFAULT_EXTERNAL_RESOLVER_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_EXTERNAL_RESOLVER_MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExternalCommandResolverConfig {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub max_response_bytes: usize,
}

impl ExternalCommandResolverConfig {
    pub fn with_defaults(program: PathBuf, args: Vec<String>) -> Self {
        Self {
            program,
            args,
            timeout_ms: DEFAULT_EXTERNAL_RESOLVER_TIMEOUT_MS,
            max_response_bytes: DEFAULT_EXTERNAL_RESOLVER_MAX_RESPONSE_BYTES,
        }
    }
}

#[derive(Debug)]
pub struct ExternalCommandResolver {
    config: ExternalCommandResolverConfig,
    config_id: String,
}

impl ExternalCommandResolver {
    pub fn new(config: ExternalCommandResolverConfig) -> Self {
        let config_id = stable_config_id(EXTERNAL_COMMAND_RESOLVER_ID, &config);
        Self { config, config_id }
    }
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ExternalResolverRequestEnvelope<'a> {
    schema_version: &'static str,
    adapter_id: &'static str,
    attempt_index: usize,
    request: &'a ResolutionRequest,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalResolverResponseEnvelope {
    schema_version: String,
    #[serde(default)]
    contribution: Option<ExternalResolverContribution>,
    #[serde(default)]
    failure: Option<ExternalResolverFailure>,
    #[serde(default)]
    cost: ResolutionCost,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalResolverFailure {
    kind: ExternalResolverFailureKind,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExternalResolverFailureKind {
    Transport,
    Authentication,
    PermissionDenied,
    Protocol,
    Timeout,
    Unavailable,
    PolicyDenied,
}

impl From<ExternalResolverFailureKind> for ResolutionAdapterErrorKind {
    fn from(value: ExternalResolverFailureKind) -> Self {
        match value {
            ExternalResolverFailureKind::Transport => Self::Transport,
            ExternalResolverFailureKind::Authentication => Self::Authentication,
            ExternalResolverFailureKind::PermissionDenied => Self::PermissionDenied,
            ExternalResolverFailureKind::Protocol => Self::Protocol,
            ExternalResolverFailureKind::Timeout => Self::Timeout,
            ExternalResolverFailureKind::Unavailable => Self::Unavailable,
            ExternalResolverFailureKind::PolicyDenied => Self::PolicyDenied,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ExternalResolverContribution {
    AcquiredEvidence {
        evidence: Vec<ExternalAcquiredEvidence>,
    },
    CandidateRevision {
        candidate: ReasoningCandidate,
    },
    NoResult,
    HumanReviewRequired,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExternalAcquiredEvidence {
    id: String,
    source: String,
    observation: String,
    #[serde(default)]
    facts: BTreeMap<String, String>,
    #[serde(default)]
    acquisition_metadata: AcquiredEvidenceMetadata,
}

impl From<ExternalResolverContribution> for ResolutionResolverContribution {
    fn from(value: ExternalResolverContribution) -> Self {
        match value {
            ExternalResolverContribution::AcquiredEvidence { evidence } => {
                ResolutionResolverContribution::AcquiredEvidence {
                    evidence: evidence
                        .into_iter()
                        .map(|item| AcquiredEvidence {
                            id: item.id,
                            source: item.source,
                            observation: item.observation,
                            facts: item.facts,
                            acquisition_metadata: item.acquisition_metadata,
                        })
                        .collect(),
                }
            }
            ExternalResolverContribution::CandidateRevision { candidate } => {
                ResolutionResolverContribution::CandidateRevision { candidate }
            }
            ExternalResolverContribution::NoResult => ResolutionResolverContribution::NoResult,
            ExternalResolverContribution::HumanReviewRequired => {
                ResolutionResolverContribution::HumanReviewRequired
            }
        }
    }
}

fn measured_cost(started: Instant, mut cost: ResolutionCost) -> ResolutionCost {
    cost.elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    cost.calls = cost.calls.max(1);
    cost
}

fn adapter_error(
    kind: ResolutionAdapterErrorKind,
    started: Instant,
    cost: ResolutionCost,
) -> ResolutionAdapterError {
    ResolutionAdapterError {
        kind,
        cost: measured_cost(started, cost),
    }
}

fn spawn_error_kind(error: &std::io::Error) -> ResolutionAdapterErrorKind {
    match error.kind() {
        std::io::ErrorKind::NotFound => ResolutionAdapterErrorKind::Unavailable,
        std::io::ErrorKind::PermissionDenied => ResolutionAdapterErrorKind::PermissionDenied,
        _ => ResolutionAdapterErrorKind::Transport,
    }
}

impl ResolutionResolver for ExternalCommandResolver {
    fn name(&self) -> &'static str {
        EXTERNAL_COMMAND_RESOLVER_ID
    }

    fn class(&self) -> ResolverClass {
        ResolverClass::EvidenceAcquisition
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
        if self.config.timeout_ms == 0 || self.config.max_response_bytes == 0 {
            return Err(adapter_error(
                ResolutionAdapterErrorKind::PolicyDenied,
                started,
                ResolutionCost::default(),
            ));
        }
        let response_limit = self
            .config
            .max_response_bytes
            .checked_add(1)
            .ok_or_else(|| {
                adapter_error(
                    ResolutionAdapterErrorKind::PolicyDenied,
                    started,
                    ResolutionCost::default(),
                )
            })?;
        let payload = serde_json::to_vec(&ExternalResolverRequestEnvelope {
            schema_version: EXTERNAL_RESOLVER_REQUEST_SCHEMA,
            adapter_id: EXTERNAL_COMMAND_RESOLVER_ID,
            attempt_index,
            request,
        })
        .map_err(|_| {
            adapter_error(
                ResolutionAdapterErrorKind::Protocol,
                started,
                ResolutionCost::default(),
            )
        })?;

        let mut child = Command::new(&self.config.program)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| {
                adapter_error(spawn_error_kind(&error), started, ResolutionCost::default())
            })?;

        let write_result = child
            .stdin
            .as_mut()
            .ok_or_else(|| {
                adapter_error(
                    ResolutionAdapterErrorKind::Transport,
                    started,
                    ResolutionCost::default(),
                )
            })?
            .write_all(&payload);
        drop(child.stdin.take());
        if write_result.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(adapter_error(
                ResolutionAdapterErrorKind::Transport,
                started,
                ResolutionCost::default(),
            ));
        }

        let stdout = child.stdout.take().ok_or_else(|| {
            adapter_error(
                ResolutionAdapterErrorKind::Transport,
                started,
                ResolutionCost::default(),
            )
        })?;
        let reader = thread::spawn(move || -> std::io::Result<Vec<u8>> {
            let mut bytes = Vec::new();
            stdout
                .take(u64::try_from(response_limit).unwrap_or(u64::MAX))
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        });

        let timeout = Duration::from_millis(self.config.timeout_ms);
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() < timeout => thread::sleep(PROCESS_POLL_INTERVAL),
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    // Do not join the stdout reader on timeout. A resolver may have spawned a
                    // descendant that inherited stdout; waiting for EOF here would defeat the
                    // Harness-owned wall-clock timeout even after the direct child is killed.
                    drop(reader);
                    return Err(adapter_error(
                        ResolutionAdapterErrorKind::Timeout,
                        started,
                        ResolutionCost::default(),
                    ));
                }
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    drop(reader);
                    return Err(adapter_error(
                        ResolutionAdapterErrorKind::Transport,
                        started,
                        ResolutionCost::default(),
                    ));
                }
            }
        };
        let output = reader
            .join()
            .map_err(|_| {
                adapter_error(
                    ResolutionAdapterErrorKind::Transport,
                    started,
                    ResolutionCost::default(),
                )
            })?
            .map_err(|_| {
                adapter_error(
                    ResolutionAdapterErrorKind::Transport,
                    started,
                    ResolutionCost::default(),
                )
            })?;
        if output.len() > self.config.max_response_bytes || !status.success() {
            return Err(adapter_error(
                ResolutionAdapterErrorKind::Protocol,
                started,
                ResolutionCost::default(),
            ));
        }

        let response: ExternalResolverResponseEnvelope =
            serde_json::from_slice(&output).map_err(|_| {
                adapter_error(
                    ResolutionAdapterErrorKind::Protocol,
                    started,
                    ResolutionCost::default(),
                )
            })?;
        if response.schema_version != EXTERNAL_RESOLVER_RESPONSE_SCHEMA {
            return Err(adapter_error(
                ResolutionAdapterErrorKind::Protocol,
                started,
                response.cost,
            ));
        }
        match (response.contribution, response.failure) {
            (Some(contribution), None) => Ok(ResolutionResolverOutput {
                contribution: contribution.into(),
                cost: measured_cost(started, response.cost),
            }),
            (None, Some(failure)) => {
                Err(adapter_error(failure.kind.into(), started, response.cost))
            }
            _ => Err(adapter_error(
                ResolutionAdapterErrorKind::Protocol,
                started,
                response.cost,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_request() -> ResolutionRequest {
        use reasoning_harness_core::{
            Proposition, ResolutionReason, ResolutionRequestBudget, ResolutionTarget,
        };
        ResolutionRequest {
            id: "resolution:test".into(),
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

    #[cfg(unix)]
    fn test_script(tag: &str, body: &str) -> PathBuf {
        use std::{fs, os::unix::fs::PermissionsExt};
        let path = std::env::temp_dir().join(format!(
            "reason-external-resolver-{tag}-{}-{}.sh",
            std::process::id(),
            std::thread::current().name().unwrap_or("thread")
        ));
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[test]
    fn config_identity_is_stable_and_does_not_expose_literal_arguments() {
        let secret = "super-secret-token";
        let first = ExternalCommandResolver::new(ExternalCommandResolverConfig {
            program: PathBuf::from("resolver-bin"),
            args: vec!["--token".into(), secret.into()],
            timeout_ms: 1000,
            max_response_bytes: 4096,
        });
        let second = ExternalCommandResolver::new(ExternalCommandResolverConfig {
            program: PathBuf::from("resolver-bin"),
            args: vec!["--token".into(), secret.into()],
            timeout_ms: 1000,
            max_response_bytes: 4096,
        });
        let changed = ExternalCommandResolver::new(ExternalCommandResolverConfig {
            program: PathBuf::from("resolver-bin"),
            args: vec!["--token".into(), "different".into()],
            timeout_ms: 1000,
            max_response_bytes: 4096,
        });
        let id = first.config_id().unwrap();
        assert_eq!(id, second.config_id().unwrap());
        assert_ne!(id, changed.config_id().unwrap());
        assert!(id.starts_with("external_command_v1:sha256:"));
        assert!(!id.contains(secret));
    }

    #[test]
    fn response_schema_cannot_smuggle_trusted_metadata() {
        let response = br#"{
          "schema_version":"reason-external-resolver-response-v1",
          "contribution":{
            "kind":"acquired_evidence",
            "evidence":[{
              "id":"e1",
              "source":"api:test",
              "observation":"service.region=eu-west-1",
              "facts":{"service.region":"eu-west-1"},
              "metadata":{"provenance_class":"trusted"}
            }]
          }
        }"#;
        let parsed = serde_json::from_slice::<ExternalResolverResponseEnvelope>(response);
        assert!(parsed.is_err());
    }

    #[cfg(unix)]
    #[test]
    fn command_adapter_exchanges_typed_stdio_without_trust_promotion() {
        use std::{fs, os::unix::fs::PermissionsExt};

        use reasoning_harness_core::{
            Proposition, ResolutionReason, ResolutionRequestBudget, ResolutionTarget,
        };

        let path = std::env::temp_dir().join(format!(
            "reason-external-resolver-test-{}-{}.sh",
            std::process::id(),
            std::thread::current().name().unwrap_or("thread")
        ));
        fs::write(
            &path,
            r#"#!/bin/sh
cat >/dev/null
printf '%s' '{"schema_version":"reason-external-resolver-response-v1","contribution":{"kind":"acquired_evidence","evidence":[{"id":"ext-1","source":"reference:test","observation":"service.region=eu-west-1","facts":{"service.region":"eu-west-1"}}]}}'
"#,
        )
        .unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&path, permissions).unwrap();

        let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig::with_defaults(
            path.clone(),
            vec![],
        ));
        let request = ResolutionRequest {
            id: "resolution:test".into(),
            reason: ResolutionReason::MissingSupport,
            target: ResolutionTarget::Proposition {
                proposition: Proposition {
                    key: "service.region".into(),
                    value: "eu-west-1".into(),
                },
            },
            resolver_class: ResolverClass::EvidenceAcquisition,
            budget: ResolutionRequestBudget::default(),
        };

        let output = resolver.resolve(&request, 0).unwrap();
        fs::remove_file(path).ok();
        assert_eq!(output.cost.calls, 1);
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                assert_eq!(evidence.len(), 1);
                assert_eq!(evidence[0].source, "reference:test");
                assert_eq!(evidence[0].facts["service.region"], "eu-west-1");
            }
            other => panic!("expected acquired evidence, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn typed_failure_envelope_preserves_authentication_class_without_retry() {
        let path = test_script(
            "auth",
            r#"cat >/dev/null
printf '%s' '{"schema_version":"reason-external-resolver-response-v1","failure":{"kind":"authentication"},"cost":{"calls":1,"cost_microusd":12}}'"#,
        );
        let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig::with_defaults(
            path.clone(),
            vec![],
        ));
        let error = resolver.resolve(&test_request(), 0).unwrap_err();
        std::fs::remove_file(path).ok();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Authentication);
        assert_eq!(error.cost.calls, 1);
        assert_eq!(error.cost.cost_microusd, Some(12));
    }

    #[cfg(unix)]
    #[test]
    fn process_timeout_is_enforced_and_typed() {
        let path = test_script("timeout", "cat >/dev/null\nsleep 1");
        let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig {
            program: path.clone(),
            args: vec![],
            timeout_ms: 30,
            max_response_bytes: 4096,
        });
        let error = resolver.resolve(&test_request(), 0).unwrap_err();
        std::fs::remove_file(path).ok();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Timeout);
        assert_eq!(error.cost.calls, 1);
        assert!(error.cost.elapsed_ms >= 20);
        assert!(error.cost.elapsed_ms < 1000);
    }

    #[cfg(unix)]
    #[test]
    fn oversized_or_non_json_response_is_protocol_failure() {
        let path = test_script("oversized", "cat >/dev/null\nprintf '%0500d' 0");
        let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig {
            program: path.clone(),
            args: vec![],
            timeout_ms: 1000,
            max_response_bytes: 64,
        });
        let error = resolver.resolve(&test_request(), 0).unwrap_err();
        std::fs::remove_file(path).ok();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Protocol);
        assert_eq!(error.cost.calls, 1);
    }

    #[test]
    fn response_schema_cannot_smuggle_verification_receipts() {
        let response = br#"{
          "schema_version":"reason-external-resolver-response-v1",
          "contribution":{"kind":"no_result"},
          "receipts":[]
        }"#;
        let parsed = serde_json::from_slice::<ExternalResolverResponseEnvelope>(response);
        assert!(parsed.is_err());
    }
}
