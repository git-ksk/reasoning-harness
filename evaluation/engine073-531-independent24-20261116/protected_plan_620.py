"""Issue 620 trusted-coordinator configuration, not a caller capability.

The operator supplies a pinned profile/plan/registry from the control plane.
All untrusted observations contain only raw evidence. The same-UID operator
remains trusted; the formal external trust boundary is deliberately HOLD.
"""
from auth620 import encode, decode, sha, match, fields, require, SHA, ID, Registry, instant
from first_claim_620 import FIXED_CONDITIONS, inventory_slots, cohort_key, FirstClaimLedger
from observation_620 import verify_observation, verify_partial_capture
from formal_scorer_candidate import ATOMS
from local_store import LocalStoreProfile, LocalAtomicStore
from review_auth import validate_policy

FULL_COMMITS = {"baseline_core":"4abee90f0501f0d8fc09b453a0b45c942fbed69e",
                "candidate_core":"7466c6f90d420a98c99713b96d6a0623e2aeef52",
                "cli":"b85a4a0b2ab03397b361a8620d7a97dd26b2eb36"}
QUALITY = {"f1":"9/10","precision":"95/100","recall":"85/100",
           "unsafe_negative":0,"invalid_citation":0,"paired_regression":False}
PINS = {"question_inventory_digest","gold_digest","source_manifest_digest",
        "scoring_contract_digest","formal_implementation_digest","normative_policy_digest",
        "design_receipt_digest","identity_evidence_digest","isolation_evidence_digest"}
PLAN_FIELDS = PINS | {"contract","evaluation_id","registry_digest","store_digest","inventory_receipt",
    "conditions","full_commits","quality","legacy_annotation_receipts","runtime_digests",
    "blind_mapping_commitment","slots","targets","legacy_policy_pin","executor_id","evaluator_id",
    "billing_account_pin","approval_start","approval_end"}
SLOT_FIELDS = {"shard","case_id","arm","binding_sha256","source_pin","bundle_pin","authority_pin"}
CAPTURE_FIELDS = {"binding_raw","trace_raw","journal_raw","stdout","stderr","sources_raw",
                  "final_bytes","observation_envelope","safety_envelope","legacy_attestation"}


def profile_digest(profile):
    require(type(profile) is LocalStoreProfile,"protected_store_profile")
    return sha(encode(profile.to_json()))


PARTIAL_FIELDS={"binding_raw","trace_raw","journal_raw","stdout","stderr","sources_raw",
                "final_bytes","display_span","envelope","status"}


class ProtectedPlan:
    """Construct only in the trusted offline coordinator, never from request JSON."""
    def __init__(self, *, profile, pin, raw, registry, inventory_raw, inventory_envelope,
                 conditions_envelope, legacy_policy_raw, judge_contexts_raw, now):
        match(pin,SHA,"plan_pin")
        require(type(registry) is Registry and sha(raw)==pin,"protected_plan_pin")
        p=fields(decode(raw),PLAN_FIELDS)
        require(p["contract"]=="620-protected-plan/1" and p["evaluation_id"]==registry.evaluation_id
                and p["registry_digest"]==registry.digest
                and p["store_digest"]==profile_digest(profile),"protected_plan_context")
        # Exact canonical bytes distinguish booleans and integers.
        require(encode(p["conditions"])==encode(FIXED_CONDITIONS)
                and p["full_commits"]==FULL_COMMITS
                and encode(p["quality"])==encode(QUALITY),"protected_fixed_conditions")
        for key in PINS | {"registry_digest","store_digest","inventory_receipt",
                           "blind_mapping_commitment","legacy_policy_pin","billing_account_pin"}:
            match(p[key],SHA,"plan_digest")
        start,end=instant(p["approval_start"]),instant(p["approval_end"])
        require(start<=now<end and (end-start).total_seconds()<=86400,"plan_approval_window")
        annotations=p["legacy_annotation_receipts"]
        require(type(annotations) is list and len(annotations)==3,"plan_annotations")
        for digest in annotations:
            match(digest,SHA,"plan_annotation")
        require(len(set(annotations))==3,"plan_annotation_duplicate")
        runtimes=fields(p["runtime_digests"],{"baseline","candidate","cli","python","dependencies","scorer"})
        for digest in runtimes.values(): match(digest,SHA,"runtime_pin")
        slots=inventory_slots(inventory_raw)
        inv=registry.verify(inventory_envelope,inventory_raw,artifact_type="inventory",
                            artifact_id="inventory",role="author",parents=[],now=now)
        require(inv==p["inventory_receipt"],"plan_inventory_receipt")
        cond=registry.verify(conditions_envelope,raw,artifact_type="execution_conditions",
                             artifact_id="conditions",role="coordinator",parents=[inv],now=now)
        require(type(p["slots"]) is list and len(p["slots"])==48,"plan_schedule_count")
        for row,slot in zip(p["slots"],slots,strict=True):
            fields(row,SLOT_FIELDS)
            require(type(row["shard"]) is int and (row["shard"],row["case_id"],row["arm"])==slot,
                    "plan_schedule_slot")
            for k in SLOT_FIELDS-{"shard","case_id","arm"}: match(row[k],SHA,"plan_slot_pin")
        for i in range(0,48,2):
            require(p["slots"][i]["source_pin"]==p["slots"][i+1]["source_pin"],
                    "paired_source_catalog_mismatch")
        cases=decode(inventory_raw)
        require(type(p["targets"]) is list and len(p["targets"])==24,"plan_target_count")
        contexts=decode(judge_contexts_raw)
        require(type(contexts) is list and len(contexts)==24,"judge_context_inventory")
        for target,case,context in zip(p["targets"],cases,contexts,strict=True):
            fields(target,{"case_id","atoms","facts","required_citations","judge_context_digest"})
            require(target["case_id"]==case["case_id"] and target["atoms"]==list(ATOMS)
                    and target["facts"]==[f"fact-{i}" for i in range(case["fact_count"])],
                    "plan_target_inventory")
            cites=target["required_citations"]
            require(type(cites) is list and 1<=len(cites)<=256,"plan_citation_count")
            for cite in cites: match(cite,ID,"plan_citation_id")
            require(len(set(cites))==len(cites),"plan_citation_duplicate")
            fields(context,{"case_id","question","facts","citations"})
            require(context["case_id"]==case["case_id"] and type(context["question"]) is str
                    and bool(context["question"]) and len(context["question"])<=65536,
                    "judge_context_question")
            for rows,ids in ((context["facts"],target["facts"]),(context["citations"],cites)):
                require(type(rows) is list and len(rows)==len(ids),"judge_context_targets")
                for item,expected in zip(rows,ids,strict=True):
                    fields(item,{"id","requirement"})
                    require(item["id"]==expected and type(item["requirement"]) is str
                            and bool(item["requirement"]) and len(item["requirement"])<=65536,
                            "judge_context_requirement")
            require(sha(encode(context))==target["judge_context_digest"],"judge_context_pin")

        require(sha(legacy_policy_raw)==p["legacy_policy_pin"],"plan_legacy_policy")
        aliases,keys=validate_policy(legacy_policy_raw)
        executor=aliases.get(p["executor_id"]); evaluator=aliases.get(p["evaluator_id"])
        require(executor is not None and evaluator is not None,"plan_legacy_identity")
        # The safety role must use the same principal/key as the legacy verifier.
        new=decode(registry._raw)["principals"]
        by_role={x["role"]:x for x in new}
        from auth620 import b64decode
        for role,principal,alias in (("executor",executor,p["executor_id"]),
                                     ("safety_reviewer",evaluator,p["evaluator_id"])):
            expected=by_role.get(role)
            require(expected is not None and principal["subject_id"]==expected["subject_id"]
                    and alias in [expected["subject_id"],*expected["aliases"]],"legacy_new_role_binding")
            if role=="safety_reviewer":
                key=keys.get(expected["key_id"])
                require(key is not None and key["subject_id"]==expected["subject_id"]
                        and not key["revoked"] and not expected["revoked"]
                        and key["public_key_hex"]==b64decode(expected["public_key"],32).hex(),
                        "legacy_new_key_binding")
        LocalAtomicStore(profile,role="executor")
        self._profile=profile
        self._raw=raw
        self._pin=pin
        self._inventory=inventory_raw
        self._inv_env=inventory_envelope
        self._cond_env=conditions_envelope
        self._policy=legacy_policy_raw
        self._judge_contexts=judge_contexts_raw
        self._registry=registry
        self._inv=inv
        self._cond=cond

    @property
    def digest(self): return self._pin

    @property
    def inventory_raw(self): return self._inventory

    @property
    def inventory_receipt(self): return self._inv

    @property
    def conditions_receipt(self): return self._cond

    @property
    def dataset_digest(self):
        p=decode(self._raw)
        # Independent dataset identity excludes mutable evaluation ID, rubric,
        # schedule, runtime, budget, output path and random blind salt.
        return sha(encode({k:p[k] for k in ("question_inventory_digest","source_manifest_digest")}))

    def inputs(self): return decode(self._raw)

    def reverify(self, registry, now):
        require(type(registry) is Registry and registry.digest==self._registry.digest,
                "plan_registry_changed")
        p=self.inputs()
        require(instant(p["approval_start"])<=now<instant(p["approval_end"]),"plan_approval_expired")
        require(sha(self._raw)==self._pin and profile_digest(self._profile)==p["store_digest"],
                "protected_plan_changed")
        require(registry.verify(self._inv_env,self._inventory,artifact_type="inventory",
                artifact_id="inventory",role="author",parents=[],now=now)==self._inv
                and registry.verify(self._cond_env,self._raw,artifact_type="execution_conditions",
                artifact_id="conditions",role="coordinator",parents=[self._inv],now=now)==self._cond,
                "protected_plan_signature_changed")
        LocalAtomicStore(self._profile,role="executor")

    def bind_ledger(self, ledger):
        p=self.inputs()
        expected={"evaluation_id":p["evaluation_id"],"dataset_sha256":self.dataset_digest,
                  "inventory_sha256":sha(self._inventory),"inventory_receipt":self._inv,
                  "conditions_receipt":self._cond,"conditions":FIXED_CONDITIONS,
                  "blind_map_sha256":p["blind_mapping_commitment"],
                  "slots":[list(x) for x in inventory_slots(self._inventory)]}
        require(type(ledger) is FirstClaimLedger and ledger.lane=="formal-preflight-fixture"
                and ledger._store.profile==self._profile and ledger.claim_record()==expected
                and ledger._root.endswith("/issue620-formal-preflight-fixture/"+cohort_key(self.dataset_digest)),
                "protected_ledger_binding")

    def verify_capture(self, *, ledger, registry, now, record, raw, run_receipt):
        """Expected context comes only from the pinned plan and durable slot."""
        self.reverify(registry,now)
        self.bind_ledger(ledger)
        fields(raw,CAPTURE_FIELDS)
        match(run_receipt,SHA,"run_receipt")
        slot=(record["shard"],record["case_id"],record["arm"])
        p=self.inputs()
        row=next((x for x in p["slots"] if (x["shard"],x["case_id"],x["arm"])==slot),None)
        require(row is not None and record["attempt"]==1,"protected_capture_slot")
        require(sha(raw["binding_raw"])==row["binding_sha256"]
                and decode(raw["binding_raw"])["question"]==self.judge_context_for(slot[1])["question"],
                "protected_binding_pin")
        # A second legacy key owned by the same alias may not bypass new
        # enrollment. The new signature is also verified over the legacy proof.
        legacy=decode(raw["legacy_attestation"])
        expected=next(x for x in decode(registry._raw)["principals"] if x["role"]=="safety_reviewer")
        require(legacy["key_id"]==expected["key_id"],"legacy_safety_signer_mismatch")
        return verify_observation(registry=registry,now=now,evaluation_id=p["evaluation_id"],
            attempt_id=record["attempt_id"],slot=slot,expected_binding_raw=raw["binding_raw"],
            source_pin=row["source_pin"],bundle_pin=row["bundle_pin"],authority_pin=row["authority_pin"],
            legacy_policy_raw=self._policy,legacy_policy_pin=p["legacy_policy_pin"],
            executor_id=p["executor_id"],evaluator_id=p["evaluator_id"],
            expected_parents=[self._inv,self._cond,run_receipt],**raw)

    def verify_partial(self,*,ledger,registry,now,record,raw,run_receipt):
        self.reverify(registry,now)
        self.bind_ledger(ledger)
        fields(raw,PARTIAL_FIELDS)
        slot=(record["shard"],record["case_id"],record["arm"])
        p=self.inputs()
        row=next((x for x in p["slots"] if (x["shard"],x["case_id"],x["arm"])==slot),None)
        require(row is not None and record["attempt"]==1
                and sha(raw["binding_raw"])==row["binding_sha256"]
                and decode(raw["binding_raw"])["question"]==self.judge_context_for(slot[1])["question"],
                "protected_partial_slot")
        match(run_receipt,SHA,"run_receipt")
        return verify_partial_capture(registry=registry,now=now,attempt_id=record["attempt_id"],
                    slot=slot,source_pin=row["source_pin"],parents=[self._inv,self._cond,run_receipt],**raw)

    def judge_context_for(self,case_id):
        context=next(x for x in decode(self._judge_contexts) if x["case_id"]==case_id)
        require(sha(encode(context))==self.targets_for(case_id)["judge_context_digest"],"judge_context_changed")
        return {k:v for k,v in context.items() if k!="case_id"}

    def targets_for(self, case_id):
        return next(x for x in self.inputs()["targets"] if x["case_id"]==case_id)
