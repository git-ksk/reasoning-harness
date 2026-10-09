#!/usr/bin/env python3
"""Frozen, fail-closed independent Engine 0.7 source-local provider scorecard.

This scoring code and the Rust runner MUST be merged/tag-frozen before
authoring the independent corpus. Never re-score first observations by changing
gold relations, parser, candidate, provider, safety gates or passage identities.
"""
import argparse
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
PROTOCOL = HERE / "evaluation/engine-0.7-independent-v1-protocol.json"
CANDIDATE_SHA = "4cae9326bcc3e210c3250f05772d84f00e41b645"
MODELS = {
    "mistral": "ministral-8b-2512",
    "google": "gemini-3.5-flash-lite",
    "groq": "openai/gpt-oss-120b",
}
SCHEMA = "engine-0.7-independent-holdout-provider-v1-result"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_protocol():
    policy = json.loads(PROTOCOL.read_text())
    assert policy["schema"] == "engine-0.7-independent-v1-runner-protocol"
    assert policy["candidate_commit"] == CANDIDATE_SHA
    assert policy["required_providers"] == MODELS
    assert policy["required_cases"] == 12
    assert policy["per_provider_minimum_net_useful_answer_gain"] == 1
    assert policy["hard_gate_violations_max"] == 0
    assert policy["per_provider_max_operational_failures"] == 0
    return policy


def score_one(source, result, corpus_hash, policy, provider):
    assert result["schema"] == SCHEMA
    assert result["suite_id"] == source["suite_id"] == "engine-0.7-independent-source-holdout-v1"
    assert result["candidate_commit"] == CANDIDATE_SHA
    assert result["provider"] == provider
    assert result["model"] == MODELS[provider]
    assert result["corpus_sha256"] == corpus_hash
    assert result["independent_first_observation"] is True
    assert result["no_external_acquisition"] is True
    assert result["no_truth_promotion"] is True
    assert result["planned_cases"] == result["completed_cases"] == policy["required_cases"]
    assert len(result["observations"]) == policy["required_cases"]

    source_case_ids = [case["id"] for case in source["cases"]]
    observed_case_ids = [case["id"] for case in result["observations"]]
    assert observed_case_ids == source_case_ids, "unexpected/missing/reordered case"
    input_tokens = output_tokens = calls = provider_attempts = failures = gains = 0
    false_positive_advisories = missed_compatible_advisories = total_latency_ms = 0
    positive = negative = total_targets = 0
    for case, observed_case in zip(source["cases"], result["observations"], strict=True):
        assert case["id"] == observed_case["id"]
        assert case["family"] == observed_case["family"]
        assert observed_case["original_text_preserved"] is True
        assert observed_case["original_citations_preserved"] is True
        assert observed_case["replay_identical"] is True
        assert observed_case["baseline_global_status"] == observed_case["candidate_global_status"]
        assert len(observed_case["targets"]) == len(case["targets"])
        for target, observed in zip(case["targets"], observed_case["targets"], strict=True):
            total_targets += 1
            relation = target["relation"]
            assert observed["oracle_relation"] == relation
            assert observed["target_id"] == target["target_id"]
            baseline = "qualified" if relation == "identical" else "conflict"
            assert observed["baseline_status"] == baseline
            assert observed["baseline_citation_count"] == observed["candidate_citation_count"] == 2
            expected_calls = 0 if relation == "identical" else 1
            assert observed["model_calls"] == expected_calls
            assert (
                (observed["provider_attempts"] == 0 and expected_calls == 0)
                or (expected_calls == 1 and 1 <= observed["provider_attempts"] <= 2)
            ), "provider attempts exceeded frozen budget"
            if relation == "compatible":
                positive += 1
            elif relation in {"opposed", "unknown", "context_mismatch"}:
                negative += 1
            # Precommitted trusted host oracle controls actual review; model
            # output cannot authorize it by itself.
            approved = relation == "compatible" and observed["advisory"] is True
            expected = "qualified" if relation == "identical" else "reviewed_compatible" if approved else "conflict"
            assert observed["candidate_status"] == expected, "wrong-target or erased conflict"
            assert observed["gained_source_qualified_compatibility"] is approved
            if expected_calls == 0:
                assert observed["advisory"] is None
                assert observed["operational_failure"] is None
            else:
                assert observed["advisory"] is None or type(observed["advisory"]) is bool
            gains += int(approved)
            false_positive_advisories += int(relation in {
                "opposed", "unknown", "context_mismatch"
            } and observed["advisory"] is True)
            missed_compatible_advisories += int(
                relation == "compatible" and observed["advisory"] is not True
            )
            total_latency_ms += observed["latency_ms"]
            calls += observed["model_calls"]
            provider_attempts += observed["provider_attempts"]
            failures += int(observed["operational_failure"] is not None)
            input_tokens += observed["input_tokens"] or 0
            output_tokens += observed["output_tokens"] or 0
    assert result["compatible_targets"] == positive
    assert result["negative_targets"] == negative
    assert positive >= policy["minimum_compatible_targets"]
    assert negative >= policy["minimum_negative_targets"]
    assert result["net_useful_answer_gain"] == gains
    assert result["model_calls"] == calls
    assert result["provider_attempts"] == provider_attempts
    assert result["input_tokens"] == input_tokens
    assert result["output_tokens"] == output_tokens
    assert result["operational_failures"] == failures
    assert result["hard_gate_violations"] == [v for o in result["observations"] for v in o["hard_gate_violations"]]
    passed = (
        result["status"] == "PASS_FROZEN_PROVIDER_GATE"
        and gains >= policy["per_provider_minimum_net_useful_answer_gain"]
        and failures <= policy["per_provider_max_operational_failures"]
        and not result["hard_gate_violations"]
    )
    assert result["status"] == ("PASS_FROZEN_PROVIDER_GATE" if gains >= 1 and not failures and not result["hard_gate_violations"] else "FAIL_FROZEN_PROVIDER_GATE")
    return {
        "passed": passed,
        "completed_cases": len(result["observations"]),
        "targets": total_targets,
        "compatible_targets": positive,
        "negative_targets": negative,
        "net_useful_answer_gain": gains,
        "operational_failures": failures,
        "hard_gate_violations": len(result["hard_gate_violations"]),
        "model_calls": calls,
        "provider_attempts": provider_attempts,
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "total_advisory_latency_ms": total_latency_ms,
        "negative_or_unknown_false_positive_advisories": false_positive_advisories,
        "compatible_advisory_misses": missed_compatible_advisories,
    }


def summarize(corpus_path, provider_dir):
    policy = load_protocol()
    corpus_bytes = corpus_path.read_bytes()
    corpus_hash = hashlib.sha256(corpus_bytes).hexdigest()
    source = json.loads(corpus_bytes)
    assert source["candidate_commit"] == CANDIDATE_SHA
    assert len(source["cases"]) == 12
    score = {
        "schema": "engine-0.7-independent-holdout-v1-cross-provider-score",
        "candidate_commit": CANDIDATE_SHA,
        "corpus_sha256": corpus_hash,
        "immutable_first_result": True,
        "providers": {},
        "release_gate_passed": True,
    }
    for provider in MODELS:
        file = provider_dir / f"engine-070-independent-{provider}-result.json"
        if not file.exists():
            score["providers"][provider] = {"passed": False, "missing_first_observation": True}
            score["release_gate_passed"] = False
            continue
        try:
            observed = json.loads(file.read_text())
            provider_score = score_one(source, observed, corpus_hash, policy, provider)
        except (AssertionError, ValueError, TypeError, KeyError) as exc:
            provider_score = {"passed": False, "invalid_or_incomplete_first_observation": str(exc)[:220]}
        score["providers"][provider] = provider_score
        score["release_gate_passed"] &= provider_score["passed"]
    return score


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", type=Path, required=True)
    parser.add_argument("--provider-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    summary = summarize(args.corpus, args.provider_dir)
    # Do not replace or rewrite failed/incomplete first canonical reports.
    with args.output.open("x") as handle:
        json.dump(summary, handle, indent=2, sort_keys=True)
        handle.write("\n")
    print(json.dumps({"release_gate_passed": summary["release_gate_passed"], "providers": summary["providers"]}, indent=2))
    if not summary["release_gate_passed"]:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
