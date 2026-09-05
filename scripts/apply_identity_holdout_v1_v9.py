#!/usr/bin/env python3
"""Add the observation-free fresh holdout surface after the frozen v8 candidate.

This transformer changes evaluation surface selection only. Identity admission,
planner policy, budgets, stop rules, and tool execution remain the v8 candidate.
"""
from __future__ import annotations

import json
from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")
FIXTURE = Path("fixtures/mcp-identity-gate-holdout-v1.json")


def one(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


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
    if payload.get("schema_version") != "reason-mcp-identity-gate-holdout-v1":
        raise SystemExit("unexpected holdout schema")
    if payload.get("trials") != 2 or payload.get("model") != "ministral-8b-latest":
        raise SystemExit("unexpected frozen holdout execution coordinate")
    candidate_sha = payload["frozen_candidate_sha"]

    text = PATH.read_text(encoding="utf-8")
    text = one(text,
        'use clap::Parser;\n',
        'use clap::{Parser, ValueEnum};\n',
        "clap-value-enum")
    text = one(text,
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v8";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v9";\n',
        "schema-v9")

    old_args = '''#[derive(Debug, Parser)]\n#[command(name = "reason-mcp-identity-gate-benchmark")]\nstruct Args {\n    #[arg(long, default_value_t = 3)]\n    trials: usize,\n    #[arg(long, default_value = "ministral-8b-latest")]\n    model: String,\n}\n'''
    new_args = f'''#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]\nenum Suite {{\n    FreshDev,\n    Holdout,\n}}\n\nimpl Suite {{\n    fn report_label(self) -> &'static str {{\n        match self {{\n            Self::FreshDev => "fresh_dev",\n            Self::Holdout => "fresh_holdout_v1",\n        }}\n    }}\n}}\n\nconst HOLDOUT_FROZEN_CANDIDATE_SHA: &str = {rust_string(candidate_sha)};\nconst HOLDOUT_TRIALS: usize = 2;\nconst HOLDOUT_MODEL: &str = "ministral-8b-latest";\n\n#[derive(Debug, Parser)]\n#[command(name = "reason-mcp-identity-gate-benchmark")]\nstruct Args {{\n    #[arg(long, value_enum, default_value = "fresh-dev")]\n    suite: Suite,\n    #[arg(long, default_value_t = 3)]\n    trials: usize,\n    #[arg(long, default_value = "ministral-8b-latest")]\n    model: String,\n}}\n'''
    text = one(text, old_args, new_args, "suite-args")

    stop_marker = '''\n#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]\n#[serde(rename_all = "snake_case")]\nenum StopReason {\n'''
    text = one(text, stop_marker, "\n" + holdout_cases_source(payload) + stop_marker, "holdout-cases")

    text = one(text,
        '    if args.trials == 0 {\n        return Err("--trials must be at least 1".into());\n    }\n    let adapter = MistralAdapter::from_env(&args.model)?;\n    let cases = dev_cases().to_vec();\n',
        '    if args.trials == 0 {\n        return Err("--trials must be at least 1".into());\n    }\n    if args.suite == Suite::Holdout {\n        if args.trials != HOLDOUT_TRIALS {\n            return Err(format!("holdout requires --trials {HOLDOUT_TRIALS}").into());\n        }\n        if args.model != HOLDOUT_MODEL {\n            return Err(format!("holdout requires --model {HOLDOUT_MODEL}").into());\n        }\n    }\n    let adapter = MistralAdapter::from_env(&args.model)?;\n    let cases = match args.suite {\n        Suite::FreshDev => dev_cases().to_vec(),\n        Suite::Holdout => holdout_cases().to_vec(),\n    };\n',
        "suite-selection")

    text = one(text,
        '    model: String,\n    prior_frozen_holdout_reused: bool,\n',
        '    model: String,\n    candidate_freeze_sha: &\'static str,\n    prior_frozen_holdout_reused: bool,\n',
        "report-candidate-freeze-field")
    text = one(text,
        '        suite: "fresh_dev",\n',
        '        suite: args.suite.report_label(),\n',
        "report-suite")
    text = one(text,
        '        model: args.model,\n        prior_frozen_holdout_reused: false,\n',
        '        model: args.model,\n        candidate_freeze_sha: HOLDOUT_FROZEN_CANDIDATE_SHA,\n        prior_frozen_holdout_reused: false,\n',
        "report-candidate-freeze-value")
    text = one(text,
        '        evaluation_policy: "fresh development cases only; prior #193 holdout is not executed or used for tuning; expected unknown treats any fact-level Accept/Reject decision as a semantic false acceptance; a new holdout will be frozen only after this candidate stabilizes",\n',
        '        evaluation_policy: match args.suite {\n            Suite::FreshDev => "fresh development cases only; prior #193 holdout is not executed or used for tuning; expected unknown treats any fact-level Accept/Reject decision as a semantic false acceptance",\n            Suite::Holdout => "fresh independent holdout v1; frozen candidate/prompt/budgets/stop semantics; prior #193 holdout is not executed or used for tuning; this holdout may be observed once and never retuned or rerun",\n        },\n',
        "suite-evaluation-policy")

    test_marker = '''    #[test]\n    fn repair_limit_exhaustion_remains_safe_unknown() {\n'''
    test = '''    #[test]\n    fn fresh_holdout_is_disjoint_unique_and_shape_frozen() {\n        let dev = dev_cases();\n        let holdout = holdout_cases();\n        let dev_surfaces = dev.iter().map(|case| normalize_query(case.initial_query)).collect::<BTreeSet<_>>();\n        let holdout_surfaces = holdout.iter().map(|case| normalize_query(case.initial_query)).collect::<BTreeSet<_>>();\n        assert_eq!(holdout.len(), 8);\n        assert_eq!(holdout_surfaces.len(), holdout.len());\n        assert!(dev_surfaces.is_disjoint(&holdout_surfaces));\n        assert_eq!(holdout.iter().filter(|case| case.expected == ExpectedOutcome::Accept).count(), 4);\n        assert_eq!(holdout.iter().filter(|case| case.expected == ExpectedOutcome::Reject).count(), 2);\n        assert_eq!(holdout.iter().filter(|case| case.expected == ExpectedOutcome::Unknown).count(), 2);\n        assert_eq!(HOLDOUT_TRIALS, 2);\n        assert_eq!(HOLDOUT_MODEL, "ministral-8b-latest");\n        assert_eq!(HOLDOUT_FROZEN_CANDIDATE_SHA.len(), 40);\n    }\n\n    #[test]\n    fn repair_limit_exhaustion_remains_safe_unknown() {\n'''
    text = one(text, test_marker, test, "holdout-contract-test")

    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
