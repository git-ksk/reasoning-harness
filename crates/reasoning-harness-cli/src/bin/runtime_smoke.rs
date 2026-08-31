use std::{collections::BTreeMap, fs, path::PathBuf, process::ExitCode, time::Instant};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    MaterializationFailureClass, ModelAdapter, ModelError, ModelErrorKind, ReasoningArtifact,
    SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID, SOFT_SEMANTIC_V3_CONFIGURATION_ID,
    SemanticDecidabilityDisposition, SemanticRuntimeError, SemanticRuntimeIdentity,
    SemanticRuntimeProfile, SoftJudgeDecision, SoftJudgeRequest, assess_semantic_decidability,
    classify_materialization_failure, run_default_semantic_runtime, run_semantic_runtime,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter};
use serde::{Deserialize, Serialize};

const SMOKE_SURFACE_ID: &str = "semantic-runtime-smoke-v1";

#[derive(Debug, Parser)]
#[command(
    name = "reason-semantic-runtime-smoke",
    about = "Live operational smoke for the adopted D3 runtime and explicit v3 rollback"
)]
struct Args {
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long)]
    model: String,
    #[arg(long, default_value = "fixtures/semantic-runtime-smoke")]
    fixtures: PathBuf,
    #[arg(long, default_value_t = 256)]
    max_tokens: u32,
    #[arg(long, default_value_t = 9000)]
    seed: u64,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Mistral,
    Google,
}

enum Generator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
}

impl Generator {
    fn from_provider(provider: Provider, model: &str) -> Result<Self, ModelError> {
        match provider {
            Provider::Mistral => MistralAdapter::from_env(model).map(Self::Mistral),
            Provider::Google => GoogleAdapter::from_env(model).map(Self::Google),
        }
    }

    fn adapter(&self) -> &dyn ModelAdapter {
        match self {
            Self::Mistral(adapter) => adapter,
            Self::Google(adapter) => adapter,
        }
    }
}

#[derive(Debug, Deserialize)]
struct SmokeFixture {
    id: String,
    request: SoftJudgeRequest,
    artifact: ReasoningArtifact,
    expected_disposition: SemanticDecidabilityDisposition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SmokeStatus {
    Passed,
    Failed,
}

#[derive(Debug, Serialize)]
struct ProfileObservation {
    profile: SemanticRuntimeProfile,
    status: SmokeStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    runtime: Option<SemanticRuntimeIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_decision: Option<SoftJudgeDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    final_decision: Option<SoftJudgeDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disposition: Option<SemanticDecidabilityDisposition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<reasoning_harness_core::ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_attempts: Option<u32>,
    latency_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_class: Option<MaterializationFailureClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<String>,
}

#[derive(Debug, Serialize)]
struct FixtureObservation {
    fixture_id: String,
    expected_disposition: SemanticDecidabilityDisposition,
    seed: u64,
    default_d3: ProfileObservation,
    rollback_v3: ProfileObservation,
}

#[derive(Debug, Serialize)]
struct SmokeOutput {
    surface_id: &'static str,
    provider: &'static str,
    requested_model: String,
    default_configuration_id: &'static str,
    rollback_configuration_id: &'static str,
    fixture_count: usize,
    attempted_provider_calls: usize,
    successful_provider_calls: usize,
    failure_counts: BTreeMap<MaterializationFailureClass, usize>,
    status: SmokeStatus,
    observations: Vec<FixtureObservation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    setup_failure_class: Option<MaterializationFailureClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    setup_failure: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    let output = run(args).await;
    let exit_code = if output.status == SmokeStatus::Passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
    exit_code
}

async fn run(args: Args) -> SmokeOutput {
    let provider_name = provider_name(args.provider);
    let fixtures = match load_fixtures(&args.fixtures) {
        Ok(fixtures) => fixtures,
        Err(error) => {
            return setup_failure(provider_name, args.model, error);
        }
    };
    for fixture in &fixtures {
        match assess_semantic_decidability(&fixture.request, &fixture.artifact) {
            Ok(assessment) if assessment.disposition == fixture.expected_disposition => {}
            Ok(assessment) => {
                return setup_failure(
                    provider_name,
                    args.model,
                    format!(
                        "runtime smoke fixture {} expected {:?} but deterministic gate produced {:?}",
                        fixture.id, fixture.expected_disposition, assessment.disposition
                    ),
                );
            }
            Err(error) => {
                return setup_failure(
                    provider_name,
                    args.model,
                    format!("runtime smoke fixture {} is invalid: {error}", fixture.id),
                );
            }
        }
    }

    let generator = match Generator::from_provider(args.provider, &args.model) {
        Ok(generator) => generator,
        Err(error) => {
            return SmokeOutput {
                surface_id: SMOKE_SURFACE_ID,
                provider: provider_name,
                requested_model: args.model,
                default_configuration_id: SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID,
                rollback_configuration_id: SOFT_SEMANTIC_V3_CONFIGURATION_ID,
                fixture_count: fixtures.len(),
                attempted_provider_calls: 0,
                successful_provider_calls: 0,
                failure_counts: BTreeMap::from([(classify_model_kind(error.kind), 1)]),
                status: SmokeStatus::Failed,
                observations: Vec::new(),
                setup_failure_class: Some(classify_model_kind(error.kind)),
                setup_failure: Some(error.to_string()),
            };
        }
    };

    let mut observations = Vec::with_capacity(fixtures.len());
    let mut failure_counts = BTreeMap::new();
    let mut attempted_provider_calls = 0;
    let mut successful_provider_calls = 0;
    let mut overall = SmokeStatus::Passed;

    for (index, fixture) in fixtures.into_iter().enumerate() {
        let seed = args.seed + index as u64;

        attempted_provider_calls += 1;
        let default_d3 = observe_profile(
            SemanticRuntimeProfile::SemanticDecidabilityD3V1,
            true,
            generator.adapter(),
            &args.model,
            &fixture,
            args.max_tokens,
            seed,
        )
        .await;
        record_result(
            &default_d3,
            &mut successful_provider_calls,
            &mut failure_counts,
            &mut overall,
        );

        attempted_provider_calls += 1;
        let rollback_v3 = observe_profile(
            SemanticRuntimeProfile::SoftSemanticV3,
            false,
            generator.adapter(),
            &args.model,
            &fixture,
            args.max_tokens,
            seed,
        )
        .await;
        record_result(
            &rollback_v3,
            &mut successful_provider_calls,
            &mut failure_counts,
            &mut overall,
        );

        observations.push(FixtureObservation {
            fixture_id: fixture.id,
            expected_disposition: fixture.expected_disposition,
            seed,
            default_d3,
            rollback_v3,
        });
    }

    SmokeOutput {
        surface_id: SMOKE_SURFACE_ID,
        provider: provider_name,
        requested_model: args.model,
        default_configuration_id: SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID,
        rollback_configuration_id: SOFT_SEMANTIC_V3_CONFIGURATION_ID,
        fixture_count: observations.len(),
        attempted_provider_calls,
        successful_provider_calls,
        failure_counts,
        status: overall,
        observations,
        setup_failure_class: None,
        setup_failure: None,
    }
}

async fn observe_profile(
    profile: SemanticRuntimeProfile,
    use_default_entrypoint: bool,
    adapter: &dyn ModelAdapter,
    requested_model: &str,
    fixture: &SmokeFixture,
    max_tokens: u32,
    seed: u64,
) -> ProfileObservation {
    let started = Instant::now();
    let result = if use_default_entrypoint {
        run_default_semantic_runtime(
            adapter,
            requested_model,
            &fixture.request,
            &fixture.artifact,
            max_tokens,
            Some(seed),
        )
        .await
    } else {
        run_semantic_runtime(
            profile,
            adapter,
            requested_model,
            &fixture.request,
            &fixture.artifact,
            max_tokens,
            Some(seed),
        )
        .await
    };
    let latency_ms = started.elapsed().as_millis();

    match result {
        Ok(observation) => {
            let disposition = observation
                .decidability
                .as_ref()
                .map(|assessment| assessment.disposition);
            let valid = validate_observation(profile, fixture, &observation);
            ProfileObservation {
                profile,
                status: if valid {
                    SmokeStatus::Passed
                } else {
                    SmokeStatus::Failed
                },
                runtime: Some(observation.runtime),
                base_decision: Some(observation.base_decision),
                final_decision: Some(observation.observation.decision),
                disposition,
                observed_model: Some(observation.model),
                usage: Some(observation.usage),
                provider_attempts: Some(observation.provider_attempts),
                latency_ms,
                failure_class: (!valid)
                    .then_some(MaterializationFailureClass::MaterializationProtocol),
                failure: (!valid).then_some("runtime observation violated smoke invariants".into()),
            }
        }
        Err(error) => ProfileObservation {
            profile,
            status: SmokeStatus::Failed,
            runtime: None,
            base_decision: None,
            final_decision: None,
            disposition: None,
            observed_model: None,
            usage: None,
            provider_attempts: None,
            latency_ms,
            failure_class: Some(classify_runtime_error(&error)),
            failure: Some(error.to_string()),
        },
    }
}

fn validate_observation(
    profile: SemanticRuntimeProfile,
    fixture: &SmokeFixture,
    observation: &reasoning_harness_core::SemanticRuntimeObservation,
) -> bool {
    match profile {
        SemanticRuntimeProfile::SemanticDecidabilityD3V1 => {
            if observation.runtime.configuration_id() != SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID {
                return false;
            }
            let Some(decidability) = observation.decidability.as_ref() else {
                return false;
            };
            if decidability.disposition != fixture.expected_disposition {
                return false;
            }
            match fixture.expected_disposition {
                SemanticDecidabilityDisposition::Permit => {
                    observation.observation.decision == observation.base_decision
                }
                SemanticDecidabilityDisposition::ForceAbstain => {
                    observation.observation.decision == SoftJudgeDecision::Abstain
                }
            }
        }
        SemanticRuntimeProfile::SoftSemanticV3 => {
            observation.runtime.configuration_id() == SOFT_SEMANTIC_V3_CONFIGURATION_ID
                && observation.decidability.is_none()
        }
    }
}

fn record_result(
    observation: &ProfileObservation,
    successful_provider_calls: &mut usize,
    failure_counts: &mut BTreeMap<MaterializationFailureClass, usize>,
    overall: &mut SmokeStatus,
) {
    if observation.status == SmokeStatus::Passed {
        *successful_provider_calls += 1;
    } else {
        *overall = SmokeStatus::Failed;
        if let Some(class) = observation.failure_class {
            *failure_counts.entry(class).or_default() += 1;
        }
    }
}

fn load_fixtures(path: &PathBuf) -> Result<Vec<SmokeFixture>, String> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| format!("cannot read runtime smoke fixtures: {error}"))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("cannot enumerate runtime smoke fixtures: {error}"))?;
    entries.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    entries.sort();
    if entries.is_empty() {
        return Err("runtime smoke fixture directory is empty".into());
    }
    entries
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path)
                .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
            serde_json::from_slice(&bytes)
                .map_err(|error| format!("cannot parse {}: {error}", path.display()))
        })
        .collect()
}

fn classify_runtime_error(error: &SemanticRuntimeError) -> MaterializationFailureClass {
    match error {
        SemanticRuntimeError::InvalidRequestedModel | SemanticRuntimeError::Decidability(_) => {
            MaterializationFailureClass::StudySetup
        }
        SemanticRuntimeError::Materialization(error) => classify_materialization_failure(error),
        SemanticRuntimeError::Baseline(error) => match error.model_error_kind() {
            Some(kind) => classify_model_kind(kind),
            None => MaterializationFailureClass::MaterializationProtocol,
        },
    }
}

const fn classify_model_kind(kind: ModelErrorKind) -> MaterializationFailureClass {
    match kind {
        ModelErrorKind::Credentials => MaterializationFailureClass::Credentials,
        ModelErrorKind::Transport => MaterializationFailureClass::Transport,
        ModelErrorKind::Provider => MaterializationFailureClass::ProviderError,
        ModelErrorKind::RateLimit => MaterializationFailureClass::RateLimit,
        ModelErrorKind::Quota => MaterializationFailureClass::Quota,
        ModelErrorKind::ProviderUnavailable => MaterializationFailureClass::ProviderUnavailable,
        ModelErrorKind::Timeout => MaterializationFailureClass::Timeout,
        ModelErrorKind::Protocol => MaterializationFailureClass::ProviderProtocol,
        ModelErrorKind::UnsupportedCapability => MaterializationFailureClass::UnsupportedCapability,
    }
}

fn setup_failure(provider: &'static str, model: String, failure: String) -> SmokeOutput {
    SmokeOutput {
        surface_id: SMOKE_SURFACE_ID,
        provider,
        requested_model: model,
        default_configuration_id: SEMANTIC_DECIDABILITY_D3_CONFIGURATION_ID,
        rollback_configuration_id: SOFT_SEMANTIC_V3_CONFIGURATION_ID,
        fixture_count: 0,
        attempted_provider_calls: 0,
        successful_provider_calls: 0,
        failure_counts: BTreeMap::from([(MaterializationFailureClass::StudySetup, 1)]),
        status: SmokeStatus::Failed,
        observations: Vec::new(),
        setup_failure_class: Some(MaterializationFailureClass::StudySetup),
        setup_failure: Some(failure),
    }
}

const fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Mistral => "mistral",
        Provider::Google => "google",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_pair_keeps_model_visible_semantics_matched_and_gate_dispositions_distinct() {
        let fixture_root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/semantic-runtime-smoke");
        let fixtures = load_fixtures(&fixture_root).unwrap();
        assert_eq!(fixtures.len(), 2);

        let permit = &fixtures[0];
        let force = &fixtures[1];
        assert_eq!(
            permit.expected_disposition,
            SemanticDecidabilityDisposition::Permit
        );
        assert_eq!(
            force.expected_disposition,
            SemanticDecidabilityDisposition::ForceAbstain
        );

        assert_eq!(permit.request.task, force.request.task);
        assert_eq!(permit.request.kind, force.request.kind);
        assert_eq!(permit.request.target, force.request.target);
        assert_eq!(permit.request.context, force.request.context);

        for fixture in fixtures {
            let assessment =
                assess_semantic_decidability(&fixture.request, &fixture.artifact).unwrap();
            assert_eq!(assessment.disposition, fixture.expected_disposition);
        }
    }
}
