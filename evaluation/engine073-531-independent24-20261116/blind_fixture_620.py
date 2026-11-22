"""Synthetic-only #620 blind review graph and full-scope opening.

No caller-supplied 'verified' token grants authority. No actual data or Gold is
accepted by a formal entrypoint. Same-UID operator is trusted; objective blind
process separation and semantic ratification remain external HOLD.
"""
import re
from auth620 import encode, decode, sha, require, match, fields, SHA, ID
from first_claim_620 import inventory_slots
from formal_scorer_candidate import ATOMS, STATES, CITES
from protected_plan_620 import ProtectedPlan

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


def citation_summary(decisions):
    """Lossless conservative collapse to the existing case-level enum.

    A known invalid citation survives uncertainty elsewhere. Unknown cannot
    become valid/missing; only explicit complete coverage can establish valid.
    This is an unratified adapter convention, not a change to scorer arithmetic.
    """
    vals=[v for k,v in decisions.items() if k.startswith("citation-")]
    require(bool(vals) and all(type(x) is str and x in CITES for x in vals),"citation_manifest")
    return next(x for x in ("invalid","unknown","missing","valid") if x in vals)


class BlindFixtureGraph:
    """Coordinator-owned in-memory projection + immutable local fixture receipts.

    The class is an offline test integration boundary, not a real scorer service.
    Actual blind judge access isolation cannot be proven by this Python object.
    """
    def __init__(self, *, ledger, registry, plan, run_receipt,
                 closure_digest, observation_inputs, mapping, salt, now, clock):
        require(type(plan) is ProtectedPlan,"protected_plan_required")
        plan.reverify(registry,now)
        plan.bind_ledger(ledger)
        inventory_raw=plan.inventory_raw
        inventory_receipt=plan.inventory_receipt
        conditions_receipt=plan.conditions_receipt
        validate_mapping(inventory_raw, mapping)
        require(map_digest(mapping, salt) == ledger.blind_map_sha256, "blind_precommit_mismatch")
        records = ledger.closed_observations(closure_digest)
        require(type(observation_inputs) is list and len(observation_inputs) == 48, "graph_count")
        observations = [plan.verify_capture(ledger=ledger,registry=registry,now=now,record=record,
                        raw=raw,run_receipt=run_receipt)
                        for record,raw in zip(records,observation_inputs,strict=True)]
        require(callable(clock),"graph_trusted_clock")
        self._clock=clock
        self._poisoned=False
        self._plan=plan
        self._run_receipt=run_receipt
        self._observation_inputs=tuple(dict(raw) for raw in observation_inputs)
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
            fixed=plan.targets_for(slot[1])
            native=observation["citation_targets"]
            require(type(native) is list and len(native)<=256 and len(set(native))==len(native),
                    "observed_citation_inventory")
            # Review every required support target AND every emitted citation,
            # including extra citations. A missing item is not silently false.
            targets = fixed["atoms"] + fixed["facts"] + [
                "citation-required-"+k for k in fixed["required_citations"]] + [
                "citation-observed-"+k for k in native]
            token = sha(encode({"blind_id": bid, "terminal_receipt": rec["observation_sha256"],
                                "admission_receipt": rec["admission_sha256"], "salt": salt}))
            self._by_id[bid] = {"observation": decode(encode(observation)),
                                "targets": targets, "token": token,"context":plan.judge_context_for(slot[1])}
        self._ids = tuple(sorted(self._by_id))

    def _persist_stage(self, stage, raw, envelope, kind, role, parents, receipt):
        package = encode({"payload":decode(raw),"envelope":decode(envelope)})
        try:
            self._ledger.append_fixture_stage(stage, package)
        except BaseException:
            self._poisoned=True
            self._ledger.halt()
            raise
        self._signed_stages[stage] = (raw,envelope,kind,role,tuple(parents),receipt)

    def _reverify_stages(self):
        require(not self._poisoned,"graph_quarantined")
        now=self._clock()
        require(now>=self._now,"graph_clock_rollback")
        self._now=now
        self._plan.reverify(self._registry,self._now)
        self._plan.bind_ledger(self._ledger)
        for stage,(raw,envelope,kind,role,parents,receipt) in self._signed_stages.items():
            require(self._ledger.read_fixture_stage(stage) ==
                    encode({"payload":decode(raw),"envelope":decode(envelope)}),
                    "fixture_stage_changed")
            require(self._registry.verify(envelope,raw,artifact_type=kind,artifact_id=stage,
                        role=role,parents=list(parents),now=self._now) == receipt,
                    "fixture_stage_signature_changed")

    def _unlocked(self):
        require(self._lock_receipt is None and not self._poisoned, "blind_locked")
        now=self._clock()
        require(now>=self._now,"graph_clock_rollback")
        self._now=now
        self._plan.reverify(self._registry,now)
        self._plan.bind_ledger(self._ledger)

    def blind_ids(self):
        return self._ids

    def judge_bundle(self, blind_id):
        """Only that blind response, not arm/shard/order/trace or A/B results."""
        self._unlocked()
        p = self._by_id[blind_id]
        return {"blind_id": blind_id, "display": p["observation"]["display"],
                "source_texts": list(p["observation"]["source_texts"]),
                "targets": list(p["targets"]), "evidence_token": p["token"],
                "judge_context":decode(encode(p["context"])),
                "citation_details":decode(encode(p["observation"]["citation_details"]))}

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
                    and ((row["value"] == "unknown") == (row["evidence_kind"] == "unknown")),
                    "judgment_evidence_kind")
            if target.startswith("citation-") and row["value"]!="unknown":
                require(row["evidence_kind"]=="citation_entailment","citation_evidence_kind")
            if target.startswith("citation-observed-"):
                require(row["value"]!="missing","observed_citation_cannot_be_missing")
            if target.startswith("citation-required-") and not packet["observation"]["citation_targets"]:
                require(row["value"]!="valid","citation_support_absent")
            basis = {"target": target, "value": row["value"], "evidence_kind": row["evidence_kind"],
                     "evidence_token": packet["token"], "display": packet["observation"]["display"],
                     "source_texts": packet["observation"]["source_texts"],
                     "judge_context":packet["context"],
                     "citation_details":packet["observation"]["citation_details"]}
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
        payload = {"closure": self._closure, "plan_digest":self._plan.digest,
                   "inventory_receipt": self._inventory_receipt,
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
        for r,raw in zip(records,self._observation_inputs,strict=True):
            o=self._plan.verify_capture(ledger=self._ledger,registry=self._registry,now=self._now,
                record=r,raw=raw,run_receipt=self._run_receipt)
            require(all(o[k]==r[k] for k in ("attempt_id","observation_sha256","admission_sha256","final_sha256")),
                    "projection_capture_changed")
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
                   "atoms": {k:d[k] for k in ATOMS}, "citations": citation_summary(d),
                   "consensus": True, "judgment_sha256": sha(encode({"blind_id": bid, "decision": d,
                        "lock_receipt": self._lock_receipt, "opening_receipt": self._opening_receipt}))})
        require(len(projected) == 48, "projection_count")
        return cases, projected
