import copy
import unittest

from display_binding import bundle_sha256, score_bound_case, validate_display_binding
from test_claim_metrics import fixture as metrics_fixture


def fixture():
    artifacts = [{"id": "answer", "kind": "final_answer", "text": "発売済み。\n"}]
    claims = [{"id": "claim1", "proposition_id": "product.launched"}]
    segments = [dict(id="s1", artifact_id="answer", start_byte=0, end_byte=15,
                     text="発売済み。", kind="fact", claim_id="claim1", proposition_id="product.launched"),
                dict(id="s2", artifact_id="answer", start_byte=15, end_byte=16,
                     text="\n", kind="nonfact", reason="formatting")]
    return artifacts, ["answer"], bundle_sha256(artifacts), segments, claims


class DisplayBinding(unittest.TestCase):
    def test_utf8_full_coverage(self):
        s = validate_display_binding(*fixture())
        self.assertEqual(s["declared_byte_coverage"], "COMPLETE")
        self.assertEqual(s["trace_and_support"], "NOT_VERIFIED")

    def test_missing_tail_is_not_full_coverage(self):
        args = list(fixture()); args[3].pop()
        with self.assertRaisesRegex(ValueError, "gap_or_overlap"): validate_display_binding(*args)

    def test_overlap_fails(self):
        args = list(fixture()); args[3][1].update(start_byte=12, text="。\n")
        with self.assertRaisesRegex(ValueError, "gap_or_overlap"): validate_display_binding(*args)

    def test_utf8_boundary_is_in_bytes_not_characters(self):
        args = list(fixture()); args[3][0]["end_byte"] = 14
        with self.assertRaisesRegex(ValueError, "splits_utf8"): validate_display_binding(*args)

    def test_swapped_or_changed_display_bytes_fail(self):
        args = list(fixture()); args[0][0]["text"] = "未発売。\n"
        with self.assertRaisesRegex(ValueError, "bundle_mismatch"): validate_display_binding(*args)

    def test_missing_caption_artifact_cannot_be_silently_omitted(self):
        args = list(fixture()); args[1].append("caption")
        with self.assertRaisesRegex(ValueError, "missing_or_extra"): validate_display_binding(*args)

    def test_duplicate_artifact_or_segment_id_fails(self):
        for index in (0, 3):
            args = list(fixture()); args[index].append(copy.deepcopy(args[index][0]))
            with self.assertRaisesRegex(ValueError, "duplicate_"): validate_display_binding(*args)

    def test_claim_without_span_fails(self):
        args = list(fixture()); args[4].append({"id": "hidden", "proposition_id": "other"})
        with self.assertRaisesRegex(ValueError, "without_display_span"): validate_display_binding(*args)

    def test_swapped_claim_proposition_fails(self):
        args = list(fixture()); args[3][0]["proposition_id"] = "not-product.launched"
        with self.assertRaisesRegex(ValueError, "misbound_fact"): validate_display_binding(*args)

    def test_empty_artifact_needs_no_span(self):
        a = [{"id": "answer", "kind": "final_answer", "text": ""}]
        self.assertEqual(validate_display_binding(a, ["answer"], bundle_sha256(a), [], [])["segments"], 0)

    def test_nonfact_reason_required_and_cannot_bind_claim(self):
        for changes in ({"reason": ""}, {"claim_id": "claim1"}):
            args = list(fixture()); args[3][1].update(changes)
            with self.assertRaises(ValueError): validate_display_binding(*args)

    def test_bool_span_and_stale_annotation_text_fail(self):
        for changes in ({"start_byte": False}, {"text": "発売"}):
            args = list(fixture()); args[3][0].update(changes)
            with self.assertRaises(ValueError): validate_display_binding(*args)

    def test_binding_and_arithmetic_join_uses_same_claim_occurrence(self):
        artifacts, ids, sha, segments, _ = fixture()
        p, r = metrics_fixture()
        p["observation_sha256"] = r["observation_sha256"] = sha
        r["claims"][0].update(id="claim1", proposition_id="product.launched")
        result = score_bound_case(p, r, artifacts, ids, segments)
        self.assertEqual(result["claim_metrics"]["f1"], 1)
        self.assertEqual(result["acceptance"], "NOT_EVALUATED")

    def test_valid_partial_display_does_not_create_complete_f1(self):
        artifacts, ids, sha, segments, _ = fixture()
        p, r = metrics_fixture()
        p["observation_sha256"] = r["observation_sha256"] = sha
        r["claims"][0].update(id="claim1", proposition_id="product.launched")
        r["operation"] = "incomplete"
        result = score_bound_case(p, r, artifacts, ids, segments)
        self.assertIsNone(result["claim_metrics"]["f1"])


if __name__ == "__main__":
    unittest.main()
