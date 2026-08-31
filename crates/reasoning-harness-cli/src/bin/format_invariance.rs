use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    CalibrationLabel, FormatFlipReport, FormatJudgeError, MatchedFormatDecision, ModelAdapter,
    ModelUsage, SoftJudgeCalibrationFixture, SoftJudgeDecision, SoftJudgeRepresentation,
    compare_soft_judge_formats, run_model_backed_soft_judge_representation,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "reason-format-study",
    about = "Calibration-only R1a semantic-judge format-invariance study"
)]
struct Args {
    /// Calibration fixture directory. Holdout directories are rejected.
    target: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long, default_value = "ministral-8b-latest")]
    model: String,
    /// Variant to compare against the implicit v3_full_json baseline. Repeatable.
    #[arg(long, value_enum, required = true)]
    representation: Vec<RepresentationArg>,
    /// Optional fixture IDs for bounded validation. Without this, all calibration fixtures run.
    #[arg(long = "fixture")]
    fixture_ids: Vec<String>,
    #[arg(long, default_value_t = 512)]
    max_tokens: u32,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, default_value_t = 1)]
    trials: usize,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Mistral,
    Google,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum RepresentationArg {
    NestedResultObject,
    DecisionFindingTuple,
    CompactKeyObject,
}

impl From<RepresentationArg> for SoftJudgeRepresentation {
    fn from(value: RepresentationArg) -> Self {
        match value {
            RepresentationArg::NestedResultObject => Self::NestedResultObject,
            RepresentationArg::DecisionFindingTuple => Self::DecisionFindingTuple,
            RepresentationArg::CompactKeyObject => Self::CompactKeyObject,
        }
    }
}

enum Generator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
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
        }
    }

    fn adapter(&self) -> &dyn ModelAdapter {
        match self {
            Self::Mistral(adapter) => adapter,
            Self::Google(adapter) => adapter,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct StudyCase {
    fixture_id: String,
    trial: usize,
    seed: Option<u64>,
    label: CalibrationLabel,
    representation: SoftJudgeRepresentation,
    execution_position: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision: Option<SoftJudgeDecision>,
    latency_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_class: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<String>,
}

#[derive(Debug, Serialize)]
struct TrialMetrics {
    trial: usize,
    expected_cases: usize,
    successful_cases: usize,
    operationally_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    precision: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recall: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision_coverage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambiguous_abstention: Option<f64>,
}

#[derive(Debug, Serialize)]
struct RepresentationReport {
    representation: SoftJudgeRepresentation,
    requested_output_format: &'static str,
    effective_enforcement_class: &'static str,
    attempted_runs: usize,
    successful_runs: usize,
    failed_runs: usize,
    protocol_completion_rate: f64,
    fallback_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_rate: Option<f64>,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    total_latency_ms: u128,
    mean_latency_ms: f64,
    trials: Vec<TrialMetrics>,
    cases: Vec<StudyCase>,
}

#[derive(Debug, Serialize)]
struct StudyOutput {
    configuration_id: &'static str,
    execution_design: &'static str,
    semantic_baseline: &'static str,
    provider: &'static str,
    model: String,
    fixture_count: usize,
    representations: Vec<RepresentationReport>,
    comparisons: Vec<FormatFlipReport>,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Args::parse()).await {
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

async fn run(args: Args) -> Result<StudyOutput, String> {
    if args.trials == 0 {
        return Err("--trials must be at least 1".into());
    }
    if let Some(seed) = args.seed {
        seed.checked_add((args.trials - 1) as u64)
            .ok_or("trial seed overflow")?;
    }
    let target = require_calibration_corpus(&args.target)?;

    let mut fixtures = load_fixtures(&target)?;
    if !args.fixture_ids.is_empty() {
        fixtures.retain(|fixture| args.fixture_ids.iter().any(|id| id == &fixture.id));
        for requested in &args.fixture_ids {
            if !fixtures.iter().any(|fixture| &fixture.id == requested) {
                return Err(format!(
                    "requested calibration fixture not found: {requested}"
                ));
            }
        }
    }
    if fixtures.is_empty() {
        return Err("no calibration fixtures selected".into());
    }

    let generator = Generator::from_provider(args.provider, &args.model)?;
    let mut representations = vec![SoftJudgeRepresentation::V3FullJson];
    for requested in args.representation {
        let representation = requested.into();
        if !representations.contains(&representation) {
            representations.push(representation);
        }
    }

    let reports = run_counterbalanced_representations(
        generator.adapter(),
        &fixtures,
        &representations,
        args.max_tokens,
        args.seed,
        args.trials,
        effective_enforcement_class(args.provider),
    )
    .await;

    let baseline_cases = matched_cases(&reports[0].cases);
    let mut comparisons = Vec::new();
    for report in reports.iter().skip(1) {
        comparisons.push(
            compare_soft_judge_formats(
                SoftJudgeRepresentation::V3FullJson,
                &baseline_cases,
                report.representation,
                &matched_cases(&report.cases),
            )
            .map_err(|error| error.to_string())?,
        );
    }

    Ok(StudyOutput {
        configuration_id: "format-invariance-r1a-v2",
        execution_design: "fixture_trial_rotating_representation_order_v1",
        semantic_baseline: "soft-semantic-v3",
        provider: provider_name(args.provider),
        model: args.model,
        fixture_count: fixtures.len(),
        representations: reports,
        comparisons,
    })
}

async fn run_counterbalanced_representations(
    adapter: &dyn ModelAdapter,
    fixtures: &[SoftJudgeCalibrationFixture],
    representations: &[SoftJudgeRepresentation],
    max_tokens: u32,
    seed: Option<u64>,
    trials: usize,
    effective_enforcement_class: &'static str,
) -> Vec<RepresentationReport> {
    let case_capacity = fixtures.len() * trials;
    let mut cases_by_representation = representations
        .iter()
        .copied()
        .map(|representation| (representation, Vec::with_capacity(case_capacity)))
        .collect::<BTreeMap<_, _>>();

    for trial in 0..trials {
        let trial_seed = seed.and_then(|base| base.checked_add(trial as u64));
        for (fixture_index, fixture) in fixtures.iter().enumerate() {
            let ordered =
                counterbalanced_representation_order(representations, fixture_index, trial);
            for (execution_position, representation) in ordered.into_iter().enumerate() {
                let started = Instant::now();
                let result = run_model_backed_soft_judge_representation(
                    adapter,
                    &fixture.request,
                    representation,
                    max_tokens,
                    trial_seed,
                )
                .await;
                let latency_ms = started.elapsed().as_millis();
                let case = match result {
                    Ok(result) => StudyCase {
                        fixture_id: fixture.id.clone(),
                        trial,
                        seed: trial_seed,
                        label: fixture.label,
                        representation,
                        execution_position,
                        decision: Some(result.decision),
                        latency_ms,
                        usage: Some(result.usage),
                        provider_model: Some(result.model),
                        finish_reason: result.finish_reason,
                        failure_class: None,
                        failure: None,
                    },
                    Err(error) => StudyCase {
                        fixture_id: fixture.id.clone(),
                        trial,
                        seed: trial_seed,
                        label: fixture.label,
                        representation,
                        execution_position,
                        decision: None,
                        latency_ms,
                        usage: error.usage().cloned(),
                        provider_model: error.provider_model().map(str::to_string),
                        finish_reason: error.finish_reason().map(str::to_string),
                        failure_class: Some(failure_class(&error)),
                        failure: Some(error.to_string()),
                    },
                };
                eprintln!(
                    "[format-study] fixture={} trial={} position={} representation={} status={}",
                    fixture.id,
                    trial + 1,
                    execution_position,
                    representation.id(),
                    if case.decision.is_some() {
                        "ok"
                    } else {
                        "failed"
                    }
                );
                cases_by_representation
                    .get_mut(&representation)
                    .expect("every requested representation has a case bucket")
                    .push(case);
            }
        }
    }

    representations
        .iter()
        .copied()
        .map(|representation| {
            summarize_representation(
                representation,
                cases_by_representation
                    .remove(&representation)
                    .expect("every requested representation has recorded cases"),
                fixtures.len(),
                trials,
                effective_enforcement_class,
            )
        })
        .collect()
}

fn counterbalanced_representation_order(
    representations: &[SoftJudgeRepresentation],
    fixture_index: usize,
    trial: usize,
) -> Vec<SoftJudgeRepresentation> {
    debug_assert!(!representations.is_empty());
    let offset = (fixture_index + trial) % representations.len();
    representations[offset..]
        .iter()
        .chain(&representations[..offset])
        .copied()
        .collect()
}

fn summarize_representation(
    representation: SoftJudgeRepresentation,
    cases: Vec<StudyCase>,
    fixtures_per_trial: usize,
    trials: usize,
    effective_enforcement_class: &'static str,
) -> RepresentationReport {
    let successful_runs = cases.iter().filter(|case| case.decision.is_some()).count();
    let attempted_runs = cases.len();
    let mut input_tokens = 0;
    let mut output_tokens = 0;
    let mut total_tokens = 0;
    for usage in cases.iter().filter_map(|case| case.usage.as_ref()) {
        input_tokens += usage.input_tokens.unwrap_or(0);
        output_tokens += usage.output_tokens.unwrap_or(0);
        total_tokens += usage.total_tokens.unwrap_or(0);
    }
    let total_latency_ms = cases.iter().map(|case| case.latency_ms).sum();
    let mut trial_metrics = Vec::with_capacity(trials);
    for trial in 0..trials {
        let trial_cases = cases
            .iter()
            .filter(|case| case.trial == trial)
            .collect::<Vec<_>>();
        let successful_cases = trial_cases
            .iter()
            .filter(|case| case.decision.is_some())
            .count();
        let complete = successful_cases == fixtures_per_trial;
        trial_metrics.push(if complete {
            semantic_trial_metrics(trial, &trial_cases)
        } else {
            TrialMetrics {
                trial,
                expected_cases: fixtures_per_trial,
                successful_cases,
                operationally_complete: false,
                precision: None,
                recall: None,
                decision_coverage: None,
                ambiguous_abstention: None,
            }
        });
    }

    RepresentationReport {
        representation,
        requested_output_format: representation.requested_output_format(),
        effective_enforcement_class,
        attempted_runs,
        successful_runs,
        failed_runs: attempted_runs - successful_runs,
        protocol_completion_rate: successful_runs as f64 / attempted_runs as f64,
        fallback_enabled: false,
        fallback_rate: None,
        input_tokens,
        output_tokens,
        total_tokens,
        total_latency_ms,
        mean_latency_ms: total_latency_ms as f64 / attempted_runs as f64,
        trials: trial_metrics,
        cases,
    }
}

fn semantic_trial_metrics(trial: usize, cases: &[&StudyCase]) -> TrialMetrics {
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut fn_ = 0usize;
    let mut decided = 0usize;
    let mut ambiguous = 0usize;
    let mut ambiguous_abstain = 0usize;

    for case in cases {
        let decision = case.decision.expect("complete trial has decisions");
        if decision != SoftJudgeDecision::Abstain {
            decided += 1;
        }
        match case.label {
            CalibrationLabel::Positive => {
                if decision == SoftJudgeDecision::Finding {
                    tp += 1;
                } else {
                    fn_ += 1;
                }
            }
            CalibrationLabel::Negative => {
                if decision == SoftJudgeDecision::Finding {
                    fp += 1;
                }
            }
            CalibrationLabel::Ambiguous => {
                ambiguous += 1;
                if decision == SoftJudgeDecision::Abstain {
                    ambiguous_abstain += 1;
                }
            }
        }
    }

    TrialMetrics {
        trial,
        expected_cases: cases.len(),
        successful_cases: cases.len(),
        operationally_complete: true,
        precision: ratio(tp, tp + fp),
        recall: ratio(tp, tp + fn_),
        decision_coverage: ratio(decided, cases.len()),
        ambiguous_abstention: ratio(ambiguous_abstain, ambiguous),
    }
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn matched_cases(cases: &[StudyCase]) -> Vec<MatchedFormatDecision> {
    cases
        .iter()
        .map(|case| MatchedFormatDecision {
            fixture_id: case.fixture_id.clone(),
            trial: case.trial,
            seed: case.seed,
            decision: case.decision,
        })
        .collect()
}

fn require_calibration_corpus(target: &Path) -> Result<PathBuf, String> {
    let target = target
        .canonicalize()
        .map_err(|error| format!("{}: {error}", target.display()))?;
    let expected = std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join("fixtures/semantic-judges")
        .canonicalize()
        .map_err(|error| format!("calibration corpus unavailable: {error}"))?;
    if target != expected {
        return Err(
            "R1 format study accepts only this checkout's fixtures/semantic-judges calibration corpus"
                .into(),
        );
    }
    Ok(target)
}

fn effective_enforcement_class(provider: Provider) -> &'static str {
    match provider {
        Provider::Mistral => "strict_json_schema",
        Provider::Google => "response_json_schema",
    }
}

fn load_fixtures(directory: &Path) -> Result<Vec<SoftJudgeCalibrationFixture>, String> {
    let mut paths = fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    paths.into_iter().map(|path| read_json(&path)).collect()
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn failure_class(error: &FormatJudgeError) -> &'static str {
    if matches!(error, FormatJudgeError::Setup(_)) {
        return "study_setup";
    }
    if error
        .finish_reason()
        .is_some_and(is_truncation_finish_reason)
    {
        return "truncation_protocol";
    }
    if error
        .finish_reason()
        .is_some_and(is_provider_generation_error_finish_reason)
    {
        return "provider_generation_error";
    }
    match error.model_error_kind() {
        Some(reasoning_harness_core::ModelErrorKind::Credentials) => "credentials",
        Some(reasoning_harness_core::ModelErrorKind::Transport) => "transport",
        Some(reasoning_harness_core::ModelErrorKind::Provider) => "provider_error",
        Some(reasoning_harness_core::ModelErrorKind::RateLimit) => "rate_limit",
        Some(reasoning_harness_core::ModelErrorKind::Quota) => "quota",
        Some(reasoning_harness_core::ModelErrorKind::ProviderUnavailable) => "provider_unavailable",
        Some(reasoning_harness_core::ModelErrorKind::Timeout) => "timeout",
        Some(reasoning_harness_core::ModelErrorKind::Protocol) => "provider_protocol",
        Some(reasoning_harness_core::ModelErrorKind::UnsupportedCapability) => {
            "unsupported_capability"
        }
        None => "representation_protocol",
    }
}

fn is_truncation_finish_reason(reason: &str) -> bool {
    matches!(
        reason.trim().to_ascii_lowercase().as_str(),
        "length" | "max_tokens" | "max_output_tokens"
    )
}

fn is_provider_generation_error_finish_reason(reason: &str) -> bool {
    reason.trim().eq_ignore_ascii_case("error")
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Mistral => "mistral",
        Provider::Google => "google",
    }
}

#[cfg(test)]
mod format_study_failure_tests {
    use super::*;

    fn invalid_with_finish_reason(reason: &str) -> FormatJudgeError {
        FormatJudgeError::InvalidRepresentation {
            message: "incomplete structured output".into(),
            model: "provider-model".into(),
            usage: ModelUsage::default(),
            finish_reason: Some(reason.into()),
        }
    }

    #[test]
    fn representation_order_rotates_across_fixtures_and_trials() {
        let representations = [
            SoftJudgeRepresentation::V3FullJson,
            SoftJudgeRepresentation::NestedResultObject,
            SoftJudgeRepresentation::DecisionFindingTuple,
            SoftJudgeRepresentation::CompactKeyObject,
        ];
        assert_eq!(
            counterbalanced_representation_order(&representations, 0, 0),
            representations
        );
        assert_eq!(
            counterbalanced_representation_order(&representations, 1, 0),
            vec![
                SoftJudgeRepresentation::NestedResultObject,
                SoftJudgeRepresentation::DecisionFindingTuple,
                SoftJudgeRepresentation::CompactKeyObject,
                SoftJudgeRepresentation::V3FullJson,
            ]
        );
        assert_eq!(
            counterbalanced_representation_order(&representations, 0, 1),
            counterbalanced_representation_order(&representations, 1, 0)
        );
    }

    #[test]
    fn finish_reason_classification_separates_provider_error_from_protocol_and_truncation() {
        assert_eq!(
            failure_class(&invalid_with_finish_reason("length")),
            "truncation_protocol"
        );
        assert_eq!(
            failure_class(&invalid_with_finish_reason("error")),
            "provider_generation_error"
        );
        assert_eq!(
            failure_class(&invalid_with_finish_reason("stop")),
            "representation_protocol"
        );
    }
}
