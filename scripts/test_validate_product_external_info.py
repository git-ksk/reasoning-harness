import hashlib
import importlib.util
import json
import pathlib
import shutil
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).with_name("validate_product_external_info.py")
spec = importlib.util.spec_from_file_location("validate_product_external_info", SCRIPT)
validator = importlib.util.module_from_spec(spec)
assert spec.loader
spec.loader.exec_module(validator)

REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
SOURCE = REPO_ROOT / "fixtures" / "product-external-info-v1"


def rewrite_manifest(root: pathlib.Path, manifest: pathlib.Path) -> None:
    lines = []
    for path in sorted(root.glob("*.json")):
        lines.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.name}")
    manifest.write_text("\n".join(lines) + "\n")


class ProductExternalInfoValidatorTests(unittest.TestCase):
    def copied_fixture(self):
        temp = tempfile.TemporaryDirectory()
        root = pathlib.Path(temp.name) / "product-external-info-v1"
        shutil.copytree(SOURCE, root)
        manifest = pathlib.Path(temp.name) / "product-external-info-v1.sha256"
        rewrite_manifest(root, manifest)
        return temp, root, manifest

    def test_frozen_corpus_validates(self):
        validator.validate_corpus(
            REPO_ROOT,
            SOURCE,
            REPO_ROOT / "fixtures" / "product-external-info-v1.sha256",
        )

    def test_resolver_facts_remain_forbidden_even_if_manifest_is_rehashed(self):
        temp, root, manifest = self.copied_fixture()
        self.addCleanup(temp.cleanup)
        path = root / "01_fresh-github-cli-full-name.json"
        payload = json.loads(path.read_text())
        payload["resolver_facts"] = {"external.github.cli.full_name": "cli/cli"}
        path.write_text(json.dumps(payload, indent=2) + "\n")
        rewrite_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "case keys must be exact|resolver_facts"):
            validator.validate_corpus(REPO_ROOT, root, manifest)

    def test_non_https_url_remains_forbidden_even_if_manifest_is_rehashed(self):
        temp, root, manifest = self.copied_fixture()
        self.addCleanup(temp.cleanup)
        path = root / "01_fresh-github-cli-full-name.json"
        payload = json.loads(path.read_text())
        payload["acquisition_profiles"][0]["fixed_arguments"]["url"] = (
            "http://api.github.com/repos/cli/cli"
        )
        path.write_text(json.dumps(payload, indent=2) + "\n")
        rewrite_manifest(root, manifest)
        with self.assertRaisesRegex(ValueError, "non-HTTPS"):
            validator.validate_corpus(REPO_ROOT, root, manifest)


if __name__ == "__main__":
    unittest.main()
