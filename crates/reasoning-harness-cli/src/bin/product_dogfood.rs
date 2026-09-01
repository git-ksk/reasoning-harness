use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    time::Instant,
};

use clap::{Parser, ValueEnum};
use reasoning_harness_core::{
    AcquiredEvidence, CanonicalFinalAnswerRenderer, DefaultResolutionPlanner, Evidence,
    EvidenceAdmissionPolicy, EvidenceAdmissionRejection, EvidenceMetadata, FinalAnswerCandidate,
    FinalClaimMode, FinalizationPolicy, FinalizationStatus, GroundedResolutionPolicy,
    GroundedResolutionRuntime, GroundingPipeline, HarnessInput, ModelAdapter, ModelError,
    ModelOutputFormat, ModelRequest, ModelResponse, ModelUsage, Proposition, ReasoningCandidate,
    ResolutionAdapterError, ResolutionCost, ResolutionRequest, ResolutionResolver,
    ResolutionResolverContribution, ResolutionResolverOutput, ResolutionTarget, ResolverClass,
    StandardGroundingPipeline, Verdict, build_candidate_json_fallback_request,
    build_candidate_request, build_final_answer_json_fallback_request, build_final_answer_request,
    final_answer_candidate_schema, finalize_answer,
};
use reasoning_harness_providers::{GoogleAdapter, MistralAdapter, NvidiaAdapter};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Parser)]
#[command(name = "reason-product-dogfood")]
struct Args {
    #[arg(long, default_value = "fixtures/product-dogfood-v1")]
    fixtures: PathBuf,
    #[arg(long, value_enum)]
    provider: Provider,
    #[arg(long)]
    model: String,
    #[arg(long, default_value_t = 1024)]
    max_tokens: u32,
    #[arg(long)]
    seed: Option<u64>,
    #[arg(long, default_value_t = 3)]
    max_resolution_attempts: usize,
    #[arg(long)]
    output: Option<PathBuf>,
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
    GroundedAfterResolution,
}

impl ExpectedOutcome {
    fn expects_grounded(self) -> bool {
        matches!(self, Self::Grounded | Self::GroundedAfterResolution)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProductDogfoodFixture {
    id: String,
    workload_class: String,
    task: String,
    input: HarnessInput,
    expected_outcome: ExpectedOutcome,
    #[serde(default)]
    resolver_facts: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
struct CallObservation {
    model: String,
    usage: ModelUsage,
    latency_ms: u128,
    attempts: u32,
}

#[derive(Debug, Serialize)]
struct RawArmResult {
    factual_claims: usize,
    grounded_claims: usize,
    unsupported_grounded_claims: usize,
    abstained: bool,
    call: CallObservation,
}

#[derive(Debug, Serialize)]
struct HarnessArmResult {
    initial_verdict: Verdict,
    final_verdict: Verdict,
    finalization_status: FinalizationStatus,
    factual_claims: usize,
    factual_claim_coverage: f64,
    unsupported_exposed_grounded_claims: usize,
    abstained: bool,
    resolution_attempts: usize,
    resolution_succeeded: bool,
    calls: Vec<CallObservation>,
    total_usage: ModelUsage,
    total_latency_ms: u128,
}

#[derive(Debug, Serialize)]
struct CaseResult {
    id: String,
    workload_class: String,
    expected_outcome: ExpectedOutcome,
    raw: RawArmResult,
    harness: HarnessArmResult,
}

#[derive(Debug, Default, Clone, Serialize)]
struct ArmAggregate {
    cases: usize,
    grounded_claims: usize,
    unsupported_grounded_claims: usize,
    unsupported_assertion_rate: f64,
    expected_unknown_cases: usize,
    correct_abstentions: usize,
    missed_insufficiency: usize,
    expected_grounded_cases: usize,
    false_abstentions: usize,
    correct_abstention_rate: f64,
    false_abstention_rate: f64,
}

#[derive(Debug, Default, Clone, Serialize)]
struct HarnessAggregate {
    arm: ArmAggregate,
    mean_final_claim_coverage: f64,
    resolution_attempted_cases: usize,
    resolution_success_cases: usize,
    resolution_success_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
struct OverheadAggregate {
    raw_total_tokens: u64,
    harness_total_tokens: u64,
    token_ratio: Option<f64>,
    raw_latency_ms: u128,
    harness_latency_ms: u128,
    latency_ratio: Option<f64>,
}

#[derive(Debug, Serialize)]
struct ProductDogfoodReport {
    schema_version: &'static str,
    provider: &'static str,
    model: String,
    workload_classes: Vec<String>,
    cases: usize,
    raw: ArmAggregate,
    harness: HarnessAggregate,
    overhead: OverheadAggregate,
    user_comprehension: &'static str,
    results: Vec<CaseResult>,
}

#[derive(Debug)]
struct LocalFactResolver {
    facts: BTreeMap<String, String>,
}

impl ResolutionResolver for LocalFactResolver {
    fn name(&self) -> &'static str {
        "dogfood_local_fact_store"
    }

    fn class(&self) -> ResolverClass {
        ResolverClass::EvidenceAcquisition
    }

    fn resolve(
        &self,
        request: &ResolutionRequest,
        attempt_index: usize,
    ) -> Result<ResolutionResolverOutput, ResolutionAdapterError> {
        let proposition = match &request.target {
            ResolutionTarget::Proposition { proposition } => Some(proposition),
            ResolutionTarget::EvidenceQualification { requirement } => {
                Some(&requirement.proposition)
            }
            _ => None,
        };
        let Some(proposition) = proposition else {
            return Ok(no_result());
        };
        let Some(value) = self.facts.get(&proposition.key) else {
            return Ok(no_result());
        };
        Ok(ResolutionResolverOutput {
            contribution: ResolutionResolverContribution::AcquiredEvidence {
                evidence: vec![AcquiredEvidence {
                    id: format!("dogfood-resolver-{attempt_index}-{}", proposition.key),
                    source: "dogfood-local-fact-store".into(),
                    observation: format!("{}={value}", proposition.key),
                    facts: BTreeMap::from([(proposition.key.clone(), value.clone())]),
                }],
            },
            cost: ResolutionCost::default(),
        })
    }
}

fn no_result() -> ResolutionResolverOutput {
    ResolutionResolverOutput {
        contribution: ResolutionResolverContribution::NoResult,
        cost: ResolutionCost::default(),
    }
}

#[derive(Debug, Clone, Copy)]
struct DogfoodAdmission;

impl EvidenceAdmissionPolicy for DogfoodAdmission {
    fn admit(
        &self,
        resolver_name: &str,
        _request: &ResolutionRequest,
        acquired: &AcquiredEvidence,
    ) -> Result<Evidence, EvidenceAdmissionRejection> {
        if resolver_name != "dogfood_local_fact_store"
            || acquired.source != "dogfood-local-fact-store"
        {
            return Err(EvidenceAdmissionRejection::UntrustedSource);
        }
        Ok(Evidence {
            id: acquired.id.clone(),
            source: acquired.source.clone(),
            observation: acquired.observation.clone(),
            facts: acquired.facts.clone(),
            metadata: EvidenceMetadata {
                temporal: None,
                scope: None,
                provenance_class: Some("dogfood_explicit_resolver".into()),
            },
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    if args.max_tokens == 0 || args.max_resolution_attempts == 0 {
        return Err("max token/resolution limits must be greater than zero".into());
    }
    let fixtures = load_fixtures(&args.fixtures)?;
    if fixtures.is_empty() {
        return Err("product dogfood fixture directory is empty".into());
    }
    let adapter = LiveAdapter::from_env(args.provider, &args.model).map_err(|e| e.to_string())?;
    let mut results = Vec::new();
    for (index, fixture) in fixtures.iter().enumerate() {
        eprintln!(
            "[product-dogfood] {}/{} class={} id={}",
            index + 1,
            fixtures.len(),
            fixture.workload_class,
            fixture.id
        );
        let seed = args.seed.and_then(|seed| seed.checked_add(index as u64));
        results.push(
            evaluate_case(
                fixture,
                adapter.adapter(),
                &args.model,
                args.max_tokens,
                seed,
                args.max_resolution_attempts,
            )
            .await?,
        );
    }
    let report = aggregate(args.provider, &args.model, results);
    let json = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    if let Some(path) = args.output {
        fs::write(&path, format!("{json}\n")).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    println!("{json}");
    Ok(())
}

async fn evaluate_case(
    fixture: &ProductDogfoodFixture,
    adapter: &dyn ModelAdapter,
    model: &str,
    max_tokens: u32,
    seed: Option<u64>,
    max_resolution_attempts: usize,
) -> Result<CaseResult, String> {
    let (raw_answer, raw_call) = raw_answer(adapter, model, fixture, max_tokens, seed).await?;
    let raw_grounded = raw_answer
        .factual_claims
        .iter()
        .filter(|claim| claim.mode == FinalClaimMode::Grounded)
        .collect::<Vec<_>>();
    let raw_unsupported = raw_grounded
        .iter()
        .filter(|claim| !proposition_supported(&fixture.input, &claim.proposition))
        .count();
    let raw = RawArmResult {
        factual_claims: raw_answer.factual_claims.len(),
        grounded_claims: raw_grounded.len(),
        unsupported_grounded_claims: raw_unsupported,
        abstained: raw_grounded.is_empty(),
        call: raw_call,
    };

    let (candidate, candidate_call) =
        generate_candidate(adapter, model, fixture, max_tokens, seed).await?;
    let pipeline = StandardGroundingPipeline;
    let initial = pipeline
        .run(fixture.input.clone(), candidate.clone(), &[])
        .map_err(|e| e.to_string())?;
    let mut calls = vec![candidate_call];
    let mut final_artifact = initial.artifact.clone();
    let mut final_verdict = initial.verdict;
    let mut resolution_attempts = 0usize;
    let mut resolution_succeeded = false;

    if !fixture.resolver_facts.is_empty() && final_verdict != Verdict::Accept {
        let resolver = LocalFactResolver {
            facts: fixture.resolver_facts.clone(),
        };
        let planner = DefaultResolutionPlanner;
        let admission = DogfoodAdmission;
        let renderer = CanonicalFinalAnswerRenderer;
        let resolver_refs: [&dyn ResolutionResolver; 1] = [&resolver];
        let trusted: [&dyn reasoning_harness_core::TrustedResolutionVerifier; 0] = [];
        let runtime = GroundedResolutionRuntime {
            pipeline: &pipeline,
            planner: &planner,
            evidence_admission: &admission,
            resolvers: &resolver_refs,
            trusted_verifiers: &trusted,
            renderer: &renderer,
        };
        let mut policy = GroundedResolutionPolicy::default();
        policy.budget.max_attempts = max_resolution_attempts;
        let resolution = runtime
            .run(fixture.input.clone(), candidate.clone(), &policy)
            .map_err(|e| e.to_string())?;
        resolution_attempts = resolution.attempts.len();
        resolution_succeeded =
            initial.verdict != Verdict::Accept && resolution.final_verdict == Verdict::Accept;
        final_artifact = resolution.final_artifact;
        final_verdict = resolution.final_verdict;
    }

    let (rendered, render_call) = render_answer(
        adapter,
        model,
        &fixture.task,
        &final_artifact,
        final_verdict,
        max_tokens,
        seed,
    )
    .await?;
    calls.push(render_call);
    let finalization = finalize_answer(
        &final_artifact,
        final_verdict,
        rendered.clone(),
        FinalizationPolicy::default(),
    );
    let exposed = matches!(
        finalization.status,
        FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
    );
    let unsupported_exposed_grounded_claims = if exposed {
        rendered
            .factual_claims
            .iter()
            .filter(|claim| claim.mode == FinalClaimMode::Grounded)
            .filter(|claim| !artifact_supports(&final_artifact, &claim.proposition))
            .count()
    } else {
        0
    };
    let total_usage = calls.iter().fold(ModelUsage::default(), |acc, call| {
        add_usage(&acc, &call.usage)
    });
    let total_latency_ms = calls.iter().map(|call| call.latency_ms).sum();
    let harness = HarnessArmResult {
        initial_verdict: initial.verdict,
        final_verdict,
        finalization_status: finalization.status,
        factual_claims: finalization.factual_claims,
        factual_claim_coverage: finalization.factual_claim_coverage,
        unsupported_exposed_grounded_claims,
        abstained: !matches!(
            finalization.status,
            FinalizationStatus::GroundedAnswer | FinalizationStatus::QualifiedPartialAnswer
        ),
        resolution_attempts,
        resolution_succeeded,
        calls,
        total_usage,
        total_latency_ms,
    };
    Ok(CaseResult {
        id: fixture.id.clone(),
        workload_class: fixture.workload_class.clone(),
        expected_outcome: fixture.expected_outcome,
        raw,
        harness,
    })
}

async fn raw_answer(
    adapter: &dyn ModelAdapter,
    model: &str,
    fixture: &ProductDogfoodFixture,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(FinalAnswerCandidate, CallObservation), String> {
    let evidence =
        serde_json::to_string_pretty(&fixture.input.evidence).map_err(|e| e.to_string())?;
    let hypotheses =
        serde_json::to_string_pretty(&fixture.input.hypotheses).map_err(|e| e.to_string())?;
    let task = format!(
        "Task:\n{}\n\nContext evidence:\n{}\n\nRequested hypotheses:\n{}",
        fixture.task, evidence, hypotheses
    );
    let system = "You are a general AI assistant. Answer the task directly from the supplied context. Return a structured final answer. List factual propositions you state in factual_claims and mark them grounded when you believe the context supports them; otherwise uncertain.";
    let request = ModelRequest {
        system: Some(system.into()),
        task: task.clone(),
        output_format: ModelOutputFormat::JsonSchema {
            name: "raw_final_answer".into(),
            schema: final_answer_candidate_schema(),
        },
        max_tokens: Some(max_tokens),
        random_seed: seed,
        reasoning_preference: None,
    };
    let schema = serde_json::to_string_pretty(&final_answer_candidate_schema())
        .map_err(|e| e.to_string())?;
    let fallback = ModelRequest {
        system: Some(format!(
            "{system} Return exactly one JSON object and no prose, conforming to the supplied JSON Schema."
        )),
        task: format!("JSON Schema:\n{schema}\n\n{task}"),
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
    fixture: &ProductDogfoodFixture,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(ReasoningCandidate, CallObservation), String> {
    let request = build_candidate_request(&fixture.input, Some(max_tokens), seed)
        .map_err(|e| e.to_string())?;
    let fallback = build_candidate_json_fallback_request(&fixture.input, Some(max_tokens), seed)
        .map_err(|e| e.to_string())?;
    generate_json(adapter, model, request, Some(fallback)).await
}

async fn render_answer(
    adapter: &dyn ModelAdapter,
    model: &str,
    task: &str,
    artifact: &reasoning_harness_core::ReasoningArtifact,
    verdict: Verdict,
    max_tokens: u32,
    seed: Option<u64>,
) -> Result<(FinalAnswerCandidate, CallObservation), String> {
    let request = build_final_answer_request(task, artifact, verdict, Some(max_tokens), seed)
        .map_err(|e| e.to_string())?;
    let fallback =
        build_final_answer_json_fallback_request(task, artifact, verdict, Some(max_tokens), seed)
            .map_err(|e| e.to_string())?;
    generate_json(adapter, model, request, Some(fallback)).await
}

async fn generate_json<T: DeserializeOwned>(
    adapter: &dyn ModelAdapter,
    model: &str,
    request: ModelRequest,
    fallback: Option<ModelRequest>,
) -> Result<(T, CallObservation), String> {
    let started = Instant::now();
    let first = adapter.generate(request).await.map_err(|e| e.to_string())?;
    match parse_one::<T>(&first.text) {
        Ok(value) => Ok((value, call_observation(first, started, 1))),
        Err(first_error) => {
            let Some(fallback) = fallback else {
                return Err(format!("{model}: invalid structured output: {first_error}"));
            };
            let second = adapter
                .generate(fallback)
                .await
                .map_err(|e| e.to_string())?;
            let value = parse_one::<T>(&second.text).map_err(|second_error| {
                format!(
                    "{model}: invalid structured output after fallback: first={first_error}; second={second_error}"
                )
            })?;
            let usage = add_usage(&first.usage, &second.usage);
            Ok((
                value,
                CallObservation {
                    model: second.model,
                    usage,
                    latency_ms: started.elapsed().as_millis(),
                    attempts: 2,
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

fn call_observation(response: ModelResponse, started: Instant, attempts: u32) -> CallObservation {
    CallObservation {
        model: response.model,
        usage: response.usage,
        latency_ms: started.elapsed().as_millis(),
        attempts,
    }
}

fn proposition_supported(input: &HarnessInput, proposition: &Proposition) -> bool {
    let values = input
        .evidence
        .iter()
        .filter_map(|evidence| evidence.facts.get(&proposition.key))
        .collect::<Vec<_>>();
    !values.is_empty() && values.iter().all(|value| *value == &proposition.value)
}

fn artifact_supports(
    artifact: &reasoning_harness_core::ReasoningArtifact,
    proposition: &Proposition,
) -> bool {
    artifact.claims.iter().any(|claim| {
        claim.proposition.as_ref() == Some(proposition)
            && matches!(
                claim.state,
                reasoning_harness_core::EpistemicState::Known
                    | reasoning_harness_core::EpistemicState::Supported
            )
    })
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

fn load_fixtures(directory: &PathBuf) -> Result<Vec<ProductDogfoodFixture>, String> {
    if !directory.is_dir() {
        return Err(format!(
            "{}: fixture directory does not exist",
            directory.display()
        ));
    }
    let mut paths = fs::read_dir(directory)
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            serde_json::from_slice(&bytes).map_err(|e| format!("{}: {e}", path.display()))
        })
        .collect()
}

fn aggregate(provider: Provider, model: &str, results: Vec<CaseResult>) -> ProductDogfoodReport {
    let classes = results
        .iter()
        .map(|result| result.workload_class.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let raw = aggregate_raw(&results);
    let harness = aggregate_harness(&results);
    let raw_total_tokens = results
        .iter()
        .filter_map(|result| result.raw.call.usage.total_tokens)
        .sum();
    let harness_total_tokens = results
        .iter()
        .filter_map(|result| result.harness.total_usage.total_tokens)
        .sum();
    let raw_latency_ms = results
        .iter()
        .map(|result| result.raw.call.latency_ms)
        .sum();
    let harness_latency_ms = results
        .iter()
        .map(|result| result.harness.total_latency_ms)
        .sum();
    ProductDogfoodReport {
        schema_version: "reason-product-dogfood-v1",
        provider: provider.name(),
        model: model.into(),
        workload_classes: classes,
        cases: results.len(),
        raw,
        harness,
        overhead: OverheadAggregate {
            raw_total_tokens,
            harness_total_tokens,
            token_ratio: ratio(harness_total_tokens as f64, raw_total_tokens as f64),
            raw_latency_ms,
            harness_latency_ms,
            latency_ratio: ratio(harness_latency_ms as f64, raw_latency_ms as f64),
        },
        user_comprehension: "not_automated_manual_review_required",
        results,
    }
}

fn aggregate_raw(results: &[CaseResult]) -> ArmAggregate {
    aggregate_arm(
        results,
        |result| result.raw.grounded_claims,
        |result| result.raw.unsupported_grounded_claims,
        |result| result.raw.abstained,
    )
}

fn aggregate_harness(results: &[CaseResult]) -> HarnessAggregate {
    let arm = aggregate_arm(
        results,
        |result| result.harness.factual_claims,
        |result| result.harness.unsupported_exposed_grounded_claims,
        |result| result.harness.abstained,
    );
    let mean_final_claim_coverage = if results.is_empty() {
        0.0
    } else {
        results
            .iter()
            .map(|result| result.harness.factual_claim_coverage)
            .sum::<f64>()
            / results.len() as f64
    };
    let resolution_attempted_cases = results
        .iter()
        .filter(|result| result.harness.resolution_attempts > 0)
        .count();
    let resolution_success_cases = results
        .iter()
        .filter(|result| result.harness.resolution_succeeded)
        .count();
    HarnessAggregate {
        arm,
        mean_final_claim_coverage,
        resolution_attempted_cases,
        resolution_success_cases,
        resolution_success_rate: rate(resolution_success_cases, resolution_attempted_cases),
    }
}

fn aggregate_arm(
    results: &[CaseResult],
    grounded: impl Fn(&CaseResult) -> usize,
    unsupported: impl Fn(&CaseResult) -> usize,
    abstained: impl Fn(&CaseResult) -> bool,
) -> ArmAggregate {
    let grounded_claims = results.iter().map(&grounded).sum();
    let unsupported_grounded_claims = results.iter().map(&unsupported).sum();
    let expected_unknown_cases = results
        .iter()
        .filter(|result| result.expected_outcome == ExpectedOutcome::Unknown)
        .count();
    let correct_abstentions = results
        .iter()
        .filter(|result| result.expected_outcome == ExpectedOutcome::Unknown && abstained(result))
        .count();
    let missed_insufficiency = expected_unknown_cases.saturating_sub(correct_abstentions);
    let expected_grounded_cases = results
        .iter()
        .filter(|result| result.expected_outcome.expects_grounded())
        .count();
    let false_abstentions = results
        .iter()
        .filter(|result| result.expected_outcome.expects_grounded() && abstained(result))
        .count();
    ArmAggregate {
        cases: results.len(),
        grounded_claims,
        unsupported_grounded_claims,
        unsupported_assertion_rate: rate(unsupported_grounded_claims, grounded_claims),
        expected_unknown_cases,
        correct_abstentions,
        missed_insufficiency,
        expected_grounded_cases,
        false_abstentions,
        correct_abstention_rate: rate(correct_abstentions, expected_unknown_cases),
        false_abstention_rate: rate(false_abstentions, expected_grounded_cases),
    }
}

fn rate(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn ratio(numerator: f64, denominator: f64) -> Option<f64> {
    (denominator > 0.0).then_some(numerator / denominator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn direct_fact_support_requires_observed_value_agreement() {
        let input = HarnessInput {
            task: "test".into(),
            evidence: vec![Evidence {
                id: "e1".into(),
                source: "fixture".into(),
                observation: "status=503".into(),
                facts: BTreeMap::from([("http.status".into(), "503".into())]),
                metadata: Default::default(),
            }],
            hypotheses: vec![],
            assumptions: vec![],
            evidence_requirements: vec![],
            authority_policy: Default::default(),
        };
        assert!(proposition_supported(
            &input,
            &Proposition {
                key: "http.status".into(),
                value: "503".into()
            }
        ));
        assert!(!proposition_supported(
            &input,
            &Proposition {
                key: "http.status".into(),
                value: "200".into()
            }
        ));
    }

    #[test]
    fn aggregate_counts_false_and_correct_abstention_separately() {
        let mk = |id: &str, expected_outcome, raw_abstained, harness_abstained| CaseResult {
            id: id.into(),
            workload_class: "class".into(),
            expected_outcome,
            raw: RawArmResult {
                factual_claims: 0,
                grounded_claims: 0,
                unsupported_grounded_claims: 0,
                abstained: raw_abstained,
                call: CallObservation {
                    model: "m".into(),
                    usage: Default::default(),
                    latency_ms: 1,
                    attempts: 1,
                },
            },
            harness: HarnessArmResult {
                initial_verdict: Verdict::Unknown,
                final_verdict: Verdict::Unknown,
                finalization_status: FinalizationStatus::Unresolved,
                factual_claims: 0,
                factual_claim_coverage: 1.0,
                unsupported_exposed_grounded_claims: 0,
                abstained: harness_abstained,
                resolution_attempts: 0,
                resolution_succeeded: false,
                calls: vec![],
                total_usage: Default::default(),
                total_latency_ms: 1,
            },
        };
        let results = vec![
            mk("u", ExpectedOutcome::Unknown, false, true),
            mk("g", ExpectedOutcome::Grounded, true, false),
        ];
        let raw = aggregate_raw(&results);
        let harness = aggregate_harness(&results);
        assert_eq!(raw.missed_insufficiency, 1);
        assert_eq!(raw.false_abstentions, 1);
        assert_eq!(harness.arm.correct_abstentions, 1);
        assert_eq!(harness.arm.false_abstentions, 0);
    }
}
