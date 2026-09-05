#!/usr/bin/env python3
"""Deterministic regression for MCP transport retry/request accounting.

The real agentic wrapper and fused MCP probe are executed with a fixture-backed
``urlopen``. This verifies that wrapper telemetry counts physical HTTP attempts,
including retries, without depending on live Wikipedia/Wikidata availability.
"""

import io
import json
import os
from pathlib import Path
import runpy
import sys
import urllib.error
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parent
ORIGINAL = ROOT / "knowledge_search_fused_mcp.py"
WRAPPER = ROOT / "knowledge_search_agentic_wrapper.py"

REQUEST = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
        "name": "search_fact",
        "arguments": {
            "query": "Tokyo",
            "language": "en",
            "property_id": "P17",
            "value_kind": "entity",
            "fact_key": "tokyo.country",
            "allow_title_retry": False,
        },
        "_meta": {"protocolVersion": "2026-07-28"},
    },
}


class FixtureResponse(io.BytesIO):
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        self.close()
        return False


def fixture_body(url):
    parsed = urllib.parse.urlparse(url.full_url if hasattr(url, "full_url") else str(url))
    query = urllib.parse.parse_qs(parsed.query)
    action = (query.get("action") or [None])[0]

    if action == "wbsearchentities":
        return {
            "search": [
                {"id": "Q1490", "label": "Tokyo", "description": "capital of Japan"}
            ]
        }
    if action == "query":
        return {
            "query": {
                "pages": [
                    {
                        "index": 1,
                        "title": "Tokyo",
                        "pageprops": {"wikibase_item": "Q1490"},
                    }
                ]
            }
        }
    if action == "wbgetentities":
        return {
            "entities": {
                "Q1490": {
                    "claims": {
                        "P17": [
                            {
                                "rank": "normal",
                                "mainsnak": {
                                    "datavalue": {"value": {"numeric-id": 17}}
                                },
                            }
                        ]
                    }
                }
            }
        }
    raise AssertionError(f"unexpected fixture URL: {parsed.geturl()}")


def run_wrapper(*, fail_first=False, fail_always=False):
    calls = []
    real_urlopen = urllib.request.urlopen
    real_stdin = sys.stdin
    real_stdout = sys.stdout
    real_original = os.environ.get("KNOWLEDGE_SEARCH_ORIGINAL")

    def fake_urlopen(request, *args, **kwargs):
        calls.append(request.full_url if hasattr(request, "full_url") else str(request))
        if fail_always or (fail_first and len(calls) == 1):
            raise urllib.error.URLError("injected transport failure")
        payload = json.dumps(fixture_body(request)).encode("utf-8")
        return FixtureResponse(payload)

    capture = io.StringIO()
    urllib.request.urlopen = fake_urlopen
    sys.stdin = io.StringIO(json.dumps(REQUEST) + "\n")
    sys.stdout = capture
    os.environ["KNOWLEDGE_SEARCH_ORIGINAL"] = str(ORIGINAL)
    try:
        try:
            runpy.run_path(str(WRAPPER), run_name="__main__")
        except SystemExit as exc:
            if exc.code not in (None, 0):
                raise AssertionError(f"wrapper exited with {exc.code}") from exc
    finally:
        urllib.request.urlopen = real_urlopen
        sys.stdin = real_stdin
        sys.stdout = real_stdout
        if real_original is None:
            os.environ.pop("KNOWLEDGE_SEARCH_ORIGINAL", None)
        else:
            os.environ["KNOWLEDGE_SEARCH_ORIGINAL"] = real_original

    lines = [line for line in capture.getvalue().splitlines() if line.strip()]
    assert len(lines) == 1, lines
    return json.loads(lines[0]), calls


def main():
    success, success_calls = run_wrapper()
    success_state = success["result"]["structuredContent"]["search_state"]
    assert success_state["outcome_kind"] == "fact_resolved"
    assert success_state["external_requests"] == 3
    assert len(success_calls) == 3
    assert success["result"]["structuredContent"]["reasoning_harness"]["facts"]["tokyo.country"] == "Q17"

    recovered, retry_calls = run_wrapper(fail_first=True)
    recovered_state = recovered["result"]["structuredContent"]["search_state"]
    assert recovered_state["outcome_kind"] == "fact_resolved"
    assert recovered_state["external_requests"] == 4
    assert len(retry_calls) == 4
    assert retry_calls[0] == retry_calls[1], "first failed request must be retried unchanged"

    failed, failed_calls = run_wrapper(fail_always=True)
    assert "result" not in failed
    assert failed["error"]["data"]["reasoning_harness"]["operational_kind"] == "transport"
    assert failed["error"]["data"]["reasoning_harness"]["external_requests"] == 2
    assert len(failed_calls) == 2
    assert failed_calls[0] == failed_calls[1], "terminal failure must exhaust exactly one retry"

    print("transport retry/request accounting contract: PASS")
    print("success: 1 MCP call => 3 HTTP attempts")
    print("single injected transport failure: 1 MCP call => 4 HTTP attempts, then recovery")
    print("persistent transport failure: 1 MCP call => 2 HTTP attempts, typed transport error")


if __name__ == "__main__":
    main()
