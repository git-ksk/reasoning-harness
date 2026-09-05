#!/usr/bin/env python3
"""Observation-free preflight for Issue #196 fresh holdout v1."""
from __future__ import annotations

import argparse
import hashlib
import json
import re
from collections import Counter
from pathlib import Path

EXPECTED_SCHEMA = "reason-mcp-identity-context-v3-holdout-v1"
EXPECTED_CLASSES = {
    "unique_bare_accept": 2,
    "trusted_context_agrees": 1,
    "trusted_context_conflicts_bare_candidate": 1,
    "ambiguous_with_trusted_context": 1,
    "ambiguity_without_context": 2,
    "wrong_claim_reject": 1,
}


def normalize(value: str) -> str:
    terms = {term.lower() for term in re.findall(r"[0-9A-Za-z]+", value)}
    return " ".join(sorted(terms))


def digest(value: str) -> str:
    return hashlib.sha256(normalize(value).encode()).hexdigest()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("fixture", type=Path)
    parser.add_argument("--forbidden-digests", type=Path, required=True)
    parser.add_argument("--selection-receipt", type=Path)
    args = parser.parse_args()

    payload = json.loads(args.fixture.read_text(encoding="utf-8"))
    assert payload["schema_version"] == EXPECTED_SCHEMA
    assert re.fullmatch(r"[0-9a-f]{40}", payload["candidate_semantics_sha"])
    assert re.fullmatch(r"[0-9a-f]{40}", payload["metric_evaluator_sha"])
    assert payload["model"] == "ministral-8b-latest"
    assert payload["trials"] == 2
    assert payload["network_observations_before_assignment"] == 0
    assert payload["post_observation_retuning_allowed"] is False

    cases = payload["cases"]
    assert len(cases) == 8
    classes = Counter(case["class"] for case in cases)
    assert dict(classes) == EXPECTED_CLASSES

    ids = [case["id"] for case in cases]
    surfaces = [normalize(case["initial_query"]) for case in cases]
    fact_keys = [case["fact_key"] for case in cases]
    assert len(ids) == len(set(ids))
    assert len(surfaces) == len(set(surfaces))
    assert len(fact_keys) == len(set(fact_keys))

    for case in cases:
        assert case["property_id"] == "P17"
        assert case["value_kind"] == "entity"
        assert case["expected"] in {"accept", "reject", "unknown"}
        if case["class"] in {
            "trusted_context_agrees",
            "trusted_context_conflicts_bare_candidate",
            "ambiguous_with_trusted_context",
        }:
            assert isinstance(case["identity_context"], str) and case["identity_context"].strip()
        else:
            assert case["identity_context"] is None
        if case["class"] == "ambiguity_without_context":
            assert case["expected"] == "unknown"
        elif case["class"] == "wrong_claim_reject":
            assert case["expected"] == "reject"
        else:
            assert case["expected"] == "accept"

    forbidden = {
        line.strip()
        for line in args.forbidden_digests.read_text(encoding="utf-8").splitlines()
        if line.strip()
    }
    assert all(re.fullmatch(r"[0-9a-f]{64}", value) for value in forbidden)
    selected = {digest(case["initial_query"]) for case in cases}
    overlap = selected & forbidden
    assert not overlap, f"holdout normalized surfaces overlap historical/prior-dev digests: {len(overlap)}"

    if args.selection_receipt:
        receipt = json.loads(args.selection_receipt.read_text(encoding="utf-8"))
        assert receipt["schema_version"] == "reason-mcp-identity-context-v3-holdout-selection-v1"
        assert receipt["candidate_semantics_sha"] == payload["candidate_semantics_sha"]
        assert receipt["metric_evaluator_sha"] == payload["metric_evaluator_sha"]
        assert receipt["holdout_fixture_sha256"] == sha256(args.fixture)
        assert receipt["forbidden_surface_digest_manifest_sha256"] == sha256(args.forbidden_digests)
        assert receipt["cases"] == len(cases)
        assert receipt["forbidden_surface_digest_count"] == len(forbidden)
        assert receipt["normalized_surface_overlap_count"] == 0
        assert receipt["network_observations_before_assignment"] == 0
        assert receipt["historical_holdouts_executed_or_replayed"] is False
        assert receipt["post_observation_retuning_allowed"] is False

    print(json.dumps({
        "schema_version": "reason-mcp-identity-context-v3-holdout-preflight-v1",
        "cases": len(cases),
        "class_counts": dict(classes),
        "forbidden_surface_digest_count": len(forbidden),
        "overlap_count": 0,
        "network_observations_before_assignment": 0,
        "historical_holdouts_executed_or_replayed": False,
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
