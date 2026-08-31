use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    CalibrationLabel, MaterializationError, ModelAdapter, ModelErrorKind, ModelUsage,
    SemanticDecidabilityAssessment, SemanticDecidabilityDisposition,
    SemanticDecidabilityStudyFixture, SoftDecisionProbe, SoftDecisionStabilityAssessment,
    SoftJudgeCalibrationFixture, SoftJudgeDecision, assess_semantic_decidability,
    assess_soft_decision_stability, compose_semantic_decidability,
    run_model_backed_soft_judge_materialization,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "reason-decidability-study",
    about = "Calibration-only D2 semantic decidability/evidence-sufficiency study"
)]
struct Args {
    /// D2 study manifest directory. Historical holdouts and other paths are rejected.
    target: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long, default_value = "gemini-3.5-flash-lite")]
    model: String,
    /// Optional D2 fixture IDs for bounded validation. Without this, all fixtures run.
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

#[derive(Debug, Clone)]
struct ResolvedVariant {
    id: String,
    expected_disposition: SemanticDecidabilityDisposition,
    assessment: SemanticDecidabilityAssessment,
}

#[derive(Debug, Clone)]
struct ResolvedFixture {
    id: String,
    source_fixture_id: String,
    semantic_label: CalibrationLabel,
    source: SoftJudgeCalibrationFixture,
    variants: Vec<ResolvedVariant>,
}

#[derive(Debug, Clone, Serialize)]
struct VariantOutcome {
    variant_id: String,
    expected_disposition: SemanticDecidabilityDisposition,
    assessment: SemanticDecidabilityAssessment,
    #[serde(skip_serializing_if = "Option::is_none")]
    composed_decision: Option<SoftJudgeDecision>,
    escalated_to_abstain: bool,
}

#[derive(Debug, Clone, Serialize)]
struct StudyCase {
    fixture_id: String,
    source_fixture_id: String,
    semantic_label: CalibrationLabel,
    trial: usize,
    seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_decision: Option<SoftJudgeDecision>,
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
    variants: Vec<VariantOutcome>,
}

#[derive(Debug, Clone, Serialize)]
struct DecisionMetrics {
    eligible_clear_cases: usize,
    eligible_clear_decisions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    eligible_clear_decision_coverage: Option<f64>,
    eligible_true_positives: usize,
    eligible_false_positives: usize,
    eligible_false_negatives: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    eligible_precision: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eligible_recall: Option<f64>,
    eligible_ambiguous_cases: usize,
    eligible_ambiguous_abstentions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    eligible_ambiguous_abstention_rate: Option<f64>,
    typed_insufficiency_variants: usize,
    base_unsafe_assertions: usize,
    composed_unsafe_assertions: usize,
    typed_insufficiency_abstentions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    typed_insufficiency_abstention_rate: Option<f64>,
    gate_escalations: usize,
    overall_variants: usize,
    overall_decisions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    overall_decision_coverage: Option<f64>,
}

#[derive(Debug, Serialize)]
struct TrialReport {
    trial: usize,
    expected_provider_calls: usize,
    successful_provider_calls: usize,
    operationally_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    metrics: Option<DecisionMetrics>,
}

#[derive(Debug, Serialize)]
struct AggregateReport {
    complete_trials: usize,
    incomplete_trials: usize,
    complete_trial_cases: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    metrics: Option<DecisionMetrics>,
}

#[derive(Debug, Serialize)]
struct FixtureStabilityReport {
    fixture_id: String,
    source_fixture_id: String,
    semantic_label: CalibrationLabel,
    base_decision_stability: SoftDecisionStabilityAssessment,
}

#[derive(Debug, Serialize)]
struct StudyOutput {
    configuration_id: &'static str,
    semantic_baseline: &'static str,
    materialization_contract: &'static str,
    decidability_contract: &'static str,
    execution_design: &'static str,
    provider: &'static str,
    model: String,
    fixture_count: usize,
    variant_count: usize,
    attempted_provider_calls: usize,
    successful_provider_calls: usize,
    failed_provider_calls: usize,
    protocol_completion_rate: f64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    total_latency_ms: u128,
    mean_latency_ms: f64,
    trials: Vec<TrialReport>,
    aggregate: AggregateReport,
    stability: Vec<FixtureStabilityReport>,
    cases: Vec<StudyCase>,
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

    // Validate the exact calibration surface and all deterministic labels before reading provider
    // credentials. Historical holdouts can never be substituted through a symlink or CLI path.
    let target = require_d2_corpus(&args.target)?;
    let source_root = require_source_calibration_corpus()?;
    let sources = load_source_fixtures(&source_root)?;
    let mut fixtures = resolve_study_fixtures(load_study_manifests(&target)?, &sources)?;

    if !args.fixture_ids.is_empty() {
        fixtures.retain(|fixture| args.fixture_ids.iter().any(|id| id == &fixture.id));
        for requested in &args.fixture_ids {
            if !fixtures.iter().any(|fixture| &fixture.id == requested) {
                return Err(format!("requested D2 fixture not found: {requested}"));
            }
        }
    }
    if fixtures.is_empty() {
        return Err("no D2 fixtures selected".into());
    }

    let generator = Generator::from_provider(args.provider, &args.model)?;
    let cases = run_trials(
        generator.adapter(),
        &fixtures,
        args.max_tokens,
        args.seed,
        args.trials,
    )
    .await;

    let attempted_provider_calls = cases.len();
    let successful_provider_calls = cases
        .iter()
        .filter(|case| case.base_decision.is_some())
        .count();
    let total_latency_ms = cases.iter().map(|case| case.latency_ms).sum::<u128>();
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;
    let mut total_tokens = 0u64;
    for usage in cases.iter().filter_map(|case| case.usage.as_ref()) {
        input_tokens += usage.input_tokens.unwrap_or(0);
        output_tokens += usage.output_tokens.unwrap_or(0);
        total_tokens += usage.total_tokens.unwrap_or(0);
    }

    let trials = summarize_trials(&cases, fixtures.len(), args.trials);
    let aggregate = summarize_aggregate(&cases, &trials);
    let stability = summarize_stability(&cases);

    Ok(StudyOutput {
        configuration_id: "semantic-decidability-d2-v1",
        semantic_baseline: "soft-semantic-v3",
        materialization_contract: "materialization-r2-v1",
        decidability_contract: "deterministic-explicit-typed-preconditions-v1",
        execution_design: "one_r2_observation_per_semantic_case_then_matched_typed_variants_v1",
        provider: provider_name(args.provider),
        model: args.model,
        fixture_count: fixtures.len(),
        variant_count: fixtures.iter().map(|fixture| fixture.variants.len()).sum(),
        attempted_provider_calls,
        successful_provider_calls,
        failed_provider_calls: attempted_provider_calls - successful_provider_calls,
        protocol_completion_rate: successful_provider_calls as f64
            / attempted_provider_calls as f64,
        input_tokens,
        output_tokens,
        total_tokens,
        total_latency_ms,
        mean_latency_ms: total_latency_ms as f64 / attempted_provider_calls as f64,
        trials,
        aggregate,
        stability,
        cases,
    })
}

async fn run_trials(
    adapter: &dyn ModelAdapter,
    fixtures: &[ResolvedFixture],
    max_tokens: u32,
    seed: Option<u64>,
    trials: usize,
) -> Vec<StudyCase> {
    let mut cases = Vec::with_capacity(fixtures.len() * trials);
    for trial in 0..trials {
        let trial_seed = seed.and_then(|base| base.checked_add(trial as u64));
        for fixture in fixtures {
            let started = Instant::now();
            let result = run_model_backed_soft_judge_materialization(
                adapter,
                &fixture.source.request,
                max_tokens,
                trial_seed,
            )
            .await;
            let latency_ms = started.elapsed().as_millis();

            let case = match result {
                Ok(observation) => {
                    let base_decision = observation.decision;
                    StudyCase {
                        fixture_id: fixture.id.clone(),
                        source_fixture_id: fixture.source_fixture_id.clone(),
                        semantic_label: fixture.semantic_label,
                        trial,
                        seed: trial_seed,
                        base_decision: Some(base_decision),
                        advisory_note_present: observation.advisory_note.is_some(),
                        latency_ms,
                        usage: Some(observation.usage),
                        provider_model: Some(observation.model),
                        finish_reason: observation.finish_reason,
                        failure_class: None,
                        failure: None,
                        variants: variant_outcomes(fixture, Some(base_decision)),
                    }
                }
                Err(error) => {
                    let failure = materialization_failure_info(error);
                    StudyCase {
                        fixture_id: fixture.id.clone(),
                        source_fixture_id: fixture.source_fixture_id.clone(),
                        semantic_label: fixture.semantic_label,
                        trial,
                        seed: trial_seed,
                        base_decision: None,
                        advisory_note_present: false,
                        latency_ms,
                        usage: failure.usage,
                        provider_model: failure.provider_model,
                        finish_reason: failure.finish_reason,
                        failure_class: Some(failure.class),
                        failure: Some(failure.message),
                        variants: variant_outcomes(fixture, None),
                    }
                }
            };
            eprintln!(
                "[decidability-study] fixture={} trial={} status={}",
                fixture.id,
                trial + 1,
                if case.base_decision.is_some() {
                    "ok"
                } else {
                    "failed"
                }
            );
            cases.push(case);
        }
    }
    cases
}

fn variant_outcomes(
    fixture: &ResolvedFixture,
    base_decision: Option<SoftJudgeDecision>,
) -> Vec<VariantOutcome> {
    fixture
        .variants
        .iter()
        .map(|variant| {
            let composed_decision = base_decision
                .map(|decision| compose_semantic_decidability(decision, &variant.assessment));
            VariantOutcome {
                variant_id: variant.id.clone(),
                expected_disposition: variant.expected_disposition,
                assessment: variant.assessment.clone(),
                composed_decision,
                escalated_to_abstain: matches!(
                    (base_decision, composed_decision),
                    (Some(base), Some(SoftJudgeDecision::Abstain)) if base != SoftJudgeDecision::Abstain
                ),
            }
        })
        .collect()
}

fn summarize_trials(
    cases: &[StudyCase],
    fixtures_per_trial: usize,
    trials: usize,
) -> Vec<TrialReport> {
    (0..trials)
        .map(|trial| {
            let trial_cases = cases
                .iter()
                .filter(|case| case.trial == trial)
                .collect::<Vec<_>>();
            let successful_provider_calls = trial_cases
                .iter()
                .filter(|case| case.base_decision.is_some())
                .count();
            let operationally_complete = successful_provider_calls == fixtures_per_trial;
            TrialReport {
                trial,
                expected_provider_calls: fixtures_per_trial,
                successful_provider_calls,
                operationally_complete,
                metrics: operationally_complete.then(|| decision_metrics(&trial_cases)),
            }
        })
        .collect()
}

fn summarize_aggregate(cases: &[StudyCase], trials: &[TrialReport]) -> AggregateReport {
    let complete_trial_ids = trials
        .iter()
        .filter(|trial| trial.operationally_complete)
        .map(|trial| trial.trial)
        .collect::<BTreeSet<_>>();
    let complete_cases = cases
        .iter()
        .filter(|case| complete_trial_ids.contains(&case.trial))
        .collect::<Vec<_>>();
    AggregateReport {
        complete_trials: complete_trial_ids.len(),
        incomplete_trials: trials.len() - complete_trial_ids.len(),
        complete_trial_cases: complete_cases.len(),
        metrics: (!complete_cases.is_empty()).then(|| decision_metrics(&complete_cases)),
    }
}

fn decision_metrics(cases: &[&StudyCase]) -> DecisionMetrics {
    let mut eligible_clear_cases = 0usize;
    let mut eligible_clear_decisions = 0usize;
    let mut eligible_true_positives = 0usize;
    let mut eligible_false_positives = 0usize;
    let mut eligible_false_negatives = 0usize;
    let mut eligible_ambiguous_cases = 0usize;
    let mut eligible_ambiguous_abstentions = 0usize;
    let mut typed_insufficiency_variants = 0usize;
    let mut base_unsafe_assertions = 0usize;
    let mut composed_unsafe_assertions = 0usize;
    let mut typed_insufficiency_abstentions = 0usize;
    let mut gate_escalations = 0usize;
    let mut overall_variants = 0usize;
    let mut overall_decisions = 0usize;

    for case in cases {
        let base = case
            .base_decision
            .expect("decision metrics require operationally complete cases");
        let permit = case
            .variants
            .iter()
            .find(|variant| variant.expected_disposition == SemanticDecidabilityDisposition::Permit)
            .expect("D2 preflight requires one permit control");
        let permit_decision = permit
            .composed_decision
            .expect("complete case has composed permit decision");

        match case.semantic_label {
            CalibrationLabel::Positive => {
                eligible_clear_cases += 1;
                eligible_clear_decisions +=
                    usize::from(permit_decision != SoftJudgeDecision::Abstain);
                if permit_decision == SoftJudgeDecision::Finding {
                    eligible_true_positives += 1;
                } else {
                    eligible_false_negatives += 1;
                }
            }
            CalibrationLabel::Negative => {
                eligible_clear_cases += 1;
                eligible_clear_decisions +=
                    usize::from(permit_decision != SoftJudgeDecision::Abstain);
                if permit_decision == SoftJudgeDecision::Finding {
                    eligible_false_positives += 1;
                }
            }
            CalibrationLabel::Ambiguous => {
                eligible_ambiguous_cases += 1;
                eligible_ambiguous_abstentions +=
                    usize::from(permit_decision == SoftJudgeDecision::Abstain);
            }
        }

        for variant in &case.variants {
            overall_variants += 1;
            let composed = variant
                .composed_decision
                .expect("complete case has every composed decision");
            overall_decisions += usize::from(composed != SoftJudgeDecision::Abstain);
            if variant.expected_disposition == SemanticDecidabilityDisposition::ForceAbstain {
                typed_insufficiency_variants += 1;
                base_unsafe_assertions += usize::from(base != SoftJudgeDecision::Abstain);
                composed_unsafe_assertions += usize::from(composed != SoftJudgeDecision::Abstain);
                typed_insufficiency_abstentions +=
                    usize::from(composed == SoftJudgeDecision::Abstain);
                gate_escalations += usize::from(variant.escalated_to_abstain);
            }
        }
    }

    DecisionMetrics {
        eligible_clear_cases,
        eligible_clear_decisions,
        eligible_clear_decision_coverage: ratio(eligible_clear_decisions, eligible_clear_cases),
        eligible_true_positives,
        eligible_false_positives,
        eligible_false_negatives,
        eligible_precision: ratio(
            eligible_true_positives,
            eligible_true_positives + eligible_false_positives,
        ),
        eligible_recall: ratio(
            eligible_true_positives,
            eligible_true_positives + eligible_false_negatives,
        ),
        eligible_ambiguous_cases,
        eligible_ambiguous_abstentions,
        eligible_ambiguous_abstention_rate: ratio(
            eligible_ambiguous_abstentions,
            eligible_ambiguous_cases,
        ),
        typed_insufficiency_variants,
        base_unsafe_assertions,
        composed_unsafe_assertions,
        typed_insufficiency_abstentions,
        typed_insufficiency_abstention_rate: ratio(
            typed_insufficiency_abstentions,
            typed_insufficiency_variants,
        ),
        gate_escalations,
        overall_variants,
        overall_decisions,
        overall_decision_coverage: ratio(overall_decisions, overall_variants),
    }
}

fn summarize_stability(cases: &[StudyCase]) -> Vec<FixtureStabilityReport> {
    let mut grouped = BTreeMap::<String, Vec<&StudyCase>>::new();
    for case in cases {
        grouped
            .entry(case.fixture_id.clone())
            .or_default()
            .push(case);
    }
    grouped
        .into_iter()
        .map(|(fixture_id, mut cases)| {
            cases.sort_by_key(|case| (case.trial, case.seed));
            let first = cases[0];
            let probes = cases
                .iter()
                .map(|case| SoftDecisionProbe {
                    probe_id: format!("trial:{}:seed:{:?}", case.trial, case.seed),
                    decision: case.base_decision,
                })
                .collect::<Vec<_>>();
            FixtureStabilityReport {
                fixture_id,
                source_fixture_id: first.source_fixture_id.clone(),
                semantic_label: first.semantic_label,
                base_decision_stability: assess_soft_decision_stability(&probes),
            }
        })
        .collect()
}

fn resolve_study_fixtures(
    manifests: Vec<SemanticDecidabilityStudyFixture>,
    sources: &BTreeMap<String, SoftJudgeCalibrationFixture>,
) -> Result<Vec<ResolvedFixture>, String> {
    let mut fixture_ids = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    let mut resolved = Vec::with_capacity(manifests.len());

    for manifest in manifests {
        if manifest.id.trim().is_empty() || !fixture_ids.insert(manifest.id.clone()) {
            return Err(format!(
                "invalid or duplicate D2 fixture id: {}",
                manifest.id
            ));
        }
        if !source_ids.insert(manifest.source_fixture_id.clone()) {
            return Err(format!(
                "D2 source fixture is reused by multiple study cases: {}",
                manifest.source_fixture_id
            ));
        }
        let source = sources.get(&manifest.source_fixture_id).ok_or_else(|| {
            format!(
                "D2 source calibration fixture not found: {}",
                manifest.source_fixture_id
            )
        })?;
        if source.label != manifest.semantic_label {
            return Err(format!(
                "D2 semantic label does not match source calibration fixture: {}",
                manifest.id
            ));
        }
        if manifest.variants.is_empty() || manifest.variants.len() > 2 {
            return Err(format!(
                "D2 fixture must contain one permit control and at most one force variant: {}",
                manifest.id
            ));
        }

        let mut variant_ids = BTreeSet::new();
        let mut permit_count = 0usize;
        let mut force_count = 0usize;
        let mut variants = Vec::with_capacity(manifest.variants.len());
        for variant in manifest.variants {
            if variant.id.trim().is_empty() || !variant_ids.insert(variant.id.clone()) {
                return Err(format!(
                    "invalid or duplicate D2 variant id in {}: {}",
                    manifest.id, variant.id
                ));
            }
            match variant.expected_disposition {
                SemanticDecidabilityDisposition::Permit => permit_count += 1,
                SemanticDecidabilityDisposition::ForceAbstain => force_count += 1,
            }
            let assessment = assess_semantic_decidability(&source.request, &variant.artifact)
                .map_err(|error| format!("{}:{}: {error}", manifest.id, variant.id))?;
            if assessment.disposition != variant.expected_disposition {
                return Err(format!(
                    "D2 gate expectation mismatch in {}:{}: expected {:?}, got {:?}",
                    manifest.id, variant.id, variant.expected_disposition, assessment.disposition
                ));
            }
            variants.push(ResolvedVariant {
                id: variant.id,
                expected_disposition: variant.expected_disposition,
                assessment,
            });
        }
        if permit_count != 1 || force_count > 1 {
            return Err(format!(
                "D2 fixture must have exactly one permit control and at most one force variant: {}",
                manifest.id
            ));
        }
        if force_count == 1 && manifest.semantic_label == CalibrationLabel::Ambiguous {
            return Err(format!(
                "D2 v1 keeps typed insufficiency separate from semantic ambiguity: {}",
                manifest.id
            ));
        }
        resolved.push(ResolvedFixture {
            id: manifest.id,
            source_fixture_id: manifest.source_fixture_id,
            semantic_label: manifest.semantic_label,
            source: source.clone(),
            variants,
        });
    }
    Ok(resolved)
}

fn require_d2_corpus(target: &Path) -> Result<PathBuf, String> {
    let target = target
        .canonicalize()
        .map_err(|error| format!("{}: {error}", target.display()))?;
    let expected = std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join("fixtures/semantic-decidability-d2")
        .canonicalize()
        .map_err(|error| format!("D2 calibration corpus unavailable: {error}"))?;
    if target != expected {
        return Err(
            "D2 decidability study accepts only this checkout's fixtures/semantic-decidability-d2 calibration corpus"
                .into(),
        );
    }
    Ok(target)
}

fn require_source_calibration_corpus() -> Result<PathBuf, String> {
    std::env::current_dir()
        .map_err(|error| error.to_string())?
        .join("fixtures/semantic-judges")
        .canonicalize()
        .map_err(|error| format!("semantic calibration source corpus unavailable: {error}"))
}

fn load_study_manifests(directory: &Path) -> Result<Vec<SemanticDecidabilityStudyFixture>, String> {
    load_json_directory(directory)
}

fn load_source_fixtures(
    directory: &Path,
) -> Result<BTreeMap<String, SoftJudgeCalibrationFixture>, String> {
    let fixtures = load_json_directory::<SoftJudgeCalibrationFixture>(directory)?;
    let mut indexed = BTreeMap::new();
    for fixture in fixtures {
        if indexed.insert(fixture.id.clone(), fixture).is_some() {
            return Err("duplicate semantic source calibration fixture id".into());
        }
    }
    Ok(indexed)
}

fn load_json_directory<T: serde::de::DeserializeOwned>(directory: &Path) -> Result<Vec<T>, String> {
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
    paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
            serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_case(
        fixture_id: &str,
        label: CalibrationLabel,
        base: SoftJudgeDecision,
        force: bool,
    ) -> StudyCase {
        let permit_assessment = SemanticDecidabilityAssessment {
            disposition: SemanticDecidabilityDisposition::Permit,
            reasons: vec![],
        };
        let mut variants = vec![VariantOutcome {
            variant_id: "permit".into(),
            expected_disposition: SemanticDecidabilityDisposition::Permit,
            assessment: permit_assessment,
            composed_decision: Some(base),
            escalated_to_abstain: false,
        }];
        if force {
            variants.push(VariantOutcome {
                variant_id: "force".into(),
                expected_disposition: SemanticDecidabilityDisposition::ForceAbstain,
                assessment: SemanticDecidabilityAssessment {
                    disposition: SemanticDecidabilityDisposition::ForceAbstain,
                    reasons: vec![
                        reasoning_harness_core::SemanticDecidabilityReason::MissingPropositionBinding,
                    ],
                },
                composed_decision: Some(SoftJudgeDecision::Abstain),
                escalated_to_abstain: base != SoftJudgeDecision::Abstain,
            });
        }
        StudyCase {
            fixture_id: fixture_id.into(),
            source_fixture_id: fixture_id.into(),
            semantic_label: label,
            trial: 0,
            seed: Some(1),
            base_decision: Some(base),
            advisory_note_present: false,
            latency_ms: 1,
            usage: None,
            provider_model: None,
            finish_reason: None,
            failure_class: None,
            failure: None,
            variants,
        }
    }

    #[test]
    fn metrics_keep_force_abstention_out_of_semantic_recall_denominator() {
        let positive = synthetic_case(
            "positive",
            CalibrationLabel::Positive,
            SoftJudgeDecision::Finding,
            true,
        );
        let negative = synthetic_case(
            "negative",
            CalibrationLabel::Negative,
            SoftJudgeDecision::NoFinding,
            true,
        );
        let metrics = decision_metrics(&[&positive, &negative]);
        assert_eq!(metrics.eligible_true_positives, 1);
        assert_eq!(metrics.eligible_false_negatives, 0);
        assert_eq!(metrics.eligible_recall, Some(1.0));
        assert_eq!(metrics.typed_insufficiency_variants, 2);
        assert_eq!(metrics.base_unsafe_assertions, 2);
        assert_eq!(metrics.composed_unsafe_assertions, 0);
        assert_eq!(metrics.typed_insufficiency_abstention_rate, Some(1.0));
    }

    #[test]
    fn ambiguous_permit_controls_have_their_own_denominator() {
        let ambiguous = synthetic_case(
            "ambiguous",
            CalibrationLabel::Ambiguous,
            SoftJudgeDecision::Abstain,
            false,
        );
        let metrics = decision_metrics(&[&ambiguous]);
        assert_eq!(metrics.eligible_clear_cases, 0);
        assert_eq!(metrics.eligible_ambiguous_cases, 1);
        assert_eq!(metrics.eligible_ambiguous_abstention_rate, Some(1.0));
        assert_eq!(metrics.typed_insufficiency_variants, 0);
    }
}
