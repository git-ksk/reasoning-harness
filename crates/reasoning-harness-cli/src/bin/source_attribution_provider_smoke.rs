use std::process::ExitCode;

use clap::Parser;
use reasoning_harness_core::{
    Evidence, EvidenceMetadata, ModelAdapter, ReasoningArtifact, SourceAttributionAuthorityCeiling,
    SourceAttributionBinding, SourceAttributionLocator, SourceAttributionState,
    SourceAttributionTargetPolicy, SourceAttributionTransformKind, SourceTextSpan,
    build_source_attribution_proposal_request, parse_source_attribution_proposal,
};
use reasoning_harness_providers::MistralAdapter;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "reason-source-attribution-provider-smoke",
    about = "Non-scorable provider compatibility smoke for #463"
)]
struct Args {
    #[arg(long, default_value = "ministral-8b-2512")]
    model: String,
}

#[derive(Debug, Serialize)]
struct Output {
    schema_version: &'static str,
    issue: u64,
    scorability: &'static str,
    semantic_cases_observed: u64,
    provider: &'static str,
    model: String,
    status: &'static str,
    provider_attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    transform_kind: Option<SourceAttributionTransformKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_kind: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let observation = "Zephyra Gate may be enabled for selected workspaces during preview.";
    let artifact = ReasoningArtifact {
        task: "Provider compatibility smoke only.".into(),
        evidence: vec![Evidence {
            id: "smoke-e1".into(),
            source: "smoke-source".into(),
            observation: observation.into(),
            facts: Default::default(),
            metadata: EvidenceMetadata::default(),
        }],
        source_attribution: SourceAttributionState {
            targets: vec![SourceAttributionTargetPolicy {
                policy_id: "smoke-policy".into(),
                target_id: "smoke-target".into(),
                target_question: "What does the synthetic smoke source say?".into(),
                authority_ceiling: SourceAttributionAuthorityCeiling::SourceLocal,
                hard_verification_required: false,
            }],
            bindings: vec![SourceAttributionBinding {
                id: "smoke-b1".into(),
                target_id: "smoke-target".into(),
                evidence_id: "smoke-e1".into(),
                source_id: "smoke-source".into(),
                source_url: None,
                locator: Some(SourceAttributionLocator {
                    heading: Some("Smoke".into()),
                    ..Default::default()
                }),
                retrieved_at_unix_seconds: Some(0),
                source_version: Some("smoke-v1".into()),
                span: SourceTextSpan {
                    start_byte: 0,
                    end_byte: observation.len(),
                },
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    let bindings = vec!["smoke-b1".to_string()];
    let allowed = vec![SourceAttributionTransformKind::Paraphrase];
    let request = match build_source_attribution_proposal_request(
        &artifact,
        "smoke-target",
        &bindings,
        &allowed,
        None,
        Some(96),
        Some(463002),
    ) {
        Ok(request) => request,
        Err(error) => {
            print_output(Output {
                schema_version: "source-attribution-provider-smoke-v1",
                issue: 463,
                scorability: "non_scorable_operational_diagnostic",
                semantic_cases_observed: 0,
                provider: "mistral",
                model: args.model,
                status: "local_contract_error",
                provider_attempts: 0,
                transform_kind: None,
                error_kind: Some(error.to_string()),
            });
            return ExitCode::FAILURE;
        }
    };

    let adapter = match MistralAdapter::from_env(&args.model) {
        Ok(adapter) => adapter,
        Err(error) => {
            print_output(Output {
                schema_version: "source-attribution-provider-smoke-v1",
                issue: 463,
                scorability: "non_scorable_operational_diagnostic",
                semantic_cases_observed: 0,
                provider: "mistral",
                model: args.model,
                status: "adapter_error",
                provider_attempts: error.provider_attempts,
                transform_kind: None,
                error_kind: Some(format!("{:?}", error.kind).to_lowercase()),
            });
            return ExitCode::FAILURE;
        }
    };

    let response = match adapter.generate(request).await {
        Ok(response) => response,
        Err(error) => {
            print_output(Output {
                schema_version: "source-attribution-provider-smoke-v1",
                issue: 463,
                scorability: "non_scorable_operational_diagnostic",
                semantic_cases_observed: 0,
                provider: "mistral",
                model: args.model,
                status: "provider_error",
                provider_attempts: error.provider_attempts,
                transform_kind: None,
                error_kind: Some(format!("{:?}", error.kind).to_lowercase()),
            });
            return ExitCode::FAILURE;
        }
    };

    match parse_source_attribution_proposal(
        &response.text,
        "smoke-target",
        &bindings,
        &allowed,
        None,
    ) {
        Ok(proposal) => {
            print_output(Output {
                schema_version: "source-attribution-provider-smoke-v1",
                issue: 463,
                scorability: "non_scorable_operational_diagnostic",
                semantic_cases_observed: 0,
                provider: "mistral",
                model: args.model,
                status: "ok",
                provider_attempts: response.provider_attempts,
                transform_kind: Some(proposal.transform_kind),
                error_kind: None,
            });
            ExitCode::SUCCESS
        }
        Err(_) => {
            print_output(Output {
                schema_version: "source-attribution-provider-smoke-v1",
                issue: 463,
                scorability: "non_scorable_operational_diagnostic",
                semantic_cases_observed: 0,
                provider: "mistral",
                model: args.model,
                status: "protocol_error",
                provider_attempts: response.provider_attempts,
                transform_kind: None,
                error_kind: Some("protocol".into()),
            });
            ExitCode::FAILURE
        }
    }
}

fn print_output(output: Output) {
    match serde_json::to_string_pretty(&output) {
        Ok(value) => println!("{value}"),
        Err(_) => println!(
            r#"{{"schema_version":"source-attribution-provider-smoke-v1","status":"serialization_error"}}"#
        ),
    }
}
