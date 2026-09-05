#!/usr/bin/env python3
"""Issue #196 v14: add frozen holdout suite over the v12 candidate/v13 metrics.

This changes evaluation surface selection/provenance only. Identity admission,
planner policy, tool execution, budgets, and stop semantics remain unchanged.
"""
from __future__ import annotations

import json
from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")
FIXTURE = Path("fixtures/mcp-identity-context-v3-holdout-v1.json")


def one(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


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


def insert_after_fn(text: str, name: str, addition: str) -> str:
    _, end = fn_bounds(text, name)
    return text[:end] + "\n\n" + addition.rstrip() + text[end:]


def rust_string(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def expected_variant(value: str) -> str:
    return {
        "accept": "ExpectedOutcome::Accept",
        "reject": "ExpectedOutcome::Reject",
        "unknown": "ExpectedOutcome::Unknown",
    }[value]


def holdout_cases_source(payload: dict) -> str:
    cases = payload["cases"]
    lines = [f"fn holdout_cases() -> [CaseSpec; {len(cases)}] {{", "    ["]
    for case in cases:
        context = (
            f"Some({rust_string(case['identity_context'])})"
            if case["identity_context"] is not None
            else "None"
        )
        lines.extend([
            "        CaseSpec {",
            f"            id: {rust_string(case['id'])},",
            f"            task: {rust_string(case['task'])},",
            f"            initial_query: {rust_string(case['initial_query'])},",
            f"            identity_context: {context},",
            f"            property_id: {rust_string(case['property_id'])},",
            f"            value_kind: {rust_string(case['value_kind'])},",
            f"            fact_key: {rust_string(case['fact_key'])},",
            f"            target_value: {rust_string(case['target_value'])},",
            f"            expected: {expected_variant(case['expected'])},",
            "        },",
        ])
    lines.extend(["    ]", "}", ""])
    return "\n".join(lines)


def main() -> int:
    payload = json.loads(FIXTURE.read_text(encoding="utf-8"))
    if payload.get("schema_version") != "reason-mcp-identity-context-v3-holdout-v1":
        raise SystemExit("unexpected holdout schema")
    if payload.get("trials") != 2 or payload.get("model") != "ministral-8b-latest":
        raise SystemExit("unexpected frozen holdout execution coordinate")

    candidate_sha = payload["candidate_semantics_sha"]
    metric_sha = payload["metric_evaluator_sha"]

    text = PATH.read_text(encoding="utf-8")
    text = one(
        text,
        'use clap::Parser;\n',
        'use clap::{Parser, ValueEnum};\n',
        "clap-value-enum",
    )
    text = one(
        text,
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v13";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v14";\n',
        "schema-v14",
    )

    old_args = '''#[derive(Debug, Parser)]\n#[command(name = "reason-mcp-identity-gate-benchmark")]\nstruct Args {\n    #[arg(long, default_value_t = 3)]\n    trials: usize,\n    #[arg(long, default_value = "ministral-8b-latest")]\n    model: String,\n}\n'''
    new_args = f'''#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]\nenum Suite {{\n    FreshDev,\n    Holdout,\n}}\n\nimpl Suite {{\n    fn report_label(self) -> &'static str {{\n        match self {{\n            Self::FreshDev => "fresh_dev",\n            Self::Holdout => "fresh_holdout_v1",\n        }}\n    }}\n}}\n\nconst HOLDOUT_CANDIDATE_SEMANTICS_SHA: &str = {rust_string(candidate_sha)};\nconst HOLDOUT_METRIC_EVALUATOR_SHA: &str = {rust_string(metric_sha)};\nconst HOLDOUT_TRIALS: usize = 2;\nconst HOLDOUT_MODEL: &str = "ministral-8b-latest";\n\n#[derive(Debug, Parser)]\n#[command(name = "reason-mcp-identity-gate-benchmark")]\nstruct Args {{\n    #[arg(long, value_enum, default_value = "fresh-dev")]\n    suite: Suite,\n    #[arg(long, default_value_t = 3)]\n    trials: usize,\n    #[arg(long, default_value = "ministral-8b-latest")]\n    model: String,\n}}\n'''
    text = one(text, old_args, new_args, "suite-args")
    text = insert_after_fn(text, "dev_cases", holdout_cases_source(payload))

    text = one(
        text,
        '    model: String,\n    prior_frozen_holdout_reused: bool,\n',
        '    model: String,\n    candidate_semantics_sha: &\'static str,\n    metric_evaluator_sha: &\'static str,\n    prior_frozen_holdout_reused: bool,\n',
        "report-freeze-fields",
    )

    text = one(
        text,
        '''    if args.trials == 0 {\n        return Err("--trials must be at least 1".into());\n    }\n    let adapter = MistralAdapter::from_env(&args.model)?;\n    let cases = dev_cases().to_vec();\n''',
        '''    if args.trials == 0 {\n        return Err("--trials must be at least 1".into());\n    }\n    if args.suite == Suite::Holdout {\n        if args.trials != HOLDOUT_TRIALS {\n            return Err(format!("holdout requires --trials {HOLDOUT_TRIALS}").into());\n        }\n        if args.model != HOLDOUT_MODEL {\n            return Err(format!("holdout requires --model {HOLDOUT_MODEL}").into());\n        }\n    }\n    let adapter = MistralAdapter::from_env(&args.model)?;\n    let cases = match args.suite {\n        Suite::FreshDev => dev_cases().to_vec(),\n        Suite::Holdout => holdout_cases().to_vec(),\n    };\n''',
        "suite-selection",
    )
    text = one(text, '        suite: "fresh_dev",\n', '        suite: args.suite.report_label(),\n', "report-suite")
    text = one(
        text,
        '        model: args.model,\n        prior_frozen_holdout_reused: false,\n',
        '        model: args.model,\n        candidate_semantics_sha: HOLDOUT_CANDIDATE_SEMANTICS_SHA,\n        metric_evaluator_sha: HOLDOUT_METRIC_EVALUATOR_SHA,\n        prior_frozen_holdout_reused: false,\n',
        "report-freeze-values",
    )
    text = one(
        text,
        '        evaluation_policy: "fresh Issue #196 development split only; historical #193 and #195 frozen holdouts are not executed, replayed, or used for tuning; terminal wrong answers are counted symmetrically as semantic_false_decisions; capability abstentions count both expected Accept->Unknown and expected Reject->Unknown symmetrically; context-unverified fact admission is a hard safety violation",\n',
        '        evaluation_policy: match args.suite {\n            Suite::FreshDev => "fresh Issue #196 development split only; historical #193 and #195 frozen holdouts are not executed, replayed, or used for tuning; terminal wrong answers are counted symmetrically as semantic_false_decisions; capability abstentions count both expected Accept->Unknown and expected Reject->Unknown symmetrically; context-unverified fact admission is a hard safety violation",\n            Suite::Holdout => "new independent Issue #196 fresh holdout v1 assigned before observation; historical #193/#195 holdouts are not executed/replayed/reused; candidate semantics, prompt, budgets, stop rules, and v13 metrics are frozen; observation is one-shot and post-observation retuning is forbidden",\n        },\n',
        "suite-evaluation-policy",
    )

    marker = '    #[test]\n    fn repair_limit_exhaustion_remains_safe_unknown() {\n'
    tests = '''    #[test]\n    fn fresh_holdout_shape_and_dev_disjointness_are_frozen() {\n        let dev = dev_cases();\n        let holdout = holdout_cases();\n        let dev_surfaces = dev.iter().map(|case| normalize_query(case.initial_query)).collect::<BTreeSet<_>>();\n        let holdout_surfaces = holdout.iter().map(|case| normalize_query(case.initial_query)).collect::<BTreeSet<_>>();\n        assert_eq!(holdout.len(), 8);\n        assert_eq!(holdout_surfaces.len(), holdout.len());\n        assert!(dev_surfaces.is_disjoint(&holdout_surfaces));\n        assert_eq!(holdout.iter().filter(|case| case.expected == ExpectedOutcome::Accept).count(), 5);\n        assert_eq!(holdout.iter().filter(|case| case.expected == ExpectedOutcome::Reject).count(), 1);\n        assert_eq!(holdout.iter().filter(|case| case.expected == ExpectedOutcome::Unknown).count(), 2);\n        assert_eq!(HOLDOUT_TRIALS, 2);\n        assert_eq!(HOLDOUT_MODEL, "ministral-8b-latest");\n        assert_eq!(HOLDOUT_CANDIDATE_SEMANTICS_SHA.len(), 40);\n        assert_eq!(HOLDOUT_METRIC_EVALUATOR_SHA.len(), 40);\n    }\n\n'''
    text = one(text, marker, tests + marker, "holdout-contract-test")

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
