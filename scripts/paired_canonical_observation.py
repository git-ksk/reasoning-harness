"""Run both coordinates of a frozen paired observation exactly once.

This helper is intentionally orchestration-only. It does not inspect evaluator
scores or retry provider failures. By default a nonzero control remains a hard
failure. A prospective evaluator may explicitly opt in to delegate a nonzero
control with preserved canonical evidence to its acceptance comparator; candidate
nonzero remains a hard failure. Optional heartbeat output is stderr-only.
"""

from __future__ import annotations

import argparse
from dataclasses import asdict, dataclass
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time
from typing import Callable, Sequence, TextIO

CONTRACT_ID = "paired-canonical-observation-v2"
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
    control_nonzero_with_evidence_allowed: bool
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
    allow_control_nonzero_with_evidence: bool = False,
) -> PairedObservationResult:
    """Execute control and candidate once each, regardless of the first exit code.

    Acceptance executes once when both coordinate evidence sets exist. By default
    both coordinate commands must exit zero. When the prospective control-delegation
    option is explicit, control nonzero may be decided by acceptance if its required
    evidence exists; candidate still must exit zero. There is no retry loop here.
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
        and (control.returncode == 0 or allow_control_nonzero_with_evidence)
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
        control_nonzero_with_evidence_allowed=allow_control_nonzero_with_evidence,
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


def _subprocess_executor(
    stdout_path: Path | None,
    *,
    label: str | None = None,
    heartbeat_seconds: int | None = None,
    progress_stream: TextIO = sys.stderr,
) -> Callable[[tuple[str, ...]], int]:
    def execute(command: tuple[str, ...]) -> int:
        stream = None
        if stdout_path is not None:
            stdout_path.parent.mkdir(parents=True, exist_ok=True)
            stream = stdout_path.open("wb")
        started = time.monotonic()
        if label is not None:
            print(f"[paired-canonical] {label} started", file=progress_stream, flush=True)
        try:
            process = subprocess.Popen(command, stdout=stream)
            if heartbeat_seconds is None:
                returncode = process.wait()
            else:
                while True:
                    try:
                        returncode = process.wait(timeout=heartbeat_seconds)
                        break
                    except subprocess.TimeoutExpired:
                        elapsed = int(time.monotonic() - started)
                        print(
                            f"[paired-canonical] {label} elapsed={elapsed}s",
                            file=progress_stream,
                            flush=True,
                        )
            if label is not None:
                elapsed = int(time.monotonic() - started)
                print(
                    f"[paired-canonical] {label} completed rc={returncode} elapsed={elapsed}s",
                    file=progress_stream,
                    flush=True,
                )
            return returncode
        finally:
            if stream is not None:
                stream.close()

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
    parser.add_argument(
        "--allow-control-nonzero-with-evidence",
        action="store_true",
        help="permit acceptance to decide a nonzero control when canonical control evidence exists",
    )
    parser.add_argument(
        "--heartbeat-seconds",
        type=int,
        help="emit orchestration-only progress every 30-60 seconds without changing captured stdout",
    )
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

    if args.heartbeat_seconds is not None and not 30 <= args.heartbeat_seconds <= 60:
        raise SystemExit("--heartbeat-seconds must be between 30 and 60")

    result = run_paired_canonical_observation(
        control_command,
        candidate_command,
        acceptance_command,
        control_required_paths=tuple(Path(path) for path in args.control_required),
        candidate_required_paths=tuple(Path(path) for path in args.candidate_required),
        acceptance_required_paths=tuple(Path(path) for path in args.acceptance_required),
        execute_control=_subprocess_executor(
            Path(args.control_stdout) if args.control_stdout else None,
            label="control",
            heartbeat_seconds=args.heartbeat_seconds,
        ),
        execute_candidate=_subprocess_executor(
            Path(args.candidate_stdout) if args.candidate_stdout else None,
            label="candidate",
            heartbeat_seconds=args.heartbeat_seconds,
        ),
        execute_acceptance=_subprocess_executor(
            Path(args.acceptance_stdout) if args.acceptance_stdout else None,
            label="acceptance",
            heartbeat_seconds=args.heartbeat_seconds,
        ),
        allow_control_nonzero_with_evidence=args.allow_control_nonzero_with_evidence,
    )
    _write_json(Path(args.output), result)
    return 0 if result.hard_gate_passed else HARD_GATE_FAILURE_EXIT


if __name__ == "__main__":
    raise SystemExit(main())
