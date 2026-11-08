"""Offline v3 timeout evidence audit; no provider connection or scoring."""
import copy
import unittest

from ci_duration_audit import analyze, CONTRACT


class AuditTest(unittest.TestCase):
    def setUp(self):
        self.plan = {"jobs": [{"id": "rw-01-baseline", "timeout_seconds": 360,
                               "budget": {"max_generations": 24},
                               "argv": ["reason", "--max-model-calls", "12", "--format", "json", "Q"],
                               "http_limits": {"min_generation_start_interval_ms": 10500}}]}
        self.execution = {"operational_failure": "native_process_timeout",
                          "streams_complete": False}
        self.events = []
        for n in range(6):
            request_id = f"http-{n}"
            self.events.append({"event": {"kind": "request", "request_id": request_id,
                         "start_elapsed_ns": int((0.01 + n*61)*1e9)}})
            self.events.append({"event": {"kind": "response", "request_id": request_id,
                         "http_status": 200}})
            self.events.append({"event": {"kind": "aborted" if n == 5 else "completed",
                         "request_id": request_id}})

    def test_realistic_six_responses_timeout_is_operational_not_quality(self):
        result = analyze(self.plan, "rw-01-baseline", self.execution, self.events)
        self.assertEqual(result["contract"], CONTRACT)
        self.assertTrue(result["not_model_quality_failure"])
        self.assertEqual(result["http200_responses"], 6)
        self.assertEqual(result["actual_cli_call_ceiling_not_expected_count"], 12)
        self.assertAlmostEqual(
            result["hypothetical_last_start_seconds_if_cli_call_ceiling_reached_at_observed_median"],
            671.01, delta=2.1)
        self.assertEqual(result["completed_attempts"], 5)
        self.assertEqual(result["aborted_attempts"], 1)
        self.assertAlmostEqual(result["observed_median_interstart_seconds"], 61, places=2)
        self.assertTrue(result["hypothetical_last_start_exceeds_job_timeout"])
        self.assertFalse(result["actual_case_answer_complete"])

    def test_mismatched_job_or_non_timeout_not_relabelled(self):
        with self.assertRaisesRegex(ValueError, "ambiguous_job"):
            analyze(self.plan, "other", self.execution, self.events)
        with self.assertRaisesRegex(ValueError, "not_timeout"):
            analyze(self.plan, "rw-01-baseline",
                    {**self.execution, "operational_failure": None}, self.events)

    def test_bogus_timing_and_unknown_request_rejected(self):
        broken = copy.deepcopy(self.events)
        broken[3]["event"]["start_elapsed_ns"] = -1
        with self.assertRaisesRegex(ValueError, "invalid_start_order"):
            analyze(self.plan, "rw-01-baseline", self.execution, broken)
        broken = copy.deepcopy(self.events)
        broken[1]["event"]["request_id"] = "never-sent"
        with self.assertRaisesRegex(ValueError, "received_without_request"):
            analyze(self.plan, "rw-01-baseline", self.execution, broken)

    def test_generation_max_is_ceiling_not_required_request_count(self):
        plan = copy.deepcopy(self.plan)
        plan["jobs"][0]["budget"]["max_generations"] = 6
        plan["jobs"][0]["argv"][2] = "6"
        result = analyze(plan, "rw-01-baseline", self.execution, self.events)
        self.assertEqual(result["allowed_generation_ceiling_not_expected_count"], 6)
        self.assertFalse(result["hypothetical_last_start_exceeds_job_timeout"])


if __name__ == "__main__":
    unittest.main()
