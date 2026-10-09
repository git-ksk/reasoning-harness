use std::{fs, path::PathBuf, process::ExitCode, time::Duration};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    EvidenceLocalQualificationV6, EvidenceRelevanceBindingProposal, EvidenceRelevanceCandidate,
    EvidenceRelevanceDisposition, EvidenceRelevanceTargetPolicy, ModelAdapter, ModelRequest,
    build_evidence_local_qualification_v8_request,
    build_evidence_relevance_binding_proposal_v5_request, build_strict_json_text_fallback_request,
    derive_effective_evidence_local_qualification_v17, materialize_evidence_relevance_v30,
    parse_evidence_local_qualification_v8, parse_evidence_relevance_binding_proposal,
};
use reasoning_harness_providers::{GoogleAdapter, GroqAdapter, MistralAdapter};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};

const SUITE_ID: &str = "engine-0.6.1-launch-independent-v2";
const MANIFEST_SHA256: &str = "dea6536caf02ba015b1beeb1b7a92989fb61143639bb10e6e8192cecb57de3a3";
const LIVE_IDS: [&str; 12] = [
    "p01", "p02", "p03", "p06", "p07", "p08", "n01", "n05", "n07", "n10", "n15", "n18",
];

#[derive(Parser)]
#[command(name = "reason-engine-061-launch-live")]
struct Args {
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long)]
    model: String,
    #[arg(long, default_value_t = false)]
    validate_only: bool,
    #[arg(long, default_value_t = 7_000)]
    inter_case_delay_ms: u64,
    #[arg(long, default_value_t = 13_047)]
    seed: u64,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
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

#[derive(Deserialize)]
struct Corpus {
    suite_id: String,
    issue: u64,
    status: String,
    protocol: String,
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    id: String,
    target: String,
    signals: Vec<serde_json::Value>,
    expected_relevant: bool,
}
#[derive(Serialize)]
struct Observation {
    id: String,
    expected_relevant: bool,
    observed_relevant: Option<bool>,
    binding: Option<EvidenceRelevanceBindingProposal>,
    raw: Option<EvidenceLocalQualificationV6>,
    failure: Option<String>,
}
#[derive(Serialize)]
struct Report {
    suite_id: &'static str,
    manifest_sha256: &'static str,
    provider: &'static str,
    model: String,
    validate_only: bool,
    expected_cases: usize,
    completed_cases: usize,
    false_positive_relevant: usize,
    positive_utility_miss: usize,
    operational_failures: usize,
    observations: Vec<Observation>,
}

fn load_corpus() -> Result<Corpus, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/engine-0.6.1-launch-independent-v2/manifest.json");
    let data = fs::read(path).map_err(|error| format!("read holdout corpus: {error}"))?;
    let hash = Sha256::digest(&data)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if hash != MANIFEST_SHA256 {
        return Err(format!("immutable corpus identity mismatch: {hash}"));
    }
    let corpus: Corpus = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    if corpus.suite_id != SUITE_ID
        || corpus.issue != 479
        || corpus.status != "fresh_unobserved_holdout"
        || corpus.protocol != "effective_v17_materialize_v30"
        || corpus.cases.len() != 30
        || !LIVE_IDS
            .iter()
            .all(|id| corpus.cases.iter().any(|case| case.id == *id))
    {
        return Err("corpus identity and precommitted subset mismatch".into());
    }
    Ok(corpus)
}

fn policy_and_candidate(
    case: &Case,
) -> Result<(EvidenceRelevanceTargetPolicy, EvidenceRelevanceCandidate), String> {
    let policy: EvidenceRelevanceTargetPolicy = serde_json::from_value(json!({
        "policy_id": format!("engine-0.6.1:{}",case.id),
        "target_id": case.id,
        "target_question": format!("When was {} launched?",case.target),
        "entity":{
            "canonical_id":format!("engine-0.6.1:{}",case.target),
            "canonical_name":case.target,
            "aliases":[]
        },
        "relation":"change_or_launch",
        "identity_requirement":"require_harness_anchor",
        "assessment_budget":{"max_model_attempts":2,"max_tokens":192,"max_elapsed_ms":90000}
    }))
    .map_err(|e| e.to_string())?;
    let candidate: EvidenceRelevanceCandidate = serde_json::from_value(json!({
        "evidence_id":format!("evidence:{}",case.id),
        "source_id":format!("source:{}",case.id),
        "signals":case.signals,
    }))
    .map_err(|e| e.to_string())?;
    Ok((policy, candidate))
}

async fn invoke_typed<T>(
    adapter: &dyn ModelAdapter,
    provider: Provider,
    request: ModelRequest,
    parse: fn(&str) -> Result<T, reasoning_harness_core::EvidenceRelevanceError>,
) -> Result<T, String> {
    let fallback = build_strict_json_text_fallback_request(&request)
        .ok_or("structured request cannot construct strict text fallback")?;
    let primary = if matches!(provider, Provider::Groq) {
        let mut text = fallback.clone();
        text.max_tokens = Some(text.max_tokens.unwrap_or_default().max(512));
        text
    } else {
        request
    };
    let first = tokio::time::timeout(Duration::from_secs(90), adapter.generate(primary))
        .await
        .map_err(|_| "provider timed out after 90 s".to_string())?;
    match first {
        Ok(response) => match parse(&response.text) {
            Ok(value) => Ok(value),
            Err(error) if !matches!(provider, Provider::Groq) => {
                let second =
                    tokio::time::timeout(Duration::from_secs(90), adapter.generate(fallback))
                        .await
                        .map_err(|_| "strict-text fallback timed out".to_string())?
                        .map_err(|e| {
                            format!("strict-text fallback provider error: {:?}: {}", e.kind, e)
                        })?;
                parse(&second.text)
                    .map_err(|e| format!("JSON schema parse {error}; strict-text parse {e}"))
            }
            Err(error) => Err(format!("typed parse: {error}")),
        },
        Err(error) if !matches!(provider, Provider::Groq) => {
            let second = tokio::time::timeout(Duration::from_secs(90), adapter.generate(fallback))
                .await
                .map_err(|_| "strict-text fallback timed out".to_string())?
                .map_err(|e| {
                    format!(
                        "provider primary {:?}: {}; fallback {:?}: {}",
                        error.kind, error, e.kind, e
                    )
                })?;
            parse(&second.text).map_err(|e| format!("strict-text typed parse: {e}"))
        }
        Err(error) => Err(format!("provider {:?}: {}", error.kind, error)),
    }
}

async fn run(args: Args) -> Result<Report, String> {
    let corpus = load_corpus()?;
    let mut report = Report {
        suite_id: SUITE_ID,
        manifest_sha256: MANIFEST_SHA256,
        provider: args.provider.name(),
        model: args.model.clone(),
        validate_only: args.validate_only,
        expected_cases: LIVE_IDS.len(),
        completed_cases: 0,
        false_positive_relevant: 0,
        positive_utility_miss: 0,
        operational_failures: 0,
        observations: Vec::new(),
    };
    if args.validate_only {
        return Ok(report);
    }
    let adapter: Box<dyn ModelAdapter> = match args.provider {
        Provider::Mistral => {
            Box::new(MistralAdapter::from_env(&args.model).map_err(|e| e.to_string())?)
        }
        Provider::Google => {
            Box::new(GoogleAdapter::from_env(&args.model).map_err(|e| e.to_string())?)
        }
        Provider::Groq => Box::new(GroqAdapter::from_env(&args.model).map_err(|e| e.to_string())?),
    };

    for (index, id) in LIVE_IDS.iter().enumerate() {
        let case = corpus
            .cases
            .iter()
            .find(|case| &case.id == id)
            .ok_or("missing case")?;
        let (policy, candidate) = policy_and_candidate(case)?;
        if index > 0 {
            tokio::time::sleep(Duration::from_millis(args.inter_case_delay_ms)).await;
        }
        let seed = args.seed + u64::try_from(index).unwrap_or_default();
        let result = async {
            let proposal_request = build_evidence_relevance_binding_proposal_v5_request(
                &policy,
                &candidate,
                Some(seed),
            )
            .map_err(|e| e.to_string())?;
            let binding: EvidenceRelevanceBindingProposal = invoke_typed(
                adapter.as_ref(),
                args.provider,
                proposal_request,
                parse_evidence_relevance_binding_proposal,
            )
            .await?;
            let local_request = build_evidence_local_qualification_v8_request(
                &policy,
                &candidate,
                Some(seed + 1000),
            )
            .map_err(|e| e.to_string())?;
            let raw: EvidenceLocalQualificationV6 = invoke_typed(
                adapter.as_ref(),
                args.provider,
                local_request,
                parse_evidence_local_qualification_v8,
            )
            .await?;
            let effective = derive_effective_evidence_local_qualification_v17(
                &policy,
                &candidate,
                Some(&binding),
                Some(&raw),
            )
            .map_err(|e| e.to_string())?;
            let outcome =
                materialize_evidence_relevance_v30(&policy, &candidate, Some(&binding), Some(&raw))
                    .map_err(|e| e.to_string())?;
            Ok::<_, String>((binding, raw, effective, outcome.disposition))
        }
        .await;
        match result {
            Ok((binding, raw, _effective, disposition)) => {
                let observed_relevant = disposition == EvidenceRelevanceDisposition::Relevant;
                report.false_positive_relevant +=
                    usize::from(observed_relevant && !case.expected_relevant);
                report.positive_utility_miss +=
                    usize::from(!observed_relevant && case.expected_relevant);
                report.observations.push(Observation {
                    id: case.id.clone(),
                    expected_relevant: case.expected_relevant,
                    observed_relevant: Some(observed_relevant),
                    binding: Some(binding),
                    raw: Some(raw),
                    failure: None,
                });
            }
            Err(error) => {
                report.operational_failures += 1;
                report.observations.push(Observation {
                    id: case.id.clone(),
                    expected_relevant: case.expected_relevant,
                    observed_relevant: None,
                    binding: None,
                    raw: None,
                    failure: Some(error),
                });
            }
        }
        report.completed_cases += 1;
        eprintln!(
            "case {} of {} complete, wrong-target={} utility-miss={} operational={}",
            report.completed_cases,
            report.expected_cases,
            report.false_positive_relevant,
            report.positive_utility_miss,
            report.operational_failures
        );
        if report.operational_failures >= 2 {
            eprintln!("stopping on two operational failures; remaining cases not scored");
            break;
        }
    }
    Ok(report)
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();
    match run(args).await {
        Err(error) => {
            eprintln!("engine 0.6.1 launch live acceptance: {error}");
            ExitCode::FAILURE
        }
        Ok(report) => {
            match serde_json::to_string_pretty(&report) {
                Ok(output) => println!("{output}"),
                Err(error) => {
                    eprintln!("serialize report: {error}");
                    return ExitCode::FAILURE;
                }
            }
            if report.validate_only
                || (report.completed_cases == report.expected_cases
                    && report.operational_failures == 0
                    && report.false_positive_relevant == 0
                    && report.positive_utility_miss == 0)
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
