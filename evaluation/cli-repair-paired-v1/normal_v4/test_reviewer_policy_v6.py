"""Independent frozen review-policy provenance and negative controls."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from reviewer_policy_v6 import POLICY_PATH, load_enrolled_v6_rubric


class V6PolicyTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)
        self.verified=(Path(__file__).parent / "reviewer_safety_policy_v6.txt").read_bytes()
        data={"contract":"reason-cli-normal-v3-native-bundle-v2","files":{}}
        for arm in ("baseline","candidate"):
            key="source/"+arm+"/"+POLICY_PATH
            target=self.root/"bundle-files"/key
            target.parent.mkdir(parents=True,exist_ok=True)
            target.write_bytes(self.verified)
            data["files"][key]={"sha256":hashlib.sha256(self.verified).hexdigest()}
        self.bundle=data
        self.save()

    def save(self):
        (self.root/"native-bundle.json").write_bytes(
            json.dumps(self.bundle,sort_keys=True,separators=(",",":"),
                       ensure_ascii=False).encode())

    def test_real_policy_both_arms_and_manifest_match(self):
        self.assertEqual(load_enrolled_v6_rubric(self.root).encode(),self.verified)
        self.assertIn("NOT an independent safety-review",self.verified.decode())

    def test_policy_must_be_present_byte_equal_and_hashed_for_both_arms(self):
        key="source/baseline/"+POLICY_PATH
        p=self.root/"bundle-files"/key
        p.write_bytes(self.verified.replace(b"Return SAFE",b"Return RUDE"))
        with self.assertRaisesRegex(ValueError,"not_native_bundle_enrolled"):
            load_enrolled_v6_rubric(self.root)
        p.write_bytes(self.verified)
        p.unlink()
        with self.assertRaisesRegex(ValueError,"missing_or_symlink"):
            load_enrolled_v6_rubric(self.root)

    def test_policy_manifest_removal_and_noncanonical_fail(self):
        self.bundle["files"].pop("source/candidate/"+POLICY_PATH)
        self.save()
        with self.assertRaisesRegex(ValueError,"not_native_bundle_enrolled"):
            load_enrolled_v6_rubric(self.root)
        self.bundle["files"]["source/candidate/"+POLICY_PATH]={
            "sha256":hashlib.sha256(self.verified).hexdigest()}
        self.save()
        (self.root/"native-bundle.json").write_text(
            json.dumps(self.bundle,indent=2))
        with self.assertRaisesRegex(ValueError,"noncanonical"):
            load_enrolled_v6_rubric(self.root)


if __name__ == "__main__":
    unittest.main()
