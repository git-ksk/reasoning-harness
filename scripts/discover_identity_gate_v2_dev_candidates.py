#!/usr/bin/env python3
"""Discover fresh development cases for the identity-gate candidate.

This is development-only instrumentation. It intentionally does not contain or
probe any entity from the previously observed #193 frozen holdout.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

from identity_gate_v2 import qualify_identity


# Fresh development-only surfaces. These are deliberately distinct from the
# previous experiment's frozen holdout and are not a future holdout set.
QUERIES = [
    "Vienna",
    "Madrid",
    "Oxford",
    "Granada",
    "Victoria",
    "Georgia",
    "Springfield",
    "Lincoln",
    "Kingston",
    "Alexandria",
]


def probe(query: str) -> dict:
    fact_key = "candidate.country"
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "search_fact",
            "arguments": {
                "query": query,
                "language": "en",
                "property_id": "P17",
                "value_kind": "entity",
                "fact_key": fact_key,
                "allow_title_retry": False,
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
    if completed.returncode != 0:
        return {
            "query": query,
            "operational_failure": True,
            "failure_kind": "adapter_process",
            "stderr": completed.stderr[-500:],
        }
    try:
        response = json.loads(completed.stdout)
    except json.JSONDecodeError:
        return {
            "query": query,
            "operational_failure": True,
            "failure_kind": "adapter_protocol",
        }
    if "error" in response:
        return {
            "query": query,
            "operational_failure": True,
            "failure_kind": response["error"].get("message", "adapter_error"),
        }

    result = response.get("result") or {}
    structured = result.get("structuredContent") or {}
    harness = structured.get("reasoning_harness") or {}
    raw_state = harness.get("search_state") or structured.get("search_state") or {}
    raw_facts = harness.get("facts") or {}
    qualified_facts, qualified_state = qualify_identity(raw_facts, raw_state)

    return {
        "query": query,
        "operational_failure": False,
        "raw_outcome_kind": raw_state.get("outcome_kind"),
        "qualified_outcome_kind": qualified_state.get("outcome_kind"),
        "identity_supported": qualified_state.get("identity_supported"),
        "identity_reasons": qualified_state.get("identity_reasons", []),
        "resolved_entity": raw_state.get("resolved_entity"),
        "corroboration_rank": raw_state.get("corroboration_rank"),
        "wikidata_candidate_ids": raw_state.get("wikidata_candidate_ids", []),
        "wikipedia_top": (raw_state.get("wikipedia_candidates") or [None])[0],
        "raw_fact": raw_facts.get(fact_key),
        "qualified_fact": qualified_facts.get(fact_key),
    }


def main() -> int:
    rows = [probe(query) for query in QUERIES]
    report = {
        "schema_version": "reason-mcp-identity-gate-dev-discovery-v1",
        "purpose": "fresh_development_case_discovery_only",
        "prior_frozen_holdout_reused": False,
        "queries": rows,
        "summary": {
            "queries": len(rows),
            "operational_failures": sum(bool(row.get("operational_failure")) for row in rows),
            "identity_supported": sum(row.get("identity_supported") is True for row in rows),
            "identity_insufficient": sum(
                row.get("qualified_outcome_kind") == "identity_insufficient" for row in rows
            ),
            "ambiguous_or_unresolved": sum(
                row.get("qualified_outcome_kind") in {
                    "ambiguous",
                    "entity_unresolved",
                    "entity_disagreement",
                    "search_unresolved",
                    "property_unresolved",
                }
                for row in rows
            ),
        },
    }
    print(json.dumps(report, indent=2, ensure_ascii=False, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
