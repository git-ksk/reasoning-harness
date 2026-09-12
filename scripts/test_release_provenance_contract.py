#!/usr/bin/env python3
from __future__ import annotations

import hashlib, json, tempfile, unittest
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from generate_release_manifest import build_manifest  # noqa: E402


class ReleaseProvenanceContractTests(unittest.TestCase):
    def test_manifest_is_deterministic_and_binds_cli_engine_tag_commit_and_assets(self):
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            names = [
                "reason-v0.5.0-linux-x86_64.tar.gz",
                "reason-v0.5.0-macos-aarch64.tar.gz",
                "reason-v0.5.0-macos-x86_64.tar.gz",
                "reason-v0.5.0-windows-x86_64.zip",
                "SHA256SUMS", "install.sh", "install.ps1",
            ]
            for i, name in enumerate(names):
                (dist / name).write_bytes(f"asset-{i}".encode())
            manifest = build_manifest(dist, "reason-v0.5.0", "0.5.0", "0.4.2", "a" * 40)
            self.assertEqual(manifest["schema_version"], "reason-release-manifest-v1")
            self.assertEqual(manifest["repository"], "git-ksk/reasoning-harness")
            self.assertEqual(manifest["tag"], "reason-v0.5.0")
            self.assertEqual(manifest["cli_version"], "0.5.0")
            self.assertEqual(manifest["engine_version"], "0.4.2")
            self.assertEqual(manifest["git_commit"], "a" * 40)
            by_name = {x["name"]: x for x in manifest["artifacts"]}
            self.assertEqual(by_name[names[0]]["sha256"], hashlib.sha256(b"asset-0").hexdigest())

    def test_manifest_requires_exactly_four_native_archives(self):
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            for name in ["reason-v0.5.0-linux-x86_64.tar.gz", "SHA256SUMS", "install.sh", "install.ps1"]:
                (dist / name).write_text("x")
            with self.assertRaises(SystemExit):
                build_manifest(dist, "reason-v0.5.0", "0.5.0", "0.4.2", "b" * 40)

    def test_release_workflow_uses_keyless_oidc_attestation_for_split_releases(self):
        workflow = (ROOT / ".github/workflows/release-cli.yml").read_text(encoding="utf-8")
        self.assertGreaterEqual(workflow.count("uses: actions/attest@v4"), 2)
        self.assertGreaterEqual(workflow.count("id-token: write"), 2)
        self.assertGreaterEqual(workflow.count("attestations: write"), 2)
        self.assertIn("subject-path: reason-v${{ needs.prepare.outputs.version }}-${{ matrix.asset }}.${{ matrix.archive }}", workflow)
        self.assertIn("dist/release-manifest.json", workflow)
        self.assertIn("--engine-version \"${{ needs.prepare.outputs.engine_version }}\"", workflow)
        self.assertIn("--commit \"${{ needs.prepare.outputs.commit }}\"", workflow)
        self.assertIn("if: startsWith(needs.prepare.outputs.tag, 'reason-v')", workflow)
        self.assertIn("already exists and is immutable; refusing republish", workflow)

    def test_installers_pin_verifier_identity_and_minimum_safe_gh(self):
        shell = (ROOT / "install.sh").read_text(encoding="utf-8")
        powershell = (ROOT / "install.ps1").read_text(encoding="utf-8")
        for text in (shell, powershell):
            self.assertIn("2.93.0", text)
            self.assertIn("git-ksk/reasoning-harness", text)
            self.assertIn("git-ksk/reasoning-harness/.github/workflows/release-cli.yml", text)
            self.assertIn("--source-ref", text)
            self.assertIn("--deny-self-hosted-runners", text)
            self.assertIn("attestation verify", text)
        self.assertIn("GH_HOST=github.com", shell)
        self.assertIn("$env:GH_HOST = 'github.com'", powershell)


if __name__ == "__main__":
    unittest.main()
