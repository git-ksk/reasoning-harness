"""Independent, no-network v3 private artifact and first-score ledger controls."""
import os
import stat
import tempfile
import unittest
from pathlib import Path

from contracts import (
    exclusive_durable_claim,
    exclusive_json,
    private_new_directory,
)


class PrivateArtifactTests(unittest.TestCase):
    def test_public_umask_cannot_expose_secret_sources_and_claims(self):
        for mask in (0o022, 0o077):
            with self.subTest(umask=oct(mask)), tempfile.TemporaryDirectory() as d:
                original = os.umask(mask)
                try:
                    root = Path(d)
                    output = root / "packets"
                    private_new_directory(output)
                    secret = output / "full-source.review-packet.json"
                    exclusive_json(secret, {"source_text": "synthetic private text"})
                    ledger = root / "private-ledger"
                    private_new_directory(ledger)
                    claim = ledger / "receipt.first-score.claim.json"
                    exclusive_durable_claim(claim, {"receipt": "synthetic"})
                    self.assertEqual(stat.S_IMODE(output.stat().st_mode), 0o700)
                    self.assertEqual(stat.S_IMODE(secret.stat().st_mode), 0o600)
                    self.assertEqual(stat.S_IMODE(ledger.stat().st_mode), 0o700)
                    self.assertEqual(stat.S_IMODE(claim.stat().st_mode), 0o600)
                    with self.assertRaises(FileExistsError):
                        exclusive_json(secret, {"source_text": "overwritten"})
                    with self.assertRaises(FileExistsError):
                        exclusive_durable_claim(claim, {"receipt": "replay"})
                finally:
                    os.umask(original)

    def test_untrusted_ledger_permissions_reject_claim_before_creation(self):
        for perms in (0o777, 0o755, 0o770):
            with self.subTest(mode=oct(perms)), tempfile.TemporaryDirectory() as d:
                ledger = Path(d) / "unsafe-ledger"
                ledger.mkdir(mode=0o700)
                ledger.chmod(perms)
                claim = ledger / "receipt.claim.json"
                with self.assertRaisesRegex(ValueError, "private_directory_owner_or_mode_unsafe"):
                    exclusive_durable_claim(claim, {"receipt": "synthetic"})
                self.assertFalse(claim.exists())

    def test_symlinked_ledger_and_target_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            ledger = root / "ledger"
            private_new_directory(ledger)
            redirect = root / "redirect"
            redirect.symlink_to(ledger, target_is_directory=True)
            with self.assertRaises(OSError):
                exclusive_durable_claim(redirect / "receipt.claim", {"receipt": "synthetic"})
            self.assertFalse((ledger / "receipt.claim").exists())
            target = ledger / "original"
            target.write_text("unchanged")
            (ledger / "receipt.claim").symlink_to(target)
            with self.assertRaises(FileExistsError):
                exclusive_durable_claim(ledger / "receipt.claim", {"receipt": "synthetic"})
            self.assertEqual(target.read_text(), "unchanged")

    def test_output_parent_and_private_json_fail_closed(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            parent = root / "untrusted"
            private_new_directory(parent)
            parent.chmod(0o777)
            with self.assertRaisesRegex(ValueError, "private_output_parent_untrusted"):
                private_new_directory(parent / "packets")
            self.assertFalse((parent / "packets").exists())
            with self.assertRaisesRegex(ValueError, "private_directory_owner_or_mode_unsafe"):
                exclusive_json(parent / "private.json", {"secret": "synthetic"})
            self.assertFalse((parent / "private.json").exists())
            parent.chmod(0o700)
            private_new_directory(parent / "packets")
            with self.assertRaises(FileExistsError):
                private_new_directory(parent / "packets")
            redirect = root / "redirect"
            redirect.symlink_to(parent, target_is_directory=True)
            with self.assertRaises(OSError):
                exclusive_json(redirect / "secret.json", {"secret": "synthetic"})
            self.assertFalse((parent / "secret.json").exists())


if __name__ == "__main__":
    unittest.main()
