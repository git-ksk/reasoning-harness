"""Issue 620 protected offline approval proof graph (no actual authorizations).

The signed wire is checked against the pinned plan and independently supplied
witness snapshot, not against caller declarations. All transitions here are
diagnostic/fixture transitions only. No provider, formal scorer, or release.
"""
from datetime import datetime, timezone, timedelta
import importlib.util
from pathlib import Path
from auth620 import encode, decode, sha, fields, match, SHA, require, instant, formal_entrypoint
from approval_620 import verify_approval, APPROVAL_ROLES
from protected_plan_620 import ProtectedPlan
_BILLING_CHECKER=Path(__file__).resolve().parents[1]/"dev-lite"/"issue531_real24_api_billing_gate.py"


def validate_billing_snapshot(raw,plan,now):
    """Check a signed, minimal account snapshot without querying any provider.

    Actual account/credential provenance is an external witness responsibility.
    No snapshot or its contents is treated as real authorization here.
    """
    p=fields(decode(raw),{"contract","account_pin","plan_digest","checked_at","expires_at",
             "subscription","budget","models"})
    require(p["contract"]=="620-billing-snapshot/1" and p["account_pin"]==plan.inputs()["billing_account_pin"]
            and p["plan_digest"]==plan.digest,"billing_account_binding")
    checked,end=instant(p["checked_at"]),instant(p["expires_at"])
    require(checked<=now<end and end-checked<=MAX_CHECKPOINT_AGE,"billing_stale")
    subscription=fields(p["subscription"],{"payg_enabled"})
    budget=fields(p["budget"],{"currency","api_budget"})
    pool=fields(budget["api_budget"],{"payg_enabled","currency","initial_budget","usage_percentage"})
    require(type(pool["initial_budget"]) is str and type(pool["usage_percentage"]) is str,
            "billing_decimal_wire")
    models=fields(p["models"],{"data"})
    require(type(models["data"]) is list and len(models["data"])<=256,"billing_model_inventory")
    for model in models["data"]:
        fields(model,{"id"})
        require(type(model["id"]) is str,"billing_model_id")
    spec=importlib.util.spec_from_file_location("rh620_pure_billing",_BILLING_CHECKER)
    module=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    result=module.check_account(subscription,budget,models)
    require(result["billing_gate"]=="PASS" and result["max_model_generations_after_admission"]==576
            and result["total_token_upper_bound"]==1348608,"billing_limit_mismatch")
    return p


PROSPECTIVE=("normative_ratification","implementation_acceptance")
MAX_CHECKPOINT_AGE=timedelta(minutes=5)
AUDIT_CODES={"checkpoint_invalid","approval_invalid","graph_invalid","billing_unverified",
             "external_gate_hold","candidate_verified","permission_denied"}
AUDIT_STAGES={"run_preflight","score_preflight","final_preflight","release_preflight"}


def redacted_status(stage, code, *, graph_verified=False):
    # No exception text, paths, subjects, raw data, key material, or Gold.
    require(type(stage) is str and stage in AUDIT_STAGES
            and type(code) is str and code in AUDIT_CODES
            and type(graph_verified) is bool,"invalid_audit_fields")
    return {**formal_entrypoint(),"stage":stage,"reason":code,
            "offline_graph_verified":graph_verified}


class WitnessWindow:
    """Locally enforce independently pinned checkpoint/time constraints.

    A local test witness is NOT independent trust, current revocation proof,
    or rollback protection against the same OS owner. External admission
    must supply these separately; no fallback to system or artifact clocks.
    """
    def __init__(self, *, ledger, registry, expected_pin, expected_sequence,
                 expected_previous, raw, envelope, clock):
        match(expected_pin,SHA,"checkpoint_pin")
        require(type(expected_sequence) is int and expected_sequence>=1,"checkpoint_sequence")
        if expected_previous is not None: match(expected_previous,SHA,"checkpoint_previous")
        require((expected_sequence==1)==(expected_previous is None),"checkpoint_chain")
        require(callable(clock),"trusted_clock_required")
        self._ledger=ledger
        self._registry=registry
        self._clock=clock
        self._pin=expected_pin
        self._sequence=expected_sequence
        self._previous=expected_previous
        self._raw=raw
        self._env=envelope
        self._last_now=None
        self._poisoned=False
        self._receipt=self._validate()
        # Create-only high-water. A replay is not another admitted checkpoint.
        if expected_previous is not None:
            previous=ledger.read_fixture_stage(f"witness-{expected_sequence-1}")
            require(previous is not None and decode(previous)["receipt"]==expected_previous,
                    "checkpoint_previous_missing")
        ledger.append_fixture_stage(f"witness-{expected_sequence}",self._package())

    def _validate(self):
        require(not self._poisoned,"checkpoint_quarantined")
        now=self._clock()
        require(type(now) is datetime and now.tzinfo is not None and now.utcoffset()==timedelta(0),
                "trusted_utc_required")
        require(self._last_now is None or now>=self._last_now,"trusted_clock_rollback")
        self._last_now=now
        p=fields(decode(self._raw),{"contract","evaluation_id","registry_digest",
                "sequence","previous","observed_at","expires_at"})
        require(sha(self._env)==self._pin and p["contract"]=="620-witness/1"
                and p["evaluation_id"]==self._registry.evaluation_id
                and p["registry_digest"]==self._registry.digest
                and type(p["sequence"]) is int and p["sequence"]==self._sequence
                and p["previous"]==self._previous,"checkpoint_binding")
        seen,end=instant(p["observed_at"]),instant(p["expires_at"])
        require(seen<=now<end and end-seen<=MAX_CHECKPOINT_AGE,"checkpoint_stale")
        sig=decode(self._env)
        require(sig["issued_at"]==p["observed_at"] and sig["expires_at"]==p["expires_at"],
                "checkpoint_time_binding")
        return self._registry.verify(self._env,self._raw,artifact_type="checkpoint",
            artifact_id=f"witness-{self._sequence}",role="journal_witness",
            parents=[] if self._previous is None else [self._previous],now=now)

    def _package(self):
        return encode({"payload":decode(self._raw),"envelope":decode(self._env),"receipt":self._pin})

    def recheck(self):
        try:
            require(self._validate()==self._receipt
                    and self._ledger.read_fixture_stage(f"witness-{self._sequence}")==self._package()
                    and self._ledger.read_fixture_stage(f"witness-{self._sequence+1}")==None,
                    "checkpoint_rollback_or_change")
            return self._last_now
        except Exception:
            self._poisoned=True
            raise

    @property
    def digest(self): return self._receipt

    @property
    def validity(self):
        p=decode(self._raw)
        return p["observed_at"],p["expires_at"]


class OfflineApprovalDAG:
    """Formal-shaped wire validation on the isolated offline fixture ledger.

    Prospective receipts precede first claim. Actual proof admission, billing
   /account identity, external trust and normative ratification remain HOLD.
    All writes are synthetic-only and never real permission consumption.
    """
    def __init__(self, *, plan, ledger, registry, witness,
                 identity_raw, identity_envelope, isolation_raw, isolation_envelope):
        require(type(plan) is ProtectedPlan and type(witness) is WitnessWindow,"protected_context")
        self.plan,self.ledger,self.registry,self.witness=plan,ledger,registry,witness
        self._stages={}
        self._started=False
        self._captures=[]
        self._graph=None
        self._billing=None
        self._billing_history=[]
        self._poisoned=False
        self._identity=(identity_raw,identity_envelope)
        self._isolation=(isolation_raw,isolation_envelope)
        now=self._current()
        p=plan.inputs()
        require(sha(identity_raw)==p["identity_evidence_digest"]
                and sha(isolation_raw)==p["isolation_evidence_digest"],"external_evidence_pin")
        # Local synthetic proof receipt authentication, not proof of independence.
        self.identity=registry.verify(identity_envelope,identity_raw,artifact_type="identity_attestation",
                       artifact_id="offline-identity",role="identity_authority",
                       parents=[plan.inventory_receipt],now=now)
        self.isolation=registry.verify(isolation_envelope,isolation_raw,artifact_type="isolation_attestation",
                       artifact_id="offline-isolation",role="isolation_witness",
                       parents=[self.identity,plan.inventory_receipt],now=now)
        self._persist("offline-identity",identity_raw,identity_envelope,self.identity)
        self._persist("offline-isolation",isolation_raw,isolation_envelope,self.isolation)

    def _require_witness_context(self, witness):
        # The protected DAG and checkpoint must share the exact trust snapshot
        # and durable cohort ledger, including at initial admission. A valid
        # signature in a different context is not evidence for this DAG.
        require(type(witness) is WitnessWindow
                and witness._registry is self.registry
                and witness._ledger is self.ledger,"checkpoint_context_mismatch")

    def _current(self):
        require(not self._poisoned,"approval_quarantined")
        try:
            self._require_witness_context(self.witness)
            now=self.witness.recheck()
            self.plan.reverify(self.registry,now)
            self.plan.bind_ledger(self.ledger)
            return now
        except Exception:
            self._poisoned=True
            raise

    def _persist(self,name,raw,env,receipt):
        try:
            self.ledger.append_fixture_stage(name,encode({"payload":decode(raw),"envelope":decode(env),"receipt":receipt}))
        except BaseException:
            self._poisoned=True
            self.ledger.halt()
            raise

    def _same(self,name,raw,env,receipt):
        return self.ledger.read_fixture_stage(name)==encode({"payload":decode(raw),"envelope":decode(env),"receipt":receipt})

    def _recheck(self,*,require_billing_fresh=False):
        now=self._current()
        try:
            for name,kind,role,material,parents,receipt in (
                ("offline-identity","identity_attestation","identity_authority",self._identity,
                     [self.plan.inventory_receipt],self.identity),
                ("offline-isolation","isolation_attestation","isolation_witness",self._isolation,
                     [self.identity,self.plan.inventory_receipt],self.isolation)):
                raw,env=material
                require(self._same(name,raw,env,receipt) and
                    self.registry.verify(env,raw,artifact_type=kind,artifact_id=name,
                        role=role,parents=parents,now=now)==receipt,"identity_or_isolation_changed")
            for raw,env,receipt,stage,parents in self._billing_history:
                p=decode(raw)
                # Historic billing evidence is bound to the approval with its
                # original checkpoint and checked-at; freshness is required at dispatch.
                require(self._same(stage,raw,env,receipt)
                    and self.registry.verify(env,raw,artifact_type="billing_attestation",
                      artifact_id=stage,role="billing_witness",parents=parents,
                      now=instant(p["checked_at"]))==receipt,"billing_snapshot_changed")
            if self._billing is not None and require_billing_fresh:
                raw,env,receipt,stage,parents=self._billing
                validate_billing_snapshot(raw,self.plan,now)
                require(parents[1]==self.witness.digest,"billing_checkpoint_stale")
            for kind,(raw,env,parents,receipt) in self._stages.items():
                require(self._same("approval-"+kind,raw,env,receipt)
                    and verify_approval(registry=self.registry,kind=kind,raw=raw,expected_raw=raw,
                        envelope=env,now=now,parents=parents)["signature_receipt"]==receipt,
                        "approval_receipt_changed")
            return now
        except Exception:
            self._poisoned=True
            raise

    def advance_witness(self,new):
        """Accept only the next independently pinned same-registry checkpoint.

        Old registry changes/clock rollback quarantine the evaluation; no
        substitute registry or retry is allowed. New billing is required for dispatch.
        """
        try:
            require(not self._poisoned,"approval_quarantined")
            self._require_witness_context(self.witness)
            self._require_witness_context(new)
            require(new._sequence==self.witness._sequence+1
                    and new._previous==self.witness.digest
                    and instant(new.validity[0])>=instant(self.witness.validity[0])
                    and new._last_now>=self.witness._last_now,"checkpoint_advance")
            new.recheck()
            self.witness=new
            self._recheck()
        except BaseException:
            self._poisoned=True
            self.ledger.halt()
            raise

    def attach_billing_snapshot(self,raw,envelope):
        now=self._recheck()
        p=validate_billing_snapshot(raw,self.plan,now)
        stage=f"offline-billing-{len(self._billing_history)+1}"
        parents=[self.plan.conditions_receipt,self.witness.digest]
        if self._billing is not None:
            previous=decode(self._billing[0])
            require(instant(p["checked_at"])>instant(previous["checked_at"]),"billing_rollback")
            parents.append(self._billing[2])
        e=decode(envelope)
        require(e["issued_at"]==p["checked_at"] and e["expires_at"]==p["expires_at"],
                "billing_envelope_validity")
        receipt=self.registry.verify(envelope,raw,artifact_type="billing_attestation",
                 artifact_id=stage,role="billing_witness",parents=parents,now=now)
        self._persist(stage,raw,envelope,receipt)
        self._billing=(raw,envelope,receipt,stage,parents)
        self._billing_history.append(self._billing)
        return receipt

    def _receipt(self,kind):
        require(kind in self._stages,"approval_parent_missing")
        return self._stages[kind][3]

    def expected(self,kind,graph=None):
        now=self._recheck(require_billing_fresh=kind=="run_permission")
        require(kind in PROSPECTIVE+("run_permission","score_permission",) and kind not in self._stages,
                "approval_order")
        p=self.plan.inputs()
        parents=[self.plan.inventory_receipt,self.plan.conditions_receipt,
                 p["design_receipt_digest"],self.identity,self.isolation,self.witness.digest,
                 *p["legacy_annotation_receipts"]]
        extra={}
        impl=None
        if kind in PROSPECTIVE:
            require(not self._started and self.ledger.no_reservations(),"prospective_after_reservation")
            if kind=="implementation_acceptance":
                parents.append(self._receipt("normative_ratification"))
        else:
            parents.extend(self._receipt(k) for k in PROSPECTIVE)
            impl=self._receipt("implementation_acceptance")
            require(self._billing is not None,"billing_unverified")
            parents.append(self._billing[2])
            if kind=="run_permission":
                require(not self._started and self.ledger.no_reservations(),"run_after_reservation")
                extra={"billing_evidence_digest":self._billing[2],
                       "schedule_digest":sha(encode(p["slots"]))}
            else:
                require(self._started and self._graph_matches(graph),"score_graph_binding")
                graph.projection()
                parents.extend([self._receipt("run_permission"),graph._closure,graph._lock_receipt,
                                graph._opening_receipt])
                extra={"closure_digest":graph._closure,"lock_digest":graph._lock_receipt,
                       "opening_digest":graph._opening_receipt}
        start=self.witness.validity[0]
        end=p["approval_end"]
        require(instant(p["approval_start"])<=instant(start)<=now<instant(end),"approval_plan_window")
        payload={"evaluation_id":p["evaluation_id"],"inventory_digest":sha(self.plan.inventory_raw),
                 "conditions_digest":self.plan.digest,"contract_digest":p["scoring_contract_digest"],
                 "policy_digest":p["normative_policy_digest"],
                 "formal_implementation_digest":p["formal_implementation_digest"],
                 "design_receipt_digest":p["design_receipt_digest"],
                 "independence_attestation_digest":self.identity,"isolation_attestation_digest":self.isolation,
                 "registry_digest":self.registry.digest,"checkpoint_digest":self.witness.digest,
                 "legacy_annotation_receipts":p["legacy_annotation_receipts"],
                 "implementation_acceptance_digest":impl,"operation":kind,
                 "nonce":"offline-"+kind+"-"+self.plan.digest[:24],"decision":"PERMIT",
                 "not_before":start,"expires_at":end,"parent_receipt_digests":parents,**extra}
        return encode(payload),parents

    def consume(self,kind,raw,envelope,graph=None):
        expected,parents=self.expected(kind,graph)
        result=verify_approval(registry=self.registry,kind=kind,raw=raw,expected_raw=expected,
                              envelope=envelope,now=self._current(),parents=parents)
        receipt=result["signature_receipt"]
        self._persist("approval-"+kind,raw,envelope,receipt)
        self._stages[kind]=(raw,envelope,list(parents),receipt)
        if kind=="run_permission": self._started=True
        # Never return a real operation capability; this receipt is diagnostic.
        return receipt

    def reserve_slot(self,slot):
        try:
            self._recheck(require_billing_fresh=True)
            require(self._started and self._receipt("run_permission") and self._billing is not None,
                    "run_permission_missing")
            return self.ledger.reserve(*slot)
        except BaseException:
            self.ledger.halt()
            raise

    def capture(self,raw):
        try:
            now=self._recheck()
            require(self._started,"run_permission_missing")
            record=self.ledger.active_reservation()
            o=self.plan.verify_capture(ledger=self.ledger,registry=self.registry,now=now,
                         record=record,raw=raw,run_receipt=self._receipt("run_permission"))
            self.ledger.record(attempt_id=o["attempt_id"],status=o["status"],calls=o["calls"],
                tokens=o["tokens"],admission_sha256=o["admission_sha256"],
                final_sha256=o["final_sha256"],observation_sha256=o["observation_sha256"])
            self._captures.append(dict(raw))
            return o
        except BaseException:
            self.ledger.halt()
            raise

    def capture_partial(self,raw):
        try:
            now=self._recheck()
            require(self._started,"run_permission_missing")
            record=self.ledger.active_reservation()
            o=self.plan.verify_partial(ledger=self.ledger,registry=self.registry,now=now,
                record=record,raw=raw,run_receipt=self._receipt("run_permission"))
            try:
                self.ledger.record(attempt_id=o["attempt_id"],status=o["status"],calls=None,tokens=None,
                    admission_sha256=None,final_sha256=None,observation_sha256=o["observation_sha256"])
            except ValueError as error:
                require(str(error)=="first_attempt_incomplete","partial_record_failed")
            saved=self.ledger.stopped_observation()
            require(saved["observation_sha256"]==o["observation_sha256"],"partial_record_failed")
            return o
        except BaseException:
            self.ledger.halt()
            raise

    def build_graph(self,mapping,salt):
        """Only this coordinator builds its evidence graph from captured bytes."""
        from blind_fixture_620 import BlindFixtureGraph
        try:
            now=self._recheck()
            require(self._graph is None and len(self._captures)==48,"graph_build_order")
            closure=self.ledger.close()
            self._graph=BlindFixtureGraph(ledger=self.ledger,registry=self.registry,plan=self.plan,
                 run_receipt=self._receipt("run_permission"),closure_digest=closure,
                 observation_inputs=list(self._captures),mapping=mapping,salt=salt,now=now,clock=self._current)
            return self._graph
        except BaseException:
            self.ledger.halt()
            raise

    def _graph_matches(self,graph):
        from blind_fixture_620 import BlindFixtureGraph
        return (type(graph) is BlindFixtureGraph and graph is self._graph and graph._ledger is self.ledger
                and graph._plan is self.plan and graph._registry is self.registry
                and graph._run_receipt==self._receipt("run_permission"))

    def inspect_run(self):
        try:
            self._recheck(require_billing_fresh=True)
            self._receipt("run_permission")
            require(self._started and self._billing is not None and self.ledger.no_reservations(),
                    "run_preflight_order")
            return redacted_status("run_preflight","external_gate_hold",graph_verified=True)
        except Exception:
            return redacted_status("run_preflight","approval_invalid")

    def inspect_score(self,graph):
        """Verifier-owned projection on the offline wire, never evaluates/formal scores."""
        try:
            self._recheck()
            require(self._graph_matches(graph),"score_graph_binding")
            self._receipt("score_permission")
            inventory,rows=graph.projection()
            p=decode(self._stages["score_permission"][0])
            require((p["closure_digest"],p["lock_digest"],p["opening_digest"])==
                    (graph._closure,graph._lock_receipt,graph._opening_receipt)
                    and len(inventory)==24 and len(rows)==48,"score_projection_binding")
            # The projection is used internally; no actual output/Gold/metrics.
            return redacted_status("score_preflight","external_gate_hold",graph_verified=True)
        except Exception:
            return redacted_status("score_preflight","graph_invalid")
