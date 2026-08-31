use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    CalibrationLabel, FormatJudgeError, MaterializationError, ModelAdapter, ModelErrorKind,
    ModelUsage, SelectiveAbstentionOutcome, SelectiveAbstentionPolicy, SoftDecisionProbe,
    SoftDecisionStabilityAssessment, SoftJudgeCalibrationFixture, SoftJudgeDecision,
    SoftJudgeRepresentation, StabilityRiskSignal, apply_selective_abstention,
    assess_soft_decision_stability, run_model_backed_soft_judge_materialization,
    run_model_backed_soft_judge_representation,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "reason-materialization-study",
    about = "Calibration-only R2 semantic-judge materialization study"
)]
struct Args {
    /// Calibration fixture directory. Holdout directories are rejected.
    target: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long, default_value = "gemini-3.5-flash-lite")]
    model: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum StudyArm {
    V3FullJson,
    HarnessMaterializedDecision,
}

impl StudyArm {
    const ALL: [Self; 2] = [Self::V3FullJson, Self::HarnessMaterializedDecision];
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
    arm: StudyArm,
    execution_position: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision: Option<SoftJudgeDecision>,
    advisory_note_present: bool,
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
struct ArmReport {
    arm: StudyArm,
    attempted_runs: usize,
    successful_runs: usize,
    failed_runs: usize,
    protocol_completion_rate: f64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    total_latency_ms: u128,
    mean_latency_ms: f64,
    advisory_note_runs: usize,
    trials: Vec<TrialMetrics>,
    cases: Vec<StudyCase>,
}

#[derive(Debug, Serialize)]
struct DecisionTransition {
    from: SoftJudgeDecision,
    to: SoftJudgeDecision,
    count: usize,
}

#[derive(Debug, Serialize)]
struct MaterializationComparison {
    baseline: StudyArm,
    materialized: StudyArm,
    matched_keys: usize,
    matched_successful_pairs: usize,
    operationally_incomplete_pairs: usize,
    changed_decisions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision_flip_rate: Option<f64>,
    transitions: Vec<DecisionTransition>,
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
struct ArmStabilityReport {
    arm: StudyArm,
    fixtures: Vec<FixtureStabilityReport>,
    policies: Vec<SelectivePolicyMetrics>,
}

#[derive(Debug, Serialize)]
struct StudyOutput {
    configuration_id: &'static str,
    execution_design: &'static str,
    semantic_baseline: &'static str,
    materialization_contract: &'static str,
    provider: &'static str,
    model: String,
    fixture_count: usize,
    effective_enforcement_class: &'static str,
    arms: Vec<ArmReport>,
    comparison: MaterializationComparison,
    stability: Vec<ArmStabilityReport>,
}

#[derive(Debug)]
struct RunObservation {
    decision: SoftJudgeDecision,
    advisory_note_present: bool,
    usage: ModelUsage,
    provider_model: String,
    finish_reason: Option<String>,
}

#[derive(Debug)]
struct FailureInfo {
    usage: Option<ModelUsage>,
    provider_model: Option<String>,
    finish_reason: Option<String>,
    class: &'static str,
    message: String,
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
    let reports = run_counterbalanced_arms(
        generator.adapter(),
        &fixtures,
        args.max_tokens,
        args.seed,
        args.trials,
    )
    .await;
    let comparison = compare_arms(&reports)?;
    let stability = reports.iter().map(summarize_stability).collect();

    Ok(StudyOutput {
        configuration_id: "materialization-r2-v1",
        execution_design: "fixture_trial_rotating_arm_order_v1",
        semantic_baseline: "soft-semantic-v3",
        materialization_contract: "decision_plus_optional_advisory_note_harness_owned_binding_v1",
        provider: provider_name(args.provider),
        model: args.model,
        fixture_count: fixtures.len(),
        effective_enforcement_class: effective_enforcement_class(args.provider),
        arms: reports,
        comparison,
        stability,
    })
}

async fn run_counterbalanced_arms(
    adapter: &dyn ModelAdapter,
    fixtures: &[SoftJudgeCalibrationFixture],
    max_tokens: u32,
    seed: Option<u64>,
    trials: usize,
) -> Vec<ArmReport> {
    let capacity = fixtures.len() * trials;
    let mut cases_by_arm = StudyArm::ALL
        .into_iter()
        .map(|arm| (arm, Vec::with_capacity(capacity)))
        .collect::<BTreeMap<_, _>>();

    for trial in 0..trials {
        let trial_seed = seed.and_then(|base| base.checked_add(trial as u64));
        for (fixture_index, fixture) in fixtures.iter().enumerate() {
            let ordered = counterbalanced_arm_order(fixture_index, trial);
            for (execution_position, arm) in ordered.into_iter().enumerate() {
                let started = Instant::now();
                let result = run_arm(adapter, fixture, arm, max_tokens, trial_seed).await;
                let latency_ms = started.elapsed().as_millis();
                let case = match result {
                    Ok(observation) => StudyCase {
                        fixture_id: fixture.id.clone(),
                        trial,
                        seed: trial_seed,
                        label: fixture.label,
                        arm,
                        execution_position,
                        decision: Some(observation.decision),
                        advisory_note_present: observation.advisory_note_present,
                        latency_ms,
                        usage: Some(observation.usage),
                        provider_model: Some(observation.provider_model),
                        finish_reason: observation.finish_reason,
                        failure_class: None,
                        failure: None,
                    },
                    Err(failure) => StudyCase {
                        fixture_id: fixture.id.clone(),
                        trial,
                        seed: trial_seed,
                        label: fixture.label,
                        arm,
                        execution_position,
                        decision: None,
                        advisory_note_present: false,
                        latency_ms,
                        usage: failure.usage,
                        provider_model: failure.provider_model,
                        finish_reason: failure.finish_reason,
                        failure_class: Some(failure.class),
                        failure: Some(failure.message),
                    },
                };
                eprintln!(
                    "[materialization-study] fixture={} trial={} position={} arm={:?} status={}",
                    fixture.id,
                    trial + 1,
                    execution_position,
                    arm,
                    if case.decision.is_some() {
                        "ok"
                    } else {
                        "failed"
                    }
                );
                cases_by_arm
                    .get_mut(&arm)
                    .expect("every study arm has a case bucket")
                    .push(case);
            }
        }
    }

    StudyArm::ALL
        .into_iter()
        .map(|arm| {
            summarize_arm(
                arm,
                cases_by_arm
                    .remove(&arm)
                    .expect("every study arm has recorded cases"),
                fixtures.len(),
                trials,
            )
        })
        .collect()
}

async fn run_arm(
    adapter: &dyn ModelAdapter,
    fixture: &SoftJudgeCalibrationFixture,
    arm: StudyArm,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<RunObservation, FailureInfo> {
    match arm {
        StudyArm::V3FullJson => run_model_backed_soft_judge_representation(
            adapter,
            &fixture.request,
            SoftJudgeRepresentation::V3FullJson,
            max_tokens,
            seed,
        )
        .await
        .map(|observation| RunObservation {
            decision: observation.decision,
            advisory_note_present: false,
            usage: observation.usage,
            provider_model: observation.model,
            finish_reason: observation.finish_reason,
        })
        .map_err(format_failure_info),
        StudyArm::HarnessMaterializedDecision => {
            run_model_backed_soft_judge_materialization(adapter, &fixture.request, max_tokens, seed)
                .await
                .map(|observation| RunObservation {
                    decision: observation.decision,
                    advisory_note_present: observation.advisory_note.is_some(),
                    usage: observation.usage,
                    provider_model: observation.model,
                    finish_reason: observation.finish_reason,
                })
                .map_err(materialization_failure_info)
        }
    }
}

fn counterbalanced_arm_order(fixture_index: usize, trial: usize) -> [StudyArm; 2] {
    if (fixture_index + trial).is_multiple_of(2) {
        StudyArm::ALL
    } else {
        [StudyArm::HarnessMaterializedDecision, StudyArm::V3FullJson]
    }
}

fn summarize_arm(
    arm: StudyArm,
    cases: Vec<StudyCase>,
    fixtures_per_trial: usize,
    trials: usize,
) -> ArmReport {
    let attempted_runs = cases.len();
    let successful_runs = cases.iter().filter(|case| case.decision.is_some()).count();
    let advisory_note_runs = cases
        .iter()
        .filter(|case| case.advisory_note_present)
        .count();
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
                ambiguous_abstention: None,
            }
        });
    }

    ArmReport {
        arm,
        attempted_runs,
        successful_runs,
        failed_runs: attempted_runs - successful_runs,
        protocol_completion_rate: successful_runs as f64 / attempted_runs as f64,
        input_tokens,
        output_tokens,
        total_tokens,
        total_latency_ms,
        mean_latency_ms: total_latency_ms as f64 / attempted_runs as f64,
        advisory_note_runs,
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

fn summarize_stability(report: &ArmReport) -> ArmStabilityReport {
    let mut by_fixture = BTreeMap::<String, Vec<&StudyCase>>::new();
    for case in &report.cases {
        by_fixture
            .entry(case.fixture_id.clone())
            .or_default()
            .push(case);
    }

    let fixtures = by_fixture
        .into_iter()
        .map(|(fixture_id, mut cases)| {
            cases.sort_by_key(|case| (case.trial, case.seed));
            let label = cases[0].label;
            let probes = cases
                .iter()
                .map(|case| SoftDecisionProbe {
                    probe_id: format!("trial:{}:seed:{:?}", case.trial, case.seed),
                    decision: case.decision,
                })
                .collect::<Vec<_>>();
            let assessment = assess_soft_decision_stability(&probes);
            let disagreement_only = apply_selective_abstention(
                &assessment,
                SelectiveAbstentionPolicy::DisagreementOnly,
            );
            let complete_unanimity = apply_selective_abstention(
                &assessment,
                SelectiveAbstentionPolicy::CompleteUnanimity,
            );
            FixtureStabilityReport {
                fixture_id,
                label,
                assessment,
                disagreement_only,
                complete_unanimity,
            }
        })
        .collect::<Vec<_>>();

    let policies = [
        SelectiveAbstentionPolicy::DisagreementOnly,
        SelectiveAbstentionPolicy::CompleteUnanimity,
    ]
    .into_iter()
    .map(|policy| selective_policy_metrics(&fixtures, policy))
    .collect();

    ArmStabilityReport {
        arm: report.arm,
        fixtures,
        policies,
    }
}

fn selective_policy_metrics(
    fixtures: &[FixtureStabilityReport],
    policy: SelectiveAbstentionPolicy,
) -> SelectivePolicyMetrics {
    let mut tp = 0usize;
    let mut fp = 0usize;
    let mut fn_ = 0usize;
    let mut decided = 0usize;
    let mut ambiguous = 0usize;
    let mut ambiguous_abstain = 0usize;
    let mut risk_fixture_count = 0usize;
    let mut operationally_incomplete_fixture_count = 0usize;
    let mut escalated_to_abstain = 0usize;

    for fixture in fixtures {
        let outcome = match policy {
            SelectiveAbstentionPolicy::DisagreementOnly => &fixture.disagreement_only,
            SelectiveAbstentionPolicy::CompleteUnanimity => &fixture.complete_unanimity,
        };
        if !fixture.assessment.risk_signals.is_empty() {
            risk_fixture_count += 1;
        }
        if fixture
            .assessment
            .risk_signals
            .contains(&StabilityRiskSignal::OperationalIncomplete)
        {
            operationally_incomplete_fixture_count += 1;
        }
        escalated_to_abstain += usize::from(outcome.escalated_to_abstain);
        if outcome.decision != SoftJudgeDecision::Abstain {
            decided += 1;
        }
        match fixture.label {
            CalibrationLabel::Positive => {
                if outcome.decision == SoftJudgeDecision::Finding {
                    tp += 1;
                } else {
                    fn_ += 1;
                }
            }
            CalibrationLabel::Negative => {
                if outcome.decision == SoftJudgeDecision::Finding {
                    fp += 1;
                }
            }
            CalibrationLabel::Ambiguous => {
                ambiguous += 1;
                if outcome.decision == SoftJudgeDecision::Abstain {
                    ambiguous_abstain += 1;
                }
            }
        }
    }

    SelectivePolicyMetrics {
        policy,
        fixture_count: fixtures.len(),
        risk_fixture_count,
        operationally_incomplete_fixture_count,
        escalated_to_abstain,
        precision: ratio(tp, tp + fp),
        recall: ratio(tp, tp + fn_),
        decision_coverage: ratio(decided, fixtures.len()),
        ambiguous_abstention: ratio(ambiguous_abstain, ambiguous),
    }
}

fn compare_arms(reports: &[ArmReport]) -> Result<MaterializationComparison, String> {
    let baseline = reports
        .iter()
        .find(|report| report.arm == StudyArm::V3FullJson)
        .ok_or("missing v3 baseline arm")?;
    let materialized = reports
        .iter()
        .find(|report| report.arm == StudyArm::HarnessMaterializedDecision)
        .ok_or("missing materialization arm")?;

    let baseline_cases = index_cases(&baseline.cases)?;
    let materialized_cases = index_cases(&materialized.cases)?;
    if baseline_cases.keys().ne(materialized_cases.keys()) {
        return Err("materialization study arms do not have identical matched keys".into());
    }

    let mut matched_successful_pairs = 0usize;
    let mut changed_decisions = 0usize;
    let mut transitions = BTreeMap::new();
    for (key, baseline_case) in &baseline_cases {
        let materialized_case = materialized_cases
            .get(key)
            .expect("identical keys were checked above");
        let (Some(from), Some(to)) = (baseline_case.decision, materialized_case.decision) else {
            continue;
        };
        matched_successful_pairs += 1;
        changed_decisions += usize::from(from != to);
        *transitions.entry((from, to)).or_insert(0usize) += 1;
    }
    let matched_keys = baseline_cases.len();

    Ok(MaterializationComparison {
        baseline: StudyArm::V3FullJson,
        materialized: StudyArm::HarnessMaterializedDecision,
        matched_keys,
        matched_successful_pairs,
        operationally_incomplete_pairs: matched_keys - matched_successful_pairs,
        changed_decisions,
        decision_flip_rate: (matched_successful_pairs > 0)
            .then(|| changed_decisions as f64 / matched_successful_pairs as f64),
        transitions: transitions
            .into_iter()
            .map(|((from, to), count)| DecisionTransition { from, to, count })
            .collect(),
    })
}

type CaseKey = (String, usize, Option<u64>);

fn index_cases(cases: &[StudyCase]) -> Result<BTreeMap<CaseKey, &StudyCase>, String> {
    let mut indexed = BTreeMap::new();
    for case in cases {
        let key = (case.fixture_id.clone(), case.trial, case.seed);
        if indexed.insert(key.clone(), case).is_some() {
            return Err(format!("duplicate matched study key: {key:?}"));
        }
    }
    Ok(indexed)
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
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
            "R2 materialization study accepts only this checkout's fixtures/semantic-judges calibration corpus"
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

fn format_failure_info(error: FormatJudgeError) -> FailureInfo {
    let class = if matches!(error, FormatJudgeError::Setup(_)) {
        "study_setup"
    } else if error
        .finish_reason()
        .is_some_and(is_truncation_finish_reason)
    {
        "truncation_protocol"
    } else if error
        .finish_reason()
        .is_some_and(is_provider_generation_error_finish_reason)
    {
        "provider_generation_error"
    } else {
        model_failure_class(error.model_error_kind(), "representation_protocol")
    };
    FailureInfo {
        usage: error.usage().cloned(),
        provider_model: error.provider_model().map(str::to_string),
        finish_reason: error.finish_reason().map(str::to_string),
        class,
        message: error.to_string(),
    }
}

fn materialization_failure_info(error: MaterializationError) -> FailureInfo {
    let class = if matches!(error, MaterializationError::Setup(_)) {
        "study_setup"
    } else if error
        .finish_reason()
        .is_some_and(is_truncation_finish_reason)
    {
        "truncation_protocol"
    } else if error
        .finish_reason()
        .is_some_and(is_provider_generation_error_finish_reason)
    {
        "provider_generation_error"
    } else {
        model_failure_class(error.model_error_kind(), "materialization_protocol")
    };
    FailureInfo {
        usage: error.usage().cloned(),
        provider_model: error.provider_model().map(str::to_string),
        finish_reason: error.finish_reason().map(str::to_string),
        class,
        message: error.to_string(),
    }
}

fn model_failure_class(
    kind: Option<ModelErrorKind>,
    protocol_default: &'static str,
) -> &'static str {
    match kind {
        Some(ModelErrorKind::Credentials) => "credentials",
        Some(ModelErrorKind::Transport) => "transport",
        Some(ModelErrorKind::Provider) => "provider_error",
        Some(ModelErrorKind::RateLimit) => "rate_limit",
        Some(ModelErrorKind::Quota) => "quota",
        Some(ModelErrorKind::ProviderUnavailable) => "provider_unavailable",
        Some(ModelErrorKind::Timeout) => "timeout",
        Some(ModelErrorKind::Protocol) => "provider_protocol",
        Some(ModelErrorKind::UnsupportedCapability) => "unsupported_capability",
        None => protocol_default,
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

fn effective_enforcement_class(provider: Provider) -> &'static str {
    match provider {
        Provider::Mistral => "strict_json_schema",
        Provider::Google => "response_json_schema",
    }
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
    fn arm_order_counterbalances_across_fixture_and_trial() {
        assert_eq!(counterbalanced_arm_order(0, 0), StudyArm::ALL);
        assert_eq!(
            counterbalanced_arm_order(1, 0),
            [StudyArm::HarnessMaterializedDecision, StudyArm::V3FullJson]
        );
        assert_eq!(
            counterbalanced_arm_order(0, 1),
            [StudyArm::HarnessMaterializedDecision, StudyArm::V3FullJson]
        );
    }
}
