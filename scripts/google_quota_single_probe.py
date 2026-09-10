#!/usr/bin/env python3
"""Send exactly one Gemini API request and emit a sanitized quota diagnostic.

This is an operational diagnostic only. It does not score model output and never retries.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any

BASE_URL = "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
SHORT_RETRY_SECONDS = 120.0


def _detail_type(detail: dict[str, Any]) -> str:
    return str(detail.get("@type", ""))


def _retry_seconds(value: Any) -> float | None:
    if not isinstance(value, str):
        return None
    match = re.fullmatch(r"\s*([0-9]+(?:\.[0-9]+)?)s\s*", value)
    return float(match.group(1)) if match else None


def classify(http_status: int, payload: dict[str, Any]) -> dict[str, Any]:
    error = payload.get("error") if isinstance(payload.get("error"), dict) else {}
    error_status = error.get("status")
    details = error.get("details") if isinstance(error.get("details"), list) else []

    violations: list[dict[str, Any]] = []
    retry_delays: list[str] = []
    error_reasons: list[str] = []

    for detail in details:
        if not isinstance(detail, dict):
            continue
        dtype = _detail_type(detail)
        if dtype.endswith("google.rpc.QuotaFailure"):
            raw_violations = detail.get("violations")
            if isinstance(raw_violations, list):
                for violation in raw_violations:
                    if isinstance(violation, dict):
                        violations.append(
                            {
                                key: violation.get(key)
                                for key in ("quotaMetric", "quotaId", "quotaDimensions", "quotaValue")
                                if key in violation
                            }
                        )
        elif dtype.endswith("google.rpc.RetryInfo"):
            retry_delay = detail.get("retryDelay")
            if isinstance(retry_delay, str):
                retry_delays.append(retry_delay)
        elif dtype.endswith("google.rpc.ErrorInfo"):
            reason = detail.get("reason")
            if isinstance(reason, str):
                error_reasons.append(reason)

    quota_ids = [str(v.get("quotaId", "")) for v in violations]
    quota_ids_lower = [q.lower() for q in quota_ids]
    daily = any(any(token in q for token in ("perday", "daily", "rpd")) for q in quota_ids_lower)
    per_minute = any(any(token in q for token in ("perminute", "minute", "rpm")) for q in quota_ids_lower)
    bounded_retry = any(
        seconds is not None and 0 <= seconds <= SHORT_RETRY_SECONDS
        for seconds in map(_retry_seconds, retry_delays)
    )

    if 200 <= http_status < 300:
        classification = "success"
    elif http_status in (401, 403):
        classification = "auth_or_access_error"
    elif http_status == 429 or error_status == "RESOURCE_EXHAUSTED":
        if daily:
            classification = "daily_quota"
        elif per_minute:
            classification = "rate_limit_per_minute"
        elif bounded_retry:
            classification = "rate_limit_short_window"
        else:
            classification = "quota_ambiguous"
    elif 500 <= http_status < 600:
        classification = "provider_error"
    else:
        classification = "client_or_other_error"

    result: dict[str, Any] = {
        "classification": classification,
        "http_status": http_status,
        "error_status": error_status,
        "quota_violations": violations,
        "retry_delays": retry_delays,
        "error_reasons": error_reasons,
    }
    if error:
        result["error_message"] = error.get("message")
    return result


def _sanitize_success(payload: dict[str, Any]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key in ("modelVersion", "responseId", "usageMetadata", "promptFeedback"):
        if key in payload:
            result[key] = payload[key]
    candidates = payload.get("candidates")
    if isinstance(candidates, list):
        result["candidate_finish_reasons"] = [
            candidate.get("finishReason")
            for candidate in candidates
            if isinstance(candidate, dict) and "finishReason" in candidate
        ]
    return result


class _NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):  # noqa: ANN001
        return None


def run_probe(model: str, api_key: str, timeout_seconds: float = 30.0) -> tuple[int, dict[str, Any]]:
    body = json.dumps(
        {
            "contents": [{"parts": [{"text": "Reply with exactly OK."}]}],
            "generationConfig": {"maxOutputTokens": 16, "temperature": 0},
        }
    ).encode("utf-8")
    request = urllib.request.Request(
        BASE_URL.format(model=model),
        data=body,
        method="POST",
        headers={
            "Content-Type": "application/json",
            "x-goog-api-key": api_key,
            "User-Agent": "reasoning-harness-google-quota-single-probe/1",
        },
    )
    opener = urllib.request.build_opener(_NoRedirect)
    try:
        with opener.open(request, timeout=timeout_seconds) as response:
            status = int(response.status)
            raw = response.read()
    except urllib.error.HTTPError as exc:
        status = int(exc.code)
        raw = exc.read()
    payload = json.loads(raw.decode("utf-8")) if raw else {}
    if not isinstance(payload, dict):
        payload = {"unexpected_payload_type": type(payload).__name__}
    return status, payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--timeout-seconds", type=float, default=30.0)
    args = parser.parse_args()

    api_key = os.environ.get("GEMINI_API_KEY") or os.environ.get("GOOGLE_API_KEY")
    if not api_key:
        print("GEMINI_API_KEY or GOOGLE_API_KEY is required", file=sys.stderr)
        return 2

    started = time.time()
    try:
        status, payload = run_probe(args.model, api_key, args.timeout_seconds)
        diagnostic = classify(status, payload)
        diagnostic.update(
            {
                "schema_version": "google-quota-single-probe-v1",
                "model": args.model,
                "requests_sent": 1,
                "retries": 0,
                "scoring_input": False,
                "elapsed_ms": int((time.time() - started) * 1000),
            }
        )
        if 200 <= status < 300:
            diagnostic["success_metadata"] = _sanitize_success(payload)
    except Exception as exc:  # network/parse failures are still diagnostic evidence
        diagnostic = {
            "schema_version": "google-quota-single-probe-v1",
            "model": args.model,
            "requests_sent": 1,
            "retries": 0,
            "scoring_input": False,
            "classification": "transport_or_parse_error",
            "elapsed_ms": int((time.time() - started) * 1000),
            "exception_type": type(exc).__name__,
            "exception_message": str(exc),
        }

    Path(args.output).write_text(json.dumps(diagnostic, indent=2, sort_keys=True) + "\n")
    print(json.dumps(diagnostic, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
