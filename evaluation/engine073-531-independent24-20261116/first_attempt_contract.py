"""Offline structural check of fresh holdout first-attempt metadata.
No Gold, answers, scoring, provider access or execution authorization.
Receipt authenticity and immutable admission journal binding are external gates.
"""
import re

ARMS = ("baseline", "candidate")
COMPLETE = "complete"
FIELDS = frozenset(("case_id", "arm", "attempt", "status", "admission_sha256", "final_sha256"))
HEX = re.compile(r"[0-9a-f]{64}\Z")
CASE = re.compile(r"[A-Za-z0-9][A-Za-z0-9_-]{0,63}\Z")


def check_first_attempts(case_ids, rows):
    """Return a structural-only HOLD/complete result; no permission to score/live."""
    def result(ok, reason):
        return {"structurally_complete": ok, "reason": reason,
                "authenticity_status": "UNVERIFIED", "approval_status": "HOLD",
                "formal_score_authorized": False, "live_authorized": False}

    if not isinstance(case_ids, list) or len(case_ids) != 24 or any(
        not isinstance(c, str) or CASE.fullmatch(c) is None for c in case_ids
    ) or len(set(case_ids)) != 24:
        return result(False, "invalid_case_inventory")
    if not isinstance(rows, list) or len(rows) != 48:
        return result(False, "missing_or_extra_observations")
    expected = {(c, a) for c in case_ids for a in ARMS}
    seen = set()
    admission_ids = set()
    for row in rows:
        if not isinstance(row, dict) or set(row) != FIELDS:
            return result(False, "invalid_observation_schema")
        c, a = row["case_id"], row["arm"]
        if not isinstance(c, str) or not isinstance(a, str):
            return result(False, "invalid_slot")
        slot = c, a
        if slot not in expected or slot in seen:
            return result(False, "unknown_or_duplicate_slot")
        if type(row["attempt"]) is not int or row["attempt"] != 1:
            return result(False, "not_first_attempt")
        if row["status"] != COMPLETE:
            return result(False, "first_attempt_not_complete")
        for k in ("admission_sha256", "final_sha256"):
            if not isinstance(row[k], str) or HEX.fullmatch(row[k]) is None:
                return result(False, "invalid_digest")
        if row["admission_sha256"] in admission_ids:
            return result(False, "duplicate_admission")
        admission_ids.add(row["admission_sha256"])
        seen.add(slot)
    if seen != expected:
        return result(False, "missing_slot")
    return result(True, "complete_metadata_only")
