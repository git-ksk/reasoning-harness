#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
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
        version: str,
        reported_version: str | None = None,
        correct_checksum: bool = True,
        os_name: str = "Linux",
        arch: str = "x86_64",
        gh_version: str = "2.93.0",
        attestation_success: bool = True,
    ) -> tuple[Path, Path, Path]:
        tools = base / "tools"
        release = base / "release"
        log = base / "gh.log"
        tools.mkdir()
        release.mkdir()

        write_executable(
            tools / "uname",
            f'#!/bin/sh\n[ "$1" = "-s" ] && echo {os_name} || echo {arch}\n',
        )
        package = base / f"reason-v{version}-linux-x86_64"
        package.mkdir()
        reported = reported_version or version
        write_executable(package / "reason", f'#!/bin/sh\necho "reason {reported}"\n')

        archive = release / f"reason-v{version}-linux-x86_64.tar.gz"
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
        verify_exit = "0" if attestation_success else "1"
        write_executable(
            tools / "gh",
            f"""#!/bin/sh
if [ "$1" = "--version" ]; then
  echo "gh version {gh_version} (mock)"
  exit 0
fi
printf 'GH_HOST=%s ARGS=%s\\n' "${{GH_HOST:-}}" "$*" >> "$MOCK_GH_LOG"
exit {verify_exit}
""",
        )
        return tools, release, log

    def run_mock(
        self,
        *,
        version: str = "0.4.2",
        reported_version: str | None = None,
        correct_checksum: bool = True,
        os_name: str = "Linux",
        arch: str = "x86_64",
        existing_binary: bool = False,
        gh_version: str = "2.93.0",
        attestation_success: bool = True,
    ) -> tuple[subprocess.CompletedProcess[str], str | None, str]:
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            bin_dir = base / "bin"
            bin_dir.mkdir()
            tools, release, gh_log = self.make_mock_release(
                base,
                version=version,
                reported_version=reported_version,
                correct_checksum=correct_checksum,
                os_name=os_name,
                arch=arch,
                gh_version=gh_version,
                attestation_success=attestation_success,
            )
            if existing_binary:
                write_executable(bin_dir / "reason", "#!/bin/sh\necho old\n")

            env = os.environ.copy()
            env["PATH"] = str(tools) + os.pathsep + env["PATH"]
            env["MOCK_RELEASE_DIR"] = str(release)
            env["MOCK_GH_LOG"] = str(gh_log)
            completed = subprocess.run(
                [str(INSTALLER), "--version", version, "--bin-dir", str(bin_dir)],
                cwd=ROOT,
                env=env,
                text=True,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
            destination = bin_dir / "reason"
            installed = destination.read_text(encoding="utf-8") if destination.exists() else None
            log_text = gh_log.read_text(encoding="utf-8") if gh_log.exists() else ""
            return completed, installed, log_text

    def test_historical_success_installs_checksum_verified_binary_without_attestation(self) -> None:
        completed, installed, log = self.run_mock()
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("reason 0.4.2", installed or "")
        self.assertEqual(log, "")
        self.assertIn("historical release", completed.stdout)

    def test_split_release_requires_exact_provenance_policy_then_checksum(self) -> None:
        completed, installed, log = self.run_mock(version="0.5.0")
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("reason 0.5.0", installed or "")
        self.assertIn("GH_HOST=github.com", log)
        self.assertIn("attestation verify", log)
        self.assertIn("--repo git-ksk/reasoning-harness", log)
        self.assertIn(
            "--signer-workflow git-ksk/reasoning-harness/.github/workflows/release-cli.yml", log
        )
        self.assertIn("--source-ref refs/tags/reason-v0.5.0", log)
        self.assertIn("--deny-self-hosted-runners", log)
        self.assertIn("GitHub/Sigstore provenance and SHA-256 verified", completed.stdout)

    def test_attestation_failure_refuses_and_preserves_existing_binary(self) -> None:
        completed, installed, _ = self.run_mock(
            version="0.5.0", attestation_success=False, existing_binary=True
        )
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("provenance verification failed", completed.stderr)
        self.assertEqual(installed, "#!/bin/sh\necho old\n")

    def test_outdated_gh_refuses_split_release_before_install(self) -> None:
        completed, installed, log = self.run_mock(
            version="0.5.0", gh_version="2.92.0", existing_binary=True
        )
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("too old", completed.stderr)
        self.assertEqual(installed, "#!/bin/sh\necho old\n")
        self.assertEqual(log, "")

    def test_checksum_mismatch_refuses_and_preserves_existing_binary(self) -> None:
        completed, installed, _ = self.run_mock(correct_checksum=False, existing_binary=True)
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("checksum mismatch", completed.stderr)
        self.assertEqual(installed, "#!/bin/sh\necho old\n")

    def test_version_mismatch_refuses_and_preserves_existing_binary(self) -> None:
        completed, installed, _ = self.run_mock(reported_version="9.9.9", existing_binary=True)
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("downloaded binary reported", completed.stderr)
        self.assertEqual(installed, "#!/bin/sh\necho old\n")

    def test_unsupported_platform_fails_before_install(self) -> None:
        completed, installed, _ = self.run_mock(arch="arm64")
        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("unsupported platform", completed.stderr)
        self.assertIsNone(installed)

    def test_installer_defaults_match_workspace_cli_version(self) -> None:
        cargo_text = (ROOT / "crates/reasoning-harness-cli/Cargo.toml").read_text(encoding="utf-8")
        match = re.search(r'^version\s*=\s*"([^"]+)"', cargo_text, re.MULTILINE)
        self.assertIsNotNone(match)
        expected = match.group(1)
        shell_text = INSTALLER.read_text(encoding="utf-8")
        powershell_text = (ROOT / "install.ps1").read_text(encoding="utf-8")
        self.assertRegex(shell_text, rf'DEFAULT_VERSION="{re.escape(expected)}"')
        self.assertRegex(powershell_text, rf'\[string\]\$Version = "{re.escape(expected)}"')


if __name__ == "__main__":
    unittest.main()
