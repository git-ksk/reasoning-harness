use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::Command,
    time::{Duration, Instant},
};

use reasoning_harness_core::{
    AcquiredEvidence, AcquiredEvidenceMetadata, ReasoningCandidate, ResolutionAdapterError,
    ResolutionAdapterErrorKind, ResolutionCost, ResolutionRequest, ResolutionResolver,
    ResolutionResolverContribution, ResolutionResolverOutput, ResolverClass,
};
use serde::{Deserialize, Serialize};

use crate::{
    SubprocessCancellation, config_identity::stable_config_id, subprocess_deadline::run_to_exit,
    subprocess_environment::isolate_subprocess_environment,
};

pub const EXTERNAL_COMMAND_RESOLVER_ID: &str = "external_command_v1";
pub const INVESTIGATION_EXTERNAL_COMMAND_RESOLVER_ID: &str = "investigation_external_command_v1";
pub const EXTERNAL_RESOLVER_REQUEST_SCHEMA: &str = "reason-external-resolver-request-v1";
pub const INVESTIGATION_EXTERNAL_RESOLVER_REQUEST_SCHEMA: &str =
    "reason-investigation-external-resolver-request-v1";
pub const EXTERNAL_RESOLVER_RESPONSE_SCHEMA: &str = "reason-external-resolver-response-v1";
pub const DEFAULT_EXTERNAL_RESOLVER_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_EXTERNAL_RESOLVER_MAX_RESPONSE_BYTES: usize = 1024 * 1024;

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
    cancellation: Option<SubprocessCancellation>,
}

impl ExternalCommandResolver {
    pub fn new(config: ExternalCommandResolverConfig) -> Self {
        let config_id = stable_config_id(EXTERNAL_COMMAND_RESOLVER_ID, &config);
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
}

#[derive(Debug)]
pub struct InvestigationExternalCommandResolver {
    config: ExternalCommandResolverConfig,
    config_id: String,
    cancellation: Option<SubprocessCancellation>,
}

impl InvestigationExternalCommandResolver {
    pub fn new(config: ExternalCommandResolverConfig) -> Self {
        let config_id = stable_config_id(INVESTIGATION_EXTERNAL_COMMAND_RESOLVER_ID, &config);
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
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct ExternalResolverRequestEnvelope<'a> {
    schema_version: &'static str,
    adapter_id: &'static str,
    attempt_index: usize,
    request: &'a ResolutionRequest,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct InvestigationExternalResolverRequestEnvelope<'a> {
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

fn execute_external_payload(
    config: &ExternalCommandResolverConfig,
    payload: Vec<u8>,
    started: Instant,
    cancellation: Option<&SubprocessCancellation>,
) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
    let mut command = Command::new(&config.program);
    command.args(&config.args);
    isolate_subprocess_environment(&mut command);
    let output = run_to_exit(
        &mut command,
        payload,
        started,
        Duration::from_millis(config.timeout_ms),
        config.max_response_bytes,
        cancellation,
    )
    .map_err(|kind| adapter_error(kind, started, ResolutionCost::default()))?;

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
        (None, Some(failure)) => Err(adapter_error(failure.kind.into(), started, response.cost)),
        _ => Err(adapter_error(
            ResolutionAdapterErrorKind::Protocol,
            started,
            response.cost,
        )),
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

        execute_external_payload(&self.config, payload, started, self.cancellation.as_ref())
    }
}

impl ResolutionResolver for InvestigationExternalCommandResolver {
    fn name(&self) -> &'static str {
        INVESTIGATION_EXTERNAL_COMMAND_RESOLVER_ID
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
        if !matches!(
            request.target,
            reasoning_harness_core::ResolutionTarget::InvestigationQuestion { .. }
        ) {
            return Err(adapter_error(
                ResolutionAdapterErrorKind::PolicyDenied,
                started,
                ResolutionCost::default(),
            ));
        }
        let payload = serde_json::to_vec(&InvestigationExternalResolverRequestEnvelope {
            schema_version: INVESTIGATION_EXTERNAL_RESOLVER_REQUEST_SCHEMA,
            adapter_id: INVESTIGATION_EXTERNAL_COMMAND_RESOLVER_ID,
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
        execute_external_payload(&self.config, payload, started, self.cancellation.as_ref())
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
[ -z "${REASON_SUBPROCESS_SENTINEL_SECRET+x}" ] || exit 90
[ -n "${PATH:-}" ] || exit 91
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
    fn investigation_adapter_uses_separate_request_identity() {
        use reasoning_harness_core::{ResolutionReason, ResolutionRequestBudget, ResolutionTarget};
        let path = test_script(
            "investigation-request",
            r#"request=$(cat)
printf '%s' "$request" | grep -q '"schema_version":"reason-investigation-external-resolver-request-v1"' || exit 2
printf '%s' "$request" | grep -q '"adapter_id":"investigation_external_command_v1"' || exit 3
printf '%s' "$request" | grep -q '"kind":"investigation_question"' || exit 4
printf '%s' '{"schema_version":"reason-external-resolver-response-v1","contribution":{"kind":"acquired_evidence","evidence":[{"id":"ext-investigation-1","source":"api:test","observation":"service.region=eu-west-1","facts":{"service.region":"eu-west-1"}}]}}'"#,
        );
        let resolver = InvestigationExternalCommandResolver::new(
            ExternalCommandResolverConfig::with_defaults(path.clone(), vec![]),
        );
        let request = ResolutionRequest {
            id: "investigation:region:api".into(),
            reason: ResolutionReason::Investigation,
            target: ResolutionTarget::InvestigationQuestion {
                target_id: "region".into(),
                question: "Which region serves the deployment?".into(),
                expected_fact_key: Some("service.region".into()),
            },
            resolver_class: ResolverClass::EvidenceAcquisition,
            budget: ResolutionRequestBudget::default(),
        };
        let output = resolver.resolve(&request, 0).unwrap();
        std::fs::remove_file(path).ok();
        assert_eq!(resolver.name(), INVESTIGATION_EXTERNAL_COMMAND_RESOLVER_ID);
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
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
    fn large_request_write_to_non_reader_respects_whole_invocation_timeout() {
        let path = test_script("blocked-write", "sleep 1");
        let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig {
            program: path.clone(),
            args: vec![],
            timeout_ms: 40,
            max_response_bytes: 4096,
        });
        let mut request = test_request();
        if let reasoning_harness_core::ResolutionTarget::Proposition { proposition } =
            &mut request.target
        {
            proposition.value = "x".repeat(2 * 1024 * 1024);
        }
        let wall = Instant::now();
        let error = resolver.resolve(&request, 0).unwrap_err();
        std::fs::remove_file(path).ok();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Timeout);
        assert!(wall.elapsed() < Duration::from_millis(500));
    }

    #[cfg(unix)]
    #[test]
    fn oversized_or_non_json_response_is_protocol_failure() {
        let path = test_script("oversized", "cat >/dev/null\nprintf '%0500d' 0");
        let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig {
            program: path.clone(),
            args: vec![],
            // This test verifies response-size classification, not timeout behavior. Keep the
            // deadline generous enough that full-workspace parallel scheduling cannot turn the
            // intended Protocol result into a legitimate Timeout.
            timeout_ms: 10_000,
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
