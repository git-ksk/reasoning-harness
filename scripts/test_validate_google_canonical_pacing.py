#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODULE_PATH = ROOT / "scripts" / "validate_google_canonical_pacing.py"
spec = importlib.util.spec_from_file_location("google_pacing", MODULE_PATH)
mod = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(mod)


VALID_WORKFLOW = r'''jobs:
  google-paired:
    needs: preflight
    runs-on: ubuntu-24.04
    env:
      INTER_CASE_DELAY_MS: '3000'
    strategy:
      fail-fast: false
      max-parallel: 1
      matrix:
        include:
          - model: gemini-3.5-flash-lite
            slug: google-gemini-3.5-flash-lite
            workers: 1
          - model: gemma-4-31b-it
            slug: google-gemma-4-31b-it
            workers: 2
    steps:
      - name: Run paired canonical
        env:
          REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS: '6000'
          REASON_GOOGLE_SHARED_PACER_PATH: /tmp/google-pacer.state
        run: |
          CONTROL_CMD='["python3","runner.py","--coordinate-role","control"]'
          CANDIDATE_CMD='["python3","runner.py","--coordinate-role","candidate"]'
  groq-candidate:
    runs-on: ubuntu-24.04
'''

VALID_MANIFEST = {
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
        "paired_control_before_candidate_unchanged": True,
    }
}


class GoogleCanonicalPacingTests(unittest.TestCase):
    def setUp(self):
        self.google = mod.load_google_policy()
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, name: str, text: str) -> Path:
        path = self.root / name
        path.write_text(text)
        return path

    def manifest(self, mutate=None) -> Path:
        data = json.loads(json.dumps(VALID_MANIFEST))
        if mutate:
            mutate(data)
        return self.write("manifest.json", json.dumps(data))

    def test_repository_policy_locks_6000ms_and_10rpm(self):
        self.assertEqual(self.google["canonical_request_start_interval_ms"], 6000)
        self.assertEqual(self.google["canonical_max_request_starts_per_minute"], 10)
        self.assertEqual(self.google["canonical_headroom_reference_rpm"], 15)
        self.assertEqual(self.google["canonical_headroom_requests_per_minute"], 5)
        self.assertEqual(self.google["canonical_inter_case_delay_ms"], 3000)

    def test_valid_successor_workflow_and_manifest(self):
        mod.validate_workflow(self.write("workflow.yml", VALID_WORKFLOW), self.google)
        mod.validate_manifest(self.manifest(), self.google)

    def test_3000ms_successor_workflow_is_rejected(self):
        path = self.write("workflow.yml", VALID_WORKFLOW.replace("'6000'", "'3000'"))
        with self.assertRaisesRegex(mod.ValidationError, "6000ms"):
            mod.validate_workflow(path, self.google)

    def test_parallel_google_model_jobs_are_rejected(self):
        path = self.write("workflow.yml", VALID_WORKFLOW.replace("max-parallel: 1", "max-parallel: 2", 1))
        with self.assertRaisesRegex(mod.ValidationError, "max-parallel"):
            mod.validate_workflow(path, self.google)

    def test_missing_shared_pacer_is_rejected(self):
        path = self.write(
            "workflow.yml",
            VALID_WORKFLOW.replace("          REASON_GOOGLE_SHARED_PACER_PATH: /tmp/google-pacer.state\n", ""),
        )
        with self.assertRaisesRegex(mod.ValidationError, "shared pacer"):
            mod.validate_workflow(path, self.google)

    def test_worker_rate_cannot_multiply_request_start_rate(self):
        # Worker count is intentionally decoupled from the one shared request gate.
        # At 6000ms the gate admits at most 10 starts/min regardless of Gemma workers=2.
        interval = self.google["canonical_request_start_interval_ms"]
        admitted_starts = 60000 // interval
        self.assertEqual(admitted_starts, 10)
        self.assertLess(admitted_starts, self.google["canonical_headroom_reference_rpm"])
        mod.validate_workflow(self.write("workflow.yml", VALID_WORKFLOW), self.google)

    def test_pacing_env_must_share_step_with_both_coordinates(self):
        text = VALID_WORKFLOW.replace(
            "          REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS: '6000'\n",
            "",
        ).replace(
            "    steps:\n",
            "    steps:\n      - name: Detached pacing\n        env:\n          REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS: '6000'\n        run: echo pacing\n",
        )
        path = self.write("workflow.yml", text)
        with self.assertRaisesRegex(mod.ValidationError, "shared pacer path"):
            mod.validate_workflow(path, self.google)

    def test_inter_case_delay_drift_is_rejected(self):
        path = self.write("workflow.yml", VALID_WORKFLOW.replace("'3000'", "'2500'", 1))
        with self.assertRaisesRegex(mod.ValidationError, "inter-case delay"):
            mod.validate_workflow(path, self.google)

    def test_manifest_must_match_6000ms(self):
        path = self.manifest(lambda d: d["parallel_execution_policy"].update(google_request_start_interval_ms=3000))
        with self.assertRaisesRegex(mod.ValidationError, "does not match"):
            mod.validate_manifest(path, self.google)

    def test_manifest_preserves_serial_boundaries(self):
        path = self.manifest(lambda d: d["parallel_execution_policy"].update(session_cases_serial=False))
        with self.assertRaisesRegex(mod.ValidationError, "session/stateful"):
            mod.validate_manifest(path, self.google)


if __name__ == "__main__":
    unittest.main()
