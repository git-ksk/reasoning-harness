import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import measure as m


def attempt(call="g1-1",status=200,number=1,i=4,o=3,ri=600,ro=8,state="reported",reason="retry",wait=30000):
    base={"model":m.MODEL,"call_id":call,"attempt":number}
    rows=[dict(base,schema_version="reason-google-attempt-telemetry-v1",event="attempt_start"),
          dict(base,schema_version="reason-google-attempt-telemetry-v1",event="http_headers",status_code=status),
          dict(base,schema_version="reason-google-attempt-telemetry-v1",event="http_response",status_code=status,
               error_class="provider_unavailable" if status==503 else None)]
    a=dict(base,schema_version="reason-google-accounted-v1")
    rows.append(dict(a,event="accounted_attempt",status_code=status,usage=state,charged_input=i,charged_output=o,
                     charged_total=i+o,reserved_input=ri,reserved_output=ro))
    if status==503:
        rows.append(dict(a,event="retry_decision",reason=reason,wait_ms=wait if reason=="retry" else 0,elapsed_ms=0))
        if reason=="retry":rows.append(dict(a,event="retry_wait",reason=reason,wait_ms=wait,elapsed_ms=wait+1))
    return "".join(json.dumps(r)+"\n" for r in rows)

def envelope(i,o,a=1,calls=1,cls=None,msg="",diag=None):
    report={"run":{"tracked_from_session_start":True,"input_tokens":i,"output_tokens":o,
                   "total_tokens":i+o if i is not None and o is not None else None,
                   "provider_attempts":a,"model_calls":calls},"budget":m.CAPS,"token_accounting":m.ACCOUNTING}
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
        obs=self.check(envelope(8,6,2),attempt()+attempt("g1-2"),0,"complete")
        self.assertEqual(obs["telemetry"]["actual_total_tokens"],14)
        self.assertEqual(obs["telemetry"]["http_attempts"],2)

    def test_503_recovery_reports_reservation_as_unknown(self):
        log=attempt(status=503,i=600,o=8,state="reserved")+attempt(number=2)
        obs=self.check(envelope(604,11,2),log,0,"complete")
        self.assertIsNone(obs["telemetry"]["actual_total_tokens"])
        self.assertEqual(obs["telemetry"]["reported_tokens"]["total"],7)
        self.assertEqual(obs["telemetry"]["reserved_unknown_tokens"]["total"],608)
        self.assertEqual(obs["telemetry"]["retries"],1)
        self.assertEqual(obs["telemetry"]["retry_wait_ms"],30001)

    def test_exact_reported_503_recovery(self):
        log=attempt(status=503,i=3,o=0)+attempt(number=2)
        obs=self.check(envelope(7,3,2),log,0,"complete")
        self.assertEqual(obs["telemetry"]["actual_total_tokens"],10)

    def test_max_three_retries_and_final_success(self):
        log="".join(attempt(status=503,number=i,i=600,o=8,state="reserved",wait=w) for i,w in ((1,30000),(2,60000),(3,120000)))+attempt(number=4)
        obs=self.check(envelope(1804,27,4),log,0,"complete")
        self.assertEqual(obs["telemetry"]["retries"],3)
        self.assertEqual(obs["telemetry"]["retry_wait_ms"],210003)

    def test_terminal_503_never_scored_and_no_post_terminal_call(self):
        log="".join(attempt(status=503,number=i,i=600,o=8,state="reserved",wait=w) for i,w in ((1,30000),(2,60000),(3,120000)))+attempt(status=503,number=4,i=600,o=8,state="reserved",reason="attempt_limit")
        raw=envelope(2400,32,4,cls="provider_unavailable",msg="Google HTTP 503; retry_decision=attempt_limit")
        with self.assertRaisesRegex(ValueError,"terminal_503_attempt_limit"):
            self.check(raw,log,1,"complete")
        with self.assertRaisesRegex(ValueError,"concealed_terminal_503"):
            self.check(envelope(2400,32,4),log,0,"complete")
        with self.assertRaises(ValueError):
            self.check(envelope(2404,35,5),log+attempt("g1-2"),0,"complete")

    def test_unknown_usage_failed_http_missing_and_retry_stop(self):
        valid=attempt(status=503,i=600,o=8,state="reserved")+attempt(number=2)
        for raw,log in ((envelope(None,3),attempt()),(envelope(4,3),attempt(status=429)),
                        (envelope(4,3),""),(envelope(4,3),attempt(number=2)),
                        (envelope(4,3),attempt()+attempt("g1-2")),
                        (envelope(4,3),attempt().replace(m.MODEL,"wrong")),
                        (envelope(604,11,2),valid.replace('"charged_total": 608','"charged_total": 0')),
                        (envelope(604,11,2),valid.replace('"wait_ms": 30000','"wait_ms": 0')),
                        (envelope(604,11,2),valid.replace('"event": "retry_wait"','"event": "invalid"')),
                        (envelope(604,11,2),valid.replace('"reserved_input": 600','"reserved_input": 599',1)),
                        (envelope(604,11,2),valid.replace('"usage": "reserved"','"usage": "unknown"'))):
            with self.assertRaises((ValueError,KeyError)):
                self.check(raw,log,0,"complete")

    def test_budget_exact_and_overrun(self):
        log="".join(attempt("g1-"+str(i),i=5,o=1024,ro=1024) for i in range(8))
        self.check(envelope(40,8192,8,8,cls="usage_budget_exceeded",msg="output-token budget exhausted (8192/8192)"),log,1,"budget_exhausted")
        with self.assertRaises(ValueError):
            self.check(envelope(40,8193,8,8),log,0,"complete")
        self.check(envelope(0,0,0,1,cls="usage_budget_exceeded",msg="google_attempt_reservation_exhausted"),"",1,"budget_reserve_exhausted")
        with self.assertRaises(ValueError):
            self.check(envelope(4,3,1),attempt(ri=32000,ro=1024),0,"complete")

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
        self.assertEqual({m.position(s,t)[2] for s in m.SHARDS for t in range(6) if m.position(s,t)[0] in m.POLICY["selected_questions"]},m.SLOTS)
        self.assertEqual(sum(m.position(s,t)[1]=="baseline" for s in range(8) for t in (0,2,4)),12)

class Claims(unittest.TestCase):
    def test_failed_ubuntu_predecessor_requires_terminal_and_preserved_evidence(self):
        head="87395e1e32d92f8c851eed16aa32a802f6a1da1c"
        old="53c3403a7a527fc62c8a82195150d2cc101337bae1b4d1be6f4708516d74e6f1"
        prior={"status":"completed","conclusion":"failure","head_sha":head,"run_attempt":1}
        artifact={"id":15035198762,"workflow_run":{"id":45640828200},"digest":"sha256:419c739392cd9fcf19902e7670cf328889ea56294da5983947a8d8181bbb23d2","expired":False}
        refs=[{"rhg2-slot-"+old+"-03-candidate":head},{},{"rhg2-halt-"+old:head}]
        with patch.object(m,"api",side_effect=[prior,artifact]),patch.object(m,"matching",side_effect=refs):
            m.verify_failed_ubuntu_predecessor()
        with patch.object(m,"api",side_effect=[prior,dict(artifact,expired=True)]),patch.object(m,"matching",side_effect=refs):
            with self.assertRaisesRegex(ValueError,"ubuntu_predecessor_artifact"):m.verify_failed_ubuntu_predecessor()
        with patch.object(m,"api",return_value=prior),patch.object(m,"matching",return_value={}):
            with self.assertRaisesRegex(ValueError,"ubuntu_predecessor_inventory"):m.verify_failed_ubuntu_predecessor()

    def test_pilot_requires_previous_failure_evidence_and_ci_smoke(self):
        old="e186e248a996b227bf00ad390323ac1f5433f8b28ad8461000ece99dfbce3b5d"
        head="1b4027c727992cac8713db3347eee174caafb1d1"
        prior={"status":"completed","conclusion":"failure","head_sha":head,"run_attempt":1}
        artifact={"workflow_run":{"id":45643273890},"digest":"sha256:db46502d152f97812b06e25662f9dc427d63fb6b77430f4530ae35d5f7a62707","expired":False}
        smoke={"status":"completed","conclusion":"success","head_sha":"2395298a0d343b40160dd1d1fa07979652cb9ce2","run_attempt":1}
        refs=[{"rhg2-slot-"+old+"-03-candidate":head},{},{"rhg2-halt-"+old:head}]
        with patch.object(m,"api",side_effect=[prior,artifact,smoke]),patch.object(m,"matching",side_effect=refs):
            m.verify_ci_pilot_predecessors()
        with patch.object(m,"api",side_effect=[prior,artifact,dict(smoke,conclusion="failure")]),patch.object(m,"matching",side_effect=refs):
            with self.assertRaisesRegex(ValueError,"ci_smoke_not_verified"):m.verify_ci_pilot_predecessors()
        with patch.object(m,"api",side_effect=[prior,dict(artifact,expired=True),smoke]),patch.object(m,"matching",side_effect=refs):
            with self.assertRaisesRegex(ValueError,"prior_accounted_artifact"):m.verify_ci_pilot_predecessors()

    def test_focus_requires_complete_pilot_with_all_four_seals(self):
        c="d621641381b3fc4d1510ad83e7ac8b48c9746f50ef9068e1a633bd846e7d9c25"
        h="f9e202c7f36b7c01db0b245d41a60244bda755f2"
        cohort,cp,sp,halt=m.names(c)
        slots={"04-baseline","04-candidate","05-baseline","05-candidate"}
        run={"status":"completed","conclusion":"success","head_sha":h,"run_attempt":1}
        claims={cp+slot:h for slot in slots}
        seals={sp+slot+"-test":h for slot in slots}
        with patch.object(m,"api",return_value=run),patch.object(m,"exact_ref",side_effect=[h,None]),patch.object(m,"matching",side_effect=[claims,seals]),patch.object(m,"verify_seal") as verify:
            m.verify_completed_pilot()
            self.assertEqual(verify.call_count,4)
        with patch.object(m,"api",return_value=run),patch.object(m,"exact_ref",side_effect=[h,None]),patch.object(m,"matching",side_effect=[claims,{}]):
            with self.assertRaisesRegex(ValueError,"pilot_seal_inventory"):m.verify_completed_pilot()
        with patch.object(m,"api",return_value=dict(run,conclusion="failure")):
            with self.assertRaisesRegex(ValueError,"pilot_run_not_complete"):m.verify_completed_pilot()

    def test_remainder_requires_completed_focus_and_disjoint_48_slots(self):
        c="6b5943f630e516788cecb725cf9c6d7de27185fc58984aa6730f93bd52460dbe"
        h="2254cc3e8167867ac50416344605a0718f2c94f7"
        cohort,cp,sp,halt=m.names(c)
        slots={f"{i:02}-{arm}" for i in (3,6,7,8,12,13,14) for arm in ("baseline","candidate")}
        run={"status":"completed","conclusion":"success","head_sha":h,"run_attempt":1}
        claims={cp+slot:h for slot in slots}
        seals={sp+slot+"-test":h for slot in slots}
        with patch.object(m,"api",return_value=run),patch.object(m,"exact_ref",side_effect=[h,None]),patch.object(m,"matching",side_effect=[claims,seals]),patch.object(m,"verify_seal") as verify:
            m.verify_completed_focus()
            self.assertEqual(verify.call_count,14)
        with patch.object(m,"api",return_value=run),patch.object(m,"exact_ref",side_effect=[h,None]),patch.object(m,"matching",side_effect=[claims,{}]):
            with self.assertRaisesRegex(ValueError,"focus_seal_inventory"):m.verify_completed_focus()
        with patch.object(m,"api",return_value=dict(run,conclusion="failure")):
            with self.assertRaisesRegex(ValueError,"focus_run_not_complete"):m.verify_completed_focus()
        with patch.object(m,"api",return_value=run),patch.object(m,"SLOTS",m.SLOTS|{"03-baseline"}):
            with self.assertRaisesRegex(ValueError,"disjoint_full_cohort"):m.verify_completed_focus()

    def test_unclaimed_recovery_preserves_halted_slot_and_9_prior_seals(self):
        c="917c06eadf99c899863cb2932639de1d18ca3219877e24e3eabee373c41d4aa5"
        h="93048cceacada5932da82dc4db69bc94ee80efe8"
        cohort,cp,sp,halt=m.names(c)
        sealed={f"{i:02}-{arm}" for i in (0,1,2,9) for arm in ("baseline","candidate")}|{"10-baseline"}
        claims={cp+slot:h for slot in sealed|{"10-candidate"}}
        seals={sp+slot+"-test":h for slot in sealed}
        run={"status":"completed","conclusion":"failure","head_sha":h,"run_attempt":1}
        with patch.object(m,"api",return_value=run),patch.object(m,"exact_ref",side_effect=[h,h]),patch.object(m,"matching",side_effect=[claims,seals]),patch.object(m,"verify_seal") as verify,patch.object(m,"check_artifact") as check:
            m.verify_halted_remainder()
            self.assertEqual(verify.call_count,9)
            check.assert_called_once_with(c,"10-candidate",46158239794,15270040707,"c6ce5b0ec502ad0890072ad63cf7b82b998a01fe6efeec6a332faac190b0bf27")
        with patch.object(m,"api",return_value=run),patch.object(m,"SLOTS",m.SLOTS|{"10-candidate"}):
            with self.assertRaisesRegex(ValueError,"unclaimed_scope"):m.verify_halted_remainder()
        with patch.object(m,"api",return_value=run),patch.object(m,"exact_ref",side_effect=[h,None]):
            with self.assertRaisesRegex(ValueError,"remainder_halt_pin"):m.verify_halted_remainder()

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
        with patch.object(m,"exact_ref",return_value=None),patch.object(m,"matching",side_effect=lambda p:{prefix+"11-baseline":self.HEAD} if p==prefix else {}):
            with self.assertRaisesRegex(ValueError,"claimed_slot_not_uniquely_sealed"):
                m.verify_existing(self.C,self.HEAD)

    def test_halted_cohort_refuses_resume(self):
        with patch.object(m,"exact_ref",return_value=self.HEAD):
            with self.assertRaisesRegex(ValueError,"cohort_halted"):
                m.verify_existing(self.C,self.HEAD)

    def test_sealed_slot_excluded_and_unclaimed_slots_available(self):
        _,prefix,seals,_=m.names(self.C)
        name=seals+"11-baseline-123-456-"+"d"*64+"-"+"e"*64
        with patch.object(m,"exact_ref",return_value=None),patch.object(m,"matching",side_effect=lambda p:{prefix+"11-baseline":self.HEAD} if p==prefix else {name:self.HEAD}),patch.object(m,"check_artifact") as check:
            self.assertEqual(m.verify_existing(self.C,self.HEAD),{"11-baseline"})
            self.assertEqual(len(m.SLOTS-{"11-baseline"}),19)
            check.assert_called_once_with(self.C,"11-baseline",123,456,"e"*64)

    def test_claim_duplicate_is_not_reexecuted(self):
        record=({}, {}, self.C, 123, 0, "baseline", "11-baseline", {})
        with patch.object(m,"context",return_value=(self.HEAD,124)),patch.object(m,"pins",return_value=record),patch.object(m,"exact_ref",side_effect=[self.HEAD,None,self.HEAD]),patch.object(m,"verify_seal") as verify,patch.object(m,"create_ref") as create,patch.object(m,"output") as out:
            m.claim(Path("unused"),Path("unused"),"resume",0,0)
            create.assert_not_called()
            verify.assert_called_once()
            out.assert_called_once_with("run","false")

    def test_initial_claim_duplicate_fails_closed(self):
        record=({}, {}, self.C, None, 0, "baseline", "11-baseline", {})
        with patch.object(m,"context",return_value=(self.HEAD,124)),patch.object(m,"pins",return_value=record),patch.object(m,"exact_ref",side_effect=[self.HEAD,None,self.HEAD]),patch.object(m,"create_ref") as create:
            with self.assertRaisesRegex(ValueError,"duplicate_slot"):
                m.claim(Path("unused"),Path("unused"),"initial",0,0)
            create.assert_not_called()

    def test_full_ref_length_guard_and_selected_scope(self):
        name=m.names(self.C)[2]+"11-baseline-12345678901-14943095748-"+"d"*64+"-"+"e"*64
        self.assertLessEqual(len(("refs/tags/"+name).encode("utf-8")),255)
        with patch.object(m,"api") as api:
            with self.assertRaisesRegex(ValueError,"github_ref_length"):
                m.create_ref("a"*246,self.HEAD)
            api.assert_not_called()
        self.assertEqual(len(m.SLOTS),20)
        self.assertEqual(m.SHARDS,(3,5,6,7))
        self.assertNotIn("03-baseline",m.SLOTS)
        self.assertNotIn("04-candidate",m.SLOTS)
        self.assertNotIn("05-baseline",m.SLOTS)
        self.assertEqual(m.SLOTS,{f"{i:02}-{a}" for i in (11,15,16,17,18,19,20,21,22,23) for a in ("baseline","candidate")})

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
