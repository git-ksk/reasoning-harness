use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    ApplicabilityScope, CandidateClaim, CanonicalFinalAnswerRenderer, EpistemicState,
    EvidenceAdmissionRejection, EvidenceAuthorityPolicy, EvidenceRequirement, FinalAnswerCandidate,
    FinalAnswerRenderer, FinalClaimMode, FinalizationPolicy, FinalizationStatus,
    GroundedResolutionPolicy, GroundedResolutionRuntime, GroundingPipeline, HarnessInput,
    ModelAdapter, ModelError, ModelOutputFormat, ModelRequest, ModelResponse, ModelUsage,
    Proposition, ReasoningCandidate, ResolutionAdapterError, ResolutionAttempt,
    ResolutionAttemptStatus, ResolutionBudget, ResolutionCost, ResolutionRequest,
    ResolutionResolver, ResolutionResolverContribution, ResolutionResolverOutput,
    ResolutionTerminalStatus, ResolverClass, StandardGroundingPipeline, Verdict,
    build_candidate_json_fallback_request, build_candidate_request, final_answer_candidate_schema,
    finalize_answer,
};
use reasoning_harness_providers::{
    ExternalEvidenceAdmissionConfig, ExternalEvidenceAdmissionPolicy, ExternalEvidenceSourcePolicy,
    GoogleAdapter, MCP_READONLY_RESOLVER_ID, McpReadOnlyResolver, McpReadOnlyResolverConfig,
    MistralAdapter, NvidiaAdapter,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

const REPORT_SCHEMA: &str = "reason-product-external-info-v1";
const COMPARISON_CONTRACT: &str = "shared-candidate-canonical-finalization-v1";

#[derive(Debug, Parser)]
#[command(name = "reason-product-external-info")]
struct Args {
    #[arg(long, default_value = "fixtures/product-external-info-v1")]
    fixtures: PathBuf,
    #[arg(long, value_enum)]
    provider: Option<Provider>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long, default_value_t = 512)]
    max_tokens: u32,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    validate_only: bool,
    /// Execute the frozen live MCP acquisition/admission/verification lane without any model calls.
    #[arg(long, default_value_t = false)]
    acquisition_probe: bool,
    /// Probe the official GitHub MCP server through mcp_readonly_v1 and assert generic output stays opaque.
    #[arg(long, default_value_t = false)]
    github_generic_probe: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum, Serialize)]
#[serde(rename_all = "snake_case")]
enum Provider {
    Mistral,
    Google,
    Nvidia,
}

impl Provider {
    fn name(self) -> &'static str {
        match self {
            Self::Mistral => "mistral",
            Self::Google => "google",
            Self::Nvidia => "nvidia",
        }
    }
}

enum LiveAdapter {
    Mistral(MistralAdapter),
    Google(GoogleAdapter),
    Nvidia(NvidiaAdapter),
}

impl LiveAdapter {
    fn from_env(provider: Provider, model: &str) -> Result<Self, ModelError> {
        match provider {
            Provider::Mistral => MistralAdapter::from_env(model).map(Self::Mistral),
            Provider::Google => GoogleAdapter::from_env(model).map(Self::Google),
            Provider::Nvidia => NvidiaAdapter::from_env(model).map(Self::Nvidia),
        }
    }

    fn adapter(&self) -> &dyn ModelAdapter {
        match self {
            Self::Mistral(adapter) => adapter,
            Self::Google(adapter) => adapter,
            Self::Nvidia(adapter) => adapter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExpectedOutcome {
    Grounded,
    Unknown,
    OperationalFailure,
}

#[derive(Debug, Clone, Deserialize)]
struct TargetSpec {
    identity: String,
    key: String,
    value: String,
}

#[derive(Debug, Clone, Deserialize)]
struct AcquisitionProfile {
    #[allow(dead_code)]
    id: String,
    server_id: String,
    program: String,
    args: Vec<String>,
    tool: String,
    allowed_tools: Vec<String>,
    source_identity: String,
    timeout_ms: u64,
    fixed_arguments: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct SourcePolicySpec {
    authority_class: String,
    max_age_seconds: u64,
    scope: Option<ApplicabilityScope>,
}

#[derive(Debug, Clone, Deserialize)]
struct AdmissionPolicySpec {
    evaluation_time_offset_seconds: i64,
    authority_policy: BTreeMap<String, u16>,
    minimum_authority_class: Option<String>,
    required_scope: Option<ApplicabilityScope>,
    sources: BTreeMap<String, SourcePolicySpec>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExternalInfoCase {
    id: String,
    capability_family: String,
    task: String,
    expected_outcome: ExpectedOutcome,
    target_proposition: TargetSpec,
    acquisition_profiles: Vec<AcquisitionProfile>,
    expected_admission_behavior: String,
    expected_operational_class: Option<String>,
    admission_policy: AdmissionPolicySpec,
}

#[derive(Debug, Clone, Serialize)]
struct CallObservation {
    model: String,
    usage: ModelUsage,
    latency_ms: u128,
    attempts: u32,
}

#[derive(Debug, Clone, Serialize)]
struct RawArm {
    exact_target_exposed: bool,
    exact_target_grounded_claim: bool,
    factual_claims: usize,
    unsupported_grounded_claims: usize,
    abstained: bool,
    call: CallObservation,
}

#[derive(Debug, Clone, Serialize)]
struct HarnessArm {
    initial_verdict: Verdict,
    final_verdict: Verdict,
    finalization_status: FinalizationStatus,
    target_verified: bool,
    exact_target_grounded: bool,
    unsupported_grounded_claims: usize,
    abstained: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ExternalArm {
    #[serde(flatten)]
    harness: HarnessArm,
    terminal_status: ResolutionTerminalStatus,
    acquisition_attempts: usize,
    acquisition_successes: usize,
    verification_successes: usize,
    admitted_evidence: usize,
    admission_rejections: BTreeMap<String, usize>,
    operational_failures: BTreeMap<String, usize>,
    identity_unsafe_admission: usize,
    authority_self_promotion: usize,
    stale_rejections: usize,
    authority_rejections: usize,
    scope_rejections: usize,
    conflict_rejections: usize,
    calls: u64,
    elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
struct CaseReport {
    id: String,
    capability_family: String,
    expected_outcome: ExpectedOutcome,
    target_identity: String,
    raw_model: RawArm,
    harness_without_external_acquisition: HarnessArm,
    harness_with_mcp_external_acquisition: ExternalArm,
}

#[derive(Debug, Default, Clone, Serialize)]
struct SemanticArmAggregate {
    semantic_cases_scored: usize,
    expected_grounded_cases_scored: usize,
    expected_unknown_cases_scored: usize,
    expected_grounded_targets_exposed: usize,
    expected_grounded_target_coverage: f64,
    expected_unknown_preserved: usize,
    expected_unknown_preservation: f64,
    false_target_abstention: usize,
    unsupported_grounded_claims: usize,
    missed_target_insufficiency: usize,
}

#[derive(Debug, Default, Clone, Serialize)]
struct ComparisonAggregate {
    raw_model: SemanticArmAggregate,
    harness_without_external_acquisition: SemanticArmAggregate,
    harness_with_mcp_external_acquisition: SemanticArmAggregate,
}

#[derive(Debug, Default, Clone, Serialize)]
struct Aggregate {
    total_cases: usize,
    semantic_cases: usize,
    semantic_cases_scored: usize,
    semantic_cases_operationally_incomplete: usize,
    operational_cases: usize,
    expected_grounded_cases: usize,
    expected_grounded_cases_scored: usize,
    expected_unknown_cases: usize,
    expected_unknown_cases_scored: usize,
    external_acquisition_attempts: usize,
    external_acquisition_successes: usize,
    verification_successes: usize,
    expected_grounded_targets_exposed: usize,
    expected_grounded_target_coverage: f64,
    expected_unknown_preserved: usize,
    expected_unknown_preservation: f64,
    false_target_abstention: usize,
    unsupported_grounded_claims: usize,
    missed_target_insufficiency: usize,
    identity_unsafe_admission: usize,
    mcp_output_authority_self_promotion: usize,
    stale_rejection: usize,
    authority_rejection: usize,
    scope_rejection: usize,
    conflict_rejection: usize,
    tool_protocol_timeout_operational_failures: usize,
    safety_gate_passed: bool,
}

#[derive(Debug, Serialize)]
struct AcquisitionProbeCase {
    id: String,
    capability_family: String,
    expected_outcome: ExpectedOutcome,
    expected_admission_behavior: String,
    expected_operational_class: Option<String>,
    external: ExternalArm,
}

#[derive(Debug, Serialize)]
struct GithubGenericProbeReport {
    schema_version: &'static str,
    adapter: &'static str,
    server: &'static str,
    tool: &'static str,
    read_only: bool,
    acquisition_success: bool,
    acquired_evidence: usize,
    acquired_fact_candidates: usize,
    generic_output_non_promoting: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    operational_error: Option<String>,
}

#[derive(Debug, Serialize)]
struct AcquisitionProbeReport {
    schema_version: &'static str,
    corpus_identity: &'static str,
    live_network_observation: bool,
    model_used: bool,
    semantic_denominator_excludes_operational_failures: bool,
    cases: Vec<AcquisitionProbeCase>,
    aggregate: Aggregate,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: &'static str,
    comparison_contract: &'static str,
    corpus_identity: &'static str,
    provider: String,
    model: String,
    seed: Option<u64>,
    max_tokens: u32,
    semantic_denominator_excludes_operational_failures: bool,
    cases: Vec<CaseReport>,
    comparison: ComparisonAggregate,
    aggregate: Aggregate,
}

struct ProfileBundleResolver {
    resolvers: Vec<McpReadOnlyResolver>,
}

impl ResolutionResolver for ProfileBundleResolver {
    fn name(&self) -> &'static str {
        MCP_READONLY_RESOLVER_ID
    }

    fn class(&self) -> ResolverClass {
        ResolverClass::EvidenceAcquisition
    }

    fn resolve(
        &self,
        request: &ResolutionRequest,
        attempt_index: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
        let mut evidence = Vec::new();
        let mut cost = ResolutionCost::default();
        for (index, resolver) in self.resolvers.iter().enumerate() {
            let inner_index = attempt_index.saturating_mul(100).saturating_add(index);
            let output = resolver.resolve(request, inner_index)?;
            cost.added_tokens = cost.added_tokens.saturating_add(output.cost.added_tokens);
            cost.elapsed_ms = cost.elapsed_ms.saturating_add(output.cost.elapsed_ms);
            cost.calls = cost.calls.saturating_add(output.cost.calls.max(1));
            cost.cost_microusd = match (cost.cost_microusd, output.cost.cost_microusd) {
                (None, None) => None,
                (left, right) => Some(left.unwrap_or(0).saturating_add(right.unwrap_or(0))),
            };
            match output.contribution {
                ResolutionResolverContribution::AcquiredEvidence {
                    evidence: mut items,
                } => {
                    evidence.append(&mut items);
                }
                ResolutionResolverContribution::NoResult => {}
                _ => {
                    return Err(ResolutionAdapterError {
                        kind: reasoning_harness_core::ResolutionAdapterErrorKind::Protocol,
                        cost,
                    });
                }
            }
        }
        Ok(ResolutionResolverOutput {
            contribution: ResolutionResolverContribution::AcquiredEvidence { evidence },
            cost,
        })
    }
}

fn now_unix() -> Result<i64, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    i64::try_from(seconds).map_err(|_| "current unix time overflow".into())
}

fn load_cases(root: &Path) -> Result<Vec<ExternalInfoCase>, String> {
    let mut paths = fs::read_dir(root)
        .map_err(|error| error.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.len() > 3
                        && name.as_bytes()[0].is_ascii_digit()
                        && name.as_bytes()[1].is_ascii_digit()
                        && name.ends_with(".json")
                })
        })
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            serde_json::from_slice(&fs::read(&path).map_err(|error| error.to_string())?)
                .map_err(|error| format!("{}: {error}", path.display()))
        })
        .collect()
}

fn proposition(case: &ExternalInfoCase) -> Proposition {
    Proposition {
        key: case.target_proposition.key.clone(),
        value: case.target_proposition.value.clone(),
    }
}

fn retrieval_slack_seconds(case: &ExternalInfoCase) -> i64 {
    let max_timeout_ms = case
        .acquisition_profiles
        .iter()
        .map(|profile| profile.timeout_ms)
        .max()
        .unwrap_or(1);
    i64::try_from(max_timeout_ms.div_ceil(1000))
        .unwrap_or(i64::MAX)
        .saturating_add(2)
}

fn harness_input(case: &ExternalInfoCase, wall_time: i64) -> HarnessInput {
    let target = proposition(case);
    let as_of = wall_time
        .saturating_add(retrieval_slack_seconds(case))
        .saturating_add(case.admission_policy.evaluation_time_offset_seconds);
    HarnessInput {
        task: case.task.clone(),
        evidence: vec![],
        hypotheses: vec![target.clone()],
        assumptions: vec![],
        evidence_requirements: vec![EvidenceRequirement {
            proposition: target,
            as_of_unix_seconds: Some(as_of),
            scope: case.admission_policy.required_scope.clone(),
            minimum_authority_class: case.admission_policy.minimum_authority_class.clone(),
        }],
        authority_policy: EvidenceAuthorityPolicy {
            ranks: case.admission_policy.authority_policy.clone(),
        },
    }
}

fn admission(case: &ExternalInfoCase, wall_time: i64) -> ExternalEvidenceAdmissionPolicy {
    // The Harness-owned semantic as-of time lives on the EvidenceRequirement. This separate
    // ceiling only bounds how late a tool result may be retrieved after the case starts.
    let retrieval_ceiling = wall_time.saturating_add(retrieval_slack_seconds(case));
    let sources = case
        .admission_policy
        .sources
        .iter()
        .map(|(source, policy)| {
            (
                source.clone(),
                ExternalEvidenceSourcePolicy {
                    authority_class: policy.authority_class.clone(),
                    max_age_seconds: policy.max_age_seconds,
                    scope: policy.scope.clone(),
                },
            )
        })
        .collect();
    ExternalEvidenceAdmissionPolicy::new(ExternalEvidenceAdmissionConfig {
        resolver_name: MCP_READONLY_RESOLVER_ID,
        evaluation_time_unix_seconds: retrieval_ceiling,
        authority_policy: EvidenceAuthorityPolicy {
            ranks: case.admission_policy.authority_policy.clone(),
        },
        minimum_authority_class: case.admission_policy.minimum_authority_class.clone(),
        required_scope: case.admission_policy.required_scope.clone(),
        sources,
    })
}

fn bundle(case: &ExternalInfoCase) -> ProfileBundleResolver {
    let resolvers = case
        .acquisition_profiles
        .iter()
        .map(|profile| {
            let mut config = McpReadOnlyResolverConfig::with_defaults(
                profile.server_id.clone(),
                PathBuf::from(&profile.program),
                profile.tool.clone(),
                profile.source_identity.clone(),
            );
            config.args = profile.args.clone();
            config.allowed_tools = profile
                .allowed_tools
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>();
            config.fixed_arguments = profile.fixed_arguments.clone();
            config.timeout_ms = profile.timeout_ms;
            McpReadOnlyResolver::new(config)
        })
        .collect();
    ProfileBundleResolver { resolvers }
}

fn resolution_policy(case: &ExternalInfoCase) -> GroundedResolutionPolicy {
    GroundedResolutionPolicy {
        budget: ResolutionBudget {
            max_attempts: 1,
            allowed_resolver_classes: BTreeSet::from([ResolverClass::EvidenceAcquisition]),
            required_authority_class: case.admission_policy.minimum_authority_class.clone(),
            ..ResolutionBudget::default()
        },
        proposition_resolver_class: ResolverClass::EvidenceAcquisition,
        ..GroundedResolutionPolicy::default()
    }
}

fn target_supported_in_artifact(
    artifact: &reasoning_harness_core::ReasoningArtifact,
    target: &Proposition,
) -> bool {
    artifact.claims.iter().any(|claim| {
        claim.proposition.as_ref() == Some(target)
            && matches!(
                claim.state,
                EpistemicState::Known | EpistemicState::Supported
            )
    })
}

fn harness_arm_from_outcome(
    artifact: &reasoning_harness_core::ReasoningArtifact,
    initial_verdict: Verdict,
    final_verdict: Verdict,
    target: &Proposition,
) -> HarnessArm {
    let rendered = CanonicalFinalAnswerRenderer.render(artifact, final_verdict);
    let finalization = finalize_answer(
        artifact,
        final_verdict,
        rendered.clone(),
        FinalizationPolicy::default(),
    );
    let exposed = matches!(
        finalization.status,
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
    );
    let exact_target_grounded = exposed
        && rendered
            .factual_claims
            .iter()
            .any(|claim| claim.mode == FinalClaimMode::Grounded && &claim.proposition == target);
    let unsupported_grounded_claims = if exposed {
        rendered
            .factual_claims
            .iter()
            .filter(|claim| claim.mode == FinalClaimMode::Grounded)
            .filter(|claim| !target_supported_in_artifact(artifact, &claim.proposition))
            .count()
    } else {
        0
    };
    HarnessArm {
        initial_verdict,
        final_verdict,
        finalization_status: finalization.status,
        target_verified: target_supported_in_artifact(artifact, target),
        exact_target_grounded,
        unsupported_grounded_claims,
        abstained: !exposed,
    }
}

fn without_external(
    input: &HarnessInput,
    candidate: &ReasoningCandidate,
    target: &Proposition,
) -> Result<HarnessArm, String> {
    let outcome = StandardGroundingPipeline
        .run(input.clone(), candidate.clone(), &[])
        .map_err(|error| error.to_string())?;
    Ok(harness_arm_from_outcome(
        &outcome.artifact,
        outcome.verdict,
        outcome.verdict,
        target,
    ))
}

fn rejection_key(rejection: EvidenceAdmissionRejection) -> &'static str {
    match rejection {
        EvidenceAdmissionRejection::UntrustedSource => "untrusted_source",
        EvidenceAdmissionRejection::MissingTrustedMetadata => "missing_trusted_metadata",
        EvidenceAdmissionRejection::InvalidEvidence => "invalid_evidence",
        EvidenceAdmissionRejection::MissingObservationTime => "missing_observation_time",
        EvidenceAdmissionRejection::MissingRetrievalTime => "missing_retrieval_time",
        EvidenceAdmissionRejection::MissingScopeMetadata => "missing_scope_metadata",
        EvidenceAdmissionRejection::MissingAuthorityClaim => "missing_authority_claim",
        EvidenceAdmissionRejection::Stale => "stale",
        EvidenceAdmissionRejection::NotYetValid => "not_yet_valid",
        EvidenceAdmissionRejection::ScopeMismatch => "scope_mismatch",
        EvidenceAdmissionRejection::ScopeExpansion => "scope_expansion",
        EvidenceAdmissionRejection::UnknownAuthorityClass => "unknown_authority_class",
        EvidenceAdmissionRejection::InsufficientAuthority => "insufficient_authority",
        EvidenceAdmissionRejection::AuthorityClaimMismatch => "authority_claim_mismatch",
    }
}

fn operational_key(status: ResolutionAttemptStatus) -> Option<&'static str> {
    match status {
        ResolutionAttemptStatus::ProtocolFailure => Some("protocol"),
        ResolutionAttemptStatus::ToolFailed => Some("tool_execution"),
        ResolutionAttemptStatus::TimedOut => Some("timeout"),
        ResolutionAttemptStatus::TransportFailure => Some("transport"),
        ResolutionAttemptStatus::AuthenticationFailure => Some("authentication"),
        ResolutionAttemptStatus::PermissionDenied => Some("permission_denied"),
        ResolutionAttemptStatus::PolicyDenied => Some("policy_denied"),
        ResolutionAttemptStatus::AdapterUnavailable => Some("unavailable"),
        _ => None,
    }
}

fn summarize_attempts(
    attempts: &[ResolutionAttempt],
) -> (
    usize,
    usize,
    usize,
    BTreeMap<String, usize>,
    BTreeMap<String, usize>,
) {
    let acquisition_attempts = attempts.len();
    let acquisition_successes = attempts
        .iter()
        .filter(|attempt| {
            matches!(
                attempt.status,
                ResolutionAttemptStatus::AppliedEvidence
                    | ResolutionAttemptStatus::RejectedUntrustedEvidence
                    | ResolutionAttemptStatus::NoResult
            )
        })
        .count();
    let admitted_evidence = attempts
        .iter()
        .map(|attempt| attempt.admitted_evidence_ids.len())
        .sum();
    let mut rejections = BTreeMap::new();
    let mut operational = BTreeMap::new();
    for attempt in attempts {
        if let Some(rejection) = attempt.admission_rejection {
            *rejections
                .entry(rejection_key(rejection).to_string())
                .or_insert(0) += 1;
        }
        if let Some(kind) = operational_key(attempt.status) {
            *operational.entry(kind.to_string()).or_insert(0) += 1;
        }
    }
    (
        acquisition_attempts,
        acquisition_successes,
        admitted_evidence,
        rejections,
        operational,
    )
}

fn with_external(
    case: &ExternalInfoCase,
    input: &HarnessInput,
    candidate: &ReasoningCandidate,
    target: &Proposition,
    wall_time: i64,
) -> Result<ExternalArm, String> {
    let resolver = bundle(case);
    let resolver_refs: [&dyn ResolutionResolver; 1] = [&resolver];
    let no_verifiers: [&dyn reasoning_harness_core::TrustedResolutionVerifier; 0] = [];
    let admission = admission(case, wall_time);
    let outcome = GroundedResolutionRuntime {
        pipeline: &StandardGroundingPipeline,
        planner: &reasoning_harness_core::DefaultResolutionPlanner,
        evidence_admission: &admission,
        resolvers: &resolver_refs,
        trusted_verifiers: &no_verifiers,
        renderer: &CanonicalFinalAnswerRenderer,
    }
    .run(input.clone(), candidate.clone(), &resolution_policy(case))
    .map_err(|error| error.to_string())?;
    let harness = harness_arm_from_outcome(
        &outcome.final_artifact,
        outcome.initial_verdict,
        outcome.final_verdict,
        target,
    );
    let (
        acquisition_attempts,
        acquisition_successes,
        admitted_evidence,
        admission_rejections,
        operational_failures,
    ) = summarize_attempts(&outcome.attempts);
    let verification_successes = usize::from(harness.target_verified);
    let identity_unsafe_admission = usize::from(
        case.expected_admission_behavior == "identity_no_fact"
            && outcome
                .final_artifact
                .evidence
                .iter()
                .any(|evidence| evidence.facts.contains_key(&target.key)),
    );
    let authority_self_promotion = usize::from(
        matches!(
            case.expected_admission_behavior.as_str(),
            "authority_claim_mismatch" | "insufficient_authority"
        ) && harness.target_verified,
    );
    let stale_rejections = admission_rejections.get("stale").copied().unwrap_or(0);
    let authority_rejections = admission_rejections
        .get("insufficient_authority")
        .copied()
        .unwrap_or(0)
        + admission_rejections
            .get("authority_claim_mismatch")
            .copied()
            .unwrap_or(0);
    let scope_rejections = admission_rejections
        .get("scope_mismatch")
        .copied()
        .unwrap_or(0)
        + admission_rejections
            .get("scope_expansion")
            .copied()
            .unwrap_or(0);
    let conflict_rejections = outcome
        .final_artifact
        .evidence_qualification_findings
        .iter()
        .filter(|finding| finding.reason == reasoning_harness_core::EvidenceQualificationFindingReason::ConflictingQualifiedEvidence)
        .count();
    Ok(ExternalArm {
        harness,
        terminal_status: outcome.terminal_status,
        acquisition_attempts,
        acquisition_successes,
        verification_successes,
        admitted_evidence,
        admission_rejections,
        operational_failures,
        identity_unsafe_admission,
        authority_self_promotion,
        stale_rejections,
        authority_rejections,
        scope_rejections,
        conflict_rejections,
        calls: outcome.usage.calls,
        elapsed_ms: outcome.usage.elapsed_ms,
    })
}

async fn raw_answer(
    adapter: &dyn ModelAdapter,
    model: &str,
    case: &ExternalInfoCase,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(FinalAnswerCandidate, CallObservation), String> {
    let system = "You are a general AI assistant. Answer the task directly without tools or external retrieval. Return a structured final answer. Mark a factual claim grounded only when you believe you have adequate support; otherwise mark it uncertain.";
    let request = ModelRequest {
        system: Some(system.into()),
        task: case.task.clone(),
        output_format: ModelOutputFormat::JsonSchema {
            name: "raw_external_info_final_answer".into(),
            schema: final_answer_candidate_schema(),
        },
        max_tokens: Some(max_tokens),
        random_seed: seed,
        reasoning_preference: None,
    };
    let schema = serde_json::to_string_pretty(&final_answer_candidate_schema())
        .map_err(|error| error.to_string())?;
    let fallback = ModelRequest {
        system: Some(format!(
            "{system} Return exactly one JSON object and no prose conforming to the supplied JSON Schema."
        )),
        task: format!("JSON Schema:\n{schema}\n\nTask:\n{}", case.task),
        output_format: ModelOutputFormat::JsonObject,
        max_tokens: Some(max_tokens),
        random_seed: seed,
        reasoning_preference: None,
    };
    generate_json(adapter, model, request, Some(fallback)).await
}

async fn generate_candidate(
    adapter: &dyn ModelAdapter,
    model: &str,
    input: &HarnessInput,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(ReasoningCandidate, CallObservation), String> {
    let request = build_candidate_request(input, Some(max_tokens), seed)
        .map_err(|error| error.to_string())?;
    let fallback = build_candidate_json_fallback_request(input, Some(max_tokens), seed)
        .map_err(|error| error.to_string())?;
    generate_json(adapter, model, request, Some(fallback)).await
}

async fn generate_json<T: DeserializeOwned>(
    adapter: &dyn ModelAdapter,
    model: &str,
    request: ModelRequest,
    fallback: Option<ModelRequest>,
) -> Result<(T, CallObservation), String> {
    let started = Instant::now();
    let first = adapter
        .generate(request)
        .await
        .map_err(|error| error.to_string())?;
    match parse_one::<T>(&first.text) {
        Ok(value) => Ok((value, call_observation(first, started))),
        Err(first_error) => {
            let Some(fallback) = fallback else {
                return Err(format!("{model}: invalid structured output: {first_error}"));
            };
            let second = adapter
                .generate(fallback)
                .await
                .map_err(|error| error.to_string())?;
            let value = parse_one::<T>(&second.text).map_err(|second_error| format!("{model}: invalid structured output after fallback: first={first_error}; second={second_error}"))?;
            let usage = add_usage(&first.usage, &second.usage);
            Ok((
                value,
                CallObservation {
                    model: second.model,
                    usage,
                    latency_ms: started.elapsed().as_millis(),
                    attempts: first
                        .provider_attempts
                        .saturating_add(second.provider_attempts),
                },
            ))
        }
    }
}

fn parse_one<T: DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    if let Ok(value) = serde_json::from_str(text) {
        return Ok(value);
    }
    let mut stream = serde_json::Deserializer::from_str(text).into_iter::<T>();
    let Some(Ok(value)) = stream.next() else {
        return serde_json::from_str(text);
    };
    Ok(value)
}

fn call_observation(response: ModelResponse, started: Instant) -> CallObservation {
    CallObservation {
        model: response.model,
        usage: response.usage,
        latency_ms: started.elapsed().as_millis(),
        attempts: response.provider_attempts,
    }
}

fn add_usage(left: &ModelUsage, right: &ModelUsage) -> ModelUsage {
    ModelUsage {
        input_tokens: add_opt(left.input_tokens, right.input_tokens),
        output_tokens: add_opt(left.output_tokens, right.output_tokens),
        total_tokens: add_opt(left.total_tokens, right.total_tokens),
    }
}

fn add_opt(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => left.checked_add(right),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn raw_metrics(
    answer: FinalAnswerCandidate,
    call: CallObservation,
    target: &Proposition,
) -> RawArm {
    let exact = answer
        .factual_claims
        .iter()
        .filter(|claim| &claim.proposition == target)
        .collect::<Vec<_>>();
    let unsupported_grounded_claims = answer
        .factual_claims
        .iter()
        .filter(|claim| claim.mode == FinalClaimMode::Grounded)
        .count();
    RawArm {
        exact_target_exposed: !exact.is_empty(),
        exact_target_grounded_claim: exact
            .iter()
            .any(|claim| claim.mode == FinalClaimMode::Grounded),
        factual_claims: answer.factual_claims.len(),
        unsupported_grounded_claims,
        abstained: answer.factual_claims.is_empty(),
        call,
    }
}

fn finalize_semantic_arm(mut arm: SemanticArmAggregate) -> SemanticArmAggregate {
    arm.expected_grounded_target_coverage = if arm.expected_grounded_cases_scored == 0 {
        1.0
    } else {
        arm.expected_grounded_targets_exposed as f64 / arm.expected_grounded_cases_scored as f64
    };
    arm.expected_unknown_preservation = if arm.expected_unknown_cases_scored == 0 {
        1.0
    } else {
        arm.expected_unknown_preserved as f64 / arm.expected_unknown_cases_scored as f64
    };
    arm
}

fn comparison_aggregate(results: &[CaseReport]) -> ComparisonAggregate {
    let mut raw = SemanticArmAggregate::default();
    let mut without_external = SemanticArmAggregate::default();
    let mut with_external = SemanticArmAggregate::default();

    for result in results {
        if result.expected_outcome == ExpectedOutcome::OperationalFailure {
            continue;
        }

        let raw_target = result.raw_model.exact_target_grounded_claim;
        raw.semantic_cases_scored += 1;
        raw.unsupported_grounded_claims += result.raw_model.unsupported_grounded_claims;

        let without_target = result
            .harness_without_external_acquisition
            .exact_target_grounded;
        without_external.semantic_cases_scored += 1;
        without_external.unsupported_grounded_claims += result
            .harness_without_external_acquisition
            .unsupported_grounded_claims;

        let external = &result.harness_with_mcp_external_acquisition;
        let external_scorable = external.operational_failures.is_empty();
        if external_scorable {
            with_external.semantic_cases_scored += 1;
            with_external.unsupported_grounded_claims +=
                external.harness.unsupported_grounded_claims;
        }

        match result.expected_outcome {
            ExpectedOutcome::Grounded => {
                raw.expected_grounded_cases_scored += 1;
                if raw_target {
                    raw.expected_grounded_targets_exposed += 1;
                } else {
                    raw.false_target_abstention += 1;
                }

                without_external.expected_grounded_cases_scored += 1;
                if without_target {
                    without_external.expected_grounded_targets_exposed += 1;
                } else {
                    without_external.false_target_abstention += 1;
                }

                if external_scorable {
                    with_external.expected_grounded_cases_scored += 1;
                    if external.harness.exact_target_grounded {
                        with_external.expected_grounded_targets_exposed += 1;
                    } else {
                        with_external.false_target_abstention += 1;
                    }
                }
            }
            ExpectedOutcome::Unknown => {
                raw.expected_unknown_cases_scored += 1;
                if raw_target {
                    raw.missed_target_insufficiency += 1;
                } else {
                    raw.expected_unknown_preserved += 1;
                }

                without_external.expected_unknown_cases_scored += 1;
                if without_target {
                    without_external.missed_target_insufficiency += 1;
                } else {
                    without_external.expected_unknown_preserved += 1;
                }

                if external_scorable {
                    with_external.expected_unknown_cases_scored += 1;
                    if external.harness.exact_target_grounded {
                        with_external.missed_target_insufficiency += 1;
                    } else {
                        with_external.expected_unknown_preserved += 1;
                    }
                }
            }
            ExpectedOutcome::OperationalFailure => unreachable!(),
        }
    }

    ComparisonAggregate {
        raw_model: finalize_semantic_arm(raw),
        harness_without_external_acquisition: finalize_semantic_arm(without_external),
        harness_with_mcp_external_acquisition: finalize_semantic_arm(with_external),
    }
}

fn validation_comparison(cases: &[ExternalInfoCase]) -> ComparisonAggregate {
    let semantic = cases
        .iter()
        .filter(|case| case.expected_outcome != ExpectedOutcome::OperationalFailure)
        .count();
    let grounded = cases
        .iter()
        .filter(|case| case.expected_outcome == ExpectedOutcome::Grounded)
        .count();
    let unknown = cases
        .iter()
        .filter(|case| case.expected_outcome == ExpectedOutcome::Unknown)
        .count();
    let arm = SemanticArmAggregate {
        semantic_cases_scored: semantic,
        expected_grounded_cases_scored: grounded,
        expected_unknown_cases_scored: unknown,
        ..SemanticArmAggregate::default()
    };
    ComparisonAggregate {
        raw_model: arm.clone(),
        harness_without_external_acquisition: arm.clone(),
        harness_with_mcp_external_acquisition: arm,
    }
}

fn aggregate(results: &[CaseReport]) -> Aggregate {
    let mut aggregate = Aggregate::default();
    aggregate.total_cases = results.len();
    for result in results {
        let external = &result.harness_with_mcp_external_acquisition;
        aggregate.external_acquisition_attempts += external.acquisition_attempts;
        aggregate.external_acquisition_successes += external.acquisition_successes;
        aggregate.verification_successes += external.verification_successes;
        aggregate.unsupported_grounded_claims += external.harness.unsupported_grounded_claims;
        aggregate.identity_unsafe_admission += external.identity_unsafe_admission;
        aggregate.mcp_output_authority_self_promotion += external.authority_self_promotion;
        aggregate.stale_rejection += external.stale_rejections;
        aggregate.authority_rejection += external.authority_rejections;
        aggregate.scope_rejection += external.scope_rejections;
        aggregate.conflict_rejection += external.conflict_rejections;
        aggregate.tool_protocol_timeout_operational_failures +=
            external.operational_failures.values().sum::<usize>();
        match result.expected_outcome {
            ExpectedOutcome::OperationalFailure => {
                aggregate.operational_cases += 1;
            }
            ExpectedOutcome::Grounded => {
                aggregate.semantic_cases += 1;
                aggregate.expected_grounded_cases += 1;
                if !external.operational_failures.is_empty() {
                    aggregate.semantic_cases_operationally_incomplete += 1;
                    continue;
                }
                aggregate.semantic_cases_scored += 1;
                aggregate.expected_grounded_cases_scored += 1;
                if external.harness.exact_target_grounded {
                    aggregate.expected_grounded_targets_exposed += 1;
                } else {
                    aggregate.false_target_abstention += 1;
                }
            }
            ExpectedOutcome::Unknown => {
                aggregate.semantic_cases += 1;
                aggregate.expected_unknown_cases += 1;
                if !external.operational_failures.is_empty() {
                    aggregate.semantic_cases_operationally_incomplete += 1;
                    continue;
                }
                aggregate.semantic_cases_scored += 1;
                aggregate.expected_unknown_cases_scored += 1;
                if external.harness.exact_target_grounded {
                    aggregate.missed_target_insufficiency += 1;
                } else {
                    aggregate.expected_unknown_preserved += 1;
                }
            }
        }
    }
    aggregate.expected_grounded_target_coverage = if aggregate.expected_grounded_cases_scored == 0 {
        1.0
    } else {
        aggregate.expected_grounded_targets_exposed as f64
            / aggregate.expected_grounded_cases_scored as f64
    };
    aggregate.expected_unknown_preservation = if aggregate.expected_unknown_cases_scored == 0 {
        1.0
    } else {
        aggregate.expected_unknown_preserved as f64 / aggregate.expected_unknown_cases_scored as f64
    };
    aggregate.safety_gate_passed = aggregate.unsupported_grounded_claims == 0
        && aggregate.missed_target_insufficiency == 0
        && aggregate.identity_unsafe_admission == 0
        && aggregate.mcp_output_authority_self_promotion == 0;
    aggregate
}

fn run_github_generic_probe() -> GithubGenericProbeReport {
    let mut config = McpReadOnlyResolverConfig::with_defaults(
        "github_official_generic_boundary_v1",
        PathBuf::from("docker"),
        "get_file_contents",
        "github:official-mcp:generic-readme",
    );
    config.args = vec![
        "run".into(),
        "-i".into(),
        "--rm".into(),
        "-e".into(),
        "GITHUB_PERSONAL_ACCESS_TOKEN".into(),
        "-e".into(),
        "GITHUB_READ_ONLY=1".into(),
        "-e".into(),
        "GITHUB_TOOLS=get_file_contents".into(),
        "ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33".into(),
    ];
    config.allowed_tools = BTreeSet::from(["get_file_contents".into()]);
    config.fixed_arguments = BTreeMap::from([
        ("owner".into(), Value::String("git-ksk".into())),
        ("repo".into(), Value::String("reasoning-harness".into())),
        ("path".into(), Value::String("README.md".into())),
        ("ref".into(), Value::String("refs/heads/main".into())),
    ]);
    config.timeout_ms = 30_000;
    let resolver = McpReadOnlyResolver::new(config);
    let request = ResolutionRequest {
        id: "product-external-info:github-generic-boundary".into(),
        reason: reasoning_harness_core::ResolutionReason::MissingSupport,
        target: reasoning_harness_core::ResolutionTarget::Proposition {
            proposition: Proposition {
                key: "external.github.generic.readme".into(),
                value: "present".into(),
            },
        },
        resolver_class: ResolverClass::EvidenceAcquisition,
        budget: reasoning_harness_core::ResolutionRequestBudget::default(),
    };
    match resolver.resolve(&request, 0) {
        Ok(output) => {
            let (acquired_evidence, acquired_fact_candidates) = match output.contribution {
                ResolutionResolverContribution::AcquiredEvidence { evidence } => {
                    let facts = evidence.iter().map(|item| item.facts.len()).sum();
                    (evidence.len(), facts)
                }
                _ => (0, 0),
            };
            GithubGenericProbeReport {
                schema_version: "reason-product-external-info-github-generic-probe-v1",
                adapter: MCP_READONLY_RESOLVER_ID,
                server: "github/github-mcp-server",
                tool: "get_file_contents",
                read_only: true,
                acquisition_success: true,
                acquired_evidence,
                acquired_fact_candidates,
                generic_output_non_promoting: acquired_fact_candidates == 0,
                operational_error: None,
            }
        }
        Err(error) => GithubGenericProbeReport {
            schema_version: "reason-product-external-info-github-generic-probe-v1",
            adapter: MCP_READONLY_RESOLVER_ID,
            server: "github/github-mcp-server",
            tool: "get_file_contents",
            read_only: true,
            acquisition_success: false,
            acquired_evidence: 0,
            acquired_fact_candidates: 0,
            generic_output_non_promoting: true,
            operational_error: Some(format!("{:?}", error.kind)),
        },
    }
}

fn synthetic_target_candidate(target: &Proposition) -> ReasoningCandidate {
    ReasoningCandidate {
        claims: vec![CandidateClaim {
            id: "product-external-info-probe-target".into(),
            statement: format!("{}={}", target.key, target.value),
            proposed_state: EpistemicState::Supported,
            proposition: Some(target.clone()),
            evidence_ids: vec![],
        }],
        inferences: vec![],
    }
}

fn aggregate_probe(results: &[AcquisitionProbeCase]) -> Aggregate {
    let projected = results
        .iter()
        .map(|result| CaseReport {
            id: result.id.clone(),
            capability_family: result.capability_family.clone(),
            expected_outcome: result.expected_outcome,
            target_identity: String::new(),
            raw_model: RawArm {
                exact_target_exposed: false,
                exact_target_grounded_claim: false,
                factual_claims: 0,
                unsupported_grounded_claims: 0,
                abstained: true,
                call: CallObservation {
                    model: "not-used".into(),
                    usage: ModelUsage::default(),
                    latency_ms: 0,
                    attempts: 0,
                },
            },
            harness_without_external_acquisition: HarnessArm {
                initial_verdict: Verdict::Unknown,
                final_verdict: Verdict::Unknown,
                finalization_status: FinalizationStatus::Unresolved,
                target_verified: false,
                exact_target_grounded: false,
                unsupported_grounded_claims: 0,
                abstained: true,
            },
            harness_with_mcp_external_acquisition: result.external.clone(),
        })
        .collect::<Vec<_>>();
    aggregate(&projected)
}

fn run_acquisition_probe(cases: &[ExternalInfoCase]) -> Result<AcquisitionProbeReport, String> {
    let mut reports = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        eprintln!(
            "[product-external-info-acquisition-probe] {}/{} family={} id={}",
            index + 1,
            cases.len(),
            case.capability_family,
            case.id
        );
        let wall_time = now_unix()?;
        let input = harness_input(case, wall_time);
        let target = proposition(case);
        let candidate = synthetic_target_candidate(&target);
        let external = with_external(case, &input, &candidate, &target, wall_time)?;
        reports.push(AcquisitionProbeCase {
            id: case.id.clone(),
            capability_family: case.capability_family.clone(),
            expected_outcome: case.expected_outcome,
            expected_admission_behavior: case.expected_admission_behavior.clone(),
            expected_operational_class: case.expected_operational_class.clone(),
            external,
        });
    }
    let aggregate = aggregate_probe(&reports);
    Ok(AcquisitionProbeReport {
        schema_version: "reason-product-external-info-acquisition-probe-v1",
        corpus_identity: "product-external-info-v1",
        live_network_observation: true,
        model_used: false,
        semantic_denominator_excludes_operational_failures: true,
        cases: reports,
        aggregate,
    })
}

fn validate_cases(cases: &[ExternalInfoCase]) -> Result<(), String> {
    if cases.len() != 21 {
        return Err(format!("expected 21 frozen cases, found {}", cases.len()));
    }
    let mut families = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut target_ids = BTreeSet::new();
    for case in cases {
        if !ids.insert(case.id.clone()) {
            return Err(format!("duplicate case id {}", case.id));
        }
        if !target_ids.insert(case.target_proposition.identity.clone()) {
            return Err(format!(
                "duplicate target identity {}",
                case.target_proposition.identity
            ));
        }
        *families
            .entry(case.capability_family.clone())
            .or_insert(0usize) += 1;
        if case.acquisition_profiles.is_empty() {
            return Err(format!("{} has no acquisition profile", case.id));
        }
        if case.expected_outcome == ExpectedOutcome::OperationalFailure
            && case.expected_operational_class.is_none()
        {
            return Err(format!("{} operational case has no typed class", case.id));
        }
    }
    if families.len() != 7 || families.values().any(|count| *count != 3) {
        return Err(format!("expected exactly 7 families x 3, got {families:?}"));
    }
    Ok(())
}

async fn run(args: &Args) -> Result<Report, String> {
    let cases = load_cases(&args.fixtures)?;
    validate_cases(&cases)?;
    if args.validate_only {
        return Ok(Report {
            schema_version: REPORT_SCHEMA,
            comparison_contract: COMPARISON_CONTRACT,
            corpus_identity: "product-external-info-v1",
            provider: "validation-only".into(),
            model: "validation-only".into(),
            seed: args.seed,
            max_tokens: args.max_tokens,
            semantic_denominator_excludes_operational_failures: true,
            cases: vec![],
            comparison: validation_comparison(&cases),
            aggregate: Aggregate {
                total_cases: 21,
                semantic_cases: 18,
                semantic_cases_scored: 18,
                semantic_cases_operationally_incomplete: 0,
                operational_cases: 3,
                expected_grounded_cases: cases
                    .iter()
                    .filter(|case| case.expected_outcome == ExpectedOutcome::Grounded)
                    .count(),
                expected_grounded_cases_scored: cases
                    .iter()
                    .filter(|case| case.expected_outcome == ExpectedOutcome::Grounded)
                    .count(),
                expected_unknown_cases: cases
                    .iter()
                    .filter(|case| case.expected_outcome == ExpectedOutcome::Unknown)
                    .count(),
                expected_unknown_cases_scored: cases
                    .iter()
                    .filter(|case| case.expected_outcome == ExpectedOutcome::Unknown)
                    .count(),
                safety_gate_passed: true,
                ..Aggregate::default()
            },
        });
    }
    let provider = args
        .provider
        .ok_or_else(|| "--provider is required unless --validate-only".to_string())?;
    let model = args
        .model
        .clone()
        .ok_or_else(|| "--model is required unless --validate-only".to_string())?;
    let live = LiveAdapter::from_env(provider, &model).map_err(|error| error.to_string())?;
    let adapter = live.adapter();
    let mut reports = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        eprintln!(
            "[product-external-info] {}/{} family={} id={}",
            index + 1,
            cases.len(),
            case.capability_family,
            case.id
        );
        let candidate_wall_time = now_unix()?;
        let candidate_input = harness_input(case, candidate_wall_time);
        let target = proposition(case);
        let case_seed = args.seed.and_then(|seed| seed.checked_add(index as u64));
        let (raw_answer, raw_call) =
            raw_answer(adapter, &model, case, args.max_tokens, case_seed).await?;
        let raw_model = raw_metrics(raw_answer, raw_call, &target);
        let (candidate, _candidate_call) = generate_candidate(
            adapter,
            &model,
            &candidate_input,
            args.max_tokens,
            case_seed,
        )
        .await?;

        // Harness owns freshness. Capture the semantic/evidence clock only after untrusted model
        // generation finishes, then share that exact evaluation input across arms B and C. This
        // prevents model latency from consuming the MCP retrieval slack while preserving the same
        // candidate in both Harness arms.
        let evaluation_wall_time = now_unix()?;
        let evaluation_input = harness_input(case, evaluation_wall_time);
        let harness_without_external_acquisition =
            without_external(&evaluation_input, &candidate, &target)?;
        let harness_with_mcp_external_acquisition = with_external(
            case,
            &evaluation_input,
            &candidate,
            &target,
            evaluation_wall_time,
        )?;
        reports.push(CaseReport {
            id: case.id.clone(),
            capability_family: case.capability_family.clone(),
            expected_outcome: case.expected_outcome,
            target_identity: case.target_proposition.identity.clone(),
            raw_model,
            harness_without_external_acquisition,
            harness_with_mcp_external_acquisition,
        });
    }
    let aggregate = aggregate(&reports);
    let report = Report {
        schema_version: REPORT_SCHEMA,
        comparison_contract: COMPARISON_CONTRACT,
        corpus_identity: "product-external-info-v1",
        provider: provider.name().into(),
        model,
        seed: args.seed,
        max_tokens: args.max_tokens,
        semantic_denominator_excludes_operational_failures: true,
        comparison: comparison_aggregate(&reports),
        cases: reports,
        aggregate,
    };
    if let Some(path) = args.output.as_ref() {
        fs::write(
            &path,
            serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
    }
    Ok(report)
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let result = if args.github_generic_probe {
        serde_json::to_value(run_github_generic_probe()).map_err(|error| error.to_string())
    } else if args.acquisition_probe {
        let cases = match load_cases(&args.fixtures).and_then(|cases| {
            validate_cases(&cases)?;
            Ok(cases)
        }) {
            Ok(cases) => cases,
            Err(error) => {
                eprintln!("product external-info evaluation failed: {error}");
                std::process::exit(2);
            }
        };
        run_acquisition_probe(&cases).and_then(|report| {
            if let Some(path) = args.output.as_ref() {
                fs::write(
                    path,
                    serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
                )
                .map_err(|error| error.to_string())?;
            }
            serde_json::to_value(report).map_err(|error| error.to_string())
        })
    } else {
        run(&args)
            .await
            .and_then(|report| serde_json::to_value(report).map_err(|error| error.to_string()))
    };
    match result {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("serialize report")
        ),
        Err(error) => {
            eprintln!("product external-info evaluation failed: {error}");
            std::process::exit(2);
        }
    }
}
