#!/usr/bin/env python3
"""Apply the v3 planner-action boundary candidate to the fresh-dev benchmark.

This is deliberately an exact-match transformation while the research candidate
is still being tuned. Every replacement asserts the previous v2 source shape so
a drifted benchmark fails closed instead of silently receiving a partial patch.
"""

from __future__ import annotations

from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def replace_count(text: str, old: str, new: str, expected: int, label: str) -> str:
    count = text.count(old)
    if count != expected:
        raise SystemExit(f"{label}: expected {expected} matches, found {count}")
    return text.replace(old, new)


def main() -> int:
    text = PATH.read_text(encoding="utf-8")

    text = replace_once(
        text,
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v2";\n'
        'const MAX_ROUNDS: usize = 6;\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v3";\n'
        'const MAX_ROUNDS: usize = 6;\n'
        'const MAX_ACTION_REPAIRS_PER_OBSERVATION: usize = 2;\n',
        "schema-and-action-repair-budget",
    )

    text = replace_once(
        text,
        '    PlannerProtocolFailure,\n'
        '    PlannerProviderFailure,\n',
        '    PlannerProtocolFailure,\n'
        '    PlannerActionRepairExhausted,\n'
        '    PlannerProviderFailure,\n',
        "planner-action-repair-stop-reason",
    )

    text = replace_once(
        text,
        '    followed_suggested_query: bool,\n'
        '}\n\n#[derive(Debug, Serialize)]\nstruct CaseReport {',
        '    followed_suggested_query: bool,\n'
        '    planner_action_repairs: Vec<Value>,\n'
        '}\n\n#[derive(Debug, Serialize)]\nstruct CaseReport {',
        "trace-action-repairs",
    )

    text = replace_once(
        text,
        '    max_model_tokens: u64,\n'
        '    no_progress_rounds: usize,\n'
        '}\n',
        '    max_model_tokens: u64,\n'
        '    no_progress_rounds: usize,\n'
        '    max_action_repairs_per_observation: usize,\n'
        '}\n',
        "budget-report-action-repairs",
    )

    anchor = '''fn suggested_query(state: &Value) -> Option<String> {\n    state\n        .get("suggested_query")\n        .and_then(Value::as_str)\n        .map(str::trim)\n        .filter(|value| !value.is_empty())\n        .map(ToOwned::to_owned)\n}\n'''
    helpers = anchor + r'''

fn planner_action_resolution(
    action: &PlannerAction,
    observation: &ToolObservation,
    tried_queries: &BTreeSet<String>,
) -> Result<(Option<String>, bool), &'static str> {
    match action.action.as_str() {
        "search" => {
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

fn invalid_planner_action_feedback(action: &PlannerAction, reason: &str) -> ToolObservation {
    ToolObservation {
        facts: BTreeMap::new(),
        observation: format!(
            "planner action rejected before tool/external request: action={}; reason={}; choose only an available typed action; if the task supplies explicit disambiguating context that has not been searched, use it with search; otherwise stop",
            action.action, reason
        ),
        search_state: json!({
            "outcome_kind": "invalid_planner_action",
            "invalid_action": action.action,
            "invalid_query": action.query,
            "validation_reason": reason,
            "external_requests": 0,
            "suggested_action": "search_or_stop"
        }),
    }
}
'''
    text = replace_once(text, anchor, helpers, "action-validation-helpers")

    old_prompt_and_plan = r'''fn planner_prompt(
    case: CaseSpec,
    observation: &ToolObservation,
    tried_queries: &BTreeSet<String>,
    round: usize,
    remaining_calls: usize,
) -> String {
    format!(
        "Task: {}\nProperty to retrieve: {} ({})\nLast qualified tool observation: {}\nCompact search state: {}\nAlready tried normalized queries: {}\nRound: {}. Remaining tool-call budget: {}.\nChoose one typed action only: search, follow_suggested_query, or stop. If compact search state contains a non-empty suggested_query that has not already been tried, prefer follow_suggested_query instead of regenerating or editing that query. The Harness may suppress a fact when entity identity is insufficient. If the task itself contains legitimate disambiguating context and there is no suggested query, use only that stated context to formulate a search query. Never invent context. Do not decide whether the target fact is true and do not override the identity gate; the Harness owns evidence admission and final correctness.",
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
                "You are an evidence-search planner inside a bounded verification harness. Return exactly one JSON object using one of these forms: {\"action\":\"search\",\"query\":\"entity label/title\"}, {\"action\":\"follow_suggested_query\",\"query\":null}, or {\"action\":\"stop\",\"query\":null}. You propose actions only. You do not decide truth, identity sufficiency, evidence admission, or final correctness. If search_state has a non-empty suggested_query that has not already been tried, choose follow_suggested_query so the Harness can use that exact validated query without model rewriting. Otherwise, if identity is insufficient and the task supplies explicit context, use only that context to reformulate. If no legitimate disambiguation exists, stop."
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

    match action.action.as_str() {
        "search" => {
            if action.query.as_deref().is_none_or(|value| value.trim().is_empty()) {
                return Err(StopReason::PlannerProtocolFailure);
            }
        }
        "follow_suggested_query" => {
            if suggested_query(&observation.search_state).is_none() {
                return Err(StopReason::PlannerProtocolFailure);
            }
        }
        "stop" => {}
        _ => return Err(StopReason::PlannerProtocolFailure),
    }

    Ok((action, tokens, response.provider_attempts))
}
'''

    new_prompt_and_plan = r'''fn planner_prompt(
    case: CaseSpec,
    observation: &ToolObservation,
    tried_queries: &BTreeSet<String>,
    round: usize,
    remaining_calls: usize,
) -> String {
    let available_suggestion = suggested_query(&observation.search_state)
        .filter(|query| !tried_queries.contains(&normalize_query(query)));
    let available_actions = if available_suggestion.is_some() {
        "search, follow_suggested_query, or stop"
    } else {
        "search or stop (follow_suggested_query is unavailable in this state)"
    };
    let identity_guidance = if state_kind(&observation.search_state) == Some("identity_insufficient") {
        "Entity identity is currently insufficient. If the task explicitly supplies disambiguating context that is not reflected in an already-tried query, search using only that stated context before stopping."
    } else {
        "Do not invent disambiguating context."
    };
    format!(
        "Task: {}\nProperty to retrieve: {} ({})\nLast qualified tool observation: {}\nCompact search state: {}\nAlready tried normalized queries: {}\nRound: {}. Remaining tool-call budget: {}.\nAvailable typed actions now: {}. {} If a validated suggested_query is available, prefer follow_suggested_query instead of regenerating or editing it. Never decide whether the target fact is true and never override the identity gate; the Harness owns identity sufficiency, evidence admission, and final correctness.",
        case.task,
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
    let action_contract = if follow_available {
        "Return exactly one JSON object: {\"action\":\"search\",\"query\":\"entity label/title\"}, {\"action\":\"follow_suggested_query\",\"query\":null}, or {\"action\":\"stop\",\"query\":null}."
    } else {
        "Return exactly one JSON object: {\"action\":\"search\",\"query\":\"entity label/title\"} or {\"action\":\"stop\",\"query\":null}. follow_suggested_query is unavailable in this state."
    };
    let system = format!(
        "You are an evidence-search planner inside a bounded verification harness. {action_contract} You propose actions only. You do not decide truth, identity sufficiency, evidence admission, or final correctness. If identity is insufficient and the task explicitly supplies disambiguating context that has not yet been searched, issue one search using only that stated context before stopping. If no legitimate disambiguation exists, stop."
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
        "search" => {
            if action.query.as_deref().is_none_or(|value| value.trim().is_empty()) {
                return Err(StopReason::PlannerProtocolFailure);
            }
        }
        "follow_suggested_query" | "stop" => {}
        _ => return Err(StopReason::PlannerProtocolFailure),
    }

    Ok((action, tokens, response.provider_attempts))
}
'''
    text = replace_once(text, old_prompt_and_plan, new_prompt_and_plan, "planner-prompt-and-syntax")

    text = replace_count(
        text,
        '                followed_suggested_query: false,\n'
        '            });',
        '                followed_suggested_query: false,\n'
        '                planner_action_repairs: Vec::new(),\n'
        '            });',
        2,
        "non-planner-round-traces",
    )

    old_planner_block = r'''        planner_calls += 1;
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

        let mut followed_suggested_query = false;
        let next_query = match action.action.as_str() {
            "search" => action.query.clone(),
            "follow_suggested_query" => {
                let value = suggested_query(&observation.search_state);
                if value.is_some() {
                    follow_suggested_query_actions += 1;
                    followed_suggested_query = true;
                }
                value
            }
            "stop" => None,
            _ => None,
        };

        traces.push(RoundTrace {
            round,
            query: query.clone(),
            observation: Some(observation.observation),
            search_state: Some(observation.search_state),
            new_progress_items: new_items.len(),
            planner_action: Some(action.action.clone()),
            planner_query: next_query.clone(),
            followed_suggested_query,
        });

        if model_tokens > MAX_MODEL_TOKENS {
            stop_reason = StopReason::ModelTokenBudget;
            break;
        }
        if action.action == "stop" {
            stop_reason = StopReason::PlannerStop;
            break;
        }
        let Some(next_query) = next_query else {
            stop_reason = StopReason::PlannerProtocolFailure;
            break;
        };
        query = next_query;
'''

    new_planner_block = r'''        let mut planner_observation = ToolObservation {
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

            match planner_action_resolution(&action, &planner_observation, &tried_queries) {
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
                    planner_observation = invalid_planner_action_feedback(&action, reason);
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
'''
    text = replace_once(text, old_planner_block, new_planner_block, "bounded-action-repair-loop")

    text = replace_once(
        text,
        '                StopReason::PlannerProtocolFailure | StopReason::PlannerProviderFailure\n',
        '                StopReason::PlannerProtocolFailure\n'
        '                    | StopReason::PlannerActionRepairExhausted\n'
        '                    | StopReason::PlannerProviderFailure\n',
        "planner-failure-attribution",
    )

    text = replace_once(
        text,
        '            max_model_tokens: MAX_MODEL_TOKENS,\n'
        '            no_progress_rounds: NO_PROGRESS_LIMIT,\n',
        '            max_model_tokens: MAX_MODEL_TOKENS,\n'
        '            no_progress_rounds: NO_PROGRESS_LIMIT,\n'
        '            max_action_repairs_per_observation: MAX_ACTION_REPAIRS_PER_OBSERVATION,\n',
        "reported-action-repair-budget",
    )

    text = replace_once(
        text,
        '        planner_action_policy: "planner chooses typed search/follow_suggested_query/stop; Harness substitutes exact tool-provided suggested_query only when follow_suggested_query is explicitly selected",\n',
        '        planner_action_policy: "planner chooses only currently available typed actions; Harness validates action availability, returns unavailable actions as typed no-external-request feedback, permits at most two bounded repairs, and substitutes exact tool-provided suggested_query only when follow_suggested_query is explicitly selected",\n',
        "reported-action-policy",
    )

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
