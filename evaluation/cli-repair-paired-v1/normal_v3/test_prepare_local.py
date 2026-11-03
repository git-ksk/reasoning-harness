"""No provider calls: real-corpus intake boundaries and captured data fidelity."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from contracts import canonical, digest
from native_admission import CATALOG, source_catalog
from native_lane import private_bytes
import prepare_local


def corpus():
    source = {"quote": "A complete quotation.", "context_summary": "Captured context", "document_id": "doc",
        "url": "https://example.test/pinned", "locator": "line 1", "source_version": "sha256:" + "b" * 64,
        "origin_group": "one-origin", "scope": {"product": "unit"}, "synthetic_mutation": False,
        "retrieved_at_unix_seconds": 1000, "reference_facts": {"unit.value": "4"}}
    case = {"id": "rw-01", "question": "What is the value?", "source_ids": ["s1"],
            "lookup_key": "unit.value", "required_scope": {"product": "unit"}}
    return {"documents": {"doc": {"retrieved_at_unix_seconds": 1000}}, "sources": {"s1": source}, "cases": [case]}


class PreparationTests(unittest.TestCase):
    def test_process_free_data_matches_previous_snapshot_resolver_exactly(self):
        data = corpus(); case = data["cases"][0]
        legacy_path = Path(__file__).resolve().parent.parent / "resolver.py"
        spec = importlib.util.spec_from_file_location("legacy_snapshot_resolver", legacy_path)
        legacy = importlib.util.module_from_spec(spec); spec.loader.exec_module(legacy)
        config, evidence = prepare_local.captured_case(data, case)
        capability = config["resolution"]["investigation"]["capabilities"][0]
        self.assertEqual(capability["evidence"], legacy.records(data, case))
        self.assertEqual(evidence[0][0]["facts"], capability["evidence"][0]["facts"])
        self.assertNotIn("program", capability)
        poisoned = copy.deepcopy(data); poisoned["cases"][0]["label"] = "opposite-gold"
        self.assertEqual(prepare_local.captured_case(poisoned, poisoned["cases"][0]), (config, evidence))

    def test_prepared_registry_is_bound_to_exact_native_struct_bytes(self):
        data = corpus(); config, snapshots = prepare_local.captured_case(data, data["cases"][0])
        item, hash_ = snapshots[0]
        self.assertEqual(hash_, digest(json.dumps(item, ensure_ascii=False, separators=(",", ":")).encode()))
        self.assertEqual(config["answer_context"]["sources"][0]["evidence_sha256"], hash_)
        catalog = {"contract": CATALOG, "sources": {"s1": {"text": "A complete quotation.", "source_version": "v1"}},
            "evidence_bindings": {item["id"]: {"source_id": "s1", "source_version": "v1", "evidence_sha256": hash_}}}
        self.assertIn("snapshot:rw-01:0", source_catalog(canonical(catalog))["evidence_bindings"])
        for bad in ("", "bad\nid", "x" * 513):
            catalog["evidence_bindings"] = {bad: next(iter(catalog["evidence_bindings"].values()))}
            with self.assertRaisesRegex(ValueError, "evidence_id"): source_catalog(canonical(catalog))

    def test_manifest_cannot_hide_extra_or_changed_material_or_escape(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            private_bytes(root / "corpus.json", b"{}")
            manifest = {"files": {"corpus.json": digest(b"{}")}}
            private_bytes(root / "manifest.json", canonical(manifest))
            with patch.object(prepare_local, "INPUT_M1", digest(canonical(manifest))):
                private_bytes(root / "extra.json", b"extra")
                with self.assertRaisesRegex(ValueError, "inventory_changed"): prepare_local.checked_input(root)
                (root / "extra.json").unlink()
                (root / "corpus.json").write_bytes(b"different")
                with self.assertRaisesRegex(ValueError, "bytes_changed"): prepare_local.checked_input(root)
            with self.assertRaisesRegex(ValueError, "manifest_changed"): prepare_local.checked_input(root)

    def test_unreviewed_host_fact_extension_is_not_silently_admitted(self):
        data = corpus(); data["cases"][0]["e2e_host_facts"] = {"invented": "true"}
        with self.assertRaisesRegex(ValueError, "requires_review"):
            prepare_local.captured_case(data, data["cases"][0])


if __name__ == "__main__": unittest.main()
