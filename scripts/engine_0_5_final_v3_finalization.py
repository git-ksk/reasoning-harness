#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
import tempfile
import time
from pathlib import Path

CORPUS_ID = "engine-0.5-final-v3-finalization"
EVALUATOR_ID = "reason-engine-0.5-final-v3-finalization"
SCORING_ID = "engine-0.5-final-v3-scoring-v1"
REPORT_SCHEMA = "reason-engine-0.5-final-v3-finalization"
CASE_SCHEMA = "engine-0.5-finalization-case-v3"
MANIFEST_SCHEMA = "engine-0.5-finalization-manifest-v3"
PRODUCT_COMMIT = "d60b9afdf0bb2a0c1986f8c8f7cb47e534a4cd90"
CONTROL_COMMIT = "12292b92bcd7890b3a81fc53b2523d172c2bde3a"
EXPECTED_TARGETS = {
    "mistral-8b": ("mistral", "ministral-8b-latest", "validated_reference"),
    "mistral-14b": ("mistral", "ministral-14b-latest", "affected_required"),
    "google-gemini-3.5-flash-lite": ("google", "gemini-3.5-flash-lite", "validated_reference"),
    "google-gemma-4-31b-it": ("google", "gemma-4-31b-it", "validated_reference"),
    "groq-gpt-oss-120b": ("groq", "openai/gpt-oss-120b", "validated_reference"),
    "groq-qwen3.8-27b": ("groq", "qwen/qwen3.8-27b", "affected_required"),
}
CANON_GROUNDED = re.compile(r"^([^;=()]+?) = (.+)$")
CANON_UNCERTAIN = re.compile(r"^uncertain\(([^;=()]+?) = (.+)\)$")


class EvalError(RuntimeError):
    pass


def load_json(path):
    return json.loads(Path(path).read_text())


def root_path():
    return Path(__file__).resolve().parents[1]


def corpus_root():
    return root_path() / "fixtures" / CORPUS_ID


def validate_corpus():
    root = corpus_root()
    manifest = load_json(root / "manifest.json")
    if manifest.get("schema_version") != MANIFEST_SCHEMA:
        raise EvalError("manifest schema drift")
    for key, expected in (
        ("corpus_identity", CORPUS_ID),
        ("evaluator_identity", EVALUATOR_ID),
        ("scoring_identity", SCORING_ID),
        ("product_commit", PRODUCT_COMMIT),
        ("control_commit", CONTROL_COMMIT),
    ):
        if manifest.get(key) != expected:
            raise EvalError(f"manifest {key} drift")

    expected_policy = {
        "surface_must_be_frozen_before_first_live_observation": True,
        "first_live_launch_per_target_is_canonical": True,
        "workflow_rerun_forbidden": True,
        "any_failed_workflow_requires_fresh_successor_identity": True,
        "historical_frozen_evidence_immutable": True,
        "no_cross_model_averaging": True,
    }
    if manifest.get("live_observation_policy") != expected_policy:
        raise EvalError("live observation policy drift")
    expected_acceptance = {
        "all_six_rows_required_independently": True,
        "grounded_finalization_does_not_require_planner_target_recall": True,
        "target_recall_remains_telemetry": True,
        "grounded_investigation_requires_harness_owned_materialization": True,
        "session_start_requires_unique_persisted_explicit_fact_identity": True,
        "session_correction_requires_exact_grounding": True,
        "session_correction_requires_harness_owned_materialization": True,
        "max_correctness_boundary_violations": 0,
        "max_external_calls_replayed": 0,
    }
    if manifest.get("acceptance") != expected_acceptance:
        raise EvalError("acceptance policy drift")

    targets = manifest.get("provider_targets") or {}
    if set(targets) != set(EXPECTED_TARGETS):
        raise EvalError("provider target set drift")
    for target_id, (provider, model, role) in EXPECTED_TARGETS.items():
        policy = targets[target_id]
        if (
            policy.get("provider") != provider
            or policy.get("model") != model
            or policy.get("role") != role
        ):
            raise EvalError(f"provider target drift: {target_id}")
        if int(policy.get("base_seed") or 0) != 823511:
            raise EvalError(f"seed drift: {target_id}")
        if int(policy.get("max_tokens") or 0) != 1024:
            raise EvalError(f"max token drift: {target_id}")
        if int(policy.get("inter_case_delay_ms") or 0) <= 0:
            raise EvalError(f"invalid pacing: {target_id}")

    files = manifest.get("case_files") or []
    if len(files) != 3 or len(set(files)) != 3:
        raise EvalError("expected exactly three unique fresh cases")
    cases = []
    kinds = []
    markers = set()
    for filename in files:
        case = load_json(root / filename)
        if case.get("schema_version") != CASE_SCHEMA:
            raise EvalError(f"{filename}: case schema drift")
        if not case.get("id") or not case.get("task"):
            raise EvalError(f"{filename}: missing identity/task")
        target = case.get("target") or {}
        if not target.get("key") or target.get("value") is None:
            raise EvalError(f"{filename}: missing exact target")
        for marker in case.get("fresh_markers") or []:
            if marker in markers:
                raise EvalError(f"{filename}: duplicate fresh marker {marker}")
            markers.add(marker)

        if case.get("kind") == "investigation":
            if "hypothesis" in case or "--hypothesis" in json.dumps(case):
                raise EvalError(f"{filename}: explicit hypothesis forbidden")
            config_path = root / case.get("config", "")
            if not config_path.is_file():
                raise EvalError(f"{filename}: missing config")
            config = load_json(config_path)
            investigation = (config.get("resolution") or {}).get("investigation") or {}
            capabilities = investigation.get("capabilities") or []
            if len(capabilities) != 1:
                raise EvalError(f"{filename}: expected one exact capability")
            capability = capabilities[0]
            if capability.get("read_only") is not True:
                raise EvalError(f"{filename}: capability must be read-only")
            if capability.get("supported_fact_keys") != [target["key"]]:
                raise EvalError(f"{filename}: exact-key capability drift")
            if case.get("relevant_capabilities") != [capability.get("id")]:
                raise EvalError(f"{filename}: relevant capability drift")
            args = capability.get("args") or []
            if "scripts/natural_e2e_fixture_resolver.py" not in args:
                raise EvalError(f"{filename}: fixture resolver drift")
            expected_source = next(
                (marker for marker in case.get("fresh_markers") or [] if marker.startswith("fixture:")),
                None,
            )
            if expected_source is None or expected_source not in args:
                raise EvalError(f"{filename}: fresh source drift")
            sources = ((capability.get("admission") or {}).get("sources") or {})
            if set(sources) != {expected_source}:
                raise EvalError(f"{filename}: admission source drift")
            if case.get("expected") == "grounded":
                if "--mode" not in args or "evidence" not in args:
                    raise EvalError(f"{filename}: grounded resolver mode drift")
                if target["key"] not in args or str(target["value"]) not in args:
                    raise EvalError(f"{filename}: grounded exact fact drift")
            elif case.get("expected") == "unknown":
                if "--mode" not in args or "no-result" not in args:
                    raise EvalError(f"{filename}: fail-closed resolver mode drift")
            else:
                raise EvalError(f"{filename}: unsupported investigation expectation")
        elif case.get("kind") == "session_correct":
            if not case.get("start_fact") or not case.get("correction"):
                raise EvalError(f"{filename}: missing session correction inputs")
            start_key, start_value = case["start_fact"].split("=", 1)
            correction_key, correction_value = case["correction"].split("=", 1)
            if start_key != target["key"] or correction_key != target["key"]:
                raise EvalError(f"{filename}: session key identity drift")
            if correction_value != str(target["value"]) or start_value == correction_value:
                raise EvalError(f"{filename}: session correction value drift")
        else:
            raise EvalError(f"{filename}: unsupported case kind")
        kinds.append(case["kind"])
        cases.append(case)

    if kinds != ["investigation", "investigation", "session_correct"]:
        raise EvalError("case ordering/kinds drift")
    return manifest, cases

def run_json(cmd, cwd):
    started = time.monotonic()
    cp = subprocess.run(
        cmd,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        cwd=cwd,
    )
    elapsed = int((time.monotonic() - started) * 1000)
    payload = None
    if cp.stdout.strip():
        try:
            payload = json.loads(cp.stdout)
        except json.JSONDecodeError:
            pass
    if cp.returncode != 0 or not isinstance(payload, dict):
        failure = ((payload or {}).get("result") or {}).get("failure") or {}
        raise EvalError(
            f"command failed exit={cp.returncode} class={failure.get('failure_class')} "
            f"message={failure.get('message') or cp.stderr.decode(errors='replace')[:500]}"
        )
    return payload, elapsed


def final_artifact(result):
    if result.get("output_contract") != "reason-natural-output-v4":
        raise EvalError("natural output contract drift")
    outcome = result.get("final_outcome") or {}
    artifact = outcome.get("artifact")
    if not isinstance(artifact, dict):
        raise EvalError("missing final artifact")
    return artifact


def proposition_supported(artifact, target):
    for claim in artifact.get("claims") or []:
        proposition = claim.get("proposition") or {}
        if (
            proposition.get("key") == target["key"]
            and str(proposition.get("value")) == str(target["value"])
            and claim.get("state") in ("known", "supported")
        ):
            return True
    return False


def exposed_assertions(finalization):
    text = finalization.get("text")
    if text is None:
        return [], 0
    assertions = []
    invalid = 0
    for segment in text.split("; "):
        match = CANON_UNCERTAIN.match(segment)
        if match:
            assertions.append((match.group(1).strip(), match.group(2).strip(), "uncertain"))
            continue
        match = CANON_GROUNDED.match(segment)
        if match:
            assertions.append((match.group(1).strip(), match.group(2).strip(), "grounded"))
            continue
        invalid += 1
    return assertions, invalid


def target_exposure(finalization, artifact, target):
    assertions, invalid = exposed_assertions(finalization)
    grounded = False
    unsupported = invalid
    for key, value, mode in assertions:
        exact = {"key": key, "value": value}
        if mode == "grounded" and not proposition_supported(artifact, exact):
            unsupported += 1
        if key == target["key"] and value == str(target["value"]) and mode == "grounded":
            grounded = True
    return grounded, unsupported, assertions


def harness_owned_investigation_target_supported(artifact, target):
    for claim in artifact.get("claims") or []:
        proposition = claim.get("proposition") or {}
        if (
            str(claim.get("id") or "").startswith("harness_investigation_admitted_fact_")
            and proposition.get("key") == target["key"]
            and str(proposition.get("value")) == str(target["value"])
            and claim.get("state") in ("known", "supported")
        ):
            return True
    return False


def investigation_result(case, result, elapsed):
    target = case["target"]
    artifact = final_artifact(result)
    finalization = result.get("finalization") or {}
    telemetry = ((result.get("investigation") or {}).get("telemetry") or {})
    targets = telemetry.get("targets") or []
    actions = telemetry.get("actions") or []
    target_recalled = any(t.get("expected_fact_key") == target["key"] for t in targets)
    admitted = sum(int(a.get("admitted_evidence") or 0) for a in actions)
    grounded, unsupported, assertions = target_exposure(finalization, artifact, target)
    hypotheses_empty = (artifact.get("hypotheses") or []) == []
    exact_supported = proposition_supported(artifact, target)
    harness_materialized = harness_owned_investigation_target_supported(artifact, target)
    if case["expected"] == "grounded":
        passed = (
            hypotheses_empty
            and admitted >= 1
            and exact_supported
            and harness_materialized
            and grounded
            and unsupported == 0
        )
    else:
        passed = (
            hypotheses_empty
            and not grounded
            and unsupported == 0
            and not exact_supported
        )
    return {
        "id": case["id"],
        "kind": case["kind"],
        "expected": case["expected"],
        "target": target,
        "hypotheses_empty": hypotheses_empty,
        "target_recalled": target_recalled,
        "admitted_evidence": admitted,
        "artifact_exact_supported": exact_supported,
        "harness_materialized_investigation_target": harness_materialized,
        "target_grounded": grounded,
        "unsupported_exposed_assertions": unsupported,
        "finalization_status": finalization.get("status"),
        "investigation_stop_reason": telemetry.get("stop_reason"),
        "assertions": [
            {"key": key, "value": value, "mode": mode}
            for key, value, mode in assertions
        ],
        "wall_clock_ms": elapsed,
        "passed": passed,
    }


def session_artifact(store):
    session = load_json(store)
    if (session.get("runtime") or {}).get("natural_output_contract") != "reason-natural-output-v4":
        raise EvalError("session natural output contract drift")
    checkpoints = (session.get("thread") or {}).get("checkpoints") or []
    if not checkpoints:
        raise EvalError("session has no checkpoint")
    return (checkpoints[-1].get("snapshot") or {}).get("artifact") or {}


def session_events(store):
    return (load_json(store).get("thread") or {}).get("events") or []


def event_kind(event):
    kind = event.get("kind")
    return kind.get("kind") if isinstance(kind, dict) else None


def event_change_kind(event):
    kind = event.get("kind")
    if not isinstance(kind, dict):
        return None
    change = kind.get("change")
    return change.get("kind") if isinstance(change, dict) else None


def explicit_user_fact_values(artifact, key):
    values = set()
    for evidence in artifact.get("evidence") or []:
        metadata = evidence.get("metadata") or {}
        if metadata.get("provenance_class") != "explicit_user_fact":
            continue
        facts = evidence.get("facts") or {}
        if key in facts:
            values.add(str(facts[key]))
    return values


def exact_explicit_user_fact_persisted(artifact, target):
    values = explicit_user_fact_values(artifact, target["key"])
    return values == {str(target["value"])}


def harness_owned_correction_target_supported(artifact, target):
    for claim in artifact.get("claims") or []:
        proposition = claim.get("proposition") or {}
        if (
            str(claim.get("id") or "").startswith("harness_session_correction_target_")
            and proposition.get("key") == target["key"]
            and str(proposition.get("value")) == str(target["value"])
            and claim.get("state") in ("known", "supported")
        ):
            return True
    return False


def run_session_correct(case, reason_bin, policy, cwd, seed):
    target = case["target"]
    with tempfile.TemporaryDirectory(prefix="reason-engine050-final-v3-session-") as td:
        store = Path(td) / "session.json"
        start = [
            str(reason_bin), "session", "start",
            "--store", str(store),
            "--id", "engine050v3-" + case["id"],
            case["task"],
            "--provider", policy["provider"],
            "--model", policy["model"],
            "--max-tokens", str(policy["max_tokens"]),
            "--seed", str(seed),
            "--no-config",
            "--format", "json",
            "--fact", case["start_fact"],
        ]
        _, start_elapsed = run_json(start, cwd)
        start_artifact = session_artifact(store)
        start_target = {
            "key": target["key"],
            "value": case["start_fact"].split("=", 1)[1],
        }
        start_supported = proposition_supported(start_artifact, start_target)
        start_explicit_fact_persisted = exact_explicit_user_fact_persisted(
            start_artifact, start_target
        )
        if not start_explicit_fact_persisted:
            raise EvalError("session start did not persist the unique explicit user fact identity")

        time.sleep(policy["inter_case_delay_ms"] / 1000)
        correct = [
            str(reason_bin), "session", "correct",
            "--store", str(store),
            "--premise", case["correction"],
            "--seed", str(seed + 1),
            "--format", "json",
        ]
        payload, elapsed = run_json(correct, cwd)
        result = payload["result"]
        artifact = session_artifact(store)
        finalization = result.get("finalization") or {}
        grounded, unsupported, assertions = target_exposure(finalization, artifact, target)
        events = session_events(store)
        corrected = any(
            event_kind(event) == "input_changed"
            and event_change_kind(event) == "premise_corrected"
            for event in events
        )
        invalidated = any(event_kind(event) == "input_state_invalidated" for event in events)
        exact_supported = proposition_supported(artifact, target)
        harness_materialized = harness_owned_correction_target_supported(artifact, target)
        passed = (
            start_explicit_fact_persisted
            and corrected
            and invalidated
            and result.get("pending_revalidation") is False
            and int(result.get("external_calls_replayed") or 0) == 0
            and exact_supported
            and harness_materialized
            and grounded
            and unsupported == 0
        )
        return {
            "id": case["id"],
            "kind": case["kind"],
            "expected": case["expected"],
            "target": target,
            "start_prior_grounded": start_supported,
            "start_prior_explicit_fact_persisted": start_explicit_fact_persisted,
            "correction_event_recorded": corrected,
            "invalidation_event_recorded": invalidated,
            "pending_revalidation": result.get("pending_revalidation"),
            "external_calls_replayed": int(result.get("external_calls_replayed") or 0),
            "artifact_exact_supported": exact_supported,
            "harness_materialized_correction_target": harness_materialized,
            "target_grounded": grounded,
            "unsupported_exposed_assertions": unsupported,
            "finalization_status": finalization.get("status"),
            "assertions": [
                {"key": key, "value": value, "mode": mode}
                for key, value, mode in assertions
            ],
            "wall_clock_ms": start_elapsed + elapsed,
            "passed": passed,
        }


def run_live(reason_bin, target_id):
    manifest, cases = validate_corpus()
    root = corpus_root()
    cwd = root_path()
    policy = manifest["provider_targets"][target_id]
    results = []
    for index, case in enumerate(cases):
        seed = policy["base_seed"] + index * 17
        if case["kind"] == "session_correct":
            result = run_session_correct(case, reason_bin, policy, cwd, seed)
        else:
            cmd = [
                str(reason_bin),
                case["task"],
                "--provider", policy["provider"],
                "--model", policy["model"],
                "--max-tokens", str(policy["max_tokens"]),
                "--seed", str(seed),
                "--config", str((root / case["config"]).resolve()),
                "--format", "json",
            ]
            if "--hypothesis" in cmd:
                raise EvalError("investigation launch unexpectedly contains --hypothesis")
            payload, elapsed = run_json(cmd, cwd)
            result = investigation_result(case, payload["result"], elapsed)
        results.append(result)
        if index + 1 < len(cases):
            time.sleep(policy["inter_case_delay_ms"] / 1000)

    correctness = sum(int(r.get("unsupported_exposed_assertions") or 0) for r in results)
    replayed = sum(int(r.get("external_calls_replayed") or 0) for r in results)
    passed = all(r["passed"] for r in results) and correctness == 0 and replayed == 0
    return {
        "schema_version": REPORT_SCHEMA,
        "corpus_identity": CORPUS_ID,
        "evaluator_identity": EVALUATOR_ID,
        "scoring_identity": SCORING_ID,
        "product_commit": PRODUCT_COMMIT,
        "control_commit": CONTROL_COMMIT,
        "provider_target": target_id,
        "provider_policy": policy,
        "live_observation_performed": True,
        "cases": results,
        "aggregate": {
            "total_cases": len(results),
            "passed_cases": sum(int(r["passed"]) for r in results),
            "correctness_boundary_violations": correctness,
            "session_external_calls_replayed": replayed,
        },
        "acceptance_passed": passed,
    }

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--reason-bin", type=Path)
    parser.add_argument("--target", choices=sorted(EXPECTED_TARGETS))
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    manifest, cases = validate_corpus()
    if args.validate_only:
        report = {
            "schema_version": REPORT_SCHEMA,
            "corpus_identity": CORPUS_ID,
            "evaluator_identity": EVALUATOR_ID,
            "scoring_identity": SCORING_ID,
            "product_commit": PRODUCT_COMMIT,
            "control_commit": CONTROL_COMMIT,
            "provider_targets": sorted(manifest["provider_targets"]),
            "cases": len(cases),
            "live_observation_performed": False,
            "valid": True,
        }
    else:
        if args.reason_bin is None or args.target is None:
            raise SystemExit("--reason-bin and --target are required for live evaluation")
        report = run_live(args.reason_bin.resolve(), args.target)
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(encoded)
    print(encoded, end="")
    if not args.validate_only and not report["acceptance_passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
