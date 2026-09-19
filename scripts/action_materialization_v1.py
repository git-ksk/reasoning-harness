#!/usr/bin/env python3
"""Fresh paired adoption holdout for Issue #283."""
from __future__ import annotations

import argparse
from collections import Counter
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from planner_reliability_v1 import (
    EvalError,
    collect_admission_rejections,
    collect_operational_attempt_failures,
    exposed_metrics,
    final_artifact,
    generation_costs,
    invocation_failure,
    load_json,
    preflight_request,
    run_json,
)

REPORT_SCHEMA = "reason-action-materialization-v1"
CORPUS_ID = "action-materialization-v1"
EVALUATOR_ID = "reason-action-materialization-v1"
SCORING_ID = "action-materialization-scoring-v1"
CONTROL_COMMIT = "94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e"
CANDIDATE_COMMIT = "7a91d272af1bab0a97bf80ed7bba027ff253d50a"
INTENT_CONTRACT = "reason-investigation-intent-v1"
MATERIALIZATION_POLICY = "target-intent-materialization-v1"


def score_case(case, result, elapsed, coordinate):
    target = case["target"]
    investigation = result.get("investigation") or {}
    telemetry = investigation.get("telemetry") or {}
    targets = telemetry.get("targets") or []
    actions = telemetry.get("actions") or []

    target_recalled = any(
        item.get("expected_fact_key") == target["key"] for item in targets
    )
    sibling_ids = [
        item.get("id")
        for item in targets
        if item.get("expected_fact_key") == target["key"] and item.get("id")
    ]
    sibling_count = len(set(sibling_ids))
    sibling_exposed = sibling_count >= int(case["minimum_same_key_targets"])

    relevant = set(case["relevant_capabilities"])
    selected = [
        action.get("action", {}).get("capability_id") for action in actions
    ]
    tool_selected = any(capability in relevant for capability in selected)
    irrelevant = sum(
        1 for capability in selected if capability and capability not in relevant
    )

    action_rejections = dict(telemetry.get("rejected_actions") or {})
    action_rejection_count = sum(int(v or 0) for v in action_rejections.values())
    intent_rejections = dict(telemetry.get("intent_rejections") or {})
    intent_rejection_count = sum(int(v or 0) for v in intent_rejections.values())

    legacy_calls = int(telemetry.get("planner_calls") or 0)
    intent_calls = int(telemetry.get("planner_intent_calls") or 0)
    materializations = int(telemetry.get("harness_intent_materializations") or 0)

    artifact = final_artifact(result)
    finalization = result.get("finalization") or {}
    exposed = exposed_metrics(finalization, artifact, target)
    admission_rejections = collect_admission_rejections(result)
    operational = collect_operational_attempt_failures(result)
    generation_failure = investigation.get("generation_failure")
    operationally_complete = (
        sum(operational.values()) == 0
        and not isinstance(generation_failure, dict)
    )

    expected = case["expected"]
    false_abstention = int(
        expected == "grounded" and not exposed["target_grounded"]
    )
    missed_insufficiency = int(
        expected == "unknown" and exposed["target_grounded"]
    )
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
            and sibling_exposed
            and tool_selected
            and action_rejection_count == 0
            and irrelevant == 0
            and false_abstention == 0
            and (expected != "grounded" or exposed["target_grounded"])
            and (expected != "unknown" or not exposed["target_grounded"])
            and expected_rejection_observed is not False
        )

    return {
        "id": case["id"],
        "role": case["role"],
        "expected": expected,
        "target": target,
        "operationally_complete": operationally_complete,
        "planner_success": planner_success,
        "target_recalled": target_recalled,
        "same_key_target_count": sibling_count,
        "same_key_target_ids": sibling_ids,
        "same_key_sibling_exposed": sibling_exposed,
        "tool_selection_success": tool_selected,
        "selected_capabilities": selected,
        "action_count": len(actions),
        "irrelevant_acquisition_attempts": irrelevant,
        "action_rejections": action_rejections,
        "action_rejection_count": action_rejection_count,
        "action_rejection_records": list(
            telemetry.get("action_rejection_records") or []
        ),
        "intent_rejections": intent_rejections,
        "intent_rejection_count": intent_rejection_count,
        "intent_rejection_records": list(
            telemetry.get("intent_rejection_records") or []
        ),
        "legacy_action_planner_calls": legacy_calls,
        "intent_planner_calls": intent_calls,
        "harness_intent_materializations": materializations,
        "control_path_exposed": (
            legacy_calls > 0 if coordinate == "control" else None
        ),
        "candidate_path_conformant": (
            intent_calls > 0
            and materializations > 0
            and intent_rejection_count == 0
            and legacy_calls == 0
            and telemetry.get("intent_contract") == INTENT_CONTRACT
            and telemetry.get("materialization_policy") == MATERIALIZATION_POLICY
            if coordinate == "candidate"
            else None
        ),
        "intent_contract": telemetry.get("intent_contract"),
        "materialization_policy": telemetry.get("materialization_policy"),
        "admission_rejections": dict(admission_rejections),
        "expected_rejection_observed": expected_rejection_observed,
        "finalization_status": finalization.get("status"),
        "false_abstention": false_abstention,
        "missed_target_insufficiency": missed_insufficiency,
        "typed_operational_failure_classes": dict(operational),
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


def summarize_trial(trial):
    cases = trial["cases"]
    complete = all(case.get("operationally_complete") is True for case in cases)
    models = Counter()
    failures = Counter()
    for case in cases:
        models.update(case.get("observed_model_identities") or {})
        failures.update(case.get("typed_operational_failure_classes") or {})
        if case.get("generation_failure_class"):
            failures[f"generation:{case['generation_failure_class']}"] += 1
        if case.get("operational_failure"):
            failures[
                case["operational_failure"].get(
                    "failure_class", "process_failure"
                )
            ] += 1
    return {
        "operationally_complete": complete,
        "planner_success": (
            all(case.get("planner_success") is True for case in cases)
            if complete
            else None
        ),
        "case_count": len(cases),
        "same_key_sibling_exposed_cases": sum(
            bool(case.get("same_key_sibling_exposed")) for case in cases
        ),
        "control_path_exposed_cases": sum(
            case.get("control_path_exposed") is True for case in cases
        ),
        "candidate_path_conformant_cases": sum(
            case.get("candidate_path_conformant") is True for case in cases
        ),
        "legacy_action_planner_calls": sum(
            int(case.get("legacy_action_planner_calls") or 0) for case in cases
        ),
        "intent_planner_calls": sum(
            int(case.get("intent_planner_calls") or 0) for case in cases
        ),
        "harness_intent_materializations": sum(
            int(case.get("harness_intent_materializations") or 0) for case in cases
        ),
        "intent_rejection_count": sum(
            int(case.get("intent_rejection_count") or 0) for case in cases
        ),
        "action_rejection_count": sum(
            int(case.get("action_rejection_count") or 0) for case in cases
        ),
        "correctness_boundary_violations": sum(
            int(case.get("correctness_boundary_violations") or 0) for case in cases
        ),
        "provider_calls_observed": sum(
            int(case.get("provider_calls_observed") or 0) for case in cases
        ),
        "provider_attempts_observed": sum(
            int(case.get("provider_attempts_observed") or 0) for case in cases
        ),
        "tokens_observed": sum(
            int(case.get("tokens_observed") or 0) for case in cases
        ),
        "provider_latency_ms_observed": sum(
            int(case.get("provider_latency_ms_observed") or 0) for case in cases
        ),
        "wall_clock_ms": sum(
            int(case.get("wall_clock_ms") or 0) for case in cases
        ),
        "observed_model_identities": dict(models),
        "operational_failure_classes": dict(failures),
    }


def aggregate(trials, manifest):
    complete = [
        trial for trial in trials
        if trial["summary"]["operationally_complete"] is True
    ]
    successes = [
        bool(trial["summary"]["planner_success"]) for trial in complete
    ]
    all_k = manifest["trial_plan"]["all_k_group"]
    grouped = [
        trial for trial in complete if trial["seed"] in all_k["seeds"]
    ]
    all_k_defined = len(grouped) == len(all_k["seeds"])
    return {
        "attempted_trials": len(trials),
        "complete_trials": len(complete),
        "trial_completion_rate": (
            len(complete) / len(trials) if trials else 0.0
        ),
        "planner_successes": sum(successes),
        "planner_success_denominator": len(successes),
        "pass_at_1": (
            sum(successes) / len(successes) if successes else None
        ),
        "all_k": {
            "k": all_k["k"],
            "defined": all_k_defined,
            "observed_all_k_success": (
                all(trial["summary"]["planner_success"] is True for trial in grouped)
                if all_k_defined
                else None
            ),
            "derived_independence_p_to_k": None,
        },
        "same_key_sibling_exposed_cases": sum(
            trial["summary"]["same_key_sibling_exposed_cases"]
            for trial in complete
        ),
        "control_path_exposed_cases": sum(
            trial["summary"]["control_path_exposed_cases"]
            for trial in complete
        ),
        "candidate_path_conformant_cases": sum(
            trial["summary"]["candidate_path_conformant_cases"]
            for trial in complete
        ),
        "legacy_action_planner_calls": sum(
            trial["summary"]["legacy_action_planner_calls"]
            for trial in complete
        ),
        "intent_planner_calls": sum(
            trial["summary"]["intent_planner_calls"]
            for trial in complete
        ),
        "harness_intent_materializations": sum(
            trial["summary"]["harness_intent_materializations"]
            for trial in complete
        ),
        "intent_rejection_count": sum(
            trial["summary"]["intent_rejection_count"]
            for trial in complete
        ),
        "action_rejection_count": sum(
            trial["summary"]["action_rejection_count"]
            for trial in complete
        ),
        "total_correctness_boundary_violations_all_attempts": sum(
            trial["summary"]["correctness_boundary_violations"]
            for trial in trials
        ),
        "provider_calls_observed": sum(
            trial["summary"]["provider_calls_observed"] for trial in trials
        ),
        "provider_attempts_observed": sum(
            trial["summary"]["provider_attempts_observed"] for trial in trials
        ),
        "tokens_observed": sum(
            trial["summary"]["tokens_observed"] for trial in trials
        ),
        "provider_latency_ms_observed": sum(
            trial["summary"]["provider_latency_ms_observed"] for trial in trials
        ),
        "wall_clock_ms": sum(
            trial["summary"]["wall_clock_ms"] for trial in trials
        ),
    }


def paired_acceptance(control_trials, candidate_trials, manifest):
    control = aggregate(control_trials, manifest)
    candidate = aggregate(candidate_trials, manifest)
    policy = manifest["acceptance"]
    expected_cases = (
        policy["min_complete_trials_each_coordinate"] * len(manifest["cases"])
    )
    checks = {
        "control_complete_trials": (
            control["complete_trials"]
            >= policy["min_complete_trials_each_coordinate"]
        ),
        "candidate_complete_trials": (
            candidate["complete_trials"]
            >= policy["min_complete_trials_each_coordinate"]
        ),
        "control_correctness_zero": (
            control["total_correctness_boundary_violations_all_attempts"]
            <= policy["max_correctness_boundary_violations_each_coordinate"]
        ),
        "candidate_correctness_zero": (
            candidate["total_correctness_boundary_violations_all_attempts"]
            <= policy["max_correctness_boundary_violations_each_coordinate"]
        ),
        "same_key_sibling_exposure_control": (
            not policy["require_same_key_sibling_exposure_all_complete_cases"]
            or control["same_key_sibling_exposed_cases"] >= expected_cases
        ),
        "same_key_sibling_exposure_candidate": (
            not policy["require_same_key_sibling_exposure_all_complete_cases"]
            or candidate["same_key_sibling_exposed_cases"] >= expected_cases
        ),
        "control_legacy_action_path_exposed": (
            not policy["require_control_legacy_action_path_all_complete_cases"]
            or control["control_path_exposed_cases"] >= expected_cases
        ),
        "candidate_materialization_path_conformant": (
            not policy["require_candidate_materialization_path_all_complete_cases"]
            or candidate["candidate_path_conformant_cases"] >= expected_cases
        ),
        "candidate_legacy_action_path_eliminated": (
            not policy["require_candidate_legacy_action_calls_zero"]
            or candidate["legacy_action_planner_calls"] == 0
        ),
        "candidate_intent_calls_each_trial": (
            not policy["require_candidate_intent_calls_positive_each_trial"]
            or all(
                trial["summary"]["intent_planner_calls"] > 0
                for trial in candidate_trials
                if trial["summary"]["operationally_complete"] is True
            )
        ),
        "candidate_materializations_each_trial": (
            not policy["require_candidate_materializations_positive_each_trial"]
            or all(
                trial["summary"]["harness_intent_materializations"] > 0
                for trial in candidate_trials
                if trial["summary"]["operationally_complete"] is True
            )
        ),
        "candidate_intent_rejections_zero": (
            not policy["require_candidate_intent_rejections_zero"]
            or candidate["intent_rejection_count"] == 0
        ),
        "candidate_action_rejections_zero": (
            not policy["require_candidate_action_rejections_zero"]
            or candidate["action_rejection_count"] == 0
        ),
        "candidate_legacy_calls_strictly_lower": (
            not policy["require_candidate_legacy_calls_strictly_lower_than_control"]
            or candidate["legacy_action_planner_calls"]
            < control["legacy_action_planner_calls"]
        ),
        "candidate_planner_success_requirement": (
            candidate["planner_successes"]
            >= policy["candidate_planner_success_trials_required"]
            and candidate["planner_success_denominator"]
            >= policy["candidate_planner_success_trials_required"]
        ),
        "candidate_planner_success_not_below_control": (
            not policy["require_candidate_planner_success_not_below_control"]
            or (
                control["pass_at_1"] is not None
                and candidate["pass_at_1"] is not None
                and candidate["pass_at_1"] >= control["pass_at_1"]
            )
        ),
    }
    return {
        "control": control,
        "candidate": candidate,
        "checks": checks,
        "acceptance_passed": all(checks.values()),
    }


def validate_corpus(root):
    manifest = load_json(root / "manifest.json")
    expected_identity = {
        "schema_version": "action-materialization-manifest-v1",
        "corpus_identity": CORPUS_ID,
        "evaluator_identity": EVALUATOR_ID,
        "scoring_identity": SCORING_ID,
        "purpose": "adoption_holdout_for_issue_283",
    }
    for key, value in expected_identity.items():
        if manifest.get(key) != value:
            raise EvalError(f"manifest {key} drift")

    dependencies = manifest.get("evaluator_dependencies") or {}
    expected_dep = dependencies.get("scripts/planner_reliability_v1.py")
    dep_path = Path(__file__).resolve().parent / "planner_reliability_v1.py"
    import hashlib
    observed_dep = hashlib.sha256(dep_path.read_bytes()).hexdigest()
    if expected_dep != observed_dep:
        raise EvalError("planner reliability helper dependency drift")

    coordinates = manifest.get("coordinates") or {}
    if (coordinates.get("control") or {}).get("commit") != CONTROL_COMMIT:
        raise EvalError("control commit drift")
    candidate = coordinates.get("candidate") or {}
    if candidate.get("commit") != CANDIDATE_COMMIT:
        raise EvalError("candidate commit drift")
    if candidate.get("intent_contract") != INTENT_CONTRACT:
        raise EvalError("candidate intent contract drift")
    if candidate.get("materialization_policy") != MATERIALIZATION_POLICY:
        raise EvalError("candidate materialization policy drift")

    expected_providers = {
        "mistral": {
            "provider": "mistral",
            "model": "ministral-8b-latest",
            "max_tokens": 1024,
            "inter_invocation_delay_ms": 1500,
        },
        "google": {
            "provider": "google",
            "model": "gemini-3.5-flash-lite",
            "max_tokens": 1024,
            "inter_invocation_delay_ms": 1500,
        },
    }
    if manifest.get("provider_targets") != expected_providers:
        raise EvalError("provider target drift")

    seeds = [97211, 97212, 97213, 97214, 97215]
    trial_plan = manifest.get("trial_plan") or {}
    if trial_plan.get("primary_trial_seeds") != seeds:
        raise EvalError("trial seed drift")
    if trial_plan.get("declared_k") != [1, 5]:
        raise EvalError("declared k drift")
    if trial_plan.get("all_k_group") != {"k": 5, "seeds": seeds}:
        raise EvalError("all-k group drift")
    if trial_plan.get("paired_coordinate_order") != (
        "trial_index_even:control_then_candidate;"
        "trial_index_odd:candidate_then_control"
    ):
        raise EvalError("paired order drift")

    semantics = manifest.get("measurement_semantics") or {}
    for key in [
        "exercised_path_observability_separate_from_utility",
        "hard_correctness_separate_from_utility",
        "operational_completeness_separate_from_semantic_metrics",
        "operationally_incomplete_trials_excluded_from_semantic_distributions",
        "no_cross_model_averaging",
        "no_majority_vote_authority",
        "provider_internal_attempts_are_operational_not_semantic_trials",
    ]:
        if semantics.get(key) is not True:
            raise EvalError("measurement semantics drift")
    if semantics.get("independence_derived_p_to_k_reported") is not False:
        raise EvalError("independence-derived p^k must remain disabled")

    contract = manifest.get("surface_contract") or {}
    if contract != {
        "case_roles": [
            "grounded_same_key_siblings",
            "stale_same_key_siblings",
        ],
        "minimum_same_key_targets_per_case": 2,
        "exact_read_only_capabilities_per_key": 2,
        "explicit_unique_priority_required": True,
        "same_key_sibling_identity_must_remain_distinct": True,
        "control_stochastic_action_path_required": True,
        "candidate_intent_materialization_path_required": True,
    }:
        raise EvalError("surface contract drift")

    cases = []
    markers = set()
    files = manifest.get("cases") or []
    if len(files) != 2 or len(set(files)) != 2:
        raise EvalError("expected exactly two fresh cases")
    roles = []
    for rel in files:
        case = load_json(root / rel)
        if case.get("schema_version") != "action-materialization-case-v1":
            raise EvalError(f"{rel}: case schema drift")
        if case.get("kind") != "investigation":
            raise EvalError(f"{rel}: investigation only")
        roles.append(case.get("role"))
        target = case.get("target") or {}
        if not target.get("key") or "value" not in target:
            raise EvalError(f"{rel}: target missing")
        if int(case.get("minimum_same_key_targets") or 0) < 2:
            raise EvalError(f"{rel}: sibling minimum missing")
        for marker in case.get("fresh_markers") or []:
            if marker in markers:
                raise EvalError(f"{rel}: duplicate fresh marker {marker}")
            markers.add(marker)

        config = load_json(root / case["config"])
        investigation = (config.get("resolution") or {}).get("investigation") or {}
        capabilities = investigation.get("capabilities") or []
        if len(capabilities) != 2:
            raise EvalError(f"{rel}: expected exactly two capabilities")
        if any(cap.get("read_only") is not True for cap in capabilities):
            raise EvalError(f"{rel}: all capabilities must be read-only")
        if any(
            target["key"] not in (cap.get("supported_fact_keys") or [])
            for cap in capabilities
        ):
            raise EvalError(f"{rel}: capability exact-key drift")
        priorities = [cap.get("selection_priority") for cap in capabilities]
        if any(not isinstance(value, int) for value in priorities):
            raise EvalError(f"{rel}: explicit priority required")
        if len(set(priorities)) != 2:
            raise EvalError(f"{rel}: priority tie forbidden")
        if set(case["relevant_capabilities"]) != {
            cap["id"] for cap in capabilities
        }:
            raise EvalError(f"{rel}: relevant capability drift")
        highest = max(
            capabilities, key=lambda cap: cap["selection_priority"]
        )["id"]
        if case.get("preferred_capability") != highest:
            raise EvalError(f"{rel}: preferred capability drift")
        if case["role"] == "stale_same_key_siblings":
            if case.get("expected") != "unknown":
                raise EvalError(f"{rel}: stale case expectation drift")
            if "stale" not in (case.get("expected_rejection") or []):
                raise EvalError(f"{rel}: stale rejection missing")
        elif case["role"] == "grounded_same_key_siblings":
            if case.get("expected") != "grounded":
                raise EvalError(f"{rel}: grounded expectation drift")
        else:
            raise EvalError(f"{rel}: unknown role")
        cases.append(case)

    if roles != ["grounded_same_key_siblings", "stale_same_key_siblings"]:
        raise EvalError("case role ordering drift")

    repo_root = root.parent.parent
    for marker in sorted(markers):
        for candidate_path in (repo_root / "fixtures").rglob("*.json"):
            if root in candidate_path.parents:
                continue
            try:
                text = candidate_path.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            if marker in text:
                raise EvalError(
                    f"fresh marker reused in {candidate_path.relative_to(repo_root)}"
                )

    expected_acceptance = {
        "min_complete_trials_each_coordinate": 5,
        "max_correctness_boundary_violations_each_coordinate": 0,
        "candidate_planner_success_trials_required": 5,
        "require_same_key_sibling_exposure_all_complete_cases": True,
        "require_control_legacy_action_path_all_complete_cases": True,
        "require_candidate_materialization_path_all_complete_cases": True,
        "require_candidate_legacy_action_calls_zero": True,
        "require_candidate_intent_calls_positive_each_trial": True,
        "require_candidate_materializations_positive_each_trial": True,
        "require_candidate_intent_rejections_zero": True,
        "require_candidate_action_rejections_zero": True,
        "require_candidate_legacy_calls_strictly_lower_than_control": True,
        "require_candidate_planner_success_not_below_control": True,
    }
    if manifest.get("acceptance") != expected_acceptance:
        raise EvalError("acceptance policy drift")
    return manifest, cases


def run_fixture_capability(root, capability):
    repo_root = root.parent
    while not (repo_root / "scripts" / "natural_e2e_fixture_resolver.py").exists():
        if repo_root.parent == repo_root:
            raise EvalError("repository root not found")
        repo_root = repo_root.parent
    completed = subprocess.run(
        [capability["program"], *capability.get("args", [])],
        input=json.dumps(preflight_request()).encode(),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=repo_root,
        check=False,
    )
    if completed.returncode != 0:
        raise EvalError(f"resolver preflight failed {capability['id']}")
    out = json.loads(completed.stdout)
    if out.get("schema_version") != "reason-external-resolver-response-v1":
        raise EvalError(f"resolver schema drift {capability['id']}")
    return out


def self_test_fixtures(root, cases):
    summary = {
        "capabilities_checked": 0,
        "positive_evidence_capabilities": 0,
        "no_result_capabilities": 0,
        "stale_rejection_contracts": 0,
    }
    for case in cases:
        config = load_json(root / case["config"])
        capabilities = config["resolution"]["investigation"]["capabilities"]
        stale_verified = False
        positive_verified = False
        for capability in capabilities:
            out = run_fixture_capability(root, capability)
            summary["capabilities_checked"] += 1
            contribution = out.get("contribution") or {}
            if contribution.get("kind") == "no_result":
                summary["no_result_capabilities"] += 1
                continue
            evidence = contribution.get("evidence") or []
            if contribution.get("kind") != "acquired_evidence" or not evidence:
                raise EvalError(f"{case['id']}: malformed fixture evidence")
            item = evidence[0]
            source = item.get("source")
            admission = capability["admission"]
            source_policy = admission["sources"].get(source)
            if source_policy is None:
                raise EvalError(f"{case['id']}: source not allowlisted")
            metadata = item.get("acquisition_metadata") or {}
            observed = metadata.get("observed_at_unix_seconds")
            evaluation = admission.get("evaluation_time_unix_seconds")
            max_age = source_policy.get("max_age_seconds")
            stale = (
                isinstance(observed, int)
                and isinstance(evaluation, int)
                and isinstance(max_age, int)
                and evaluation - observed > max_age
            )
            if case["role"] == "stale_same_key_siblings":
                stale_verified |= stale
            else:
                if stale:
                    raise EvalError(f"{case['id']}: positive evidence stale")
                positive_verified = True
                summary["positive_evidence_capabilities"] += 1
        if case["role"] == "stale_same_key_siblings":
            if not stale_verified:
                raise EvalError(f"{case['id']}: stale fixture not verified")
            summary["stale_rejection_contracts"] += 1
        elif not positive_verified:
            raise EvalError(f"{case['id']}: no positive evidence")
    return summary


def run_case(root, repo_root, binary, case, seed, provider, env, coordinate):
    command = [
        str(binary),
        case["task"],
        "--provider", provider["provider"],
        "--model", provider["model"],
        "--max-tokens", str(provider["max_tokens"]),
        "--seed", str(seed),
        "--config", str((root / case["config"]).resolve()),
        "--format", "json",
    ]
    completed, payload, elapsed = run_json(command, repo_root, env)
    if (
        completed.returncode != 0
        or not isinstance(payload, dict)
        or "result" not in payload
    ):
        report = invocation_failure(case, completed, payload, elapsed)
    else:
        report = score_case(case, payload["result"], elapsed, coordinate)
    report["case_seed"] = seed
    return report


def run_trial(root, repo_root, binary, cases, seed, provider, env, coordinate):
    reports = []
    delay = provider["inter_invocation_delay_ms"] / 1000
    for index, case in enumerate(cases):
        reports.append(
            run_case(
                root, repo_root, binary, case, seed + index,
                provider, env, coordinate
            )
        )
        if index + 1 < len(cases):
            time.sleep(delay)
    trial = {
        "trial_id": f"{coordinate}-seed-{seed}",
        "coordinate": coordinate,
        "seed": seed,
        "cases": reports,
    }
    trial["summary"] = summarize_trial(trial)
    return trial


def run_paired(root, repo_root, control_bin, candidate_bin, cases, manifest, provider, env):
    control_trials = []
    candidate_trials = []
    delay = provider["inter_invocation_delay_ms"] / 1000
    for index, seed in enumerate(manifest["trial_plan"]["primary_trial_seeds"]):
        order = (
            ("control", "candidate")
            if index % 2 == 0
            else ("candidate", "control")
        )
        pair = {}
        for coordinate in order:
            binary = control_bin if coordinate == "control" else candidate_bin
            pair[coordinate] = run_trial(
                root, repo_root, binary, cases, seed, provider, env, coordinate
            )
            time.sleep(delay)
        control_trials.append(pair["control"])
        candidate_trials.append(pair["candidate"])
    return control_trials, candidate_trials


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", default="fixtures/action-materialization-v1")
    parser.add_argument("--control-bin")
    parser.add_argument("--candidate-bin")
    parser.add_argument("--provider-target", choices=["mistral", "google"])
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--attempt-marker")
    parser.add_argument("--output")
    args = parser.parse_args()

    root = Path(args.fixtures).resolve()
    manifest, cases = validate_corpus(root)

    if args.validate_only:
        print(json.dumps({
            "schema_version": REPORT_SCHEMA,
            "corpus_identity": CORPUS_ID,
            "evaluator_identity": EVALUATOR_ID,
            "scoring_identity": SCORING_ID,
            "valid": True,
            "live_observation_performed": False,
            "control_commit": CONTROL_COMMIT,
            "candidate_commit": CANDIDATE_COMMIT,
            "providers": sorted(manifest["provider_targets"]),
            "trials_each_coordinate": len(
                manifest["trial_plan"]["primary_trial_seeds"]
            ),
            "cases_per_trial": len(cases),
        }, indent=2, sort_keys=True))
        return 0

    if args.preflight:
        print(json.dumps({
            "schema_version": REPORT_SCHEMA,
            "corpus_identity": CORPUS_ID,
            "valid": True,
            "live_observation_performed": False,
            "fixture_preflight": self_test_fixtures(root, cases),
        }, indent=2, sort_keys=True))
        return 0

    if not args.control_bin or not args.candidate_bin or not args.provider_target:
        raise EvalError("live mode requires both binaries and provider target")
    control_bin = Path(args.control_bin).resolve()
    candidate_bin = Path(args.candidate_bin).resolve()
    if not control_bin.is_file() or not candidate_bin.is_file():
        raise EvalError("control/candidate binary not found")

    provider = manifest["provider_targets"][args.provider_target]
    repo_root = Path(__file__).resolve().parents[1]
    env = os.environ.copy()

    marker = {
        "schema_version": "action-materialization-live-attempt-v1",
        "corpus_identity": CORPUS_ID,
        "provider_target": args.provider_target,
        "provider": provider["provider"],
        "model": provider["model"],
        "control_commit": CONTROL_COMMIT,
        "candidate_commit": CANDIDATE_COMMIT,
        "first_live_launch_boundary_entered": True,
        "started_at_utc": datetime.now(timezone.utc).isoformat(),
        "github_run_id": os.environ.get("GITHUB_RUN_ID"),
        "github_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
    }
    if args.attempt_marker:
        Path(args.attempt_marker).write_text(
            json.dumps(marker, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    control_trials, candidate_trials = run_paired(
        root, repo_root, control_bin, candidate_bin,
        cases, manifest, provider, env
    )
    paired = paired_acceptance(control_trials, candidate_trials, manifest)

    control_models = Counter()
    candidate_models = Counter()
    for trial in control_trials:
        control_models.update(trial["summary"]["observed_model_identities"])
    for trial in candidate_trials:
        candidate_models.update(trial["summary"]["observed_model_identities"])

    out = {
        "schema_version": REPORT_SCHEMA,
        "corpus_identity": CORPUS_ID,
        "evaluator_identity": EVALUATOR_ID,
        "scoring_identity": SCORING_ID,
        "live_observation_performed": True,
        "provider_target": args.provider_target,
        "provider": provider["provider"],
        "requested_model": provider["model"],
        "observed_model_identities": {
            "control": dict(control_models),
            "candidate": dict(candidate_models),
        },
        "observed_at_utc": datetime.now(timezone.utc).isoformat(),
        "github_run_id": os.environ.get("GITHUB_RUN_ID"),
        "github_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        "coordinates": manifest["coordinates"],
        "trial_plan": manifest["trial_plan"],
        "measurement_semantics": manifest["measurement_semantics"],
        "planner_success_predicate": manifest["planner_success_predicate"],
        "acceptance_policy": manifest["acceptance"],
        "control_trials": control_trials,
        "candidate_trials": candidate_trials,
        "paired_summary": paired,
        "acceptance_passed": paired["acceptance_passed"],
    }
    text = json.dumps(out, indent=2, sort_keys=True)
    print(text)
    if args.output:
        Path(args.output).write_text(text + "\n", encoding="utf-8")
    return 0 if out["acceptance_passed"] else 3


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except EvalError as error:
        print(f"evaluation error: {error}", file=sys.stderr)
        raise SystemExit(2)
