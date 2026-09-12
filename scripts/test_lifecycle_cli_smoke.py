#!/usr/bin/env python3
import json
import os
import platform
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REASON = ROOT / "target" / "debug" / ("reason.exe" if platform.system() == "Windows" else "reason")


def run(args, expected):
    env = os.environ.copy()
    env["REASON_HOME"] = str(ROOT / "target" / "lifecycle-smoke-home")
    completed = subprocess.run(
        [str(REASON), *args], text=True, capture_output=True, env=env, check=False
    )
    assert completed.returncode == expected, (args, completed.returncode, completed.stdout, completed.stderr)
    return completed


def main():
    assert REASON.is_file(), REASON
    dry = run(["uninstall", "--dry-run", "--purge-data", "--format", "json"], 0)
    assert dry.stderr == "", dry.stderr
    doc = json.loads(dry.stdout)
    assert doc["command"] == "uninstall"
    assert doc["result"]["dry_run"] is True
    assert doc["result"]["binary_removed"] is False
    assert doc["result"]["binary_removal_scheduled"] is False
    assert REASON.is_file(), "dry-run removed the CLI binary"

    rollback = run(["update", "--rollback", "0.4.2", "--yes", "--format", "json"], 1)
    assert rollback.stderr == "", rollback.stderr
    doc = json.loads(rollback.stdout)
    assert doc["command"] == "update"
    assert doc["result"]["failure"]["failure_class"] == "historical_release_boundary"

    for command in ("update", "uninstall"):
        help_result = run([command, "--help"], 0)
        text = help_result.stdout.lower()
        for forbidden in ("--insecure", "--skip-verify", "--no-verify"):
            assert forbidden not in text, (command, forbidden, text)


if __name__ == "__main__":
    main()
