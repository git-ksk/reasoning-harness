"""Public synthetic-only vectors; no actual evaluation data or key material."""
import copy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import threading
import unittest
from unittest.mock import patch

from nacl.signing import SigningKey
from auth620 import (Registry, encode, decode, sha, b64encode, b64decode, DOMAIN,
                     VERSION, formal_entrypoint, ReviewAuthentication)
from first_claim_620 import FirstClaimLedger, inventory_slots, cohort_key
from local_store import initialize_local_store

NOW = datetime(2026, 1, 1, 0, 0, 10, tzinfo=timezone.utc)
EVAL = "synthetic-620-fixed"
START = "2026-01-01T00:00:00Z"
END = "2026-01-01T00:01:00Z"
ROLES = ("author", "coordinator", "safety_reviewer", "capture", "judge_a", "judge_b",
         "adjudicator", "adjudication_ratifier", "blind_lock_admin", "unblinder",
         "normative_ratifier", "implementation_acceptor", "run_approver", "score_approver",
         "executor", "journal_witness", "identity_authority", "isolation_witness", "billing_witness")


def h(text):
    return sha(text.encode())


def fixture_registry():
    # Public deterministic seeds for offline tests only, NEVER enrolled or trusted
    # for real operations. The keys must never be used for production.
    keys = {r: SigningKey(hashlib.sha256(("PUBLIC SYNTHETIC 620 " + r).encode()).digest()) for r in ROLES}
    data = {"protocol_version": VERSION, "evaluation_id": EVAL, "principals": [
        {"subject_id": "subject-" + r, "aliases": ["alias-" + r], "role": r,
         "conflict_groups": ["conflict-" + r], "key_id": "key-" + r,
         "public_key": b64encode(keys[r].verify_key.encode()), "not_before": START,
         "expires_at": END, "revoked": False} for r in keys]}
    return data, keys


def sign(reg, keys, payload, *, kind, role, artifact_id, parents, **overrides):
    value = {"schema_version": "1", "artifact_type": kind, "artifact_id": artifact_id,
             "evaluation_id": EVAL, "protocol_version": VERSION, "signer_id": "subject-" + role,
             "key_id": "key-" + role, "signature_algorithm": "Ed25519", "issued_at": START,
             "expires_at": END, "payload_digest": sha(payload), "parent_receipt_digests": parents,
             "registry_digest": reg.digest, "scope": {"issue": 620, "evaluation_id": EVAL, "operation": kind}}
    value.update(overrides)
    return encode({**value, "signature": b64encode(keys[role].sign(DOMAIN + encode(value)).signature)})


def fixture_inventory():
    return encode([{"case_id": f"synthetic-620-{i:02}", "case_type": "positive" if i < 16 else "negative",
                    "fact_count": 2} for i in range(24)])


class CanonicalTests(unittest.TestCase):
    def test_exact_unicode_and_integer_vector(self):
        expected = b'{"a":[0,9007199254740991,true,null],"z":"e\xcc\x81 \xe9\xa3\x9f"}'
        value = {"z": "e\u0301 食", "a": [0, 9007199254740991, True, None]}
        self.assertEqual(encode(value), expected)
        self.assertEqual(decode(expected), value)
        self.assertNotEqual(encode({"z": "\u00e9"}), encode({"z": "e\u0301"}))

    def test_reject_recursive_duplicate_and_noncanonical_numbers(self):
        for raw in (b'{"x":{"a":1,"\\u0061":2}}', b'{"a":-0}', b'{"a":1e0}', b'{"a":1.0}',
                    b'{"a":-1}', b'{"a":9007199254740992}', b'{"a":NaN}', b'{"a":Infinity}',
                    b'{"a":01}', b'{"a": 1}', b'{"a":1}\n', b'\xef\xbb\xbf{}',
                    b'{"a":"\\ud800"}', b'{"a":"\xff"}', '{"食":1}'.encode(),
                    b'[' * 34 + b'null' + b']' * 34, b' ' * (4*1024*1024+1)):
            with self.subTest(raw=raw[:48]), self.assertRaises(ValueError):
                decode(raw)

    def test_canonical_b64(self):
        self.assertEqual(b64decode("AA", 1), b"\0")
        for bad in ("AB", "AA=", "A+", "A/", " A"):
            with self.assertRaises(ValueError):
                b64decode(bad, 1)

    def test_rfc8032_public_known_vector(self):
        pk = bytes.fromhex("d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a")
        sig = bytes.fromhex("e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e06522490155"
                            "5fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b")
        ReviewAuthentication(b"", b"")._verify_signature(pk, sig, b"")
        for key in (b"\0"*32, b"\1"+b"\0"*31, b"\xff"*32):
            with self.assertRaises(ValueError):
                ReviewAuthentication(b"", b"")._verify_signature(key, sig, b"")


class RegistryTests(unittest.TestCase):
    def setUp(self):
        self.data, self.keys = fixture_registry()
        self.raw = encode(self.data)
        self.reg = Registry(self.raw, pinned_sha256=sha(self.raw), evaluation_id=EVAL)
        self.payload = encode({"test": "fixed fixture"})
        self.args = dict(artifact_type="inventory", artifact_id="inventory-1", role="author", parents=[], now=NOW)

    def signed(self, **changes):
        return sign(self.reg, self.keys, self.payload, kind="inventory", role="author",
                    artifact_id=changes.pop("artifact_id", "inventory-1"), parents=[], **changes)

    def test_exact_verified_signature_and_repeat_read_no_permission(self):
        raw = self.signed()
        self.assertEqual(self.reg.verify(raw, self.payload, **self.args), sha(raw))
        self.assertEqual(self.reg.verify(raw, self.payload, **self.args), sha(raw))
        self.assertFalse(formal_entrypoint(verified=sha(raw))["formal_score_authorized"])

    def test_signed_wrong_scope_time_parent_version_and_payload(self):
        for change in ({"evaluation_id":"foreign"}, {"protocol_version":"legacy"},
                       {"scope":{"issue":True,"evaluation_id":EVAL,"operation":"inventory"}},
                       {"issued_at":"2026-01-01T00:00:11Z"}, {"expires_at":"2026-01-01T00:00:10Z"},
                       {"signer_id":"alias-author"}, {"key_id":"missing"}, {"key_id":"key-judge_a"},
                       {"parent_receipt_digests":[h("wrong")]}, {"registry_digest":h("old")},
                       {"artifact_id":"wrong"}, {"payload_digest":h("bad")}, {"unrecognized":True}):
            with self.subTest(change=change), self.assertRaises(ValueError):
                self.reg.verify(self.signed(**change), self.payload, **self.args)
        with self.assertRaises(ValueError):
            self.reg.verify(self.signed(), encode({"test":"other"}), **self.args)
        with self.assertRaises(ValueError):
            self.reg.verify(self.signed(), self.payload, **{**self.args,"role":"judge_a"})

    def test_tampered_signature_and_old_domain(self):
        data = decode(self.signed())
        data["signature"] = b64encode(b"\0"*64)
        with self.assertRaises(ValueError):
            self.reg.verify(encode(data), self.payload, **self.args)
        data.pop("signature")
        data["signature"] = b64encode(self.keys["author"].sign(b"legacy\0" + encode(data)).signature)
        with self.assertRaises(ValueError):
            self.reg.verify(encode(data), self.payload, **self.args)

    def test_trust_pin_duplicate_identity_key_conflict_and_revocation(self):
        for change in (
            lambda p: p[1].update(public_key=p[0]["public_key"]),
            lambda p: p[1].update(aliases=[p[0]["subject_id"]]),
            lambda p: p[1].update(role=p[0]["role"]),
            lambda p: p[1].update(conflict_groups=p[0]["conflict_groups"]),
            lambda p: p[1].update(public_key=b64encode(b"\0"*32)),
            lambda p: p[1].update(revoked=0)):
            data = copy.deepcopy(self.data); change(data["principals"])
            raw=encode(data)
            with self.assertRaises(ValueError):
                Registry(raw, pinned_sha256=sha(raw), evaluation_id=EVAL)
        data=copy.deepcopy(self.data); data["principals"][0]["revoked"]=True
        raw=encode(data)
        with self.assertRaises(ValueError):
            Registry(raw, pinned_sha256=sha(self.raw), evaluation_id=EVAL)
        reg=Registry(raw,pinned_sha256=sha(raw),evaluation_id=EVAL)
        signed=sign(reg,self.keys,self.payload,kind="inventory",role="author",artifact_id="inventory-1",parents=[])
        with self.assertRaises(ValueError):
            reg.verify(signed,self.payload,**self.args)


class LedgerTests(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.profile=initialize_local_store(Path(self.tmp.name).resolve()/"store")
        self.inventory=fixture_inventory()
        self.args=dict(dataset_sha256=h("fixed invented dataset"), inventory_raw=self.inventory,
                       inventory_receipt=h("inventory"),conditions_receipt=h("conditions"),evaluation_id=EVAL,blind_map_sha256=h("map"))

    def create(self, **changes):
        return FirstClaimLedger(self.profile, **{**self.args, **changes})

    def record(self, ledger, slot, **changes):
        attempt=ledger.reserve(*slot)
        ledger.record(**{**dict(attempt_id=attempt,status="complete",calls=1,tokens=5,
                     admission_sha256=h("admission"+attempt), final_sha256=h("identical content allowed"),
                     observation_sha256=h("observation"+attempt)),**changes})
        return attempt

    def test_full_8_48_closure_and_no_relabel_or_reset(self):
        ledger=self.create()
        for slot in inventory_slots(self.inventory):
            self.record(ledger,slot)
        self.assertEqual(len(ledger.close()),64)
        with self.assertRaises(ValueError):
            ledger.close()
        with self.assertRaises(FileExistsError):
            self.create(evaluation_id="foreign-rename")
        self.assertEqual(len(cohort_key(self.args["dataset_sha256"])),64)

    def test_incomplete_and_bad_shard_stop(self):
        ledger=self.create()
        with self.assertRaises(ValueError):
            ledger.reserve(2, "synthetic-620-00", "baseline")
        with self.assertRaises(ValueError):
            ledger.reserve(*inventory_slots(self.inventory)[0])

    def test_cancel_and_overbudget_stop(self):
        for index, overrides in enumerate(({"status":"cancelled","calls":None,"tokens":None,"final_sha256":None},
                                           {"calls":13},{"tokens":28097},{"calls":True})):
            l=self.create(dataset_sha256=h("separate synthetic probe "+str(index)))
            with self.assertRaises(ValueError):
                self.record(l,inventory_slots(self.inventory)[0],**overrides)
            with self.assertRaises(ValueError): l.reserve(*inventory_slots(self.inventory)[1])

    def test_unknown_usage_no_replacement_and_missing_final_preserved(self):
        ledger=self.create()
        with self.assertRaises(ValueError):
            self.record(ledger,inventory_slots(self.inventory)[0],status="timeout",calls=None,tokens=None,
                        final_sha256=None)
        with self.assertRaises(ValueError):
            ledger.close()
        with self.assertRaises(FileExistsError):
            self.create()

    def test_simultaneous_global_claim_only_one(self):
        barrier=threading.Barrier(2)
        outcomes=[]
        def claim():
            barrier.wait()
            try:
                self.create()
                outcomes.append("won")
            except FileExistsError:
                outcomes.append("lost")
        ts=[threading.Thread(target=claim) for _ in range(2)]
        for t in ts: t.start()
        for t in ts: t.join()
        self.assertCountEqual(outcomes, ["won","lost"])

    def test_simultaneous_fixture_nonce_only_one(self):
        ledger=self.create(); barrier=threading.Barrier(2); outcomes=[]
        def consume():
            barrier.wait()
            try:
                ledger.append_fixture_stage("fixed-synthetic-nonce",encode({"purpose":"fixture"}))
                outcomes.append("won")
            except FileExistsError:
                outcomes.append("lost")
        children=[threading.Thread(target=consume) for _ in range(2)]
        for t in children: t.start()
        for t in children: t.join()
        self.assertCountEqual(outcomes,["won","lost"])

    def test_killed_after_reservation_no_reopen(self):
        args={**self.args,"inventory_raw":self.inventory.decode()}
        child=subprocess.Popen([sys.executable,"-B","-c",
            "import json,sys,time;from pathlib import Path;sys.path.insert(0,sys.argv[1]);"
            "from auth620 import encode;from local_store import LocalStoreProfile;"
            "from first_claim_620 import FirstClaimLedger,inventory_slots;"
            "a=json.loads(sys.argv[3]);a['inventory_raw']=a['inventory_raw'].encode();"
            "p=LocalStoreProfile.from_json(json.loads(sys.argv[2]));l=FirstClaimLedger(p,**a);"
            "l.reserve(*inventory_slots(a['inventory_raw'])[0]);print('inflight',flush=True);time.sleep(60)",
            str(Path(__file__).resolve().parent),json.dumps(self.profile.to_json()),json.dumps(args)],
            stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
        try:
            self.assertEqual(child.stdout.readline().strip(),"inflight")
        finally:
            child.kill(); child.communicate(timeout=5)
        with self.assertRaises(FileExistsError):
            self.create()

    def test_inventory_count_usage_cap_and_ambiguous_write(self):
        bad=decode(self.inventory); bad.pop()
        with self.assertRaises(ValueError): inventory_slots(encode(bad))
        bad=decode(self.inventory); bad[0]["fact_count"]=True
        with self.assertRaises(ValueError): inventory_slots(encode(bad))
        ledger=self.create()
        original=ledger._store.create
        def uncertain(key,raw):
            original(key,raw)
            if key.endswith("-inflight.json"):
                raise OSError("synthetic ambiguous acknowledgement")
        with patch.object(ledger._store,"create",side_effect=uncertain):
            with self.assertRaises(OSError): ledger.reserve(*inventory_slots(self.inventory)[0])
        with self.assertRaises(ValueError): ledger.reserve(*inventory_slots(self.inventory)[0])
        with self.assertRaises(FileExistsError): self.create()

    def test_duplicate_observation_and_caps(self):
        l=self.create()
        slots=inventory_slots(self.inventory)
        self.record(l,slots[0],observation_sha256=h("duplicate"))
        with self.assertRaises(ValueError):
            self.record(l,slots[1],observation_sha256=h("duplicate"))
        with self.assertRaises(ValueError):
            l.close()


class ApprovalWireTests(unittest.TestCase):
    def test_signed_future_wire_is_not_authority(self):
        from approval_620 import verify_approval,DIGESTS
        data,keys=fixture_registry(); raw=encode(data)
        reg=Registry(raw,pinned_sha256=sha(raw),evaluation_id=EVAL)
        kind="run_permission"; role="run_approver"; parents=[h("synthetic previous")]
        p={k:h("synthetic "+k) for k in DIGESTS}
        p.update(evaluation_id=EVAL,registry_digest=reg.digest,
                 legacy_annotation_receipts=[h("synthetic annotation "+str(i)) for i in range(3)],
                 implementation_acceptance_digest=h("synthetic accept"),
                 operation=kind,nonce="wire-run-once",decision="PERMIT",
                 not_before=START,expires_at=END,parent_receipt_digests=parents,
                 billing_evidence_digest=h("synthetic billing"),schedule_digest=h("synthetic schedule"))
        def verify(value, expected=None):
            payload=encode(value)
            env=sign(reg,keys,payload,kind=kind,role=role,artifact_id=p["nonce"],parents=parents)
            return verify_approval(registry=reg,kind=kind,raw=payload,expected_raw=encode(expected or p),
                                   envelope=env,now=NOW,parents=parents)
        out=verify(p)
        self.assertEqual(out["approval_status"],"HOLD")
        self.assertFalse(out["live_authorized"])
        self.assertFalse(out["formal_score_authorized"])
        for change in ({"billing_evidence_digest":h("foreign")},{"operation":"score_permission"},
                       {"legacy_annotation_receipts":[h("only one")]},{"unknown":True}):
            with self.assertRaises(ValueError): verify({**p,**change})

    def test_expired_fixture_run_rejected_at_slot_reservation(self):
        from fixture_permissions_620 import FixturePermissions
        from datetime import timedelta
        data,keys=fixture_registry(); r=encode(data)
        reg=Registry(r,pinned_sha256=sha(r),evaluation_id=EVAL)
        with tempfile.TemporaryDirectory() as tmp:
            profile=initialize_local_store(Path(tmp).resolve()/"store")
            inv=fixture_inventory()
            l=FirstClaimLedger(profile,dataset_sha256=sha(inv),inventory_raw=inv,
                inventory_receipt=h("synthetic inventory"),conditions_receipt=h("synthetic conditions"),
                evaluation_id=EVAL,blind_map_sha256=h("synthetic map"))
            f=FixturePermissions(l,reg,h("synthetic inventory"),h("synthetic conditions"),NOW)
            for kind,role in (("normative_ratification","normative_ratifier"),
                              ("implementation_acceptance","implementation_acceptor")):
                raw=f.expected_prospective(kind)
                f.prospective(kind,raw,sign(reg,keys,raw,kind=kind,role=role,artifact_id=kind,parents=f.parents))
            f.attach_fixture_billing()
            p,parents=f.expected_run()
            f.consume_run(p,sign(reg,keys,p,kind="run_permission",role="run_approver",
                                 artifact_id="fixture-run-once",parents=parents))
            f.now=NOW+timedelta(minutes=1)
            with self.assertRaises(ValueError): f.reserve_slot(*inventory_slots(inv)[0])
            with self.assertRaises(ValueError): l.reserve(*inventory_slots(inv)[0])

    def test_missing_final_partial_capture_diagnostic_only(self):
        from observation_620 import verify_partial_capture
        from native_admission import CATALOG
        data,keys=fixture_registry(); raw=encode(data)
        reg=Registry(raw,pinned_sha256=sha(raw),evaluation_id=EVAL)
        attempt=h("invented incomplete attempt")
        source=encode({"contract":CATALOG,"sources":{},"evidence_bindings":{}})
        stdout=b"invented partial adverse statement"
        slot=(1,"synthetic-620-00","candidate")
        p={"attempt_id":attempt,"slot":list(slot),"status":"timeout",
           "stdout_sha256":sha(stdout),"stderr_sha256":sha(b""),"journal_sha256":sha(b"partial"),
           "trace_sha256":sha(b"partial trace"),"sources_sha256":sha(source),
           "binding_sha256":sha(b"fixture binding"),"display_span":None,
           "partial_display_sha256":None,"final_sha256":None,"calls":None,"tokens":None,
           "formal_eligible":False}
        parents=[h("fixture parent")]
        envelope=sign(reg,keys,encode(p),kind="raw_evidence",role="capture",
                      artifact_id="partial-"+attempt,parents=parents)
        kw=dict(registry=reg,now=NOW,attempt_id=attempt,slot=slot,status="timeout",
                stdout=stdout,stderr=b"",journal_raw=b"partial",trace_raw=b"partial trace",
                sources_raw=source,source_pin=sha(source),final_bytes=None,binding_raw=b"fixture binding",
                display_span=None,envelope=envelope,parents=parents)
        out=verify_partial_capture(**kw)
        self.assertIsNone(out["final_sha256"])
        self.assertIsNone(out["tokens"])
        self.assertTrue(out["diagnostic_only"])
        self.assertFalse(out["formal_eligible"])
        with self.assertRaises(ValueError): verify_partial_capture(**{**kw,"stdout":b"changed"})

if __name__=="__main__":
    unittest.main()
