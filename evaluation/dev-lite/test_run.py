"""Provider-free validation; no credential, model or external API usage."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import run as lite


def checksum(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


class DevLiteTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.binaries = {}
        arms = {}
        for arm in lite.ARMS:
            binary = self.root / ("reason-" + arm)
            binary.write_bytes(("synthetic-fake-" + arm).encode())
            self.binaries[arm] = binary
            arms[arm] = {"binary": str(binary), "binary_sha256": checksum(binary)}
        self.build_path = self.root / "build.json"
        self.build_path.write_text(json.dumps({"contract": "reason-cli-normal-v3-native-paired-build-v1",
             "cli_commit": "a" * 40, "release_or_adoption": False, "arms": arms}))
        jobs = []
        for index in range(1, 25):
            cid = f"rw-{index:02d}"
            config = self.root / (cid + ".json")
            config.write_text("{}")
            for arm in lite.ARMS:
                jobs.append({"case_id": cid, "arm": arm, "job_id": cid + "-" + arm,
                             "argv": [str(self.binaries[arm]), "--plain", "--provider", "mistral",
                                      "--model", "ministral-8b-2512", "--max-model-calls", "12",
                                      "--max-tokens", "1024", "--max-output-tokens", "4096",
                                      "--max-total-tokens", "28096", "--config", str(config),
                                      "--format", "json", "Question " + cid],
                             "request": {"request_sha256": "b" * 64, "request": {"prompt": cid}}})
        self.requests_path = self.root / "requests.json"
        self.requests_path.write_text(json.dumps({"contract": "reason-cli-normal-v3-prospective-requests-v1",
                                                    "jobs": jobs}))
        self.caps = {"model_calls": 6, "generation_tokens": 512,
                     "output_tokens": 1536, "total_tokens": 12000}

    def prepared(self, cases=None):
        return lite.prepare(self.requests_path, self.build_path, cases or list(lite.DEFAULT_CASES), self.caps)

    def cli_result(self, *, usage=True, failure=False):
        result = {"finalization": {"status": "grounded_answer"},
                  "usage": {"run": {"tracked_from_session_start": True,
                                    "model_calls": 2, "provider_attempts": 2,
                                    "input_tokens": 40, "output_tokens": 10, "total_tokens": 50}}}
        if not usage:
            result.pop("usage")
        if failure:
            result["status"] = "failed"
        return json.dumps({"schema_version": "reason-cli-output-v1",
                           "command": "ask", "result": result}).encode()

    def test_four_or_six_distinct_cases_and_alternating_arms(self):
        p = self.prepared()
        self.assertEqual(p["cases"], list(lite.DEFAULT_CASES))
        self.assertEqual([(x["arm"]) for x in p["jobs"][:6]],
                         ["baseline", "candidate", "candidate", "baseline", "baseline", "candidate"])
        self.assertEqual(len(self.prepared([f"rw-{i:02d}" for i in range(1, 7)])["jobs"]), 12)
        self.assertEqual(p["formal_f1"], None)
        self.assertFalse(p["quality_go"])
        self.assertTrue(all("--evaluation-isolated-config" in row["argv"] for row in p["jobs"]))
        self.assertTrue(all(lite.flag(row["argv"], "--max-model-calls") == "6" for row in p["jobs"]))

    def test_rebind_requires_explicit_opt_in(self):
        data = json.loads(self.requests_path.read_text())
        stale = self.root / "old-binary"
        stale.write_bytes(b"old-prepared-binary")
        for row in data["jobs"]:
            row["argv"][0] = str(stale)
        self.requests_path.write_text(json.dumps(data))
        with self.assertRaisesRegex(lite.GateError, "binary_build_path_mismatch"):
            self.prepared()
        p = lite.prepare(self.requests_path, self.build_path,
                         list(lite.DEFAULT_CASES), self.caps, rebind_binaries=True)
        self.assertTrue(p["development_binary_rebind"])
        self.assertTrue(all(Path(x["argv"][0]) == self.binaries[x["arm"]].resolve() for x in p["jobs"]))

    def test_reject_invalid_subset_and_cap(self):
        for cases in [[], ["rw-01"] * 4, ["rw-25", "rw-01", "rw-02", "rw-03"]]:
            with self.assertRaises(lite.GateError):
                # cases_from_string tests cardinality; prepare tests identity.
                lite.cases_from_string(",".join(cases))
        with self.assertRaises(lite.GateError):
            lite.prepare(self.requests_path, self.build_path, ["rw-01", "rw-02", "rw-03", "rw-25"], self.caps)
        with self.assertRaises(lite.GateError):
            lite.prepare(self.requests_path, self.build_path, list(lite.DEFAULT_CASES),
                         {**self.caps, "model_calls": 13})

    def test_reject_tampered_binary_and_paired_requests(self):
        self.binaries["candidate"].write_bytes(b"changed")
        with self.assertRaisesRegex(lite.GateError, "binary_hash_mismatch"):
            self.prepared()
        self.binaries["candidate"].write_bytes(b"synthetic-fake-candidate")
        data = json.loads(self.requests_path.read_text())
        data["jobs"][1]["request"]["request_sha256"] = "c" * 64
        self.requests_path.write_text(json.dumps(data))
        with self.assertRaisesRegex(lite.GateError, "first_requests_not_paired"):
            self.prepared()

    def test_provider_free_native_preflight_pair_identity(self):
        plan = self.prepared()
        def fake_native(args, **kwargs):
            destination = Path(args[-1])
            cid = destination.name[:5]
            destination.write_text(json.dumps({
                "contract": "reason-native-first-request-plan-v1",
                "provider": "mistral", "model": "ministral-8b-2512",
                "question": "Question " + cid,
                "request_sha256": hashlib.sha256(cid.encode()).hexdigest(),
                "request": {"prompt": cid}}))
            self.assertNotIn("MISTRAL_API_KEY", kwargs["env"])
            return subprocess.CompletedProcess(args, 0, b"", b"")
        with patch.object(lite.subprocess, "run", side_effect=fake_native) as mocked:
            result = lite.native_preflight(plan)
        self.assertEqual(result["native_plans_verified"], 8)
        self.assertEqual(result["model_api_calls"], 0)
        self.assertEqual(mocked.call_count, 8)

    def test_preflight_rejects_unmatched_first_requests(self):
        plan = self.prepared()
        def fake_mismatch(args, **kwargs):
            destination = Path(args[-1])
            cid = destination.name[:5]
            arm = "baseline" if "baseline" in destination.name else "candidate"
            destination.write_text(json.dumps({
                "contract": "reason-native-first-request-plan-v1",
                "provider": "mistral", "model": "ministral-8b-2512",
                "question": "Question " + cid,
                "request_sha256": hashlib.sha256((cid+arm).encode()).hexdigest(),
                "request": {"prompt": cid+arm}}))
            return subprocess.CompletedProcess(args, 0, b"", b"")
        with patch.object(lite.subprocess, "run", side_effect=fake_mismatch):
            with self.assertRaisesRegex(lite.GateError, "development_first_requests_not_identical"):
                lite.native_preflight(plan)

    def test_live_requires_external_no_charge_confirmation(self):
        output = self.root / "out"
        with self.assertRaisesRegex(lite.GateError, "explicit_current_free_only"):
            lite.run_live(self.prepared(), output, confirmed=False, token="fixture",
                          timeout=10, total_budget=96000)
        self.assertFalse(output.exists())

    def test_accounted_offline_fixture_has_no_f1_or_signatures(self):
        output = self.root / "out"
        fake = subprocess.CompletedProcess(args=[], returncode=0, stdout=self.cli_result(), stderr=b"")
        with (patch.dict(os.environ, {"REASON_DEV_LITE_FREE_CONFIRMED": "1"}),
              patch.object(lite.subprocess, "run", return_value=fake) as mocked):
            result = lite.run_live(self.prepared(), output, confirmed=True, token="fixture-only",
                                   timeout=3, total_budget=96000)
        self.assertEqual(result["status"], "complete")
        self.assertEqual(result["completed_jobs"], 8)
        self.assertEqual(result["total_known_tokens"], 400)
        self.assertFalse(result["quality_go"])
        self.assertEqual(result["formal_f1"], None)
        self.assertEqual(mocked.call_count, 8)
        self.assertEqual(output.stat().st_mode & 0o777, 0o700)
        self.assertEqual((output / "final-summary.json").stat().st_mode & 0o777, 0o600)
        self.assertNotIn("fixture-only", (output / "plan.json").read_text())
        self.assertTrue(all(x["review_status"] == "unreviewed" for x in result["result"]))

    def test_malformed_cli_shapes_are_fails_closed(self):
        for raw in (b"null", b"[]", b'{"schema_version":"reason-cli-output-v1","command":"ask","result":null}',
                    b'{"schema_version":"reason-cli-output-v1","command":"ask","result":{"usage":[]}}'):
            with self.assertRaises(lite.GateError):
                lite.usage_from_cli(raw)

    def test_unknown_usage_halts_and_does_not_retry(self):
        output = self.root / "out"
        fake = subprocess.CompletedProcess(args=[], returncode=0,
                                           stdout=self.cli_result(usage=False), stderr=b"")
        with (patch.dict(os.environ, {"REASON_DEV_LITE_FREE_CONFIRMED": "1"}),
              patch.object(lite.subprocess, "run", return_value=fake) as mocked):
            result = lite.run_live(self.prepared(), output, confirmed=True,
                                   token="fixture-only", timeout=3, total_budget=96000)
        self.assertEqual(result["status"], "stopped_unknown_usage_or_output")
        self.assertEqual(result["completed_jobs"], 1)
        self.assertEqual(mocked.call_count, 1)
        self.assertEqual(len(list(output.glob("checkpoint-*.json"))), 1)

    def test_timeout_halts_without_retry(self):
        output = self.root / "out"
        with (patch.dict(os.environ, {"REASON_DEV_LITE_FREE_CONFIRMED": "1"}),
              patch.object(lite.subprocess, "run",
                           side_effect=subprocess.TimeoutExpired(cmd="fake", timeout=3)) as mocked):
            result = lite.run_live(self.prepared(), output, confirmed=True,
                                   token="fixture-only", timeout=3, total_budget=96000)
        self.assertEqual(result["status"], "stopped_timeout")
        self.assertEqual(mocked.call_count, 1)

    def test_private_destination_and_global_reservation(self):
        plan = self.prepared()
        with patch.dict(os.environ, {"REASON_DEV_LITE_FREE_CONFIRMED": "1"}):
            with self.assertRaisesRegex(lite.GateError, "global_budget"):
                lite.run_live(plan, self.root / "out", confirmed=True,
                              token="fixture-only", timeout=3, total_budget=1)
            with self.assertRaisesRegex(lite.GateError, "outside_repository"):
                lite.run_live(plan, lite.REPO / "unwanted-evaluation", confirmed=True,
                              token="fixture-only", timeout=3, total_budget=96000)


if __name__ == "__main__":
    unittest.main()
