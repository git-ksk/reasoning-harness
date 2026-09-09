"""Run both coordinates of a frozen paired observation exactly once.

This helper is intentionally orchestration-only.  It does not inspect evaluator
scores, retry provider failures, or reinterpret a coordinate's exit status.  Its
purpose is to preserve candidate evidence when a control command exits nonzero,
then fail the overall gate after all available canonical evidence is recorded.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
import hashlib
import json
from pathlib import Path
import subprocess
from typing import Callable, Sequence

CONTRACT_ID = "paired-canonical-observation-v1"
HARD_GATE_FAILURE_EXIT = 3


@dataclass(frozen=True)
class InvocationResult:
    launched: bool
    returncode: int | None
    command_fingerprint: str
    required_evidence_present: bool
    missing_required_evidence: tuple[str, ...]
    launch_error: str | None


@dataclass(frozen=True)
class PairedObservationResult:
    contract: str
    control: InvocationResult
    candidate: InvocationResult
    acceptance: InvocationResult | None
    acceptance_skip_reason: str | None
    hard_gate_passed: bool


def command_fingerprint(command: Sequence[str]) -> str:
    encoded = json.dumps(list(command), ensure_ascii=False, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def _missing_required(paths: Sequence[Path]) -> tuple[str, ...]:
    return tuple(str(path) for path in paths if not path.exists())


def _invoke_once(
    command: Sequence[str],
    required_paths: Sequence[Path],
    execute: Callable[[tuple[str, ...]], int],
) -> InvocationResult:
    frozen_command = tuple(str(part) for part in command)
    fingerprint = command_fingerprint(frozen_command)
    try:
        returncode = execute(frozen_command)
        if not isinstance(returncode, int):
            raise TypeError("execute must return an int return code")
        launch_error = None
        launched = True
    except OSError as error:
        returncode = None
        launch_error = f"{type(error).__name__}: {error}"
        launched = False

    missing = _missing_required(required_paths)
    return InvocationResult(
        launched=launched,
        returncode=returncode,
        command_fingerprint=fingerprint,
        required_evidence_present=not missing,
        missing_required_evidence=missing,
        launch_error=launch_error,
    )


def run_paired_canonical_observation(
    control_command: Sequence[str],
    candidate_command: Sequence[str],
    acceptance_command: Sequence[str],
    *,
    control_required_paths: Sequence[Path] = (),
    candidate_required_paths: Sequence[Path] = (),
    acceptance_required_paths: Sequence[Path] = (),
    execute_control: Callable[[tuple[str, ...]], int],
    execute_candidate: Callable[[tuple[str, ...]], int],
    execute_acceptance: Callable[[tuple[str, ...]], int],
) -> PairedObservationResult:
    """Execute control and candidate once each, regardless of the first exit code.

    Acceptance executes once when both coordinate evidence sets exist.  Overall
    success still requires both coordinate commands and acceptance to succeed and
    all required evidence to exist.  There is deliberately no retry loop here.
    """

    if not control_command or not candidate_command or not acceptance_command:
        raise ValueError("control, candidate, and acceptance commands must be non-empty")
    if (
        not control_required_paths
        or not candidate_required_paths
        or not acceptance_required_paths
    ):
        raise ValueError(
            "control, candidate, and acceptance must each declare required evidence paths"
        )

    control = _invoke_once(control_command, control_required_paths, execute_control)
    candidate = _invoke_once(candidate_command, candidate_required_paths, execute_candidate)

    coordinate_evidence_present = (
        control.required_evidence_present and candidate.required_evidence_present
    )
    acceptance: InvocationResult | None
    acceptance_skip_reason: str | None
    if coordinate_evidence_present:
        acceptance = _invoke_once(
            acceptance_command,
            acceptance_required_paths,
            execute_acceptance,
        )
        acceptance_skip_reason = None
    else:
        acceptance = None
        acceptance_skip_reason = "coordinate_required_evidence_missing"

    hard_gate_passed = (
        control.launched
        and control.returncode == 0
        and control.required_evidence_present
        and candidate.launched
        and candidate.returncode == 0
        and candidate.required_evidence_present
        and acceptance is not None
        and acceptance.launched
        and acceptance.returncode == 0
        and acceptance.required_evidence_present
    )

    return PairedObservationResult(
        contract=CONTRACT_ID,
        control=control,
        candidate=candidate,
        acceptance=acceptance,
        acceptance_skip_reason=acceptance_skip_reason,
        hard_gate_passed=hard_gate_passed,
    )


def _json_command(raw: str, name: str) -> tuple[str, ...]:
    try:
        value = json.loads(raw)
    except json.JSONDecodeError as error:
        raise ValueError(f"{name} must be a JSON array of strings: {error}") from error
    if not isinstance(value, list) or not value or not all(
        isinstance(part, str) and part for part in value
    ):
        raise ValueError(f"{name} must be a non-empty JSON array of non-empty strings")
    return tuple(value)


def _subprocess_executor(stdout_path: Path | None) -> Callable[[tuple[str, ...]], int]:
    def execute(command: tuple[str, ...]) -> int:
        if stdout_path is None:
            return subprocess.run(command, check=False).returncode
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        with stdout_path.open("wb") as stream:
            return subprocess.run(command, check=False, stdout=stream).returncode

    return execute


def _write_json(path: Path, result: PairedObservationResult) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp")
    temporary.write_text(
        json.dumps(asdict(result), indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    temporary.replace(path)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--control-command-json", required=True)
    parser.add_argument("--candidate-command-json", required=True)
    parser.add_argument("--acceptance-command-json", required=True)
    parser.add_argument("--control-required", action="append", default=[])
    parser.add_argument("--candidate-required", action="append", default=[])
    parser.add_argument("--acceptance-required", action="append", default=[])
    parser.add_argument("--control-stdout")
    parser.add_argument("--candidate-stdout")
    parser.add_argument("--acceptance-stdout")
    parser.add_argument("--output", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        control_command = _json_command(args.control_command_json, "control command")
        candidate_command = _json_command(args.candidate_command_json, "candidate command")
        acceptance_command = _json_command(args.acceptance_command_json, "acceptance command")
    except ValueError as error:
        raise SystemExit(str(error)) from error

    result = run_paired_canonical_observation(
        control_command,
        candidate_command,
        acceptance_command,
        control_required_paths=tuple(Path(path) for path in args.control_required),
        candidate_required_paths=tuple(Path(path) for path in args.candidate_required),
        acceptance_required_paths=tuple(Path(path) for path in args.acceptance_required),
        execute_control=_subprocess_executor(Path(args.control_stdout) if args.control_stdout else None),
        execute_candidate=_subprocess_executor(Path(args.candidate_stdout) if args.candidate_stdout else None),
        execute_acceptance=_subprocess_executor(
            Path(args.acceptance_stdout) if args.acceptance_stdout else None
        ),
    )
    _write_json(Path(args.output), result)
    return 0 if result.hard_gate_passed else HARD_GATE_FAILURE_EXIT


if __name__ == "__main__":
    raise SystemExit(main())
