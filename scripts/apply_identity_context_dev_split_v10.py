#!/usr/bin/env python3
"""Materialize the preassigned Issue #196 fresh-development split."""
from __future__ import annotations

import json
from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")
FIXTURE = Path("fixtures/mcp-identity-context-v3-dev.json")


def fn_bounds(text: str, name: str) -> tuple[int, int]:
    marker = f"fn {name}("
    start = text.find(marker)
    if start < 0:
        raise SystemExit(f"function not found: {name}")
    brace = text.find("{", start)
    depth = 0
    for i in range(brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return start, i + 1
    raise SystemExit(f"function closing brace not found: {name}")


def rust(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def expected(value: str) -> str:
    return {
        "accept": "ExpectedOutcome::Accept",
        "reject": "ExpectedOutcome::Reject",
        "unknown": "ExpectedOutcome::Unknown",
    }[value]


def render(cases: list[dict]) -> str:
    lines = [f"fn dev_cases() -> [CaseSpec; {len(cases)}] {{", "    ["]
    for case in cases:
        context = (
            f"Some({rust(case['identity_context'])})"
            if case["identity_context"] is not None
            else "None"
        )
        lines.extend([
            "        CaseSpec {",
            f"            id: {rust(case['id'])},",
            f"            task: {rust(case['task'])},",
            f"            initial_query: {rust(case['initial_query'])},",
            f"            identity_context: {context},",
            f"            property_id: {rust(case['property_id'])},",
            f"            value_kind: {rust(case['value_kind'])},",
            f"            fact_key: {rust(case['fact_key'])},",
            f"            target_value: {rust(case['target_value'])},",
            f"            expected: {expected(case['expected'])},",
            "        },",
        ])
    lines.extend(["    ]", "}"])
    return "\n".join(lines)


def main() -> int:
    payload = json.loads(FIXTURE.read_text(encoding="utf-8"))
    if payload.get("schema_version") != "reason-mcp-identity-context-v3-dev-v1":
        raise SystemExit("unexpected fresh-dev fixture schema")
    cases = payload.get("cases")
    if not isinstance(cases, list) or len(cases) != 8:
        raise SystemExit("fresh-dev fixture must contain exactly 8 cases")

    text = PATH.read_text(encoding="utf-8")
    start, end = fn_bounds(text, "dev_cases")
    text = text[:start] + render(cases) + text[end:]
    text = text.replace(
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v9";',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v10";',
        1,
    )
    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
