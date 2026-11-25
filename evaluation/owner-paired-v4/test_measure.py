import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import measure as m


def attempt(call="g1-1",status=200,number=1):
    rows=[{"event":"attempt_start"},{"event":"http_headers","status_code":status},
          {"event":"http_response","status_code":status}]
    return "".join(json.dumps(dict(r,schema_version="reason-google-attempt-telemetry-v1",
                                    model=m.MODEL,call_id=call,attempt=number))+"\n" for r in rows)

def envelope(i,o,a=1,calls=1,cls=None,msg="",diag=None):
    report={"run":{"tracked_from_session_start":True,"input_tokens":i,"output_tokens":o,
                   "total_tokens":i+o if i is not None and o is not None else None,
                   "provider_attempts":a,"model_calls":calls},"budget":m.CAPS}
    if cls:
        failure={"failure_class":cls,"message":msg}
        if diag: failure["model_output_diagnostic"]=diag
        result={"status":"failed","failure":failure,"partial_diagnostics":{"usage":report}}
    else:
        result={"status":"succeeded","usage":report,"finalization":{"text":"synthetic","status":"qualified_partial_answer"}}
    return json.dumps({"schema_version":"reason-cli-output-v1","command":"ask","result":result}).encode()

class Measurement(unittest.TestCase):
    def check(self,raw,log,rc,expected):
        obs={"status":"incomplete"}
        m.terminal_observation(raw,log.encode(),rc,obs)
        self.assertEqual(obs["status"],expected)
        return obs

    def test_success_and_structured_fallback_counted(self):
        obs=self.check(envelope(4,3,2),attempt()+attempt("g1-2"),0,"complete")
        self.assertEqual(obs["telemetry"]["actual_total_tokens"],7)
        self.assertEqual(obs["telemetry"]["http_attempts"],2)

    def test_unknown_usage_failed_http_missing_and_retry_stop(self):
        for raw,log in ((envelope(None,3),attempt()),(envelope(4,3),attempt(status=429)),
                        (envelope(4,3),""),(envelope(4,3),attempt(number=2)),
                        (envelope(4,3),attempt()+attempt("g1-2")),
                        (envelope(4,3),attempt().replace(m.MODEL,"wrong"))):
            with self.assertRaises((ValueError,KeyError)):
                self.check(raw,log,0,"complete")

    def test_budget_exact_and_overrun(self):
        log="".join(attempt("g1-"+str(i)) for i in range(8))
        self.check(envelope(40,8192,8,8,cls="usage_budget_exceeded",msg="output-token budget exhausted (8192/8192)"),log,1,"budget_exhausted")
        with self.assertRaises(ValueError):
            self.check(envelope(40,8193,8,8),log,0,"complete")

    def test_local_decode_failure_is_terminal_only_if_accounted(self):
        raw=envelope(4,3,cls="protocol",diag={"origin":"local_model_output_decode","phase":"initial_candidate"})
        self.check(raw,attempt(),1,"local_decode_failed")
        with self.assertRaises(ValueError):self.check(raw,"",1,"local_decode_failed")

    def test_global_failures_stop(self):
        for cls in ("quota","rate_limit","credentials","transport","timeout","protocol"):
            with self.assertRaises(ValueError):
                self.check(envelope(4,3,cls=cls),attempt(),1,"complete")

    def test_key_binding_refuses_wrong_key_before_network(self):
        with patch.dict(os.environ,{"ENGINE620_GEMINI_KEY_SHA256":"a"*64}),patch.object(m.urllib.request,"build_opener") as op:
            with self.assertRaisesRegex(ValueError,"billing_project_key_mismatch"):m.billing("wrong")
            op.assert_not_called()

    def test_all_48_slots_unique_and_selected_scope(self):
        rows=[m.position(s,t) for s in range(8) for t in range(6)]
        self.assertEqual(len({r[2] for r in rows}),48)
        self.assertEqual({m.position(s,t)[2] for s in m.SHARDS for t in range(6)},m.SLOTS)
        self.assertEqual(sum(m.position(s,t)[1]=="baseline" for s in range(8) for t in (0,2,4)),12)

class Claims(unittest.TestCase):
    def test_platform_migration_requires_cancelled_unstarted_predecessor(self):
        prior={"status":"completed","conclusion":"cancelled","head_sha":"b42673fd530c04b21f0391f301e4fe0dfae3d849","run_attempt":1}
        with patch.object(m,"api",side_effect=[prior,{"total_count":0}]),patch.object(m,"matching",return_value=[]):
            m.verify_unstarted_macos_predecessor()
        with patch.object(m,"api",return_value=prior),patch.object(m,"matching",return_value=[{"claimed":True}]):
            with self.assertRaisesRegex(ValueError,"macos_predecessor_observations_exist"):
                m.verify_unstarted_macos_predecessor()
        with patch.object(m,"api",return_value=dict(prior,status="in_progress")):
            with self.assertRaisesRegex(ValueError,"macos_predecessor_not_cancelled"):
                m.verify_unstarted_macos_predecessor()

    C="a"*64
    HEAD="b"*40

    def test_no_unsealed_claim_may_resume(self):
        _,prefix,seals,halt=m.names(self.C)
        with patch.object(m,"exact_ref",return_value=None),patch.object(m,"matching",side_effect=lambda p:{prefix+"03-baseline":self.HEAD} if p==prefix else {}):
            with self.assertRaisesRegex(ValueError,"claimed_slot_not_uniquely_sealed"):
                m.verify_existing(self.C,self.HEAD)

    def test_halted_cohort_refuses_resume(self):
        with patch.object(m,"exact_ref",return_value=self.HEAD):
            with self.assertRaisesRegex(ValueError,"cohort_halted"):
                m.verify_existing(self.C,self.HEAD)

    def test_sealed_slot_excluded_and_unclaimed_slots_available(self):
        _,prefix,seals,_=m.names(self.C)
        name=seals+"03-baseline-123-456-"+"d"*64+"-"+"e"*64
        with patch.object(m,"exact_ref",return_value=None),patch.object(m,"matching",side_effect=lambda p:{prefix+"03-baseline":self.HEAD} if p==prefix else {name:self.HEAD}),patch.object(m,"check_artifact") as check:
            self.assertEqual(m.verify_existing(self.C,self.HEAD),{"03-baseline"})
            self.assertEqual(len(m.SLOTS-{"03-baseline"}),17)
            check.assert_called_once_with(self.C,"03-baseline",123,456,"e"*64)

    def test_claim_duplicate_is_not_reexecuted(self):
        record=({}, {}, self.C, 123, 0, "baseline", "03-baseline", {})
        with patch.object(m,"context",return_value=(self.HEAD,124)),patch.object(m,"pins",return_value=record),patch.object(m,"exact_ref",side_effect=[self.HEAD,None,self.HEAD]),patch.object(m,"verify_seal") as verify,patch.object(m,"create_ref") as create,patch.object(m,"output") as out:
            m.claim(Path("unused"),Path("unused"),"resume",0,0)
            create.assert_not_called()
            verify.assert_called_once()
            out.assert_called_once_with("run","false")

    def test_initial_claim_duplicate_fails_closed(self):
        record=({}, {}, self.C, None, 0, "baseline", "03-baseline", {})
        with patch.object(m,"context",return_value=(self.HEAD,124)),patch.object(m,"pins",return_value=record),patch.object(m,"exact_ref",side_effect=[self.HEAD,None,self.HEAD]),patch.object(m,"create_ref") as create:
            with self.assertRaisesRegex(ValueError,"duplicate_slot"):
                m.claim(Path("unused"),Path("unused"),"initial",0,0)
            create.assert_not_called()

    def test_full_ref_length_guard_and_selected_scope(self):
        name=m.names(self.C)[2]+"03-baseline-12345678901-14943095748-"+"d"*64+"-"+"e"*64
        self.assertLessEqual(len(("refs/tags/"+name).encode("utf-8")),255)
        with patch.object(m,"api") as api:
            with self.assertRaisesRegex(ValueError,"github_ref_length"):
                m.create_ref("a"*246,self.HEAD)
            api.assert_not_called()
        self.assertEqual(len(m.SLOTS),18)
        self.assertEqual(m.SHARDS,(1,2,4))
        self.assertNotIn("00-baseline",m.SLOTS)

    def test_encrypt_only_evidence_no_home(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            slot=root/"raw"/"00-baseline";slot.mkdir(parents=True)
            m.save(slot/"reservation.json",{"synthetic":True})
            m.save(slot/"stdout.json",b"synthetic")
            (slot/"home").mkdir()
            m.save(slot/"home"/"not-evidence",b"not evidence")
            m.encrypt(root,0,0)
            self.assertGreater((root/"00-baseline.cms").stat().st_size,0)
            with self.assertRaises(FileExistsError):
                m.encrypt(root,0,0)


if __name__=="__main__":
    unittest.main()
