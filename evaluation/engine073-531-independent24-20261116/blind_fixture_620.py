"""Synthetic-only #620 blind review graph and full-scope opening.

No caller-supplied 'verified' token grants authority. No actual data or Gold is
accepted by a formal entrypoint. Same-UID operator is trusted; objective blind
process separation and semantic ratification remain external HOLD.
"""
import re
from auth620 import encode, decode, sha, require, match, fields, SHA, ID
from first_claim_620 import inventory_slots
from formal_scorer_candidate import ATOMS, STATES, CITES
from observation_620 import verify_observation

BLIND_ID = re.compile(r"blind-[0-9a-f]{32}\Z")


def map_digest(mapping, salt):
    match(salt, SHA, "map_salt")
    return sha(encode({"mapping": mapping, "salt": salt}))


def validate_mapping(inventory_raw, mapping):
    require(type(mapping) is list and len(mapping) == 48, "blind_map_count")
    seen, slots = set(), set()
    for x in mapping:
        fields(x, {"blind_id", "shard", "case_id", "arm"})
        bid = match(x["blind_id"], BLIND_ID, "blind_id")
        require(type(x["shard"]) is int and type(x["case_id"]) is str
                and type(x["arm"]) is str, "blind_map_type")
        slot = (x["shard"], x["case_id"], x["arm"])
        require(bid not in seen and slot not in slots, "blind_map_duplicate")
        seen.add(bid); slots.add(slot)
    require(slots == set(inventory_slots(inventory_raw))
            and [x["blind_id"] for x in mapping] == sorted(seen), "blind_map_bijection")


class BlindFixtureGraph:
    """Coordinator-owned in-memory projection + immutable local fixture receipts.

    The class is an offline test integration boundary, not a real scorer service.
    Actual blind judge access isolation cannot be proven by this Python object.
    """
    def __init__(self, *, ledger, registry, inventory_raw, inventory_receipt, conditions_receipt,
                 closure_digest, observation_inputs, mapping, salt, now):
        validate_mapping(inventory_raw, mapping)
        require(map_digest(mapping, salt) == ledger.blind_map_sha256, "blind_precommit_mismatch")
        match(inventory_receipt, SHA, "inventory_receipt")
        match(conditions_receipt, SHA, "conditions_receipt")
        records = ledger.closed_observations(closure_digest)
        require(type(observation_inputs) is list and len(observation_inputs) == 48, "graph_count")
        # Reverify raw signed capture evidence; never accept arbitrary "verified" dicts.
        observations = [verify_observation(**{**inputs, "registry": registry, "now": now})
                        for inputs in observation_inputs]
        self._ledger, self._registry, self._now = ledger, registry, now
        self._inventory_raw = inventory_raw
        self._inventory_receipt, self._conditions_receipt = inventory_receipt, conditions_receipt
        self._closure = closure_digest
        self._map_bytes = encode(mapping)
        self._salt = salt
        self._reviews, self._adjudicated, self._decisions = {}, {}, {}
        self._signed_stages = {}
        self._lock_receipt = self._opening_receipt = None
        self._by_id = {}
        by_slot = {(x["shard"], x["case_id"], x["arm"]): x["blind_id"] for x in mapping}
        by_case = {c["case_id"]: c for c in decode(inventory_raw)}
        for rec, observation in zip(records, observations, strict=True):
            slot = (rec["shard"], rec["case_id"], rec["arm"])
            require(observation["slot"] == list(slot)
                    and all(observation[k] == rec[k] for k in (
                        "attempt_id", "status", "observation_sha256", "admission_sha256", "final_sha256"))
                    and observation["calls"] == rec["used_calls"]
                    and observation["tokens"] == rec["used_tokens"]
                    and sha(observation["display"].encode("utf-8")) == rec["final_sha256"]
                    and len(observation["display"].encode("utf-8")) == observation["final_length"],
                    "graph_observation_mismatch")
            bid = by_slot[slot]
            targets = list(ATOMS) + [f"fact-{i}" for i in range(by_case[slot[1]]["fact_count"])]
            # The fixture's citation inventory is pre-fixed to one cite per slot.
            require(observation["citation_targets"] == ["native-0"], "fixture_citation_inventory")
            targets += ["citation-native-0"]
            token = sha(encode({"blind_id": bid, "terminal_receipt": rec["observation_sha256"],
                                "admission_receipt": rec["admission_sha256"], "salt": salt}))
            self._by_id[bid] = {"observation": decode(encode(observation)),
                                "targets": targets, "token": token}
        self._ids = tuple(sorted(self._by_id))

    def _persist_stage(self, stage, raw, envelope, kind, role, parents, receipt):
        package = encode({"payload":decode(raw),"envelope":decode(envelope)})
        self._ledger.append_fixture_stage(stage, package)
        self._signed_stages[stage] = (raw,envelope,kind,role,tuple(parents),receipt)

    def _reverify_stages(self):
        for stage,(raw,envelope,kind,role,parents,receipt) in self._signed_stages.items():
            require(self._ledger.read_fixture_stage(stage) ==
                    encode({"payload":decode(raw),"envelope":decode(envelope)}),
                    "fixture_stage_changed")
            require(self._registry.verify(envelope,raw,artifact_type=kind,artifact_id=stage,
                        role=role,parents=list(parents),now=self._now) == receipt,
                    "fixture_stage_signature_changed")

    def _unlocked(self):
        require(self._lock_receipt is None, "blind_locked")

    def blind_ids(self):
        return self._ids

    def judge_bundle(self, blind_id):
        """Only that blind response, not arm/shard/order/trace or A/B results."""
        self._unlocked()
        p = self._by_id[blind_id]
        return {"blind_id": blind_id, "display": p["observation"]["display"],
                "source_texts": list(p["observation"]["source_texts"]),
                "targets": list(p["targets"]), "evidence_token": p["token"]}

    def _values(self, blind_id, raw, expected_targets):
        manifest = fields(decode(raw), {"blind_id", "decisions"})
        require(manifest["blind_id"] == blind_id and type(manifest["decisions"]) is list,
                "judgment_blind_id")
        packet = self._by_id[blind_id]
        require(len(manifest["decisions"]) == len(expected_targets), "judgment_missing")
        result = {}
        for target, row in zip(expected_targets, manifest["decisions"], strict=True):
            fields(row, {"target", "value", "evidence_token", "evidence_kind", "basis_digest"})
            require(row["target"] == target and row["evidence_token"] == packet["token"],
                    "judgment_attribution")
            allowed = CITES if target.startswith("citation-") else STATES
            require(type(row["value"]) is str and row["value"] in allowed, "judgment_state")
            match(row["basis_digest"], SHA, "judgment_basis")
            allowed_kinds = {"explicit_adverse", "explicit_favorable", "final_output_dependent_omission",
                             "citation_entailment", "qualification", "unknown"}
            require(type(row["evidence_kind"]) is str and row["evidence_kind"] in allowed_kinds
                    and (row["value"] != "unknown" or row["evidence_kind"] == "unknown"),
                    "judgment_evidence_kind")
            basis = {"target": target, "value": row["value"], "evidence_kind": row["evidence_kind"],
                     "evidence_token": packet["token"], "display": packet["observation"]["display"],
                     "source_texts": packet["observation"]["source_texts"]}
            require(row["basis_digest"] == sha(encode(basis)), "judgment_basis_mismatch")
            result[target] = row["value"]
        return result

    def submit(self, blind_id, role, raw, envelope):
        self._unlocked()
        require(type(role) is str and role in ("judge_a", "judge_b") and blind_id in self._by_id
                and (blind_id, role) not in self._reviews, "duplicate_or_foreign_judgment")
        p = self._by_id[blind_id]
        values = self._values(blind_id, raw, p["targets"])
        parent = [p["observation"]["observation_sha256"], p["observation"]["admission_sha256"]]
        receipt = self._registry.verify(envelope, raw, artifact_type="atomic_judgment",
            artifact_id=blind_id + "-" + role, role=role, parents=parent, now=self._now)
        self._persist_stage(blind_id+"-"+role,raw,envelope,"atomic_judgment",role,parent,receipt)
        self._reviews[blind_id, role] = (values, receipt)
        return receipt

    def _disagreements(self, blind_id):
        require(all((blind_id, r) in self._reviews for r in ("judge_a", "judge_b")),
                "judgment_pair_missing")
        a, b = [self._reviews[blind_id, r] for r in ("judge_a", "judge_b")]
        return [k for k in self._by_id[blind_id]["targets"] if a[0][k] != b[0][k]], [a[1], b[1]]

    def adjudicate(self, blind_id, raw, envelope, ratification_raw, ratification_envelope):
        self._unlocked()
        require(blind_id in self._by_id and blind_id not in self._adjudicated, "repeated_adjudication")
        conflicts, parents = self._disagreements(blind_id)
        require(bool(conflicts), "unnecessary_adjudication")
        decisions = self._values(blind_id, raw, conflicts)
        receipt = self._registry.verify(envelope, raw, artifact_type="adjudication",
            artifact_id=blind_id+"-adjudication", role="adjudicator", parents=parents, now=self._now)
        expected_ratification = {"blind_id": blind_id, "adjudication_receipt": receipt,
                                 "decision": "synthetic-confirmed"}
        require(decode(ratification_raw) == expected_ratification, "ratification_binding")
        ratifier = self._registry.verify(ratification_envelope, ratification_raw,
            artifact_type="adjudication_ratification", artifact_id=blind_id+"-ratification",
            role="adjudication_ratifier", parents=[*parents, receipt], now=self._now)
        self._persist_stage(blind_id+"-adjudication",raw,envelope,"adjudication","adjudicator",parents,receipt)
        self._persist_stage(blind_id+"-ratification",ratification_raw,ratification_envelope,"adjudication_ratification","adjudication_ratifier",[*parents,receipt],ratifier)
        self._adjudicated[blind_id] = (decisions, receipt, ratifier)
        return ratifier

    def _resolved(self):
        rows, parents = {}, [self._closure]
        for bid in self._ids:
            disagreements, receipts = self._disagreements(bid)
            parent_a = self._reviews[bid, "judge_a"][0]
            row = dict(parent_a)
            parents.extend(receipts)
            if disagreements:
                require(bid in self._adjudicated, "adjudication_missing")
                decisions, receipt, ratifier = self._adjudicated[bid]
                row.update(decisions)
                parents.extend((receipt, ratifier))
            rows[bid] = row
        return rows, parents

    def lock_payload(self):
        self._unlocked()
        self._reverify_stages()
        rows, parents = self._resolved()
        payload = {"closure": self._closure, "inventory_receipt": self._inventory_receipt,
                   "conditions_receipt": self._conditions_receipt,
                   "map_commitment": self._ledger.blind_map_sha256,
                   "decision_manifest_sha256": sha(encode(rows)),
                   "observation_count": 48}
        return encode(payload), parents

    def lock(self, raw, envelope):
        expected, parents = self.lock_payload()
        require(raw == expected, "lock_manifest_mismatch")
        receipt = self._registry.verify(envelope, raw, artifact_type="blind_lock",
            artifact_id="blind-lock", role="blind_lock_admin", parents=parents, now=self._now)
        self._persist_stage("blind-lock",raw,envelope,"blind_lock","blind_lock_admin",parents,receipt)
        self._decisions, _ = self._resolved()
        self._lock_receipt = receipt
        return receipt

    def unblind(self, raw, envelope):
        require(self._lock_receipt is not None and self._opening_receipt is None, "blind_open_order")
        self._reverify_stages()
        data = fields(decode(raw), {"mapping", "salt", "lock_receipt"})
        validate_mapping(self._inventory_raw, data["mapping"])
        require(data["lock_receipt"] == self._lock_receipt
                and encode(data["mapping"]) == self._map_bytes
                and data["salt"] == self._salt
                and map_digest(data["mapping"], data["salt"]) == self._ledger.blind_map_sha256,
                "blind_open_mismatch")
        receipt = self._registry.verify(envelope, raw, artifact_type="unblind",
            artifact_id="full-unblind", role="unblinder", parents=[self._lock_receipt], now=self._now)
        self._persist_stage("full-unblind",raw,envelope,"unblind","unblinder",[self._lock_receipt],receipt)
        self._opening_receipt = receipt
        return receipt

    def projection(self):
        """Projection from closed, locked, fully opened fixture graph; never formal."""
        require(self._lock_receipt is not None and self._opening_receipt is not None,
                "unblinding_required")
        self._reverify_stages()
        decisions, _ = self._resolved()
        require(decisions == self._decisions, "locked_decisions_changed")
        records = self._ledger.closed_observations(self._closure)
        by_slot = {(r["shard"], r["case_id"], r["arm"]): r for r in records}
        cases = decode(self._inventory_raw)
        by_case = {c["case_id"]: c for c in cases}
        projected = []
        for item in decode(self._map_bytes):
            bid = item["blind_id"]
            r = by_slot[item["shard"], item["case_id"], item["arm"]]
            d = self._decisions[bid]
            projected.append({k:r[k] for k in ("case_id","arm","attempt","status","admission_sha256","final_sha256")}
                | {"facts": [d[f"fact-{i}"] for i in range(by_case[r["case_id"]]["fact_count"])],
                   "atoms": {k:d[k] for k in ATOMS}, "citations": d["citation-native-0"],
                   "consensus": True, "judgment_sha256": sha(encode({"blind_id": bid, "decision": d,
                        "lock_receipt": self._lock_receipt, "opening_receipt": self._opening_receipt}))})
        require(len(projected) == 48, "projection_count")
        return cases, projected
