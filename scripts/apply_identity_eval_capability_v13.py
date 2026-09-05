#!/usr/bin/env python3
"""Issue #196 v13: symmetric capability-abstention accounting only.

This transformer does not change identity admission, planner policy, tool execution,
budgets, or stop semantics. It makes expected Accept/Reject -> Unknown misses visible
symmetrically before the next holdout freeze.
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
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v12";\n',
        'const REPORT_SCHEMA: &str = "reason-mcp-identity-context-gate-benchmark-v13";\n',
        "schema-v13",
    )
    text = one(
        text,
        '    false_abstentions: usize,\n    semantic_false_decisions: usize,\n',
        '    false_abstentions: usize,\n    capability_abstentions: usize,\n    accept_capability_abstentions: usize,\n    reject_capability_abstentions: usize,\n    semantic_false_decisions: usize,\n',
        "aggregate-capability-fields",
    )
    text = one(
        text,
        'fn aggregate(samples: &[CaseReport], cases: usize, trials: usize) -> Aggregate {\n',
        'fn is_capability_abstention(expected: ExpectedOutcome, final_outcome: ExpectedOutcome) -> bool {\n    matches!(expected, ExpectedOutcome::Accept | ExpectedOutcome::Reject)\n        && final_outcome == ExpectedOutcome::Unknown\n}\n\nfn aggregate(samples: &[CaseReport], cases: usize, trials: usize) -> Aggregate {\n',
        "capability-helper",
    )
    text = one(
        text,
        '''    let false_abstentions = samples\n        .iter()\n        .filter(|sample| {\n            sample.expected == ExpectedOutcome::Accept\n                && sample.final_outcome == ExpectedOutcome::Unknown\n                && !sample.operational_failure\n        })\n        .count();\n''',
        '''    let false_abstentions = samples\n        .iter()\n        .filter(|sample| {\n            sample.expected == ExpectedOutcome::Accept\n                && sample.final_outcome == ExpectedOutcome::Unknown\n                && !sample.operational_failure\n        })\n        .count();\n    let accept_capability_abstentions = false_abstentions;\n    let reject_capability_abstentions = samples\n        .iter()\n        .filter(|sample| {\n            sample.expected == ExpectedOutcome::Reject\n                && is_capability_abstention(sample.expected, sample.final_outcome)\n                && !sample.operational_failure\n        })\n        .count();\n    let capability_abstentions = accept_capability_abstentions + reject_capability_abstentions;\n''',
        "aggregate-capability-calculation",
    )
    text = one(
        text,
        '        false_abstentions,\n        semantic_false_decisions,\n',
        '        false_abstentions,\n        capability_abstentions,\n        accept_capability_abstentions,\n        reject_capability_abstentions,\n        semantic_false_decisions,\n',
        "aggregate-capability-values",
    )
    marker = '    #[test]\n    fn unknown_expectation_treats_any_fact_decision_as_false_acceptance() {\n'
    test = '''    #[test]
    fn capability_abstention_predicate_is_symmetric_for_accept_and_reject() {
        assert!(is_capability_abstention(ExpectedOutcome::Accept, ExpectedOutcome::Unknown));
        assert!(is_capability_abstention(ExpectedOutcome::Reject, ExpectedOutcome::Unknown));
        assert!(!is_capability_abstention(ExpectedOutcome::Unknown, ExpectedOutcome::Unknown));
        assert!(!is_capability_abstention(ExpectedOutcome::Accept, ExpectedOutcome::Reject));
        assert!(!is_capability_abstention(ExpectedOutcome::Reject, ExpectedOutcome::Accept));
    }

'''
    text = one(text, marker, test + marker, "capability-accounting-test")
    text = one(
        text,
        '        evaluation_policy: "fresh Issue #196 development split only; historical #193 and #195 frozen holdouts are not executed, replayed, or used for tuning; terminal wrong answers are counted symmetrically as semantic_false_decisions; context-unverified fact admission is a hard safety violation",\n',
        '        evaluation_policy: "fresh Issue #196 development split only; historical #193 and #195 frozen holdouts are not executed, replayed, or used for tuning; terminal wrong answers are counted symmetrically as semantic_false_decisions; capability abstentions count both expected Accept->Unknown and expected Reject->Unknown symmetrically; context-unverified fact admission is a hard safety violation",\n',
        "reported-v13-evaluation-policy",
    )
    PATH.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
