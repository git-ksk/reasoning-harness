use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    io::{self, Read},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use clap::Parser;
use reasoning_harness_core::{
    CandidateClaim, CanonicalFinalAnswerRenderer, EpistemicState, EvidenceAuthorityPolicy,
    EvidenceRequirement, FinalAnswerRenderer, FinalClaimMode, GroundedResolutionOutcome,
    GroundedResolutionPolicy, GroundedResolutionRuntime, HarnessInput, Proposition,
    ReasoningCandidate, RejectAllEvidenceAdmission, ResolutionBudget, ResolutionResolver,
    ResolutionTerminalStatus, ResolverClass, ScopeCoverage, StandardGroundingPipeline,
    TrustedResolutionVerifier, Verdict,
};
use reasoning_harness_providers::{
    EXTERNAL_COMMAND_RESOLVER_ID, ExternalCommandResolver, ExternalCommandResolverConfig,
    ExternalEvidenceAdmissionConfig, ExternalEvidenceAdmissionPolicy, ExternalEvidenceSourcePolicy,
    TrustedCommandVerifier, TrustedCommandVerifierConfig,
};
use serde::Serialize;
use serde_json::json;

const REPORT_SCHEMA: &str = "reason-external-resolution-acceptance-v1";
const EVALUATION_TIME: i64 = 2_000;
const SOURCE: &str = "acceptance:external";
const LIVE_AWS_SOURCE: &str = "aws:whats-new:feed";
const LIVE_AWS_URL: &str = "https://aws.amazon.com/about-aws/whats-new/recent/feed/";

#[derive(Debug, Parser)]
#[command(name = "reason-external-resolution-acceptance")]
struct Args {
    #[arg(long)]
    output: Option<PathBuf>,
    /// Run the separately reported network-dependent AWS public-information smoke.
    #[arg(long, default_value_t = false)]
    live_aws: bool,
    #[arg(long, hide = true)]
    adapter_case: Option<String>,
    #[arg(long, hide = true)]
    verifier_case: Option<String>,
    #[arg(long, hide = true, default_value_t = false)]
    live_aws_adapter: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExpectedOutcome {
    Accept,
    Reject,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
enum AcquisitionKind {
    FreshFact,
    Opaque,
    Stale,
    WrongScope,
    Irrelevant,
    Conflict,
    OperationalFailure,
    BudgetExhaustion,
}

#[derive(Debug, Clone, Copy)]
struct CaseSpec {
    id: &'static str,
    workload_class: &'static str,
    target_key: &'static str,
    target_value: &'static str,
    expected: ExpectedOutcome,
    acquisition: AcquisitionKind,
    trusted_verifier: bool,
}

fn cases() -> [CaseSpec; 8] {
    [
        CaseSpec {
            id: "fresh_external_fact_recovery",
            workload_class: "fresh_public_style_fact",
            target_key: "service.region",
            target_value: "eu-west-1",
            expected: ExpectedOutcome::Accept,
            acquisition: AcquisitionKind::FreshFact,
            trusted_verifier: false,
        },
        CaseSpec {
            id: "opaque_data_then_trusted_verifier",
            workload_class: "acquisition_then_independent_verification",
            target_key: "policy.signature_valid",
            target_value: "true",
            expected: ExpectedOutcome::Accept,
            acquisition: AcquisitionKind::Opaque,
            trusted_verifier: true,
        },
        CaseSpec {
            id: "stale_external_evidence",
            workload_class: "freshness_rejection",
            target_key: "service.region",
            target_value: "eu-west-1",
            expected: ExpectedOutcome::Unknown,
            acquisition: AcquisitionKind::Stale,
            trusted_verifier: false,
        },
        CaseSpec {
            id: "wrong_scope_external_evidence",
            workload_class: "scope_rejection",
            target_key: "service.region",
            target_value: "eu-west-1",
            expected: ExpectedOutcome::Unknown,
            acquisition: AcquisitionKind::WrongScope,
            trusted_verifier: false,
        },
        CaseSpec {
            id: "irrelevant_external_data",
            workload_class: "insufficient_acquisition",
            target_key: "service.region",
            target_value: "eu-west-1",
            expected: ExpectedOutcome::Unknown,
            acquisition: AcquisitionKind::Irrelevant,
            trusted_verifier: false,
        },
        CaseSpec {
            id: "conflicting_external_evidence",
            workload_class: "trusted_contradiction",
            target_key: "service.region",
            target_value: "eu-west-1",
            expected: ExpectedOutcome::Reject,
            acquisition: AcquisitionKind::Conflict,
            trusted_verifier: false,
        },
        CaseSpec {
            id: "external_transport_failure",
            workload_class: "operational_failure",
            target_key: "service.region",
            target_value: "eu-west-1",
            expected: ExpectedOutcome::Unknown,
            acquisition: AcquisitionKind::OperationalFailure,
            trusted_verifier: false,
        },
        CaseSpec {
            id: "external_budget_exhaustion",
            workload_class: "budget_exhaustion",
            target_key: "service.region",
            target_value: "eu-west-1",
            expected: ExpectedOutcome::Unknown,
            acquisition: AcquisitionKind::BudgetExhaustion,
            trusted_verifier: false,
        },
    ]
}

fn find_case(id: &str) -> Option<CaseSpec> {
    cases().into_iter().find(|case| case.id == id)
}

#[derive(Debug, Serialize)]
struct CaseReport {
    id: String,
    workload_class: String,
    expected: ExpectedOutcome,
    initial_verdict: Verdict,
    final_verdict: Verdict,
    acquisition_terminal: ResolutionTerminalStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    verifier_terminal: Option<ResolutionTerminalStatus>,
    acquisition_success: bool,
    verification_success: bool,
    grounded_target: bool,
    unsupported_grounded_claims: usize,
    missed_target_insufficiency: usize,
    false_abstention: usize,
    calls: u64,
    elapsed_ms: u64,
    added_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_microusd: Option<u64>,
    typed_operational_failure: bool,
    final_claim_coverage: f64,
}

#[derive(Debug, Default, Serialize)]
struct Aggregate {
    cases: usize,
    initially_unsupported_cases: usize,
    verified_recoveries: usize,
    verified_recovery_rate: f64,
    unsupported_grounded_claims: usize,
    missed_target_insufficiency: usize,
    correct_abstentions: usize,
    false_abstentions: usize,
    acquisition_successes: usize,
    verification_successes: usize,
    calls: u64,
    elapsed_ms: u64,
    added_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_microusd: Option<u64>,
    typed_operational_failures: usize,
    mean_final_claim_coverage: f64,
    acceptance_passed: bool,
}

#[derive(Debug, Serialize)]
struct LiveAwsReport {
    source: &'static str,
    url: &'static str,
    observed_at_unix_seconds: i64,
    initial_verdict: Verdict,
    final_verdict: Verdict,
    terminal_status: ResolutionTerminalStatus,
    acquisition_success: bool,
    grounded_target: bool,
    unsupported_grounded_claims: usize,
    calls: u64,
    elapsed_ms: u64,
    observation: String,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: &'static str,
    suite: &'static str,
    frozen_research_inputs_used: bool,
    cases: Vec<CaseReport>,
    aggregate: Aggregate,
    #[serde(skip_serializing_if = "Option::is_none")]
    live_aws: Option<LiveAwsReport>,
}

fn target(case: CaseSpec) -> Proposition {
    Proposition {
        key: case.target_key.into(),
        value: case.target_value.into(),
    }
}

fn input_and_candidate(case: CaseSpec) -> (HarnessInput, ReasoningCandidate) {
    let proposition = target(case);
    (
        HarnessInput {
            task: format!("external acceptance {}", case.id),
            evidence: vec![],
            hypotheses: vec![proposition.clone()],
            assumptions: vec![],
            evidence_requirements: if matches!(case.acquisition, AcquisitionKind::Opaque) {
                vec![]
            } else {
                vec![EvidenceRequirement {
                    proposition: proposition.clone(),
                    as_of_unix_seconds: Some(EVALUATION_TIME),
                    scope: matches!(case.acquisition, AcquisitionKind::WrongScope)
                        .then(|| scope("eu-west-1")),
                    minimum_authority_class: Some("primary".into()),
                }]
            },
            authority_policy: EvidenceAuthorityPolicy {
                ranks: BTreeMap::from([("primary".into(), 10)]),
            },
        },
        ReasoningCandidate {
            claims: vec![CandidateClaim {
                id: "candidate-target".into(),
                statement: "model-proposed target".into(),
                proposed_state: EpistemicState::Supported,
                proposition: Some(proposition),
                evidence_ids: vec![],
            }],
            inferences: vec![],
        },
    )
}

fn scope(region: &str) -> BTreeMap<String, ScopeCoverage> {
    BTreeMap::from([(
        "region".into(),
        ScopeCoverage::Values {
            values: BTreeSet::from([region.into()]),
        },
    )])
}

fn admission(case: CaseSpec) -> ExternalEvidenceAdmissionPolicy {
    let required_scope =
        matches!(case.acquisition, AcquisitionKind::WrongScope).then(|| scope("eu-west-1"));
    let source_scope = required_scope
        .as_ref()
        .map(|_| BTreeMap::from([("region".into(), ScopeCoverage::Any)]));
    ExternalEvidenceAdmissionPolicy::new(ExternalEvidenceAdmissionConfig {
        resolver_name: EXTERNAL_COMMAND_RESOLVER_ID,
        evaluation_time_unix_seconds: EVALUATION_TIME,
        authority_policy: EvidenceAuthorityPolicy {
            ranks: BTreeMap::from([("primary".into(), 10)]),
        },
        minimum_authority_class: Some("primary".into()),
        required_scope,
        sources: BTreeMap::from([(
            SOURCE.into(),
            ExternalEvidenceSourcePolicy {
                authority_class: "primary".into(),
                max_age_seconds: 60,
                scope: source_scope,
            },
        )]),
    })
}

fn acquisition_policy(case: CaseSpec) -> GroundedResolutionPolicy {
    GroundedResolutionPolicy {
        budget: ResolutionBudget {
            max_attempts: 1,
            max_added_tokens: matches!(case.acquisition, AcquisitionKind::BudgetExhaustion)
                .then_some(0),
            allowed_resolver_classes: BTreeSet::from([ResolverClass::EvidenceAcquisition]),
            required_authority_class: None,
            ..ResolutionBudget::default()
        },
        proposition_resolver_class: ResolverClass::EvidenceAcquisition,
        ..GroundedResolutionPolicy::default()
    }
}

fn verifier_policy() -> GroundedResolutionPolicy {
    GroundedResolutionPolicy {
        budget: ResolutionBudget {
            max_attempts: 1,
            allowed_resolver_classes: BTreeSet::from([ResolverClass::DeterministicVerifier]),
            ..ResolutionBudget::default()
        },
        proposition_resolver_class: ResolverClass::DeterministicVerifier,
        ..GroundedResolutionPolicy::default()
    }
}

fn artifact_input(outcome: &GroundedResolutionOutcome) -> HarnessInput {
    HarnessInput {
        task: outcome.final_artifact.task.clone(),
        evidence: outcome.final_artifact.evidence.clone(),
        hypotheses: outcome.final_artifact.hypotheses.clone(),
        assumptions: outcome.final_artifact.assumptions.clone(),
        evidence_requirements: outcome.final_artifact.evidence_requirements.clone(),
        authority_policy: outcome.final_artifact.authority_policy.clone(),
    }
}

fn authorized_target(outcome: &GroundedResolutionOutcome, proposition: &Proposition) -> bool {
    outcome.final_artifact.claims.iter().any(|claim| {
        claim.proposition.as_ref() == Some(proposition)
            && matches!(
                claim.state,
                EpistemicState::Known | EpistemicState::Supported
            )
    })
}

fn rendered_metrics(
    outcome: &GroundedResolutionOutcome,
    proposition: &Proposition,
) -> (bool, usize) {
    let rendered =
        CanonicalFinalAnswerRenderer.render(&outcome.final_artifact, outcome.final_verdict);
    let grounded_target = rendered
        .factual_claims
        .iter()
        .any(|claim| claim.mode == FinalClaimMode::Grounded && &claim.proposition == proposition);
    let unsupported = rendered
        .factual_claims
        .iter()
        .filter(|claim| {
            claim.mode == FinalClaimMode::Grounded
                && !outcome.final_artifact.claims.iter().any(|artifact_claim| {
                    artifact_claim.proposition.as_ref() == Some(&claim.proposition)
                        && matches!(
                            artifact_claim.state,
                            EpistemicState::Known | EpistemicState::Supported
                        )
                })
        })
        .count();
    (grounded_target, unsupported)
}

fn sum_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (None, None) => None,
        (left, right) => Some(left.unwrap_or(0).saturating_add(right.unwrap_or(0))),
    }
}

fn run_case(case: CaseSpec) -> Result<CaseReport, String> {
    let exe = env::current_exe().map_err(|error| error.to_string())?;
    let (input, candidate) = input_and_candidate(case);
    let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig::with_defaults(
        exe.clone(),
        vec!["--adapter-case".into(), case.id.into()],
    ));
    let resolver_refs: [&dyn ResolutionResolver; 1] = [&resolver];
    let no_verifiers: [&dyn TrustedResolutionVerifier; 0] = [];
    let admission = admission(case);
    let acquisition = GroundedResolutionRuntime {
        pipeline: &StandardGroundingPipeline,
        planner: &reasoning_harness_core::DefaultResolutionPlanner,
        evidence_admission: &admission,
        resolvers: &resolver_refs,
        trusted_verifiers: &no_verifiers,
        renderer: &CanonicalFinalAnswerRenderer,
    }
    .run(input, candidate.clone(), &acquisition_policy(case))
    .map_err(|error| error.to_string())?;

    let mut final_outcome = acquisition.clone();
    let mut verifier_terminal = None;
    let mut verification_success = false;
    let mut verifier_usage = reasoning_harness_core::ResolutionUsage::default();
    if case.trusted_verifier && acquisition.final_verdict != Verdict::Accept {
        let verifier = TrustedCommandVerifier::new(TrustedCommandVerifierConfig::with_defaults(
            "acceptance-reference-oracle".into(),
            exe,
            vec!["--verifier-case".into(), case.id.into()],
        ));
        let trusted: [&dyn TrustedResolutionVerifier; 1] = [&verifier];
        let no_resolvers: [&dyn ResolutionResolver; 0] = [];
        let verified = GroundedResolutionRuntime {
            pipeline: &StandardGroundingPipeline,
            planner: &reasoning_harness_core::DefaultResolutionPlanner,
            evidence_admission: &RejectAllEvidenceAdmission,
            resolvers: &no_resolvers,
            trusted_verifiers: &trusted,
            renderer: &CanonicalFinalAnswerRenderer,
        }
        .run(artifact_input(&acquisition), candidate, &verifier_policy())
        .map_err(|error| error.to_string())?;
        verification_success = verified
            .attempts
            .iter()
            .any(|attempt| attempt.verification_receipts > 0);
        verifier_terminal = Some(verified.terminal_status);
        verifier_usage = verified.usage.clone();
        final_outcome = verified;
    }

    let proposition = target(case);
    let (grounded_target, unsupported_grounded_claims) =
        rendered_metrics(&final_outcome, &proposition);
    let acquisition_success = acquisition.attempts.iter().any(|attempt| {
        matches!(
            attempt.status,
            reasoning_harness_core::ResolutionAttemptStatus::AppliedEvidence
                | reasoning_harness_core::ResolutionAttemptStatus::RejectedUntrustedEvidence
        )
    });
    let expected_unknown = case.expected == ExpectedOutcome::Unknown;
    let missed_target_insufficiency = usize::from(expected_unknown && grounded_target);
    let false_abstention =
        usize::from(case.expected == ExpectedOutcome::Accept && !grounded_target);
    let typed_operational_failure = matches!(
        acquisition.terminal_status,
        ResolutionTerminalStatus::TimedOut
            | ResolutionTerminalStatus::Denied
            | ResolutionTerminalStatus::OperationalFailure
            | ResolutionTerminalStatus::Exhausted
    ) && matches!(
        case.acquisition,
        AcquisitionKind::OperationalFailure | AcquisitionKind::BudgetExhaustion
    );
    let calls = acquisition.usage.calls.saturating_add(verifier_usage.calls);
    let elapsed_ms = acquisition
        .usage
        .elapsed_ms
        .saturating_add(verifier_usage.elapsed_ms);
    let added_tokens = acquisition
        .usage
        .added_tokens
        .saturating_add(verifier_usage.added_tokens);
    let cost_microusd = sum_optional(
        acquisition.usage.cost_microusd,
        verifier_usage.cost_microusd,
    );

    if final_outcome.initial_verdict != Verdict::Unknown
        && acquisition.initial_verdict != Verdict::Unknown
    {
        return Err(format!("{} did not begin unsupported", case.id));
    }
    let expected_verdict = match case.expected {
        ExpectedOutcome::Accept => Verdict::Accept,
        ExpectedOutcome::Reject => Verdict::Reject,
        ExpectedOutcome::Unknown => Verdict::Unknown,
    };
    if final_outcome.final_verdict != expected_verdict {
        return Err(format!(
            "{} final verdict {:?}, expected {:?}",
            case.id, final_outcome.final_verdict, expected_verdict
        ));
    }
    if case.expected == ExpectedOutcome::Accept && !authorized_target(&final_outcome, &proposition)
    {
        return Err(format!("{} accepted without target authority", case.id));
    }

    Ok(CaseReport {
        id: case.id.into(),
        workload_class: case.workload_class.into(),
        expected: case.expected,
        initial_verdict: acquisition.initial_verdict,
        final_verdict: final_outcome.final_verdict,
        acquisition_terminal: acquisition.terminal_status,
        verifier_terminal,
        acquisition_success,
        verification_success,
        grounded_target,
        unsupported_grounded_claims,
        missed_target_insufficiency,
        false_abstention,
        calls,
        elapsed_ms,
        added_tokens,
        cost_microusd,
        typed_operational_failure,
        final_claim_coverage: final_outcome.finalization.factual_claim_coverage,
    })
}

fn aggregate(cases: &[CaseReport]) -> Aggregate {
    let initially_unsupported_cases = cases
        .iter()
        .filter(|case| case.initial_verdict == Verdict::Unknown)
        .count();
    let verified_recoveries = cases
        .iter()
        .filter(|case| {
            case.initial_verdict == Verdict::Unknown && case.final_verdict == Verdict::Accept
        })
        .count();
    let expected_unknown = cases
        .iter()
        .filter(|case| case.expected == ExpectedOutcome::Unknown)
        .count();
    let correct_abstentions = cases
        .iter()
        .filter(|case| case.expected == ExpectedOutcome::Unknown && !case.grounded_target)
        .count();
    let unsupported_grounded_claims = cases
        .iter()
        .map(|case| case.unsupported_grounded_claims)
        .sum();
    let missed_target_insufficiency = cases
        .iter()
        .map(|case| case.missed_target_insufficiency)
        .sum();
    let false_abstentions = cases.iter().map(|case| case.false_abstention).sum();
    let acquisition_successes = cases.iter().filter(|case| case.acquisition_success).count();
    let verification_successes = cases
        .iter()
        .filter(|case| case.verification_success)
        .count();
    let calls = cases.iter().map(|case| case.calls).sum();
    let elapsed_ms = cases.iter().map(|case| case.elapsed_ms).sum();
    let added_tokens = cases.iter().map(|case| case.added_tokens).sum();
    let cost_microusd = cases
        .iter()
        .fold(None, |total, case| sum_optional(total, case.cost_microusd));
    let typed_operational_failures = cases
        .iter()
        .filter(|case| case.typed_operational_failure)
        .count();
    let mean_final_claim_coverage = if cases.is_empty() {
        0.0
    } else {
        cases
            .iter()
            .map(|case| case.final_claim_coverage)
            .sum::<f64>()
            / cases.len() as f64
    };
    let verified_recovery_rate = if initially_unsupported_cases == 0 {
        0.0
    } else {
        verified_recoveries as f64 / initially_unsupported_cases as f64
    };
    Aggregate {
        cases: cases.len(),
        initially_unsupported_cases,
        verified_recoveries,
        verified_recovery_rate,
        unsupported_grounded_claims,
        missed_target_insufficiency,
        correct_abstentions,
        false_abstentions,
        acquisition_successes,
        verification_successes,
        calls,
        elapsed_ms,
        added_tokens,
        cost_microusd,
        typed_operational_failures,
        mean_final_claim_coverage,
        acceptance_passed: unsupported_grounded_claims == 0
            && missed_target_insufficiency == 0
            && verified_recoveries >= 1
            && correct_abstentions == expected_unknown
            && false_abstentions == 0,
    }
}

fn adapter_response(case: CaseSpec) -> serde_json::Value {
    match case.acquisition {
        AcquisitionKind::OperationalFailure => json!({
            "schema_version": "reason-external-resolver-response-v1",
            "failure": {"kind": "transport"}
        }),
        AcquisitionKind::BudgetExhaustion => json!({
            "schema_version": "reason-external-resolver-response-v1",
            "contribution": {"kind": "no_result"},
            "cost": {"added_tokens": 1}
        }),
        kind => {
            let (observed, facts, scope_value, observation) = match kind {
                AcquisitionKind::FreshFact => (
                    1_980,
                    BTreeMap::from([(case.target_key.to_string(), case.target_value.to_string())]),
                    None,
                    "fresh exact external fact".to_string(),
                ),
                AcquisitionKind::Opaque => (
                    1_980,
                    BTreeMap::new(),
                    None,
                    "opaque signed payload requiring independent verifier".to_string(),
                ),
                AcquisitionKind::Stale => (
                    1_800,
                    BTreeMap::from([(case.target_key.to_string(), case.target_value.to_string())]),
                    None,
                    "stale exact external fact".to_string(),
                ),
                AcquisitionKind::WrongScope => (
                    1_980,
                    BTreeMap::from([(case.target_key.to_string(), case.target_value.to_string())]),
                    Some(scope("us-east-1")),
                    "fresh fact from wrong region scope".to_string(),
                ),
                AcquisitionKind::Irrelevant => (
                    1_980,
                    BTreeMap::from([("service.status".to_string(), "healthy".to_string())]),
                    None,
                    "fresh but irrelevant external fact".to_string(),
                ),
                AcquisitionKind::Conflict => (
                    1_980,
                    BTreeMap::from([(case.target_key.to_string(), "us-east-1".to_string())]),
                    None,
                    "fresh external fact contradicting target".to_string(),
                ),
                AcquisitionKind::OperationalFailure | AcquisitionKind::BudgetExhaustion => {
                    unreachable!()
                }
            };
            json!({
                "schema_version": "reason-external-resolver-response-v1",
                "contribution": {
                    "kind": "acquired_evidence",
                    "evidence": [{
                        "id": "e1",
                        "source": SOURCE,
                        "observation": observation,
                        "facts": facts,
                        "acquisition_metadata": {
                            "observed_at_unix_seconds": observed,
                            "retrieved_at_unix_seconds": 1_990,
                            "scope": scope_value,
                            "claimed_authority_class": "primary"
                        }
                    }]
                }
            })
        }
    }
}

fn verifier_response(case: CaseSpec) -> serde_json::Value {
    if case.trusted_verifier {
        json!({
            "schema_version": "reason-trusted-verifier-response-v1",
            "result": {"conclusion": "supported", "evidence_ids": ["e1"]}
        })
    } else {
        json!({
            "schema_version": "reason-trusted-verifier-response-v1",
            "result": {"conclusion": "no_result", "evidence_ids": []}
        })
    }
}

fn unix_seconds() -> Result<i64, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    i64::try_from(seconds).map_err(|_| "current time exceeds i64".into())
}

async fn live_aws_adapter_response() -> serde_json::Value {
    let now = match unix_seconds() {
        Ok(now) => now,
        Err(_) => {
            return json!({"schema_version":"reason-external-resolver-response-v1","failure":{"kind":"protocol"}});
        }
    };
    let response = match reqwest::Client::new().get(LIVE_AWS_URL).send().await {
        Ok(response) => response,
        Err(_) => {
            return json!({"schema_version":"reason-external-resolver-response-v1","failure":{"kind":"transport"}});
        }
    };
    let status = response.status();
    let text = match response.text().await {
        Ok(text) => text,
        Err(_) => {
            return json!({"schema_version":"reason-external-resolver-response-v1","failure":{"kind":"transport"}});
        }
    };
    if !status.is_success() || !text.contains("<rss") || !text.contains("<lastBuildDate>") {
        return json!({"schema_version":"reason-external-resolver-response-v1","failure":{"kind":"protocol"}});
    }
    let last_build = text
        .split("<lastBuildDate>")
        .nth(1)
        .and_then(|value| value.split("</lastBuildDate>").next())
        .unwrap_or("present");
    json!({
        "schema_version": "reason-external-resolver-response-v1",
        "contribution": {
            "kind": "acquired_evidence",
            "evidence": [{
                "id": "aws-feed-live",
                "source": LIVE_AWS_SOURCE,
                "observation": format!("HTTP {}; RSS lastBuildDate={last_build}", status.as_u16()),
                "facts": {"aws.whats_new_feed_available": "true"},
                "acquisition_metadata": {
                    "observed_at_unix_seconds": now,
                    "retrieved_at_unix_seconds": now,
                    "claimed_authority_class": "aws_public"
                }
            }]
        }
    })
}

async fn run_live_aws() -> Result<LiveAwsReport, String> {
    let now = unix_seconds()?;
    let exe = env::current_exe().map_err(|error| error.to_string())?;
    let target = Proposition {
        key: "aws.whats_new_feed_available".into(),
        value: "true".into(),
    };
    let input = HarnessInput {
        task: "Confirm that the current public AWS What's New RSS feed is available".into(),
        evidence: vec![],
        hypotheses: vec![target.clone()],
        assumptions: vec![],
        evidence_requirements: vec![],
        authority_policy: EvidenceAuthorityPolicy {
            ranks: BTreeMap::from([("aws_public".into(), 10)]),
        },
    };
    let candidate = ReasoningCandidate {
        claims: vec![CandidateClaim {
            id: "aws-live-target".into(),
            statement: "AWS feed is available".into(),
            proposed_state: EpistemicState::Supported,
            proposition: Some(target.clone()),
            evidence_ids: vec![],
        }],
        inferences: vec![],
    };
    let resolver = ExternalCommandResolver::new(ExternalCommandResolverConfig::with_defaults(
        exe,
        vec!["--live-aws-adapter".into()],
    ));
    let admission = ExternalEvidenceAdmissionPolicy::new(ExternalEvidenceAdmissionConfig {
        resolver_name: EXTERNAL_COMMAND_RESOLVER_ID,
        evaluation_time_unix_seconds: now,
        authority_policy: EvidenceAuthorityPolicy {
            ranks: BTreeMap::from([("aws_public".into(), 10)]),
        },
        minimum_authority_class: Some("aws_public".into()),
        required_scope: None,
        sources: BTreeMap::from([(
            LIVE_AWS_SOURCE.into(),
            ExternalEvidenceSourcePolicy {
                authority_class: "aws_public".into(),
                max_age_seconds: 300,
                scope: None,
            },
        )]),
    });
    let resolvers: [&dyn ResolutionResolver; 1] = [&resolver];
    let no_verifiers: [&dyn TrustedResolutionVerifier; 0] = [];
    let outcome = GroundedResolutionRuntime {
        pipeline: &StandardGroundingPipeline,
        planner: &reasoning_harness_core::DefaultResolutionPlanner,
        evidence_admission: &admission,
        resolvers: &resolvers,
        trusted_verifiers: &no_verifiers,
        renderer: &CanonicalFinalAnswerRenderer,
    }
    .run(
        input,
        candidate,
        &GroundedResolutionPolicy {
            budget: ResolutionBudget {
                max_attempts: 1,
                allowed_resolver_classes: BTreeSet::from([ResolverClass::EvidenceAcquisition]),
                required_authority_class: Some("aws_public".into()),
                ..ResolutionBudget::default()
            },
            ..GroundedResolutionPolicy::default()
        },
    )
    .map_err(|error| error.to_string())?;
    let (grounded_target, unsupported_grounded_claims) = rendered_metrics(&outcome, &target);
    let evidence = outcome
        .final_artifact
        .evidence
        .iter()
        .find(|evidence| evidence.id == "aws-feed-live")
        .ok_or_else(|| "live AWS evidence was not admitted".to_string())?;
    if outcome.final_verdict != Verdict::Accept
        || !grounded_target
        || unsupported_grounded_claims != 0
    {
        return Err("live AWS recovery did not satisfy safe acceptance".into());
    }
    Ok(LiveAwsReport {
        source: LIVE_AWS_SOURCE,
        url: LIVE_AWS_URL,
        observed_at_unix_seconds: now,
        initial_verdict: outcome.initial_verdict,
        final_verdict: outcome.final_verdict,
        terminal_status: outcome.terminal_status,
        acquisition_success: true,
        grounded_target,
        unsupported_grounded_claims,
        calls: outcome.usage.calls,
        elapsed_ms: outcome.usage.elapsed_ms,
        observation: evidence.observation.clone(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Some(id) = args.adapter_case.as_deref() {
        let case = find_case(id).ok_or("unknown adapter case")?;
        let mut stdin = String::new();
        io::stdin().read_to_string(&mut stdin)?;
        let _: serde_json::Value = serde_json::from_str(&stdin)?;
        println!("{}", serde_json::to_string(&adapter_response(case))?);
        return Ok(());
    }
    if let Some(id) = args.verifier_case.as_deref() {
        let case = find_case(id).ok_or("unknown verifier case")?;
        let mut stdin = String::new();
        io::stdin().read_to_string(&mut stdin)?;
        let _: serde_json::Value = serde_json::from_str(&stdin)?;
        println!("{}", serde_json::to_string(&verifier_response(case))?);
        return Ok(());
    }
    if args.live_aws_adapter {
        let mut stdin = String::new();
        io::stdin().read_to_string(&mut stdin)?;
        let _: serde_json::Value = serde_json::from_str(&stdin)?;
        println!(
            "{}",
            serde_json::to_string(&live_aws_adapter_response().await)?
        );
        return Ok(());
    }

    let mut reports = Vec::new();
    for case in cases() {
        reports.push(run_case(case)?);
    }
    let aggregate = aggregate(&reports);
    if !aggregate.acceptance_passed {
        return Err("deterministic external-resolution acceptance gate failed".into());
    }
    let live_aws = if args.live_aws {
        Some(run_live_aws().await?)
    } else {
        None
    };
    let report = Report {
        schema_version: REPORT_SCHEMA,
        suite: "external-resolution-acceptance-v1",
        frozen_research_inputs_used: false,
        cases: reports,
        aggregate,
        live_aws,
    };
    let bytes = serde_json::to_vec_pretty(&report)?;
    if let Some(path) = args.output {
        fs::write(path, &bytes)?;
    }
    println!("{}", String::from_utf8(bytes)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declared_acceptance_set_covers_required_failure_shapes() {
        let specs = cases();
        assert!(
            specs
                .iter()
                .any(|case| matches!(case.acquisition, AcquisitionKind::FreshFact))
        );
        assert!(specs.iter().any(
            |case| matches!(case.acquisition, AcquisitionKind::Opaque) && case.trusted_verifier
        ));
        assert!(
            specs
                .iter()
                .any(|case| matches!(case.acquisition, AcquisitionKind::Stale))
        );
        assert!(
            specs
                .iter()
                .any(|case| matches!(case.acquisition, AcquisitionKind::WrongScope))
        );
        assert!(
            specs
                .iter()
                .any(|case| matches!(case.acquisition, AcquisitionKind::Irrelevant))
        );
        assert!(
            specs
                .iter()
                .any(|case| matches!(case.acquisition, AcquisitionKind::OperationalFailure))
        );
        assert!(
            specs
                .iter()
                .any(|case| matches!(case.acquisition, AcquisitionKind::BudgetExhaustion))
        );
    }

    #[test]
    fn report_contract_is_not_a_frozen_research_identity() {
        assert_eq!(REPORT_SCHEMA, "reason-external-resolution-acceptance-v1");
        assert!(!REPORT_SCHEMA.contains("stage-c"));
        assert!(!REPORT_SCHEMA.contains("rsd"));
    }
}
