#!/usr/bin/env python3
import argparse
import json
import os
import re
import subprocess
import sys
import time
from collections import Counter, defaultdict
from datetime import datetime, timezone
from pathlib import Path

REPORT_SCHEMA = "reason-planner-reliability-v1"
CORPUS_ID = "planner-reliability-v1"
EVALUATOR_ID = "reason-planner-reliability-v1"
SCORING_ID = "planner-reliability-scoring-v1"
CANON_GROUNDED = re.compile(r"^([^;=()]+?) = (.+)$")
CANON_UNCERTAIN = re.compile(r"^uncertain\(([^;=()]+?) = (.+)\)$")
OPERATIONAL_ATTEMPT_STATUSES = {
    "applied_candidate_revision",
    "human_review_required",
    "adapter_unavailable",
    "malformed_output",
    "adapter_failed",
    "transport_failure",
    "authentication_failure",
    "permission_denied",
    "negotiation_failure",
    "session_failure",
    "protocol_failure",
    "tool_failed",
    "timed_out",
    "policy_denied",
    "budget_exceeded",
}

class EvalError(Exception):
    pass

def load_json(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)

def final_artifact(result):
    if result.get("output_contract") != "reason-natural-output-v4":
        raise EvalError("natural output contract must be reason-natural-output-v4")
    outcome = result.get("final_outcome")
    if not isinstance(outcome, dict) or not isinstance(outcome.get("artifact"), dict):
        raise EvalError("natural output is missing final_outcome.artifact")
    return outcome["artifact"]

def unwrap_partial(status, text):
    if status != "qualified_partial_answer" or not isinstance(text, str):
        return text
    wrappers = [
        (
            "verified partial: ",
            "; other generated claims remain unresolved and are omitted",
        ),
        (
            "verified target only: ",
            "; full reasoning artifact remains rejected because structurally independent non-target state was contradicted",
        ),
    ]
    for prefix, suffix in wrappers:
        if text.startswith(prefix) and text.endswith(suffix):
            return text[len(prefix) : -len(suffix)]
    return text

def parse_exposed(text):
    if text is None:
        return [], 0
    assertions = []
    bad = 0
    for seg in text.split("; "):
        m = CANON_UNCERTAIN.match(seg)
        if m:
            assertions.append((m.group(1).strip(), m.group(2).strip(), "uncertain"))
            continue
        m = CANON_GROUNDED.match(seg)
        if m:
            assertions.append((m.group(1).strip(), m.group(2).strip(), "grounded"))
            continue
        bad += 1
    return assertions, bad

def artifact_supports(artifact, key, value, mode):
    for claim in artifact.get("claims", []):
        proposition = claim.get("proposition") or {}
        if proposition.get("key") == key and str(proposition.get("value")) == value:
            state = claim.get("state")
            if mode == "grounded":
                return state in ("known", "supported")
            return state in ("known", "supported", "inferred", "assumed", "unknown")
    return False

def exposed_metrics(finalization, artifact, target):
    status = finalization.get("status")
    text = unwrap_partial(status, finalization.get("text"))
    assertions, bad = parse_exposed(text)
    blocked_text = int(status == "requires_verification" and text is not None)
    unsupported = bad + blocked_text
    target_grounded = False
    target_uncertain = False
    for key, value, mode in assertions:
        if not artifact_supports(artifact, key, value, mode):
            unsupported += 1
        if key == target["key"] and value == str(target["value"]):
            target_grounded |= mode == "grounded"
            target_uncertain |= mode == "uncertain"
    uncovered = max(
        0,
        int(finalization.get("factual_claims", 0))
        - int(finalization.get("covered_claims", 0)),
    )
    answer_emitting = status in ("grounded_answer", "qualified_partial_answer")
    structured = uncovered if answer_emitting else 0
    blocked_unverified = uncovered if status == "requires_verification" else 0
    return {
        "unsupported_exposed_assertions": unsupported,
        "exposed_text_contract_violations": bad + blocked_text,
        "unsupported_structured_claims": structured,
        "blocked_unverified_propositions": blocked_unverified,
        "target_grounded": target_grounded,
        "target_uncertain": target_uncertain,
        "assertions": [
            {"key": k, "value": v, "mode": m} for k, v, m in assertions
        ],
    }

def generation_costs(obj):
    calls = tokens = latency = attempts = 0
    models = Counter()

    def walk(x):
        nonlocal calls, tokens, latency, attempts
        if isinstance(x, dict):
            if (
                "usage" in x
                and "latency_ms" in x
                and "model" in x
                and isinstance(x.get("usage"), dict)
            ):
                usage = x["usage"]
                total = usage.get("total_tokens")
                if total is None:
                    total = (usage.get("input_tokens") or 0) + (
                        usage.get("output_tokens") or 0
                    )
                tokens += int(total or 0)
                latency += int(x.get("latency_ms") or 0)
                calls += 1
                attempts += int(x.get("provider_attempts") or 1)
                models[str(x.get("model"))] += 1
            for value in x.values():
                walk(value)
        elif isinstance(x, list):
            for value in x:
                walk(value)

    walk(obj)
    return {
        "provider_calls_observed": calls,
        "provider_attempts_observed": attempts,
        "tokens_observed": tokens,
        "provider_latency_ms_observed": latency,
        "observed_model_identities": dict(models),
    }

def collect_admission_rejections(result):
    out = Counter()
    for round_data in result.get("resolution_rounds") or []:
        for attempt in round_data.get("attempts") or []:
            reason = attempt.get("admission_rejection")
            if reason:
                out[reason] += 1
    return out

def collect_operational_attempt_failures(result):
    out = Counter()
    for round_data in result.get("resolution_rounds") or []:
        for attempt in round_data.get("attempts") or []:
            status = attempt.get("status")
            if status in OPERATIONAL_ATTEMPT_STATUSES:
                out[status] += 1
    return out

def run_json(cmd, cwd, env):
    started = time.monotonic()
    cp = subprocess.run(
        cmd,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=cwd,
        env=env,
    )
    elapsed = int((time.monotonic() - started) * 1000)
    payload = None
    try:
        if cp.stdout.strip():
            payload = json.loads(cp.stdout)
    except Exception:
        pass
    return cp, payload, elapsed

def invocation_failure(case, cp, payload, elapsed):
    failure = (
        (payload or {}).get("result", {}).get("failure", {})
        if isinstance(payload, dict)
        else {}
    )
    return {
        "id": case["id"],
        "role": case["role"],
        "expected": case["expected"],
        "operationally_complete": False,
        "planner_success": None,
        "operational_failure": {
            "exit_code": cp.returncode,
            "failure_class": failure.get("failure_class", "process_failure"),
            "message": failure.get("message")
            or cp.stderr.decode(errors="replace")[:500],
        },
        "wall_clock_ms": elapsed,
    }

def score_case(case, result, elapsed):
    target = case["target"]
    telemetry = (result.get("investigation") or {}).get("telemetry") or {}
    targets = telemetry.get("targets") or []
    actions = telemetry.get("actions") or []
    target_recalled = any(
        t.get("expected_fact_key") == target["key"] for t in targets
    )
    relevant = set(case.get("relevant_capabilities") or [])
    selected = [a.get("action", {}).get("capability_id") for a in actions]
    tool_selected = any(cap in relevant for cap in selected)
    irrelevant_acquisition_attempts = sum(
        1 for cap in selected if cap and cap not in relevant
    )
    action_rejections = dict(telemetry.get("rejected_actions") or {})
    rejection_records = list(telemetry.get("action_rejection_records") or [])
    action_rejection_count = sum(int(v or 0) for v in action_rejections.values())
    invalid_shape_rejections = int(action_rejections.get("invalid_shape", 0))
    valid_action_shape = invalid_shape_rejections == 0

    coverage = case.get("coverage_contract") or {}
    trigger_exposed = None
    mechanism_conformant = None
    mechanism_classification = None
    avoidable_followup_stall = 0
    if coverage.get("kind") == "no_result_followup_observational":
        first = coverage["first_capability"]
        follow = coverage["followup_capability"]
        relevant_indices = [
            i
            for i, action in enumerate(actions)
            if action.get("action", {}).get("capability_id") in (first, follow)
        ]
        first_index = relevant_indices[0] if relevant_indices else None
        first_cap = None
        first_status = None
        if first_index is not None:
            first_action = actions[first_index]
            first_cap = first_action.get("action", {}).get("capability_id")
            first_status = first_action.get("status")
        trigger_exposed = bool(
            first_index is not None
            and first_cap == first
            and first_status == "no_result"
        )
        next_action = (
            actions[first_index + 1]
            if trigger_exposed and first_index + 1 < len(actions)
            else None
        )
        next_cap = (
            next_action.get("action", {}).get("capability_id")
            if isinstance(next_action, dict)
            else None
        )
        harness_selected = (
            int(telemetry.get("harness_no_result_followup_selections") or 0) >= 1
        )
        mechanism_conformant = bool(
            trigger_exposed and next_cap == follow and harness_selected
        )
        mechanism_classification = (
            "conformant"
            if mechanism_conformant
            else ("nonconformant" if trigger_exposed else "unexposed")
        )
        avoidable_followup_stall = int(target_recalled and len(actions) == 0)

    artifact = final_artifact(result)
    finalization = result.get("finalization") or {}
    exposed = exposed_metrics(finalization, artifact, target)
    admission_rejections = collect_admission_rejections(result)
    typed_operational = collect_operational_attempt_failures(result)
    typed_operational_count = sum(typed_operational.values())
    generation_failure = (result.get("investigation") or {}).get("generation_failure")
    generation_failure_observed = int(isinstance(generation_failure, dict))
    operationally_complete = (
        typed_operational_count == 0 and generation_failure_observed == 0
    )

    expected = case["expected"]
    false_abstention = int(expected == "grounded" and not exposed["target_grounded"])
    missed_insufficiency = int(expected == "unknown" and exposed["target_grounded"])
    expected_rejection = case.get("expected_rejection") or []
    expected_rejection_observed = (
        None
        if not expected_rejection
        else any(admission_rejections.get(name, 0) > 0 for name in expected_rejection)
    )
    correctness = (
        exposed["unsupported_exposed_assertions"]
        + exposed["unsupported_structured_claims"]
        + missed_insufficiency
    )

    planner_success = None
    if operationally_complete:
        planner_success = bool(
            correctness == 0
            and target_recalled
            and tool_selected
            and action_rejection_count == 0
            and irrelevant_acquisition_attempts == 0
            and avoidable_followup_stall == 0
            and false_abstention == 0
            and (
                expected != "grounded" or exposed["target_grounded"]
            )
            and (
                expected != "unknown" or not exposed["target_grounded"]
            )
            and (
                expected_rejection_observed is not False
            )
            and (
                trigger_exposed is not True or mechanism_conformant is True
            )
        )

    return {
        "id": case["id"],
        "role": case["role"],
        "expected": expected,
        "target": target,
        "operationally_complete": operationally_complete,
        "planner_success": planner_success,
        "target_recalled": target_recalled,
        "tool_selection_success": tool_selected,
        "valid_action_shape": valid_action_shape,
        "selected_capabilities": selected,
        "action_count": len(actions),
        "action_rejections": action_rejections,
        "action_rejection_count": action_rejection_count,
        "action_rejection_records": rejection_records,
        "invalid_shape_rejections": invalid_shape_rejections,
        "irrelevant_acquisition_attempts": irrelevant_acquisition_attempts,
        "harness_unique_selections": int(
            telemetry.get("harness_unique_selections") or 0
        ),
        "harness_precedence_selections": int(
            telemetry.get("harness_precedence_selections") or 0
        ),
        "harness_no_result_followup_selections": int(
            telemetry.get("harness_no_result_followup_selections") or 0
        ),
        "planner_calls": int(telemetry.get("planner_calls") or 0),
        "stop_reason": telemetry.get("stop_reason"),
        "trigger_exposed": trigger_exposed,
        "mechanism_conformant": mechanism_conformant,
        "mechanism_classification": mechanism_classification,
        "avoidable_followup_stall": avoidable_followup_stall,
        "admission_rejections": dict(admission_rejections),
        "expected_rejection_observed": expected_rejection_observed,
        "finalization_status": finalization.get("status"),
        "false_abstention": false_abstention,
        "missed_target_insufficiency": missed_insufficiency,
        "typed_operational_action_failures": typed_operational_count,
        "typed_operational_failure_classes": dict(typed_operational),
        "generation_failure_observed": generation_failure_observed,
        "generation_failure_class": (
            generation_failure.get("failure_class")
            if isinstance(generation_failure, dict)
            else None
        ),
        "correctness_boundary_violations": correctness,
        "wall_clock_ms": elapsed,
        **exposed,
        **generation_costs(result),
    }

def validate_corpus(root):
    root = Path(root)
    manifest = load_json(root / "manifest.json")
    expected_identity = {
        "schema_version": "planner-reliability-manifest-v1",
        "corpus_identity": CORPUS_ID,
        "evaluator_identity": EVALUATOR_ID,
        "scoring_identity": SCORING_ID,
        "purpose": "baseline_for_issue_283",
    }
    for key, value in expected_identity.items():
        if manifest.get(key) != value:
            raise EvalError(f"manifest {key} drift")

    product = manifest.get("product_coordinate") or {}
    if product != {
        "reason_cli": "0.5.2",
        "harness_engine": "0.4.2",
        "commit": "bcbd326e147fae21f06a601f988e6ab060ca41bb",
    }:
        raise EvalError("product coordinate drift")

    providers = manifest.get("provider_targets") or {}
    expected_providers = {
        "mistral": {
            "provider": "mistral",
            "model": "ministral-8b-latest",
            "max_tokens": 1024,
            "inter_case_delay_ms": 1500,
        },
        "google": {
            "provider": "google",
            "model": "gemini-3.5-flash-lite",
            "max_tokens": 1024,
            "inter_case_delay_ms": 1500,
        },
    }
    if providers != expected_providers:
        raise EvalError("provider target drift")

    trial_plan = manifest.get("trial_plan") or {}
    seeds = trial_plan.get("primary_trial_seeds")
    if seeds != [86101, 86102, 86103, 86104, 86105]:
        raise EvalError("primary trial seed drift")
    if trial_plan.get("declared_k") != [1, 5]:
        raise EvalError("declared k drift")
    all_k = trial_plan.get("all_k_group") or {}
    if all_k.get("k") != 5 or all_k.get("seeds") != seeds:
        raise EvalError("all-k group drift")
    pair = trial_plan.get("matched_surface_sensitivity") or {}
    if pair != {
        "surface_a": "surface-a",
        "surface_b": "surface-b",
        "seed": 86105,
    }:
        raise EvalError("matched-surface policy drift")

    live_policy = manifest.get("live_observation_policy") or {}
    expected_live_policy = {
        "surface_must_be_frozen_before_first_live_observation": True,
        "first_live_launch_per_provider_is_canonical": True,
        "semantic_retry_forbidden": True,
        "failed_observation_requires_fresh_successor_identity": True,
        "pre_live_infrastructure_retry_allowed": True,
    }
    if live_policy != expected_live_policy:
        raise EvalError("live observation policy drift")

    surface_contract = manifest.get("surface_contract") or {}
    if surface_contract != {
        "case_roles": ["direct_grounded", "no_result_followup", "stale_unknown"],
        "minimum_exact_read_only_candidates_per_target": 2,
        "nonmatching_distractor_required": True,
        "selection_priority_forbidden": True,
        "stochastic_action_selector_exposure_required": True,
    }:
        raise EvalError("surface contract drift")

    semantics = manifest.get("measurement_semantics") or {}
    required_true = [
        "exercised_path_observability_separate_from_utility",
        "hard_correctness_separate_from_utility",
        "operational_completeness_separate_from_semantic_metrics",
        "operationally_incomplete_trials_excluded_from_semantic_distributions",
        "utility_success_not_required_for_measurement_acceptance",
        "no_cross_model_averaging",
        "no_majority_vote_authority",
    ]
    if any(semantics.get(key) is not True for key in required_true):
        raise EvalError("measurement semantics drift")
    if semantics.get("independence_derived_p_to_k_reported") is not False:
        raise EvalError("independence estimate must remain disabled")

    surfaces = manifest.get("surfaces") or {}
    if set(surfaces) != {"surface-a", "surface-b"}:
        raise EvalError("surface set drift")

    loaded = {}
    markers = set()
    for surface_name, surface in surfaces.items():
        files = surface.get("case_files") or []
        if len(files) != 3 or len(set(files)) != 3:
            raise EvalError(f"{surface_name}: expected three unique cases")
        cases = []
        roles = []
        for rel in files:
            case = load_json(root / rel)
            if case.get("schema_version") != "planner-reliability-case-v1":
                raise EvalError(f"{rel}: schema drift")
            if case.get("kind") != "investigation":
                raise EvalError(f"{rel}: investigation only")
            if "hypothesis" in case or "start_hypothesis" in case:
                raise EvalError(f"{rel}: explicit hypothesis forbidden")
            role = case.get("role")
            roles.append(role)
            target = case.get("target") or {}
            if not target.get("key") or "value" not in target:
                raise EvalError(f"{rel}: target missing")
            for marker in case.get("fresh_markers") or []:
                if marker in markers:
                    raise EvalError(f"{rel}: duplicate marker {marker}")
                markers.add(marker)
            cfg = load_json(root / case["config"])
            inv = (cfg.get("resolution") or {}).get("investigation") or {}
            caps = inv.get("capabilities") or []
            if len(caps) < 3:
                raise EvalError(f"{rel}: expected multi-capability planner surface")
            if any(cap.get("read_only") is not True for cap in caps):
                raise EvalError(f"{rel}: all capabilities must be read-only")
            if any("selection_priority" in cap for cap in caps):
                raise EvalError(f"{rel}: selection_priority forbidden in stochastic baseline")
            relevant = set(case.get("relevant_capabilities") or [])
            exact_relevant = [
                cap
                for cap in caps
                if cap.get("id") in relevant
                and target["key"] in (cap.get("supported_fact_keys") or [])
            ]
            if len(exact_relevant) < 2:
                raise EvalError(f"{rel}: need at least two exact relevant capabilities")
            distractors = [
                cap
                for cap in caps
                if cap.get("id") not in relevant
                and target["key"] not in (cap.get("supported_fact_keys") or [])
            ]
            if not distractors:
                raise EvalError(f"{rel}: missing nonmatching distractor")
            if role == "no_result_followup":
                contract = case.get("coverage_contract") or {}
                if contract.get("kind") != "no_result_followup_observational":
                    raise EvalError(f"{rel}: followup coverage contract drift")
                if contract.get("first_capability") not in relevant:
                    raise EvalError(f"{rel}: followup first capability not relevant")
                if contract.get("followup_capability") not in relevant:
                    raise EvalError(f"{rel}: followup capability not relevant")
            if role == "stale_unknown":
                if case.get("expected") != "unknown" or "stale" not in (
                    case.get("expected_rejection") or []
                ):
                    raise EvalError(f"{rel}: stale contract drift")
            cases.append(case)
        if roles != ["direct_grounded", "no_result_followup", "stale_unknown"]:
            raise EvalError(f"{surface_name}: role ordering drift")
        loaded[surface_name] = cases

    a = loaded["surface-a"]
    b = loaded["surface-b"]
    for ca, cb in zip(a, b):
        if (
            ca["role"] != cb["role"]
            or ca["expected"] != cb["expected"]
            or bool(ca.get("coverage_contract"))
            != bool(cb.get("coverage_contract"))
        ):
            raise EvalError("surface information-equivalence shape drift")

    acceptance = manifest.get("acceptance") or {}
    if acceptance != {
        "min_complete_primary_trials": 5,
        "require_matched_surface_pair_complete": True,
        "max_correctness_boundary_violations": 0,
        "planner_utility_success_required": False,
    }:
        raise EvalError("acceptance policy drift")
    return manifest, loaded


def preflight_request():
    return {
        "schema_version": "reason-investigation-external-resolver-request-v1",
        "adapter_id": "investigation_external_command_v1",
        "attempt_index": 0,
        "request": {
            "id": "preflight",
            "reason": "investigation",
            "target": {
                "kind": "investigation_question",
                "target_id": "preflight",
                "question": "preflight",
                "expected_fact_key": "preflight.key",
            },
            "resolver_class": "evidence_acquisition",
            "budget": {
                "max_attempts": 1,
                "max_added_tokens": 0,
                "max_elapsed_ms": 5000,
            },
        },
    }

def run_fixture_capability(root, cap):
    repo_root = root.parent
    while not (repo_root / "scripts" / "natural_e2e_fixture_resolver.py").exists():
        if repo_root.parent == repo_root:
            raise EvalError("repository root not found for fixture preflight")
        repo_root = repo_root.parent
    cp = subprocess.run(
        [cap["program"], *cap.get("args", [])],
        input=json.dumps(preflight_request()).encode(),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=repo_root,
    )
    if cp.returncode != 0:
        raise EvalError(
            f"resolver preflight failed {cap['id']}: "
            f"{cp.stderr.decode(errors='replace')}"
        )
    try:
        out = json.loads(cp.stdout)
    except Exception as error:
        raise EvalError(f"resolver JSON {cap['id']}: {error}")
    if out.get("schema_version") != "reason-external-resolver-response-v1":
        raise EvalError(f"resolver schema {cap['id']}")
    return out

def self_test_fixtures(root, surfaces):
    root = Path(root)
    summary = {
        "capabilities_checked": 0,
        "positive_evidence_capabilities": 0,
        "no_result_capabilities": 0,
        "stale_rejection_contracts": 0,
        "followup_sequence_contracts": 0,
    }
    for surface_name, cases in surfaces.items():
        for case in cases:
            cfg = load_json(root / case["config"])
            caps = {
                cap["id"]: cap
                for cap in cfg["resolution"]["investigation"]["capabilities"]
            }
            target_key = case["target"]["key"]
            relevant = set(case["relevant_capabilities"])
            stale_verified = False
            positive_verified = False
            for cap in caps.values():
                out = run_fixture_capability(root, cap)
                summary["capabilities_checked"] += 1
                contribution = out.get("contribution") or {}
                if contribution.get("kind") == "no_result":
                    summary["no_result_capabilities"] += 1
                    continue
                if contribution.get("kind") != "acquired_evidence":
                    raise EvalError(
                        f"{surface_name}:{case['id']}:{cap['id']}: "
                        "unexpected resolver contribution"
                    )
                evidence = contribution.get("evidence") or []
                if not evidence:
                    raise EvalError(
                        f"{surface_name}:{case['id']}:{cap['id']}: empty evidence"
                    )
                item = evidence[0]
                source = item.get("source")
                admission = cap.get("admission") or {}
                source_policy = (admission.get("sources") or {}).get(source)
                if source_policy is None:
                    raise EvalError(
                        f"{surface_name}:{case['id']}:{cap['id']}: "
                        "resolver source not allowlisted"
                    )
                metadata = item.get("acquisition_metadata") or {}
                observed = metadata.get("observed_at_unix_seconds")
                evaluation = admission.get("evaluation_time_unix_seconds")
                max_age = source_policy.get("max_age_seconds")
                is_stale = (
                    isinstance(observed, int)
                    and isinstance(evaluation, int)
                    and isinstance(max_age, int)
                    and evaluation - observed > max_age
                )
                if cap["id"] in relevant and target_key in (
                    cap.get("supported_fact_keys") or []
                ):
                    if case["role"] == "stale_unknown":
                        stale_verified |= is_stale
                    else:
                        if is_stale:
                            raise EvalError(
                                f"{surface_name}:{case['id']}: "
                                "positive relevant evidence is stale"
                            )
                        positive_verified = True
                        summary["positive_evidence_capabilities"] += 1
            if case["role"] == "stale_unknown":
                if not stale_verified:
                    raise EvalError(
                        f"{surface_name}:{case['id']}: stale fixture not verified"
                    )
                summary["stale_rejection_contracts"] += 1
            elif not positive_verified:
                raise EvalError(
                    f"{surface_name}:{case['id']}: "
                    "no mechanically admissible positive evidence"
                )
            contract = case.get("coverage_contract") or {}
            if contract.get("kind") == "no_result_followup_observational":
                first = run_fixture_capability(root, caps[contract["first_capability"]])
                follow = run_fixture_capability(
                    root, caps[contract["followup_capability"]]
                )
                if (first.get("contribution") or {}).get("kind") != "no_result":
                    raise EvalError(
                        f"{surface_name}:{case['id']}: first capability not no_result"
                    )
                follow_contribution = follow.get("contribution") or {}
                if (
                    follow_contribution.get("kind") != "acquired_evidence"
                    or not (follow_contribution.get("evidence") or [])
                ):
                    raise EvalError(
                        f"{surface_name}:{case['id']}: followup capability not evidence"
                    )
                summary["followup_sequence_contracts"] += 1
    return summary

def summarize_trial(trial):
    cases = trial["cases"]
    complete = all(c.get("operationally_complete") is True for c in cases)
    if not complete:
        planner_success = None
    else:
        planner_success = all(c.get("planner_success") is True for c in cases)
    action_rejections = Counter()
    stop_reasons = Counter()
    observed_models = Counter()
    operational_classes = Counter()
    for case in cases:
        action_rejections.update(case.get("action_rejections") or {})
        if case.get("stop_reason") is not None:
            stop_reasons[str(case["stop_reason"])] += 1
        observed_models.update(case.get("observed_model_identities") or {})
        if case.get("operational_failure"):
            operational_classes[
                case["operational_failure"].get("failure_class", "process_failure")
            ] += 1
        operational_classes.update(case.get("typed_operational_failure_classes") or {})
        if case.get("generation_failure_class"):
            operational_classes[
                f"generation:{case['generation_failure_class']}"
            ] += 1

    trigger_cases = [c for c in cases if c.get("trigger_exposed") is not None]
    exposed = sum(c.get("trigger_exposed") is True for c in trigger_cases)
    conformant = sum(
        c.get("mechanism_conformant") is True
        for c in trigger_cases
        if c.get("trigger_exposed") is True
    )
    return {
        "operationally_complete": complete,
        "planner_success": planner_success,
        "case_count": len(cases),
        "target_recall_hits": sum(bool(c.get("target_recalled")) for c in cases),
        "tool_selection_hits": sum(bool(c.get("tool_selection_success")) for c in cases),
        "valid_action_shape_hits": sum(bool(c.get("valid_action_shape")) for c in cases),
        "action_rejection_count": sum(action_rejections.values()),
        "action_rejections": dict(action_rejections),
        "invalid_shape_rejections": int(action_rejections.get("invalid_shape", 0)),
        "avoidable_followup_stalls": sum(
            int(c.get("avoidable_followup_stall") or 0) for c in cases
        ),
        "false_abstentions": sum(int(c.get("false_abstention") or 0) for c in cases),
        "correctness_boundary_violations": sum(
            int(c.get("correctness_boundary_violations") or 0) for c in cases
        ),
        "trigger_exposed_cases": exposed,
        "trigger_required_cases": len(trigger_cases),
        "mechanism_conformant_cases": conformant,
        "mechanism_denominator": exposed,
        "stop_reasons": dict(stop_reasons),
        "operational_failure_classes": dict(operational_classes),
        "provider_calls_observed": sum(
            int(c.get("provider_calls_observed") or 0) for c in cases
        ),
        "provider_attempts_observed": sum(
            int(c.get("provider_attempts_observed") or 0) for c in cases
        ),
        "tokens_observed": sum(int(c.get("tokens_observed") or 0) for c in cases),
        "provider_latency_ms_observed": sum(
            int(c.get("provider_latency_ms_observed") or 0) for c in cases
        ),
        "observed_model_identities": dict(observed_models),
        "wall_clock_ms": sum(int(c.get("wall_clock_ms") or 0) for c in cases),
    }

def metric_distribution(values):
    if not values:
        return {"count": 0, "mean": None, "min": None, "max": None, "values": []}
    return {
        "count": len(values),
        "mean": sum(values) / len(values),
        "min": min(values),
        "max": max(values),
        "values": values,
    }

def aggregate_primary(trials, manifest):
    complete = [t for t in trials if t["summary"]["operationally_complete"]]
    incomplete = [t for t in trials if not t["summary"]["operationally_complete"]]
    planner_successes = [
        bool(t["summary"]["planner_success"]) for t in complete
    ]
    by_case = defaultdict(list)
    for trial in complete:
        for case in trial["cases"]:
            by_case[case["role"]].append(case)

    per_case = {}
    for role, cases in by_case.items():
        rejection_counts = Counter()
        stop_reasons = Counter()
        for case in cases:
            rejection_counts.update(case.get("action_rejections") or {})
            if case.get("stop_reason") is not None:
                stop_reasons[str(case["stop_reason"])] += 1
        exposed_cases = [c for c in cases if c.get("trigger_exposed") is not None]
        exposed = sum(c.get("trigger_exposed") is True for c in exposed_cases)
        conformant = sum(
            c.get("mechanism_conformant") is True
            for c in exposed_cases
            if c.get("trigger_exposed") is True
        )
        per_case[role] = {
            "denominator_complete_trials": len(cases),
            "target_recall_frequency": (
                sum(bool(c.get("target_recalled")) for c in cases) / len(cases)
                if cases
                else None
            ),
            "valid_action_shape_frequency": (
                sum(bool(c.get("valid_action_shape")) for c in cases) / len(cases)
                if cases
                else None
            ),
            "relevant_tool_selection_frequency": (
                sum(bool(c.get("tool_selection_success")) for c in cases) / len(cases)
                if cases
                else None
            ),
            "trigger_exposure_frequency": (
                exposed / len(exposed_cases) if exposed_cases else None
            ),
            "mechanism_conformance_rate_when_exposed": (
                conformant / exposed if exposed else None
            ),
            "avoidable_followup_stalls": sum(
                int(c.get("avoidable_followup_stall") or 0) for c in cases
            ),
            "false_abstentions": sum(
                int(c.get("false_abstention") or 0) for c in cases
            ),
            "action_rejection_distribution": dict(rejection_counts),
            "stop_reason_distribution": dict(stop_reasons),
        }

    fields = [
        "target_recall_hits",
        "tool_selection_hits",
        "valid_action_shape_hits",
        "action_rejection_count",
        "invalid_shape_rejections",
        "avoidable_followup_stalls",
        "false_abstentions",
        "trigger_exposed_cases",
        "correctness_boundary_violations",
    ]
    distributions = {
        field: metric_distribution(
            [int(t["summary"].get(field) or 0) for t in complete]
        )
        for field in fields
    }

    k_group = manifest["trial_plan"]["all_k_group"]
    k_trials = [
        next((t for t in trials if t["seed"] == seed), None)
        for seed in k_group["seeds"]
    ]
    k_defined = all(
        t is not None and t["summary"]["operationally_complete"] for t in k_trials
    )
    all_k_success = (
        all(t["summary"]["planner_success"] is True for t in k_trials)
        if k_defined
        else None
    )
    operational_classes = Counter()
    for trial in incomplete:
        operational_classes.update(
            trial["summary"].get("operational_failure_classes") or {}
        )

    return {
        "attempted_trials": len(trials),
        "complete_trials": len(complete),
        "incomplete_trials": len(incomplete),
        "trial_completion_rate": len(complete) / len(trials) if trials else None,
        "incomplete_trial_ids": [t["trial_id"] for t in incomplete],
        "operational_failure_classes": dict(operational_classes),
        "pass_at_1": (
            sum(planner_successes) / len(planner_successes)
            if planner_successes
            else None
        ),
        "planner_successes": sum(planner_successes),
        "planner_success_denominator": len(planner_successes),
        "all_k": {
            "k": k_group["k"],
            "trial_seeds": k_group["seeds"],
            "defined": k_defined,
            "observed_all_k_success": all_k_success,
            "derived_independence_p_to_k": None,
            "derived_independence_p_to_k_reason": "not_reported_no_iid_assumption",
        },
        "metric_distributions_complete_trials_only": distributions,
        "per_case_frequencies_complete_trials_only": per_case,
        "total_correctness_boundary_violations_all_attempts": sum(
            int(t["summary"].get("correctness_boundary_violations") or 0)
            for t in trials
        ),
    }

def surface_pair(primary_trials, matched_trial, manifest):
    seed = manifest["trial_plan"]["matched_surface_sensitivity"]["seed"]
    primary = next(t for t in primary_trials if t["seed"] == seed)
    a = primary["summary"]
    b = matched_trial["summary"]
    fields = [
        "target_recall_hits",
        "tool_selection_hits",
        "valid_action_shape_hits",
        "action_rejection_count",
        "invalid_shape_rejections",
        "avoidable_followup_stalls",
        "false_abstentions",
        "trigger_exposed_cases",
        "correctness_boundary_violations",
    ]
    complete = a["operationally_complete"] and b["operationally_complete"]
    return {
        "seed": seed,
        "surface_a_trial_id": primary["trial_id"],
        "surface_b_trial_id": matched_trial["trial_id"],
        "pair_complete": complete,
        "surface_a_planner_success": a["planner_success"] if complete else None,
        "surface_b_planner_success": b["planner_success"] if complete else None,
        "metric_deltas_surface_b_minus_surface_a": (
            {field: int(b.get(field) or 0) - int(a.get(field) or 0) for field in fields}
            if complete
            else None
        ),
        "claim_scope": "single_predeclared_information_equivalent_matched_seed_pair_descriptive_only",
    }

def run_trial(
    root,
    repo_root,
    reason_bin,
    surface_name,
    cases,
    trial_seed,
    provider_spec,
    env,
    delay_ms,
):
    reports = []
    for index, case in enumerate(cases):
        case_seed = trial_seed + index
        cmd = [
            str(reason_bin),
            case["task"],
            "--provider",
            provider_spec["provider"],
            "--model",
            provider_spec["model"],
            "--max-tokens",
            str(provider_spec["max_tokens"]),
            "--seed",
            str(case_seed),
            "--config",
            str((root / case["config"]).resolve()),
            "--format",
            "json",
        ]
        cp, payload, elapsed = run_json(cmd, repo_root, env)
        if cp.returncode != 0 or not isinstance(payload, dict) or "result" not in payload:
            report = invocation_failure(case, cp, payload, elapsed)
        else:
            report = score_case(case, payload["result"], elapsed)
        report["case_seed"] = case_seed
        reports.append(report)
        if index + 1 < len(cases):
            time.sleep(delay_ms / 1000)
    trial = {
        "trial_id": f"{surface_name}-seed-{trial_seed}",
        "surface": surface_name,
        "seed": trial_seed,
        "cases": reports,
    }
    trial["summary"] = summarize_trial(trial)
    return trial

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fixtures", default="fixtures/planner-reliability-v1"
    )
    parser.add_argument("--reason-bin")
    parser.add_argument("--provider-target", choices=["mistral", "google"])
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--attempt-marker")
    parser.add_argument("--output")
    args = parser.parse_args()

    root = Path(args.fixtures).resolve()
    manifest, surfaces = validate_corpus(root)
    if args.validate_only:
        out = {
            "schema_version": REPORT_SCHEMA,
            "corpus_identity": CORPUS_ID,
            "evaluator_identity": EVALUATOR_ID,
            "scoring_identity": SCORING_ID,
            "valid": True,
            "live_observation_performed": False,
            "providers": sorted(manifest["provider_targets"]),
            "primary_trials": len(manifest["trial_plan"]["primary_trial_seeds"]),
            "matched_surface_trials": 1,
            "cases_per_surface": len(surfaces["surface-a"]),
        }
        print(json.dumps(out, indent=2, sort_keys=True))
        return 0

    if args.preflight:
        out = {
            "schema_version": REPORT_SCHEMA,
            "corpus_identity": CORPUS_ID,
            "valid": True,
            "live_observation_performed": False,
            "fixture_preflight": self_test_fixtures(root, surfaces),
        }
        print(json.dumps(out, indent=2, sort_keys=True))
        return 0

    if not args.reason_bin or not args.provider_target:
        raise EvalError("live mode requires --reason-bin and --provider-target")

    provider_spec = manifest["provider_targets"][args.provider_target]
    reason_bin = Path(args.reason_bin).resolve()
    if not reason_bin.is_file():
        raise EvalError("reason binary not found")
    repo_root = Path(__file__).resolve().parents[1]
    env = os.environ.copy()
    delay_ms = provider_spec["inter_case_delay_ms"]

    marker = {
        "schema_version": "planner-reliability-live-attempt-v1",
        "corpus_identity": CORPUS_ID,
        "provider_target": args.provider_target,
        "provider": provider_spec["provider"],
        "model": provider_spec["model"],
        "first_live_launch_boundary_entered": True,
        "started_at_utc": datetime.now(timezone.utc).isoformat(),
        "github_run_id": os.environ.get("GITHUB_RUN_ID"),
        "github_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
    }
    if args.attempt_marker:
        Path(args.attempt_marker).write_text(
            json.dumps(marker, indent=2, sort_keys=True) + "\n"
        )

    primary_trials = []
    for seed in manifest["trial_plan"]["primary_trial_seeds"]:
        primary_trials.append(
            run_trial(
                root,
                repo_root,
                reason_bin,
                "surface-a",
                surfaces["surface-a"],
                seed,
                provider_spec,
                env,
                delay_ms,
            )
        )

    matched_seed = manifest["trial_plan"]["matched_surface_sensitivity"]["seed"]
    matched_trial = run_trial(
        root,
        repo_root,
        reason_bin,
        "surface-b",
        surfaces["surface-b"],
        matched_seed,
        provider_spec,
        env,
        delay_ms,
    )

    aggregate = aggregate_primary(primary_trials, manifest)
    sensitivity = surface_pair(primary_trials, matched_trial, manifest)
    acceptance_policy = manifest["acceptance"]
    acceptance_passed = bool(
        aggregate["complete_trials"]
        >= acceptance_policy["min_complete_primary_trials"]
        and (
            not acceptance_policy["require_matched_surface_pair_complete"]
            or sensitivity["pair_complete"]
        )
        and aggregate["total_correctness_boundary_violations_all_attempts"]
        <= acceptance_policy["max_correctness_boundary_violations"]
    )

    observed_models = Counter()
    for trial in primary_trials + [matched_trial]:
        observed_models.update(trial["summary"]["observed_model_identities"])

    out = {
        "schema_version": REPORT_SCHEMA,
        "corpus_identity": CORPUS_ID,
        "evaluator_identity": EVALUATOR_ID,
        "scoring_identity": SCORING_ID,
        "live_observation_performed": True,
        "provider_target": args.provider_target,
        "provider": provider_spec["provider"],
        "requested_model": provider_spec["model"],
        "observed_model_identities": dict(observed_models),
        "observed_at_utc": datetime.now(timezone.utc).isoformat(),
        "github_run_id": os.environ.get("GITHUB_RUN_ID"),
        "github_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        "product_coordinate": manifest["product_coordinate"],
        "trial_plan": manifest["trial_plan"],
        "measurement_semantics": manifest["measurement_semantics"],
        "planner_success_predicate": manifest["planner_success_predicate"],
        "acceptance_policy": acceptance_policy,
        "primary_trials": primary_trials,
        "matched_surface_trial": matched_trial,
        "aggregate": aggregate,
        "surface_sensitivity": sensitivity,
        "acceptance_passed": acceptance_passed,
    }
    text = json.dumps(out, indent=2, sort_keys=True)
    print(text)
    if args.output:
        Path(args.output).write_text(text + "\n")
    return 0 if acceptance_passed else 3

if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except EvalError as error:
        print(
            json.dumps(
                {
                    "schema_version": REPORT_SCHEMA,
                    "valid": False,
                    "error": str(error),
                }
            )
        )
        raise SystemExit(2)
