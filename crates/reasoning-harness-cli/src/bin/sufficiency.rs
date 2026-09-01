use std::{collections::BTreeMap, fs, path::PathBuf, time::Instant};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    EvidenceSufficiencyCalibrationFixture, EvidenceSufficiencyLabel, EvidenceSufficiencyModelError,
    EvidenceSufficiencyObservation, ModelAdapter, ModelErrorKind, ModelUsage,
    run_model_backed_evidence_sufficiency, validate_artifact,
    validate_evidence_sufficiency_fixture,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "reason-sufficiency-study")]
struct Args {
    #[arg(default_value = "fixtures/evidence-sufficiency-rsd0")]
    fixtures: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long)]
    model: String,
    #[arg(long, default_value_t = 1)]
    trials: usize,
    #[arg(long, default_value_t = 128)]
    max_tokens: u32,
    #[arg(long, default_value_t = 5000)]
    seed: u64,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Mistral,
    Google,
}

impl Provider {
    const fn name(self) -> &'static str {
        match self {
            Self::Mistral => "mistral",
            Self::Google => "google",
        }
    }
}

enum Generator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
}

impl Generator {
    fn from_env(provider: Provider, model: &str) -> Result<Self, String> {
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
    family: String,
    trial: usize,
    seed: u64,
    expected: EvidenceSufficiencyLabel,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed: Option<EvidenceSufficiencyLabel>,
    exact_match: bool,
    false_safe: bool,
    false_abstain: bool,
    latency_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    usage: Option<ModelUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_attempts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_reason: Option<reasoning_harness_core::EvidenceSufficiencyFallbackReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_class: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct Metrics {
    attempted_runs: usize,
    successful_runs: usize,
    failed_runs: usize,
    operational_completion_rate: f64,
    exact_3class_accuracy: Option<f64>,
    conservative_binary_accuracy: Option<f64>,
    false_safe_count: usize,
    false_safe_rate: Option<f64>,
    false_abstain_count: usize,
    false_abstain_rate: Option<f64>,
    confusion: BTreeMap<String, BTreeMap<String, usize>>,
    per_label_recall: BTreeMap<String, Option<f64>>,
    failure_classes: BTreeMap<&'static str, usize>,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    provider_attempts: u64,
    fallback_runs: usize,
    total_latency_ms: u128,
}

#[derive(Debug, Serialize)]
struct Output {
    study_id: &'static str,
    phase: &'static str,
    corpus: &'static str,
    provider: &'static str,
    requested_model: String,
    trials: usize,
    max_tokens: u32,
    base_seed: u64,
    fixture_count: usize,
    authority_semantics: &'static str,
    metrics: Metrics,
    cases: Vec<StudyCase>,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    if args.trials == 0 {
        return Err("--trials must be at least 1".into());
    }
    if args.max_tokens == 0 {
        return Err("--max-tokens must be at least 1".into());
    }
    let fixtures = load_fixtures(&args.fixtures)?;
    if fixtures.is_empty() {
        return Err("sufficiency corpus is empty".into());
    }
    let generator = Generator::from_env(args.provider, &args.model)?;
    let mut cases = Vec::with_capacity(fixtures.len() * args.trials);

    for trial in 0..args.trials {
        let seed = args
            .seed
            .checked_add(trial as u64)
            .ok_or("trial seed overflowed u64")?;
        for fixture in &fixtures {
            let started = Instant::now();
            let result = run_model_backed_evidence_sufficiency(
                generator.adapter(),
                &fixture.request,
                &fixture.artifact,
                args.max_tokens,
                Some(seed),
            )
            .await;
            cases.push(case_from_result(fixture, trial, seed, started, result));
        }
    }

    let output = Output {
        study_id: "evidence-sufficiency-rsd1-v1",
        phase: "RSD1 calibration only; no product authority",
        corpus: "evidence-sufficiency-rsd0",
        provider: args.provider.name(),
        requested_model: args.model,
        trials: args.trials,
        max_tokens: args.max_tokens,
        base_seed: args.seed,
        fixture_count: fixtures.len(),
        authority_semantics: "sufficient never creates authority; insufficient/mixed are diagnostic-only in RSD1",
        metrics: aggregate(&cases),
        cases,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn load_fixtures(path: &PathBuf) -> Result<Vec<EvidenceSufficiencyCalibrationFixture>, String> {
    let text = path.to_string_lossy();
    if text.contains("holdout-v4") || text.contains("holdout-v5") {
        return Err("RSD1 refuses frozen semantic holdout paths".into());
    }
    let mut paths = fs::read_dir(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    });
    paths.sort();
    let mut fixtures = Vec::with_capacity(paths.len());
    for path in paths {
        let fixture: EvidenceSufficiencyCalibrationFixture = serde_json::from_slice(
            &fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?,
        )
        .map_err(|error| format!("{}: {error}", path.display()))?;
        validate_evidence_sufficiency_fixture(&fixture)
            .map_err(|error| format!("{}: {error}", fixture.id))?;
        let validation = validate_artifact(&fixture.artifact);
        if !validation.is_ok() {
            return Err(format!(
                "{}: invalid artifact {:?}",
                fixture.id, validation.diagnostics
            ));
        }
        fixtures.push(fixture);
    }
    Ok(fixtures)
}

fn case_from_result(
    fixture: &EvidenceSufficiencyCalibrationFixture,
    trial: usize,
    seed: u64,
    started: Instant,
    result: Result<EvidenceSufficiencyObservation, EvidenceSufficiencyModelError>,
) -> StudyCase {
    let latency_ms = started.elapsed().as_millis();
    match result {
        Ok(observation) => {
            let observed = observation.decision;
            StudyCase {
                fixture_id: fixture.id.clone(),
                family: fixture.family.clone(),
                trial,
                seed,
                expected: fixture.label,
                observed: Some(observed),
                exact_match: observed == fixture.label,
                false_safe: is_non_sufficient(fixture.label)
                    && observed == EvidenceSufficiencyLabel::Sufficient,
                false_abstain: fixture.label == EvidenceSufficiencyLabel::Sufficient
                    && is_non_sufficient(observed),
                latency_ms,
                model: Some(observation.model),
                usage: Some(observation.usage),
                provider_attempts: Some(observation.provider_attempts),
                fallback_reason: Some(observation.fallback_reason),
                failure_class: None,
                failure_message: None,
            }
        }
        Err(error) => StudyCase {
            fixture_id: fixture.id.clone(),
            family: fixture.family.clone(),
            trial,
            seed,
            expected: fixture.label,
            observed: None,
            exact_match: false,
            false_safe: false,
            false_abstain: false,
            latency_ms,
            model: error.provider_model().map(ToString::to_string),
            usage: error.usage().cloned(),
            provider_attempts: None,
            fallback_reason: None,
            failure_class: Some(failure_class(&error)),
            failure_message: Some(error.to_string()),
        },
    }
}

fn aggregate(cases: &[StudyCase]) -> Metrics {
    let attempted_runs = cases.len();
    let successful = cases
        .iter()
        .filter(|case| case.observed.is_some())
        .collect::<Vec<_>>();
    let successful_runs = successful.len();
    let failed_runs = attempted_runs - successful_runs;
    let exact = successful.iter().filter(|case| case.exact_match).count();
    let false_safe_count = successful.iter().filter(|case| case.false_safe).count();
    let false_abstain_count = successful.iter().filter(|case| case.false_abstain).count();
    let expected_non_sufficient = successful
        .iter()
        .filter(|case| is_non_sufficient(case.expected))
        .count();
    let expected_sufficient = successful
        .iter()
        .filter(|case| case.expected == EvidenceSufficiencyLabel::Sufficient)
        .count();
    let binary_correct = successful
        .iter()
        .filter(|case| {
            let observed = case.observed.expect("successful case has observation");
            is_non_sufficient(case.expected) == is_non_sufficient(observed)
        })
        .count();

    let mut confusion = BTreeMap::<String, BTreeMap<String, usize>>::new();
    let mut expected_counts = BTreeMap::<String, usize>::new();
    let mut correct_counts = BTreeMap::<String, usize>::new();
    for case in &successful {
        let observed = case.observed.expect("successful case has observation");
        let expected_key = label_name(case.expected).to_string();
        let observed_key = label_name(observed).to_string();
        *confusion
            .entry(expected_key.clone())
            .or_default()
            .entry(observed_key)
            .or_default() += 1;
        *expected_counts.entry(expected_key.clone()).or_default() += 1;
        if observed == case.expected {
            *correct_counts.entry(expected_key).or_default() += 1;
        }
    }
    let mut per_label_recall = BTreeMap::new();
    for label in [
        EvidenceSufficiencyLabel::Sufficient,
        EvidenceSufficiencyLabel::Insufficient,
        EvidenceSufficiencyLabel::Mixed,
    ] {
        let key = label_name(label).to_string();
        let total = expected_counts.get(&key).copied().unwrap_or(0);
        let correct = correct_counts.get(&key).copied().unwrap_or(0);
        per_label_recall.insert(key, ratio(correct, total));
    }

    let mut failure_classes = BTreeMap::new();
    for case in cases.iter().filter(|case| case.observed.is_none()) {
        if let Some(class) = case.failure_class {
            *failure_classes.entry(class).or_default() += 1;
        }
    }
    let mut input_tokens = 0;
    let mut output_tokens = 0;
    let mut total_tokens = 0;
    let mut provider_attempts = 0;
    let mut fallback_runs = 0;
    for case in &successful {
        if let Some(usage) = &case.usage {
            input_tokens += usage.input_tokens.unwrap_or(0);
            output_tokens += usage.output_tokens.unwrap_or(0);
            total_tokens += usage.total_tokens.unwrap_or(0);
        }
        provider_attempts += u64::from(case.provider_attempts.unwrap_or(0));
        if case.fallback_reason.is_some_and(|reason| {
            reason != reasoning_harness_core::EvidenceSufficiencyFallbackReason::NotNeeded
        }) {
            fallback_runs += 1;
        }
    }

    Metrics {
        attempted_runs,
        successful_runs,
        failed_runs,
        operational_completion_rate: attempted_runs
            .checked_sub(0)
            .filter(|count| *count > 0)
            .map_or(0.0, |_| successful_runs as f64 / attempted_runs as f64),
        exact_3class_accuracy: ratio(exact, successful_runs),
        conservative_binary_accuracy: ratio(binary_correct, successful_runs),
        false_safe_count,
        false_safe_rate: ratio(false_safe_count, expected_non_sufficient),
        false_abstain_count,
        false_abstain_rate: ratio(false_abstain_count, expected_sufficient),
        confusion,
        per_label_recall,
        failure_classes,
        input_tokens,
        output_tokens,
        total_tokens,
        provider_attempts,
        fallback_runs,
        total_latency_ms: cases.iter().map(|case| case.latency_ms).sum(),
    }
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

const fn is_non_sufficient(label: EvidenceSufficiencyLabel) -> bool {
    matches!(
        label,
        EvidenceSufficiencyLabel::Insufficient | EvidenceSufficiencyLabel::Mixed
    )
}

const fn label_name(label: EvidenceSufficiencyLabel) -> &'static str {
    match label {
        EvidenceSufficiencyLabel::Sufficient => "sufficient",
        EvidenceSufficiencyLabel::Insufficient => "insufficient",
        EvidenceSufficiencyLabel::Mixed => "mixed",
    }
}

fn failure_class(error: &EvidenceSufficiencyModelError) -> &'static str {
    match error {
        EvidenceSufficiencyModelError::Setup(_) => "study_setup",
        EvidenceSufficiencyModelError::InvalidOutput { .. } => "sufficiency_protocol",
        EvidenceSufficiencyModelError::Model(error) => match error.kind {
            ModelErrorKind::Credentials => "credentials",
            ModelErrorKind::Transport => "transport",
            ModelErrorKind::Provider => "provider_error",
            ModelErrorKind::RateLimit => "rate_limit",
            ModelErrorKind::Quota => "quota",
            ModelErrorKind::ProviderUnavailable => "provider_unavailable",
            ModelErrorKind::Timeout => "timeout",
            ModelErrorKind::Protocol => "provider_protocol",
            ModelErrorKind::UnsupportedCapability => "unsupported_capability",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn case(expected: EvidenceSufficiencyLabel, observed: EvidenceSufficiencyLabel) -> StudyCase {
        StudyCase {
            fixture_id: "f".into(),
            family: "x".into(),
            trial: 0,
            seed: 1,
            expected,
            observed: Some(observed),
            exact_match: expected == observed,
            false_safe: is_non_sufficient(expected)
                && observed == EvidenceSufficiencyLabel::Sufficient,
            false_abstain: expected == EvidenceSufficiencyLabel::Sufficient
                && is_non_sufficient(observed),
            latency_ms: 1,
            model: Some("m".into()),
            usage: Some(ModelUsage::default()),
            provider_attempts: Some(1),
            fallback_reason: Some(
                reasoning_harness_core::EvidenceSufficiencyFallbackReason::NotNeeded,
            ),
            failure_class: None,
            failure_message: None,
        }
    }

    #[test]
    fn conservative_metrics_separate_false_safe_from_false_abstain() {
        let metrics = aggregate(&[
            case(
                EvidenceSufficiencyLabel::Sufficient,
                EvidenceSufficiencyLabel::Insufficient,
            ),
            case(
                EvidenceSufficiencyLabel::Insufficient,
                EvidenceSufficiencyLabel::Sufficient,
            ),
            case(
                EvidenceSufficiencyLabel::Mixed,
                EvidenceSufficiencyLabel::Mixed,
            ),
        ]);
        assert_eq!(metrics.false_safe_count, 1);
        assert_eq!(metrics.false_abstain_count, 1);
        assert_eq!(metrics.exact_3class_accuracy, Some(1.0 / 3.0));
        assert_eq!(metrics.conservative_binary_accuracy, Some(1.0 / 3.0));
    }

    #[test]
    fn loader_refuses_frozen_holdout_paths() {
        let error =
            load_fixtures(&PathBuf::from("fixtures/semantic-decidability-holdout-v5")).unwrap_err();
        assert!(error.contains("refuses frozen semantic holdout"));
    }
}
