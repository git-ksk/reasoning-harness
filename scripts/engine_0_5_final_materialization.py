#!/usr/bin/env python3
"""Fresh candidate-only materialization acceptance for Harness Engine 0.5.0."""
from __future__ import annotations

import argparse
import hashlib
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

REPORT_SCHEMA = "reason-engine-0.5-final-v1-materialization"
CORPUS_ID = "engine-0.5-final-v1-materialization"
EVALUATOR_ID = "reason-engine-0.5-final-v1-materialization"
SCORING_ID = "engine-0.5-final-v1-materialization-scoring-v1"
PRECHANGE_COMMIT = "94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e"
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

    action_path_success = None
    if operationally_complete:
        action_path_success = bool(
            correctness == 0
            and target_recalled
            and sibling_exposed
            and tool_selected
            and action_rejection_count == 0
            and irrelevant == 0
            and expected_rejection_observed is not False
        )

    return {
        "id": case["id"],
        "role": case["role"],
        "expected": expected,
        "target": target,
        "operationally_complete": operationally_complete,
        "action_path_success": action_path_success,
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
    finalization_statuses = Counter()
    for case in cases:
        models.update(case.get("observed_model_identities") or {})
        status = case.get("finalization_status")
        if status:
            finalization_statuses[status] += 1
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
        "action_path_success": (
            all(case.get("action_path_success") is True for case in cases)
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
        "downstream_finalization": {
            "status_counts": dict(finalization_statuses),
            "false_abstention_cases": sum(
                int(case.get("false_abstention") or 0) for case in cases
            ),
            "blocked_unverified_propositions": sum(
                int(case.get("blocked_unverified_propositions") or 0)
                for case in cases
            ),
        },
    }



EXPECTED_TARGETS = {
    "mistral-8b": ("mistral", "ministral-8b-latest", "validated_required"),
    "mistral-14b": ("mistral", "ministral-14b-latest", "observed_characterization"),
    "google-gemini-3.5-flash-lite": ("google", "gemini-3.5-flash-lite", "validated_required"),
    "google-gemma-4-31b-it": ("google", "gemma-4-31b-it", "validated_required"),
    "groq-gpt-oss-120b": ("groq", "openai/gpt-oss-120b", "validated_required"),
    "groq-qwen3.8-27b": ("groq", "qwen/qwen3.8-27b", "observed_characterization"),
    "groq-gpt-oss-20b": ("groq", "openai/gpt-oss-20b", "observed_characterization"),
    "nvidia-nemotron": ("nvidia", "nvidia/nemotron-3.5-lightning-30b-a3b", "limited_negative_control"),
}

def validate_corpus(root):
    manifest = load_json(root / "manifest.json")
    expected_identity = {
        "schema_version": "engine-0.5-final-materialization-manifest-v1",
        "corpus_identity": CORPUS_ID,
        "evaluator_identity": EVALUATOR_ID,
        "scoring_identity": SCORING_ID,
        "product_commit": CANDIDATE_COMMIT,
        "prechange_commit": PRECHANGE_COMMIT,
    }
    for key, value in expected_identity.items():
        if manifest.get(key) != value:
            raise EvalError(f"manifest {key} drift")

    dependencies = manifest.get("evaluator_dependencies") or {}
    expected_dep = dependencies.get("scripts/planner_reliability_v1.py")
    dep_path = Path(__file__).resolve().parent / "planner_reliability_v1.py"
    observed_dep = hashlib.sha256(dep_path.read_bytes()).hexdigest()
    if expected_dep != observed_dep:
        raise EvalError("planner reliability helper dependency drift")

    targets = manifest.get("provider_targets") or {}
    if set(targets) != set(EXPECTED_TARGETS):
        raise EvalError("provider target set drift")
    for target_id, (provider, model, role) in EXPECTED_TARGETS.items():
        policy = targets[target_id]
        if policy.get("provider") != provider or policy.get("model") != model or policy.get("role") != role:
            raise EvalError(f"provider target drift: {target_id}")
        if int(policy.get("base_seed") or 0) != 771211:
            raise EvalError(f"seed drift: {target_id}")
        if int(policy.get("max_tokens") or 0) != 1024:
            raise EvalError(f"max token drift: {target_id}")
        if int(policy.get("inter_case_delay_ms") or 0) <= 0:
            raise EvalError(f"invalid pacing: {target_id}")

    if manifest.get("trial_plan") != {
        "primary_trial_seeds": [771211],
        "case_seed_derivation": "trial_seed + zero_based_case_index",
    }:
        raise EvalError("trial plan drift")

    contract = manifest.get("surface_contract") or {}
    if contract != {
        "case_roles": ["grounded_same_key_siblings", "stale_same_key_siblings"],
        "minimum_same_key_targets_per_case": 2,
        "exact_read_only_capabilities_per_key": 2,
        "explicit_unique_priority_required": True,
        "same_key_sibling_identity_must_remain_distinct": True,
        "candidate_intent_materialization_path_required": True,
    }:
        raise EvalError("surface contract drift")

    expected_acceptance = {
        "complete_cases_required": 2,
        "max_correctness_boundary_violations": 0,
        "require_same_key_sibling_exposure_all_cases": True,
        "require_candidate_materialization_path_all_cases": True,
        "require_candidate_legacy_action_calls_zero": True,
        "require_candidate_intent_calls_positive": True,
        "require_candidate_materializations_positive": True,
        "require_intent_rejections_zero": True,
        "require_action_rejections_zero": True,
    }
    if manifest.get("acceptance") != expected_acceptance:
        raise EvalError("acceptance policy drift")

    cases = []
    markers = set()
    files = manifest.get("cases") or []
    if len(files) != 2 or len(set(files)) != 2:
        raise EvalError("expected exactly two fresh cases")
    roles = []
    for rel in files:
        case = load_json(root / rel)
        if case.get("schema_version") != "engine-0.5-final-materialization-case-v1":
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
        if any((cap.get("args") or [None])[0] != "scripts/engine_0_5_final_materialization_resolver.py" for cap in capabilities):
            raise EvalError(f"{rel}: dedicated target-aware resolver required")
        if any(target["key"] not in (cap.get("supported_fact_keys") or []) for cap in capabilities):
            raise EvalError(f"{rel}: capability exact-key drift")
        priorities = [cap.get("selection_priority") for cap in capabilities]
        if any(not isinstance(value, int) for value in priorities) or len(set(priorities)) != 2:
            raise EvalError(f"{rel}: explicit unique priority required")
        if set(case["relevant_capabilities"]) != {cap["id"] for cap in capabilities}:
            raise EvalError(f"{rel}: relevant capability drift")
        highest = max(capabilities, key=lambda cap: cap["selection_priority"])["id"]
        if case.get("preferred_capability") != highest:
            raise EvalError(f"{rel}: preferred capability drift")
        if case["role"] == "stale_same_key_siblings":
            if case.get("expected") != "unknown" or "stale" not in (case.get("expected_rejection") or []):
                raise EvalError(f"{rel}: stale expectation drift")
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
                raise EvalError(f"fresh marker reused in {candidate_path.relative_to(repo_root)}")
    return manifest, cases
def fixture_request(target_id):
    request = preflight_request()
    request["request"]["id"] = f"preflight:{target_id}"
    request["request"]["target"]["target_id"] = target_id
    return request


def run_fixture_capability(root, capability, target_id="preflight"):
    repo_root = root.parent
    while not (repo_root / "scripts" / "engine_0_5_final_materialization_resolver.py").exists():
        if repo_root.parent == repo_root:
            raise EvalError("repository root not found")
        repo_root = repo_root.parent
    completed = subprocess.run(
        [capability["program"], *capability.get("args", [])],
        input=json.dumps(fixture_request(target_id)).encode(),
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
        "target_aware_identity_checks": 0,
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

            sibling_a = run_fixture_capability(root, capability, "sibling-a")
            sibling_b = run_fixture_capability(root, capability, "sibling-b")
            sibling_a_repeat = run_fixture_capability(root, capability, "sibling-a")

            def evidence_id(response):
                contribution = response.get("contribution") or {}
                items = contribution.get("evidence") or []
                return items[0].get("id") if items else None

            id_a = evidence_id(sibling_a)
            id_b = evidence_id(sibling_b)
            id_a_repeat = evidence_id(sibling_a_repeat)
            if not id_a or not id_b or id_a == id_b:
                raise EvalError(
                    f"{case['id']}:{capability['id']}: target-aware evidence identity failed"
                )
            if id_a != id_a_repeat:
                raise EvalError(
                    f"{case['id']}:{capability['id']}: same-target evidence identity not idempotent"
                )
            summary["target_aware_identity_checks"] += 1

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
    delay = provider["inter_case_delay_ms"] / 1000
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



def candidate_acceptance(summary, policy):
    checks = {
        "operationally_complete": summary["operationally_complete"] is True,
        "action_path_success": summary["action_path_success"] is True,
        "correctness_zero": summary["correctness_boundary_violations"] <= policy["max_correctness_boundary_violations"],
        "same_key_siblings_all_cases": summary["same_key_sibling_exposed_cases"] == policy["complete_cases_required"],
        "materialization_path_all_cases": summary["candidate_path_conformant_cases"] == policy["complete_cases_required"],
        "legacy_action_calls_zero": summary["legacy_action_planner_calls"] == 0,
        "intent_calls_positive": summary["intent_planner_calls"] > 0,
        "materializations_positive": summary["harness_intent_materializations"] > 0,
        "intent_rejections_zero": summary["intent_rejection_count"] == 0,
        "action_rejections_zero": summary["action_rejection_count"] == 0,
    }
    return {"checks": checks, "acceptance_passed": all(checks.values())}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixtures", default="fixtures/engine-0.5-final-v1-materialization")
    parser.add_argument("--candidate-bin")
    parser.add_argument("--target", choices=sorted(EXPECTED_TARGETS))
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
            "candidate_commit": CANDIDATE_COMMIT,
            "providers": sorted(manifest["provider_targets"]),
            "trials": 1,
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

    if not args.candidate_bin or not args.target:
        raise EvalError("live mode requires candidate binary and target")
    candidate_bin = Path(args.candidate_bin).resolve()
    if not candidate_bin.is_file():
        raise EvalError("candidate binary not found")

    provider = manifest["provider_targets"][args.target]
    repo_root = Path(__file__).resolve().parents[1]
    env = os.environ.copy()
    seed = int(provider["base_seed"])

    marker = {
        "schema_version": "engine-0.5-final-materialization-attempt-v1",
        "corpus_identity": CORPUS_ID,
        "provider_target": args.target,
        "provider": provider["provider"],
        "model": provider["model"],
        "candidate_commit": CANDIDATE_COMMIT,
        "first_live_launch_boundary_entered": True,
        "started_at_utc": datetime.now(timezone.utc).isoformat(),
        "github_run_id": os.environ.get("GITHUB_RUN_ID"),
        "github_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
    }
    if args.attempt_marker:
        Path(args.attempt_marker).write_text(json.dumps(marker, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    trial = run_trial(root, repo_root, candidate_bin, cases, seed, provider, env, "candidate")
    summary = trial["summary"]
    policy = manifest["acceptance"]
    acceptance = candidate_acceptance(summary, policy)
    checks = acceptance["checks"]
    acceptance_passed = acceptance["acceptance_passed"]
    out = {
        "schema_version": REPORT_SCHEMA,
        "corpus_identity": CORPUS_ID,
        "evaluator_identity": EVALUATOR_ID,
        "scoring_identity": SCORING_ID,
        "product_commit": CANDIDATE_COMMIT,
        "prechange_commit": PRECHANGE_COMMIT,
        "live_observation_performed": True,
        "provider_target": args.target,
        "provider": provider["provider"],
        "requested_model": provider["model"],
        "role": provider["role"],
        "observed_model_identities": summary["observed_model_identities"],
        "observed_at_utc": datetime.now(timezone.utc).isoformat(),
        "github_run_id": os.environ.get("GITHUB_RUN_ID"),
        "github_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
        "trial": trial,
        "summary": summary,
        "checks": checks,
        "acceptance_passed": acceptance_passed,
    }
    text = json.dumps(out, indent=2, sort_keys=True)
    print(text)
    if args.output:
        Path(args.output).write_text(text + "\n", encoding="utf-8")
    return 0 if acceptance_passed else 3


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except EvalError as error:
        print(f"evaluation error: {error}", file=sys.stderr)
        raise SystemExit(2)
