import tempfile
from pathlib import Path
import unittest

from scripts.paired_canonical_observation import run_paired_canonical_observation


class PairedCanonicalObservationTests(unittest.TestCase):
    def setUp(self):
        self.tempdir = tempfile.TemporaryDirectory()
        self.root = Path(self.tempdir.name)
        self.control_report = self.root / "control.json"
        self.candidate_report = self.root / "candidate.json"
        self.acceptance_report = self.root / "acceptance.json"

    def tearDown(self):
        self.tempdir.cleanup()

    def _executor(self, name, returncode, *, create=None, seen=None):
        def execute(command):
            if seen is not None:
                seen.append((name, command))
            if create is not None:
                create.write_text("{}\n", encoding="utf-8")
            return returncode

        return execute

    def _run(self, control_rc=0, candidate_rc=0, acceptance_rc=0, *, create_control=True, create_candidate=True):
        seen = []
        result = run_paired_canonical_observation(
            ["control", "--seed", "95100"],
            ["candidate", "--seed", "95100"],
            ["acceptance"],
            control_required_paths=[self.control_report],
            candidate_required_paths=[self.candidate_report],
            acceptance_required_paths=[self.acceptance_report],
            execute_control=self._executor(
                "control",
                control_rc,
                create=self.control_report if create_control else None,
                seen=seen,
            ),
            execute_candidate=self._executor(
                "candidate",
                candidate_rc,
                create=self.candidate_report if create_candidate else None,
                seen=seen,
            ),
            execute_acceptance=self._executor(
                "acceptance", acceptance_rc, create=self.acceptance_report, seen=seen
            ),
        )
        return result, seen

    def test_all_pass_invokes_each_coordinate_and_acceptance_once(self):
        result, seen = self._run()
        self.assertTrue(result.hard_gate_passed)
        self.assertEqual([name for name, _ in seen], ["control", "candidate", "acceptance"])

    def test_control_nonzero_can_be_delegated_to_acceptance_only_when_explicitly_enabled(self):
        seen = []
        result = run_paired_canonical_observation(
            ["control"],
            ["candidate"],
            ["acceptance"],
            control_required_paths=[self.control_report],
            candidate_required_paths=[self.candidate_report],
            acceptance_required_paths=[self.acceptance_report],
            execute_control=self._executor("control", 4, create=self.control_report, seen=seen),
            execute_candidate=self._executor("candidate", 0, create=self.candidate_report, seen=seen),
            execute_acceptance=self._executor("acceptance", 0, create=self.acceptance_report, seen=seen),
            allow_control_nonzero_with_evidence=True,
        )
        self.assertTrue(result.hard_gate_passed)
        self.assertEqual(result.control.returncode, 4)
        self.assertEqual([name for name, _ in seen], ["control", "candidate", "acceptance"])

    def test_control_failure_with_report_still_runs_candidate_and_acceptance_once(self):
        result, seen = self._run(control_rc=3)
        self.assertFalse(result.hard_gate_passed)
        self.assertEqual(result.control.returncode, 3)
        self.assertEqual(result.candidate.returncode, 0)
        self.assertEqual([name for name, _ in seen], ["control", "candidate", "acceptance"])

    def test_candidate_failure_with_report_keeps_control_and_acceptance_evidence(self):
        result, seen = self._run(candidate_rc=3)
        self.assertFalse(result.hard_gate_passed)
        self.assertEqual([name for name, _ in seen], ["control", "candidate", "acceptance"])

    def test_both_coordinate_failures_with_reports_are_each_invoked_once(self):
        result, seen = self._run(control_rc=3, candidate_rc=4)
        self.assertFalse(result.hard_gate_passed)
        self.assertEqual([name for name, _ in seen].count("control"), 1)
        self.assertEqual([name for name, _ in seen].count("candidate"), 1)
        self.assertEqual([name for name, _ in seen].count("acceptance"), 1)

    def test_missing_control_report_does_not_prevent_candidate_but_skips_acceptance(self):
        result, seen = self._run(control_rc=3, create_control=False)
        self.assertFalse(result.hard_gate_passed)
        self.assertFalse(result.control.required_evidence_present)
        self.assertTrue(result.candidate.required_evidence_present)
        self.assertIsNone(result.acceptance)
        self.assertEqual(result.acceptance_skip_reason, "coordinate_required_evidence_missing")
        self.assertEqual([name for name, _ in seen], ["control", "candidate"])

    def test_missing_candidate_report_is_hard_failure_and_skips_acceptance(self):
        result, seen = self._run(candidate_rc=3, create_candidate=False)
        self.assertFalse(result.hard_gate_passed)
        self.assertTrue(result.control.required_evidence_present)
        self.assertFalse(result.candidate.required_evidence_present)
        self.assertEqual([name for name, _ in seen], ["control", "candidate"])

    def test_acceptance_failure_is_not_hidden(self):
        result, seen = self._run(acceptance_rc=7)
        self.assertFalse(result.hard_gate_passed)
        self.assertIsNotNone(result.acceptance)
        self.assertEqual(result.acceptance.returncode, 7)
        self.assertEqual([name for name, _ in seen], ["control", "candidate", "acceptance"])

    def test_launch_error_on_control_still_runs_candidate(self):
        seen = []

        def missing_control(command):
            seen.append(("control", command))
            raise FileNotFoundError("control binary missing")

        result = run_paired_canonical_observation(
            ["control"],
            ["candidate"],
            ["acceptance"],
            control_required_paths=[self.control_report],
            candidate_required_paths=[self.candidate_report],
            acceptance_required_paths=[self.acceptance_report],
            execute_control=missing_control,
            execute_candidate=self._executor(
                "candidate", 0, create=self.candidate_report, seen=seen
            ),
            execute_acceptance=self._executor(
                "acceptance", 0, create=self.acceptance_report, seen=seen
            ),
        )
        self.assertFalse(result.hard_gate_passed)
        self.assertFalse(result.control.launched)
        self.assertIn("FileNotFoundError", result.control.launch_error)
        self.assertEqual([name for name, _ in seen], ["control", "candidate"])
        self.assertIsNone(result.acceptance)
        self.assertEqual(result.acceptance_skip_reason, "coordinate_required_evidence_missing")

    def test_commands_are_fingerprinted_but_not_stored_in_result(self):
        result, _ = self._run()
        self.assertEqual(len(result.control.command_fingerprint), 64)
        self.assertFalse(hasattr(result.control, "command"))

    def test_empty_command_is_rejected_before_execution(self):
        with self.assertRaises(ValueError):
            run_paired_canonical_observation(
                [],
                ["candidate"],
                ["acceptance"],
                execute_control=lambda command: 0,
                execute_candidate=lambda command: 0,
                execute_acceptance=lambda command: 0,
            )


if __name__ == "__main__":
    unittest.main()
