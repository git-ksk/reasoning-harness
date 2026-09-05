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

const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v12";
const MAX_ROUNDS: usize = 6;
const MAX_ACTION_REPAIRS_PER_OBSERVATION: usize = 2;
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
    identity_context: Option<&'static str>,
    property_id: &'static str,
    value_kind: &'static str,
    fact_key: &'static str,
    target_value: &'static str,
    expected: ExpectedOutcome,
}

fn dev_cases() -> [CaseSpec; 8] {
    [
        CaseSpec {
            id: "bratislava_country_direct",
            task: "Determine the country of the city Bratislava using external evidence.",
            initial_query: "Bratislava",
            identity_context: None,
            property_id: "P17",
            value_kind: "entity",
            fact_key: "bratislava.country",
            target_value: "Q214",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "lisbon_country_context_agreement",
            task: "Determine the country of the city Lisbon in Portugal. Use the stated identity context as a required identity constraint.",
            initial_query: "Lisbon",
            identity_context: Some("Portugal"),
            property_id: "P17",
            value_kind: "entity",
            fact_key: "lisbon.country",
            target_value: "Q45",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "newcastle_country_context_conflict",
            task: "Determine the country of the city Newcastle in Australia. Use the stated identity context as a required identity constraint.",
            initial_query: "Newcastle",
            identity_context: Some("Australia"),
            property_id: "P17",
            value_kind: "entity",
            fact_key: "newcastle.country",
            target_value: "Q408",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "newport_country_ambiguous_context",
            task: "Determine the country of Newport in Wales. If the bare surface is ambiguous, use only the stated trusted identity context to resolve it.",
            initial_query: "Newport",
            identity_context: Some("Wales"),
            property_id: "P17",
            value_kind: "entity",
            fact_key: "newport.country",
            target_value: "Q145",
            expected: ExpectedOutcome::Accept,
        },
        CaseSpec {
            id: "lincoln_ambiguity_abstention",
            task: "Without adding unstated context, determine the country for the place referred to only as 'Lincoln'. If the referent is ambiguous, abstain.",
            initial_query: "Lincoln",
            identity_context: None,
            property_id: "P17",
            value_kind: "entity",
            fact_key: "lincoln.country",
            target_value: "Q30",
            expected: ExpectedOutcome::Unknown,
        },
        CaseSpec {
            id: "hamilton_ambiguity_abstention",
            task: "Without adding unstated context, determine the country for the place referred to only as 'Hamilton'. If the referent is ambiguous, abstain.",
            initial_query: "Hamilton",
            identity_context: None,
            property_id: "P17",
            value_kind: "entity",
            fact_key: "hamilton.country",
            target_value: "Q16",
            expected: ExpectedOutcome::Unknown,
        },
        CaseSpec {
            id: "hamburg_wrong_country_refutation",
            task: "Check the claim that Hamburg is in France. Acquire external evidence before deciding whether the claim is supported.",
            initial_query: "Hamburg",
            identity_context: None,
            property_id: "P17",
            value_kind: "entity",
            fact_key: "hamburg.country",
            target_value: "Q142",
            expected: ExpectedOutcome::Reject,
        },
        CaseSpec {
            id: "zagreb_country_direct",
            task: "Determine the country of the city Zagreb using external evidence.",
            initial_query: "Zagreb",
            identity_context: None,
            property_id: "P17",
            value_kind: "entity",
            fact_key: "zagreb.country",
            target_value: "Q224",
            expected: ExpectedOutcome::Accept,
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
    PlannerActionRepairExhausted,
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
    external_requests: usize,
}

#[derive(Debug, Serialize)]
struct RoundTrace {
    round: usize,
    query: String,
    observation: Option<String>,
    search_state: Option<Value>,
    new_progress_items: usize,
    new_novelty_items: usize,
    planner_action: Option<String>,
    planner_query: Option<String>,
    followed_suggested_query: bool,
    planner_action_repairs: Vec<Value>,
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
    external_requests: usize,
    planner_calls: usize,
    provider_attempts: u32,
    model_tokens: u64,
    elapsed_ms: u64,
    observed_value: Option<String>,
    identity_insufficient_observations: usize,
    identity_supported_observations: usize,
    invalid_action_observations: usize,
    invalid_actions_blocked_before_external_request: usize,
    follow_suggested_query_actions: usize,
    recovered_after_invalid_action: bool,
    context_unverified_fact_admissions: usize,
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
    semantic_false_decisions: usize,
    context_unverified_fact_admissions: usize,
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
    invalid_action_observations: usize,
    samples_with_invalid_actions: usize,
    invalid_actions_blocked_before_external_request: usize,
    recovered_after_invalid_action: usize,
    invalid_action_recovery_rate: f64,
    follow_suggested_query_actions: usize,
    external_requests: usize,
    mean_rounds: f64,
    mean_tool_calls: f64,
    mean_external_requests: f64,
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
    historical_frozen_holdouts_reused: bool,
    identity_policy: &'static str,
    planner_action_policy: &'static str,
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
    max_action_repairs_per_observation: usize,
}

fn query_terms(value: &str) -> BTreeSet<String> {
    value
        .split(|ch: char| !ch.is_alphanumeric())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn normalize_query(value: &str) -> String {
    query_terms(value).into_iter().collect::<Vec<_>>().join(" ")
}

fn grounded_identity_context_search(case: CaseSpec, query: &str) -> bool {
    let initial = query_terms(case.initial_query);
    let proposed = query_terms(query);
    if initial.is_empty() || !initial.is_subset(&proposed) {
        return false;
    }
    let Some(context) = case.identity_context else {
        return false;
    };
    let allowed_context = query_terms(context);
    if allowed_context.is_empty() || !allowed_context.is_subset(&proposed) {
        return false;
    }
    let allowed = initial.union(&allowed_context).cloned().collect::<BTreeSet<_>>();
    proposed.is_subset(&allowed)
}

fn identity_context_progress_item(case: CaseSpec, query: &str) -> Option<String> {
    grounded_identity_context_search(case, query).then(|| {
        format!(
            "trusted_identity_context:{}",
            normalize_query(case.identity_context.unwrap_or_default())
        )
    })
}

fn state_kind(state: &Value) -> Option<&str> {
    state.get("outcome_kind").and_then(Value::as_str)
}

fn suggested_query(state: &Value) -> Option<String> {
    if state
        .get("suggested_query_origin")
        .and_then(Value::as_str)
        != Some("harness_trusted_identity_context")
    {
        return None;
    }
    state
        .get("suggested_query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn canonical_identity_context_query(case: CaseSpec) -> Option<String> {
    case.identity_context
        .map(|context| format!("{}, {}", case.initial_query, context))
}

fn trusted_identity_context_search_remaining(
    case: CaseSpec,
    tried_queries: &BTreeSet<String>,
) -> bool {
    canonical_identity_context_query(case)
        .is_some_and(|query| !tried_queries.contains(&normalize_query(&query)))
}

fn trusted_identity_context_metadata_compatible(case: CaseSpec, state: &Value) -> bool {
    let Some(context) = case.identity_context else {
        return false;
    };
    let required = query_terms(context);
    if required.is_empty() {
        return false;
    }

    let mut observed = BTreeSet::new();
    for key in ["corroboration_entity_label", "corroboration_entity_description"] {
        if let Some(value) = state.get(key).and_then(Value::as_str) {
            observed.extend(query_terms(value));
        }
    }
    if let Some(title) = state
        .get("wikipedia_candidates")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(|value| value.get("title"))
        .and_then(Value::as_str)
    {
        observed.extend(query_terms(title));
    }
    required.is_subset(&observed)
}


fn planner_action_resolution(
    case: CaseSpec,
    action: &PlannerAction,
    observation: &ToolObservation,
    tried_queries: &BTreeSet<String>,
) -> Result<(Option<String>, bool), &'static str> {
    match action.action.as_str() {
        "search" => {
            if suggested_query(&observation.search_state).is_some() {
                return Err("canonical_suggestion_available_use_follow");
            }
            let query = action
                .query
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or("missing_search_query")?
                .to_string();
            if tried_queries.contains(&normalize_query(&query)) {
                return Err("duplicate_search_query");
            }
            if !grounded_identity_context_search(case, &query) {
                return Err("search_not_grounded_in_trusted_identity_context");
            }
            Ok((Some(query), false))
        }
        "follow_suggested_query" => {
            let query = suggested_query(&observation.search_state)
                .ok_or("missing_suggested_query")?;
            if tried_queries.contains(&normalize_query(&query)) {
                return Err("suggested_query_already_tried");
            }
            Ok((Some(query), true))
        }
        "stop" => Ok((None, false)),
        _ => Err("unknown_action"),
    }
}

fn invalid_planner_action_feedback(
    action: &PlannerAction,
    reason: &str,
    prior: &ToolObservation,
) -> ToolObservation {
    let mut search_state = prior.search_state.clone();
    if let Some(state) = search_state.as_object_mut() {
        state.insert("outcome_kind".into(), Value::String("invalid_planner_action".into()));
        state.insert("invalid_action".into(), Value::String(action.action.clone()));
        state.insert("invalid_query".into(), json!(action.query));
        state.insert("validation_reason".into(), Value::String(reason.into()));
        state.insert("external_requests".into(), json!(0));
    }
    ToolObservation {
        facts: BTreeMap::new(),
        observation: format!(
            "planner action rejected before tool/external request: action={}; reason={}; preserve the current Harness-advertised action set and choose only an available typed action",
            action.action, reason
        ),
        search_state,
    }
}

fn qualify_identity(observation: ToolObservation, case: CaseSpec) -> ToolObservation {
    qualify_identity_for_query(observation, case, case.initial_query)
}

fn qualify_identity_for_query(
    mut observation: ToolObservation,
    case: CaseSpec,
    query: &str,
) -> ToolObservation {
    let context_required = case.identity_context.is_some();
    let context_query_grounded = context_required && grounded_identity_context_search(case, query);
    let context_metadata_compatible =
        context_required && trusted_identity_context_metadata_compatible(case, &observation.search_state);
    let context_verified = context_query_grounded && context_metadata_compatible;
    let outcome_kind = state_kind(&observation.search_state).map(ToOwned::to_owned);

    if outcome_kind.as_deref() != Some("fact_resolved") {
        if let Some(state) = observation.search_state.as_object_mut() {
            state.insert(
                "identity_context_required".into(),
                Value::Bool(context_required),
            );
            state.insert(
                "identity_context_query_grounded".into(),
                Value::Bool(context_query_grounded),
            );
            state.insert(
                "identity_context_metadata_compatible".into(),
                Value::Bool(context_metadata_compatible),
            );
            state.insert(
                "identity_context_verified".into(),
                Value::Bool(context_verified),
            );

            if context_required
                && !context_query_grounded
                && outcome_kind.as_deref() != Some("invalid_query")
            {
                if let Some(query) = canonical_identity_context_query(case) {
                    state.insert("suggested_query".into(), Value::String(query));
                    state.insert(
                        "suggested_query_origin".into(),
                        Value::String("harness_trusted_identity_context".into()),
                    );
                    state.insert(
                        "suggested_action".into(),
                        Value::String("follow_suggested_query".into()),
                    );
                }
                let upstream_observation = observation.observation.clone();
                observation.observation = format!(
                    "Harness requires trusted identity context before identity sufficiency; canonical follow-up is available only from Harness-owned context; upstream_observation={upstream_observation}"
                );
            } else if context_query_grounded {
                state.remove("suggested_query");
                state.remove("suggested_query_origin");
                state.insert("suggested_action".into(), Value::String("stop".into()));
            }
        }
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
    let corroboration_mode = observation
        .search_state
        .get("corroboration_mode")
        .and_then(Value::as_str);
    let direct_wikibase_verified = observation
        .search_state
        .get("direct_wikibase_verified")
        .and_then(Value::as_bool)
        .unwrap_or(false);
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

    let rank1_search_supported = corroboration_rank == Some(1);
    let direct_context_supported = context_required
        && context_query_grounded
        && direct_wikibase_verified
        && corroboration_mode == Some("wikipedia_wikibase_direct");
    if !rank1_search_supported && !direct_context_supported {
        reasons.push("cross_source_identity_evidence_insufficient");
    }
    if context_required && !context_query_grounded {
        reasons.push("trusted_identity_context_not_verified_by_query");
    } else if context_required && !context_metadata_compatible {
        reasons.push("trusted_identity_context_not_supported_by_candidate_metadata");
    }

    let upstream_observation = observation.observation.clone();
    let Some(state) = observation.search_state.as_object_mut() else {
        observation.facts.clear();
        let mut fallback = json!({
            "outcome_kind": "identity_insufficient",
            "upstream_outcome_kind": "fact_resolved",
            "identity_supported": false,
            "identity_reasons": ["invalid_search_state_shape"],
            "identity_context_required": context_required,
            "identity_context_query_grounded": context_query_grounded,
            "identity_context_metadata_compatible": context_metadata_compatible,
            "identity_context_verified": false,
            "suggested_action": "stop"
        });
        if context_required && !context_query_grounded {
            if let Some(query) = canonical_identity_context_query(case) {
                if let Some(state) = fallback.as_object_mut() {
                    state.insert("suggested_query".into(), Value::String(query));
                    state.insert(
                        "suggested_query_origin".into(),
                        Value::String("harness_trusted_identity_context".into()),
                    );
                    state.insert(
                        "suggested_action".into(),
                        Value::String("follow_suggested_query".into()),
                    );
                }
            }
        }
        observation.search_state = fallback;
        observation.observation = format!(
            "Harness identity qualification withheld upstream fact: invalid search-state shape; upstream_observation={upstream_observation}"
        );
        return observation;
    };

    state.insert(
        "identity_context_required".into(),
        Value::Bool(context_required),
    );
    state.insert(
        "identity_context_query_grounded".into(),
        Value::Bool(context_query_grounded),
    );
    state.insert(
        "identity_context_metadata_compatible".into(),
        Value::Bool(context_metadata_compatible),
    );
    state.insert(
        "identity_context_verified".into(),
        Value::Bool(context_verified),
    );

    if reasons.is_empty() {
        state.remove("suggested_query");
        state.remove("suggested_query_origin");
        state.insert("identity_supported".into(), Value::Bool(true));
        state.insert(
            "identity_reason".into(),
            Value::String(if direct_context_supported {
                "direct_wikibase_verification_with_trusted_context_metadata".into()
            } else if context_required {
                "rank1_cross_source_agreement_with_trusted_context_metadata".into()
            } else {
                "rank1_cross_source_agreement".into()
            }),
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
    if context_required && !context_query_grounded {
        if let Some(query) = canonical_identity_context_query(case) {
            state.insert("suggested_query".into(), Value::String(query));
            state.insert(
                "suggested_query_origin".into(),
                Value::String("harness_trusted_identity_context".into()),
            );
            state.insert(
                "suggested_action".into(),
                Value::String("follow_suggested_query".into()),
            );
        }
    } else {
        state.remove("suggested_query");
        state.remove("suggested_query_origin");
        state.insert("suggested_action".into(), Value::String("stop".into()));
    }
    observation.observation = format!(
        "Harness identity qualification withheld upstream fact: reasons={}; trusted context requires both a bounded context query and deterministic compatibility with corroborating candidate metadata; upstream_observation={upstream_observation}",
        reasons.join(",")
    );
    observation
}

fn target_progress_items(case: CaseSpec, query: &str, state: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Some(item) = identity_context_progress_item(case, query) {
        out.insert(item);
    }
    if let Some(value) = suggested_query(state) {
        out.insert(format!("suggested_query:{value}"));
    }

    let identity_insufficient = state_kind(state) == Some("identity_insufficient");
    if !identity_insufficient {
        if let Some(value) = state.get("resolved_entity").and_then(Value::as_str) {
            if !value.trim().is_empty() {
                out.insert(format!("resolved_entity:{value}"));
            }
        }
        if let Some(values) = state.get("property_values").and_then(Value::as_array) {
            for value in values {
                if let Some(value) = value.as_str() {
                    out.insert(format!("property_values:{value}"));
                }
            }
        }
    }
    out
}

fn novelty_items(state: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for key in ["wikidata_candidate_ids", "title_retry_candidate_ids", "identity_reasons"] {
        if let Some(values) = state.get(key).and_then(Value::as_array) {
            for value in values {
                if let Some(value) = value.as_str() {
                    out.insert(format!("{key}:{value}"));
                }
            }
        }
    }
    for key in ["wikipedia_top_entity", "wikipedia_top_title", "outcome_kind", "identity_reason", "validation_reason"] {
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
    let allow_title_retry = case.identity_context.is_some()
        && grounded_identity_context_search(case, query);
    let allow_direct_wikibase_fallback = allow_title_retry;
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
                "allow_title_retry": allow_title_retry,
                "allow_direct_wikibase_fallback": allow_direct_wikibase_fallback
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
            external_requests: 0,
        })?;
    child
        .stdin
        .as_mut()
        .ok_or(ToolFailure {
            kind: StopReason::ToolProtocolFailure,
            external_requests: 0,
        })?
        .write_all(format!("{}\n", request).as_bytes())
        .map_err(|_| ToolFailure {
            kind: StopReason::ToolProtocolFailure,
            external_requests: 0,
        })?;
    let output = child.wait_with_output().map_err(|_| ToolFailure {
        kind: StopReason::ToolProtocolFailure,
        external_requests: 0,
    })?;
    let response: Value = serde_json::from_slice(&output.stdout).map_err(|_| ToolFailure {
        kind: StopReason::ToolProtocolFailure,
        external_requests: 0,
    })?;
    if response.get("error").is_some() {
        let operational = response
            .pointer("/error/data/reasoning_harness/operational_kind")
            .and_then(Value::as_str);
        let external_requests = response
            .pointer("/error/data/reasoning_harness/external_requests")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        return Err(ToolFailure {
            kind: if operational == Some("transport") {
                StopReason::ToolTransportFailure
            } else {
                StopReason::ToolProtocolFailure
            },
            external_requests,
        });
    }
    let harness = response
        .pointer("/result/structuredContent/reasoning_harness")
        .ok_or(ToolFailure {
            kind: StopReason::ToolProtocolFailure,
            external_requests: 0,
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
    Ok(qualify_identity_for_query(
        ToolObservation {
            facts,
            observation,
            search_state,
        },
        case,
        query,
    ))
}

fn planner_prompt(
    case: CaseSpec,
    observation: &ToolObservation,
    tried_queries: &BTreeSet<String>,
    round: usize,
    remaining_calls: usize,
) -> String {
    let available_suggestion = suggested_query(&observation.search_state)
        .filter(|query| !tried_queries.contains(&normalize_query(query)));
    let context_search_available = trusted_identity_context_search_remaining(case, tried_queries);
    let available_actions = if available_suggestion.is_some() {
        "follow_suggested_query or stop (canonical Harness suggestion available; free-form search is unavailable)"
    } else if context_search_available {
        "search or stop (no canonical suggestion is available)"
    } else {
        "stop only (no trusted identity context or validated suggested query is available)"
    };
    let identity_guidance = if state_kind(&observation.search_state) == Some("identity_insufficient") {
        "Entity identity is currently insufficient. If the task explicitly supplies disambiguating context that is not reflected in an already-tried query, search using only that stated context before stopping."
    } else {
        "Do not invent disambiguating context."
    };
    format!(
        "Task: {}\nTrusted identity context (Harness-owned input, distinct from target property/claim): {}\nProperty to retrieve: {} ({})\nLast qualified tool observation: {}\nCompact search state: {}\nAlready tried normalized queries: {}\nRound: {}. Remaining tool-call budget: {}.\nAvailable typed actions now: {}. {} If a validated suggested_query is available, choose follow_suggested_query to execute that exact query; do not regenerate or edit it. If search is available, its query may contain only the entity-surface tokens and all trusted-identity-context tokens, with no connector/descriptive/target-property/candidate-derived words. Never decide whether the target fact is true and never override the identity gate; the Harness owns identity sufficiency, evidence admission, and final correctness.",
        case.task,
        case.identity_context.unwrap_or("<none>"),
        case.property_id,
        case.value_kind,
        observation.observation,
        observation.search_state,
        tried_queries.iter().cloned().collect::<Vec<_>>().join(", "),
        round,
        remaining_calls,
        available_actions,
        identity_guidance,
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
    let follow_available = suggested_query(&observation.search_state)
        .is_some_and(|query| !tried_queries.contains(&normalize_query(&query)));
    let context_search_available = trusted_identity_context_search_remaining(case, tried_queries);
    let action_contract = if follow_available {
        "Return exactly one JSON object: {\"action\":\"follow_suggested_query\",\"query\":null} or {\"action\":\"stop\",\"query\":null}. A canonical Harness suggestion exists, so search is unavailable."
    } else if context_search_available {
        "Return exactly one JSON object: {\"action\":\"search\",\"query\":\"entity surface and trusted identity context only; no other words\"} or {\"action\":\"stop\",\"query\":null}."
    } else {
        "Return exactly {\"action\":\"stop\",\"query\":null}. No trusted identity context or suggested query is available."
    };
    let system = format!(
        "You are an evidence-search planner inside a bounded verification harness. {action_contract} You propose actions only. You do not decide truth, identity sufficiency, evidence admission, or final correctness. Trusted identity context is supplied separately by the Harness; never derive identity context from the target property, candidate list, or your own world knowledge. If no trusted context or validated suggested query is available, stop."
    );
    let response = adapter
        .generate(ModelRequest {
            task: planner_prompt(case, observation, tried_queries, round, remaining_calls),
            system: Some(system),
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

    match action.action.as_str() {
        "search" | "follow_suggested_query" | "stop" => {}
        _ => return Err(StopReason::PlannerProtocolFailure),
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
    let mut external_requests = 0usize;
    let mut planner_calls = 0usize;
    let mut provider_attempts = 0u32;
    let mut model_tokens = 0u64;
    let mut observed_value = None;
    let mut stop_reason = StopReason::MaxRounds;
    let mut final_outcome = ExpectedOutcome::Unknown;
    let mut identity_insufficient_observations = 0usize;
    let mut identity_supported_observations = 0usize;
    let mut invalid_action_observations = 0usize;
    let mut invalid_actions_blocked_before_external_request = 0usize;
    let mut follow_suggested_query_actions = 0usize;
    let mut context_unverified_fact_admissions = 0usize;

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
                external_requests = external_requests.saturating_add(error.external_requests);
                stop_reason = error.kind;
                break;
            }
        };
        external_requests = external_requests.saturating_add(
            observation
                .search_state
                .get("external_requests")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize,
        );
        let outcome_kind = state_kind(&observation.search_state);
        if outcome_kind == Some("identity_insufficient") {
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
        if outcome_kind == Some("invalid_query") {
            invalid_action_observations += 1;
            if observation
                .search_state
                .get("external_requests")
                .and_then(Value::as_u64)
                == Some(0)
            {
                invalid_actions_blocked_before_external_request += 1;
            }
        }

        let new_items = target_progress_items(case, &query, &observation.search_state)
            .difference(&seen_progress)
            .cloned()
            .collect::<Vec<_>>();
        let novelty = novelty_items(&observation.search_state);
        if new_items.is_empty() {
            no_progress_rounds += 1;
        } else {
            no_progress_rounds = 0;
            seen_progress.extend(new_items.iter().cloned());
        }

        if let Some(value) = observation.facts.get(case.fact_key).cloned() {
            if case.identity_context.is_some()
                && observation
                    .search_state
                    .get("identity_context_verified")
                    .and_then(Value::as_bool)
                    != Some(true)
            {
                context_unverified_fact_admissions += 1;
            }
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
                new_novelty_items: novelty.len(),
                planner_action: None,
                planner_query: None,
                followed_suggested_query: false,
                planner_action_repairs: Vec::new(),
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
                new_novelty_items: novelty.len(),
                planner_action: None,
                planner_query: None,
                followed_suggested_query: false,
                planner_action_repairs: Vec::new(),
            });
            break;
        }
        if model_tokens >= MAX_MODEL_TOKENS {
            stop_reason = StopReason::ModelTokenBudget;
            break;
        }

        let mut planner_observation = ToolObservation {
            facts: BTreeMap::new(),
            observation: observation.observation.clone(),
            search_state: observation.search_state.clone(),
        };
        let mut planner_action_repairs = Vec::<Value>::new();
        let mut resolved_action: Option<(PlannerAction, Option<String>, bool)> = None;
        let mut planner_terminal_failure = None;

        for repair_attempt in 0..=MAX_ACTION_REPAIRS_PER_OBSERVATION {
            planner_calls += 1;
            let seed = (trial as u64)
                .saturating_mul(10_000)
                .saturating_add(round as u64)
                .saturating_add((repair_attempt as u64).saturating_mul(100))
                .saturating_add(case.id.bytes().map(u64::from).sum::<u64>());
            let planned = plan_next(
                adapter,
                case,
                &planner_observation,
                &tried_queries,
                round,
                MAX_TOOL_CALLS.saturating_sub(tool_calls),
                seed,
            )
            .await;
            let (action, tokens, attempts) = match planned {
                Ok(value) => value,
                Err(reason) => {
                    planner_terminal_failure = Some(reason);
                    break;
                }
            };
            model_tokens = model_tokens.saturating_add(tokens);
            provider_attempts = provider_attempts.saturating_add(attempts);
            if model_tokens > MAX_MODEL_TOKENS {
                planner_terminal_failure = Some(StopReason::ModelTokenBudget);
                break;
            }

            match planner_action_resolution(case, &action, &planner_observation, &tried_queries) {
                Ok((next_query, followed_suggested_query)) => {
                    resolved_action = Some((action, next_query, followed_suggested_query));
                    break;
                }
                Err(reason) => {
                    invalid_action_observations += 1;
                    invalid_actions_blocked_before_external_request += 1;
                    planner_action_repairs.push(json!({
                        "attempt": repair_attempt + 1,
                        "action": action.action,
                        "query": action.query,
                        "validation_reason": reason,
                        "external_requests": 0
                    }));
                    if repair_attempt == MAX_ACTION_REPAIRS_PER_OBSERVATION {
                        planner_terminal_failure = Some(StopReason::PlannerActionRepairExhausted);
                        break;
                    }
                    planner_observation = invalid_planner_action_feedback(&action, reason, &planner_observation);
                }
            }
        }

        if let Some(reason) = planner_terminal_failure {
            stop_reason = reason;
            traces.push(RoundTrace {
                round,
                query: query.clone(),
                observation: Some(observation.observation),
                search_state: Some(observation.search_state),
                new_progress_items: new_items.len(),
                new_novelty_items: novelty.len(),
                planner_action: None,
                planner_query: None,
                followed_suggested_query: false,
                planner_action_repairs,
            });
            break;
        }

        let Some((action, next_query, followed_suggested_query)) = resolved_action else {
            stop_reason = StopReason::PlannerProtocolFailure;
            traces.push(RoundTrace {
                round,
                query: query.clone(),
                observation: Some(observation.observation),
                search_state: Some(observation.search_state),
                new_progress_items: new_items.len(),
                new_novelty_items: novelty.len(),
                planner_action: None,
                planner_query: None,
                followed_suggested_query: false,
                planner_action_repairs,
            });
            break;
        };

        if followed_suggested_query {
            follow_suggested_query_actions += 1;
        }

        traces.push(RoundTrace {
            round,
            query: query.clone(),
            observation: Some(observation.observation),
            search_state: Some(observation.search_state),
            new_progress_items: new_items.len(),
            new_novelty_items: novelty.len(),
            planner_action: Some(action.action.clone()),
            planner_query: next_query.clone(),
            followed_suggested_query,
            planner_action_repairs,
        });

        if action.action == "stop" {
            stop_reason = StopReason::PlannerStop;
            break;
        }
        let Some(next_query) = next_query else {
            stop_reason = StopReason::PlannerProtocolFailure;
            break;
        };
        query = next_query;
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
        external_requests,
        planner_calls,
        provider_attempts,
        model_tokens,
        elapsed_ms: started.elapsed().as_millis() as u64,
        observed_value,
        identity_insufficient_observations,
        identity_supported_observations,
        invalid_action_observations,
        invalid_actions_blocked_before_external_request,
        follow_suggested_query_actions,
        recovered_after_invalid_action: invalid_action_observations > 0 && passed,
        context_unverified_fact_admissions,
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

fn is_false_acceptance(expected: ExpectedOutcome, final_outcome: ExpectedOutcome) -> bool {
    match expected {
        ExpectedOutcome::Accept => false,
        ExpectedOutcome::Reject => final_outcome == ExpectedOutcome::Accept,
        ExpectedOutcome::Unknown => final_outcome != ExpectedOutcome::Unknown,
    }
}

fn is_semantic_false_decision(
    expected: ExpectedOutcome,
    final_outcome: ExpectedOutcome,
) -> bool {
    final_outcome != ExpectedOutcome::Unknown && final_outcome != expected
}

fn aggregate(samples: &[CaseReport], cases: usize, trials: usize) -> Aggregate {
    let passed_samples = samples.iter().filter(|sample| sample.passed).count();
    let false_acceptances = samples
        .iter()
        .filter(|sample| is_false_acceptance(sample.expected, sample.final_outcome))
        .count();
    let false_abstentions = samples
        .iter()
        .filter(|sample| {
            sample.expected == ExpectedOutcome::Accept
                && sample.final_outcome == ExpectedOutcome::Unknown
                && !sample.operational_failure
        })
        .count();
    let semantic_false_decisions = samples
        .iter()
        .filter(|sample| is_semantic_false_decision(sample.expected, sample.final_outcome))
        .count();
    let context_unverified_fact_admissions = samples
        .iter()
        .map(|sample| sample.context_unverified_fact_admissions)
        .sum();
    let operational_unresolved = samples
        .iter()
        .filter(|sample| sample.operational_failure)
        .count();
    let planner_failures = samples
        .iter()
        .filter(|sample| {
            matches!(
                sample.stop_reason,
                StopReason::PlannerProtocolFailure
                    | StopReason::PlannerActionRepairExhausted
                    | StopReason::PlannerProviderFailure
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
    let invalid_action_observations = samples
        .iter()
        .map(|sample| sample.invalid_action_observations)
        .sum();
    let samples_with_invalid_actions = samples
        .iter()
        .filter(|sample| sample.invalid_action_observations > 0)
        .count();
    let invalid_actions_blocked_before_external_request = samples
        .iter()
        .map(|sample| sample.invalid_actions_blocked_before_external_request)
        .sum();
    let recovered_after_invalid_action = samples
        .iter()
        .filter(|sample| sample.recovered_after_invalid_action)
        .count();
    let follow_suggested_query_actions = samples
        .iter()
        .map(|sample| sample.follow_suggested_query_actions)
        .sum();
    let external_requests = samples
        .iter()
        .map(|sample| sample.external_requests)
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
        semantic_false_decisions,
        context_unverified_fact_admissions,
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
        invalid_action_observations,
        samples_with_invalid_actions,
        invalid_actions_blocked_before_external_request,
        recovered_after_invalid_action,
        invalid_action_recovery_rate: rate(
            recovered_after_invalid_action,
            samples_with_invalid_actions,
        ),
        follow_suggested_query_actions,
        external_requests,
        mean_rounds: mean(|sample| sample.rounds as u64),
        mean_tool_calls: mean(|sample| sample.tool_calls as u64),
        mean_external_requests: mean(|sample| sample.external_requests as u64),
        mean_planner_calls: mean(|sample| sample.planner_calls as u64),
        mean_model_tokens: mean(|sample| sample.model_tokens),
        p50_elapsed_ms: percentile(&latencies, 50),
        p95_elapsed_ms: percentile(&latencies, 95),
        max_elapsed_ms: latencies.iter().copied().max().unwrap_or(0),
    }
}




#[cfg(test)]
mod v6_contract_tests {
    use super::*;

    fn case(context: Option<&'static str>) -> CaseSpec {
        CaseSpec {
            id: "synthetic",
            task: "synthetic",
            initial_query: "Alpha",
            identity_context: context,
            property_id: "P1",
            value_kind: "entity",
            fact_key: "alpha.fact",
            target_value: "Q9",
            expected: ExpectedOutcome::Unknown,
        }
    }

    fn obs(state: Value) -> ToolObservation {
        ToolObservation {
            facts: BTreeMap::new(),
            observation: "test".into(),
            search_state: state,
        }
    }

    fn rank2_fact() -> ToolObservation {
        ToolObservation {
            facts: BTreeMap::from([("alpha.fact".into(), "Q9".into())]),
            observation: "upstream rank2 fact".into(),
            search_state: json!({
                "outcome_kind": "fact_resolved",
                "resolved_entity": "Q1",
                "corroboration_rank": 2,
                "wikipedia_candidates": [{"title":"Alpha","wikibase_item":"Q1","disambiguation":false}],
                "property_values": ["Q9"]
            }),
        }
    }

    #[test]
    fn rank2_with_trusted_context_remains_unadmitted_and_gets_canonical_suggestion() {
        let qualified = qualify_identity(rank2_fact(), case(Some("Region")));
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["outcome_kind"], "identity_insufficient");
        assert_eq!(qualified.search_state["identity_supported"], false);
        assert_eq!(qualified.search_state["suggested_query"], "Alpha, Region");
        assert_eq!(qualified.search_state["suggested_action"], "follow_suggested_query");
        assert!(qualified.search_state["identity_reasons"].as_array().unwrap().iter().any(|v| v == "cross_source_identity_evidence_insufficient"));
    }

    #[test]
    fn rank2_without_trusted_context_remains_unadmitted_and_stops() {
        let qualified = qualify_identity(rank2_fact(), case(None));
        assert!(qualified.facts.is_empty());
        assert!(qualified.search_state.get("suggested_query").is_none());
        assert_eq!(qualified.search_state["suggested_action"], "stop");
    }

    #[test]
    fn canonical_follow_executes_exact_suggestion() {
        let c = case(Some("Region"));
        let observation = obs(json!({"suggested_query":"Alpha, Region","suggested_query_origin":"harness_trusted_identity_context"}));
        let tried = BTreeSet::from([normalize_query("Alpha")]);
        let action = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let (query, followed) = planner_action_resolution(c, &action, &observation, &tried).unwrap();
        assert_eq!(query.as_deref(), Some("Alpha, Region"));
        assert!(followed);
    }

    #[test]
    fn freeform_search_is_unavailable_while_canonical_suggestion_exists() {
        let c = case(Some("Region"));
        let observation = obs(json!({"suggested_query":"Alpha, Region","suggested_query_origin":"harness_trusted_identity_context"}));
        let action = PlannerAction { action: "search".into(), query: Some("Alpha, Region".into()) };
        assert_eq!(
            planner_action_resolution(c, &action, &observation, &BTreeSet::new()),
            Err("canonical_suggestion_available_use_follow")
        );
    }

    #[test]
    fn invalid_feedback_preserves_suggestion_and_blocks_external_request() {
        let prior = obs(json!({
            "outcome_kind":"identity_insufficient",
            "suggested_query":"Alpha, Region",
            "suggested_action":"follow_suggested_query"
        }));
        let action = PlannerAction { action: "search".into(), query: Some("Alpha plus Region".into()) };
        let feedback = invalid_planner_action_feedback(&action, "canonical_suggestion_available_use_follow", &prior);
        assert_eq!(feedback.search_state["suggested_query"], "Alpha, Region");
        assert_eq!(feedback.search_state["suggested_action"], "follow_suggested_query");
        assert_eq!(feedback.search_state["external_requests"], 0);
    }

    #[test]
    fn unavailable_follow_is_typed_invalid_with_zero_external_requests() {
        let c = case(Some("Region"));
        let action = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let prior = obs(json!({"outcome_kind":"identity_insufficient"}));
        let error = planner_action_resolution(c, &action, &prior, &BTreeSet::new())
            .expect_err("follow without suggestion must be unavailable");
        assert_eq!(error, "missing_suggested_query");
        let feedback = invalid_planner_action_feedback(&action, error, &prior);
        assert_eq!(feedback.search_state["external_requests"], 0);
    }

    #[test]
    fn planner_cannot_invent_identity_context() {
        let c = case(None);
        let action = PlannerAction { action: "search".into(), query: Some("Alpha country".into()) };
        assert_eq!(
            planner_action_resolution(c, &action, &obs(json!({})), &BTreeSet::new()),
            Err("search_not_grounded_in_trusted_identity_context")
        );
    }

    #[test]
    fn exact_trusted_context_is_allowed_without_suggestion_but_extra_terms_are_not() {
        let c = case(Some("Region"));
        let valid = PlannerAction { action: "search".into(), query: Some("Alpha (Region)".into()) };
        let (query, followed) = planner_action_resolution(c, &valid, &obs(json!({})), &BTreeSet::new()).unwrap();
        assert_eq!(query.as_deref(), Some("Alpha (Region)"));
        assert!(!followed);

        let invented = PlannerAction { action: "search".into(), query: Some("Alpha plus Region".into()) };
        assert_eq!(
            planner_action_resolution(c, &invented, &obs(json!({})), &BTreeSet::new()),
            Err("search_not_grounded_in_trusted_identity_context")
        );
    }

    #[test]
    fn semantic_duplicate_and_a_b_a_cycle_are_rejected() {
        let c = case(Some("Region"));
        let tried = BTreeSet::from([normalize_query("Alpha"), normalize_query("Alpha, Region")]);
        let action = PlannerAction { action: "search".into(), query: Some("Alpha (Region)".into()) };
        assert_eq!(
            planner_action_resolution(c, &action, &obs(json!({})), &tried),
            Err("duplicate_search_query")
        );
    }

    #[test]
    fn candidate_churn_is_novelty_not_target_progress() {
        let c = case(None);
        let a = json!({"outcome_kind":"search_unresolved","wikidata_candidate_ids":["Q1"]});
        let b = json!({"outcome_kind":"search_unresolved","wikidata_candidate_ids":["Q2"]});
        assert!(target_progress_items(c, "Alpha", &a).is_empty());
        assert!(target_progress_items(c, "Alpha", &b).is_empty());
        assert_ne!(novelty_items(&a), novelty_items(&b));
    }

    #[test]
    fn unknown_expectation_treats_any_fact_decision_as_false_acceptance() {
        assert!(is_false_acceptance(ExpectedOutcome::Unknown, ExpectedOutcome::Accept));
        assert!(is_false_acceptance(ExpectedOutcome::Unknown, ExpectedOutcome::Reject));
        assert!(!is_false_acceptance(ExpectedOutcome::Unknown, ExpectedOutcome::Unknown));
        assert!(is_false_acceptance(ExpectedOutcome::Reject, ExpectedOutcome::Accept));
        assert!(!is_false_acceptance(ExpectedOutcome::Reject, ExpectedOutcome::Reject));
        assert!(!is_false_acceptance(ExpectedOutcome::Accept, ExpectedOutcome::Accept));
    }

    #[test]
    fn bare_rank1_fact_with_trusted_context_is_not_admitted() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = json!(1);
        let qualified = qualify_identity_for_query(upstream, case(Some("Region")), "Alpha");
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["outcome_kind"], "identity_insufficient");
        assert_eq!(qualified.search_state["identity_context_required"], true);
        assert_eq!(qualified.search_state["identity_context_verified"], false);
        assert_eq!(qualified.search_state["suggested_query"], "Alpha, Region");
        assert!(qualified.search_state["identity_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "trusted_identity_context_not_verified_by_query"));
    }

    #[test]
    fn rank1_fact_after_canonical_context_query_is_admitted() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = json!(1);
        upstream.search_state["corroboration_entity_label"] = json!("Alpha");
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity in Region");
        let qualified = qualify_identity_for_query(
            upstream,
            case(Some("Region")),
            "Alpha, Region",
        );
        assert_eq!(qualified.facts.get("alpha.fact").map(String::as_str), Some("Q9"));
        assert_eq!(qualified.search_state["identity_supported"], true);
        assert_eq!(qualified.search_state["identity_context_verified"], true);
        assert_eq!(
            qualified.search_state["identity_reason"],
            "rank1_cross_source_agreement_with_trusted_context_metadata"
        );
    }

    #[test]
    fn ambiguous_bare_surface_with_trusted_context_gets_canonical_followup() {
        let qualified = qualify_identity_for_query(
            obs(json!({"outcome_kind":"ambiguous","external_requests":2})),
            case(Some("Region")),
            "Alpha",
        );
        assert_eq!(qualified.search_state["outcome_kind"], "ambiguous");
        assert_eq!(qualified.search_state["suggested_query"], "Alpha, Region");
        assert_eq!(qualified.search_state["suggested_action"], "follow_suggested_query");
        assert_eq!(qualified.search_state["identity_context_verified"], false);
    }

    #[test]
    fn ambiguity_after_context_query_does_not_loop_the_same_context() {
        let qualified = qualify_identity_for_query(
            obs(json!({"outcome_kind":"ambiguous","external_requests":2,"suggested_action":"stop"})),
            case(Some("Region")),
            "Alpha, Region",
        );
        assert!(qualified.search_state.get("suggested_query").is_none());
        let tried = BTreeSet::from([
            normalize_query("Alpha"),
            normalize_query("Alpha, Region"),
        ]);
        assert!(!trusted_identity_context_search_remaining(case(Some("Region")), &tried));
    }

    #[test]
    fn semantic_false_decision_is_symmetric_for_terminal_wrong_answers() {
        assert!(is_semantic_false_decision(
            ExpectedOutcome::Accept,
            ExpectedOutcome::Reject
        ));
        assert!(is_semantic_false_decision(
            ExpectedOutcome::Reject,
            ExpectedOutcome::Accept
        ));
        assert!(is_semantic_false_decision(
            ExpectedOutcome::Unknown,
            ExpectedOutcome::Accept
        ));
        assert!(is_semantic_false_decision(
            ExpectedOutcome::Unknown,
            ExpectedOutcome::Reject
        ));
        assert!(!is_semantic_false_decision(
            ExpectedOutcome::Accept,
            ExpectedOutcome::Unknown
        ));
        assert!(!is_semantic_false_decision(
            ExpectedOutcome::Accept,
            ExpectedOutcome::Accept
        ));
    }

    #[test]
    fn trusted_context_direct_wikibase_verification_can_admit_without_search_rank() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = Value::Null;
        upstream.search_state["corroboration_mode"] = json!("wikipedia_wikibase_direct");
        upstream.search_state["direct_wikibase_verified"] = json!(true);
        upstream.search_state["corroboration_entity_label"] = json!("Alpha");
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity in Region");
        let qualified = qualify_identity_for_query(
            upstream,
            case(Some("Region")),
            "Alpha, Region",
        );
        assert_eq!(qualified.facts.get("alpha.fact").map(String::as_str), Some("Q9"));
        assert_eq!(qualified.search_state["identity_supported"], true);
        assert_eq!(qualified.search_state["identity_context_verified"], true);
        assert_eq!(
            qualified.search_state["identity_reason"],
            "direct_wikibase_verification_with_trusted_context_metadata"
        );
    }

    #[test]
    fn direct_wikibase_verification_never_relaxes_bare_surface_rank_rule() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = Value::Null;
        upstream.search_state["corroboration_mode"] = json!("wikipedia_wikibase_direct");
        upstream.search_state["direct_wikibase_verified"] = json!(true);
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity in Region");
        let qualified = qualify_identity_for_query(upstream, case(None), "Alpha");
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["outcome_kind"], "identity_insufficient");
        assert!(qualified.search_state["identity_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "cross_source_identity_evidence_insufficient"));
    }

    #[test]
    fn direct_wikibase_verification_without_context_metadata_remains_unadmitted() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = Value::Null;
        upstream.search_state["corroboration_mode"] = json!("wikipedia_wikibase_direct");
        upstream.search_state["direct_wikibase_verified"] = json!(true);
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity elsewhere");
        let qualified = qualify_identity_for_query(
            upstream,
            case(Some("Region")),
            "Alpha, Region",
        );
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["identity_context_metadata_compatible"], false);
    }

    #[test]
    fn context_query_without_compatible_candidate_metadata_is_not_admitted() {
        let mut upstream = rank2_fact();
        upstream.search_state["corroboration_rank"] = json!(1);
        upstream.search_state["corroboration_entity_description"] = json!("synthetic entity elsewhere");
        let qualified = qualify_identity_for_query(
            upstream,
            case(Some("Region")),
            "Alpha, Region",
        );
        assert!(qualified.facts.is_empty());
        assert_eq!(qualified.search_state["identity_context_query_grounded"], true);
        assert_eq!(qualified.search_state["identity_context_metadata_compatible"], false);
        assert_eq!(qualified.search_state["identity_context_verified"], false);
        assert!(qualified.search_state.get("suggested_query").is_none());
        assert_eq!(qualified.search_state["suggested_action"], "stop");
    }

    #[test]
    fn adapter_candidate_suggestion_is_not_a_planner_action() {
        let state = json!({
            "outcome_kind":"entity_disagreement",
            "suggested_query":"Alpha, Candidate Region"
        });
        assert!(suggested_query(&state).is_none());
        let action = PlannerAction { action: "follow_suggested_query".into(), query: None };
        let error = planner_action_resolution(
            case(Some("Region")),
            &action,
            &obs(state),
            &BTreeSet::from([normalize_query("Alpha, Region")]),
        ).unwrap_err();
        assert_eq!(error, "missing_suggested_query");
    }

    #[test]
    fn repair_limit_exhaustion_remains_safe_unknown() {
        let c = case(None);
        let tried = BTreeSet::new();
        let mut observation = obs(json!({"outcome_kind":"ambiguous"}));
        let invalid = PlannerAction { action: "search".into(), query: Some("Alpha country".into()) };
        let mut exhausted = false;
        for attempt in 0..=MAX_ACTION_REPAIRS_PER_OBSERVATION {
            let reason = planner_action_resolution(c, &invalid, &observation, &tried).unwrap_err();
            if attempt == MAX_ACTION_REPAIRS_PER_OBSERVATION {
                exhausted = true;
                break;
            }
            observation = invalid_planner_action_feedback(&invalid, reason, &observation);
        }
        assert!(exhausted);
        assert!(observation.facts.is_empty());
        assert!(!StopReason::PlannerActionRepairExhausted.operational());
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
        historical_frozen_holdouts_reused: false,
        identity_policy: "candidate-set membership is plausibility only; no-context fact admission keeps the rank1 cross-source gate; trusted-context fact admission requires the Harness-bounded context query plus deterministic context-token compatibility, and may use either rank1 search corroboration or a bounded direct fetch of the exact non-disambiguation Wikipedia Wikibase QID; direct-QID evidence never relaxes bare-surface admission",
        planner_action_policy: "planner is untrusted; only suggestions marked as Harness-owned trusted-context actions are executable; adapter/candidate suggested queries are observations only; the planner may follow the exact Harness suggestion or stop and never decides identity sufficiency",
        budgets: BudgetReport {
            max_rounds: MAX_ROUNDS,
            max_tool_calls: MAX_TOOL_CALLS,
            max_elapsed_ms: MAX_ELAPSED_MS,
            max_model_tokens: MAX_MODEL_TOKENS,
            no_progress_rounds: NO_PROGRESS_LIMIT,
            max_action_repairs_per_observation: MAX_ACTION_REPAIRS_PER_OBSERVATION,
        },
        cache_policy: "live_uncached_public_endpoints; latency is diagnostic, not an SLA claim",
        evaluation_policy: "fresh Issue #196 development split only; historical #193 and #195 frozen holdouts are not executed, replayed, or used for tuning; terminal wrong answers are counted symmetrically as semantic_false_decisions; context-unverified fact admission is a hard safety violation",
        samples,
        aggregate,
    };
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
