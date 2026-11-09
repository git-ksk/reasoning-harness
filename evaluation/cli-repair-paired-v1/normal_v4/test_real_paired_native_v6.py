"""Real DISTINCT baseline/candidate native CLI loopback for V6 shard.

No external Mistral provider calls, no account charges, no forged review:
signed synthetic known-content control only. Tests actual native process
supervision, two physical HTTP fixture replies, two pre-advance signed gates,
registered authority ledger, and exactly-once completion with 2/2 jobs.
"""
import copy
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/"normal_v3"))

import authority
from contracts import canonical,digest,frozen_request_digest
from local_store import initialize_local_store,LocalAtomicStore
from native_lane import NativeLane, CONTROL
import test_native_admission as native
import test_authority_580 as legacy


@unittest.skipUnless(os.environ.get("REASON_V6_NATIVE_CLI_BASELINE") and
                     os.environ.get("REASON_V6_NATIVE_CLI_CANDIDATE"),
                     "distinct v6 native CLI pair unavailable")
class RealV6PairedNativeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        native.ActualNativeAdmissionTests.setUpClass()
        cls.source_control = native.ActualNativeAdmissionTests

        from native_bundle import NativeBundle
        from native_lane import plan_first_request
        controls = cls.source_control
        root = Path(__file__).resolve().parents[3]
        binary_paths={
            arm:Path(os.environ["REASON_V6_NATIVE_CLI_"+arm.upper()]).resolve(strict=True)
            for arm in ("baseline","candidate")}
        if binary_paths["baseline"].read_bytes()==binary_paths["candidate"].read_bytes():
            raise ValueError("v6_native_two_distinct_binaries_required")
        controls.bundle=NativeBundle.enroll(
            source_roots={"baseline":root,"candidate":root},
            binaries=binary_paths,
            configurations={"policy":controls.policy_path,"sources":controls.catalog_path})
        controls.argv_by_arm={}
        controls.planned={}
        for arm, binary in binary_paths.items():
            argv=[str(binary),"--no-config","--plain","--provider","mistral",
                  "--model","fixture-model","--max-model-calls","4",
                  "--max-tokens","32","--max-total-tokens","128",
                  "--format","json",native.QUESTION]
            controls.argv_by_arm[arm]=argv
            controls.planned[arm]=plan_first_request(
                controls.bundle,arm,argv,controls.root/(arm+"-v6-request.json"))

    @classmethod
    def tearDownClass(cls):
        native.ActualNativeAdmissionTests.tearDownClass()

    def setUp(self):
        self.temp=tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name).resolve()
        controls=self.source_control
        self.question=native.QUESTION
        self.plan=copy.deepcopy(controls.template.plan)
        self.plan["schema"]="engine07-operational-paired-shard-v1"
        self.plan["purpose"]=CONTROL
        self.plan["case_ids"]=["rw-01"]
        self.plan["questions"]={"rw-01":self.question}
        self.plan["jobs"]=self.plan["jobs"][:2]
        self.plan["frozen_request_bindings"]={}
        self.plan["shard"]={
            "contract":"reason-cli-real24-v4-paired-shard-v1",
            "global_manifest_sha256":"f"*64,
            "case_id":"rw-01","case_index":0,"total_cases":24,
            "case_question_sha256":digest(self.question.encode()),
            "job_ids":[],"arm_order":[],"first_request_digests":{}}
        reserve=0
        for job in self.plan["jobs"]:
            arm=job["arm"]
            jid="rw-01-"+arm
            job["id"]=jid
            job["argv"]=list(controls.argv_by_arm[arm])
            job["native_request_plan"]=controls.planned[arm]
            job["timeout_seconds"]=15
            job["budget"]={"max_generations":4,
                "max_input_tokens_per_generation":20000,
                "max_output_tokens_per_generation":32,
                "max_attempts_per_generation":1}
            self.plan["shard"]["job_ids"].append(jid)
            self.plan["shard"]["arm_order"].append(arm)
            self.plan["shard"]["first_request_digests"][arm]=controls.planned[arm]["request_sha256"]
            reserve+=authority._job_reserve(job["budget"])
            self.plan["frozen_request_bindings"][jid]=frozen_request_digest(
                job_id=jid,case_id="rw-01",arm=arm,
                question=self.question,request=controls.planned[arm]["request"],
                request_sha256=controls.planned[arm]["request_sha256"],
                provider="mistral",model="fixture-model",
                receipt_sha256=digest(controls.template.local_receipt))
        self.plan["token_budget"]=reserve
        self.assertEqual(len(authority.freeze_jobs(self.plan)),2)
        profile=initialize_local_store(self.root/"authority")
        self.store=LocalAtomicStore(profile,role="executor")
        binding=controls.bundle.binding("trusted-executor","trusted-evaluator")
        _,receipt=controls.template.fixture("v6-shard-control",plan=self.plan,binding=binding)
        receipt["contract"]=authority.NATIVE_RECEIPT
        receipt["native_admission"]={
            "bundle_sha256":controls.bundle.sha256,
            "policy_sha256":digest(controls.reviewer.policy_bytes),
            "sources_sha256":digest(controls.catalog_bytes)}
        LocalAtomicStore(profile,role="issuer").create(
            f"{authority.PREFIX}/receipts/v6-shard-control.json",
            canonical(receipt))
        self.session=authority.TrustedAuthority(self.store).start(
            "v6-shard-control",self.plan,controls.template.local_receipt,binding)
        self.server=native.ScriptedServer("normal")
        self.addCleanup(self.server.close)
        self.lane=NativeLane(session=self.session,plan=self.plan,bundle=controls.bundle,
            policy_bytes=controls.reviewer.policy_bytes,catalog_bytes=controls.catalog_bytes,
            output=self.root/"lane",control_url=self.server.url)

    def test_exact_two_real_native_cli_generations_and_registered_gates(self):
        accepted=[]
        for index in range(2):
            packet,expected=self.lane.run_next()
            self.assertTrue(packet["stdout"])
            self.assertEqual(expected["job_id"],self.plan["jobs"][index]["id"])
            count=len(self.server.requests)
            with self.assertRaisesRegex(ValueError,"review_pending"):
                self.lane.run_next()
            self.assertEqual(len(self.server.requests),count)
            signed=self.source_control.reviewer.sign(
                expected,packet["stdout"].encode(),packet["stderr"].encode())
            accepted.append(self.lane.accept_review(signed))
            self.assertEqual(self.lane.next,index+1)
            self.assertIsNotNone(self.store.read(
                authority._key("v6-shard-control","admission/"+expected["job_id"])))
        self.assertEqual(len(accepted),2)
        self.assertIsNotNone(self.store.read(
            authority._key("v6-shard-control","completed")))
        self.assertEqual(json.loads(self.store.read(
            authority._key("v6-shard-control","completed")))["job_count"],2)
        self.assertGreaterEqual(len(self.server.requests),2)
        with self.assertRaises(ValueError):
            self.lane.run_next()


if __name__=="__main__":
    unittest.main()
