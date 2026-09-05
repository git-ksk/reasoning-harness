#!/usr/bin/env python3
from pathlib import Path

PATH = Path("crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs")


def one(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{label}: expected exactly one match, found {count}")
    return text.replace(old, new, 1)


def main() -> int:
    text = PATH.read_text(encoding="utf-8")
    text = one(text,
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v7";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-gate-benchmark-v8";\n',
        "schema-v8")
    text = one(text,
        'fn aggregate(samples: &[CaseReport], cases: usize, trials: usize) -> Aggregate {\n',
        'fn is_false_acceptance(expected: ExpectedOutcome, final_outcome: ExpectedOutcome) -> bool {\n    match expected {\n        ExpectedOutcome::Accept => false,\n        ExpectedOutcome::Reject => final_outcome == ExpectedOutcome::Accept,\n        ExpectedOutcome::Unknown => final_outcome != ExpectedOutcome::Unknown,\n    }\n}\n\nfn aggregate(samples: &[CaseReport], cases: usize, trials: usize) -> Aggregate {\n',
        "false-acceptance-helper")
    text = one(text,
        '    let false_acceptances = samples\n        .iter()\n        .filter(|sample| {\n            sample.expected != ExpectedOutcome::Accept\n                && sample.final_outcome == ExpectedOutcome::Accept\n        })\n        .count();\n',
        '    let false_acceptances = samples\n        .iter()\n        .filter(|sample| is_false_acceptance(sample.expected, sample.final_outcome))\n        .count();\n',
        "aggregate-false-acceptance-semantics")
    text = one(text,
        '    #[test]\n    fn repair_limit_exhaustion_remains_safe_unknown() {\n',
        '    #[test]\n    fn unknown_expectation_treats_any_fact_decision_as_false_acceptance() {\n        assert!(is_false_acceptance(ExpectedOutcome::Unknown, ExpectedOutcome::Accept));\n        assert!(is_false_acceptance(ExpectedOutcome::Unknown, ExpectedOutcome::Reject));\n        assert!(!is_false_acceptance(ExpectedOutcome::Unknown, ExpectedOutcome::Unknown));\n        assert!(is_false_acceptance(ExpectedOutcome::Reject, ExpectedOutcome::Accept));\n        assert!(!is_false_acceptance(ExpectedOutcome::Reject, ExpectedOutcome::Reject));\n        assert!(!is_false_acceptance(ExpectedOutcome::Accept, ExpectedOutcome::Accept));\n    }\n\n    #[test]\n    fn repair_limit_exhaustion_remains_safe_unknown() {\n',
        "semantic-safety-test")
    text = one(text,
        '        evaluation_policy: "fresh development cases only; prior #193 holdout is not executed or used for tuning; a new holdout will be frozen only after this candidate stabilizes",\n',
        '        evaluation_policy: "fresh development cases only; prior #193 holdout is not executed or used for tuning; expected unknown treats any fact-level Accept/Reject decision as a semantic false acceptance; a new holdout will be frozen only after this candidate stabilizes",\n',
        "reported-evaluation-policy")
    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
