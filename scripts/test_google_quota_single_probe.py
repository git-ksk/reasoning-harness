#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import unittest
from unittest import mock

SCRIPT = Path(__file__).with_name("google_quota_single_probe.py")
spec = importlib.util.spec_from_file_location("google_quota_single_probe", SCRIPT)
module = importlib.util.module_from_spec(spec)
assert spec.loader is not None
spec.loader.exec_module(module)


class GoogleQuotaSingleProbeTests(unittest.TestCase):
    def test_success(self):
        self.assertEqual(module.classify(200, {})["classification"], "success")

    def test_explicit_daily_wins_over_retry(self):
        payload = {"error": {"status": "RESOURCE_EXHAUSTED", "details": [
            {"@type": "type.googleapis.com/google.rpc.QuotaFailure", "violations": [
                {"quotaId": "GenerateRequestsPerDayPerProjectPerModel-FreeTier"}
            ]},
            {"@type": "type.googleapis.com/google.rpc.RetryInfo", "retryDelay": "39.4s"},
        ]}}
        self.assertEqual(module.classify(429, payload)["classification"], "daily_quota")

    def test_explicit_per_minute(self):
        payload = {"error": {"status": "RESOURCE_EXHAUSTED", "details": [
            {"@type": "type.googleapis.com/google.rpc.QuotaFailure", "violations": [
                {"quotaId": "GenerateRequestsPerMinutePerProjectPerModel-FreeTier", "quotaValue": "30"}
            ]}
        ]}}
        result = module.classify(429, payload)
        self.assertEqual(result["classification"], "rate_limit_per_minute")
        self.assertEqual(result["quota_violations"][0]["quotaValue"], "30")

    def test_bounded_retry_without_quota_id(self):
        payload = {"error": {"status": "RESOURCE_EXHAUSTED", "details": [
            {"@type": "type.googleapis.com/google.rpc.QuotaFailure", "violations": [
                {"quotaMetric": "generativelanguage.googleapis.com/generate_content_free_tier_requests"}
            ]},
            {"@type": "type.googleapis.com/google.rpc.RetryInfo", "retryDelay": "49.75s"},
        ]}}
        self.assertEqual(module.classify(429, payload)["classification"], "rate_limit_short_window")


    def test_run_probe_sends_one_request_and_keeps_key_out_of_url(self):
        class FakeResponse:
            status = 200
            def __enter__(self):
                return self
            def __exit__(self, exc_type, exc, tb):
                return False
            def read(self):
                return b'{"modelVersion":"gemini-test","candidates":[]}'

        class FakeOpener:
            def __init__(self):
                self.calls = []
            def open(self, request, timeout):
                self.calls.append((request, timeout))
                return FakeResponse()

        opener = FakeOpener()
        with mock.patch.object(module.urllib.request, "build_opener", return_value=opener):
            status, payload = module.run_probe("gemini-test", "super-secret", 7.0)

        self.assertEqual(status, 200)
        self.assertEqual(payload["modelVersion"], "gemini-test")
        self.assertEqual(len(opener.calls), 1)
        request, timeout = opener.calls[0]
        self.assertEqual(timeout, 7.0)
        self.assertNotIn("super-secret", request.full_url)
        self.assertEqual(request.get_header("X-goog-api-key"), "super-secret")

    def test_success_sanitizer_excludes_generated_text(self):
        payload = {
            "modelVersion": "gemini-test",
            "usageMetadata": {"totalTokenCount": 4},
            "candidates": [{
                "finishReason": "STOP",
                "content": {"parts": [{"text": "sensitive generated body"}]},
            }],
        }
        sanitized = module._sanitize_success(payload)
        self.assertEqual(sanitized["candidate_finish_reasons"], ["STOP"])
        self.assertNotIn("sensitive generated body", str(sanitized))

    def test_ambiguous_quota_without_short_retry(self):
        payload = {"error": {"status": "RESOURCE_EXHAUSTED", "details": [
            {"@type": "type.googleapis.com/google.rpc.QuotaFailure", "violations": [
                {"quotaMetric": "generativelanguage.googleapis.com/generate_content_free_tier_requests"}
            ]}
        ]}}
        self.assertEqual(module.classify(429, payload)["classification"], "quota_ambiguous")


if __name__ == "__main__":
    unittest.main()
