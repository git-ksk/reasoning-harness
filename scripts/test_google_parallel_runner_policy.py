#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts" / "google_parallel_runner_policy.py"
spec = importlib.util.spec_from_file_location("google_runner_policy", MODULE_PATH)
mod = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(mod)

MANIFEST = {
    "provider_policy": {"inter_case_delay_ms": 3000},
    "parallel_execution_policy": {
        "google_request_start_interval_ms": 6000,
        "google_shared_pacer_required_for_parallel": True,
        "google_model_jobs_serial": True,
        "google_model_job_max_parallel": 1,
        "gemma_investigation_workers": 2,
        "adaptive_followup_cases_serial": True,
        "mcp_nonpromotion_cases_serial": True,
        "session_cases_serial": True,
    },
}
VALID_ENV = {"REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS": "6000", "REASON_GOOGLE_SHARED_PACER_PATH": "/tmp/reason-google-pacer.state"}

class GoogleParallelRunnerPolicyTests(unittest.TestCase):
    def validate(self, **kwargs):
        params = {"provider": "google", "model": "gemma-4-31b-it", "workers": 2, "inter_case_delay_ms": 3000, "env": VALID_ENV}
        params.update(kwargs)
        return mod.validate_parallel_runner_policy(MANIFEST, **params)

    def test_6000_pacing_and_3000_delay_are_independent_and_valid(self):
        self.validate()

    def test_pacing_drift_is_rejected(self):
        env = dict(VALID_ENV, REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS="3000")
        with self.assertRaisesRegex(mod.RunnerPolicyError, "pacing mismatch"):
            self.validate(env=env)

    def test_delay_drift_is_rejected(self):
        with self.assertRaisesRegex(mod.RunnerPolicyError, "inter-case delay mismatch"):
            self.validate(inter_case_delay_ms=6000)

    def test_missing_or_relative_shared_pacer_is_rejected(self):
        for shared in ("", "relative/pacer.state"):
            with self.subTest(shared=shared):
                env = dict(VALID_ENV, REASON_GOOGLE_SHARED_PACER_PATH=shared)
                with self.assertRaisesRegex(mod.RunnerPolicyError, "pacer path must be absolute"):
                    self.validate(env=env)

    def test_workers_two_on_gemini_or_other_provider_is_rejected(self):
        for provider, model in (("google", "gemini-3.5-flash-lite"), ("groq", "openai/gpt-oss-120b")):
            with self.subTest(provider=provider, model=model):
                with self.assertRaisesRegex(mod.RunnerPolicyError, "only for google/gemma"):
                    self.validate(provider=provider, model=model)

    def test_serial_worker_does_not_require_parallel_pacer_contract(self):
        self.validate(workers=1, env={})

    def test_cli_pre_live_dry_run_uses_runner_policy_without_network(self):
        with tempfile.TemporaryDirectory() as tmp:
            manifest = Path(tmp) / "manifest.json"
            manifest.write_text(json.dumps(MANIFEST))
            env = os.environ.copy()
            env.update(VALID_ENV)
            cp = subprocess.run([
                sys.executable, str(MODULE_PATH), "--manifest", str(manifest),
                "--provider", "google", "--model", "gemma-4-31b-it",
                "--workers", "2", "--inter-case-delay-ms", "3000",
            ], env=env, text=True, capture_output=True, check=False)
            self.assertEqual(cp.returncode, 0, cp.stderr)
            self.assertIn("Google parallel runner policy: OK", cp.stdout)

if __name__ == "__main__":
    unittest.main()
