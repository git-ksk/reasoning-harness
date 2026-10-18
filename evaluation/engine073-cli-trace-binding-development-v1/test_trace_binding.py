import copy
import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest

from trace_binding import load_saved_capture, sha, strict_json, validate_capture, verify_full_snapshot

REPO = Path(os.environ.get("ENGINE073_TRACE_TEST_REPO", Path(__file__).resolve().parents[2]))
CAPTURE = REPO / "evaluation/engine07-cli-path-preparation-v3"
CORPUS = REPO / "evaluation/engine07-realworld-v3/corpus.json"


def fixture():
    r = CAPTURE / "run-candidate"
    corpus_bytes = CORPUS.read_bytes()
    return [json.loads((r / "stdout.json").read_text()), json.loads((r / "trace.json").read_text()),
            [json.loads(line) for line in (r / "resolver.jsonl").read_text().splitlines()],
            json.loads((r / "exit.json").read_text()), json.loads((r / "config.json").read_text()),
            json.loads(corpus_bytes), sha(corpus_bytes)]


def sync_artifact(args):
    artifact = args[0]["result"]["final_outcome"]["artifact"]
    for event in args[1]["events"]:
        if event["kind"] == "grounding_state" and event["phase"] == "pre_final_render_artifact":
            event["outcome"]["artifact"] = copy.deepcopy(artifact)


def snapshot_fixture():
    raw = b"<html><style>ignore</style><p>Alpha is <b>ready</b>.</p><script>ignore</script></html>"
    normalized = b"Alpha is ready ."
    source = dict(document_id="doc", url="https://example.invalid/doc", source_version="sha256:" + sha(raw),
                  normalized_byte_start=0, normalized_byte_end=len(normalized), quote=normalized.decode(), quote_sha256=sha(normalized))
    doc = dict(document_id="doc", url=source["url"], source_version=source["source_version"],
               normalization="htmlparser-skip-script-style-join-data-whitespace-v1", raw_sha256=sha(raw), normalized_sha256=sha(normalized))
    return source, doc, raw, normalized


class CaptureBinding(unittest.TestCase):
    def test_saved_pair_joins_display_to_sources_without_semantic_score(self):
        anchor = sha((CAPTURE / "artifact-sha256.json").read_bytes())
        for arm in ["baseline", "candidate"]:
            out = load_saved_capture(CAPTURE, arm, CORPUS, anchor)
            self.assertEqual(out["binding_status"], "STRUCTURAL_MATCH")
            self.assertEqual(len(out["display_claims"]), 1)
            self.assertEqual(len(out["display_claims"][0]["internal_bindings"]), 2)
            self.assertEqual(out["displayed_citations"]["count"], 0)
            self.assertIsNone(out["actual_answer_f1"])
            self.assertEqual(out["acceptance"], "NOT_EVALUATED")

    def test_actual_display_is_not_the_unexposed_renderer_text(self):
        args = fixture()
        renderer = next(e["candidate"]["text"] for e in args[1]["events"] if e["kind"] == "final_render_state")
        out = validate_capture(*args)
        self.assertNotEqual(out["display_artifacts"][0]["text"], renderer)
        self.assertEqual(out["display_artifacts"][0]["text"], "node.buffer.uint8array = true")

    def test_bool_returncode_and_failure_do_not_become_abstentions(self):
        for rc in [True, 1, None]:
            args = fixture(); args[3]["returncode"] = rc
            with self.assertRaisesRegex(ValueError, "process_not_successful"): validate_capture(*args)

    def test_model_response_without_request_rejected(self):
        args = fixture(); args[1]["events"].pop(0)
        for n, e in enumerate(args[1]["events"]): e["sequence"] = n
        with self.assertRaisesRegex(ValueError, "before_request"): validate_capture(*args)

    def test_model_correlation_phase_and_sequence_must_match(self):
        for key, value in [("correlation_id", "foreign"), ("phase", "wrong"), ("sequence", True)]:
            args = fixture(); args[1]["events"][1][key] = value
            with self.assertRaises(ValueError): validate_capture(*args)

    def test_missing_model_terminal_is_incomplete(self):
        args = fixture(); args[1]["events"].pop(1)
        for n, e in enumerate(args[1]["events"]): e["sequence"] = n
        with self.assertRaisesRegex(ValueError, "incomplete_model_trace"): validate_capture(*args)

    def test_accepted_output_and_last_trace_display_must_be_equal(self):
        args = fixture(); args[0]["result"]["finalization"]["text"] = "invented"
        with self.assertRaisesRegex(ValueError, "terminal_display_mismatch"): validate_capture(*args)

    def test_fact_swap_even_in_both_output_and_trace_is_caught_by_acquisition(self):
        args = fixture(); args[0]["result"]["final_outcome"]["artifact"]["evidence"][0]["facts"] = {"forged": "true"}; sync_artifact(args)
        with self.assertRaisesRegex(ValueError, "admitted_evidence_changed"): validate_capture(*args)

    def test_admission_requires_same_acquisition_target(self):
        args = fixture(); args[2][0]["request"]["request"]["target"]["target_id"] = "other"
        with self.assertRaisesRegex(ValueError, "acquisition_wrong_target"): validate_capture(*args)

    def test_question_and_scope_must_match_the_captured_task(self):
        args = fixture(); args[0]["result"]["task"] = "a different question"
        with self.assertRaisesRegex(ValueError, "question_mismatch"): validate_capture(*args)
        args = fixture()
        for cap in args[4]["resolution"]["investigation"]["capabilities"]:
            cap["admission"]["required_scope"] = {"product": {"kind": "values", "values": ["other"]}}
        with self.assertRaisesRegex(ValueError, "task_admission_scope_mismatch"): validate_capture(*args)

    def test_reused_journal_request_identity_is_ambiguous(self):
        args = fixture(); args[2].append(copy.deepcopy(args[2][0]))
        with self.assertRaisesRegex(ValueError, "ambiguous_resolver"): validate_capture(*args)

    def test_source_quote_and_corpus_binding_cannot_be_substituted(self):
        args = fixture(); args[2][0]["corpus_sha256"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "corpus_mismatch"): validate_capture(*args)
        args = fixture(); args[5]["sources"]["node-subclass"]["quote"] = "forged"
        with self.assertRaisesRegex(ValueError, "source_binding_mismatch"): validate_capture(*args)

    def test_receipt_is_needed_even_for_a_claim_marked_supported(self):
        args = fixture(); args[0]["result"]["final_outcome"]["artifact"]["verification_receipts"] = []; sync_artifact(args)
        with self.assertRaisesRegex(ValueError, "supported_claim_without_receipt"): validate_capture(*args)

    def test_receipt_cannot_bind_a_different_proposition(self):
        args = fixture(); args[0]["result"]["final_outcome"]["artifact"]["verification_receipts"][0]["proposition"]["value"] = "false"; sync_artifact(args)
        with self.assertRaisesRegex(ValueError, "misbound_receipt"): validate_capture(*args)

    def test_orphan_receipt_and_extra_unadmitted_claim_evidence_fail(self):
        args = fixture(); args[0]["result"]["final_outcome"]["artifact"]["verification_receipts"][0]["claim_id"] = "foreign"; sync_artifact(args)
        with self.assertRaisesRegex(ValueError, "orphan_or_misbound_receipt"): validate_capture(*args)
        args = fixture(); args[0]["result"]["final_outcome"]["artifact"]["claims"][0]["evidence_ids"].append("foreign"); sync_artifact(args)
        with self.assertRaisesRegex(ValueError, "claim_has_unadmitted_evidence"): validate_capture(*args)

    def test_host_authority_and_effective_time_are_required(self):
        args = fixture()
        for cap in args[4]["resolution"]["investigation"]["capabilities"]:
            cap["admission"]["sources"]["node-subclass"]["authority_class"] = "untrusted"
        with self.assertRaisesRegex(ValueError, "authority_below_minimum"): validate_capture(*args)
        args = fixture()
        for cap in args[4]["resolution"]["investigation"]["capabilities"]:
            cap["admission"]["evaluation_time_unix_seconds"] += 1000000
        with self.assertRaisesRegex(ValueError, "not_effective"): validate_capture(*args)

    def test_duplicate_json_keys_are_rejected(self):
        with self.assertRaises(ValueError): strict_json('{"id":1,"id":2}')
        with self.assertRaises(ValueError): strict_json('{"value":NaN}')

    def test_hash_anchor_and_changed_capture_rejected_before_parse(self):
        with self.assertRaisesRegex(ValueError, "anchor_mismatch"):
            load_saved_capture(CAPTURE, "candidate", CORPUS, "0" * 64)
        with tempfile.TemporaryDirectory(prefix="engine073-trace-hash-") as tmp:
            root = Path(tmp); (root / "run-candidate").mkdir()
            (root / "artifact-sha256.json").write_bytes((CAPTURE / "artifact-sha256.json").read_bytes())
            (root / "run-candidate/stdout.json").write_text('{}')
            anchor = sha((root / "artifact-sha256.json").read_bytes())
            with self.assertRaisesRegex(ValueError, "capture_file_hash_mismatch"):
                load_saved_capture(root, "candidate", CORPUS, anchor)


class FullSnapshotBinding(unittest.TestCase):
    def test_raw_normalization_and_exact_quote_chain(self):
        self.assertEqual(verify_full_snapshot(*snapshot_fixture())["status"], "EXACT_CAPTURED_BYTES")

    def test_raw_and_normalized_snapshot_changes_fail(self):
        for index in [2, 3]:
            args = list(snapshot_fixture()); args[index] += b" changed"
            with self.assertRaises(ValueError): verify_full_snapshot(*args)

    def test_wrong_locator_version_and_quote_are_rejected(self):
        for key, value in [("normalized_byte_start", 1), ("source_version", "sha256:wrong"), ("quote", "forged")]:
            args = list(snapshot_fixture()); args[0][key] = value
            with self.assertRaises(ValueError): verify_full_snapshot(*args)


if __name__ == "__main__":
    unittest.main()
