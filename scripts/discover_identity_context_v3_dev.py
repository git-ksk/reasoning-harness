#!/usr/bin/env python3
"""Live discovery for the already-assigned Issue #196 fresh-dev split.

The fixture is assigned and overlap-checked before this script is ever run.
This script never reads or executes historical frozen holdouts.
"""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

FIXTURE = Path("fixtures/mcp-identity-context-v3-dev.json")


def probe(case: dict, query: str, coordinate: str) -> dict:
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search_fact",
            "arguments": {
                "query": query,
                "language": "en",
                "property_id": case["property_id"],
                "value_kind": case["value_kind"],
                "fact_key": case["fact_key"],
                "allow_title_retry": coordinate == "trusted_context",
            },
            "_meta": {"protocolVersion": "2026-07-28"},
        },
    }
    completed = subprocess.run(
        ["python3", "scripts/knowledge_search_fused_mcp.py"],
        input=json.dumps(request) + "\n",
        text=True,
        capture_output=True,
        check=False,
    )
    base = {
        "id": case["id"],
        "class": case["class"],
        "coordinate": coordinate,
        "query": query,
    }
    if completed.returncode != 0:
        return base | {
            "operational_failure": True,
            "failure_kind": "adapter_process",
            "stderr": completed.stderr[-500:],
        }
    try:
        response = json.loads(completed.stdout)
    except json.JSONDecodeError:
        return base | {
            "operational_failure": True,
            "failure_kind": "adapter_protocol",
        }
    if "error" in response:
        error = response.get("error") or {}
        data = error.get("data") or {}
        rh = data.get("reasoning_harness") or {}
        return base | {
            "operational_failure": True,
            "failure_kind": rh.get("operational_kind") or error.get("message", "adapter_error"),
            "external_requests": rh.get("external_requests", 0),
        }

    structured = (response.get("result") or {}).get("structuredContent") or {}
    harness = structured.get("reasoning_harness") or {}
    state = harness.get("search_state") or structured.get("search_state") or {}
    facts = harness.get("facts") or {}
    top = (state.get("wikipedia_candidates") or [None])[0]
    return base | {
        "operational_failure": False,
        "outcome_kind": state.get("outcome_kind"),
        "resolved_entity": state.get("resolved_entity"),
        "corroboration_rank": state.get("corroboration_rank"),
        "wikipedia_top": top,
        "wikidata_candidate_ids": state.get("wikidata_candidate_ids", []),
        "fact": facts.get(case["fact_key"]),
        "external_requests": state.get("external_requests", 0),
    }


def main() -> int:
    payload = json.loads(FIXTURE.read_text(encoding="utf-8"))
    rows = []
    for case in payload["cases"]:
        rows.append(probe(case, case["initial_query"], "initial"))
        context = case.get("identity_context")
        if context:
            rows.append(probe(case, f"{case['initial_query']}, {context}", "trusted_context"))

    report = {
        "schema_version": "reason-mcp-identity-context-v3-dev-discovery-v1",
        "purpose": "post_assignment_fresh_development_discovery_only",
        "historical_holdouts_executed_or_replayed": False,
        "probes": rows,
        "summary": {
            "cases": len(payload["cases"]),
            "probes": len(rows),
            "operational_failures": sum(bool(row.get("operational_failure")) for row in rows),
            "fact_resolved": sum(row.get("outcome_kind") == "fact_resolved" for row in rows),
            "ambiguous": sum(row.get("outcome_kind") == "ambiguous" for row in rows),
            "unresolved": sum(
                row.get("outcome_kind") in {
                    "entity_unresolved",
                    "entity_disagreement",
                    "search_unresolved",
                    "property_unresolved",
                }
                for row in rows
            ),
            "external_requests": sum(int(row.get("external_requests") or 0) for row in rows),
        },
    }
    print(json.dumps(report, indent=2, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
