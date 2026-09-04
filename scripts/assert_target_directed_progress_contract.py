#!/usr/bin/env python3
"""Deterministic regression for the agentic no-progress contract.

This is intentionally model/network free. It locks the orchestration boundary rule that
fresh candidate novelty alone must not reset the bounded loop's no-progress counter.
"""

TARGET_ARRAY_KEYS = ("property_values",)
TARGET_SCALAR_KEYS = ("resolved_entity", "suggested_query")
NO_PROGRESS_LIMIT = 2


def target_progress_items(state):
    out = set()
    for key in TARGET_ARRAY_KEYS:
        for value in state.get(key) or []:
            if isinstance(value, str):
                out.add(f"{key}:{value}")
    for key in TARGET_SCALAR_KEYS:
        value = state.get(key)
        if isinstance(value, str) and value:
            out.add(f"{key}:{value}")
    return out


def apply_round(state, seen, no_progress_rounds):
    new_items = target_progress_items(state) - seen
    if new_items:
        seen.update(new_items)
        return 0, False, new_items
    no_progress_rounds += 1
    return no_progress_rounds, no_progress_rounds >= NO_PROGRESS_LIMIT, new_items


def novelty_state(label):
    return {
        "progress_contract": "target_directed_v1",
        "novelty_snapshot": {
            "wikidata_candidate_ids": [f"Q-{label}"],
            "wikipedia_candidates": [
                {"title": f"candidate-{label}", "wikibase_item": f"Q-{label}", "disambiguation": False}
            ],
        },
        "outcome_kind": "search_unresolved",
    }


def main():
    seen = set()
    no_progress = 0

    # Two different result sets are still zero target progress. Candidate churn must
    # terminate at the deterministic limit rather than keeping the search alive.
    no_progress, stopped, new_items = apply_round(novelty_state("A"), seen, no_progress)
    assert not stopped
    assert no_progress == 1
    assert not new_items

    no_progress, stopped, new_items = apply_round(novelty_state("B"), seen, no_progress)
    assert stopped
    assert no_progress == NO_PROGRESS_LIMIT
    assert not new_items

    # A legitimate next-query constraint is target-directed progress and resets the
    # counter even if candidate novelty is also present.
    seen = set()
    no_progress = 1
    reformulation = novelty_state("C")
    reformulation["suggested_query"] = "Paris"
    no_progress, stopped, new_items = apply_round(reformulation, seen, no_progress)
    assert not stopped
    assert no_progress == 0
    assert new_items == {"suggested_query:Paris"}

    # Local validation feedback by itself is not progress. The planner gets one chance
    # to repair it, but repeating non-progress feedback is bounded.
    invalid = {
        "progress_contract": "target_directed_v1",
        "outcome_kind": "invalid_query",
        "validation_reason": "property_id_must_not_be_in_query",
        "external_requests": 0,
        "novelty_snapshot": {},
    }
    seen = set()
    no_progress = 0
    no_progress, stopped, _ = apply_round(invalid, seen, no_progress)
    assert not stopped and no_progress == 1
    no_progress, stopped, _ = apply_round(invalid, seen, no_progress)
    assert stopped and no_progress == 2

    # A resolved entity/property is target progress and therefore survives the filter.
    resolved = {
        "progress_contract": "target_directed_v1",
        "outcome_kind": "fact_resolved",
        "resolved_entity": "Q90",
        "property_values": ["Q142"],
        "novelty_snapshot": {"wikidata_candidate_ids": ["Q90", "Q1"]},
    }
    items = target_progress_items(resolved)
    assert items == {"resolved_entity:Q90", "property_values:Q142"}

    print("target-directed progress contract: PASS")
    print("two distinct novelty-only rounds => no_progress stop at limit=2")


if __name__ == "__main__":
    main()
