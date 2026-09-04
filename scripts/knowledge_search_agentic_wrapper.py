#!/usr/bin/env python3
"""Meter and reshape the experimental agentic search MCP response.

The underlying fused MCP probe stays unchanged so Layer B remains a stable fixed-policy
baseline. This wrapper is installed only for the agentic benchmark lane. It counts every
actual urllib HTTP attempt, including retries, and moves candidate churn under a novelty
snapshot so the existing Harness no-progress gate only observes target-directed fields.
"""

import io
import json
import os
import runpy
import sys
import urllib.request


ORIGINAL = os.environ.get("KNOWLEDGE_SEARCH_ORIGINAL")
if not ORIGINAL:
    raise SystemExit("KNOWLEDGE_SEARCH_ORIGINAL is required")

HTTP_REQUESTS = 0
_REAL_URLOPEN = urllib.request.urlopen


def counted_urlopen(*args, **kwargs):
    global HTTP_REQUESTS
    HTTP_REQUESTS += 1
    return _REAL_URLOPEN(*args, **kwargs)


def reshape_search_state(state):
    if not isinstance(state, dict):
        return state
    shaped = dict(state)
    novelty = {}
    for key in (
        "wikidata_candidate_ids",
        "wikipedia_candidates",
        "title_retry_candidate_ids",
        "wikipedia_top_entity",
        "wikipedia_top_title",
    ):
        if key in shaped:
            novelty[key] = shaped.pop(key)
    shaped["novelty_snapshot"] = novelty
    shaped["external_requests"] = HTTP_REQUESTS
    shaped["progress_contract"] = "target_directed_v1"
    return shaped


def transform(payload):
    result = payload.get("result")
    if isinstance(result, dict):
        structured = result.get("structuredContent")
        if isinstance(structured, dict):
            if "search_state" in structured:
                structured["search_state"] = reshape_search_state(structured["search_state"])
            harness = structured.get("reasoning_harness")
            if isinstance(harness, dict) and "search_state" in harness:
                harness["search_state"] = reshape_search_state(harness["search_state"])

    error = payload.get("error")
    if isinstance(error, dict):
        data = error.setdefault("data", {})
        if isinstance(data, dict):
            harness = data.setdefault("reasoning_harness", {})
            if isinstance(harness, dict):
                harness["external_requests"] = HTTP_REQUESTS
    return payload


urllib.request.urlopen = counted_urlopen
capture = io.StringIO()
real_stdout = sys.stdout
sys.stdout = capture
exit_code = 0
try:
    runpy.run_path(ORIGINAL, run_name="__main__")
except SystemExit as exc:
    if isinstance(exc.code, int):
        exit_code = exc.code
finally:
    sys.stdout = real_stdout
    urllib.request.urlopen = _REAL_URLOPEN

for line in capture.getvalue().splitlines():
    if not line.strip():
        continue
    try:
        payload = json.loads(line)
    except json.JSONDecodeError:
        print(line)
        continue
    print(json.dumps(transform(payload), ensure_ascii=False))

raise SystemExit(exit_code)
