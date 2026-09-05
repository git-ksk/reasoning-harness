use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    process::{Command, Stdio},
    time::Instant,
};

use clap::Parser;
use reasoning_harness_core::{ModelAdapter, ModelOutputFormat, ModelRequest};
use reasoning_harness_providers::MistralAdapter;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v1";
const MAX_ROUNDS: usize = 6;
const MAX_TOOL_CALLS: usize = 10;
const MAX_ELAPSED_MS: u64 = 30_000;
const MAX_MODEL_TOKENS: u64 = 8_000;
const NO_PROGRESS_LIMIT: usize = 2;

#[derive(Debug, Parser)]
#[command(name = "reason-mcp-identity-gate-benchmark")]
struct Args {
    #[arg(long, default_value_t = 3)]
    trials: usize,
    #[arg(long, default_value = "ministral-8b-latest")]
    model: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExpectedOutcome {
    Accept,
    Reject,
    Unknown,
}

#[derive(Debug, Clone, Copy)]
struct CaseSpec {
    id: &'static str,
    task: &'static str,
    initial_query: &'static str,
    property_id: &'static str,
    value_kind: &'static str,
    fact_key: &'static str,
    target_value: &'static str,
    expected: ExpectedOutcome,
}

fn dev_cases() -> [CaseSpec; 7] {
    [
        CaseSpec {
            id: "vienna_country_direct",
            task: "Determine the country of the city Vienna, Austria using external evidence.",
            initial_query: "Vienna",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "vienna.country",
            target_value: "Q40",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "madrid_country_direct",
            task: "Determine the country of the city Madrid, Spain using external evidence.",
            initial_query: "Madrid",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "madrid.country",
            target_value: "Q29",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "vienna_wrong_country_refutation",
            task: "Check the claim that Vienna is in France. Acquire external evidence before deciding whether the claim is supported.",
            initial_query: "Vienna",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "vienna.country",
            target_value: "Q142",
            expected: ExpectedOutcome::Reject,
        },
        CaseSpec {
            id: "oxford_country_context_recovery",
            task: "Determine the country of the city Oxford in England. Use the stated context if more entity evidence is required.",
            initial_query: "Oxford",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "oxford.country",
            target_value: "Q145",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "alexandria_country_context_recovery",
            task: "Determine the country of the city Alexandria in Egypt. Use the stated context if more entity evidence is required.",
            initial_query: "Alexandria",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "alexandria.country",
            target_value: "Q79",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "victoria_ambiguity_abstention",
            task: "Without adding unstated context, determine the country for the place referred to only as 'Victoria'. If the referent is ambiguous, abstain.",
            initial_query: "Victoria",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "victoria.country",
            target_value: "Q16",
            expected: ExpectedOutcome::Unknown,
        },
        CaseSpec {
            id: "georgia_ambiguity_abstention",
            task: "Without adding unstated context, determine the country for the place referred to only as 'Georgia'. If the referent is ambiguous, abstain.",
            initial_query: "Georgia",
            property_id: "P17",
            value_kind: "entity",
            fact_key: "georgia.country",
            target_value: "Q230",
            expected: ExpectedOutcome::Unknown,
        },
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum StopReason {
    ResolvedSupported,
    ResolvedRefuted,
    PlannerStop,
    DuplicateQuery,
    NoProgress,
    MaxRounds,
    MaxToolCalls,
    WallClockBudget,
    ModelTokenBudget,
    PlannerProtocolFailure,
    PlannerProviderFailure,
    ToolTransportFailure,
    ToolProtocolFailure,
}

impl StopReason {
    fn operational(self) -> bool {
        matches!(
            self,
            Self::PlannerProviderFailure | Self::ToolTransportFailure | Self::ToolProtocolFailure
        )
    }

    fn budget(self) -> bool {
        matches!(
            self,
            Self::MaxRounds | Self::MaxToolCalls | Self::WallClockBudget | Self::ModelTokenBudget
        )
    }
}

#[derive(Debug, Deserialize)]
struct PlannerAction {
    action: String,
    #[serde(default)]
    query: Option<String>,
}

#[derive(Debug)]
struct ToolObservation {
    facts: BTreeMap<String, String>,
    observation: String,
    search_state: Value,
}

#[derive(Debug)]
struct ToolFailure {
    kind: StopReason,
}

#[derive(Debug, Serialize)]
struct RoundTrace {
    round: usize,
    query: String,
    observation: Option<String>,
    search_state: Option<Value>,
    new_progress_items: usize,
    planner_action: Option<String>,
    planner_query: Option<String>,
}

#[derive(Debug, Serialize)]
struct CaseReport {
    id: &'static str,
    trial: usize,
    expected: ExpectedOutcome,
    final_outcome: ExpectedOutcome,
    stop_reason: StopReason,
    passed: bool,
    operational_failure: bool,
    rounds: usize,
    tool_calls: usize,
    planner_calls: usize,
    provider_attempts: u32,
    model_tokens: u64,
    elapsed_ms: u64,
    observed_value: Option<String>,
    identity_insufficient_observations: usize,
    identity_supported_observations: usize,
    traces: Vec<RoundTrace>,
}

#[derive(Debug, Serialize)]
struct Aggregate {
    cases: usize,
    trials_per_case: usize,
    samples: usize,
    passed_samples: usize,
    expectation_success_rate: f64,
    false_acceptances: usize,
    false_abstentions: usize,
    operational_unresolved: usize,
    planner_failures: usize,
    tool_failures: usize,
    budget_exhaustions: usize,
    no_progress_stops: usize,
    duplicate_query_stops: usize,
    identity_insufficient_observations: usize,
    samples_with_identity_insufficient: usize,
    identity_insufficient_accept_cases: usize,
    recovered_identity_insufficient_accept_cases: usize,
    identity_insufficient_accept_recovery_rate: f64,
    identity_supported_observations: usize,
    mean_rounds: f64,
    mean_tool_calls: f64,
    mean_planner_calls: f64,
    mean_model_tokens: f64,
    p50_elapsed_ms: u64,
    p95_elapsed_ms: u64,
    max_elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
struct Report {
    schema_version: &'static str,
    layer: &'static str,
    suite: &'static str,
    provider: &'static str,
    model: String,
    prior_frozen_holdout_reused: bool,
    identity_policy: &'static str,
    budgets: BudgetReport,
    cache_policy: &'static str,
    evaluation_policy: &'static str,
    samples: Vec<CaseReport>,
    aggregate: Aggregate,
}

#[derive(Debug, Serialize)]
struct BudgetReport {
    max_rounds: usize,
    max_tool_calls: usize,
    max_elapsed_ms: u64,
    max_model_tokens: u64,
    no_progress_rounds: usize,
}

fn normalize_query(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn qualify_identity(mut observation: ToolObservation) -> ToolObservation {
    if observation
        .search_state
        .get("outcome_kind")
        .and_then(Value::as_str)
        != Some("fact_resolved")
    {
        return observation;
    }

    let resolved_entity = observation
        .search_state
        .get("resolved_entity")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let corroboration_rank = observation
        .search_state
        .get("corroboration_rank")
        .and_then(Value::as_u64);
    let wikipedia_top = observation
        .search_state
        .get("wikipedia_candidates")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .cloned();

    let mut reasons = Vec::<&str>::new();
    if resolved_entity.as_deref().is_none_or(str::is_empty) {
        reasons.push("missing_resolved_entity");
    }

    match wikipedia_top.as_ref() {
        None => reasons.push("missing_wikipedia_top_candidate"),
        Some(top) => {
            if top
                .get("disambiguation")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                reasons.push("wikipedia_top_is_disambiguation");
            }
            let wikipedia_top_entity = top.get("wikibase_item").and_then(Value::as_str);
            if wikipedia_top_entity.is_none_or(str::is_empty) {
                reasons.push("missing_wikipedia_top_entity");
            } else if wikipedia_top_entity != resolved_entity.as_deref() {
                reasons.push("resolved_entity_differs_from_wikipedia_top");
            }
        }
    }

    if corroboration_rank != Some(1) {
        reasons.push("cross_source_agreement_not_rank1");
    }

    let upstream_observation = observation.observation.clone();
    let Some(state) = observation.search_state.as_object_mut() else {
        observation.facts.clear();
        observation.search_state = json!({
            "outcome_kind": "identity_insufficient",
            "upstream_outcome_kind": "fact_resolved",
            "identity_supported": false,
            "identity_reasons": ["invalid_search_state_shape"],
            "suggested_action": "search_with_existing_context_or_stop"
        });
        observation.observation = format!(
            "Harness identity qualification withheld upstream fact: invalid search-state shape; upstream_observation={upstream_observation}"
        );
        return observation;
    };

    if reasons.is_empty() {
        state.insert("identity_supported".into(), Value::Bool(true));
        state.insert(
            "identity_reason".into(),
            Value::String("rank1_cross_source_agreement".into()),
        );
        return observation;
    }

    observation.facts.clear();
    state.insert(
        "upstream_outcome_kind".into(),
        Value::String("fact_resolved".into()),
    );
    state.insert(
        "outcome_kind".into(),
        Value::String("identity_insufficient".into()),
    );
    state.insert("identity_supported".into(), Value::Bool(false));
    state.insert("identity_reasons".into(), json!(reasons));
    state.insert(
        "suggested_action".into(),
        Value::String("search_with_existing_context_or_stop".into()),
    );
    observation.observation = format!(
        "Harness identity qualification withheld upstream fact: reasons={}; use only stated task/observation context to seek stronger entity identity evidence, otherwise stop; upstream_observation={upstream_observation}",
        reasons.join(",")
    );
    observation
}

fn progress_items(state: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for key in [
        "wikidata_candidate_ids",
        "title_retry_candidate_ids",
        "property_values",
        "identity_reasons",
    ] {
        if let Some(values) = state.get(key).and_then(Value::as_array) {
            for value in values {
                if let Some(value) = value.as_str() {
                    out.insert(format!("{key}:{value}"));
                }
            }
        }
    }
    for key in [
        "resolved_entity",
        "wikipedia_top_entity",
        "wikipedia_top_title",
        "suggested_query",
        "outcome_kind",
        "identity_reason",
    ] {
        if let Some(value) = state.get(key).and_then(Value::as_str) {
            out.insert(format!("{key}:{value}"));
        }
    }
    if let Some(values) = state.get("wikipedia_candidates").and_then(Value::as_array) {
        for value in values {
            if let Some(title) = value.get("title").and_then(Value::as_str) {
                out.insert(format!("wikipedia_title:{title}"));
            }
            if let Some(entity) = value.get("wikibase_item").and_then(Value::as_str) {
                out.insert(format!("wikipedia_entity:{entity}"));
            }
        }
    }
    out
}

fn invoke_tool(case: CaseSpec, query: &str) -> Result<ToolObservation, ToolFailure> {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search_fact",
            "arguments": {
                "query": query,
                "language": "en",
                "property_id": case.property_id,
                "value_kind": case.value_kind,
                "fact_key": case.fact_key,
                "allow_title_retry": false
            },
            "_meta": {"protocolVersion": "2026-07-28"}
        }
    });
    let mut child = Command::new("python3")
        .arg("scripts/knowledge_search_fused_mcp.py")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| ToolFailure {
            kind: StopReason::ToolProtocolFailure,
        })?;
    child
        .stdin
        .as_mut()
        .ok_or(ToolFailure {
            kind: StopReason::ToolProtocolFailure,
        })?
        .write_all(format!("{}\n", request).as_bytes())
        .map_err(|_| ToolFailure {
            kind: StopReason::ToolProtocolFailure,
        })?;
    let output = child.wait_with_output().map_err(|_| ToolFailure {
        kind: StopReason::ToolProtocolFailure,
    })?;
    let response: Value = serde_json::from_slice(&output.stdout).map_err(|_| ToolFailure {
        kind: StopReason::ToolProtocolFailure,
    })?;
    if response.get("error").is_some() {
        let operational = response
            .pointer("/error/data/reasoning_harness/operational_kind")
            .and_then(Value::as_str);
        return Err(ToolFailure {
            kind: if operational == Some("transport") {
                StopReason::ToolTransportFailure
            } else {
                StopReason::ToolProtocolFailure
            },
        });
    }
    let harness = response
        .pointer("/result/structuredContent/reasoning_harness")
        .ok_or(ToolFailure {
            kind: StopReason::ToolProtocolFailure,
        })?;
    let observation = harness
        .get("observation")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let facts = harness
        .get("facts")
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
        .unwrap_or_default();
    let search_state = harness
        .get("search_state")
        .cloned()
        .or_else(|| {
            response
                .pointer("/result/structuredContent/search_state")
                .cloned()
        })
        .unwrap_or_else(|| json!({}));
    Ok(qualify_identity(ToolObservation {
        facts,
        observation,
        search_state,
    }))
}

fn planner_prompt(
    case: CaseSpec,
    observation: &ToolObservation,
    tried_queries: &BTreeSet<String>,
    round: usize,
    remaining_calls: usize,
) -> String {
    format!(
        "Task: {}\nProperty to retrieve: {} ({})\nLast qualified tool observation: {}\nCompact search state: {}\nAlready tried normalized queries: {}\nRound: {}. Remaining tool-call budget: {}.\nChoose only the next search query or stop. The Harness may suppress a fact when entity identity is insufficient. If the task itself contains legitimate disambiguating context, use that stated context to reformulate the entity query. Never invent context. Do not decide whether the target fact is true and do not override the identity gate; the Harness owns evidence admission and final correctness.",
        case.task,
        case.property_id,
        case.value_kind,
        observation.observation,
        observation.search_state,
        tried_queries.iter().cloned().collect::<Vec<_>>().join(", "),
        round,
        remaining_calls,
    )
}

async fn plan_next(
    adapter: &dyn ModelAdapter,
    case: CaseSpec,
    observation: &ToolObservation,
    tried_queries: &BTreeSet<String>,
    round: usize,
    remaining_calls: usize,
    seed: u64,
) -> Result<(PlannerAction, u64, u32), StopReason> {
    let response = adapter
        .generate(ModelRequest {
            task: planner_prompt(case, observation, tried_queries, round, remaining_calls),
            system: Some(
                "You are an evidence-search planner inside a bounded verification harness. Return one JSON object only: {\"action\":\"search\",\"query\":\"...\"} or {\"action\":\"stop\",\"query\":null}. You propose actions only. You do not decide truth, identity sufficiency, evidence admission, or final correctness. If identity is insufficient and the task supplies explicit context, use only that context to reformulate. If no legitimate disambiguation exists, stop."
                    .into(),
            ),
            output_format: ModelOutputFormat::JsonObject,
            max_tokens: Some(192),
            random_seed: Some(seed),
            reasoning_preference: None,
        })
        .await
        .map_err(|_| StopReason::PlannerProviderFailure)?;
    let tokens = response.usage.total_tokens.unwrap_or(0);
    let action: PlannerAction = serde_json::from_str(&response.text)
        .map_err(|_| StopReason::PlannerProtocolFailure)?;
    if !matches!(action.action.as_str(), "search" | "stop") {
        return Err(StopReason::PlannerProtocolFailure);
    }
    if action.action == "search"
        && action.query.as_deref().is_none_or(|value| value.trim().is_empty())
    {
        return Err(StopReason::PlannerProtocolFailure);
    }
    Ok((action, tokens, response.provider_attempts))
}

async fn run_case(adapter: &dyn ModelAdapter, case: CaseSpec, trial: usize) -> CaseReport {
    let started = Instant::now();
    let mut query = case.initial_query.to_string();
    let mut tried_queries = BTreeSet::new();
    let mut seen_progress = BTreeSet::new();
    let mut no_progress_rounds = 0usize;
    let mut traces = Vec::new();
    let mut tool_calls = 0usize;
    let mut planner_calls = 0usize;
    let mut provider_attempts = 0u32;
    let mut model_tokens = 0u64;
    let mut observed_value = None;
    let mut stop_reason = StopReason::MaxRounds;
    let mut final_outcome = ExpectedOutcome::Unknown;
    let mut identity_insufficient_observations = 0usize;
    let mut identity_supported_observations = 0usize;

    for round in 1..=MAX_ROUNDS {
        if started.elapsed().as_millis() as u64 >= MAX_ELAPSED_MS {
            stop_reason = StopReason::WallClockBudget;
            break;
        }
        if tool_calls >= MAX_TOOL_CALLS {
            stop_reason = StopReason::MaxToolCalls;
            break;
        }
        let normalized = normalize_query(&query);
        if !tried_queries.insert(normalized) {
            stop_reason = StopReason::DuplicateQuery;
            break;
        }

        tool_calls += 1;
        let observation = match invoke_tool(case, &query) {
            Ok(value) => value,
            Err(error) => {
                stop_reason = error.kind;
                break;
            }
        };
        if observation
            .search_state
            .get("outcome_kind")
            .and_then(Value::as_str)
            == Some("identity_insufficient")
        {
            identity_insufficient_observations += 1;
        }
        if observation
            .search_state
            .get("identity_supported")
            .and_then(Value::as_bool)
            == Some(true)
        {
            identity_supported_observations += 1;
        }

        let new_items = progress_items(&observation.search_state)
            .difference(&seen_progress)
            .cloned()
            .collect::<Vec<_>>();
        if new_items.is_empty() {
            no_progress_rounds += 1;
        } else {
            no_progress_rounds = 0;
            seen_progress.extend(new_items.iter().cloned());
        }

        if let Some(value) = observation.facts.get(case.fact_key).cloned() {
            observed_value = Some(value.clone());
            if value == case.target_value {
                final_outcome = ExpectedOutcome::Accept;
                stop_reason = StopReason::ResolvedSupported;
            } else {
                final_outcome = ExpectedOutcome::Reject;
                stop_reason = StopReason::ResolvedRefuted;
            }
            traces.push(RoundTrace {
                round,
                query: query.clone(),
                observation: Some(observation.observation),
                search_state: Some(observation.search_state),
                new_progress_items: new_items.len(),
                planner_action: None,
                planner_query: None,
            });
            break;
        }

        if no_progress_rounds >= NO_PROGRESS_LIMIT {
            stop_reason = StopReason::NoProgress;
            traces.push(RoundTrace {
                round,
                query: query.clone(),
                observation: Some(observation.observation),
                search_state: Some(observation.search_state),
                new_progress_items: 0,
                planner_action: None,
                planner_query: None,
            });
            break;
        }
        if model_tokens >= MAX_MODEL_TOKENS {
            stop_reason = StopReason::ModelTokenBudget;
            break;
        }

        planner_calls += 1;
        let seed = (trial as u64)
            .saturating_mul(10_000)
            .saturating_add(round as u64)
            .saturating_add(case.id.bytes().map(u64::from).sum::<u64>());
        let planned = plan_next(
            adapter,
            case,
            &observation,
            &tried_queries,
            round,
            MAX_TOOL_CALLS.saturating_sub(tool_calls),
            seed,
        )
        .await;
        let (action, tokens, attempts) = match planned {
            Ok(value) => value,
            Err(reason) => {
                stop_reason = reason;
                break;
            }
        };
        model_tokens = model_tokens.saturating_add(tokens);
        provider_attempts = provider_attempts.saturating_add(attempts);
        let next_query = action.query.clone();
        traces.push(RoundTrace {
            round,
            query: query.clone(),
            observation: Some(observation.observation),
            search_state: Some(observation.search_state),
            new_progress_items: new_items.len(),
            planner_action: Some(action.action.clone()),
            planner_query: next_query.clone(),
        });
        if model_tokens > MAX_MODEL_TOKENS {
            stop_reason = StopReason::ModelTokenBudget;
            break;
        }
        if action.action == "stop" {
            stop_reason = StopReason::PlannerStop;
            break;
        }
        query = next_query.unwrap_or_default();
    }

    let operational_failure = stop_reason.operational();
    let passed = !operational_failure && !stop_reason.budget() && final_outcome == case.expected;
    CaseReport {
        id: case.id,
        trial,
        expected: case.expected,
        final_outcome,
        stop_reason,
        passed,
        operational_failure,
        rounds: tool_calls,
        tool_calls,
        planner_calls,
        provider_attempts,
        model_tokens,
        elapsed_ms: started.elapsed().as_millis() as u64,
        observed_value,
        identity_insufficient_observations,
        identity_supported_observations,
        traces,
    }
}

fn rate(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut values = values.to_vec();
    values.sort_unstable();
    let rank = ((values.len() - 1) * percentile + 99) / 100;
    values[rank.min(values.len() - 1)]
}

fn aggregate(samples: &[CaseReport], cases: usize, trials: usize) -> Aggregate {
    let passed_samples = samples.iter().filter(|sample| sample.passed).count();
    let false_acceptances = samples
        .iter()
        .filter(|sample| {
            sample.expected != ExpectedOutcome::Accept
                && sample.final_outcome == ExpectedOutcome::Accept
        })
        .count();
    let false_abstentions = samples
        .iter()
        .filter(|sample| {
            sample.expected == ExpectedOutcome::Accept
                && sample.final_outcome == ExpectedOutcome::Unknown
                && !sample.operational_failure
        })
        .count();
    let operational_unresolved = samples
        .iter()
        .filter(|sample| sample.operational_failure)
        .count();
    let planner_failures = samples
        .iter()
        .filter(|sample| {
            matches!(
                sample.stop_reason,
                StopReason::PlannerProtocolFailure | StopReason::PlannerProviderFailure
            )
        })
        .count();
    let tool_failures = samples
        .iter()
        .filter(|sample| {
            matches!(
                sample.stop_reason,
                StopReason::ToolTransportFailure | StopReason::ToolProtocolFailure
            )
        })
        .count();
    let budget_exhaustions = samples
        .iter()
        .filter(|sample| sample.stop_reason.budget())
        .count();
    let no_progress_stops = samples
        .iter()
        .filter(|sample| sample.stop_reason == StopReason::NoProgress)
        .count();
    let duplicate_query_stops = samples
        .iter()
        .filter(|sample| sample.stop_reason == StopReason::DuplicateQuery)
        .count();
    let identity_insufficient_observations = samples
        .iter()
        .map(|sample| sample.identity_insufficient_observations)
        .sum();
    let samples_with_identity_insufficient = samples
        .iter()
        .filter(|sample| sample.identity_insufficient_observations > 0)
        .count();
    let identity_insufficient_accept_cases = samples
        .iter()
        .filter(|sample| {
            sample.expected == ExpectedOutcome::Accept
                && sample.identity_insufficient_observations > 0
        })
        .count();
    let recovered_identity_insufficient_accept_cases = samples
        .iter()
        .filter(|sample| {
            sample.expected == ExpectedOutcome::Accept
                && sample.identity_insufficient_observations > 0
                && sample.final_outcome == ExpectedOutcome::Accept
                && sample.passed
        })
        .count();
    let identity_supported_observations = samples
        .iter()
        .map(|sample| sample.identity_supported_observations)
        .sum();
    let latencies = samples
        .iter()
        .map(|sample| sample.elapsed_ms)
        .collect::<Vec<_>>();
    let mean = |f: fn(&CaseReport) -> u64| -> f64 {
        if samples.is_empty() {
            0.0
        } else {
            samples.iter().map(f).sum::<u64>() as f64 / samples.len() as f64
        }
    };

    Aggregate {
        cases,
        trials_per_case: trials,
        samples: samples.len(),
        passed_samples,
        expectation_success_rate: rate(passed_samples, samples.len()),
        false_acceptances,
        false_abstentions,
        operational_unresolved,
        planner_failures,
        tool_failures,
        budget_exhaustions,
        no_progress_stops,
        duplicate_query_stops,
        identity_insufficient_observations,
        samples_with_identity_insufficient,
        identity_insufficient_accept_cases,
        recovered_identity_insufficient_accept_cases,
        identity_insufficient_accept_recovery_rate: rate(
            recovered_identity_insufficient_accept_cases,
            identity_insufficient_accept_cases,
        ),
        identity_supported_observations,
        mean_rounds: mean(|sample| sample.rounds as u64),
        mean_tool_calls: mean(|sample| sample.tool_calls as u64),
        mean_planner_calls: mean(|sample| sample.planner_calls as u64),
        mean_model_tokens: mean(|sample| sample.model_tokens),
        p50_elapsed_ms: percentile(&latencies, 50),
        p95_elapsed_ms: percentile(&latencies, 95),
        max_elapsed_ms: latencies.iter().copied().max().unwrap_or(0),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.trials == 0 {
        return Err("--trials must be at least 1".into());
    }
    let adapter = MistralAdapter::from_env(&args.model)?;
    let cases = dev_cases().to_vec();
    let mut samples = Vec::new();
    for trial in 1..=args.trials {
        for case in cases.iter().copied() {
            samples.push(run_case(&adapter, case, trial).await);
        }
    }
    let aggregate = aggregate(&samples, cases.len(), args.trials);
    let report = Report {
        schema_version: REPORT_SCHEMA,
        layer: "agentic_planner_with_harness_identity_gate",
        suite: "fresh_dev",
        provider: "mistral",
        model: args.model,
        prior_frozen_holdout_reused: false,
        identity_policy: "candidate-set membership is plausibility only; fact admission requires non-disambiguation Wikipedia top identity == resolved entity and cross-source corroboration rank 1",
        budgets: BudgetReport {
            max_rounds: MAX_ROUNDS,
            max_tool_calls: MAX_TOOL_CALLS,
            max_elapsed_ms: MAX_ELAPSED_MS,
            max_model_tokens: MAX_MODEL_TOKENS,
            no_progress_rounds: NO_PROGRESS_LIMIT,
        },
        cache_policy: "live_uncached_public_endpoints; latency is diagnostic, not an SLA claim",
        evaluation_policy: "fresh development cases only; prior #193 holdout is not executed or used for tuning; a new holdout will be frozen only after this candidate stabilizes",
        samples,
        aggregate,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
