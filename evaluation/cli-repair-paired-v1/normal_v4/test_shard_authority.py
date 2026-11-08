"""Paired shard authority tests. Does not weaken the v3 frozen 48 contract."""
from pathlib import Path
import sys
import copy
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "normal_v3"))
from test_offline_lane import make_plan
from contracts import canonical,digest
import authority


def fixture():
    import tempfile
    root=tempfile.TemporaryDirectory()
    plan, receipt, script, _ = make_plan(Path(root.name))
    p=copy.deepcopy(plan)
    case=p["case_ids"][0]
    p["case_ids"]=["rw-01"]
    question=p["questions"][case]
    p["questions"]={"rw-01":question}
    p["jobs"]=p["jobs"][:2]
    p["frozen_request_bindings"]={}
    shard={
        "contract":"reason-cli-real24-v4-paired-shard-v1",
        "global_manifest_sha256":"f"*64,
        "case_id":"rw-01","case_index":0,"total_cases":24,
        "case_question_sha256":digest(question.encode("utf-8")),
        "arm_order":[],"job_ids":[],"first_request_digests":{}
    }
    for j in p["jobs"]:
        arm=j["arm"]
        j["id"]="rw-01-"+arm
        j["native_request_plan"]={"request_sha256": "a"*64 if arm=="baseline" else "b"*64}
        p["frozen_request_bindings"][j["id"]]="a"*64 if arm=="baseline" else "b"*64
        shard["job_ids"].append(j["id"])
        shard["arm_order"].append(arm)
        shard["first_request_digests"][arm]=j["native_request_plan"]["request_sha256"]
    p["schema"]="engine07-operational-paired-shard-v1"
    p["shard"]=shard
    p["token_budget"]=max(p["token_budget"],2*24*(12512+1024))
    return root,p


class PairedAuthorityTests(unittest.TestCase):
    def test_two_shard_jobs_bound_without_modifying_v3_rule(self):
        tmp,p=fixture()
        self.addCleanup(tmp.cleanup)
        rows=authority.freeze_jobs(p)
        self.assertEqual(len(rows),2)
        self.assertEqual({x["arm"] for x in rows},{"baseline","candidate"})
        self.assertEqual([r["job_id"] for r in rows],p["shard"]["job_ids"])
        # Mutating back to the old schema must fail even with a valid pair.
        old=copy.deepcopy(p)
        old["schema"]="engine07-operational-plan-v1"
        with self.assertRaisesRegex(ValueError,"invalid_paired_cohort"):
            authority.freeze_jobs(old)

    def test_two_job_authority_reservations_and_completion_are_exact(self):
        from test_authority_580 import FakeRemoteObjectStore
        tmp,p=fixture()
        self.addCleanup(tmp.cleanup)
        local_receipt=canonical({"contract":"v4-test-local-receipt"})
        binding=authority.AuthorityBinding(
            source_sha256="a"*64,image_sha256="b"*64,
            cli_build_sha256="c"*64,runtime_sha256="d"*64,
            executor_id="v4-test-executor",evaluator_id="v4-test-reviewer")
        receipt={"contract":authority.RECEIPT,
            "evaluation_id":"synthetic-v4-pair-101",
            "plan_sha256":digest(canonical(p)),
            "legacy_receipt_sha256":digest(local_receipt),
            "provider":p["provider"],"model":p["model"],
            "token_budget":p["token_budget"],
            "jobs":authority.freeze_jobs(p),
            **vars(binding)}
        store=FakeRemoteObjectStore()
        store.objects[f"{authority.PREFIX}/receipts/{receipt['evaluation_id']}.json"]=canonical(receipt)
        session=authority.TrustedAuthority(store).start(
            receipt["evaluation_id"],p,local_receipt,binding)
        for job in p["jobs"]:
            jid=job["id"]
            session.reserve(jid)
            session.before_spawn(jid)
            self.assertTrue(session.observe(jid,tokens=1,attempts=1,
                observation_sha256=digest(canonical({"id":jid}))))
        session.complete()
        completed=store.read(authority._key(receipt["evaluation_id"],"completed"))
        self.assertEqual(__import__("json").loads(completed)["job_count"],2)
        with self.assertRaises(FileExistsError):
            session.complete()
        with self.assertRaises(FileExistsError):
            authority.TrustedAuthority(store).start(
                receipt["evaluation_id"],p,local_receipt,binding)

    def test_missing_pair_forged_source_or_wrong_case_index_fails(self):
        tmp,p=fixture()
        self.addCleanup(tmp.cleanup)
        for mutation in (
            lambda x:x["jobs"].pop(),
            lambda x:x["shard"].update(global_manifest_sha256="broken"),
            lambda x:x["shard"].update(case_index=17),
            lambda x:x["jobs"][0]["native_request_plan"].update(request_sha256="1"*64),
            lambda x:x["shard"]["job_ids"].reverse(),
        ):
            bad=copy.deepcopy(p)
            mutation(bad)
            with self.assertRaises(ValueError):
                authority.freeze_jobs(bad)


if __name__=="__main__":
    unittest.main()
