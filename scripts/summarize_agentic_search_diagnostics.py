#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


def load(path):
    return json.loads(Path(path).read_text())


def novelty_keys(trace):
    state = trace.get("search_state") or {}
    snap = state.get("novelty_snapshot") or {}
    out = set()
    for value in snap.get("wikidata_candidate_ids") or []:
        if value:
            out.add(f"wikidata:{value}")
    for value in snap.get("title_retry_candidate_ids") or []:
        if value:
            out.add(f"title_retry:{value}")
    for item in snap.get("wikipedia_candidates") or []:
        if not isinstance(item, dict):
            continue
        title = item.get("title")
        entity = item.get("wikibase_item")
        if title:
            out.add(f"wikipedia_title:{title}")
        if entity:
            out.add(f"wikipedia_entity:{entity}")
    if snap.get("wikipedia_top_entity"):
        out.add(f"wikipedia_top_entity:{snap['wikipedia_top_entity']}")
    if snap.get("wikipedia_top_title"):
        out.add(f"wikipedia_top_title:{snap['wikipedia_top_title']}")
    return out


def summarize_sample(sample):
    seen_novelty = set()
    external_requests = 0
    target_progress_items = 0
    target_progress_rounds = 0
    novelty_only_rounds = 0
    zero_external_request_tool_calls = 0
    progress_contract_rounds = 0
    round_rows = []

    for trace in sample.get("traces", []):
        state = trace.get("search_state") or {}
        requests = state.get("external_requests")
        if isinstance(requests, int):
            external_requests += requests
            if requests == 0:
                zero_external_request_tool_calls += 1
        if state.get("progress_contract") == "target_directed_v1":
            progress_contract_rounds += 1

        target = trace.get("new_progress_items") or 0
        target_progress_items += target
        if target > 0:
            target_progress_rounds += 1

        keys = novelty_keys(trace)
        new_novelty = keys - seen_novelty
        seen_novelty.update(new_novelty)
        if new_novelty and target == 0:
            novelty_only_rounds += 1

        round_rows.append({
            "round": trace.get("round"),
            "query": trace.get("query"),
            "outcome_kind": state.get("outcome_kind"),
            "external_requests": requests,
            "target_progress_items": target,
            "new_novelty_items": len(new_novelty),
        })

    return {
        "id": sample.get("id"),
        "trial": sample.get("trial"),
        "passed": sample.get("passed"),
        "stop_reason": sample.get("stop_reason"),
        "tool_calls": sample.get("tool_calls", 0),
        "planner_calls": sample.get("planner_calls", 0),
        "external_requests": external_requests,
        "target_progress_items": target_progress_items,
        "target_progress_rounds": target_progress_rounds,
        "unique_novelty_items": len(seen_novelty),
        "novelty_only_rounds": novelty_only_rounds,
        "zero_external_request_tool_calls": zero_external_request_tool_calls,
        "progress_contract_rounds": progress_contract_rounds,
        "rounds": round_rows,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()

    report = load(args.input)
    samples = [summarize_sample(sample) for sample in report.get("samples", [])]
    total_tool_calls = sum(sample["tool_calls"] for sample in samples)
    total_external_requests = sum(sample["external_requests"] for sample in samples)
    total_target_progress = sum(sample["target_progress_items"] for sample in samples)
    total_target_progress_rounds = sum(sample["target_progress_rounds"] for sample in samples)
    total_novelty_only_rounds = sum(sample["novelty_only_rounds"] for sample in samples)
    total_zero_request_calls = sum(sample["zero_external_request_tool_calls"] for sample in samples)
    total_contract_rounds = sum(sample["progress_contract_rounds"] for sample in samples)

    result = {
        "schema_version": "reason-mcp-agentic-loop-diagnostics-v1",
        "progress_contract": "target_directed_v1",
        "samples": len(samples),
        "aggregate": {
            "tool_calls": total_tool_calls,
            "external_requests": total_external_requests,
            "external_requests_per_tool_call": (
                total_external_requests / total_tool_calls if total_tool_calls else 0.0
            ),
            "zero_external_request_tool_calls": total_zero_request_calls,
            "target_progress_items": total_target_progress,
            "target_progress_rounds": total_target_progress_rounds,
            "novelty_only_rounds": total_novelty_only_rounds,
            "progress_contract_rounds": total_contract_rounds,
        },
        "sample_diagnostics": samples,
        "notes": [
            "external_requests counts actual urllib HTTP attempts, including transport retries, not MCP invocations.",
            "candidate discovery is retained as novelty telemetry but is intentionally excluded from the Harness no-progress reset.",
            "new_progress_items from the benchmark now measures only top-level target-directed state under this wrapper contract.",
        ],
    }
    text = json.dumps(result, ensure_ascii=False, indent=2)
    if args.output:
        Path(args.output).write_text(text + "\n")
    print(text)


if __name__ == "__main__":
    main()
