use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use reasoning_harness_core::{
    CandidateClaim, CanonicalFinalAnswerRenderer, EpistemicState, EvidenceAuthorityPolicy,
    EvidenceRequirement, GroundedResolutionPolicy, GroundedResolutionRuntime, HarnessInput,
    Proposition, ReasoningCandidate, ResolutionBudget, ResolutionResolver, ResolverClass,
    StandardGroundingPipeline, TrustedResolutionVerifier, Verdict,
};
use reasoning_harness_providers::{
    ExternalEvidenceAdmissionConfig, ExternalEvidenceAdmissionPolicy, ExternalEvidenceSourcePolicy,
    MCP_READONLY_RESOLVER_ID, McpReadOnlyResolver, McpReadOnlyResolverConfig,
};
use serde::Serialize;
use serde_json::Value;

const REPORT_SCHEMA: &str = "reason-mcp-knowledge-benchmark-v1";
const SOURCE: &str = "mcp:wikidata:typed_fact";
const AUTHORITY: &str = "wikidata_public";
const ACQUISITION_WINDOW_SECONDS: i64 = 120;

#[derive(Debug, Parser)]
#[command(name = "reason-mcp-knowledge-benchmark")]
struct Args {
    #[arg(long, default_value_t = 3)]
    trials: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExpectedOutcome {
    Accept,
    Reject,
    Unknown,
}

impl ExpectedOutcome {
    const fn verdict(self) -> Verdict {
        match self {
            Self::Accept => Verdict::Accept,
            Self::Reject => Verdict::Reject,
            Self::Unknown => Verdict::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CaseSpec {
    id: &'static str,
    workload: &'static str,
    entity_id: &'static str,
    property_id: &'static str,
    value_kind: &'static str,
    fact_key: &'static str,
    target_value: &'static str,
    expected: ExpectedOutcome,
}

fn cases() -> [CaseSpec; 8] {
    [
        CaseSpec {
            id: "tokyo_country",
            workload: "entity_relation",
            entity_id: "Q1490",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "tokyo.country",
            target_value: "Q17",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "paris_country",
            workload: "entity_relation",
            entity_id: "Q90",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "paris.country",
            target_value: "Q142",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "germany_capital",
            workload: "entity_relation",
            entity_id: "Q183",
            property_id: "P36",
            value_kind: "entity",
            fact_key: "germany.capital",
            target_value: "Q64",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "japan_continent",
            workload: "entity_relation",
            entity_id: "Q17",
            property_id: "P30",
            value_kind: "entity",
            fact_key: "japan.continent",
            target_value: "Q48",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "mount_fuji_elevation_source_value",
            workload: "quantity",
            entity_id: "Q39231",
            property_id: "P2044",
            value_kind: "quantity",
            fact_key: "mount_fuji.elevation_m",
            target_value: "3777.24",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "mount_fuji_elevation_disagreement",
            workload: "quantity_contradiction",
            entity_id: "Q39231",
            property_id: "P2044",
            value_kind: "quantity",
            fact_key: "mount_fuji.elevation_m",
            target_value: "3776",
            expected: ExpectedOutcome::Reject,
        },
        CaseSpec {
            id: "tokyo_wrong_country",
            workload: "contradiction",
            entity_id: "Q1490",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "tokyo.country",
            target_value: "Q142",
            expected: ExpectedOutcome::Reject,
        },
        CaseSpec {
            id: "tokyo_missing_date_of_death",
            workload: "missing_fact",
            entity_id: "Q1490",
            property_id: "P570",
            value_kind: "time",
            fact_key: "tokyo.date_of_death",
            target_value: "+2020-01-01T00:00:00Z",
            expected: ExpectedOutcome::Unknown,
        },
    ]
}

#[derive(Debug, Serialize)]
struct CaseReport {
    id: &'static str,
    workload: &'static str,
    trial: usize,
    expected: ExpectedOutcome,
    initial_verdict: Verdict,
    final_verdict: Verdict,
    terminal_status: reasoning_harness_core::ResolutionTerminalStatus,
    observed_value: Option<String>,
    calls: u64,
    elapsed_ms: u64,
    passed: bool,
}

#[derive(Debug, Serialize)]
struct Aggregate {
    cases: usize,
    trials_per_case: usize,
    samples: usize,
    passed_samples: usize,
    expectation_success_rate: f64,
    expected_accept_samples: usize,
    verified_recoveries: usize,
    accept_recovery_rate: f64,
    correct_rejections: usize,
    correct_unknowns: usize,
    false_acceptances: usize,
    total_calls: u64,
    mean_elapsed_ms: f64,
    p50_elapsed_ms: u64,
    p95_elapsed_ms: u64,
    max_elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: &'static str,
    adapter: &'static str,
    source: &'static str,
    authority: &'static str,
    acquisition_window_seconds: i64,
    note: &'static str,
    samples: Vec<CaseReport>,
    aggregate: Aggregate,
}

fn unix_seconds() -> Result<i64, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    i64::try_from(seconds).map_err(|_| "current time exceeds i64".into())
}

fn input_and_candidate(case: CaseSpec, evaluation_time: i64) -> (HarnessInput, ReasoningCandidate) {
    let proposition = Proposition {
        key: case.fact_key.into(),
        value: case.target_value.into(),
    };
    (
        HarnessInput {
            task: format!("Live Wikidata typed-fact benchmark: {}", case.id),
            evidence: vec![],
            hypotheses: vec![proposition.clone()],
            assumptions: vec![],
            evidence_requirements: vec![EvidenceRequirement {
                proposition: proposition.clone(),
                as_of_unix_seconds: Some(evaluation_time),
                scope: None,
                minimum_authority_class: Some(AUTHORITY.into()),
            }],
            authority_policy: EvidenceAuthorityPolicy {
                ranks: BTreeMap::from([(AUTHORITY.into(), 10)]),
            },
        },
        ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: format!("claim:{}", case.id),
                statement: format!("{}={}", case.fact_key, case.target_value),
                proposed_state: EpistemicState::Supported,
                proposition: Some(proposition),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        },
    )
}

fn resolver(case: CaseSpec) -> McpReadOnlyResolver {
    let mut config = McpReadOnlyResolverConfig::with_defaults(
        "wikidata-typed-fact-benchmark",
        PathBuf::from("python3"),
        "typed_fact",
        SOURCE,
    );
    config.args = vec!["scripts/wikidata_typed_fact_mcp.py".into()];
    config.fixed_arguments = BTreeMap::from([
        ("entity_id".into(), Value::String(case.entity_id.into())),
        ("property_id".into(), Value::String(case.property_id.into())),
        ("value_kind".into(), Value::String(case.value_kind.into())),
        ("fact_key".into(), Value::String(case.fact_key.into())),
    ]);
    config.timeout_ms = 15_000;
    config.max_response_bytes = 256 * 1024;
    McpReadOnlyResolver::new(config)
}

fn admission(evaluation_time: i64) -> ExternalEvidenceAdmissionPolicy {
    ExternalEvidenceAdmissionPolicy::new(ExternalEvidenceAdmissionConfig {
        resolver_name: MCP_READONLY_RESOLVER_ID,
        evaluation_time_unix_seconds: evaluation_time,
        authority_policy: EvidenceAuthorityPolicy {
            ranks: BTreeMap::from([(AUTHORITY.into(), 10)]),
        },
        minimum_authority_class: Some(AUTHORITY.into()),
        required_scope: None,
        sources: BTreeMap::from([(
            SOURCE.into(),
            ExternalEvidenceSourcePolicy {
                authority_class: AUTHORITY.into(),
                max_age_seconds: 300,
                scope: None,
            },
        )]),
    })
}

fn run_case(case: CaseSpec, trial: usize) -> Result<CaseReport, String> {
    let evaluation_time = unix_seconds()?.saturating_add(ACQUISITION_WINDOW_SECONDS);
    let (input, candidate) = input_and_candidate(case, evaluation_time);
    let resolver = resolver(case);
    let admission = admission(evaluation_time);
    let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
    let trusted_verifiers: [&dyn TrustedResolutionVerifier; 0] = [];
    let outcome = GroundedResolutionRuntime {
        pipeline: &StandardGroundingPipeline,
        planner: &reasoning_harness_core::DefaultResolutionPlanner,
        evidence_admission: &admission,
        resolvers: &resolvers,
        trusted_verifiers: &trusted_verifiers,
        renderer: &CanonicalFinalAnswerRenderer,
    }
    .run(
        input,
        candidate,
        &GroundedResolutionPolicy {
            budget: ResolutionBudget {
                max_attempts: 1,
                allowed_resolver_classes: BTreeSet::from([ResolverClass::EvidenceAcquisition]),
                required_authority_class: Some(AUTHORITY.into()),
                ..ResolutionBudget::default()
            },
            ..GroundedResolutionPolicy::default()
        },
    )
    .map_err(|error| error.to_string())?;

    let observed_value = outcome
        .final_artifact
        .evidence
        .iter()
        .find_map(|evidence| evidence.facts.get(case.fact_key).cloned());
    Ok(CaseReport {
        id: case.id,
        workload: case.workload,
        trial,
        expected: case.expected,
        initial_verdict: outcome.initial_verdict,
        final_verdict: outcome.final_verdict,
        terminal_status: outcome.terminal_status,
        observed_value,
        calls: u64::from(outcome.usage.calls),
        elapsed_ms: outcome.usage.elapsed_ms,
        passed: outcome.final_verdict == case.expected.verdict(),
    })
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = ((sorted.len() - 1) * percentile + 99) / 100;
    sorted[rank.min(sorted.len() - 1)]
}

fn aggregate(samples: &[CaseReport], trials: usize) -> Aggregate {
    let passed_samples = samples.iter().filter(|sample| sample.passed).count();
    let expected_accept_samples = samples
        .iter()
        .filter(|sample| sample.expected == ExpectedOutcome::Accept)
        .count();
    let verified_recoveries = samples
        .iter()
        .filter(|sample| {
            sample.expected == ExpectedOutcome::Accept
                && sample.initial_verdict == Verdict::Unknown
                && sample.final_verdict == Verdict::Accept
        })
        .count();
    let correct_rejections = samples
        .iter()
        .filter(|sample| {
            sample.expected == ExpectedOutcome::Reject && sample.final_verdict == Verdict::Reject
        })
        .count();
    let correct_unknowns = samples
        .iter()
        .filter(|sample| {
            sample.expected == ExpectedOutcome::Unknown && sample.final_verdict == Verdict::Unknown
        })
        .count();
    let false_acceptances = samples
        .iter()
        .filter(|sample| sample.expected != ExpectedOutcome::Accept && sample.final_verdict == Verdict::Accept)
        .count();
    let total_calls = samples.iter().map(|sample| sample.calls).sum();
    let latencies = samples.iter().map(|sample| sample.elapsed_ms).collect::<Vec<_>>();
    let mean_elapsed_ms = if latencies.is_empty() {
        0.0
    } else {
        latencies.iter().sum::<u64>() as f64 / latencies.len() as f64
    };
    Aggregate {
        cases: cases().len(),
        trials_per_case: trials,
        samples: samples.len(),
        passed_samples,
        expectation_success_rate: if samples.is_empty() {
            0.0
        } else {
            passed_samples as f64 / samples.len() as f64
        },
        expected_accept_samples,
        verified_recoveries,
        accept_recovery_rate: if expected_accept_samples == 0 {
            0.0
        } else {
            verified_recoveries as f64 / expected_accept_samples as f64
        },
        correct_rejections,
        correct_unknowns,
        false_acceptances,
        total_calls,
        mean_elapsed_ms,
        p50_elapsed_ms: percentile(&latencies, 50),
        p95_elapsed_ms: percentile(&latencies, 95),
        max_elapsed_ms: latencies.iter().copied().max().unwrap_or(0),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.trials == 0 {
        return Err("--trials must be at least 1".into());
    }
    let mut samples = Vec::new();
    for trial in 1..=args.trials {
        for case in cases() {
            samples.push(run_case(case, trial)?);
        }
    }
    let aggregate = aggregate(&samples, args.trials);
    let report = Report {
        schema_version: REPORT_SCHEMA,
        adapter: MCP_READONLY_RESOLVER_ID,
        source: SOURCE,
        authority: AUTHORITY,
        acquisition_window_seconds: ACQUISITION_WINDOW_SECONDS,
        note: "Latency includes MCP process spawn, Wikidata HTTP acquisition, response parsing, admission, and resolution runtime. It excludes LLM generation. A bounded future evaluation window isolates the known live-acquisition timestamp boundary. Quantity cases deliberately include both the source-native live value and a conflicting commonly cited value so source disagreement is measured separately from adapter success.",
        samples,
        aggregate,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
