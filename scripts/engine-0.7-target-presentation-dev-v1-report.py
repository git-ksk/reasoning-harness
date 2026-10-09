#!/usr/bin/env python3
"""Fail-closed frozen synthetic target-presentation development results for #490."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SUITE_PATH = ROOT / "fixtures/engine-0.7-target-presentation-dev-v1/manifest.json"
CHECKSUM_PATH = SUITE_PATH.with_name("manifest.sha256")
SHA256 = "fe27ecc6704992109e8da100235bc4ba5ffbafec1073ebc4fa572993304b4214"
PRECOMMIT_SHA = "f5c098b2f5df2c93f5385a96bde1719c5e313a37"
CASE_MARKER = "ENGINE_070_TARGET_PRESENTATION_CASE "
SUMMARY_MARKER = "ENGINE_070_TARGET_PRESENTATION_SUMMARY "


def spec():
    raw = SUITE_PATH.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == SHA256, "precommit fixture changed"
    assert CHECKSUM_PATH.read_text() == SHA256 + "  fixtures/engine-0.7-target-presentation-dev-v1/manifest.json\n"
    frozen = json.loads(raw)
    assert frozen["schema"] == "engine-0.7-target-presentation-dev-v1-spec"
    assert frozen["issue"] == 490
    assert frozen["kind"] == "synthetic_development_only"
    assert frozen["baseline"] == "engine-v0.6.1"
    assert frozen["precommit"] is True
    assert len(frozen["cases"]) == 14
    assert len({case["id"] for case in frozen["cases"]}) == 14
    return frozen


def parse(log):
    cases, summaries = {}, []
    for line in log.splitlines():
        for marker, kind in ((CASE_MARKER, "case"), (SUMMARY_MARKER, "summary")):
            pos = line.find(marker)
            if pos < 0 or (pos != 0 and not line.startswith("test ")):
                continue
            item = json.loads(line[pos + len(marker):])
            if kind == "case":
                assert item["id"] not in cases, "duplicated observed case"
                cases[item["id"]] = item
            else:
                summaries.append(item)
    assert len(summaries) == 1, "missing or duplicated summary"
    assert "test result: ok. 1 passed; 0 failed;" in log, "Rust runner did not pass"
    return cases, summaries[0]


def score(frozen, observed, summary):
    assert set(observed) == {case["id"] for case in frozen["cases"]}, "case mismatch"
    assert summary["cases"] == summary["passed"] == 14
    for name in ("model_calls", "provider_attempts", "external_acquisition", "truth_promotions"):
        assert summary[name] == 0, f"{name} must be zero"

    details = []
    expected_counts = {"ok": 11, "reject": 3}
    totals = {"ok": 0, "reject": 0}
    useful_cases = 0
    for case in frozen["cases"]:
        got = observed[case["id"]]
        assert got["id"] == case["id"]
        assert got["expected"] == got["observed"] == case["expected_result"]
        totals[got["observed"]] += 1
        result = got["details"]
        if got["observed"] == "reject":
            assert result == {"rejected": True}
        else:
            assert result["target_statuses"] == case["expected_target_statuses"]
            assert result["legacy_status"] == case["expected_original_status"]
            assert result["target_citations"] == case["expected_target_citations"]
            assert result["legacy_citations"] == sum(result["target_citations"])
            assert result["replay_equal"] is True
            if result["legacy_status"] == "conflict" and "reviewed_compatible" in result["target_statuses"]:
                useful_cases += 1
        details.append({"id":case["id"],"expected":case["expected_result"],
                        "observed":got["observed"],"detail":result})
    assert totals == expected_counts
    assert useful_cases == 6, "precommitted useful cases changed"
    return {
        "schema": "engine-0.7-target-presentation-dev-v1-result",
        "status": "PASS_SYNTHETIC_DEVELOPMENT_ONLY",
        "precommit_tag": "engine-0.7-target-presentation-dev-v1-spec-freeze",
        "precommit_commit": PRECOMMIT_SHA,
        "manifest_sha256": SHA256,
        "baseline": "engine-v0.6.1",
        "scorecard": {
            "cases": 14,
            "passed": 14,
            "legacy_conflict_with_reviewed_target": useful_cases,
            "rejected": 3,
            "model_calls": 0,
            "provider_attempts": 0,
            "external_acquisition": 0,
            "truth_promotions": 0,
            "original_citations_preserved": True,
            "replay_preserved": True,
            "independent_provider_holdout": "NOT_EXECUTED",
        },
        "limitations": [
            "Source-local target display only; not evidence of independent external truth.",
            "Development examples are synthetic and cannot be reused for the independent holdout.",
            "Host-authenticated reviewer input is needed to mark a target reviewed compatible.",
            "No published Engine or CLI release is changed."
        ],
        "case_results": details,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--log", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    result = score(spec(), *parse(args.log.read_text()))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, sort_keys=True, indent=2, ensure_ascii=False) + "\n")
    print(f"PASS: precommitted target-local presentation 14/14 (source-local useful cases={result['scorecard']['legacy_conflict_with_reviewed_target']})")
    print("result:", args.output)


if __name__ == "__main__":
    main()
