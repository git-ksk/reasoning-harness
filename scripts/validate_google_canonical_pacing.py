#!/usr/bin/env python3
"""Validate prospective Google natural-language E2E canonical pacing.

Historical frozen workflows are intentionally not rewritten. A fresh successor can
invoke this validator before credentials are exposed to prove that its workflow and
manifest use the repository-wide Google canonical pacing policy.
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY_PATH = ROOT / "config" / "cross-model-concurrency-policy.json"


class ValidationError(ValueError):
    pass


def load_google_policy(path: Path = POLICY_PATH) -> dict:
    data = json.loads(path.read_text())
    if data.get("schema_version") != "cross-model-concurrency-policy-v1":
        raise ValidationError("unexpected concurrency policy schema")
    google = data.get("providers", {}).get("google")
    if not isinstance(google, dict):
        raise ValidationError("Google concurrency policy missing")

    interval = google.get("canonical_request_start_interval_ms")
    max_rpm = google.get("canonical_max_request_starts_per_minute")
    ceiling = google.get("canonical_headroom_reference_rpm")
    headroom_rpm = google.get("canonical_headroom_requests_per_minute")
    inter_case_delay = google.get("canonical_inter_case_delay_ms")
    if interval != 6000:
        raise ValidationError("Google canonical request-start interval must be 6000ms")
    if max_rpm != 10:
        raise ValidationError("Google canonical max request starts must be 10 RPM")
    if ceiling != 15:
        raise ValidationError("Google canonical headroom reference must remain 15 RPM")
    if headroom_rpm != 5:
        raise ValidationError("Google canonical request-count headroom must remain 5 RPM")
    if inter_case_delay != 3000:
        raise ValidationError("canonical inter-case delay must remain 3000ms")
    if 60000 // interval != max_rpm:
        raise ValidationError("Google interval/RPM policy is internally inconsistent")
    if max_rpm >= ceiling:
        raise ValidationError("Google canonical pacing must remain below the reference ceiling")
    if ceiling - max_rpm != headroom_rpm:
        raise ValidationError("Google canonical headroom policy is internally inconsistent")
    if google.get("model_job_parallelism") != 1 or google.get("canonical_google_model_jobs_serial") is not True:
        raise ValidationError("Google canonical model jobs must remain serialized")
    if google.get("shared_pacer_required_for_parallel_fixtures") is not True:
        raise ValidationError("parallel Google fixtures must require one shared pacer")
    return google


def _job_block(text: str, job_name: str = "google-paired") -> str:
    match = re.search(rf"(?ms)^  {re.escape(job_name)}:\s*\n(.*?)(?=^  [A-Za-z0-9_-]+:\s*\n|\Z)", text)
    if not match:
        raise ValidationError(f"workflow job {job_name!r} not found")
    return match.group(1)


def _step_block_containing(job_block: str, marker: str) -> str:
    lines = job_block.splitlines(keepends=True)
    marker_index = next((i for i, line in enumerate(lines) if marker in line), None)
    if marker_index is None:
        raise ValidationError(f"workflow marker {marker!r} not found")
    start = marker_index
    while start >= 0 and not re.match(r"^      - ", lines[start]):
        start -= 1
    if start < 0:
        raise ValidationError(f"workflow marker {marker!r} is not inside a step")
    end = start + 1
    while end < len(lines) and not re.match(r"^      - ", lines[end]):
        end += 1
    return "".join(lines[start:end])


def validate_workflow(path: Path, google: dict) -> None:
    text = path.read_text()
    block = _job_block(text)
    interval = google["canonical_request_start_interval_ms"]

    if not re.search(r"(?m)^\s{6}max-parallel:\s*1\s*$", block):
        raise ValidationError("Google matrix must use max-parallel: 1")
    interval_matches = re.findall(
        r"(?m)^\s+REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS:\s*['\"]?(\d+)['\"]?\s*$",
        block,
    )
    if interval_matches != [str(interval)]:
        raise ValidationError(
            f"Google job must define exactly one request-start interval of {interval}ms; got {interval_matches}"
        )

    pacing_step = _step_block_containing(block, "REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS")
    if "REASON_GOOGLE_SHARED_PACER_PATH:" not in pacing_step:
        raise ValidationError("paired canonical step must configure a shared pacer path")
    if pacing_step.count('"--coordinate-role","control"') != 1:
        raise ValidationError("paired control command must share the canonical pacing step")
    if pacing_step.count('"--coordinate-role","candidate"') != 1:
        raise ValidationError("paired candidate command must share the canonical pacing step")

    delay = google["canonical_inter_case_delay_ms"]
    delay_matches = re.findall(
        r"(?m)^\s+INTER_CASE_DELAY_MS:\s*['\"]?(\d+)['\"]?\s*$",
        block,
    )
    if delay_matches != [str(delay)]:
        raise ValidationError(
            f"Google job must preserve one inter-case delay of {delay}ms; got {delay_matches}"
        )

    # Preserve the bounded worker policy: Gemini serial; Gemma may overlap only
    # eligible stateless investigation cases behind the shared request gate.
    if not re.search(r"(?s)model:\s*gemini-3\.5-flash-lite.*?workers:\s*1", block):
        raise ValidationError("Gemini canonical worker count must remain 1")
    if not re.search(r"(?s)model:\s*gemma-4-31b-it.*?workers:\s*2", block):
        raise ValidationError("Gemma canonical investigation worker count must remain 2")


def validate_manifest(path: Path, google: dict) -> None:
    data = json.loads(path.read_text())
    parallel = data.get("parallel_execution_policy") or {}
    provider = data.get("provider_policy") or {}
    interval = google["canonical_request_start_interval_ms"]
    if provider.get("inter_case_delay_ms") != google["canonical_inter_case_delay_ms"]:
        raise ValidationError("manifest inter-case delay does not match repository policy")
    if parallel.get("google_request_start_interval_ms") != interval:
        raise ValidationError("manifest Google request-start interval does not match repository policy")
    if parallel.get("google_shared_pacer_required_for_parallel") is not True:
        raise ValidationError("manifest must require shared Google pacing for parallel work")
    if parallel.get("google_model_jobs_serial") is not True:
        raise ValidationError("manifest must keep Google model jobs serialized")
    if parallel.get("google_model_job_max_parallel") != 1:
        raise ValidationError("manifest Google model max parallel must remain 1")
    if parallel.get("gemma_investigation_workers") != 2:
        raise ValidationError("manifest Gemma investigation workers must remain 2")
    if parallel.get("adaptive_followup_cases_serial") is not True:
        raise ValidationError("adaptive follow-up cases must remain serial")
    if parallel.get("mcp_nonpromotion_cases_serial") is not True:
        raise ValidationError("MCP non-promotion cases must remain serial")
    if parallel.get("session_cases_serial") is not True:
        raise ValidationError("session/stateful cases must remain serial")
    if parallel.get("paired_control_before_candidate_unchanged") is not True:
        raise ValidationError("paired control -> candidate ordering must remain unchanged")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workflow", type=Path)
    parser.add_argument("--manifest", type=Path)
    args = parser.parse_args()

    google = load_google_policy()
    if args.workflow:
        validate_workflow(args.workflow, google)
    if args.manifest:
        validate_manifest(args.manifest, google)
    print("Google canonical pacing policy: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
