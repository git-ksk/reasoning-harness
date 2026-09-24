use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, Instant},
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    ContextSufficiency, EvidenceAcquisitionDisposition, EvidenceNeedDecision, EvidenceNeedMode,
    EvidenceNeedProposal, EvidenceNeedTargetKind, EvidenceNeedTargetPolicy,
    ExistingEvidenceReuseStatus, ModelAdapter, ModelError, ModelErrorKind, ModelRequest,
    ModelUsage, SuppliedContextState, build_evidence_need_proposal_request,
    build_json_object_fallback_request, materialize_evidence_need, parse_evidence_need_proposal,
};
use reasoning_harness_providers::{GoogleAdapter, GroqAdapter, MistralAdapter, NvidiaAdapter};
use serde::{Deserialize, Serialize};

const CONFIGURATION_ID: &str = "evidence-need-routing-live-calibration-v1";
const EXPECTED_SUITE_ID: &str = "evidence-need-routing-calibration-v1";
const EXPECTED_STATUS: &str = "fresh_unobserved_calibration";
const EXPECTED_RELATIVE_DIR: &str = "fixtures/evidence-need-routing-calibration-v1";

#[derive(Debug, Parser)]
#[command(
    name = "reason-evidence-need-study",
    about = "Fresh model-backed calibration for target-local evidence-need routing"
)]
struct Args {
    /// Exact calibration directory. Other corpus paths are rejected.
    target: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    /// Exact provider model identifier.
    #[arg(long)]
    model: String,
    /// Optional fixture IDs for bounded preflight or diagnosis. Omit for canonical full calibration.
    #[arg(long = "fixture")]
    fixture_ids: Vec<String>,
    #[arg(long, default_value_t = 128)]
    max_tokens: u32,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, default_value_t = 1000)]
    inter_case_delay_ms: u64,
    /// Atomic progress checkpoint. In-progress checkpoints are explicitly non-scorable.
    #[arg(long)]
    checkpoint: Option<PathBuf>,
    /// Validate the corpus/materialization surface without making provider calls.
    #[arg(long, default_value_t = false)]
    validate_only: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Provider {
    Mistral,
    Google,
    Nvidia,
    Groq,
}

impl Provider {
    fn name(self) -> &'static str {
        match self {
            Self::Mistral => "mistral",
            Self::Google => "google",
            Self::Nvidia => "nvidia",
            Self::Groq => "groq",
        }
    }
}

enum Generator {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
    Nvidia(NvidiaAdapter),
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
            Provider::Nvidia => NvidiaAdapter::from_env(model)
                .map(Self::Nvidia)
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
            Self::Nvidia(adapter) => adapter,
            Self::Groq(adapter) => adapter,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CalibrationManifest {
    suite_id: String,
    issue: u64,
    status: String,
    source_rule: String,
    cases: Vec<CalibrationCase>,
}

#[derive(Debug, Deserialize)]
struct CalibrationCase {
    id: String,
    family: String,
    #[serde(default)]
    pair_id: Option<String>,
    task: String,
    target: String,
    target_kind: EvidenceNeedTargetKind,
    context: Vec<String>,
    context_state: SuppliedContextState,
    context_sufficiency: ContextSufficiency,
    baseline: EvidenceNeedMode,
    minimum: EvidenceNeedMode,
    #[serde(default)]
    downgrade: Option<EvidenceNeedMode>,
    flags: CalibrationFlags,
    reuse: ExistingEvidenceReuseStatus,
    #[serde(default = "default_true")]
    resolver_available: bool,
    expected_proposal: EvidenceNeedMode,
    expected_mode: EvidenceNeedMode,
    expected_acquisition: EvidenceAcquisitionDisposition,
    #[serde(default)]
    expected_operational_if_unavailable: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
struct CalibrationFlags {
    allow_no_factual_evidence: bool,
    allow_context_only: bool,
    allow_external_optional: bool,
    explicit_verification_intent: bool,
    current_state_required: bool,
    trusted_verification_required: bool,
}

impl CalibrationCase {
    fn policy(&self) -> EvidenceNeedTargetPolicy {
        EvidenceNeedTargetPolicy {
            policy_id: format!("{EXPECTED_SUITE_ID}:{}", self.id),
            target_id: self.id.clone(),
            target_question: self.target.clone(),
            target_kind: self.target_kind,
            baseline_mode: self.baseline,
            minimum_mode: self.minimum,
            model_downgrade_floor: self.downgrade,
            allow_no_factual_evidence: self.flags.allow_no_factual_evidence,
            allow_context_only: self.flags.allow_context_only,
            allow_external_optional: self.flags.allow_external_optional,
            explicit_verification_intent: self.flags.explicit_verification_intent,
            current_state_required: self.flags.current_state_required,
            trusted_verification_required: self.flags.trusted_verification_required,
            supplied_context: self.context_state,
            context_sufficiency: self.context_sufficiency,
            existing_evidence: self.reuse,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct UsageSummary {
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
}

impl UsageSummary {
    fn add(&mut self, usage: &ModelUsage) {
        self.input_tokens += usage.input_tokens.unwrap_or(0);
        self.output_tokens += usage.output_tokens.unwrap_or(0);
        self.total_tokens += usage.total_tokens.unwrap_or(0);
    }
}

#[derive(Debug, Clone, Serialize)]
struct CaseObservation {
    id: String,
    family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pair_id: Option<String>,
    expected_proposal: EvidenceNeedMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    observed_proposal: Option<EvidenceNeedMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proposal_match: Option<bool>,
    expected_mode: EvidenceNeedMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    materialized_mode: Option<EvidenceNeedMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode_match: Option<bool>,
    expected_acquisition: EvidenceAcquisitionDisposition,
    #[serde(skip_serializing_if = "Option::is_none")]
    acquisition: Option<EvidenceAcquisitionDisposition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    acquisition_match: Option<bool>,
    resolver_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    expected_operational_if_unavailable: Option<String>,
    used_json_fallback: bool,
    provider_attempts: u32,
    latency_ms: u128,
    usage: UsageSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    finish_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    decision: Option<EvidenceNeedDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct CalibrationMetrics {
    cases: usize,
    successful_provider_cases: usize,
    failed_provider_cases: usize,
    proposal_exact_matches: usize,
    proposal_exact_accuracy: Option<f64>,
    materialized_mode_matches: usize,
    materialized_mode_accuracy: Option<f64>,
    acquisition_matches: usize,
    acquisition_accuracy: Option<f64>,
    correctness_boundary_violations: usize,
    utility_misses: usize,
    provider_attempts: u64,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    total_latency_ms: u128,
    mean_latency_ms: Option<f64>,
    failure_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
struct StudyOutput {
    configuration_id: &'static str,
    suite_id: String,
    issue: u64,
    source_rule: String,
    corpus_status_at_observation: String,
    candidate_commit: String,
    provider: String,
    model: String,
    max_tokens: u32,
    seed: Option<u64>,
    canonical_full_calibration: bool,
    scorability: &'static str,
    metrics: CalibrationMetrics,
    observations: Vec<CaseObservation>,
}

#[derive(Debug, Serialize)]
struct Checkpoint<'a> {
    checkpoint_version: &'static str,
    run_status: &'static str,
    configuration_id: &'static str,
    suite_id: &'a str,
    provider: &'a str,
    model: &'a str,
    expected_cases: usize,
    completed_cases: usize,
    scorability: &'static str,
    observations: &'a [CaseObservation],
}

#[derive(Debug)]
struct CallOutcome {
    proposal: EvidenceNeedProposal,
    used_json_fallback: bool,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
}

#[derive(Debug)]
struct CallFailure {
    class: String,
    message: String,
    used_json_fallback: bool,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(output) => match serde_json::to_string_pretty(&output) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("serialize evidence-need study: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("evidence-need study failed: {error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<StudyOutput, String> {
    let args = Args::parse();
    let manifest = load_manifest(&args.target, &args.fixture_ids)?;
    let selected = select_cases(&manifest, &args.fixture_ids)?;

    // Validate every deterministic policy before any live observation.
    for case in &selected {
        let baseline = materialize_evidence_need(&case.policy(), None)
            .map_err(|error| format!("invalid calibration policy {}: {error}", case.id))?;
        if case.target_kind == EvidenceNeedTargetKind::NonFactual
            && baseline.mode != EvidenceNeedMode::NoFactualEvidence
        {
            return Err(format!(
                "non-factual calibration target {} did not remain non-factual",
                case.id
            ));
        }
    }

    if args.validate_only {
        return Ok(StudyOutput {
            configuration_id: CONFIGURATION_ID,
            suite_id: manifest.suite_id.clone(),
            issue: manifest.issue,
            source_rule: manifest.source_rule.clone(),
            corpus_status_at_observation: manifest.status.clone(),
            candidate_commit: git_head().unwrap_or_else(|_| "unknown".into()),
            provider: args.provider.name().into(),
            model: args.model,
            max_tokens: args.max_tokens,
            seed: args.seed,
            canonical_full_calibration: false,
            scorability: "validate_only_non_scorable",
            metrics: empty_metrics(selected.len()),
            observations: Vec::new(),
        });
    }

    let generator = Generator::from_provider(args.provider, &args.model)?;
    let provider = args.provider.name().to_owned();
    let mut observations = Vec::with_capacity(selected.len());

    if let Some(path) = args.checkpoint.as_deref() {
        write_checkpoint(
            path,
            &manifest.suite_id,
            &provider,
            &args.model,
            selected.len(),
            &observations,
            "in_progress",
        )?;
    }

    for (index, case) in selected.iter().enumerate() {
        let case_seed = args.seed.and_then(|base| base.checked_add(index as u64));
        let request = build_evidence_need_proposal_request(
            &case.task,
            &case.policy(),
            &case.context,
            Some(args.max_tokens),
            case_seed,
        )
        .map_err(|error| format!("build request {}: {error}", case.id))?;

        let started = Instant::now();
        let result = call_model_for_proposal(generator.adapter(), request).await;
        let latency_ms = started.elapsed().as_millis();

        let observation = match result {
            Ok(call) => {
                let proposal_mode = call.proposal.mode;
                match materialize_evidence_need(&case.policy(), Some(&call.proposal)) {
                    Ok(decision) => CaseObservation {
                        id: case.id.clone(),
                        family: case.family.clone(),
                        pair_id: case.pair_id.clone(),
                        expected_proposal: case.expected_proposal,
                        observed_proposal: Some(proposal_mode),
                        proposal_match: Some(proposal_mode == case.expected_proposal),
                        expected_mode: case.expected_mode,
                        materialized_mode: Some(decision.mode),
                        mode_match: Some(decision.mode == case.expected_mode),
                        expected_acquisition: case.expected_acquisition,
                        acquisition: Some(decision.acquisition),
                        acquisition_match: Some(decision.acquisition == case.expected_acquisition),
                        resolver_available: case.resolver_available,
                        expected_operational_if_unavailable: case
                            .expected_operational_if_unavailable
                            .clone(),
                        used_json_fallback: call.used_json_fallback,
                        provider_attempts: call.provider_attempts,
                        latency_ms,
                        usage: call.usage,
                        provider_model: call.provider_model,
                        finish_reason: call.finish_reason,
                        decision: Some(decision),
                        failure_class: None,
                        failure: None,
                    },
                    Err(error) => CaseObservation {
                        id: case.id.clone(),
                        family: case.family.clone(),
                        pair_id: case.pair_id.clone(),
                        expected_proposal: case.expected_proposal,
                        observed_proposal: Some(proposal_mode),
                        proposal_match: Some(proposal_mode == case.expected_proposal),
                        expected_mode: case.expected_mode,
                        materialized_mode: None,
                        mode_match: None,
                        expected_acquisition: case.expected_acquisition,
                        acquisition: None,
                        acquisition_match: None,
                        resolver_available: case.resolver_available,
                        expected_operational_if_unavailable: case
                            .expected_operational_if_unavailable
                            .clone(),
                        used_json_fallback: call.used_json_fallback,
                        provider_attempts: call.provider_attempts,
                        latency_ms,
                        usage: call.usage,
                        provider_model: call.provider_model,
                        finish_reason: call.finish_reason,
                        decision: None,
                        failure_class: Some("materialization".into()),
                        failure: Some(error.to_string()),
                    },
                }
            }
            Err(failure) => CaseObservation {
                id: case.id.clone(),
                family: case.family.clone(),
                pair_id: case.pair_id.clone(),
                expected_proposal: case.expected_proposal,
                observed_proposal: None,
                proposal_match: None,
                expected_mode: case.expected_mode,
                materialized_mode: None,
                mode_match: None,
                expected_acquisition: case.expected_acquisition,
                acquisition: None,
                acquisition_match: None,
                resolver_available: case.resolver_available,
                expected_operational_if_unavailable: case
                    .expected_operational_if_unavailable
                    .clone(),
                used_json_fallback: failure.used_json_fallback,
                provider_attempts: failure.provider_attempts,
                latency_ms,
                usage: failure.usage,
                provider_model: failure.provider_model,
                finish_reason: failure.finish_reason,
                decision: None,
                failure_class: Some(failure.class),
                failure: Some(failure.message),
            },
        };

        eprintln!(
            "[evidence-need-study] case={} status={} proposal={:?} mode={:?} acquisition={:?} fallback={}",
            case.id,
            if observation.failure.is_none() {
                "ok"
            } else {
                "failed"
            },
            observation.observed_proposal,
            observation.materialized_mode,
            observation.acquisition,
            observation.used_json_fallback,
        );
        observations.push(observation);

        if let Some(path) = args.checkpoint.as_deref() {
            write_checkpoint(
                path,
                &manifest.suite_id,
                &provider,
                &args.model,
                selected.len(),
                &observations,
                "in_progress",
            )?;
        }

        if index + 1 < selected.len() && args.inter_case_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(args.inter_case_delay_ms)).await;
        }
    }

    if let Some(path) = args.checkpoint.as_deref() {
        write_checkpoint(
            path,
            &manifest.suite_id,
            &provider,
            &args.model,
            selected.len(),
            &observations,
            "completed",
        )?;
    }

    let canonical_full_calibration =
        args.fixture_ids.is_empty() && selected.len() == manifest.cases.len();
    let operationally_complete = observations.iter().all(|case| case.failure.is_none());
    let scorability = if canonical_full_calibration && operationally_complete {
        "complete_calibration_observation"
    } else {
        "non_canonical_or_operationally_incomplete"
    };

    Ok(StudyOutput {
        configuration_id: CONFIGURATION_ID,
        suite_id: manifest.suite_id,
        issue: manifest.issue,
        source_rule: manifest.source_rule,
        corpus_status_at_observation: manifest.status,
        candidate_commit: git_head().unwrap_or_else(|_| "unknown".into()),
        provider,
        model: args.model,
        max_tokens: args.max_tokens,
        seed: args.seed,
        canonical_full_calibration,
        scorability,
        metrics: summarize_metrics(&observations),
        observations,
    })
}

async fn call_model_for_proposal(
    adapter: &dyn ModelAdapter,
    request: ModelRequest,
) -> Result<CallOutcome, CallFailure> {
    let mut attempts = 0u32;
    let mut usage = UsageSummary {
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
    };
    let mut last_model = None;
    let mut last_finish_reason = None;

    match adapter.generate(request.clone()).await {
        Ok(response) => {
            attempts = attempts.saturating_add(response.provider_attempts);
            usage.add(&response.usage);
            last_model = Some(response.model.clone());
            last_finish_reason = response.finish_reason.clone();
            match parse_evidence_need_proposal(&response.text) {
                Ok(proposal) => Ok(CallOutcome {
                    proposal,
                    used_json_fallback: false,
                    provider_attempts: attempts,
                    usage,
                    provider_model: last_model,
                    finish_reason: last_finish_reason,
                }),
                Err(primary_parse_error) => {
                    let Some(fallback) = build_json_object_fallback_request(&request) else {
                        return Err(CallFailure {
                            class: "protocol".into(),
                            message: format!(
                                "primary structured proposal parse failed and no fallback exists: {primary_parse_error}"
                            ),
                            used_json_fallback: false,
                            provider_attempts: attempts,
                            usage,
                            provider_model: last_model,
                            finish_reason: last_finish_reason,
                        });
                    };
                    return call_fallback(
                        adapter,
                        fallback,
                        attempts,
                        usage,
                        last_model,
                        last_finish_reason,
                        format!("primary structured proposal parse failed: {primary_parse_error}"),
                    )
                    .await;
                }
            }
        }
        Err(error) if error.kind == ModelErrorKind::UnsupportedCapability => {
            attempts = attempts.saturating_add(error.provider_attempts);
            let Some(fallback) = build_json_object_fallback_request(&request) else {
                return Err(model_failure(
                    error,
                    false,
                    attempts,
                    usage,
                    last_model,
                    last_finish_reason,
                ));
            };
            call_fallback(
                adapter,
                fallback,
                attempts,
                usage,
                last_model,
                last_finish_reason,
                "primary JSON-Schema capability unsupported".into(),
            )
            .await
        }
        Err(error) => {
            attempts = attempts.saturating_add(error.provider_attempts);
            Err(model_failure(
                error,
                false,
                attempts,
                usage,
                last_model,
                last_finish_reason,
            ))
        }
    }
}

async fn call_fallback(
    adapter: &dyn ModelAdapter,
    fallback: ModelRequest,
    prior_attempts: u32,
    mut usage: UsageSummary,
    prior_model: Option<String>,
    prior_finish_reason: Option<String>,
    primary_context: String,
) -> Result<CallOutcome, CallFailure> {
    match adapter.generate(fallback).await {
        Ok(response) => {
            let attempts = prior_attempts.saturating_add(response.provider_attempts);
            usage.add(&response.usage);
            let model = Some(response.model.clone());
            let finish_reason = response.finish_reason.clone();
            match parse_evidence_need_proposal(&response.text) {
                Ok(proposal) => Ok(CallOutcome {
                    proposal,
                    used_json_fallback: true,
                    provider_attempts: attempts,
                    usage,
                    provider_model: model,
                    finish_reason,
                }),
                Err(error) => Err(CallFailure {
                    class: "protocol".into(),
                    message: format!(
                        "{primary_context}; JSON-object fallback proposal parse failed: {error}"
                    ),
                    used_json_fallback: true,
                    provider_attempts: attempts,
                    usage,
                    provider_model: model,
                    finish_reason,
                }),
            }
        }
        Err(error) => {
            let attempts = prior_attempts.saturating_add(error.provider_attempts);
            let mut failure = model_failure(
                error,
                true,
                attempts,
                usage,
                prior_model,
                prior_finish_reason,
            );
            failure.message = format!("{primary_context}; fallback failed: {}", failure.message);
            Err(failure)
        }
    }
}

fn model_failure(
    error: ModelError,
    used_json_fallback: bool,
    provider_attempts: u32,
    usage: UsageSummary,
    provider_model: Option<String>,
    finish_reason: Option<String>,
) -> CallFailure {
    CallFailure {
        class: model_error_class(error.kind).into(),
        message: error.message,
        used_json_fallback,
        provider_attempts,
        usage,
        provider_model,
        finish_reason,
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

fn load_manifest(target: &Path, fixture_ids: &[String]) -> Result<CalibrationManifest, String> {
    let current = std::env::current_dir().map_err(|error| error.to_string())?;
    let expected = current
        .join(EXPECTED_RELATIVE_DIR)
        .canonicalize()
        .map_err(|error| format!("canonicalize expected calibration directory: {error}"))?;
    let target = target
        .canonicalize()
        .map_err(|error| format!("canonicalize target directory: {error}"))?;
    if target != expected {
        return Err(format!(
            "evidence-need study accepts only this checkout's {EXPECTED_RELATIVE_DIR}"
        ));
    }

    let raw = fs::read_to_string(target.join("manifest.json"))
        .map_err(|error| format!("read calibration manifest: {error}"))?;
    let manifest: CalibrationManifest = serde_json::from_str(&raw)
        .map_err(|error| format!("parse calibration manifest: {error}"))?;

    if manifest.suite_id != EXPECTED_SUITE_ID {
        return Err(format!(
            "unexpected calibration suite id {:?}",
            manifest.suite_id
        ));
    }
    if manifest.issue != 461 {
        return Err(format!("unexpected issue binding {}", manifest.issue));
    }
    if manifest.status != EXPECTED_STATUS {
        return Err(format!(
            "calibration status must be {EXPECTED_STATUS:?} before first live observation, got {:?}",
            manifest.status
        ));
    }

    let mut seen = BTreeMap::new();
    for case in &manifest.cases {
        if seen.insert(case.id.as_str(), ()).is_some() {
            return Err(format!("duplicate calibration case id {}", case.id));
        }
    }
    for requested in fixture_ids {
        if !seen.contains_key(requested.as_str()) {
            return Err(format!("unknown requested calibration case {requested}"));
        }
    }

    Ok(manifest)
}

fn select_cases<'a>(
    manifest: &'a CalibrationManifest,
    fixture_ids: &[String],
) -> Result<Vec<&'a CalibrationCase>, String> {
    if fixture_ids.is_empty() {
        return Ok(manifest.cases.iter().collect());
    }
    let wanted = fixture_ids
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let selected = manifest
        .cases
        .iter()
        .filter(|case| wanted.contains(case.id.as_str()))
        .collect::<Vec<_>>();
    if selected.len() != wanted.len() {
        return Err("requested calibration selection was incomplete".into());
    }
    Ok(selected)
}

fn summarize_metrics(observations: &[CaseObservation]) -> CalibrationMetrics {
    let successful_provider_cases = observations
        .iter()
        .filter(|case| case.failure.is_none())
        .count();
    let failed_provider_cases = observations.len() - successful_provider_cases;
    let proposal_exact_matches = observations
        .iter()
        .filter(|case| case.proposal_match == Some(true))
        .count();
    let materialized_mode_matches = observations
        .iter()
        .filter(|case| case.mode_match == Some(true))
        .count();
    let acquisition_matches = observations
        .iter()
        .filter(|case| case.acquisition_match == Some(true))
        .count();

    let correctness_boundary_violations = observations
        .iter()
        .filter(|case| is_correctness_boundary_violation(case))
        .count();
    let utility_misses = observations
        .iter()
        .filter(|case| is_utility_miss(case))
        .count();

    let provider_attempts = observations
        .iter()
        .map(|case| u64::from(case.provider_attempts))
        .sum();
    let input_tokens = observations
        .iter()
        .map(|case| case.usage.input_tokens)
        .sum();
    let output_tokens = observations
        .iter()
        .map(|case| case.usage.output_tokens)
        .sum();
    let total_tokens = observations
        .iter()
        .map(|case| case.usage.total_tokens)
        .sum();
    let total_latency_ms = observations.iter().map(|case| case.latency_ms).sum();
    let mean_latency_ms =
        (!observations.is_empty()).then(|| total_latency_ms as f64 / observations.len() as f64);

    let mut failure_counts = BTreeMap::new();
    for class in observations
        .iter()
        .filter_map(|case| case.failure_class.as_ref())
    {
        *failure_counts.entry(class.clone()).or_insert(0) += 1;
    }

    CalibrationMetrics {
        cases: observations.len(),
        successful_provider_cases,
        failed_provider_cases,
        proposal_exact_matches,
        proposal_exact_accuracy: ratio(proposal_exact_matches, successful_provider_cases),
        materialized_mode_matches,
        materialized_mode_accuracy: ratio(materialized_mode_matches, successful_provider_cases),
        acquisition_matches,
        acquisition_accuracy: ratio(acquisition_matches, successful_provider_cases),
        correctness_boundary_violations,
        utility_misses,
        provider_attempts,
        input_tokens,
        output_tokens,
        total_tokens,
        total_latency_ms,
        mean_latency_ms,
        failure_counts,
    }
}

fn empty_metrics(cases: usize) -> CalibrationMetrics {
    CalibrationMetrics {
        cases,
        successful_provider_cases: 0,
        failed_provider_cases: 0,
        proposal_exact_matches: 0,
        proposal_exact_accuracy: None,
        materialized_mode_matches: 0,
        materialized_mode_accuracy: None,
        acquisition_matches: 0,
        acquisition_accuracy: None,
        correctness_boundary_violations: 0,
        utility_misses: 0,
        provider_attempts: 0,
        input_tokens: 0,
        output_tokens: 0,
        total_tokens: 0,
        total_latency_ms: 0,
        mean_latency_ms: None,
        failure_counts: BTreeMap::new(),
    }
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn is_correctness_boundary_violation(case: &CaseObservation) -> bool {
    let Some(actual_mode) = case.materialized_mode else {
        return false;
    };
    if actual_mode < case.expected_mode {
        return true;
    }

    let Some(actual_acquisition) = case.acquisition else {
        return false;
    };
    match case.expected_acquisition {
        EvidenceAcquisitionDisposition::ExternalRequired => !matches!(
            actual_acquisition,
            EvidenceAcquisitionDisposition::ExternalRequired
                | EvidenceAcquisitionDisposition::TrustedVerificationRequired
        ),
        EvidenceAcquisitionDisposition::TrustedVerificationRequired => {
            actual_acquisition != EvidenceAcquisitionDisposition::TrustedVerificationRequired
        }
        _ => false,
    }
}

fn is_utility_miss(case: &CaseObservation) -> bool {
    let Some(actual_mode) = case.materialized_mode else {
        return false;
    };
    if actual_mode > case.expected_mode {
        return true;
    }

    let Some(actual_acquisition) = case.acquisition else {
        return false;
    };
    actual_acquisition != case.expected_acquisition && !is_correctness_boundary_violation(case)
}

fn write_checkpoint(
    path: &Path,
    suite_id: &str,
    provider: &str,
    model: &str,
    expected_cases: usize,
    observations: &[CaseObservation],
    run_status: &'static str,
) -> Result<(), String> {
    let checkpoint = Checkpoint {
        checkpoint_version: "evidence-need-calibration-checkpoint-v1",
        run_status,
        configuration_id: CONFIGURATION_ID,
        suite_id,
        provider,
        model,
        expected_cases,
        completed_cases: observations.len(),
        scorability: if run_status == "completed"
            && observations.len() == expected_cases
            && observations.iter().all(|case| case.failure.is_none())
        {
            "complete_calibration_observation"
        } else if run_status == "completed" {
            "operationally_incomplete_non_scorable"
        } else {
            "non_scorable_in_progress"
        },
        observations,
    };
    let bytes = serde_json::to_vec_pretty(&checkpoint)
        .map_err(|error| format!("serialize checkpoint: {error}"))?;
    let temporary = path.with_extension("tmp");
    fs::write(&temporary, bytes).map_err(|error| format!("write checkpoint: {error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("replace checkpoint: {error}"))
}

fn git_head() -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .map_err(|error| format!("read git head: {error}"))?;
    if !output.status.success() {
        return Err("git rev-parse HEAD failed".into());
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("decode git head: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(
        expected_mode: EvidenceNeedMode,
        actual_mode: EvidenceNeedMode,
        expected_acquisition: EvidenceAcquisitionDisposition,
        actual_acquisition: EvidenceAcquisitionDisposition,
    ) -> CaseObservation {
        CaseObservation {
            id: "case".into(),
            family: "test".into(),
            pair_id: None,
            expected_proposal: expected_mode,
            observed_proposal: Some(actual_mode),
            proposal_match: Some(expected_mode == actual_mode),
            expected_mode,
            materialized_mode: Some(actual_mode),
            mode_match: Some(expected_mode == actual_mode),
            expected_acquisition,
            acquisition: Some(actual_acquisition),
            acquisition_match: Some(expected_acquisition == actual_acquisition),
            resolver_available: true,
            expected_operational_if_unavailable: None,
            used_json_fallback: false,
            provider_attempts: 1,
            latency_ms: 1,
            usage: UsageSummary {
                input_tokens: 1,
                output_tokens: 1,
                total_tokens: 2,
            },
            provider_model: Some("fixture".into()),
            finish_reason: None,
            decision: None,
            failure_class: None,
            failure: None,
        }
    }

    #[test]
    fn weaker_required_mode_is_a_correctness_violation() {
        let case = observation(
            EvidenceNeedMode::ExternalRequired,
            EvidenceNeedMode::ContextOnly,
            EvidenceAcquisitionDisposition::ExternalRequired,
            EvidenceAcquisitionDisposition::ContextOnly,
        );
        assert!(is_correctness_boundary_violation(&case));
        assert!(!is_utility_miss(&case));
    }

    #[test]
    fn stronger_local_route_is_a_utility_miss_not_a_correctness_violation() {
        let case = observation(
            EvidenceNeedMode::ContextOnly,
            EvidenceNeedMode::ExternalRequired,
            EvidenceAcquisitionDisposition::ContextOnly,
            EvidenceAcquisitionDisposition::ExternalRequired,
        );
        assert!(!is_correctness_boundary_violation(&case));
        assert!(is_utility_miss(&case));
    }

    #[test]
    fn exact_route_is_neither_violation_nor_utility_miss() {
        let case = observation(
            EvidenceNeedMode::ContextOnly,
            EvidenceNeedMode::ContextOnly,
            EvidenceAcquisitionDisposition::ContextOnly,
            EvidenceAcquisitionDisposition::ContextOnly,
        );
        assert!(!is_correctness_boundary_violation(&case));
        assert!(!is_utility_miss(&case));
    }
}
