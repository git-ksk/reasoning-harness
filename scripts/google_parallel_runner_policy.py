#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Mapping

class RunnerPolicyError(ValueError):
    pass

def _require(condition: bool, message: str) -> None:
    if not condition:
        raise RunnerPolicyError(message)
def validate_parallel_runner_policy(manifest: Mapping[str, object], *, provider: str, model: str, workers: int, inter_case_delay_ms: int, env: Mapping[str, str] | None = None) -> None:
    """Validate bounded parallel Google runner invariants without model/network work."""
    if workers != 2:
        return
    _require((provider, model) == ("google", "gemma-4-31b-it"), "parallel investigation workers are allowed only for google/gemma-4-31b-it")
    provider_policy = manifest.get("provider_policy")
    parallel = manifest.get("parallel_execution_policy")
    _require(isinstance(provider_policy, Mapping), "manifest provider policy missing")
    _require(isinstance(parallel, Mapping), "manifest parallel execution policy missing")
    expected_delay = provider_policy.get("inter_case_delay_ms")
    expected_pacing = parallel.get("google_request_start_interval_ms")
    _require(isinstance(expected_delay, int), "manifest inter-case delay missing")
    _require(isinstance(expected_pacing, int), "manifest Google request-start interval missing")
    _require(inter_case_delay_ms == expected_delay, f"inter-case delay mismatch: expected {expected_delay}ms, got {inter_case_delay_ms}ms")
    _require(parallel.get("gemma_investigation_workers") == 2, "manifest Gemma workers must remain 2")
    _require(parallel.get("google_shared_pacer_required_for_parallel") is True, "manifest must require shared Google pacing for parallel work")
    _require(parallel.get("google_model_jobs_serial") is True, "Google model jobs must remain serial")
    _require(parallel.get("google_model_job_max_parallel") == 1, "Google model max parallel must remain 1")
    _require(parallel.get("adaptive_followup_cases_serial") is True, "adaptive cases must remain serial")
    _require(parallel.get("mcp_nonpromotion_cases_serial") is True, "MCP cases must remain serial")
    _require(parallel.get("session_cases_serial") is True, "session cases must remain serial")
    actual_env = os.environ if env is None else env
    shared = actual_env.get("REASON_GOOGLE_SHARED_PACER_PATH", "")
    _require(bool(shared) and Path(shared).is_absolute(), "shared Google pacer path must be absolute")
    pacing = actual_env.get("REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS", "")
    try:
        pacing_ms = int(pacing)
    except (TypeError, ValueError):
        raise RunnerPolicyError("Google request-start pacing must be an integer") from None
    _require(pacing_ms == expected_pacing, f"Google request-start pacing mismatch: expected {expected_pacing}ms, got {pacing_ms}ms")

def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--provider", required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--workers", type=int, required=True)
    parser.add_argument("--inter-case-delay-ms", type=int, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    validate_parallel_runner_policy(manifest, provider=args.provider, model=args.model, workers=args.workers, inter_case_delay_ms=args.inter_case_delay_ms)
    print("Google parallel runner policy: OK")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
