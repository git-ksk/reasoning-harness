#!/usr/bin/env python3
"""Apply canonical title-style trusted-context query shaping after v6.

The Harness still withholds rank>1 facts. This changes only the exact canonical
suggested query exposed after identity insufficiency from whitespace-joined
surface/context to a title-style ``surface, context`` form that both external
search sources accept consistently in fresh development probes.
"""
from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")


def one(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def main() -> int:
    text = PATH.read_text(encoding="utf-8")
    text = one(
        text,
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v6";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v7";\n',
        "schema-v7",
    )
    text = one(
        text,
        '            Value::String(format!("{} {}", case.initial_query, context)),\n',
        '            Value::String(format!("{}, {}", case.initial_query, context)),\n',
        "canonical-title-style-query",
    )

    start = text.index('\n#[cfg(test)]\nmod v6_contract_tests {')
    end = text.index('\n#[tokio::main]\n', start)
    tests = text[start:end]
    replaced = tests.replace('"Alpha Region"', '"Alpha, Region"')
    if replaced == tests:
        raise SystemExit("v7-tests: expected canonical query literals to replace")
    text = text[:start] + replaced + text[end:]

    text = one(
        text,
        '        planner_action_policy: "planner is untrusted; Harness keeps rank1 identity sufficiency unchanged, emits one canonical query only from separately supplied trusted identity context after identity insufficiency, and executes it only when the planner explicitly selects follow_suggested_query; free-form search is unavailable while that suggestion exists; invalid actions preserve the suggestion and are blocked before external request",\n',
        '        planner_action_policy: "planner is untrusted; Harness keeps rank1 identity sufficiency unchanged, emits one canonical title-style surface-comma-context query only from separately supplied trusted identity context after identity insufficiency, and executes it only when the planner explicitly selects follow_suggested_query; free-form search is unavailable while that suggestion exists; invalid actions preserve the suggestion and are blocked before external request",\n',
        "report-policy-v7",
    )

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
