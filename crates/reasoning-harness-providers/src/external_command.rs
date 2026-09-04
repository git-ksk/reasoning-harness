use std::{
    collections::BTreeMap,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

use reasoning_harness_core::{
    AcquiredEvidence, AcquiredEvidenceMetadata, ReasoningCandidate, ResolutionAdapterError,
    ResolutionAdapterErrorKind, ResolutionCost, ResolutionRequest, ResolutionResolver,
    ResolutionResolverContribution, ResolutionResolverOutput, ResolverClass,
};
use serde::{Deserialize, Serialize};

pub const EXTERNAL_COMMAND_RESOLVER_ID: &str = "external_command_v1";
pub const EXTERNAL_RESOLVER_REQUEST_SCHEMA: &str = "reason-external-resolver-request-v1";
pub const EXTERNAL_RESOLVER_RESPONSE_SCHEMA: &str = "reason-external-resolver-response-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalCommandResolverConfig {
    pub program: PathBuf,
    pub args: Vec<String>,
}

#[derive(Debug)]
pub struct ExternalCommandResolver {
    config: ExternalCommandResolverConfig,
}

impl ExternalCommandResolver {
    pub fn new(config: ExternalCommandResolverConfig) -> Self {
        Self { config }
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
    contribution: ExternalResolverContribution,
    #[serde(default)]
    cost: ResolutionCost,
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

fn adapter_error(kind: ResolutionAdapterErrorKind) -> ResolutionAdapterError {
    ResolutionAdapterError {
        kind,
        cost: ResolutionCost::default(),
    }
}

impl ResolutionResolver for ExternalCommandResolver {
    fn name(&self) -> &'static str {
        EXTERNAL_COMMAND_RESOLVER_ID
    }

    fn class(&self) -> ResolverClass {
        ResolverClass::EvidenceAcquisition
    }

    fn resolve(
        &self,
        request: &ResolutionRequest,
        attempt_index: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
        let payload = serde_json::to_vec(&ExternalResolverRequestEnvelope {
            schema_version: EXTERNAL_RESOLVER_REQUEST_SCHEMA,
            adapter_id: EXTERNAL_COMMAND_RESOLVER_ID,
            attempt_index,
            request,
        })
        .map_err(|_| adapter_error(ResolutionAdapterErrorKind::Failed))?;

        let mut child = Command::new(&self.config.program)
            .args(&self.config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| adapter_error(ResolutionAdapterErrorKind::Unavailable))?;

        child
            .stdin
            .as_mut()
            .ok_or_else(|| adapter_error(ResolutionAdapterErrorKind::Failed))?
            .write_all(&payload)
            .map_err(|_| adapter_error(ResolutionAdapterErrorKind::Failed))?;
        drop(child.stdin.take());

        let output = child
            .wait_with_output()
            .map_err(|_| adapter_error(ResolutionAdapterErrorKind::Failed))?;
        if !output.status.success() {
            return Err(adapter_error(ResolutionAdapterErrorKind::Failed));
        }

        let response: ExternalResolverResponseEnvelope = serde_json::from_slice(&output.stdout)
            .map_err(|_| adapter_error(ResolutionAdapterErrorKind::MalformedOutput))?;
        if response.schema_version != EXTERNAL_RESOLVER_RESPONSE_SCHEMA {
            return Err(adapter_error(ResolutionAdapterErrorKind::MalformedOutput));
        }

        Ok(ResolutionResolverOutput {
            contribution: response.contribution.into(),
            cost: response.cost,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig {
            program: path.clone(),
            args: vec![],
        });
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
        match output.contribution {
            ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                assert_eq!(evidence.len(), 1);
                assert_eq!(evidence[0].source, "reference:test");
                assert_eq!(evidence[0].facts["service.region"], "eu-west-1");
            }
            other => panic!("expected acquired evidence, got {other:?}"),
        }
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
