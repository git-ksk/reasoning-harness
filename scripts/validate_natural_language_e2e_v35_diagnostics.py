#!/usr/bin/env python3
"""Validate v35 candidate diagnostic sidecars without making them scoring input."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re

CONTRACT = "reason-natural-diagnostic-trace-v1"
MODES = {"json_schema", "json_object", "text"}
PARSE_CLASSES = {"eof", "syntax", "data", "io"}
STATUS_CLASSES = {
    "complete", "incomplete", "budget_exceeded", "token_limit",
    "provider_error", "missing", "other",
}
STRUCTURED_FAILURE_MARKERS = (
    "invalid strict text JSON",
    "invalid structured planner JSON",
    "structured planner fallback failed",
)


class DiagnosticError(Exception):
    pass


def _classes(message: str, stem: str) -> set[str]:
    # Covers parse_class= and first_/second_parse_class= without depending on fallback count.
    return set(re.findall(rf"(?:^|[ _])(?:first_|second_)?{stem}=([a-z_]+)", message))


def validate_trace(path: Path) -> dict[str, int]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise DiagnosticError(f"{path}: unreadable diagnostic JSON: {error}") from error
    if value.get("contract") != CONTRACT:
        raise DiagnosticError(f"{path}: diagnostic contract mismatch")
    events = value.get("events")
    if not isinstance(events, list) or not events:
        raise DiagnosticError(f"{path}: diagnostic events missing")

    request_count = 0
    response_count = 0
    structured_failure_count = 0
    for event in events:
        if not isinstance(event, dict):
            raise DiagnosticError(f"{path}: non-object diagnostic event")
        kind = event.get("kind")
        if kind == "model_request":
            mode = event.get("structured_mode")
            if mode not in MODES:
                raise DiagnosticError(f"{path}: invalid structured_mode {mode!r}")
            request_count += 1
        elif kind == "model_response":
            response = event.get("response")
            if not isinstance(response, dict):
                raise DiagnosticError(f"{path}: model_response payload missing")
            byte_count = response.get("bytes")
            if not isinstance(byte_count, int) or isinstance(byte_count, bool) or byte_count < 0:
                raise DiagnosticError(f"{path}: response bytes must be a non-negative integer")
            if "finish_reason" in response and not isinstance(response["finish_reason"], str):
                raise DiagnosticError(f"{path}: finish_reason must be a string when present")
            response_count += 1
        elif kind == "model_failure":
            message = event.get("message")
            if not isinstance(message, str):
                raise DiagnosticError(f"{path}: model_failure message missing")
            if event.get("failure_class") == "protocol" and any(
                marker in message for marker in STRUCTURED_FAILURE_MARKERS
            ):
                parse_classes = _classes(message, "parse_class")
                status_classes = _classes(message, "status_class")
                if not parse_classes or not parse_classes <= PARSE_CLASSES:
                    raise DiagnosticError(
                        f"{path}: structured protocol failure lacks valid parse_class: {parse_classes}"
                    )
                if not status_classes or not status_classes <= STATUS_CLASSES:
                    raise DiagnosticError(
                        f"{path}: structured protocol failure lacks valid status_class: {status_classes}"
                    )
                if not re.search(r"(?:^|[ _])(?:first_|second_)?bytes=\d+", message):
                    raise DiagnosticError(f"{path}: structured protocol failure lacks byte count")
                # finish_reason may be provider-missing, but the diagnostic key must still be emitted.
                if not re.search(r"(?:^|[ _])(?:first_|second_)?finish_reason=", message):
                    raise DiagnosticError(f"{path}: structured protocol failure lacks finish_reason key")
                structured_failure_count += 1

    if request_count == 0:
        raise DiagnosticError(f"{path}: no model_request diagnostic events")
    return {
        "model_requests": request_count,
        "model_responses": response_count,
        "structured_protocol_failures": structured_failure_count,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--trace-dir", required=True)
    parser.add_argument("--expected-count", type=int, default=10)
    parser.add_argument("--output")
    args = parser.parse_args()

    trace_dir = Path(args.trace_dir)
    paths = sorted(trace_dir.glob("*.json")) if trace_dir.is_dir() else []
    if len(paths) != args.expected_count:
        raise DiagnosticError(
            f"expected {args.expected_count} diagnostic sidecars in {trace_dir}, got {len(paths)}"
        )
    totals = {"model_requests": 0, "model_responses": 0, "structured_protocol_failures": 0}
    for path in paths:
        row = validate_trace(path)
        for key in totals:
            totals[key] += row[key]
    out = {
        "schema_version": "natural-language-e2e-v35-diagnostic-validation-v1",
        "valid": True,
        "scoring_input": False,
        "trace_count": len(paths),
        **totals,
    }
    text = json.dumps(out, indent=2, sort_keys=True)
    print(text)
    if args.output:
        Path(args.output).write_text(text + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except DiagnosticError as error:
        print(json.dumps({
            "schema_version": "natural-language-e2e-v35-diagnostic-validation-v1",
            "valid": False,
            "scoring_input": False,
            "error": str(error),
        }, indent=2, sort_keys=True))
        raise SystemExit(2)
