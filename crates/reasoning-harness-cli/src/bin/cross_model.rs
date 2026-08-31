use std::{
    collections::{BTreeMap, BTreeSet},
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
    name = "reason-cross-model-study",
    about = "Calibration-only R3b cross-model semantic-judge stability study"
)]
struct Args {
    /// Study corpus directory. Must exactly match the selected corpus identity.
    target: PathBuf,
    #[arg(long, value_enum, default_value_t = Corpus::Calibration)]
    corpus: Corpus,
    /// Provider/model source in `provider:model` form. Repeat at least twice.
    #[arg(long = "source", required = true)]
    source_specs: Vec<String>,
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
enum Corpus {
    Calibration,
    HoldoutV4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Provider {
    Mistral,
    Google,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SourceSpec {
    provider: Provider,
    model: String,
}

impl SourceSpec {
    fn parse(value: &str) -> Result<Self, String> {
        let (provider, model) = value
            .split_once(':')
            .ok_or_else(|| format!("invalid --source {value:?}; expected provider:model"))?;
        let provider = match provider.trim().to_ascii_lowercase().as_str() {
            "mistral" => Provider::Mistral,
            "google" => Provider::Google,
            other => return Err(format!("unsupported cross-model provider: {other}")),
        };
        let model = model.trim();
        if model.is_empty() {
            return Err("cross-model source model must not be empty".into());
        }
        Ok(Self {
            provider,
            model: model.into(),
        })
    }

    fn id(&self) -> String {
        format!("{}:{}", provider_name(self.provider), self.model)
    }
}

enum Generator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
}

impl Generator {
    fn from_spec(spec: &SourceSpec) -> Result<Self, String> {
        match spec.provider {
            Provider::Mistral => MistralAdapter::from_env(&spec.model)
                .map(Self::Mistral)
                .map_err(|error| error.to_string()),
            Provider::Google => GoogleAdapter::from_env(&spec.model)
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

struct SourceRunner {
    spec: SourceSpec,
    generator: Generator,
}

#[derive(Debug, Clone, Serialize)]
struct StudyCase {
    source_id: String,
    provider: &'static str,
    model: String,
    fixture_id: String,
    trial: usize,
    seed: Option<u64>,
    label: CalibrationLabel,
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
    clear_case_coverage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambiguous_abstention: Option<f64>,
}

#[derive(Debug, Serialize)]
struct SourceReport {
    source_id: String,
    provider: &'static str,
    model: String,
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
struct TrialCrossModelReport {
    trial: usize,
    seed: Option<u64>,
    assessment: SoftDecisionStabilityAssessment,
    disagreement_only: SelectiveAbstentionOutcome,
    complete_unanimity: SelectiveAbstentionOutcome,
}

#[derive(Debug, Serialize)]
struct FixtureCrossModelReport {
    fixture_id: String,
    label: CalibrationLabel,
    per_trial: Vec<TrialCrossModelReport>,
    combined_assessment: SoftDecisionStabilityAssessment,
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
    clear_case_coverage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ambiguous_abstention: Option<f64>,
}

#[derive(Debug, Serialize)]
struct CrossModelStabilityReport {
    source_count: usize,
    trials: usize,
    expected_probes_per_fixture: usize,
    fixtures: Vec<FixtureCrossModelReport>,
    policies: Vec<SelectivePolicyMetrics>,
}

#[derive(Debug, Serialize)]
struct StudyOutput {
    configuration_id: &'static str,
    corpus: &'static str,
    execution_design: &'static str,
    materialization_contract: &'static str,
    representation: MaterializationRepresentation,
    fixture_count: usize,
    source_count: usize,
    sources: Vec<SourceReport>,
    combined_cross_model_stability: CrossModelStabilityReport,
}

#[derive(Debug)]
struct DecisionMetrics {
    precision: Option<f64>,
    recall: Option<f64>,
    decision_coverage: Option<f64>,
    clear_case_coverage: Option<f64>,
    ambiguous_abstention: Option<f64>,
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

    // Corpus identity is checked before adapters touch credentials.
    let target = require_study_corpus(&args.target, args.corpus)?;
    let specs = parse_source_specs(&args.source_specs)?;
    let mut fixtures = load_fixtures(&target)?;
    select_fixtures(&mut fixtures, &args.fixture_ids)?;

    let mut runners = Vec::with_capacity(specs.len());
    for spec in specs {
        let generator = Generator::from_spec(&spec)?;
        runners.push(SourceRunner { spec, generator });
    }

    let reports =
        run_counterbalanced_sources(&runners, &fixtures, args.max_tokens, args.seed, args.trials)
            .await;
    let combined_cross_model_stability = summarize_cross_model_stability(&reports, args.trials)?;

    Ok(StudyOutput {
        // Corpus identity changes the evaluation surface, not the frozen semantic candidate.
        configuration_id: "cross-model-selective-abstention-r3b-v1",
        corpus: corpus_name(args.corpus),
        execution_design: "fixture_trial_rotating_source_order_v1",
        materialization_contract: "decision_owned_by_model_binding_owned_by_harness_v1",
        representation: MaterializationRepresentation::DecisionNoteObject,
        fixture_count: fixtures.len(),
        source_count: runners.len(),
        sources: reports,
        combined_cross_model_stability,
    })
}

fn parse_source_specs(values: &[String]) -> Result<Vec<SourceSpec>, String> {
    if values.len() < 2 {
        return Err("R3b cross-model study requires at least two --source values".into());
    }
    let specs = values
        .iter()
        .map(|value| SourceSpec::parse(value))
        .collect::<Result<Vec<_>, _>>()?;
    let unique = specs.iter().map(SourceSpec::id).collect::<BTreeSet<_>>();
    if unique.len() != specs.len() {
        return Err("R3b cross-model study rejects duplicate provider/model sources".into());
    }
    Ok(specs)
}

fn select_fixtures(
    fixtures: &mut Vec<SoftJudgeCalibrationFixture>,
    requested_ids: &[String],
) -> Result<(), String> {
    if !requested_ids.is_empty() {
        fixtures.retain(|fixture| requested_ids.iter().any(|id| id == &fixture.id));
        for requested in requested_ids {
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
    Ok(())
}

async fn run_counterbalanced_sources(
    sources: &[SourceRunner],
    fixtures: &[SoftJudgeCalibrationFixture],
    max_tokens: u32,
    seed: Option<u64>,
    trials: usize,
) -> Vec<SourceReport> {
    let capacity = fixtures.len() * trials;
    let mut cases_by_source = sources
        .iter()
        .map(|source| (source.spec.id(), Vec::with_capacity(capacity)))
        .collect::<BTreeMap<_, _>>();

    for trial in 0..trials {
        let trial_seed = seed.and_then(|base| base.checked_add(trial as u64));
        for (fixture_index, fixture) in fixtures.iter().enumerate() {
            for (execution_position, source_index) in
                counterbalanced_source_order(sources.len(), fixture_index, trial)
                    .into_iter()
                    .enumerate()
            {
                let source = &sources[source_index];
                let source_id = source.spec.id();
                let started = Instant::now();
                let result = run_model_backed_soft_judge_materialization_representation(
                    source.generator.adapter(),
                    &fixture.request,
                    MaterializationRepresentation::DecisionNoteObject,
                    max_tokens,
                    trial_seed,
                )
                .await;
                let latency_ms = started.elapsed().as_millis();
                let case = match result {
                    Ok(observation) => StudyCase {
                        source_id: source_id.clone(),
                        provider: provider_name(source.spec.provider),
                        model: source.spec.model.clone(),
                        fixture_id: fixture.id.clone(),
                        trial,
                        seed: trial_seed,
                        label: fixture.label,
                        execution_position,
                        decision: Some(observation.decision),
                        latency_ms,
                        usage: Some(observation.usage),
                        finish_reason: observation.finish_reason,
                        failure_class: None,
                    },
                    Err(error) => StudyCase {
                        source_id: source_id.clone(),
                        provider: provider_name(source.spec.provider),
                        model: source.spec.model.clone(),
                        fixture_id: fixture.id.clone(),
                        trial,
                        seed: trial_seed,
                        label: fixture.label,
                        execution_position,
                        decision: None,
                        latency_ms,
                        usage: error.usage().cloned(),
                        finish_reason: error.finish_reason().map(str::to_string),
                        failure_class: Some(materialization_failure_class(&error)),
                    },
                };
                eprintln!(
                    "[cross-model-study] fixture={} trial={} position={} source={} status={}",
                    fixture.id,
                    trial + 1,
                    execution_position,
                    source_id,
                    if case.decision.is_some() {
                        "ok"
                    } else {
                        "failed"
                    }
                );
                cases_by_source
                    .get_mut(&source_id)
                    .expect("every source has a case bucket")
                    .push(case);
            }
        }
    }

    sources
        .iter()
        .map(|source| {
            let source_id = source.spec.id();
            summarize_source(
                &source.spec,
                cases_by_source
                    .remove(&source_id)
                    .expect("every source has recorded cases"),
                fixtures.len(),
                trials,
            )
        })
        .collect()
}

fn counterbalanced_source_order(
    source_count: usize,
    fixture_index: usize,
    trial: usize,
) -> Vec<usize> {
    debug_assert!(source_count > 0);
    let offset = (fixture_index + trial) % source_count;
    (offset..source_count).chain(0..offset).collect()
}

fn summarize_source(
    spec: &SourceSpec,
    cases: Vec<StudyCase>,
    fixtures_per_trial: usize,
    trials: usize,
) -> SourceReport {
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
                clear_case_coverage: None,
                ambiguous_abstention: None,
            }
        });
    }

    SourceReport {
        source_id: spec.id(),
        provider: provider_name(spec.provider),
        model: spec.model.clone(),
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

fn semantic_trial_metrics(trial: usize, cases: &[&StudyCase]) -> TrialMetrics {
    let metrics = metrics_from_decisions(
        cases
            .iter()
            .map(|case| (case.label, case.decision.expect("complete source trial"))),
        cases.len(),
    );
    TrialMetrics {
        trial,
        expected_cases: cases.len(),
        successful_cases: cases.len(),
        operationally_complete: true,
        precision: metrics.precision,
        recall: metrics.recall,
        decision_coverage: metrics.decision_coverage,
        clear_case_coverage: metrics.clear_case_coverage,
        ambiguous_abstention: metrics.ambiguous_abstention,
    }
}

fn summarize_cross_model_stability(
    reports: &[SourceReport],
    trials: usize,
) -> Result<CrossModelStabilityReport, String> {
    let mut by_fixture =
        BTreeMap::<String, (CalibrationLabel, BTreeMap<usize, Vec<SoftDecisionProbe>>)>::new();
    for report in reports {
        for case in &report.cases {
            let entry = by_fixture
                .entry(case.fixture_id.clone())
                .or_insert_with(|| (case.label, BTreeMap::new()));
            if entry.0 != case.label {
                return Err(format!("label mismatch for fixture {}", case.fixture_id));
            }
            entry
                .1
                .entry(case.trial)
                .or_default()
                .push(SoftDecisionProbe {
                    probe_id: format!(
                        "{}:trial:{}:seed:{:?}",
                        case.source_id, case.trial, case.seed
                    ),
                    decision: case.decision,
                });
        }
    }

    let expected_probes_per_fixture = reports.len() * trials;
    let mut fixtures = Vec::with_capacity(by_fixture.len());
    for (fixture_id, (label, probes_by_trial)) in by_fixture {
        let mut per_trial = Vec::with_capacity(trials);
        let mut combined = Vec::with_capacity(expected_probes_per_fixture);
        for trial in 0..trials {
            let probes = probes_by_trial
                .get(&trial)
                .ok_or_else(|| format!("missing trial {trial} for fixture {fixture_id}"))?;
            if probes.len() != reports.len() {
                return Err(format!(
                    "cross-model source count mismatch for fixture {fixture_id} trial {trial}"
                ));
            }
            combined.extend(probes.iter().cloned());
            let assessment = assess_soft_decision_stability(probes);
            let seed = reports
                .first()
                .and_then(|report| {
                    report
                        .cases
                        .iter()
                        .find(|case| case.fixture_id == fixture_id && case.trial == trial)
                })
                .and_then(|case| case.seed);
            per_trial.push(TrialCrossModelReport {
                trial,
                seed,
                disagreement_only: apply_selective_abstention(
                    &assessment,
                    SelectiveAbstentionPolicy::DisagreementOnly,
                ),
                complete_unanimity: apply_selective_abstention(
                    &assessment,
                    SelectiveAbstentionPolicy::CompleteUnanimity,
                ),
                assessment,
            });
        }
        let combined_assessment = assess_soft_decision_stability(&combined);
        if combined_assessment.expected_probes != expected_probes_per_fixture {
            return Err(format!(
                "combined probe count mismatch for fixture {fixture_id}"
            ));
        }
        fixtures.push(FixtureCrossModelReport {
            fixture_id,
            label,
            disagreement_only: apply_selective_abstention(
                &combined_assessment,
                SelectiveAbstentionPolicy::DisagreementOnly,
            ),
            complete_unanimity: apply_selective_abstention(
                &combined_assessment,
                SelectiveAbstentionPolicy::CompleteUnanimity,
            ),
            per_trial,
            combined_assessment,
        });
    }

    let policies = [
        SelectiveAbstentionPolicy::DisagreementOnly,
        SelectiveAbstentionPolicy::CompleteUnanimity,
    ]
    .into_iter()
    .map(|policy| selective_policy_metrics(&fixtures, policy))
    .collect();

    Ok(CrossModelStabilityReport {
        source_count: reports.len(),
        trials,
        expected_probes_per_fixture,
        fixtures,
        policies,
    })
}

fn selective_policy_metrics(
    fixtures: &[FixtureCrossModelReport],
    policy: SelectiveAbstentionPolicy,
) -> SelectivePolicyMetrics {
    let decisions = fixtures.iter().map(|fixture| {
        let outcome = policy_outcome(fixture, policy);
        (fixture.label, outcome.decision)
    });
    let metrics = metrics_from_decisions(decisions, fixtures.len());
    SelectivePolicyMetrics {
        policy,
        fixture_count: fixtures.len(),
        risk_fixture_count: fixtures
            .iter()
            .filter(|fixture| !fixture.combined_assessment.risk_signals.is_empty())
            .count(),
        operationally_incomplete_fixture_count: fixtures
            .iter()
            .filter(|fixture| {
                fixture
                    .combined_assessment
                    .risk_signals
                    .contains(&StabilityRiskSignal::OperationalIncomplete)
            })
            .count(),
        escalated_to_abstain: fixtures
            .iter()
            .filter(|fixture| policy_outcome(fixture, policy).escalated_to_abstain)
            .count(),
        precision: metrics.precision,
        recall: metrics.recall,
        decision_coverage: metrics.decision_coverage,
        clear_case_coverage: metrics.clear_case_coverage,
        ambiguous_abstention: metrics.ambiguous_abstention,
    }
}

fn policy_outcome(
    fixture: &FixtureCrossModelReport,
    policy: SelectiveAbstentionPolicy,
) -> &SelectiveAbstentionOutcome {
    match policy {
        SelectiveAbstentionPolicy::DisagreementOnly => &fixture.disagreement_only,
        SelectiveAbstentionPolicy::CompleteUnanimity => &fixture.complete_unanimity,
    }
}

fn metrics_from_decisions(
    decisions: impl Iterator<Item = (CalibrationLabel, SoftJudgeDecision)>,
    total: usize,
) -> DecisionMetrics {
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut fn_ = 0usize;
    let mut decided = 0usize;
    let mut clear_cases = 0usize;
    let mut clear_decided = 0usize;
    let mut ambiguous = 0usize;
    let mut ambiguous_abstain = 0usize;
    for (label, decision) in decisions {
        if decision != SoftJudgeDecision::Abstain {
            decided += 1;
        }
        if label != CalibrationLabel::Ambiguous {
            clear_cases += 1;
            if decision != SoftJudgeDecision::Abstain {
                clear_decided += 1;
            }
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
        clear_case_coverage: ratio(clear_decided, clear_cases),
        ambiguous_abstention: ratio(ambiguous_abstain, ambiguous),
    }
}

fn require_study_corpus(target: &Path, corpus: Corpus) -> Result<PathBuf, String> {
    let target = target
        .canonicalize()
        .map_err(|error| format!("{}: {error}", target.display()))?;
    let relative = match corpus {
        Corpus::Calibration => "fixtures/semantic-judges",
        Corpus::HoldoutV4 => "fixtures/semantic-judges-holdout-v4",
    };
    let expected = std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join(relative)
        .canonicalize()
        .map_err(|error| format!("{relative} corpus unavailable: {error}"))?;
    if target != expected {
        return Err(format!(
            "cross-model study corpus mismatch: --corpus {} requires this checkout's {relative}",
            corpus_name(corpus)
        ));
    }
    Ok(target)
}

fn corpus_name(corpus: Corpus) -> &'static str {
    match corpus {
        Corpus::Calibration => "calibration",
        Corpus::HoldoutV4 => "holdout_v4",
    }
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
    fn source_spec_parser_accepts_supported_provider_model_pairs() {
        let google = SourceSpec::parse("google:gemini-3.5-flash-lite").unwrap();
        assert_eq!(google.provider, Provider::Google);
        assert_eq!(google.model, "gemini-3.5-flash-lite");
        assert_eq!(google.id(), "google:gemini-3.5-flash-lite");

        let mistral = SourceSpec::parse("mistral:ministral-8b-latest").unwrap();
        assert_eq!(mistral.provider, Provider::Mistral);
        assert_eq!(mistral.id(), "mistral:ministral-8b-latest");
    }

    #[test]
    fn source_spec_validation_requires_two_distinct_sources() {
        assert!(parse_source_specs(&["google:x".into()]).is_err());
        assert!(parse_source_specs(&["google:x".into(), "google:x".into()]).is_err());
        assert!(parse_source_specs(&["unknown:x".into(), "google:y".into()]).is_err());
    }

    #[test]
    fn source_order_rotates_without_changing_membership() {
        assert_eq!(counterbalanced_source_order(3, 0, 0), vec![0, 1, 2]);
        assert_eq!(counterbalanced_source_order(3, 1, 0), vec![1, 2, 0]);
        assert_eq!(counterbalanced_source_order(3, 0, 2), vec![2, 0, 1]);
    }

    #[test]
    fn clear_case_coverage_counts_only_positive_and_negative_cases() {
        let metrics = metrics_from_decisions(
            [
                (CalibrationLabel::Positive, SoftJudgeDecision::Finding),
                (CalibrationLabel::Negative, SoftJudgeDecision::Abstain),
                (CalibrationLabel::Ambiguous, SoftJudgeDecision::Abstain),
            ]
            .into_iter(),
            3,
        );
        assert_eq!(metrics.decision_coverage, Some(1.0 / 3.0));
        assert_eq!(metrics.clear_case_coverage, Some(0.5));
        assert_eq!(metrics.ambiguous_abstention, Some(1.0));
    }
}
