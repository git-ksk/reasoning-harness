#!/usr/bin/env python3
"""Fail-closed deterministic development report for local source reviewer #496.

This report does not establish real-user authentication, production utility,
provider-model acceptance, or an independent holdout.
"""
import argparse
import collections
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "fixtures/engine-0.7-reviewer-host-development-v1/manifest.json"
CHECKSUM = SPEC.parent / "manifest.sha256"
FROZEN = "cc0b0acc4bb53ac94eaeb0d60489b225691f2ec5427a5bbf5398da3e7e35bee4"
FROZEN_COMMIT = "fa87aac3c6fc602c2874b604f6e027a4d093784a"
CASE_MARKER = "ENGINE_070_HOST_REVIEW_CASE "
SUMMARY_MARKER = "ENGINE_070_HOST_REVIEW_SUMMARY "

def load_spec():
    raw = SPEC.read_bytes()
    assert hashlib.sha256(raw).hexdigest() == FROZEN, "precommit fixture mutated"
    assert CHECKSUM.read_text() == FROZEN + "  manifest.json\n"
    spec = json.loads(raw)
    assert spec["schema"] == "engine-0.7-local-reviewer-host-development-v1-spec"
    assert len(spec["cases"]) == 18
    assert len({x["id"] for x in spec["cases"]}) == 18
    return spec

def parse(log):
    cases = {}
    summaries = []
    for line in log.splitlines():
        for marker, destination in ((CASE_MARKER, "case"), (SUMMARY_MARKER, "summary")):
            idx = line.find(marker)
            if idx < 0 or (idx != 0 and not line.startswith("test ")):
                continue
            item = json.loads(line[idx + len(marker):])
            if destination == "case":
                assert item["id"] not in cases, "duplicate observed case"
                cases[item["id"]] = item
            else:
                summaries.append(item)
    assert len(summaries) == 1
    assert any(f"test result: ok. {count} passed; 0 failed;" in log for count in (4, 6, 8)), "Rust suite did not pass"
    return cases, summaries[0]

def score(spec, observations, summary):
    assert set(observations) == {x["id"] for x in spec["cases"]}
    assert summary["cases"] == summary["passed"] == 18
    for key in ("model_calls", "provider_attempts", "external_acquisition", "truth_promotions"):
        assert summary[key] == 0, f"{key} is not zero"
    counts = collections.Counter()
    rows = []
    for fixture in spec["cases"]:
        obs = observations[fixture["id"]]
        assert obs["id"] == fixture["id"]
        assert obs["expected"] == obs["observed"] == fixture["expected"]
        counts[obs["observed"]] += 1
        rows.append({"id": fixture["id"], "scenario": fixture["theme"],
                     "expected": fixture["expected"], "observed": obs["observed"]})
    assert counts == {
        "reject": 13,
        "reviewed_compatible": 2,
        "conflict": 2,
        "qualified": 1,
    }
    assert observations["human-equivalent"]["observed"] == "reviewed_compatible"
    assert observations["no-review"]["observed"] == "conflict"
    assert observations["opposed-claims"]["observed"] == "reject"
    assert observations["revocation"]["observed"] == "reject"
    assert observations["headless"]["observed"] == "reject"
    return {
        "schema": "engine-0.7-local-reviewer-host-development-v1-result",
        "status": "PASS_DETERMINISTIC_DEVELOPMENT_ONLY",
        "source_baseline": "engine-v0.6.1",
        "precommit_spec_freeze_tag": "engine-0.7-reviewer-host-dev-v1-spec-freeze",
        "precommit_spec_commit": FROZEN_COMMIT,
        "spec_sha256": FROZEN,
        "scorecard": {
            "cases": 18,
            "passed_precommitted_expectations": 18,
            "outcomes": dict(sorted(counts.items())),
            "independent_model_holdout": "NOT_STARTED",
            "physical_keyring_enrollment": "NOT_EXECUTED",
            "external_model_calls": 0,
            "provider_attempts": 0,
            "external_acquisition_calls": 0,
            "hard_truth_promotions": 0,
            "development_display_improvements": 1,
            "other_compatible_case_is_replay_control": True
        },
        "hard_limitations": [
            "Keys were injected into deterministic tests; no real user's OS keyring was enrolled.",
            "The local logged-in OS account is the trust boundary; malicious programs running as the same user are not excluded.",
            "The host receives deliberate human input, not an external authenticator or independent model semantic evaluator.",
            "A local manually approved compatible wording classification does not verify source accuracy or independent corroboration.",
            "Reason CLI 0.5.4 public binaries and Engine source release 0.6.1 remain unchanged.",
            "No independent Mistral, Google or Groq live acceptance has yet occurred."
        ],
        "case_results": rows
    }

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--log", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    report = score(load_spec(), *parse(args.log.read_text()))
    data = json.dumps(report, indent=2, ensure_ascii=False, sort_keys=True) + "\n"
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(data)
    print("PASS: precommitted local reviewer host development scenarios 18/18")
    print("result:", args.output)

if __name__ == "__main__":
    main()
