"""Incomplete-first-observation diagnostic evidence for Issue 620.

Never promotes partial output, a missing final display, or an unresolved
judgment to score eligibility. Separate blind A/B and ratifier signatures
retain explicit adverse evidence provisionally for later normative review.
"""
from auth620 import encode,decode,sha,fields,require
from blind_fixture_620 import validate_mapping,map_digest
from formal_preflight_620 import OfflineApprovalDAG

ADVERSE=("wrong_requested_assertion","material_out_of_scope_error",
         "unsafe_negative_assertion","invalid_citation")


class PartialAdverseReview:
    def __init__(self,*,flow,raw,mapping,salt):
        require(type(flow) is OfflineApprovalDAG,"partial_protected_context")
        now=flow._recheck()
        record=flow.ledger.stopped_observation()
        plan=flow.plan
        validate_mapping(plan.inventory_raw,mapping)
        require(map_digest(mapping,salt)==plan.inputs()["blind_mapping_commitment"],
                "partial_blind_map")
        observation=plan.verify_partial(ledger=flow.ledger,registry=flow.registry,now=now,
                   record=record,raw=raw,run_receipt=flow._receipt("run_permission"))
        require(observation["observation_sha256"]==record["observation_sha256"]
                and observation["status"]==record["status"]
                and record["final_sha256"] is None
                and record["used_calls"] is None and record["used_tokens"] is None,
                "partial_observation_binding")
        item=next(x for x in mapping if (x["shard"],x["case_id"],x["arm"])==
                  (record["shard"],record["case_id"],record["arm"]))
        self._flow=flow
        self._raw=dict(raw)
        self._record=record
        self._receipt=observation["observation_sha256"]
        self._blind=item["blind_id"]
        self._bytes=raw["final_bytes"]
        self._sources=decode(raw["sources_raw"])["sources"]
        self._source_texts=[self._sources[x]["text"] for x in sorted(self._sources)]
        self._context=plan.judge_context_for(record["case_id"])
        self._token=sha(encode({"receipt":self._receipt,"blind_id":self._blind,"salt":salt}))
        self._reviews={}
        self._resolved=None
        self._proofs={}
        self._persist("partial-capture",encode({k:v for k,v in observation.items() if k not in
                ("observation_sha256","admission_sha256","evidence_eligibility","diagnostic_only")}),
                raw["envelope"],self._receipt)

    def _persist(self,stage,raw,env,receipt):
        data=encode({"payload":decode(raw),"envelope":decode(env),"receipt":receipt})
        self._flow.ledger.append_fixture_stage(stage,data)
        self._proofs[stage]=data

    def _recheck(self):
        flow=self._flow
        now=flow._recheck()
        require(flow.ledger.stopped_observation()==self._record,"partial_record_changed")
        o=flow.plan.verify_partial(ledger=flow.ledger,registry=flow.registry,now=now,
            record=self._record,raw=self._raw,run_receipt=flow._receipt("run_permission"))
        require(o["observation_sha256"]==self._receipt,"partial_capture_changed")
        for stage,package in self._proofs.items():
            require(flow.ledger.read_fixture_stage(stage)==package,"partial_stage_changed")
            if stage=="partial-capture": continue
            # Re-verify exact stored signed stage, including validity and current registry.
            p=decode(package)
            e=p["envelope"]
            kind=e["artifact_type"]
            role=({"partial-judge_a":"judge_a","partial-judge_b":"judge_b",
                   "partial-adjudication":"adjudicator","partial-ratification":"adjudication_ratifier"}[stage])
            require(flow.registry.verify(encode(e),encode(p["payload"]),artifact_type=kind,
                         artifact_id=stage,role=role,parents=e["parent_receipt_digests"],
                         now=now)==p["receipt"],"partial_signature_changed")
        return now

    def judge_bundle(self):
        self._recheck()
        require(self._resolved is None,"partial_locked")
        return {"blind_id":self._blind,"evidence_token":self._token,
                "partial_display":None if self._bytes is None else self._bytes.decode("utf-8"),
                "source_texts":list(self._source_texts),"targets":list(ADVERSE),
                "judge_context":decode(encode(self._context))}

    def _values(self,raw,targets):
        p=fields(decode(raw),{"blind_id","evidence_token","decisions"})
        require(p["blind_id"]==self._blind and p["evidence_token"]==self._token,
                "partial_judgment_attribution")
        require(type(p["decisions"]) is list and len(p["decisions"])==len(targets),
                "partial_judgment_inventory")
        values={}
        for target,row in zip(targets,p["decisions"],strict=True):
            fields(row,{"target","value","byte_span","basis_digest"})
            require(row["target"]==target and type(row["value"]) is str
                    and row["value"] in {"true","unknown"},"partial_adverse_only")
            if row["value"]=="unknown":
                require(row["byte_span"] is None and row["basis_digest"] is None,"unknown_partial_span")
            else:
                span=row["byte_span"]
                require(self._bytes is not None and type(span) is list and len(span)==2
                        and all(type(x) is int for x in span),"partial_adverse_span")
                start,end=span
                require(0<=start<end<=len(self._bytes),"partial_adverse_span")
                # Strict byte-range binding, not semantic proof by hash alone.
                excerpt=self._bytes[start:end].decode("utf-8","strict")
                basis=encode({"token":self._token,"target":target,"span":span,
                              "excerpt":excerpt,"source_texts":self._source_texts,
                              "judge_context":self._context})
                require(row["basis_digest"]==sha(basis),"partial_basis_mismatch")
            values[target]=row["value"]
        return values

    def submit(self,role,raw,envelope):
        now=self._recheck()
        require(self._resolved is None and role in ("judge_a","judge_b")
                and role not in self._reviews,"partial_judge_order")
        values=self._values(raw,ADVERSE)
        stage="partial-"+role
        receipt=self._flow.registry.verify(envelope,raw,artifact_type="atomic_judgment",
                    artifact_id=stage,role=role,parents=[self._receipt],now=now)
        self._persist(stage,raw,envelope,receipt)
        self._reviews[role]=(values,receipt)
        return receipt

    def conflicts(self):
        require(set(self._reviews)=={"judge_a","judge_b"},"partial_pair_missing")
        a,b=[self._reviews[r] for r in ("judge_a","judge_b")]
        return [x for x in ADVERSE if a[0][x]!=b[0][x]],[a[1],b[1]]

    def adjudicate(self,raw,envelope,ratification_raw,ratification_envelope):
        now=self._recheck()
        require(self._resolved is None,"partial_locked")
        conflicts,parents=self.conflicts()
        require(bool(conflicts),"partial_no_disagreement")
        values=self._values(raw,conflicts)
        receipt=self._flow.registry.verify(envelope,raw,artifact_type="adjudication",
                   artifact_id="partial-adjudication",role="adjudicator",parents=parents,now=now)
        expected={"blind_id":self._blind,"adjudication_receipt":receipt,
                  "decision":"diagnostic-only","formal_eligible":False}
        require(decode(ratification_raw)==expected,"partial_ratification_binding")
        ratifier=self._flow.registry.verify(ratification_envelope,ratification_raw,
                   artifact_type="adjudication_ratification",artifact_id="partial-ratification",
                   role="adjudication_ratifier",parents=[*parents,receipt],now=now)
        self._persist("partial-adjudication",raw,envelope,receipt)
        self._persist("partial-ratification",ratification_raw,ratification_envelope,ratifier)
        self._resolved={**self._reviews["judge_a"][0],**values}
        return ratifier

    def diagnostic(self):
        self._recheck()
        conflicts,parents=self.conflicts()
        if conflicts:
            require(self._resolved is not None,"partial_adjudication_missing")
            values=self._resolved
        else:
            values=self._reviews["judge_a"][0]
        summary={"status":"HOLD","diagnostic_only":True,"evidence_eligibility":"UNRATIFIED",
                 "adverse_claims":sum(v=="true" for v in values.values()),
                 "unknown_claims":sum(v=="unknown" for v in values.values()),
                 "final_sha256":None,"formal_f1":None,"formal_score_authorized":False,
                 "live_authorized":False,"quality_go":False,"release_go":False}
        if self._resolved is None: self._resolved=dict(values)
        return summary
