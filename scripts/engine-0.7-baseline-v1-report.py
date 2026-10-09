#!/usr/bin/env python3
"""Validate and summarize frozen Engine 0.6.1 -> Engine 0.7 dev baseline output.

A *development* baseline report, not independent model acceptance. No network access,
provider API keys, model calls, persisted session mutation, or retroactive rescore.
"""

import argparse
import hashlib
import json
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = ROOT / "fixtures/engine-0.7-baseline-v1/manifest.json"
SPEC_SHA = ROOT / "fixtures/engine-0.7-baseline-v1/manifest.sha256"
FROZEN_SHA256 = "3bf35129ab786e7b069c6b8805b92c42a9e1b9cd10e3752c0b60f8f7fe66b268"
FROZEN_COMMIT = "4abee90f0501f0d8fc09b453a0b45c942fbed69e"
CASE_PREFIX = "ENGINE_070_BASELINE_CASE "
SUMMARY_PREFIX = "ENGINE_070_BASELINE_SUMMARY "


def verified_spec():
    raw = SPEC.read_bytes()
    actual = hashlib.sha256(raw).hexdigest()
    assert actual == FROZEN_SHA256, f"immutable fixture changed: {actual}"
    assert SPEC_SHA.read_text() == f"{FROZEN_SHA256}  manifest.json\n"
    spec = json.loads(raw)
    assert spec["schema"] == "engine-0.7-baseline-spec-v1"
    assert spec["engine_version"] == "0.6.1"
    assert spec["engine_baseline_commit"] == FROZEN_COMMIT
    cases = spec["cases"]
    assert len(cases) == 29
    ids = [case["id"] for case in cases]
    assert len(set(ids)) == len(ids)
    return spec


def measured_cases(log):
    cases, summaries = {}, []
    for line in log.splitlines():
        # Rust libtest may print a concurrent test result on the same line.
        # Accept only that known harness prefix; never silently drop a case.
        marker_at = line.find(CASE_PREFIX)
        if marker_at >= 0 and (marker_at == 0 or line.startswith("test ")):
            case = json.loads(line[marker_at + len(CASE_PREFIX):])
            assert case["id"] not in cases, f"duplicate measured case: {case['id']}"
            cases[case["id"]] = case
        elif line.startswith(SUMMARY_PREFIX):
            summaries.append(json.loads(line[len(SUMMARY_PREFIX):]))
    assert len(summaries) == 1, f"exactly one summary required, got {len(summaries)}"
    return cases, summaries[0]


def summarize(spec, cases, aggregate):
    expected_cases = spec["cases"]
    assert set(cases) == {case["id"] for case in expected_cases}
    assert aggregate["engine_version"] == "0.6.1"
    assert aggregate["compared_to_frozen_0_6_1"] is True
    assert aggregate["cases"] == len(cases) == 29
    assert aggregate["matches_predicted_baseline"] == len(cases)
    for name in ("model_calls", "provider_attempts", "external_acquisition_calls", "provider_failures"):
        assert aggregate[name] == 0, f"non-deterministic baseline activity: {name}"

    gaps = Counter()
    family = Counter()
    for expected in expected_cases:
        record = cases[expected["id"]]
        assert record["id"] == expected["id"]
        assert record["family"] == expected["family"]
        assert record["gap_hypothesis"] == expected["gap_hypothesis"]
        assert record["predicted_baseline"] == record["observed"] == expected["expected_baseline"]
        assert record["baseline_prediction_matches"] is True
        family[record["family"]] += 1
        if record["gap_hypothesis"] != "none":
            gaps[record["gap_hypothesis"]] += 1
        if record["family"] == "source":
            assert record["diagnostics"]["hard_verification_receipts"] == 0
            assert record["diagnostics"]["source_local_only"] is True
            assert record["diagnostics"]["origin_oracle_passed_to_engine"] is False
        if record["family"] == "need":
            assert record["diagnostics"]["no_external_acquisition_executed"] is True

    assert dict(family) == {"source": 10, "qualification": 11, "need": 8}
    assert dict(gaps) == {
        "lineage_not_encoded": 3,
        "supersession_not_encoded": 1,
        "wording_difference_false_conflict_candidate": 1,
    }
    # The human-assessed paraphrase pair was pre-labeled as equivalent.
    # This measures an *over-conservative outcome*, not unsupported factual authority.
    paraphrase = cases["src-04"]
    assert paraphrase["observed"]["status"] == "conflict"
    assert paraphrase["observed"]["citations"] == 2

    return {
        "schema": "engine-0.7-development-baseline-result-v1",
        "status": "PASS_PRECOMMITTED_0_6_1_PREDICTIONS",
        "scope": "deterministic development only; no independent provider/semantic acceptance",
        "baseline": {
            "tag": spec["exact_engine_baseline_tag"],
            "core_commit": FROZEN_COMMIT,
            "engine_version": "0.6.1",
            "frozen_fixture_sha256": FROZEN_SHA256,
            "spec_freeze_tag": "engine-0.7-baseline-v1-spec-freeze",
        },
        "scorecard": {
            "cases": 29,
            "predicted_behavior_matches": 29,
            "family_counts": dict(sorted(family.items())),
            "unchanged_behavior_negative_controls": 29 - sum(gaps.values()),
            "candidate_gap_scenarios": dict(sorted(gaps.items())),
            "confirmed_conservative_paraphrase_conflict_case_ids": ["src-04"],
            "measured_unsupported_truth_promotions_in_source_only_path": 0,
            "model_calls": 0,
            "provider_attempts": 0,
            "external_acquisition_calls": 0,
            "provider_failures": 0,
            "no_claimed_real_world_utility_score": True,
        },
        "case_results": [
            {
                "id": fixture["id"],
                "family": fixture["family"],
                "gap_hypothesis": fixture["gap_hypothesis"],
                "observed": cases[fixture["id"]]["observed"],
                "diagnostics": cases[fixture["id"]]["diagnostics"],
            }
            for fixture in expected_cases
        ],
        "interpretation": {
            "lineage": "Three oracle-labeled source scenarios cannot pass their origin lineage into the existing typed attribution contract. Two citations are NOT an assertion of two independent sources; no false-corroboration violation is claimed.",
            "temporal_revision": "Different version strings / later retrieval times do not establish trusted supersession. The observed conflict is safe, and this spec alone does not demonstrate a release-blocking temporal bug.",
            "paraphrase": "An oracle-labeled semantically equivalent wording pair generates source-local Conflict and two exact citations. This supports investigating avoidable qualified-output conflict; no incorrect Known/Supported conclusion was generated.",
            "answerability": "Eight evidence-need cases behaved as predicted, including stale and ambiguous reacquisition. Broader end-to-end required-information integration needs further controlled evaluation before new mechanism work is justified.",
            "limits": "No live LLM or complete CLI task was exercised. The 29/29 metric indicates deterministic baseline reproducibility, not future Engine 0.7.0 utility, general-world reliability or independent holdout success."
        }
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--log", type=Path, required=True, help="cargo test -- --nocapture output")
    parser.add_argument("--output", type=Path, help="validated, deterministic report JSON file")
    args = parser.parse_args()
    spec = verified_spec()
    log_text = args.log.read_text(encoding="utf-8")
    assert "test result: ok. 2 passed; 0 failed;" in log_text, (
        "full Rust measurement + metamorphic guard tests did not both complete successfully"
    )
    cases, summary = measured_cases(log_text)
    result = summarize(spec, cases, summary)
    content = json.dumps(result, ensure_ascii=False, sort_keys=True, indent=2) + "\n"
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(content, encoding="utf-8")
        print(f"wrote report: {args.output}")
    print(
        f"PASS: {result['scorecard']['predicted_behavior_matches']}/29 frozen 0.6.1 "
        "development predictions; 1 conservative paraphrase conflict observed; "
        "0 model/provider/acquisition calls"
    )


if __name__ == "__main__":
    main()
