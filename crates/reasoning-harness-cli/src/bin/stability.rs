use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    CalibrationLabel, MaterializationError, MaterializationRepresentation, ModelAdapter,
    ModelErrorKind, ModelUsage, SelectiveAbstentionOutcome, SelectiveAbstentionPolicy,
    SoftDecisionProbe, SoftDecisionStabilityAssessment, SoftJudgeCalibrationFixture,
    SoftJudgeDecision, StabilityRiskSignal, apply_selective_abstention,
    assess_soft_decision_stability, run_model_backed_soft_judge_materialization_representation,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "reason-stability-study",
    about = "Calibration-only R3 semantic-judge stability/selective-abstention study"
)]
struct Args {
    target: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long, default_value = "gemini-3.5-flash-lite")]
    model: String,
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
    representation: MaterializationRepresentation,
    execution_position: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision: Option<SoftJudgeDecision>,
    latency_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_class: Option<&'static str>,
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
    representation: MaterializationRepresentation,
    attempted_runs: usize,
    successful_runs: usize,
    failed_runs: usize,
    protocol_completion_rate: f64,
    total_tokens: u64,
    total_latency_ms: u128,
    mean_latency_ms: f64,
    trials: Vec<TrialMetrics>,
    cases: Vec<StudyCase>,
}

#[derive(Debug, Serialize)]
struct FixtureStabilityReport {
    fixture_id: String,
    label: CalibrationLabel,
    assessment: SoftDecisionStabilityAssessment,
    disagreement_only: SelectiveAbstentionOutcome,
    complete_unanimity: SelectiveAbstentionOutcome,
}

#[derive(Debug, Serialize)]
struct SelectivePolicyMetrics {
    policy: SelectiveAbstentionPolicy,
    fixture_count: usize,
    risk_fixture_count: usize,
    operationally_incomplete_fixture_count: usize,
    escalated_to_abstain: usize,
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
struct StabilityReport {
    expected_probes_per_fixture: usize,
    fixtures: Vec<FixtureStabilityReport>,
    policies: Vec<SelectivePolicyMetrics>,
}

#[derive(Debug, Serialize)]
struct StudyOutput {
    configuration_id: &'static str,
    execution_design: &'static str,
    materialization_contract: &'static str,
    provider: &'static str,
    model: String,
    fixture_count: usize,
    representations: Vec<RepresentationReport>,
    combined_stability: StabilityReport,
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
    let reports = run_counterbalanced_representations(
        generator.adapter(),
        &fixtures,
        args.max_tokens,
        args.seed,
        args.trials,
    )
    .await;
    let combined_stability = summarize_combined_stability(&reports, args.trials)?;

    Ok(StudyOutput {
        configuration_id: "selective-abstention-r3-v1",
        execution_design: "fixture_trial_rotating_r2_representation_order_v1",
        materialization_contract: "decision_owned_by_model_binding_owned_by_harness_v1",
        provider: provider_name(args.provider),
        model: args.model,
        fixture_count: fixtures.len(),
        representations: reports,
        combined_stability,
    })
}

async fn run_counterbalanced_representations(
    adapter: &dyn ModelAdapter,
    fixtures: &[SoftJudgeCalibrationFixture],
    max_tokens: u32,
    seed: Option<u64>,
    trials: usize,
) -> Vec<RepresentationReport> {
    let capacity = fixtures.len() * trials;
    let mut cases_by_representation = MaterializationRepresentation::ALL
        .into_iter()
        .map(|representation| (representation, Vec::with_capacity(capacity)))
        .collect::<BTreeMap<_, _>>();

    for trial in 0..trials {
        let trial_seed = seed.and_then(|base| base.checked_add(trial as u64));
        for (fixture_index, fixture) in fixtures.iter().enumerate() {
            let ordered = counterbalanced_representation_order(fixture_index, trial);
            for (execution_position, representation) in ordered.into_iter().enumerate() {
                let started = Instant::now();
                let result = run_model_backed_soft_judge_materialization_representation(
                    adapter,
                    &fixture.request,
                    representation,
                    max_tokens,
                    trial_seed,
                )
                .await;
                let latency_ms = started.elapsed().as_millis();
                let case = match result {
                    Ok(observation) => StudyCase {
                        fixture_id: fixture.id.clone(),
                        trial,
                        seed: trial_seed,
                        label: fixture.label,
                        representation,
                        execution_position,
                        decision: Some(observation.decision),
                        latency_ms,
                        usage: Some(observation.usage),
                        finish_reason: observation.finish_reason,
                        failure_class: None,
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
                        finish_reason: error.finish_reason().map(str::to_string),
                        failure_class: Some(materialization_failure_class(&error)),
                    },
                };
                eprintln!(
                    "[stability-study] fixture={} trial={} position={} representation={} status={}",
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
                    .expect("every representation has a case bucket")
                    .push(case);
            }
        }
    }

    MaterializationRepresentation::ALL
        .into_iter()
        .map(|representation| {
            summarize_representation(
                representation,
                cases_by_representation
                    .remove(&representation)
                    .expect("every representation has recorded cases"),
                fixtures.len(),
                trials,
            )
        })
        .collect()
}

fn counterbalanced_representation_order(
    fixture_index: usize,
    trial: usize,
) -> Vec<MaterializationRepresentation> {
    let representations = MaterializationRepresentation::ALL;
    let offset = (fixture_index + trial) % representations.len();
    representations[offset..]
        .iter()
        .chain(&representations[..offset])
        .copied()
        .collect()
}

fn summarize_representation(
    representation: MaterializationRepresentation,
    cases: Vec<StudyCase>,
    fixtures_per_trial: usize,
    trials: usize,
) -> RepresentationReport {
    let attempted_runs = cases.len();
    let successful_runs = cases.iter().filter(|case| case.decision.is_some()).count();
    let total_tokens = cases
        .iter()
        .filter_map(|case| case.usage.as_ref().and_then(|usage| usage.total_tokens))
        .sum();
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
        trial_metrics.push(if successful_cases == fixtures_per_trial {
            semantic_metrics(trial, &trial_cases)
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
        attempted_runs,
        successful_runs,
        failed_runs: attempted_runs - successful_runs,
        protocol_completion_rate: successful_runs as f64 / attempted_runs as f64,
        total_tokens,
        total_latency_ms,
        mean_latency_ms: total_latency_ms as f64 / attempted_runs as f64,
        trials: trial_metrics,
        cases,
    }
}

fn semantic_metrics(trial: usize, cases: &[&StudyCase]) -> TrialMetrics {
    let decisions = cases
        .iter()
        .map(|case| (case.label, case.decision.expect("complete trial")));
    let metrics = metrics_from_decisions(decisions, cases.len());
    TrialMetrics {
        trial,
        expected_cases: cases.len(),
        successful_cases: cases.len(),
        operationally_complete: true,
        precision: metrics.precision,
        recall: metrics.recall,
        decision_coverage: metrics.decision_coverage,
        ambiguous_abstention: metrics.ambiguous_abstention,
    }
}

fn summarize_combined_stability(
    reports: &[RepresentationReport],
    trials: usize,
) -> Result<StabilityReport, String> {
    let mut by_fixture = BTreeMap::<String, (CalibrationLabel, Vec<SoftDecisionProbe>)>::new();
    for report in reports {
        for case in &report.cases {
            let entry = by_fixture
                .entry(case.fixture_id.clone())
                .or_insert_with(|| (case.label, Vec::new()));
            if entry.0 != case.label {
                return Err(format!("label mismatch for fixture {}", case.fixture_id));
            }
            entry.1.push(SoftDecisionProbe {
                probe_id: format!(
                    "{}:trial:{}:seed:{:?}",
                    report.representation.id(),
                    case.trial,
                    case.seed
                ),
                decision: case.decision,
            });
        }
    }

    let expected_probes_per_fixture = MaterializationRepresentation::ALL.len() * trials;
    let fixtures = by_fixture
        .into_iter()
        .map(|(fixture_id, (label, probes))| {
            let assessment = assess_soft_decision_stability(&probes);
            FixtureStabilityReport {
                fixture_id,
                label,
                disagreement_only: apply_selective_abstention(
                    &assessment,
                    SelectiveAbstentionPolicy::DisagreementOnly,
                ),
                complete_unanimity: apply_selective_abstention(
                    &assessment,
                    SelectiveAbstentionPolicy::CompleteUnanimity,
                ),
                assessment,
            }
        })
        .collect::<Vec<_>>();

    if fixtures
        .iter()
        .any(|fixture| fixture.assessment.expected_probes != expected_probes_per_fixture)
    {
        return Err("combined stability probe count mismatch".into());
    }

    let policies = [
        SelectiveAbstentionPolicy::DisagreementOnly,
        SelectiveAbstentionPolicy::CompleteUnanimity,
    ]
    .into_iter()
    .map(|policy| selective_policy_metrics(&fixtures, policy))
    .collect();

    Ok(StabilityReport {
        expected_probes_per_fixture,
        fixtures,
        policies,
    })
}

#[derive(Debug)]
struct DecisionMetrics {
    precision: Option<f64>,
    recall: Option<f64>,
    decision_coverage: Option<f64>,
    ambiguous_abstention: Option<f64>,
}

fn metrics_from_decisions(
    decisions: impl Iterator<Item = (CalibrationLabel, SoftJudgeDecision)>,
    total: usize,
) -> DecisionMetrics {
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut fn_ = 0usize;
    let mut decided = 0usize;
    let mut ambiguous = 0usize;
    let mut ambiguous_abstain = 0usize;
    for (label, decision) in decisions {
        if decision != SoftJudgeDecision::Abstain {
            decided += 1;
        }
        match label {
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
    DecisionMetrics {
        precision: ratio(tp, tp + fp),
        recall: ratio(tp, tp + fn_),
        decision_coverage: ratio(decided, total),
        ambiguous_abstention: ratio(ambiguous_abstain, ambiguous),
    }
}

fn selective_policy_metrics(
    fixtures: &[FixtureStabilityReport],
    policy: SelectiveAbstentionPolicy,
) -> SelectivePolicyMetrics {
    let decisions = fixtures.iter().map(|fixture| {
        let outcome = match policy {
            SelectiveAbstentionPolicy::DisagreementOnly => &fixture.disagreement_only,
            SelectiveAbstentionPolicy::CompleteUnanimity => &fixture.complete_unanimity,
        };
        (fixture.label, outcome.decision)
    });
    let metrics = metrics_from_decisions(decisions, fixtures.len());
    SelectivePolicyMetrics {
        policy,
        fixture_count: fixtures.len(),
        risk_fixture_count: fixtures
            .iter()
            .filter(|fixture| !fixture.assessment.risk_signals.is_empty())
            .count(),
        operationally_incomplete_fixture_count: fixtures
            .iter()
            .filter(|fixture| {
                fixture
                    .assessment
                    .risk_signals
                    .contains(&StabilityRiskSignal::OperationalIncomplete)
            })
            .count(),
        escalated_to_abstain: fixtures
            .iter()
            .filter(|fixture| {
                let outcome = match policy {
                    SelectiveAbstentionPolicy::DisagreementOnly => &fixture.disagreement_only,
                    SelectiveAbstentionPolicy::CompleteUnanimity => &fixture.complete_unanimity,
                };
                outcome.escalated_to_abstain
            })
            .count(),
        precision: metrics.precision,
        recall: metrics.recall,
        decision_coverage: metrics.decision_coverage,
        ambiguous_abstention: metrics.ambiguous_abstention,
    }
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
            "R3 stability study accepts only this checkout's fixtures/semantic-judges calibration corpus"
                .into(),
        );
    }
    Ok(target)
}

fn load_fixtures(directory: &Path) -> Result<Vec<SoftJudgeCalibrationFixture>, String> {
    let mut paths = fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    paths.retain(|path| path.extension().is_some_and(|ext| ext == "json"));
    paths.sort();
    paths.into_iter().map(|path| read_json(&path)).collect()
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

fn materialization_failure_class(error: &MaterializationError) -> &'static str {
    if matches!(error, MaterializationError::Setup(_)) {
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
        Some(ModelErrorKind::Credentials) => "credentials",
        Some(ModelErrorKind::Transport) => "transport",
        Some(ModelErrorKind::Provider) => "provider_error",
        Some(ModelErrorKind::RateLimit) => "rate_limit",
        Some(ModelErrorKind::Quota) => "quota",
        Some(ModelErrorKind::ProviderUnavailable) => "provider_unavailable",
        Some(ModelErrorKind::Timeout) => "timeout",
        Some(ModelErrorKind::Protocol) => "provider_protocol",
        Some(ModelErrorKind::UnsupportedCapability) => "unsupported_capability",
        None => "materialization_protocol",
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

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Mistral => "mistral",
        Provider::Google => "google",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn representation_order_rotates_without_changing_membership() {
        let first = counterbalanced_representation_order(0, 0);
        let second = counterbalanced_representation_order(1, 0);
        assert_eq!(first, MaterializationRepresentation::ALL);
        assert_eq!(
            second,
            vec![
                MaterializationRepresentation::CompactDecisionNoteObject,
                MaterializationRepresentation::NestedDecisionNoteObject,
                MaterializationRepresentation::DecisionNoteObject,
            ]
        );
    }
}
