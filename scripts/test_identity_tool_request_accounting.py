#!/usr/bin/env python3
import importlib.util
import json
import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

MODULE_PATH = Path(__file__).with_name("knowledge_search_fused_mcp.py")
spec = importlib.util.spec_from_file_location("knowledge_search_fused_mcp", MODULE_PATH)
probe = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(probe)


class IdentityToolRequestAccountingTests(unittest.TestCase):
    def setUp(self):
        probe.EXTERNAL_REQUESTS = 0

    def test_transport_attempts_count_actual_urlopen_attempts(self):
        with patch.object(probe.urllib.request, "urlopen", side_effect=OSError("injected")):
            with self.assertRaises(OSError):
                probe.get_json("https://example.invalid/api", {"q": "x"}, timeout=0.01)
        self.assertEqual(probe.EXTERNAL_REQUESTS, 2)
        state = probe.state("x", "search_unresolved")
        self.assertEqual(state["external_requests"], 2)

    def test_invalid_search_is_rejected_before_any_external_request(self):
        request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": "search_fact",
                "arguments": {
                    "query": "Example P17",
                    "language": "en",
                    "property_id": "P17",
                    "value_kind": "entity",
                    "fact_key": "example.country",
                    "allow_title_retry": False,
                },
                "_meta": {"protocolVersion": "2026-07-28"},
            },
        }
        completed = subprocess.run(
            [sys.executable, str(MODULE_PATH)],
            input=json.dumps(request) + "\n",
            text=True,
            capture_output=True,
            check=True,
        )
        response = json.loads(completed.stdout)
        state = response["result"]["structuredContent"]["search_state"]
        self.assertEqual(state["outcome_kind"], "invalid_query")
        self.assertEqual(state["external_requests"], 0)
        self.assertEqual(state["validation_reason"], "property_id_must_not_be_in_query")


if __name__ == "__main__":
    unittest.main()
