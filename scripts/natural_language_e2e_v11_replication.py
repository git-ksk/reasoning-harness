#!/usr/bin/env python3
"""Cross-model replication runner for immutable natural-language E2E v11.

This wrapper deliberately reuses the frozen v11 evaluator/scoring implementation
without weakening v11's canonical Mistral-only guard.
"""
from __future__ import annotations

import argparse
import json
import os
import time
from pathlib import Path

import natural_language_e2e_v11 as v11

REPLICATION_ID = "natural-language-e2e-v11-cross-model-v1"
DEFAULT_MANIFEST = Path("fixtures/natural-language-e2e-v11-cross-model-v1/manifest.json")
DEFAULT_FIXTURES = Path("fixtures/natural-language-e2e-v11")


def load_replication_manifest(path: Path) -> dict:
    manifest = json.loads(path.read_text())
    if manifest.get("schema_version") != "natural-language-e2e-v11-cross-model-manifest-v1":
        raise v11.EvalError("replication manifest schema mismatch")
    if manifest.get("replication_identity") != REPLICATION_ID:
        raise v11.EvalError("replication identity mismatch")
    ref = manifest.get("reference", {})
    if ref.get("corpus_identity") != v11.CORPUS_ID:
        raise v11.EvalError("reference corpus identity mismatch")
    if ref.get("freeze_commit") != "a758af17a998493c1005702365b100e05b05f95d":
        raise v11.EvalError("reference freeze commit drift")
    if ref.get("product_commit") != "29a9e4be6273dbffeda324e15517dc64930ad315":
        raise v11.EvalError("reference product commit drift")
    policy = manifest.get("semantic_policy", {})
    expected_policy = {
        "base_seed": 57000,
        "max_tokens": 1024,
        "inter_case_delay_ms": 1500,
        "reuse_reference_evaluator_and_scoring": True,
        "trigger_miss_is_not_mechanism_failure": True,
        "zero_trigger_denominator_is_inconclusive": True,
        "provider_specific_transport_pacing_allowed": True,
    }
    if {key: policy.get(key) for key in expected_policy} != expected_policy:
        raise v11.EvalError("replication semantic policy drift")
    targets = manifest.get("targets")
    if not isinstance(targets, list) or not targets:
        raise v11.EvalError("replication targets missing")
    seen = set()
    for target in targets:
        coord = (target.get("provider"), target.get("model"))
        if not all(isinstance(x, str) and x for x in coord) or coord in seen:
            raise v11.EvalError("invalid or duplicate replication target")
        seen.add(coord)
    canonical = (ref.get("canonical_provider"), ref.get("canonical_model"))
    if canonical in seen:
        raise v11.EvalError("canonical v11 Mistral coordinate must not be rerun")
    return manifest


def allowed_target(manifest: dict, provider: str, model: str) -> bool:
    return any(t["provider"] == provider and t["model"] == model for t in manifest["targets"])


def run(args: argparse.Namespace) -> dict:
    replication = load_replication_manifest(Path(args.replication_manifest))
    root = Path(args.fixtures)
    reference_manifest, cases = v11.validate_corpus(root)
    ref = replication["reference"]
    if reference_manifest.get("product_coordinate", {}).get("commit") != ref["product_commit"]:
        raise v11.EvalError("v11 product coordinate differs from replication reference")
    if args.validate_only:
        return {
            "schema_version": "reason-natural-language-e2e-v11-cross-model-v1",
            "replication_identity": REPLICATION_ID,
            "valid": True,
            "live_observation_performed": False,
            "reference_evaluator_identity": v11.EVALUATOR_ID,
            "reference_scoring_identity": v11.SCORING_ID,
            "reference_freeze_commit": ref["freeze_commit"],
            "reference_product_commit": ref["product_commit"],
            "cases": len(cases),
            "targets": replication["targets"],
        }
    if args.preflight:
        checked = v11.self_test_resolvers(root, cases)
        admission = v11.self_test_admission_contracts(root, cases)
        return {
            "schema_version": "reason-natural-language-e2e-v11-cross-model-v1",
            "replication_identity": REPLICATION_ID,
            "valid": True,
            "live_observation_performed": False,
            "model_used": False,
            "resolver_capabilities_checked": checked,
            "admission_contracts": admission,
            "cases": len(cases),
        }
    provider = args.provider
    model = args.model
    if not provider or not model or not allowed_target(replication, provider, model):
        raise v11.EvalError("provider/model is not a frozen replication target")
    policy = replication["semantic_policy"]
    seed = args.seed if args.seed is not None else policy["base_seed"]
    max_tokens = args.max_tokens if args.max_tokens is not None else policy["max_tokens"]
    delay = args.inter_case_delay_ms if args.inter_case_delay_ms is not None else policy["inter_case_delay_ms"]
    if (seed, max_tokens, delay) != (
        policy["base_seed"], policy["max_tokens"], policy["inter_case_delay_ms"]
    ):
        raise v11.EvalError("replication semantic coordinate mismatch")
    if not args.reason_bin or not Path(args.reason_bin).exists():
        raise v11.EvalError("--reason-bin must reference built reason executable")

    reason_bin = Path(args.reason_bin).resolve()
    cwd = Path.cwd()
    env = os.environ.copy()
    reports = []
    live_boundary_recorded = False
    for idx, (_, case) in enumerate(cases):
        case_seed = seed + idx
        if not live_boundary_recorded:
            marker = {
                "schema_version": "reason-natural-language-e2e-v11-cross-model-v1",
                "replication_identity": REPLICATION_ID,
                "corpus_identity": v11.CORPUS_ID,
                "live_case_launch_boundary_entered": True,
                "case_id": case["id"],
                "provider": provider,
                "model": model,
                "seed": case_seed,
                "product_coordinate": reference_manifest["product_coordinate"],
                "reference_freeze_commit": ref["freeze_commit"],
                "github_run_id": os.environ.get("GITHUB_RUN_ID"),
                "github_run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT"),
            }
            if args.attempt_marker:
                Path(args.attempt_marker).write_text(json.dumps(marker, indent=2, sort_keys=True) + "\n")
            live_boundary_recorded = True
        if case["kind"] == "investigation":
            cmd = [
                str(reason_bin), case["task"], "--provider", provider, "--model", model,
                "--max-tokens", str(max_tokens), "--seed", str(case_seed), "--config",
                str((root / case["config"]).resolve()), "--format", "json",
            ]
            cp, payload, elapsed = v11.run_json(cmd, cwd, env)
            if cp.returncode != 0 or not isinstance(payload, dict) or "result" not in payload:
                reports.append(v11.invocation_failure(case["id"], cp, payload, elapsed))
            else:
                reports.append(v11.score_investigation(case, payload["result"], elapsed))
        else:
            reports.append(v11.run_session_case(case, reason_bin, provider, model, max_tokens, case_seed, cwd, env, delay))
        if idx + 1 < len(cases):
            time.sleep(delay / 1000)

    aggregate = v11.aggregate(reports)
    gates = v11.evaluate_report_gates(reference_manifest, aggregate, len(reports), len(cases))
    return {
        "schema_version": v11.REPORT_SCHEMA,
        "replication_identity": REPLICATION_ID,
        "cross_model_replication": True,
        "corpus_identity": v11.CORPUS_ID,
        "evaluator_identity": v11.EVALUATOR_ID,
        "scoring_identity": v11.SCORING_ID,
        "live_observation_performed": True,
        "provider": provider,
        "model": model,
        "seed": seed,
        "max_tokens": max_tokens,
        "inter_case_delay_ms": delay,
        "raw_model_comparison_claimed": False,
        "product_coordinate": reference_manifest["product_coordinate"],
        "reference_freeze_commit": ref["freeze_commit"],
        "reference_canonical_run": ref["canonical_actions_run"],
        "reference_provider_policy": reference_manifest["provider_policy"],
        "mcp_policy": reference_manifest["mcp_policy"],
        "measurement_semantics": reference_manifest["measurement_semantics"],
        "cases": reports,
        "aggregate": aggregate,
        **gates,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--replication-manifest", default=str(DEFAULT_MANIFEST))
    parser.add_argument("--fixtures", default=str(DEFAULT_FIXTURES))
    parser.add_argument("--reason-bin")
    parser.add_argument("--provider")
    parser.add_argument("--model")
    parser.add_argument("--seed", type=int)
    parser.add_argument("--max-tokens", type=int)
    parser.add_argument("--inter-case-delay-ms", type=int)
    parser.add_argument("--attempt-marker")
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--output")
    args = parser.parse_args()
    result = run(args)
    text = json.dumps(result, indent=2, sort_keys=True)
    print(text)
    if args.output:
        Path(args.output).write_text(text + "\n")
    if result.get("live_observation_performed"):
        return 0 if result.get("report_gate_passed") else 3
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except v11.EvalError as exc:
        print(json.dumps({"schema_version": "reason-natural-language-e2e-v11-cross-model-v1", "valid": False, "error": str(exc)}))
        raise SystemExit(2)
