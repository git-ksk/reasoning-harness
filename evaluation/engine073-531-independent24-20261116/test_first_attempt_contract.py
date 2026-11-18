import copy
import hashlib
import unittest
from first_attempt_contract import check_first_attempts


class FirstAttemptContractTests(unittest.TestCase):
    def setUp(self):
        self.ids = ["synthetic-%02d" % i for i in range(24)]
        self.rows = []
        for c in self.ids:
            for a in ("baseline", "candidate"):
                digest = hashlib.sha256((c + a).encode()).hexdigest()
                self.rows.append(dict(case_id=c, arm=a, attempt=1, status="complete",
                                      admission_sha256=digest, final_sha256="a" * 64))

    def test_complete_is_not_authority(self):
        r = check_first_attempts(self.ids, self.rows)
        self.assertTrue(r["structurally_complete"])
        self.assertFalse(r["formal_score_authorized"])
        self.assertFalse(r["live_authorized"])

    def test_incomplete_or_unknown_blocks(self):
        for status in ("timeout", "unknown", "cancelled", "incomplete", "ambiguous", None):
            with self.subTest(status=status):
                rows = copy.deepcopy(self.rows)
                rows[0]["status"] = status
                self.assertFalse(check_first_attempts(self.ids, rows)["structurally_complete"])

    def test_second_attempt_cannot_replace_first(self):
        for attempt in (2, 0, True, "1"):
            rows = copy.deepcopy(self.rows)
            rows[0]["attempt"] = attempt
            self.assertFalse(check_first_attempts(self.ids, rows)["structurally_complete"])

    def test_duplicate_or_missing_slot_blocks(self):
        rows = copy.deepcopy(self.rows)
        rows[-1] = rows[0]
        self.assertFalse(check_first_attempts(self.ids, rows)["structurally_complete"])
        self.assertFalse(check_first_attempts(self.ids, rows[:-1])["structurally_complete"])

    def test_unknown_slot_and_bad_digest(self):
        for key, value in (("case_id", "unknown"), ("arm", "other"),
                           ("admission_sha256", ""), ("final_sha256", None),
                           ("case_id", [])):
            rows = copy.deepcopy(self.rows)
            rows[0][key] = value
            self.assertFalse(check_first_attempts(self.ids, rows)["structurally_complete"])

    def test_duplicate_admission_blocks(self):
        rows = copy.deepcopy(self.rows)
        rows[1]["admission_sha256"] = rows[0]["admission_sha256"]
        self.assertFalse(check_first_attempts(self.ids, rows)["structurally_complete"])

    def test_no_payload_fields_allowed(self):
        rows = copy.deepcopy(self.rows)
        rows[0]["extra"] = "synthetic"
        self.assertFalse(check_first_attempts(self.ids, rows)["structurally_complete"])

    def test_invalid_case_inventory(self):
        self.assertFalse(check_first_attempts(self.ids[:-1], self.rows)["structurally_complete"])
        self.assertFalse(check_first_attempts(self.ids[:-1] + self.ids[:1], self.rows)["structurally_complete"])


if __name__ == "__main__":
    unittest.main()
