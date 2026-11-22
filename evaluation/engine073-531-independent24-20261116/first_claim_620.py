"""Single-coordinator #620 synthetic/offline first-claim journal.

Reuses LocalAtomicStore append-only fsync, root identity and exclusive creation.
No dispatch API, no resume, no external authority and no formal scoring.
The trusted operator pins the store profile and dataset identity separately.
"""
from collections import Counter
import threading

from auth620 import encode, decode, sha, fields, match, SHA, ID, require
from authority import PREFIX
from local_store import LocalAtomicStore
from first_attempt_contract import ARMS, CASE, FIELDS, check_first_attempts

FIXED_IDENTITY = {"baseline_core": "4abee90", "candidate_core": "7466c6f",
                  "cli": "b85a4a0", "provider": "Mistral", "model": "ministral-8b-2512"}
FIXED_CONDITIONS = {**FIXED_IDENTITY, "calls_per_cli": 12, "tokens_per_cli": 28096,
                    "shards": 8, "cases": 24, "observations": 48,
                    "calls_total": 576, "tokens_total": 1348608}
STATUSES = {"complete", "timeout", "cancelled", "incomplete", "unknown", "ambiguous", "error"}


def inventory_slots(raw):
    items = decode(raw)
    require(type(items) is list and len(items) == 24, "inventory_count")
    for x in items:
        fields(x, {"case_id", "case_type", "fact_count"})
        match(x["case_id"], CASE, "case_id")
        require(type(x["case_type"]) is str and x["case_type"] in ("positive", "negative")
                and type(x["fact_count"]) is int and x["fact_count"] > 0, "inventory_case")
    require(len({x["case_id"] for x in items}) == 24
            and Counter(x["case_type"] for x in items) == {"positive": 16, "negative": 8},
            "inventory_balance")
    return tuple((i // 3 + 1, x["case_id"], a) for i, x in enumerate(items) for a in ARMS)


def cohort_key(dataset_sha256):
    """No eval ID, output path, shard, mutable budget or nonce in this identity."""
    match(dataset_sha256, SHA, "dataset_pin")
    return sha(encode({"contract": "620-cohort/1", "dataset_sha256": dataset_sha256,
                       "identity": FIXED_IDENTITY}))


class FirstClaimLedger:
    """Offline coordinator, trusted single host/OS account; never reopen to resume.

    A partially written claim stays burned. The record root is shared across
    evaluation IDs. External antirollback and real run approval are absent.
    """
    def __init__(self, profile, *, dataset_sha256, inventory_raw, inventory_receipt,
                 conditions_receipt, evaluation_id, blind_map_sha256):
        match(evaluation_id, ID, "evaluation_id")
        # These are local bindings, not authentication or approval. Formal entry
        # must be a distinct service and is deliberately disabled.
        match(inventory_receipt, SHA, "inventory_receipt")
        match(conditions_receipt, SHA, "conditions_receipt")
        match(blind_map_sha256, SHA, "blind_map_sha256")
        self.blind_map_sha256 = blind_map_sha256
        self._slots = inventory_slots(inventory_raw)
        self._store = LocalAtomicStore(profile, role="executor")
        self._root = PREFIX + "/cohorts/issue620-synthetic/" + cohort_key(dataset_sha256)
        self._mutex = threading.Lock()
        self._poisoned = False
        self._next = 0
        self._active = None
        self._observations = []
        self._closed = False
        self._create("claim", {"evaluation_id": evaluation_id, "dataset_sha256": dataset_sha256,
                               "inventory_sha256": sha(inventory_raw), "inventory_receipt": inventory_receipt,
                               "conditions_receipt": conditions_receipt, "conditions": FIXED_CONDITIONS,
                               "blind_map_sha256": blind_map_sha256,
                               "slots": [list(s) for s in self._slots]})

    def _create(self, name, data):
        self._store.create(self._root + "/" + name + ".json", encode(data))

    def _guard(self):
        require(not self._poisoned and not self._closed, "global_stop")

    def _stop(self):
        self._poisoned = True
        try:
            self._create("stop", {"status": "HOLD"})
        except (OSError, ValueError):
            pass  # original claim stays burned even if stop persistence fails.

    def halt(self):
        with self._mutex:
            self._stop()

    def reserve(self, shard, case_id, arm):
        with self._mutex:
            self._guard()
            try:
                require(type(shard) is int and self._active is None and self._next < 48
                        and (shard, case_id, arm) == self._slots[self._next], "unexpected_slot")
                index = self._next
                attempt_id = sha(encode({"cohort": self._root, "slot": index, "attempt": 1}))
                record = {"shard": shard, "case_id": case_id, "arm": arm, "attempt": 1,
                          "attempt_id": attempt_id, "calls": 12, "tokens": 28096}
                self._create(f"slot-{index:02}-reserve", record)
                self._create(f"slot-{index:02}-inflight", record)
                self._active = record
                return attempt_id
            except BaseException:
                self._stop()
                raise

    def record(self, *, attempt_id, status, calls, tokens, admission_sha256, final_sha256,
               observation_sha256):
        with self._mutex:
            self._guard()
            try:
                a = self._active
                require(a is not None and attempt_id == a["attempt_id"], "wrong_attempt")
                require(type(status) is str and status in STATUSES, "status")
                match(observation_sha256, SHA, "observation_sha256")
                if admission_sha256 is not None:
                    match(admission_sha256, SHA, "admission_sha256")
                if final_sha256 is not None:
                    match(final_sha256, SHA, "final_sha256")
                for n, limit in ((calls, 12), (tokens, 28096)):
                    require(n is None or (type(n) is int and 0 <= n <= limit), "usage_limit")
                require(not any(observation_sha256 == o["observation_sha256"]
                                or (admission_sha256 is not None and admission_sha256 == o["admission_sha256"])
                                for o in self._observations), "duplicate_observation")
                row = {**a, "status": status, "used_calls": calls, "used_tokens": tokens,
                       "admission_sha256": admission_sha256, "final_sha256": final_sha256,
                       "observation_sha256": observation_sha256}
                self._create(f"slot-{self._next:02}-observed", row)
                self._observations.append(row)
                require(status == "complete" and type(calls) is int and calls > 0
                        and type(tokens) is int and final_sha256 is not None and admission_sha256 is not None, "first_attempt_incomplete")
                self._next += 1
                self._active = None
            except BaseException:
                self._stop()
                raise

    def close(self):
        with self._mutex:
            self._guard()
            try:
                rows = []
                require(self._next == 48 and self._active is None, "closure_incomplete")
                for index, (shard, cid, arm) in enumerate(self._slots):
                    reserve = decode(self._store.read(self._root + f"/slot-{index:02}-reserve.json"))
                    inflight = decode(self._store.read(self._root + f"/slot-{index:02}-inflight.json"))
                    observed = decode(self._store.read(self._root + f"/slot-{index:02}-observed.json"))
                    require(reserve == inflight and observed == self._observations[index]
                            and all(observed[k] == v for k, v in reserve.items())
                            and (shard, cid, arm) == (reserve["shard"], reserve["case_id"], reserve["arm"]),
                            "persisted_observation_mismatch")
                    rows.append({k: observed[k] for k in FIELDS})
                ids = [s[1] for s in self._slots[::2]]
                require(check_first_attempts(ids, rows)["structurally_complete"], "closure_first_attempt")
                closure = {"observations": [sha(encode(x)) for x in self._observations],
                           "used_calls": sum(x["used_calls"] for x in self._observations),
                           "used_tokens": sum(x["used_tokens"] for x in self._observations),
                           "shards": 8, "slots": 48, "blind_map_sha256": self.blind_map_sha256,
                           "formal_score_authorized": False}
                self._create("closure", closure)
                self._closed = True
                return sha(encode(closure))
            except BaseException:
                self._stop()
                raise


    def closed_observations(self, closure_digest):
        """Read the exact durable synthetic closure; not an authorization assertion."""
        with self._mutex:
            require(self._closed and not self._poisoned, "closure_not_completed")
            raw = self._store.read(self._root + "/closure.json")
            require(sha(raw) == closure_digest, "closure_digest_mismatch")
            closure = decode(raw)
            records = [decode(self._store.read(self._root + f"/slot-{i:02}-observed.json"))
                       for i in range(48)]
            require(closure["observations"] == [sha(encode(x)) for x in records]
                    and records == self._observations, "closure_records_changed")
            return records

    def read_fixture_stage(self, stage):
        match(stage, ID, "fixture_stage")
        return self._store.read(self._root + "/fixture-stages/" + stage + ".json")

    def append_fixture_stage(self, stage, raw):
        """Create-only synthetic evidence namespace, never consumes real permissions."""
        match(stage, ID, "fixture_stage")
        decode(raw)
        self._store.create(self._root + "/fixture-stages/" + stage + ".json", raw)
