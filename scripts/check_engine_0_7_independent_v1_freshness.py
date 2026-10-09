#!/usr/bin/env python3
"""Unrelated new-corpus freshness verifier, frozen before corpus authoring."""
import argparse
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
QUOTE_FIELDS = {"statement", "statements", "left", "right", "quote", "observation", "excerpt", "source_excerpt", "original_excerpt", "source_text"}
EXCLUDE = {"engine-0.7-independent-holdout-v1"}


def words(value):
    return tuple(re.findall(r"[a-z0-9]+", value.lower()))


def windows(value, size=8):
    tokens = words(value)
    return set(zip(*(tokens[i:] for i in range(size)))) if len(tokens) >= size else set()


def walk(obj, key=""):
    if isinstance(obj, dict):
        for name, value in obj.items():
            yield from walk(value, name)
    elif isinstance(obj, list):
        for value in obj:
            yield from walk(value, key)
    elif isinstance(obj, str) and key in QUOTE_FIELDS:
        yield obj


def previous_surfaces():
    ids, window_set = set(), set()
    for path in ROOT.glob("fixtures/**/manifest.json"):
        if any(fragment in str(path) for fragment in EXCLUDE):
            continue
        try:
            data = json.loads(path.read_text())
        except (ValueError, UnicodeError):
            continue
        def walk_ids(node, key=""):
            if isinstance(node, dict):
                for k, v in node.items():
                    yield from walk_ids(v, k)
            elif isinstance(node, list):
                for v in node:
                    yield from walk_ids(v, key)
            elif isinstance(node, str) and key in {"id", "target_id", "source_id"}:
                yield node
        ids.update(walk_ids(data))
        for quote in walk(data):
            window_set.update(windows(quote))
    # Development binary tests were also frozen before fresh corpus creation.
    text = (ROOT / "crates/reasoning-harness-cli/src/bin/engine_0_7_independent.rs").read_text()
    for quoted in re.findall(r'"([^"\n]{40,})"', text):
        window_set.update(windows(quoted))
    return ids, window_set


def check_freshness(corpus):
    old_ids, old_windows = previous_surfaces()
    seen_ids = set()
    seen_quotes = set()
    case_families = set()
    for case in corpus["cases"]:
        assert case["id"] not in old_ids and case["id"] not in seen_ids, "reused case identity"
        seen_ids.add(case["id"])
        case_families.add(case["family"])
        for target in case["targets"]:
            assert target["target_id"] not in old_ids and target["target_id"] not in seen_ids, "reused target identity"
            seen_ids.add(target["target_id"])
            for source in target["sources"]:
                sid = source["source_id"]
                assert sid not in old_ids and sid not in seen_ids, "reused source identity"
                seen_ids.add(sid)
                quote = source["quote"]
                assert len(words(quote)) >= 8, "source quote too short for eight-token independence"
                assert quote not in seen_quotes or target["relation"] == "identical", "duplicated fresh quote"
                seen_quotes.add(quote)
                overlap = windows(quote) & old_windows
                assert not overlap, f"reused eight-word source passage: {case['id']}"
    assert len(corpus["cases"]) == 12
    assert len(case_families) >= 4, "insufficient scenario family variety"
    return {"status":"FRESH_DISJOINT","case_count":len(corpus["cases"]), "families":sorted(case_families)}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", type=Path, required=True)
    args = parser.parse_args()
    result=check_freshness(json.loads(args.corpus.read_text()))
    print(json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
