"""Independent adversarial tests for #579: offline synthetic artifacts only."""
import os
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from contracts import (
    _open_trusted_directory, exclusive_durable_claim, exclusive_json,
    private_new_directory,
)


class IndependentPrivateFilesystemTests(unittest.TestCase):
    def test_middle_symlink_cannot_redirect_output_or_claim(self):
        with tempfile.TemporaryDirectory() as t:
            root = Path(t).resolve()
            real = root / "real"
            (real / "nested").mkdir(parents=True)
            alias = root / "alias"
            alias.symlink_to(real, target_is_directory=True)
            for action in (
                lambda: private_new_directory(alias / "nested" / "new-output"),
                lambda: exclusive_json(alias / "nested" / "secret.json", {"secret": "synthetic"}),
                lambda: exclusive_durable_claim(alias / "nested" / "receipt.claim", {"receipt": "synthetic"}),
            ):
                with self.subTest(action=str(action)), self.assertRaises((OSError, ValueError)):
                    action()
            self.assertEqual(list((real / "nested").iterdir()), [])

    @unittest.skipUnless(sys.platform == "darwin", "macOS ACL API test")
    def test_acl_allow_cannot_bypass_0700_directory(self):
        with tempfile.TemporaryDirectory() as t:
            root = Path(t).resolve()
            ledger = root / "ledger"
            ledger.mkdir(mode=0o700)
            try:
                subprocess.run(["/bin/chmod", "+a", "everyone allow read,search", str(ledger)], check=True)
                self.assertEqual(stat.S_IMODE(ledger.stat().st_mode), 0o700)
                with self.assertRaisesRegex(ValueError, "acl"):
                    exclusive_durable_claim(ledger / "receipt.claim", {"receipt": "synthetic"})
                self.assertFalse((ledger / "receipt.claim").exists())
            finally:
                subprocess.run(["/bin/chmod", "-N", str(ledger)], check=True)

    @unittest.skipUnless(sys.platform == "darwin", "macOS ACL API test")
    def test_restrictive_deny_acl_remains_compatible(self):
        with tempfile.TemporaryDirectory() as t:
            root = Path(t).resolve()
            ledger = root / "ledger"
            private_new_directory(ledger)
            try:
                subprocess.run(["/bin/chmod", "+a", "everyone deny delete", str(ledger)], check=True)
                exclusive_json(ledger / "nonsecret.json", {"test": "synthetic"})
                self.assertTrue((ledger / "nonsecret.json").is_file())
            finally:
                subprocess.run(["/bin/chmod", "-N", str(ledger)], check=True)

    def test_untrusted_middle_ancestor_rejected(self):
        with tempfile.TemporaryDirectory() as t:
            root = Path(t).resolve()
            unsafe = root / "unsafe"
            nested = unsafe / "private"
            nested.mkdir(parents=True, mode=0o700)
            unsafe.chmod(0o777)
            try:
                with self.assertRaisesRegex(ValueError, "private_output_parent_untrusted"):
                    exclusive_durable_claim(nested / "claim", {"receipt": "synthetic"})
                with self.assertRaisesRegex(ValueError, "private_output_parent_untrusted"):
                    private_new_directory(nested / "output")
            finally:
                unsafe.chmod(0o700)
            self.assertEqual(list(nested.iterdir()), [])

    def test_root_sticky_parent_allows_private_directory(self):
        # Native /private/tmp on macOS (not the /tmp symlink), /tmp on Linux.
        import uuid
        root = Path("/private/tmp" if sys.platform == "darwin" else "/tmp")
        output = root / ("rh585-synthetic-" + uuid.uuid4().hex)
        try:
            private_new_directory(output)
            exclusive_json(output / "data.json", {"value": "synthetic"})
            self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o700)
            self.assertEqual(stat.S_IMODE((output / "data.json").stat().st_mode), 0o600)
        finally:
            if (output / "data.json").exists():
                (output / "data.json").unlink()
            if output.exists():
                output.rmdir()

    def test_partial_json_failure_does_not_overwrite_initial_artifact(self):
        with tempfile.TemporaryDirectory() as t:
            output = Path(t).resolve() / "output"
            private_new_directory(output)
            target = output / "partial.json"
            with self.assertRaises(TypeError):
                exclusive_json(target, {"unserializable": object()})
            self.assertTrue(target.exists())
            with self.assertRaises(FileExistsError):
                exclusive_json(target, {"retry": "synthetic"})

    def test_claim_fsync_fail_does_not_report_success_or_overwrite(self):
        with tempfile.TemporaryDirectory() as t:
            ledger = Path(t).resolve() / "ledger"
            private_new_directory(ledger)
            target = ledger / "claim.json"
            with mock.patch("os.fsync", side_effect=OSError("simulated fsync failure")):
                with self.assertRaises(OSError):
                    exclusive_durable_claim(target, {"receipt": "synthetic"})
            self.assertTrue(target.exists())
            with self.assertRaises(FileExistsError):
                exclusive_durable_claim(target, {"receipt": "retry"})

    def test_umask_000_creates_private_bytes(self):
        with tempfile.TemporaryDirectory() as t:
            previous = os.umask(0)
            try:
                output = Path(t).resolve() / "result"
                private_new_directory(output)
                exclusive_json(output / "source.json", {"secret": "synthetic"})
                self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o700)
                self.assertEqual(stat.S_IMODE((output / "source.json").stat().st_mode), 0o600)
            finally:
                os.umask(previous)


if __name__ == "__main__":
    unittest.main()
