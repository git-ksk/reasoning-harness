import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "v31_diagnostics", ROOT / "scripts/validate_natural_language_e2e_v31_diagnostics.py"
)
M = importlib.util.module_from_spec(SPEC)
assert SPEC.loader is not None
SPEC.loader.exec_module(M)


class V31DiagnosticValidationTests(unittest.TestCase):
    def _write(self, value):
        temp = tempfile.NamedTemporaryFile("w", suffix=".json", delete=False)
        json.dump(value, temp)
        temp.close()
        self.addCleanup(lambda: Path(temp.name).unlink(missing_ok=True))
        return Path(temp.name)

    def test_success_trace_requires_mode_and_bounded_response_metadata(self):
        path = self._write({
            "contract": M.CONTRACT,
            "events": [
                {"kind": "model_request", "structured_mode": "json_schema"},
                {"kind": "model_response", "response": {"bytes": 42, "finish_reason": "stop"}},
            ],
        })
        row = M.validate_trace(path)
        self.assertEqual(row["model_requests"], 1)
        self.assertEqual(row["model_responses"], 1)

    def test_structured_protocol_failure_requires_parse_status_finish_and_bytes(self):
        path = self._write({
            "contract": M.CONTRACT,
            "events": [
                {"kind": "model_request", "structured_mode": "json_object"},
                {
                    "kind": "model_failure",
                    "failure_class": "protocol",
                    "message": "provider returned invalid structured planner JSON after fallback: first_parse_class=eof; first_finish_reason=incomplete; first_status_class=incomplete; first_bytes=474; second_parse_class=data; second_finish_reason=completed; second_status_class=complete; second_bytes=481",
                },
            ],
        })
        row = M.validate_trace(path)
        self.assertEqual(row["structured_protocol_failures"], 1)

    def test_incomplete_is_not_reclassified_as_token_limit(self):
        path = self._write({
            "contract": M.CONTRACT,
            "events": [
                {"kind": "model_request", "structured_mode": "json_schema"},
                {
                    "kind": "model_failure",
                    "failure_class": "protocol",
                    "message": "provider returned invalid structured planner JSON after fallback: parse_class=eof; finish_reason=incomplete; status_class=incomplete; bytes=474",
                },
            ],
        })
        self.assertEqual(M.validate_trace(path)["structured_protocol_failures"], 1)

    def test_missing_status_class_fails(self):
        path = self._write({
            "contract": M.CONTRACT,
            "events": [
                {"kind": "model_request", "structured_mode": "text"},
                {
                    "kind": "model_failure",
                    "failure_class": "protocol",
                    "message": "provider returned invalid strict text JSON after fallback: parse_class=eof; finish_reason=incomplete; bytes=10",
                },
            ],
        })
        with self.assertRaises(M.DiagnosticError):
            M.validate_trace(path)


if __name__ == "__main__":
    unittest.main()
