import unittest

from scripts.operational_case_retry import (
    CaseAttemptResult,
    provider_failure,
    run_case_with_operational_retry,
)


def investigation_failure(failure_class, message, provider="google"):
    return {
        "result": {
            "investigation": {
                "generation_failure": {
                    "provider": provider,
                    "model": "test-model",
                    "failure_class": failure_class,
                    "message": message,
                }
            }
        }
    }


def product_failure(failure_class, message):
    return {"result": {"status": "failed", "failure": {"failure_class": failure_class, "message": message}}}


class OperationalCaseRetryTests(unittest.TestCase):
    def test_retryable_provider_failure_then_success_reuses_exact_command(self):
        seen = []
        results = iter([
            CaseAttemptResult(0, investigation_failure("provider_unavailable", "Google HTTP 503")),
            CaseAttemptResult(0, {"result": {"report_gate_passed": False, "answer": "semantic result"}}),
        ])

        def execute(command):
            seen.append(command)
            return next(results)

        outcome = run_case_with_operational_retry(["reason", "task", "--seed", "83001"], execute)
        self.assertEqual(len(seen), 2)
        self.assertEqual(seen[0], seen[1])
        self.assertEqual(outcome.canonical_attempt, 2)
        self.assertFalse(outcome.retry_exhausted)
        self.assertTrue(outcome.records[0].retryable_operational_failure)
        self.assertFalse(outcome.records[1].retryable_operational_failure)
        self.assertEqual(outcome.records[0].command_fingerprint, outcome.records[1].command_fingerprint)

    def test_repeated_transient_failure_stops_at_bound(self):
        calls = 0

        def execute(_command):
            nonlocal calls
            calls += 1
            return CaseAttemptResult(0, investigation_failure("timeout", "provider timed out"))

        outcome = run_case_with_operational_retry(["reason", "task"], execute, max_attempts=2)
        self.assertEqual(calls, 2)
        self.assertEqual(outcome.canonical_attempt, 2)
        self.assertTrue(outcome.retry_exhausted)
        self.assertTrue(outcome.records[-1].canonical)

    def test_google_empty_text_protocol_is_narrowly_retryable_for_released_control(self):
        results = iter([
            CaseAttemptResult(
                0,
                investigation_failure(
                    "protocol",
                    "Gemini Interactions response contained no model text output",
                ),
            ),
            CaseAttemptResult(0, {"result": {"ok": True}}),
        ])
        outcome = run_case_with_operational_retry(["reason", "task"], lambda command: next(results))
        self.assertEqual(outcome.canonical_attempt, 2)
        self.assertEqual(outcome.records[0].failure_subtype, "google_empty_model_text")

    def test_invalid_json_protocol_is_not_retried(self):
        calls = 0

        def execute(_command):
            nonlocal calls
            calls += 1
            return CaseAttemptResult(
                0,
                investigation_failure(
                    "protocol",
                    "provider returned invalid structured planner JSON after fallback",
                ),
            )

        outcome = run_case_with_operational_retry(["reason", "task"], execute)
        self.assertEqual(calls, 1)
        self.assertEqual(outcome.canonical_attempt, 1)
        self.assertFalse(outcome.records[0].retryable_operational_failure)

    def test_semantic_or_scoring_failure_is_never_retried(self):
        calls = 0

        def execute(_command):
            nonlocal calls
            calls += 1
            return CaseAttemptResult(
                0,
                {
                    "result": {
                        "report_gate_passed": False,
                        "correctness_boundary_violations": 1,
                        "false_abstention": 1,
                    }
                },
            )

        outcome = run_case_with_operational_retry(["reason", "task"], execute)
        self.assertEqual(calls, 1)
        self.assertEqual(outcome.canonical_attempt, 1)

    def test_top_level_released_provider_transport_failure_is_retryable(self):
        failure = provider_failure(
            product_failure(
                "transport",
                "provider=google model=test failure_class=transport latency_ms=12: connection reset",
            )
        )
        self.assertIsNotNone(failure)
        results = iter([
            CaseAttemptResult(
                1,
                product_failure(
                    "transport",
                    "provider=google model=test failure_class=transport latency_ms=12: connection reset",
                ),
            ),
            CaseAttemptResult(0, {"result": {"ok": True}}),
        ])
        outcome = run_case_with_operational_retry(["reason", "task"], lambda command: next(results))
        self.assertEqual(outcome.canonical_attempt, 2)
        self.assertEqual(outcome.records[0].failure_source, "product_failure")

    def test_quota_credentials_provider_400_and_action_failure_are_not_retried(self):
        payloads = [
            investigation_failure("quota", "quota exhausted"),
            investigation_failure("credentials", "bad credential"),
            investigation_failure("provider_error", "HTTP 400"),
            {
                "result": {
                    "investigation": {
                        "telemetry": {
                            "actions": [
                                {"status": "operational_failure", "failure_class": "timeout"}
                            ]
                        }
                    }
                }
            },
        ]
        for payload in payloads:
            with self.subTest(payload=payload):
                calls = 0

                def execute(_command):
                    nonlocal calls
                    calls += 1
                    return CaseAttemptResult(0, payload)

                outcome = run_case_with_operational_retry(["reason", "task"], execute)
                self.assertEqual(calls, 1)
                self.assertEqual(outcome.canonical_attempt, 1)

    def test_non_provider_top_level_protocol_is_not_misclassified(self):
        self.assertIsNone(provider_failure(product_failure("protocol", "invalid input JSON")))

    def test_invalid_attempt_bound_fails_before_execution(self):
        with self.assertRaises(ValueError):
            run_case_with_operational_retry(["reason"], lambda command: CaseAttemptResult(0, {}), max_attempts=0)


if __name__ == "__main__":
    unittest.main()
