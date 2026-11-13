"""Offline integrity and privacy checks for Development Lite comparison report."""
from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

import review
import run as lite
import test_run as fixtures


class ReportTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.DevLiteTests("test_four_or_six_distinct_cases_and_alternating_arms")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.output = self.fixture.root / "private-run"

    def fake_run(self, *, status=None, malformed=False):
        if malformed:
            return lambda args, **kwargs: subprocess.CompletedProcess(args, 0, b"non-json", b"")
        def execute(argv, **kwargs):
            result = json.loads(self.fixture.cli_result().decode())
            result["result"]["task"] = argv[-1]
            result["result"]["finalization"]["text"] = (
                "Evidence-based answer from " +
                ("baseline" if "baseline" in argv[0] else "candidate"))
            result["result"]["answer_context"] = {
                "citations": [{"source_id": "original-manual-captured", "quote": "example"}]}
            if status:
                result["result"]["finalization"]["status"] = status
            return subprocess.CompletedProcess(argv, 0, json.dumps(result).encode(), b"")
        return execute

    def create_run(self, *, malformed=False):
        with (patch.dict(os.environ, {"REASON_DEV_LITE_FREE_CONFIRMED": "1"}),
              patch.object(lite.subprocess, "run", side_effect=self.fake_run(malformed=malformed))):
            return lite.run_live(self.fixture.prepared(), self.output,
                                 confirmed=True, token="fixture-only",
                                 timeout=3, total_budget=96000)

    def test_completed_private_report_is_unscored_and_source_names_visible(self):
        run = self.create_run()
        self.assertEqual(run["completed_jobs"], 8)
        root, report, count, cases = review.build_report(self.output)
        self.assertEqual(count, 8)
        self.assertEqual(cases, 4)
        content = report.decode()
        for needle in ("rw-01", "rw-06", "rw-07", "rw-23",
                       "Evidence-based answer from baseline",
                       "Evidence-based answer from candidate",
                       "original-manual-captured", "Human reviewer worksheet",
                       "Formal F1: **not computed**", "Quality GO: **false**"):
            self.assertIn(needle, content)
        self.assertNotIn("fixture-only", content)
        self.assertNotIn("quality_go=true", content)
        self.assertEqual(review.main(["--run-dir", str(self.output)]), 0)
        file = self.output / "comparison-review.md"
        self.assertEqual(file.stat().st_mode & 0o777, 0o600)
        with self.assertRaisesRegex(lite.GateError, "refuse_to_overwrite"):
            root, report, _, _ = review.build_report(self.output)
            review_root_path = root / "comparison-review.md"
            lite.need(not review_root_path.exists(), "refuse_to_overwrite_existing_review")

    def test_partial_run_does_not_fabricate_other_arm(self):
        result = self.create_run(malformed=True)
        self.assertEqual(result["completed_jobs"], 1)
        self.assertEqual(result["status"], "stopped_unknown_usage_or_output")
        _, report, count, cases = review.build_report(self.output)
        self.assertEqual((count, cases), (1, 4))
        text = report.decode()
        self.assertIn("Not executed. **Not scored.**", text)
        self.assertIn("No valid CLI JSON; do not grade.", text)

    def test_tampered_output_is_rejected_before_report(self):
        self.create_run()
        path = self.output / "rw-01-baseline.stdout"
        path.write_bytes(path.read_bytes() + b"altered")
        os.chmod(path, 0o600)
        with self.assertRaisesRegex(lite.GateError, "review_output_sha256_mismatch"):
            review.build_report(self.output)
        self.assertFalse((self.output / "comparison-review.md").exists())

    def test_world_readable_file_and_folder_refused(self):
        self.create_run()
        p = self.output / "rw-01-baseline.stdout"
        os.chmod(p, 0o644)
        with self.assertRaisesRegex(lite.GateError, "review_file_permission_or_size"):
            review.build_report(self.output)
        os.chmod(p, 0o600)
        os.chmod(self.output, 0o755)
        with self.assertRaisesRegex(lite.GateError, "private_run_folder_mode_0700_required"):
            review.build_report(self.output)

    def test_symlink_run_directory_refused(self):
        self.create_run()
        link = self.fixture.root / "symlink-run"
        link.symlink_to(self.output)
        with self.assertRaisesRegex(lite.GateError, "private_run_folder_required"):
            review.build_report(link)

    def test_unexpected_job_or_unofficial_score_refused(self):
        self.create_run()
        path = self.output / "final-summary.json"
        final = json.loads(path.read_text())
        final["formal_f1"] = 1.0
        path.write_text(json.dumps(final))
        os.chmod(path, 0o600)
        with self.assertRaisesRegex(lite.GateError, "review_manifest_mismatch"):
            review.build_report(self.output)

    def test_completed_status_must_include_both_arms(self):
        self.create_run()
        path = self.output / "final-summary.json"
        final = json.loads(path.read_text())
        final["result"] = final["result"][:1]
        final["completed_jobs"] = 1
        path.write_text(json.dumps(final))
        os.chmod(path, 0o600)
        with self.assertRaisesRegex(lite.GateError, "completed_run_missing_results"):
            review.build_report(self.output)

    def test_token_manifest_mismatch_is_not_silent(self):
        self.create_run()
        path = self.output / "final-summary.json"
        final = json.loads(path.read_text())
        final["total_known_tokens"] += 1
        path.write_text(json.dumps(final))
        os.chmod(path, 0o600)
        with self.assertRaisesRegex(lite.GateError, "review_token_manifest_mismatch"):
            review.build_report(self.output)

    def test_question_mismatch_and_html_controls(self):
        source = self.fixture.cli_result()
        data = json.loads(source)
        data["result"]["task"] = "wrong question"
        with self.assertRaisesRegex(lite.GateError, "review_output_question_changed"):
            review.summarize_file(json.dumps(data).encode(), expected_question="expected")
        cleaned = review.protect_text("unsafe\u202e\n<div>")
        self.assertIn("unsafe\\u202e", cleaned)
        self.assertIn("    <div>", cleaned)
        self.assertIn("TRUNCATED FOR REVIEW", review.protect_text("a" * (review.MAX_ANSWER_CHARS + 1)))
        malformed = {"schema_version": "reason-cli-output-v1", "result": {
            "answer_context": {"citations": [{"source_id": "bad\n|injection"}]},
            "finalization": {"text": "ok"}}}
        info = review.summarize_file(json.dumps(malformed).encode(), expected_question="expected")
        self.assertEqual(info["citations"], ["(noncanonical-source-id)"])


if __name__ == "__main__":
    unittest.main()
