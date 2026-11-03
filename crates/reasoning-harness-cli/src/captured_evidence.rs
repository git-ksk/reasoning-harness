//! Fixed local acquisition data, never a hard verification receipt or user fact.
//! Uses the ordinary investigation admission policy without starting a process.
use std::collections::BTreeSet;

use reasoning_harness_core::{
    AcquiredEvidence, ResolutionAdapterError, ResolutionAdapterErrorKind, ResolutionCost,
    ResolutionRequest, ResolutionResolver, ResolutionResolverContribution,
    ResolutionResolverOutput, ResolutionTarget, ResolverClass,
};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) const RESOLVER_ID: &str = "investigation_captured_evidence_v1";

#[derive(Debug, Clone)]
pub(crate) struct CapturedEvidenceResolver {
    evidence: Vec<AcquiredEvidence>,
    config_id: String,
}

impl CapturedEvidenceResolver {
    pub(crate) fn new(records: &[Value]) -> Result<Self, String> {
        let bytes = serde_json::to_vec(records).map_err(|error| error.to_string())?;
        if records.is_empty() || records.len() > 64 || bytes.len() > 65536 {
            return Err("captured evidence requires 1-64 records within 65536 bytes".into());
        }
        let mut ids = BTreeSet::new();
        let mut evidence = Vec::new();
        for record in records {
            let object = record
                .as_object()
                .ok_or("captured evidence must be an object")?;
            if object.keys().any(|key| {
                !matches!(
                    key.as_str(),
                    "id" | "source" | "observation" | "facts" | "acquisition_metadata"
                )
            }) {
                return Err("unknown captured evidence field".into());
            }
            let item: AcquiredEvidence = serde_json::from_value(record.clone())
                .map_err(|error| format!("invalid captured evidence: {error}"))?;
            if item.id.trim().is_empty()
                || !ids.insert(item.id.clone())
                || item.source.trim().is_empty()
                || item.observation.trim().is_empty()
                || item.facts.keys().any(|key| key.trim().is_empty())
            {
                return Err("captured evidence requires unique ids and nonempty source/observation/fact keys".into());
            }
            evidence.push(item);
        }
        Ok(Self {
            evidence,
            config_id: format!(
                "{RESOLVER_ID}:{}",
                Sha256::digest(bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            ),
        })
    }
}

impl ResolutionResolver for CapturedEvidenceResolver {
    fn name(&self) -> &'static str {
        RESOLVER_ID
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
        _: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
        if !matches!(
            request.target,
            ResolutionTarget::InvestigationQuestion { .. }
        ) {
            return Err(ResolutionAdapterError {
                kind: ResolutionAdapterErrorKind::PolicyDenied,
                cost: ResolutionCost::default(),
            });
        }
        Ok(ResolutionResolverOutput {
            contribution: ResolutionResolverContribution::AcquiredEvidence {
                evidence: self.evidence.clone(),
            },
            cost: ResolutionCost {
                calls: 1,
                // Same curated observation character accounting as the previous
                // external snapshot adapter. This is not provider-token usage.
                added_tokens: self
                    .evidence
                    .iter()
                    .map(|item| item.observation.chars().count() as u64 / 4)
                    .sum(),
                ..Default::default()
            },
        })
    }
}
