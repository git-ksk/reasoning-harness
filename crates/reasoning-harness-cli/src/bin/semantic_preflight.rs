use std::{collections::BTreeMap, process::ExitCode};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    MaterializationError, MaterializationFailureClass, ModelAdapter, ModelError,
    SemanticRuntimeIdentity, SemanticRuntimeProfile, SoftJudgeDecision,
    classify_materialization_failure, run_materialization_capability_preflight,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter, NvidiaAdapter};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "reason-semantic-preflight",
    about = "Protocol-only R2/D3 semantic runtime capability preflight"
)]
struct Args {
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long)]
    model: String,
    #[arg(long, default_value_t = 128)]
    max_tokens: u32,
    /// Number of protocol-only probes. Every probe must be compatible for an overall pass.
    #[arg(long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..=10))]
    probes: u8,
    /// Base seed. Probe N uses base_seed + N.
    #[arg(long, default_value_t = 0)]
    seed: u64,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Mistral,
    Google,
    Nvidia,
}

enum Generator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
    Nvidia(NvidiaAdapter),
}

impl Generator {
    fn from_provider(provider: Provider, model: &str) -> Result<Self, ModelError> {
        match provider {
            Provider::Mistral => MistralAdapter::from_env(model).map(Self::Mistral),
            Provider::Google => GoogleAdapter::from_env(model).map(Self::Google),
            Provider::Nvidia => NvidiaAdapter::from_env(model).map(Self::Nvidia),
        }
    }

    fn adapter(&self) -> &dyn ModelAdapter {
        match self {
            Self::Mistral(adapter) => adapter,
            Self::Google(adapter) => adapter,
            Self::Nvidia(adapter) => adapter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CapabilityStatus {
    Compatible,
    Incompatible,
    OperationallyIncomplete,
}

#[derive(Debug, Serialize)]
struct PreflightProbe {
    probe: u8,
    seed: u64,
    status: CapabilityStatus,
    protocol_compatible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_decision: Option<SoftJudgeDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<reasoning_harness_core::ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_class: Option<MaterializationFailureClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<String>,
}

#[derive(Debug, Serialize)]
struct PreflightOutput {
    runtime: SemanticRuntimeIdentity,
    capability_id: &'static str,
    materialization_contract: &'static str,
    provider: &'static str,
    requested_model: String,
    status: CapabilityStatus,
    protocol_compatible: bool,
    requested_probes: u8,
    attempted_probes: usize,
    successful_probes: usize,
    failure_counts: BTreeMap<MaterializationFailureClass, usize>,
    probes: Vec<PreflightProbe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    setup_failure_class: Option<MaterializationFailureClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    setup_failure: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    match run(args).await {
        Ok(output) => {
            let exit_code = if output.status == CapabilityStatus::Compatible {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            exit_code
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn run(args: Args) -> Result<PreflightOutput, String> {
    args.seed
        .checked_add(u64::from(args.probes - 1))
        .ok_or("preflight probe seed overflow")?;

    let provider = provider_name(args.provider);
    let runtime = SemanticRuntimeProfile::SemanticDecidabilityD3V1.identity();
    let generator = match Generator::from_provider(args.provider, &args.model) {
        Ok(generator) => generator,
        Err(error) => {
            let error = MaterializationError::Model(error);
            let failure_class = classify_materialization_failure(&error);
            return Ok(PreflightOutput {
                runtime,
                capability_id: reasoning_harness_core::R2_MATERIALIZATION_CAPABILITY_ID,
                materialization_contract: reasoning_harness_core::MATERIALIZATION_R2_CONTRACT_ID,
                provider,
                requested_model: args.model,
                status: status_for_failure(failure_class),
                protocol_compatible: false,
                requested_probes: args.probes,
                attempted_probes: 0,
                successful_probes: 0,
                failure_counts: BTreeMap::from([(failure_class, 1)]),
                probes: vec![],
                setup_failure_class: Some(failure_class),
                setup_failure: Some(error.to_string()),
            });
        }
    };

    let mut probes = Vec::with_capacity(usize::from(args.probes));
    for probe_index in 0..args.probes {
        let probe_seed = args.seed + u64::from(probe_index);
        let probe_number = probe_index + 1;
        match run_materialization_capability_preflight(
            generator.adapter(),
            args.max_tokens,
            Some(probe_seed),
        )
        .await
        {
            Ok(result) => probes.push(PreflightProbe {
                probe: probe_number,
                seed: probe_seed,
                status: CapabilityStatus::Compatible,
                protocol_compatible: true,
                observed_model: Some(result.model),
                observed_decision: Some(result.observed_decision),
                usage: Some(result.usage),
                finish_reason: result.finish_reason,
                failure_class: None,
                failure: None,
            }),
            Err(error) => {
                let failure_class = classify_materialization_failure(&error);
                let status = status_for_failure(failure_class);
                probes.push(PreflightProbe {
                    probe: probe_number,
                    seed: probe_seed,
                    status,
                    protocol_compatible: false,
                    observed_model: error.provider_model().map(str::to_string),
                    observed_decision: None,
                    usage: error.usage().cloned(),
                    finish_reason: error.finish_reason().map(str::to_string),
                    failure_class: Some(failure_class),
                    failure: Some(error.to_string()),
                });
                // Provider/quota/transport failures do not establish protocol incompatibility and
                // repeating them can amplify quota/rate-limit pressure. Protocol failures continue
                // through the bounded series so intermittent incompatibility remains visible.
                if status == CapabilityStatus::OperationallyIncomplete {
                    break;
                }
            }
        }
    }

    Ok(summarize_preflight(
        runtime,
        provider,
        args.model,
        args.probes,
        probes,
    ))
}

fn summarize_preflight(
    runtime: SemanticRuntimeIdentity,
    provider: &'static str,
    requested_model: String,
    requested_probes: u8,
    probes: Vec<PreflightProbe>,
) -> PreflightOutput {
    let successful_probes = probes
        .iter()
        .filter(|probe| probe.protocol_compatible)
        .count();
    let mut failure_counts = BTreeMap::new();
    for failure_class in probes.iter().filter_map(|probe| probe.failure_class) {
        *failure_counts.entry(failure_class).or_insert(0) += 1;
    }
    let status = if probes.len() == usize::from(requested_probes)
        && successful_probes == usize::from(requested_probes)
    {
        CapabilityStatus::Compatible
    } else if probes
        .iter()
        .any(|probe| probe.status == CapabilityStatus::Incompatible)
    {
        CapabilityStatus::Incompatible
    } else {
        CapabilityStatus::OperationallyIncomplete
    };

    PreflightOutput {
        runtime,
        capability_id: reasoning_harness_core::R2_MATERIALIZATION_CAPABILITY_ID,
        materialization_contract: reasoning_harness_core::MATERIALIZATION_R2_CONTRACT_ID,
        provider,
        requested_model,
        status,
        protocol_compatible: status == CapabilityStatus::Compatible,
        requested_probes,
        attempted_probes: probes.len(),
        successful_probes,
        failure_counts,
        probes,
        setup_failure_class: None,
        setup_failure: None,
    }
}

fn status_for_failure(failure_class: MaterializationFailureClass) -> CapabilityStatus {
    match failure_class {
        MaterializationFailureClass::ProviderProtocol
        | MaterializationFailureClass::UnsupportedCapability
        | MaterializationFailureClass::MaterializationProtocol => CapabilityStatus::Incompatible,
        _ => CapabilityStatus::OperationallyIncomplete,
    }
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Mistral => "mistral",
        Provider::Google => "google",
        Provider::Nvidia => "nvidia",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reasoning_harness_core::SOFT_SEMANTIC_V3_CONFIGURATION_ID;

    fn probe(status: CapabilityStatus) -> PreflightProbe {
        let failure_class = match status {
            CapabilityStatus::Compatible => None,
            CapabilityStatus::Incompatible => {
                Some(MaterializationFailureClass::MaterializationProtocol)
            }
            CapabilityStatus::OperationallyIncomplete => Some(MaterializationFailureClass::Quota),
        };
        PreflightProbe {
            probe: 1,
            seed: 0,
            status,
            protocol_compatible: status == CapabilityStatus::Compatible,
            observed_model: None,
            observed_decision: None,
            usage: None,
            finish_reason: None,
            failure_class,
            failure: failure_class.map(|class| class.to_string()),
        }
    }

    #[test]
    fn mixed_protocol_series_is_incompatible_not_compatible() {
        let runtime = SemanticRuntimeProfile::SemanticDecidabilityD3V1.identity();
        let output = summarize_preflight(
            runtime,
            "nvidia",
            "m".into(),
            3,
            vec![
                probe(CapabilityStatus::Compatible),
                probe(CapabilityStatus::Incompatible),
                probe(CapabilityStatus::Compatible),
            ],
        );
        assert_eq!(output.status, CapabilityStatus::Incompatible);
        assert!(!output.protocol_compatible);
        assert_eq!(output.successful_probes, 2);
        assert_eq!(
            output
                .failure_counts
                .get(&MaterializationFailureClass::MaterializationProtocol),
            Some(&1)
        );
        assert_eq!(
            output.runtime.rollback_configuration_id(),
            Some(SOFT_SEMANTIC_V3_CONFIGURATION_ID)
        );
    }

    #[test]
    fn truncated_series_is_operationally_incomplete() {
        let runtime = SemanticRuntimeProfile::SemanticDecidabilityD3V1.identity();
        let output = summarize_preflight(
            runtime,
            "google",
            "m".into(),
            3,
            vec![
                probe(CapabilityStatus::Compatible),
                probe(CapabilityStatus::OperationallyIncomplete),
            ],
        );
        assert_eq!(output.status, CapabilityStatus::OperationallyIncomplete);
        assert!(!output.protocol_compatible);
    }
}
