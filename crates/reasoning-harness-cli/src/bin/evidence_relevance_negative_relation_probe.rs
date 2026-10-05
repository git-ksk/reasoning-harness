use std::{fs, path::PathBuf, process::ExitCode, time::Duration};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    EvidenceNegativeRelationConfirmation, EvidenceRelevanceCandidate,
    EvidenceRelevanceTargetPolicy, ModelAdapter, ModelErrorKind,
    build_evidence_negative_relation_confirmation_request,
    parse_evidence_negative_relation_confirmation,
};
use reasoning_harness_providers::{GoogleAdapter, GroqAdapter, MistralAdapter};
use serde::{Deserialize, Serialize};

const CONFIGURATION_ID: &str = "evidence-relevance-successor-v10-negative-relation-probe";
const SUITE_ID: &str = "evidence-relevance-successor-v10-development";
const EXPECTED_STATUS: &str = "fresh_independent_development";
const EXPECTED_CASES: usize = 16;

#[derive(Debug, Parser)]
#[command(
    name = "reason-evidence-relevance-negative-relation-probe",
    about = "Fresh one-sided negative-relation diagnostic for Issue #468"
)]
struct Args {
    target: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long)]
    model: String,
    #[arg(long, default_value_t = 3)]
    trials: usize,
    #[arg(long, default_value_t = 4681000)]
    seed: u64,
    #[arg(long, default_value_t = 0)]
    inter_case_delay_ms: u64,
    #[arg(long, default_value_t = false)]
    validate_only: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Mistral,
    Google,
    Groq,
}

impl Provider {
    fn name(self) -> &'static str {
        match self {
            Self::Mistral => "mistral",
            Self::Google => "google",
            Self::Groq => "groq",
        }
    }
}

enum Generator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
    Groq(GroqAdapter),
}

impl Generator {
    fn from_provider(provider: Provider, model: &str) -> Result<Self, String> {
        match provider {
            Provider::Mistral => MistralAdapter::from_env(model)
                .map(Self::Mistral)
                .map_err(|error| error.to_string()),
            Provider::Google => GoogleAdapter::from_env(model)
                .map(Self::Google)
                .map_err(|error| error.to_string()),
            Provider::Groq => GroqAdapter::from_env(model)
                .map(Self::Groq)
                .map_err(|error| error.to_string()),
        }
    }

    fn adapter(&self) -> &dyn ModelAdapter {
        match self {
            Self::Mistral(adapter) => adapter,
            Self::Google(adapter) => adapter,
            Self::Groq(adapter) => adapter,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Manifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    cases: Vec<ProbeCase>,
}

#[derive(Debug, Deserialize)]
struct ProbeCase {
    id: String,
    family: String,
    policy: EvidenceRelevanceTargetPolicy,
    candidate: EvidenceRelevanceCandidate,
    expected_confirmation: EvidenceNegativeRelationConfirmation,
}

#[derive(Debug, Serialize)]
struct Observation {
    case_id: String,
    family: String,
    trial: usize,
    seed: u64,
    expected_confirmation: EvidenceNegativeRelationConfirmation,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_confirmation: Option<EvidenceNegativeRelationConfirmation>,
    confirmation_match: bool,
    provider_attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<String>,
}

#[derive(Debug, Serialize)]
struct Metrics {
    observations: usize,
    successful_observations: usize,
    failed_observations: usize,
    confirmation_matches: usize,
    false_confirmations: usize,
    missed_confirmations: usize,
}

#[derive(Debug, Serialize)]
struct Output {
    configuration_id: &'static str,
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    provider: String,
    model: String,
    trials: usize,
    seed: u64,
    metrics: Metrics,
    observations: Vec<Observation>,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(output) => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<Output, String> {
    let args = Args::parse();
    if args.trials == 0 {
        return Err("--trials must be at least 1".into());
    }

    let manifest_path = args.target.join("manifest.json");
    let manifest: Manifest = serde_json::from_slice(
        &fs::read(&manifest_path)
            .map_err(|error| format!("read {}: {error}", manifest_path.display()))?,
    )
    .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;

    if manifest.suite_id != SUITE_ID
        || manifest.status != EXPECTED_STATUS
        || manifest.issue != 468
        || manifest.cases.len() != EXPECTED_CASES
    {
        return Err("unexpected successor-v10 negative-relation manifest identity".into());
    }

    if args.validate_only {
        return Ok(Output {
            configuration_id: CONFIGURATION_ID,
            suite_id: manifest.suite_id,
            issue: manifest.issue,
            status: manifest.status,
            source_rule: manifest.source_rule,
            provider: args.provider.name().into(),
            model: args.model,
            trials: args.trials,
            seed: args.seed,
            metrics: summarize(&[]),
            observations: Vec::new(),
        });
    }

    let generator = Generator::from_provider(args.provider, &args.model)?;
    let mut observations = Vec::with_capacity(manifest.cases.len() * args.trials);

    for trial in 0..args.trials {
        for (case_index, case) in manifest.cases.iter().enumerate() {
            let seed = args
                .seed
                .checked_add((trial * manifest.cases.len() + case_index) as u64)
                .ok_or("probe seed overflow")?;
            observations.push(observe_case(generator.adapter(), case, trial, seed).await);
            if args.inter_case_delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(args.inter_case_delay_ms)).await;
            }
        }
    }

    Ok(Output {
        configuration_id: CONFIGURATION_ID,
        suite_id: manifest.suite_id,
        issue: manifest.issue,
        status: manifest.status,
        source_rule: manifest.source_rule,
        provider: args.provider.name().into(),
        model: args.model,
        trials: args.trials,
        seed: args.seed,
        metrics: summarize(&observations),
        observations,
    })
}

async fn observe_case(
    adapter: &dyn ModelAdapter,
    case: &ProbeCase,
    trial: usize,
    seed: u64,
) -> Observation {
    let request = match build_evidence_negative_relation_confirmation_request(
        &case.policy,
        &case.candidate,
        Some(seed),
    ) {
        Ok(request) => request,
        Err(error) => {
            return failure_observation(case, trial, seed, "request", error.to_string(), 0);
        }
    };

    let result = tokio::time::timeout(
        Duration::from_millis(case.policy.assessment_budget.max_elapsed_ms),
        adapter.generate(request),
    )
    .await;

    match result {
        Err(_) => failure_observation(
            case,
            trial,
            seed,
            "assessment_timeout",
            "negative-relation probe deadline exceeded".into(),
            0,
        ),
        Ok(Err(error)) => failure_observation(
            case,
            trial,
            seed,
            model_error_class(error.kind),
            error.message,
            error.provider_attempts,
        ),
        Ok(Ok(response)) => match parse_evidence_negative_relation_confirmation(&response.text) {
            Ok(observed) => Observation {
                case_id: case.id.clone(),
                family: case.family.clone(),
                trial,
                seed,
                expected_confirmation: case.expected_confirmation,
                observed_confirmation: Some(observed),
                confirmation_match: observed == case.expected_confirmation,
                provider_attempts: response.provider_attempts,
                failure_class: None,
                failure: None,
            },
            Err(error) => failure_observation(
                case,
                trial,
                seed,
                "protocol",
                error.to_string(),
                response.provider_attempts,
            ),
        },
    }
}

fn failure_observation(
    case: &ProbeCase,
    trial: usize,
    seed: u64,
    class: &str,
    message: String,
    provider_attempts: u32,
) -> Observation {
    Observation {
        case_id: case.id.clone(),
        family: case.family.clone(),
        trial,
        seed,
        expected_confirmation: case.expected_confirmation,
        observed_confirmation: None,
        confirmation_match: false,
        provider_attempts,
        failure_class: Some(class.into()),
        failure: Some(message),
    }
}

fn summarize(observations: &[Observation]) -> Metrics {
    let successful = observations
        .iter()
        .filter(|item| item.failure_class.is_none())
        .count();
    let confirmation_matches = observations
        .iter()
        .filter(|item| item.confirmation_match)
        .count();
    let false_confirmations = observations
        .iter()
        .filter(|item| {
            item.expected_confirmation == EvidenceNegativeRelationConfirmation::NotConfirmed
                && item.observed_confirmation
                    == Some(EvidenceNegativeRelationConfirmation::ConfirmedDifferentRelation)
        })
        .count();
    let missed_confirmations = observations
        .iter()
        .filter(|item| {
            item.expected_confirmation
                == EvidenceNegativeRelationConfirmation::ConfirmedDifferentRelation
                && item.observed_confirmation
                    == Some(EvidenceNegativeRelationConfirmation::NotConfirmed)
        })
        .count();

    Metrics {
        observations: observations.len(),
        successful_observations: successful,
        failed_observations: observations.len().saturating_sub(successful),
        confirmation_matches,
        false_confirmations,
        missed_confirmations,
    }
}

fn model_error_class(kind: ModelErrorKind) -> &'static str {
    match kind {
        ModelErrorKind::Credentials => "credentials",
        ModelErrorKind::Transport => "transport",
        ModelErrorKind::Provider => "provider",
        ModelErrorKind::RateLimit => "rate_limit",
        ModelErrorKind::Quota => "quota",
        ModelErrorKind::ProviderUnavailable => "provider_unavailable",
        ModelErrorKind::Timeout => "timeout",
        ModelErrorKind::Protocol => "protocol",
        ModelErrorKind::UnsupportedCapability => "unsupported_capability",
    }
}
