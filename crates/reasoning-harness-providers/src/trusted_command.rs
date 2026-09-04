use std::{
    collections::BTreeSet,
    io::{Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use reasoning_harness_core::{
    Evidence, EvidenceAuthorityPolicy, EvidenceQualificationInspector, EvidenceQualificationStatus,
    EvidenceRequirement, ReasoningArtifact, ResolutionAdapterError, ResolutionAdapterErrorKind,
    ResolutionCost, ResolutionRequest, TrustedResolutionVerifier, TrustedVerifierResolutionOutput,
    VerificationConclusion, VerificationReceipt,
};
use serde::{Deserialize, Serialize};

use crate::config_identity::stable_config_id;

pub const TRUSTED_COMMAND_VERIFIER_ID: &str = "trusted_command_verifier_v1";
pub const TRUSTED_COMMAND_REQUEST_SCHEMA: &str = "reason-trusted-verifier-request-v1";
pub const TRUSTED_COMMAND_RESPONSE_SCHEMA: &str = "reason-trusted-verifier-response-v1";
pub const DEFAULT_TRUSTED_COMMAND_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_TRUSTED_COMMAND_MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrustedCommandVerifierConfig {
    /// Operator-assigned identity for the deterministic/explicitly trusted oracle.
    pub verifier_id: String,
    pub program: PathBuf,
    pub args: Vec<String>,
    pub timeout_ms: u64,
    pub max_response_bytes: usize,
}

impl TrustedCommandVerifierConfig {
    pub fn with_defaults(verifier_id: String, program: PathBuf, args: Vec<String>) -> Self {
        Self {
            verifier_id,
            program,
            args,
            timeout_ms: DEFAULT_TRUSTED_COMMAND_TIMEOUT_MS,
            max_response_bytes: DEFAULT_TRUSTED_COMMAND_MAX_RESPONSE_BYTES,
        }
    }
}

#[derive(Debug)]
pub struct TrustedCommandVerifier {
    config: TrustedCommandVerifierConfig,
    config_id: String,
}

impl TrustedCommandVerifier {
    pub fn new(config: TrustedCommandVerifierConfig) -> Self {
        let config_id = stable_config_id(TRUSTED_COMMAND_VERIFIER_ID, &config);
        Self { config, config_id }
    }
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct TrustedVerifierRequestEnvelope<'a> {
    schema_version: &'static str,
    adapter_id: &'static str,
    verifier_id: &'a str,
    attempt_index: usize,
    request: &'a ResolutionRequest,
    evidence: &'a [Evidence],
    #[serde(skip_serializing_if = "Option::is_none")]
    requirement: Option<&'a EvidenceRequirement>,
    authority_policy: &'a EvidenceAuthorityPolicy,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustedVerifierResponseEnvelope {
    schema_version: String,
    #[serde(default)]
    result: Option<TrustedVerifierResult>,
    #[serde(default)]
    failure: Option<TrustedVerifierFailure>,
    #[serde(default)]
    cost: ResolutionCost,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustedVerifierResult {
    conclusion: TrustedVerifierConclusion,
    #[serde(default)]
    evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TrustedVerifierConclusion {
    Supported,
    Contradicted,
    NoResult,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TrustedVerifierFailure {
    kind: TrustedVerifierFailureKind,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TrustedVerifierFailureKind {
    Transport,
    Authentication,
    PermissionDenied,
    Protocol,
    ToolExecution,
    Timeout,
    Unavailable,
    PolicyDenied,
}

impl From<TrustedVerifierFailureKind> for ResolutionAdapterErrorKind {
    fn from(value: TrustedVerifierFailureKind) -> Self {
        match value {
            TrustedVerifierFailureKind::Transport => Self::Transport,
            TrustedVerifierFailureKind::Authentication => Self::Authentication,
            TrustedVerifierFailureKind::PermissionDenied => Self::PermissionDenied,
            TrustedVerifierFailureKind::Protocol => Self::Protocol,
            TrustedVerifierFailureKind::ToolExecution => Self::ToolExecution,
            TrustedVerifierFailureKind::Timeout => Self::Timeout,
            TrustedVerifierFailureKind::Unavailable => Self::Unavailable,
            TrustedVerifierFailureKind::PolicyDenied => Self::PolicyDenied,
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

fn target_proposition(
    target: &reasoning_harness_core::ResolutionTarget,
) -> Option<&reasoning_harness_core::Proposition> {
    match target {
        reasoning_harness_core::ResolutionTarget::Proposition { proposition } => Some(proposition),
        reasoning_harness_core::ResolutionTarget::EvidenceQualification { requirement } => {
            Some(&requirement.proposition)
        }
        reasoning_harness_core::ResolutionTarget::CausalRelation { .. }
        | reasoning_harness_core::ResolutionTarget::ClaimRevision { .. }
        | reasoning_harness_core::ResolutionTarget::HumanReview { .. } => None,
    }
}

fn requirement_for<'a>(
    artifact: &'a ReasoningArtifact,
    request: &ResolutionRequest,
) -> Option<&'a EvidenceRequirement> {
    let proposition = target_proposition(&request.target)?;
    artifact
        .evidence_requirements
        .iter()
        .find(|requirement| &requirement.proposition == proposition)
}

fn validate_evidence_binding(
    artifact: &ReasoningArtifact,
    request: &ResolutionRequest,
    evidence_ids: &[String],
) -> Result<(), ResolutionAdapterErrorKind> {
    if evidence_ids.is_empty() {
        return Err(ResolutionAdapterErrorKind::Protocol);
    }
    let mut unique = BTreeSet::new();
    for id in evidence_ids {
        if id.trim().is_empty() || !unique.insert(id.as_str()) {
            return Err(ResolutionAdapterErrorKind::Protocol);
        }
        if !artifact.evidence.iter().any(|evidence| evidence.id == *id) {
            return Err(ResolutionAdapterErrorKind::Protocol);
        }
    }

    let Some(proposition) = target_proposition(&request.target) else {
        return Err(ResolutionAdapterErrorKind::PolicyDenied);
    };
    if requirement_for(artifact, request).is_none() {
        return Ok(());
    }

    let inspection = EvidenceQualificationInspector.inspect(artifact);
    for id in evidence_ids {
        let qualified = inspection.assessments.iter().any(|assessment| {
            assessment.proposition == *proposition
                && assessment.evidence_id == *id
                && assessment.status == EvidenceQualificationStatus::Qualified
        });
        if !qualified {
            return Err(ResolutionAdapterErrorKind::PolicyDenied);
        }
    }
    Ok(())
}

impl TrustedResolutionVerifier for TrustedCommandVerifier {
    fn name(&self) -> &'static str {
        TRUSTED_COMMAND_VERIFIER_ID
    }

    fn config_id(&self) -> Option<&str> {
        Some(&self.config_id)
    }

    fn verify(
        &self,
        request: &ResolutionRequest,
        artifact: &ReasoningArtifact,
        attempt_index: usize,
    ) -> Result<TrustedVerifierResolutionOutput, ResolutionAdapterError> {
        let started = Instant::now();
        if self.config.verifier_id.trim().is_empty()
            || self.config.timeout_ms == 0
            || self.config.max_response_bytes == 0
            || target_proposition(&request.target).is_none()
        {
            return Err(adapter_error(
                ResolutionAdapterErrorKind::PolicyDenied,
                started,
                ResolutionCost::default(),
            ));
        }

        let envelope = TrustedVerifierRequestEnvelope {
            schema_version: TRUSTED_COMMAND_REQUEST_SCHEMA,
            adapter_id: TRUSTED_COMMAND_VERIFIER_ID,
            verifier_id: &self.config.verifier_id,
            attempt_index,
            request,
            evidence: &artifact.evidence,
            requirement: requirement_for(artifact, request),
            authority_policy: &artifact.authority_policy,
        };
        let payload = serde_json::to_vec(&envelope).map_err(|_| {
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
                    return Err(adapter_error(
                        ResolutionAdapterErrorKind::Timeout,
                        started,
                        ResolutionCost::default(),
                    ));
                }
                Err(_) => {
                    let _ = child.kill();
                    let _ = child.wait();
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

        let response: TrustedVerifierResponseEnvelope =
            serde_json::from_slice(&output).map_err(|_| {
                adapter_error(
                    ResolutionAdapterErrorKind::Protocol,
                    started,
                    ResolutionCost::default(),
                )
            })?;
        if response.schema_version != TRUSTED_COMMAND_RESPONSE_SCHEMA {
            return Err(adapter_error(
                ResolutionAdapterErrorKind::Protocol,
                started,
                response.cost,
            ));
        }
        match (response.result, response.failure) {
            (None, Some(failure)) => {
                Err(adapter_error(failure.kind.into(), started, response.cost))
            }
            (Some(result), None) => {
                if matches!(result.conclusion, TrustedVerifierConclusion::NoResult) {
                    if !result.evidence_ids.is_empty() {
                        return Err(adapter_error(
                            ResolutionAdapterErrorKind::Protocol,
                            started,
                            response.cost,
                        ));
                    }
                    return Ok(TrustedVerifierResolutionOutput {
                        receipts: vec![],
                        cost: measured_cost(started, response.cost),
                    });
                }
                validate_evidence_binding(artifact, request, &result.evidence_ids)
                    .map_err(|kind| adapter_error(kind, started, response.cost))?;
                let proposition = target_proposition(&request.target)
                    .expect("proposition target checked above")
                    .clone();
                let conclusion = match result.conclusion {
                    TrustedVerifierConclusion::Supported => VerificationConclusion::Supported,
                    TrustedVerifierConclusion::Contradicted => VerificationConclusion::Contradicted,
                    TrustedVerifierConclusion::NoResult => unreachable!(),
                };
                let receipt = VerificationReceipt {
                    id: format!("trusted-command:{}:{attempt_index}", request.id),
                    verifier: self.config.verifier_id.clone(),
                    claim_statement: None,
                    proposition: Some(proposition),
                    claim_id: None,
                    conclusion,
                    evidence_ids: result.evidence_ids,
                };
                Ok(TrustedVerifierResolutionOutput {
                    receipts: vec![receipt],
                    cost: measured_cost(started, response.cost),
                })
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
    use std::{collections::BTreeMap, fs};

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    use reasoning_harness_core::{
        ApplicabilityScope, EvidenceMetadata, Proposition, ResolutionReason,
        ResolutionRequestBudget, ResolutionTarget, ResolverClass, ScopeCoverage, TemporalValidity,
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
            resolver_class: ResolverClass::DeterministicVerifier,
            budget: ResolutionRequestBudget::default(),
        }
    }

    fn artifact(qualified: bool) -> ReasoningArtifact {
        let proposition = target_proposition(&request().target).unwrap().clone();
        let scope: ApplicabilityScope = BTreeMap::from([(
            "region".into(),
            ScopeCoverage::Values {
                values: ["eu-west-1".into()].into_iter().collect(),
            },
        )]);
        ReasoningArtifact {
            task: "determine region".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "external admitted source".into(),
                observation: "service.region=eu-west-1".into(),
                facts: BTreeMap::from([("service.region".into(), "eu-west-1".into())]),
                metadata: EvidenceMetadata {
                    temporal: Some(TemporalValidity {
                        effective_from_unix_seconds: Some(0),
                        effective_until_unix_seconds: Some(if qualified { 2_000 } else { 500 }),
                    }),
                    scope: Some(scope.clone()),
                    provenance_class: Some("primary".into()),
                },
            }],
            hypotheses: vec![proposition.clone()],
            assumptions: vec![],
            evidence_requirements: vec![EvidenceRequirement {
                proposition,
                as_of_unix_seconds: Some(1_000),
                scope: Some(scope),
                minimum_authority_class: Some("primary".into()),
            }],
            authority_policy: EvidenceAuthorityPolicy {
                ranks: BTreeMap::from([("primary".into(), 10)]),
            },
            candidate_diagnostics: vec![],
            claims: vec![],
            verification_receipts: vec![],
            adversarial_findings: vec![],
            assumption_findings: vec![],
            evidence_qualification_findings: vec![],
            inferences: vec![],
        }
    }

    #[cfg(unix)]
    fn script(body: &str, name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "reason-trusted-verifier-{}-{name}.sh",
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
    fn verifier_constructs_exact_receipt_in_harness_not_from_external_fields() {
        let path = script(
            r#"#!/bin/sh
cat >/dev/null
printf '%s' '{"schema_version":"reason-trusted-verifier-response-v1","result":{"conclusion":"supported","evidence_ids":["e1"]}}'
"#,
            "support",
        );
        let verifier = TrustedCommandVerifier::new(TrustedCommandVerifierConfig::with_defaults(
            "reference-policy-oracle".into(),
            path.clone(),
            vec![],
        ));
        let output = verifier.verify(&request(), &artifact(true), 0).unwrap();
        fs::remove_file(path).ok();
        assert_eq!(output.receipts.len(), 1);
        let receipt = &output.receipts[0];
        assert_eq!(receipt.verifier, "reference-policy-oracle");
        assert_eq!(
            receipt.proposition.as_ref(),
            target_proposition(&request().target)
        );
        assert_eq!(receipt.evidence_ids, vec!["e1"]);
        assert_eq!(receipt.conclusion, VerificationConclusion::Supported);
    }

    #[cfg(unix)]
    #[test]
    fn external_response_cannot_smuggle_receipt_or_verifier_identity() {
        let path = script(
            r#"#!/bin/sh
cat >/dev/null
printf '%s' '{"schema_version":"reason-trusted-verifier-response-v1","result":{"conclusion":"supported","evidence_ids":["e1"],"verifier":"attacker"}}'
"#,
            "smuggle",
        );
        let verifier = TrustedCommandVerifier::new(TrustedCommandVerifierConfig::with_defaults(
            "reference-policy-oracle".into(),
            path.clone(),
            vec![],
        ));
        let error = verifier.verify(&request(), &artifact(true), 0).unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Protocol);
    }

    #[cfg(unix)]
    #[test]
    fn qualified_requirement_cannot_be_bypassed_by_trusted_command_binding() {
        let path = script(
            r#"#!/bin/sh
cat >/dev/null
printf '%s' '{"schema_version":"reason-trusted-verifier-response-v1","result":{"conclusion":"supported","evidence_ids":["e1"]}}'
"#,
            "stale",
        );
        let verifier = TrustedCommandVerifier::new(TrustedCommandVerifierConfig::with_defaults(
            "reference-policy-oracle".into(),
            path.clone(),
            vec![],
        ));
        let error = verifier
            .verify(&request(), &artifact(false), 0)
            .unwrap_err();
        fs::remove_file(path).ok();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::PolicyDenied);
    }

    #[cfg(unix)]
    #[test]
    fn contradiction_and_operational_failure_stay_typed() {
        let contradiction_path = script(
            r#"#!/bin/sh
cat >/dev/null
printf '%s' '{"schema_version":"reason-trusted-verifier-response-v1","result":{"conclusion":"contradicted","evidence_ids":["e1"]}}'
"#,
            "contradiction",
        );
        let verifier = TrustedCommandVerifier::new(TrustedCommandVerifierConfig::with_defaults(
            "reference-policy-oracle".into(),
            contradiction_path.clone(),
            vec![],
        ));
        let output = verifier.verify(&request(), &artifact(true), 0).unwrap();
        fs::remove_file(contradiction_path).ok();
        assert_eq!(
            output.receipts[0].conclusion,
            VerificationConclusion::Contradicted
        );

        let failure_path = script(
            r#"#!/bin/sh
cat >/dev/null
printf '%s' '{"schema_version":"reason-trusted-verifier-response-v1","failure":{"kind":"authentication"}}'
"#,
            "auth",
        );
        let verifier = TrustedCommandVerifier::new(TrustedCommandVerifierConfig::with_defaults(
            "reference-policy-oracle".into(),
            failure_path.clone(),
            vec![],
        ));
        let error = verifier.verify(&request(), &artifact(true), 0).unwrap_err();
        fs::remove_file(failure_path).ok();
        assert_eq!(error.kind, ResolutionAdapterErrorKind::Authentication);
    }
    #[cfg(unix)]
    #[test]
    fn opaque_acquired_data_needs_explicit_trusted_verifier_to_promote() {
        use reasoning_harness_core::{
            CandidateClaim, CanonicalFinalAnswerRenderer, DefaultResolutionPlanner, EpistemicState,
            GroundedResolutionPolicy, GroundedResolutionRuntime, HarnessInput, ReasoningCandidate,
            RejectAllEvidenceAdmission, ResolutionBudget, StandardGroundingPipeline, Verdict,
        };

        let proposition = Proposition {
            key: "service.region".into(),
            value: "eu-west-1".into(),
        };
        let input = HarnessInput {
            task: "determine region".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "admitted external blob".into(),
                observation: "opaque signed response".into(),
                facts: BTreeMap::new(),
                metadata: EvidenceMetadata::default(),
            }],
            hypotheses: vec![proposition.clone()],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: EvidenceAuthorityPolicy::default(),
        };
        let candidate = ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: "c1".into(),
                statement: "model says eu-west-1".into(),
                proposed_state: EpistemicState::Supported,
                proposition: Some(proposition),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        };
        let no_verifiers: [&dyn TrustedResolutionVerifier; 0] = [];
        let no_resolvers: [&dyn reasoning_harness_core::ResolutionResolver; 0] = [];
        let baseline = GroundedResolutionRuntime {
            pipeline: &StandardGroundingPipeline,
            planner: &DefaultResolutionPlanner,
            evidence_admission: &RejectAllEvidenceAdmission,
            resolvers: &no_resolvers,
            trusted_verifiers: &no_verifiers,
            renderer: &CanonicalFinalAnswerRenderer,
        }
        .run(
            input.clone(),
            candidate.clone(),
            &GroundedResolutionPolicy::default(),
        )
        .unwrap();
        assert_eq!(baseline.final_verdict, Verdict::Unknown);

        let path = script(
            r#"#!/bin/sh
cat >/dev/null
printf '%s' '{"schema_version":"reason-trusted-verifier-response-v1","result":{"conclusion":"supported","evidence_ids":["e1"]}}'
"#,
            "runtime-support",
        );
        let verifier = TrustedCommandVerifier::new(TrustedCommandVerifierConfig::with_defaults(
            "reference-signature-oracle".into(),
            path.clone(),
            vec![],
        ));
        let trusted: [&dyn TrustedResolutionVerifier; 1] = [&verifier];
        let policy = GroundedResolutionPolicy {
            budget: ResolutionBudget {
                allowed_resolver_classes: BTreeSet::from([
                    reasoning_harness_core::ResolverClass::DeterministicVerifier,
                ]),
                ..ResolutionBudget::default()
            },
            proposition_resolver_class:
                reasoning_harness_core::ResolverClass::DeterministicVerifier,
            ..GroundedResolutionPolicy::default()
        };
        let resolved = GroundedResolutionRuntime {
            pipeline: &StandardGroundingPipeline,
            planner: &DefaultResolutionPlanner,
            evidence_admission: &RejectAllEvidenceAdmission,
            resolvers: &no_resolvers,
            trusted_verifiers: &trusted,
            renderer: &CanonicalFinalAnswerRenderer,
        }
        .run(input, candidate, &policy)
        .unwrap();
        fs::remove_file(path).ok();
        assert_eq!(resolved.initial_verdict, Verdict::Unknown);
        assert_eq!(resolved.final_verdict, Verdict::Accept);
        assert_eq!(resolved.attempts[0].verification_receipts, 1);
        assert_eq!(
            resolved.final_artifact.verification_receipts[0].verifier,
            "reference-signature-oracle"
        );
    }
}
