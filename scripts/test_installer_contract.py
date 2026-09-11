#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parent.parent
INSTALLER = ROOT / "install.sh"


def write_executable(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")
    path.chmod(0o755)


class InstallerContractTests(unittest.TestCase):
    def make_mock_release(
        self,
        base: Path,
        *,
        reported_version: str = "0.4.2",
        correct_checksum: bool = True,
        os_name: str = "Linux",
        arch: str = "x86_64",
    ) -> tuple[Path, Path]:
        tools = base / "tools"
        release = base / "release"
        tools.mkdir()
        release.mkdir()

        write_executable(
            tools / "uname",
            f'#!/bin/sh\n[ "$1" = "-s" ] && echo {os_name} || echo {arch}\n',
        )
        package = base / "reason-v0.4.2-linux-x86_64"
        package.mkdir()
        write_executable(package / "reason", f'#!/bin/sh\necho "reason {reported_version}"\n')

        archive = release / "reason-v0.4.2-linux-x86_64.tar.gz"
        with tarfile.open(archive, "w:gz") as output:
            output.add(package, arcname=package.name)
        digest = hashlib.sha256(archive.read_bytes()).hexdigest()
        if not correct_checksum:
            digest = "0" * 64
        (release / "SHA256SUMS").write_text(f"{digest}  {archive.name}\n", encoding="utf-8")

        write_executable(
            tools / "curl",
            """#!/bin/sh
out=""
url=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) out="$2"; shift 2 ;;
    -*) shift ;;
    *) url="$1"; shift ;;
  esac
done
cp "$MOCK_RELEASE_DIR/${url##*/}" "$out"
""",
        )
        return tools, release

    def run_mock(
        self,
        *,
        reported_version: str = "0.4.2",
        correct_checksum: bool = True,
        os_name: str = "Linux",
        arch: str = "x86_64",
        existing_binary: bool = False,
    ) -> tuple[subprocess.CompletedProcess[str], str | None]:
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            bin_dir = base / "bin"
            bin_dir.mkdir()
            tools, release = self.make_mock_release(
                base,
                reported_version=reported_version,
                correct_checksum=correct_checksum,
                os_name=os_name,
                arch=arch,
            )
            if existing_binary:
                write_executable(bin_dir / "reason", "#!/bin/sh\necho old\n")

            env = os.environ.copy()
            env["PATH"] = str(tools) + os.pathsep + env["PATH"]
            env["MOCK_RELEASE_DIR"] = str(release)
            completed = subprocess.run(
                [str(INSTALLER), "--version", "0.4.2", "--bin-dir", str(bin_dir)],
                cwd=ROOT,
                env=env,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            destination = bin_dir / "reason"
            installed = destination.read_text(encoding="utf-8") if destination.exists() else None
            return completed, installed

    def test_success_installs_verified_binary(self) -> None:
        completed, installed = self.run_mock()
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIsNotNone(installed)
        self.assertIn("reason 0.4.2", installed or "")

    def test_checksum_mismatch_refuses_and_preserves_existing_binary(self) -> None:
        completed, installed = self.run_mock(correct_checksum=False, existing_binary=True)
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("checksum mismatch", completed.stderr)
        self.assertEqual(installed, "#!/bin/sh\necho old\n")

    def test_version_mismatch_refuses_and_preserves_existing_binary(self) -> None:
        completed, installed = self.run_mock(reported_version="9.9.9", existing_binary=True)
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("downloaded binary reported", completed.stderr)
        self.assertEqual(installed, "#!/bin/sh\necho old\n")

    def test_unsupported_platform_fails_before_install(self) -> None:
        completed, installed = self.run_mock(arch="arm64")
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("unsupported platform", completed.stderr)
        self.assertIsNone(installed)

    def test_installer_defaults_match_workspace_cli_version(self) -> None:
        cargo = tomllib.loads((ROOT / "crates/reasoning-harness-cli/Cargo.toml").read_text(encoding="utf-8"))
        expected = cargo["package"]["version"]
        shell_text = INSTALLER.read_text(encoding="utf-8")
        powershell_text = (ROOT / "install.ps1").read_text(encoding="utf-8")
        self.assertRegex(shell_text, rf'DEFAULT_VERSION="{re.escape(expected)}"')
        self.assertRegex(powershell_text, rf'\[string\]\$Version = "{re.escape(expected)}"')


if __name__ == "__main__":
    unittest.main()
