#!/usr/bin/env python3
"""Fail-closed frozen development scorecard for Engine 0.7 source reconciliation (#490).

This is development-only, **not** independent model/holdout acceptance.
"""

import argparse
import collections
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "fixtures/engine-0.7-source-reconciliation-development-v1/manifest.json"
CHECKSUMS = SPEC.parent / "manifest.sha256"
FROZEN_SHA256 = "8d50e04a24d09a2620827b15639a89c396b100dfa5d104baa69b3b9357190714"
FROZEN_COMMIT = "426b70195623d1b4d7196c5d6a1ab4a2272741c6"
CASE = "ENGINE_070_RECONCILIATION_CASE "
SUMMARY = "ENGINE_070_RECONCILIATION_SUMMARY "


def load_spec():
    raw = SPEC.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == FROZEN_SHA256
    assert CHECKSUMS.read_text(encoding="utf-8") == f"{FROZEN_SHA256}  manifest.json\n"
    spec = json.loads(raw)
    assert spec["contract"] == "engine-0.7-source-reconciliation-development-v1-spec"
    assert spec["expected_cases"] == len(spec["cases"]) == 22
    assert len({c["id"] for c in spec["cases"]}) == 22
    return spec


def observe(log):
    cases, summary = {}, []
    for line in log.splitlines():
        idx = line.find(CASE)
        if idx >= 0 and (idx == 0 or line.startswith("test ")):
            item = json.loads(line[idx + len(CASE):])
            assert item["id"] not in cases, f"duplicate result {item['id']}"
            cases[item["id"]] = item
        idx = line.find(SUMMARY)
        if idx >= 0 and (idx == 0 or line.startswith("test ")):
            summary.append(json.loads(line[idx + len(SUMMARY):]))
    assert len(summary) == 1
    assert "test result: ok. 5 passed; 0 failed;" in log
    return cases, summary[0]


def score(spec, cases, summary):
    assert set(cases) == {c["id"] for c in spec["cases"]}
    for key in ("model_calls", "provider_attempts", "external_calls",
                "hard_authority_promotions", "source_state_mutations"):
        assert summary[key] == 0, f"{key} must be zero"
    assert summary["cases"] == summary["passed"] == 22
    status_counts = collections.Counter()
    records = []
    for fixture in spec["cases"]:
        item = cases[fixture["id"]]
        assert item["observed"] == item["expected"] == fixture["expected"]
        status_counts[item["observed"]] += 1
        details = item["details"]
        if item["observed"] != "error":
            assert details["hard_receipts"] == 0
            assert details["replay_equal"] is True
            assert details["citations"] == len(fixture["statements"])
            if item["observed"] == "reviewed_compatible":
                assert details["legacy_conflict_targets"] == 1
                assert details["remaining_conflicts"] == []
                assert details["reviewed_targets"] == ["t1"]
                assert details["legacy_text"].count("According to") == len(fixture["statements"])
            if item["observed"] == "conflict":
                assert details["legacy_conflict_targets"] == 1
        records.append({
            "id": item["id"], "expected": item["expected"],
            "observed": item["observed"], "details": details,
        })
    assert status_counts == {
        "conflict": 9, "error": 7, "reviewed_compatible": 4, "qualified": 2
    }
    # Externally established equivalent wording of two exact admitted quotes
    # improves the *overlay*, while original citations and v1 Conflict survive.
    assert cases["equiv-01"]["observed"] == "reviewed_compatible"
    assert cases["safe-04"]["observed"] == "conflict"
    assert cases["safe-12"]["observed"] == "error"
    return {
        "schema": "engine-0.7-source-reconciliation-development-v1-result",
        "status": "PASS_DETERMINISTIC_DEVELOPMENT_ONLY",
        "provenance": {
            "original_engine_release": "engine-v0.6.1",
            "original_engine_version": "0.6.1",
            "spec_freeze_tag": "engine-0.7-reconciliation-dev-v1-spec-freeze",
            "spec_freeze_commit": FROZEN_COMMIT,
            "spec_sha256": FROZEN_SHA256,
            "independent_holdout": "NOT_STARTED",
        },
        "scorecard": {
            "cases": 22,
            "matched_precommitted_expectations": 22,
            "outcomes": dict(sorted(status_counts.items())),
            "reviewed_compatible_with_original_conflict_unchanged": 4,
            "original_citations_retained": True,
            "legacy_v1_persisted_flags_mutated": False,
            "source_only_zero_hard_truth_promotion": True,
            "model_calls": 0,
            "provider_attempts": 0,
            "external_calls": 0,
        },
        "limitations": [
            "Trusted host review is an explicit external semantic oracle; no model has yet produced or verified independent equivalence.",
            "Only exact source-local quotes, matching explicit source metadata/version and bounded pairwise claims are supported.",
            "No change to the released Engine 0.6.1 or Reason CLI 0.5.4 distribution.",
            "No live Mistral/Google/Groq acceptance, measured model utility, general-domain source conflict accuracy or release authorization.",
        ],
        "case_results": records,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    report = score(load_spec(), *observe(args.log.read_text(encoding="utf-8")))
    formatted = json.dumps(report, indent=2, ensure_ascii=False, sort_keys=True) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(formatted, encoding="utf-8")
        print("report:", args.output)
    print("PASS: 22/22 precommitted deterministic reconciliation controls; legacy v1 unchanged")


if __name__ == "__main__":
    main()
