#!/usr/bin/env python3
"""Observation-free preflight for the Issue #195 fresh identity holdout."""
from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from pathlib import Path

EXPECTED_SCHEMA = "reason-mcp-identity-gate-holdout-v1"
EXPECTED_CLASSES = {
    "direct_accept": 2,
    "context_accept": 2,
    "ambiguity_unknown": 2,
    "wrong_claim_reject": 2,
}


def surfaces_from_rust(text: str, *, dev_only: bool) -> set[str]:
    if dev_only:
        start = text.index("fn dev_cases()")
        end = text.index("\n#[derive", start)
        text = text[start:end]
    return {
        match.group(1).strip().casefold()
        for match in re.finditer(r'initial_query:\s*"([^"]+)"', text)
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("fixture", type=Path)
    parser.add_argument("--benchmark-source", type=Path, required=True)
    parser.add_argument("--prior-source", type=Path)
    args = parser.parse_args()

    payload = json.loads(args.fixture.read_text(encoding="utf-8"))
    assert payload["schema_version"] == EXPECTED_SCHEMA
    assert re.fullmatch(r"[0-9a-f]{40}", payload["frozen_candidate_sha"])
    assert payload["model"] == "ministral-8b-latest"
    assert payload["trials"] == 2
    assert payload["post_observation_retuning_allowed"] is False

    cases = payload["cases"]
    assert isinstance(cases, list) and len(cases) == 8
    classes = Counter(case["class"] for case in cases)
    assert dict(classes) == EXPECTED_CLASSES

    ids = [case["id"] for case in cases]
    surfaces = [case["initial_query"].strip().casefold() for case in cases]
    fact_keys = [case["fact_key"] for case in cases]
    assert len(ids) == len(set(ids))
    assert len(surfaces) == len(set(surfaces))
    assert len(fact_keys) == len(set(fact_keys))

    for case in cases:
        assert case["property_id"] == "P17"
        assert case["value_kind"] == "entity"
        assert case["expected"] in {"accept", "reject", "unknown"}
        if case["class"] == "ambiguity_unknown":
            assert case["expected"] == "unknown"
            assert case["identity_context"] is None
        elif case["class"] == "wrong_claim_reject":
            assert case["expected"] == "reject"
            assert case["identity_context"] is None
        else:
            assert case["expected"] == "accept"
            assert isinstance(case["identity_context"], str) and case["identity_context"].strip()

    benchmark = args.benchmark_source.read_text(encoding="utf-8")
    dev_surfaces = surfaces_from_rust(benchmark, dev_only=True)
    selected = set(surfaces)
    dev_overlap = selected & dev_surfaces
    assert not dev_overlap, f"fresh holdout overlaps fresh dev surfaces: {sorted(dev_overlap)}"

    prior_overlap_count = None
    if args.prior_source:
        prior = args.prior_source.read_text(encoding="utf-8")
        prior_surfaces = surfaces_from_rust(prior, dev_only=False)
        prior_overlap = selected & prior_surfaces
        assert not prior_overlap, f"fresh holdout overlaps prior frozen holdout: {sorted(prior_overlap)}"
        prior_overlap_count = 0

    print(json.dumps({
        "schema_version": "reason-mcp-identity-holdout-preflight-v1",
        "cases": len(cases),
        "class_counts": dict(classes),
        "dev_overlap_count": 0,
        "prior_holdout_overlap_count": prior_overlap_count,
        "network_observations": 0,
    }, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
