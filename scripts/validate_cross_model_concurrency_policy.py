#!/usr/bin/env python3
from __future__ import annotations
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
POLICY = ROOT / "config/cross-model-concurrency-policy.json"
WORKFLOWS = ROOT / ".github" / "workflows"


def fail(message: str) -> None:
    raise SystemExit(f"cross-model concurrency policy violation: {message}")


def load_policy() -> dict:
    data = json.loads(POLICY.read_text())
    if data.get("schema_version") != "cross-model-concurrency-policy-v1":
        fail("unexpected policy schema")
    if data.get("principle") != "provider-aware-lanes":
        fail("provider-aware-lanes principle missing")
    providers = data.get("providers", {})
    expected = {"mistral", "google", "nvidia", "groq"}
    if set(providers) != expected:
        fail(f"provider inventory drift: {sorted(providers)}")
    if providers["mistral"].get("model_job_parallelism") != 1:
        fail("Mistral model-job parallelism must remain serialized by default")
    if providers["google"].get("model_job_parallelism") != 1:
        fail("Google model-job parallelism must remain serialized by default")
    google = providers["google"]
    if google.get("canonical_request_start_interval_ms") != 6000:
        fail("Google canonical request-start interval must remain 6000ms")
    if google.get("canonical_max_request_starts_per_minute") != 10:
        fail("Google canonical request-start rate must remain <=10 RPM")
    if google.get("canonical_headroom_reference_rpm") != 15:
        fail("Google canonical headroom reference must remain 15 RPM")
    if google.get("canonical_headroom_requests_per_minute") != 5:
        fail("Google canonical request-count headroom must remain 5 RPM")
    if google.get("canonical_inter_case_delay_ms") != 3000:
        fail("Google canonical inter-case delay must remain 3000ms")
    if google.get("canonical_google_model_jobs_serial") is not True:
        fail("Google canonical model jobs must remain serialized")
    if google.get("shared_pacer_required_for_parallel_fixtures") is not True:
        fail("Google parallel fixtures must share one request pacer")
    if providers["nvidia"].get("model_job_parallelism") != 1:
        fail("NVIDIA model-job parallelism must remain serialized by default")
    if providers["groq"].get("model_job_parallelism", 0) < 2:
        fail("Groq current per-model quota lane must preserve model-level parallelism")
    return data


def exception_map(policy: dict) -> dict[str, str]:
    out = {}
    for item in policy.get("historical_exceptions", []):
        workflow = item.get("workflow")
        anchor = item.get("anchor")
        if not workflow or not anchor:
            fail("historical exception requires workflow and immutable anchor")
        out[workflow] = anchor
    return out


def check_mixed_provider_global_serialization(policy: dict) -> None:
    exceptions = exception_map(policy)
    for path in sorted(WORKFLOWS.glob("*.yml")):
        text = path.read_text()
        if "cross-model" not in path.name and "cross-model" not in text:
            continue
        providers = set(re.findall(r"^\s*-\s+provider:\s*([A-Za-z0-9_-]+)\s*$", text, re.M))
        mixed = len(providers) > 1
        globally_serial = bool(re.search(r"^\s*max-parallel:\s*1\s*$", text, re.M))
        if not (mixed and globally_serial):
            continue
        rel = str(path.relative_to(ROOT))
        if rel not in exceptions:
            fail(
                f"{rel} mixes providers {sorted(providers)} under max-parallel:1; "
                "split provider lanes or document an exact frozen historical exception"
            )
        anchor = exceptions[rel]
        if anchor not in text:
            fail(f"{rel} exception does not contain immutable anchor {anchor}")


def check_known_provider_controls() -> None:
    live = (WORKFLOWS / "live-benchmark.yml").read_text()
    if "group: reasoning-harness-mistral-live" not in live:
        fail("Mistral repository concurrency group missing from live benchmark")
    google_block = live.split("\n  google:", 1)[1].split("\n  ", 1)[0] if "\n  google:" in live else ""
    # The simple split above is intentionally not relied on for semantics; verify the known marker globally too.
    if "name: google-${{ matrix.model }}" not in live or "max-parallel: 1" not in live:
        fail("Google serialized model-job control missing from live benchmark")
    v4 = (WORKFLOWS / "product-external-info-v4-cross-model.yml").read_text()
    if "name: groq-${{ matrix.model }}" not in v4 or "max-parallel: 3" not in v4:
        fail("Groq v4 per-model parallel lane missing")
    if "REASON_GROQ_MIN_REQUEST_INTERVAL_MS" not in v4 or "REASON_GROQ_TOKENS_PER_MINUTE" not in v4:
        fail("Groq provider-local pacing controls missing")


def main() -> int:
    policy = load_policy()
    check_mixed_provider_global_serialization(policy)
    check_known_provider_controls()
    print("cross-model concurrency policy: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
