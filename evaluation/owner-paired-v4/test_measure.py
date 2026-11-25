import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import measure as m


def attempt(status, a, state, i, o):
    return f"[mistral-accounted-attempt] status={status} attempt={a} usage={state} timestamp_ms=1 charged_input={i} charged_output={o} charged_total={i+o}\n"


def decision(a, state="reserved", reason="retry", wait=30000, cls="temporary_capacity"):
    allowed="true" if reason=="retry" else "false"
    return f"[mistral-rate-limit] status=429 timestamp_ms=1 attempt={a} retry_index={a-1} class={cls} retry_allowed={allowed} reason={reason} wait_ms={wait} usage={state} headers=absent\n"


def retry_wait(a, wait=30000):
    return f"[mistral-retry-wait] attempt={a} requested_ms={wait} elapsed_ms={wait+2}\n"


def envelope(i,o,a=1,calls=1,cls=None,msg="",diag=None):
    report={"run":{"tracked_from_session_start":True,"input_tokens":i,"output_tokens":o,
                   "total_tokens":i+o,"provider_attempts":a,"model_calls":calls},
            "budget":m.CAPS,"token_accounting":m.ACCOUNTING}
    if cls:
        failure={"failure_class":cls,"message":msg}
        if diag:
            failure["model_output_diagnostic"]=diag
        result={"status":"failed","failure":failure,"partial_diagnostics":{"usage":report}}
    else:
        result={"status":"succeeded","usage":report,"finalization":{"text":"invented answer","status":"qualified_partial_answer"}}
    return json.dumps({"schema_version":"reason-cli-output-v1","command":"ask","result":result}).encode()


class Measurement(unittest.TestCase):
    def check(self, raw, log, rc, expected):
        obs={"status":"incomplete"}
        m.terminal_observation(raw,log.encode(),rc,obs)
        self.assertEqual(obs["status"],expected)
        return obs

    def test_success_no_retry(self):
        obs=self.check(envelope(4,3),attempt(200,1,"reported",4,3),0,"complete")
        self.assertEqual(obs["telemetry"]["actual_total_tokens"],7)
        self.assertEqual(obs["telemetry"]["http_attempts"],1)

    def test_recovered_429_separates_actual_and_reserved(self):
        log=attempt(429,1,"reserved",600,8)+decision(1)+retry_wait(1)+attempt(200,2,"reported",4,3)
        obs=self.check(envelope(604,11,2),log,0,"complete")
        self.assertIsNone(obs["telemetry"]["actual_total_tokens"])
        self.assertEqual(obs["telemetry"]["reported_tokens"]["total"],7)
        self.assertEqual(obs["telemetry"]["reserved_unknown_tokens"]["total"],608)
        self.assertEqual(obs["telemetry"]["retries"],1)
        self.assertEqual(obs["telemetry"]["retry_wait_ms"],30002)

    def test_reported_429_recovery_exact(self):
        log=attempt(429,1,"reported",5,0)+decision(1,"reported")+retry_wait(1)+attempt(200,2,"reported",4,3)
        obs=self.check(envelope(9,3,2),log,0,"complete")
        self.assertEqual(obs["telemetry"]["actual_total_tokens"],12)

    def test_isolated_terminal_429_continues_but_never_scores(self):
        log=attempt(429,1,"reserved",600,8)+decision(1,reason="token_reservation_limit",wait=0)
        raw=envelope(600,8,cls="rate_limit",msg="Mistral returned HTTP 429; class=temporary_capacity; retry_decision=token_reservation_limit; usage=reserved")
        obs=self.check(raw,log,1,"transient_failed")
        self.assertNotIn("final_sha256",obs)
        self.assertIsNone(obs["answer_status"])
        self.assertIn(obs["status"],m.TERMINAL)

    def test_exact_budget_exhaustion_and_local_preflight_refusal(self):
        log="".join(attempt(200,1,"reported",5,1024) for _ in range(8))
        self.check(envelope(40,8192,8,8,cls="usage_budget_exceeded",msg="output-token budget exhausted (8192/8192)"),log,1,"budget_exhausted")
        obs=self.check(envelope(0,0,0,1,cls="usage_budget_exceeded",msg="mistral_attempt_reservation_exhausted"),"",1,"budget_reserve_exhausted")
        self.assertNotIn("final_sha256",obs)

    def test_local_decode_failure_is_terminal_only_if_accounted(self):
        raw=envelope(4,3,cls="protocol",diag={"origin":"local_model_output_decode","phase":"initial_candidate"})
        self.check(raw,attempt(200,1,"reported",4,3),1,"local_decode_failed")
        with self.assertRaises(ValueError):
            self.check(raw,"",1,"local_decode_failed")

    def test_global_conditions_never_continue(self):
        for cls in ("quota","credentials","transport","timeout","protocol"):
            with self.subTest(cls=cls):
                with self.assertRaises(ValueError):
                    self.check(envelope(4,3,cls=cls),attempt(200,1,"reported",4,3),1,"complete")
        log=attempt(429,1,"reserved",600,8)+decision(1,reason="retry_wait_limit",wait=0)
        with self.assertRaises(ValueError):
            self.check(envelope(600,8,cls="rate_limit",msg="class=temporary_capacity; retry_decision=retry_wait_limit"),log,1,"transient_failed")

    def test_no_guessing_usage_and_no_replacement(self):
        valid=attempt(429,1,"reserved",600,8)+decision(1)+retry_wait(1)+attempt(200,2,"reported",4,3)
        for log in (valid.replace("charged_total=608","charged_total=0"),
                    valid.replace(retry_wait(1),""),
                    valid.replace("status=200 attempt=2","status=200 attempt=1"),
                    valid.replace("retry_allowed=true","retry_allowed=false")):
            with self.subTest(log=log):
                with self.assertRaises(ValueError):
                    self.check(envelope(604,11,2),log,0,"complete")
        no_retry=attempt(429,1,"reserved",600,8)+decision(1,reason="token_reservation_limit",wait=0)
        with self.assertRaises(ValueError):
            self.check(envelope(604,11,2),no_retry+attempt(200,1,"reported",4,3),0,"complete")
        raw=json.loads(envelope(4,3))
        del raw["result"]["usage"]["token_accounting"]
        with self.assertRaises((ValueError,KeyError)):
            self.check(json.dumps(raw).encode(),attempt(200,1,"reported",4,3),0,"complete")

    def test_all_48_slots_unique_and_arm_order_balanced(self):
        rows=[m.position(s,t) for s in range(8) for t in range(6)]
        self.assertEqual(len({r[2] for r in rows}),48)
        self.assertEqual({m.position(s,t)[2] for s in m.SHARDS for t in range(6)},m.SLOTS)
        self.assertEqual(sum(m.position(s,t)[1]=="baseline" for s in range(8) for t in (0,2,4)),12)


class Claims(unittest.TestCase):
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
