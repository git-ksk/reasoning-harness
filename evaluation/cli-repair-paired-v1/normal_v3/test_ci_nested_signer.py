"""Mac Bash default-umask regression for authenticated final packet reads."""
import os
from pathlib import Path
import stat
import tempfile
import unittest

from ci_review_sign import _write_private_review_packet


class PrivateNestedPacketTests(unittest.TestCase):
    def test_nested_final_packet_private_under_0022_umask(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);os.chmod(root,0o700)
            old=os.umask(0o022)
            try:
                for name in ("observations/rw-01-baseline.json",
                             "observations/deep/rw-01-candidate.json",
                             "packets/packet-index.json"):
                    _write_private_review_packet(root,name,b"{}")
                for path in (root/"observations",root/"observations/deep",root/"packets"):
                    self.assertEqual(stat.S_IMODE(path.stat().st_mode),0o700)
                self.assertEqual((root/"observations/deep/rw-01-candidate.json").read_bytes(),b"{}")
            finally:
                os.umask(old)

    def test_symlink_parent_rejected_before_external_write(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder)/"private";root.mkdir(mode=0o700)
            target=Path(folder)/"outside";target.mkdir(mode=0o700)
            (root/"observations").symlink_to(target,target_is_directory=True)
            with self.assertRaisesRegex(ValueError,"symlink_parent"):
                _write_private_review_packet(root,"observations/foreign.json",b"foreign")
            self.assertEqual(list(target.iterdir()),[])


if __name__=="__main__":
    unittest.main()
