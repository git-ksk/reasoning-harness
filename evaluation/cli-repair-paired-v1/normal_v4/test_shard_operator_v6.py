"""Provider-free V6 CI stage and registered-reviewer signing controls."""
import copy
import json
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "normal_v3"))

from nacl.signing import SigningKey
from contracts import canonical,digest
from ci_prepare import context
from ci_run import drive
from ci_review_sign import start, envelope
from ci_exchange import APPROVAL, authenticate
from native_lane import private_bytes
from test_shard_contract import fixture
from shard_contract import global_manifest,select_case


class ShardV6OperatorTests(unittest.TestCase):
    def test_context_uses_one_tag_and_requires_exact_matrix_case(self):
        now=int(time.time())
        env={"GITHUB_REPOSITORY":"git-ksk/reasoning-harness",
             "GITHUB_RUN_ATTEMPT":"1","GITHUB_SHA":"a"*40,
             "GITHUB_RUN_ID":"54321",
             "GITHUB_EVENT_NAME":"push",
             "GITHUB_REF":"refs/tags/cli-normal-real24-development-v6-rw-01",
             "GITHUB_WORKFLOW":"cli-normal-real24-development-v6",
             "V4_CASE_ID":"rw-01",
             "V4_JOB_STARTED_UNIX":str(now)}
        valid=context(env,"live",shard_case_id="rw-01")
        self.assertEqual(valid["execution_tag"],"cli-normal-real24-development-v6-rw-01")
        self.assertEqual(valid["shard_case_id"],"rw-01")
        self.assertEqual(valid["job_deadline_unix"],now+290*60)
        for metadata in (
            dict(env,GITHUB_WORKFLOW='cli-normal-real24-development-v5'),
            dict(env,GITHUB_WORKFLOW='arbitrary-workflow'),
            {k:v for k,v in env.items() if k!='GITHUB_WORKFLOW'},
        ):
            with self.assertRaisesRegex(ValueError,"unregistered_real24_cohort|live_one_shot_tag_required"):
                context(metadata,"live",shard_case_id="rw-01")
        wrong=dict(env,V4_CASE_ID="rw-02")
        with self.assertRaisesRegex(ValueError,"invalid_case_shard"):
            context(wrong,"live",shard_case_id="rw-01")
        wrong=dict(env,GITHUB_REF="refs/tags/cli-normal-real24-development-v3")
        with self.assertRaisesRegex(ValueError,"one_shot_tag_required"):
            context(wrong,"live",shard_case_id="rw-01")
        prep=dict(env,GITHUB_EVENT_NAME="workflow_dispatch",GITHUB_REF="refs/heads/main")
        self.assertIsNone(context(prep,"prepare",shard_case_id="rw-01")["execution_tag"])

    def test_drive_per_shard_uses_private_stage_namespace_and_two_reviews(self):
        class Lane:
            plan={"jobs":[{"id":"rw-01-baseline","timeout_seconds":1200},
                          {"id":"rw-01-candidate","timeout_seconds":1200}]}
            next=0
            def run_next(self):
                ident=self.plan["jobs"][self.next]["id"]
                return {"job_id":ident},{"job_id":ident}
            def accept_review(self,review):
                self.next+=1
                return {"reviewed_job":self.plan["jobs"][self.next-1]["id"]}
        published=[];received=[]
        def publish(stage, raw):
            published.append(stage)
        def receive(stage,expected,deadline):
            received.append(stage)
            return {"verified":expected["job_id"]}
        lane=Lane()
        results=drive(lane,publish,receive,time.monotonic()+8000,stage_prefix="rw-01-")
        self.assertEqual(published,["rw-01-job-00","rw-01-job-01"])
        self.assertEqual(received,published)
        self.assertEqual(len(results),2)

    def prepare_signer(self):
        t=tempfile.TemporaryDirectory()
        self.addCleanup(t.cleanup)
        root=Path(t.name)
        os.chmod(root,0o700)
        (root/"input").mkdir(mode=0o700)
        corpus,requests=fixture()
        source="f"*40
        manifest=global_manifest(corpus,requests,source)
        shard=select_case(manifest,requests,"rw-01")
        session={"mode":"live","execution_tag":"cli-normal-real24-development-v6-rw-01",
                 "repository":"git-ksk/reasoning-harness",
                 "run_id":101,"run_attempt":1,"source_commit":source,
                 "shard_case_id":"rw-01"}
        keys=SigningKey.generate()
        policy={"contract":"reason-cli-normal-v3-review-policy-v1",
                "policy_id":"test-v6-paired",
                "principals":[
                    {"subject_id":"executor","aliases":["exec"],"role":"agent","conflict_groups":["exec"]},
                    {"subject_id":"reviewer","aliases":["review"],"role":"agent","conflict_groups":["review"]}],
                "keys":[{"key_id":"review-key","subject_id":"reviewer",
                         "public_key_hex":keys.verify_key.encode().hex(),"revoked":False}]}
        operator={"evaluator_id":"review","executor_id":"exec","fee_mode":"free-only",
                  "additional_charges_enabled":False,"review_policy":policy}
        receipt={"contract":"synthetic-test-receipt"}
        plan={"schema":"engine07-operational-paired-shard-v1","shard":shard,
              "receipt_sha256":digest(canonical(receipt)),
              "jobs":[{"id":"rw-01-baseline"},{"id":"rw-01-candidate"}]}
        bundle={"contract":"synthetic-test-bundle"}
        authority={"contract":"synthetic-test-authority"}
        gold={"contract":"reason-cli-normal-v3-gold-v1",
              "receipt_sha256":digest(canonical(receipt)),
              "cases":[{"case_id":"rw-01","question":"Frozen question rw-01",
                        "label":"positive","required_groups":["g1"]}]}
        for fn,val in (
            ("session.json",session),("plan.json",plan),("gold.json",gold),
            ("receipt.json",receipt),("native-bundle.json",bundle),
            ("authority-receipt.json",authority),
            ("readiness.json",{"authentication_and_model_available":True,
                "model_generations":0,"metadata_requests_attempted":1}),
            ("shard.json",shard),("global-manifest.json",manifest),
            ("requests.json",{"jobs":requests})):
            private_bytes(root/fn,canonical(val))
        private_bytes(root/"input/corpus.json",canonical(corpus))
        policy_bytes=canonical(policy)
        request={
            "contract":APPROVAL,
            "session_sha256":digest(canonical(session)),
            "run_id":session["run_id"],"stage":"start",
            "native_bundle_sha256":digest(canonical(bundle)),
            "plan_sha256":digest(canonical(plan)),
            "authority_receipt_sha256":digest(canonical(authority)),
            "policy_sha256":digest(policy_bytes),"fee_mode":"free-only"}
        private_bytes(root/"start-request.json",canonical(request))
        decision={"contract":"reason-cli-v3-independent-reviewed-start-v1",
                  "decision":"approve","start_request_sha256":digest(canonical(request)),
                  "full_sources_reviewed":True,
                  "notes":"synthetic v4 full-24-manifest signing policy verification only"}
        return root,operator,keys,policy_bytes,decision,request

    def test_signer_accepts_frozen_pair_only_and_rejects_altered_global_manifest(self):
        root,operator,key,policy,decision,request=self.prepare_signer()
        signed=start(root,decision,operator,key,"review-key",policy)
        self.assertEqual(authenticate(signed,policy,"review","exec",APPROVAL,request)["decision"],"approve")
        tampered= json.loads((root/"global-manifest.json").read_bytes())
        tampered["cases"][0]["question_sha256"]="0"*64
        (root/"global-manifest.json").write_bytes(canonical(tampered))
        with self.assertRaises(ValueError):
            start(root,decision,operator,key,"review-key",policy)


if __name__=="__main__":
    unittest.main()
