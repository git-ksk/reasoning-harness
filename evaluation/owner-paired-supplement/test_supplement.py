import copy
import unittest
from unittest.mock import patch
import supplement as s
def fixture():
    run=dict(id=s.RUN,head_sha=s.HEAD,head_branch=s.ORIGINAL,event="push",run_attempt=1,status="completed",conclusion="failure")
    jobs=[dict(name=n,conclusion="success") for n in ("offline-controls","preflight","claim")]
    for i in range(8):
        ok=i in s.COMPLETE
        jobs.append(dict(name=f"live ({i})",id=s.MISSING.get(i,100+i),status="completed",conclusion="success" if ok else "cancelled",runner_id=1 if ok else 0,runner_name="runner" if ok else "",steps=[{}] if ok else []))
    return run,jobs,[dict(name=n,expired=False) for n in s.ARTIFACTS]
class Controls(unittest.TestCase):
    def test_exact_predecessor(self):s.validate_original(*fixture())
    def test_refuse_replayed_or_active(self):
        for k,v in [("run_attempt",2),("status","in_progress"),("head_sha","x")]:
            r,j,a=fixture();r[k]=v
            with self.assertRaises(ValueError):s.validate_original(r,j,a)
    def test_refuse_any_execution_evidence(self):
        for k,v in [("runner_id",2),("runner_name","runner"),("steps",[{}]),("id",0),("conclusion","failure")]:
            r,j,a=fixture();next(x for x in j if x["name"]=="live (0)")[k]=v
            with self.assertRaises(ValueError):s.validate_original(r,j,a)
    def test_refuse_bad_inventory(self):
        for change in ("duplicate","missing","unexpected","expired"):
            r,j,a=fixture()
            if change=="duplicate":j.append(j[0])
            if change=="missing":a.pop()
            if change=="unexpected":a.append(dict(name="shard-0",expired=False))
            if change=="expired":a[0]["expired"]=True
            with self.assertRaises(ValueError):s.validate_original(r,j,a)
    def test_only_missing_allowed(self):
        for i in s.COMPLETE:
            with self.assertRaises(ValueError):s.claim_name(i)
        self.assertNotEqual(s.claim_name(0),s.claim_name(4))
    def test_context_denies_retry_or_local(self):
        env=dict(GITHUB_ACTIONS="true",GITHUB_RUN_ATTEMPT="1",GITHUB_EVENT_NAME="push",GITHUB_REPOSITORY=s.REPO,GITHUB_REF="refs/tags/"+s.TAG,GITHUB_SHA="a"*40,GITHUB_RUN_ID="123")
        with patch.dict("os.environ",env,clear=True):s.context()
        for k,v in [("GITHUB_RUN_ATTEMPT","2"),("GITHUB_ACTIONS","false"),("GITHUB_REF","refs/tags/"+s.ORIGINAL)]:
            with patch.dict("os.environ",dict(env,**{k:v}),clear=True):
                with self.assertRaises(ValueError):s.context()
    def test_preflight_binds_completed_pairs(self):
        p={"questions":[{"source_id":"s"+str(i//2)} for i in range(24)]}
        pre={"tag":s.ORIGINAL,"measurement_policy":s.m.POLICY,"pair_count":24,
             "hashes":[s.m.sha(str(i).encode()) for i in range(24)],
             "config_hashes":{"s"+str(i):s.m.sha(str(i).encode()) for i in range(12)}}
        rows=[[i,p["questions"][i]["source_id"],pre["hashes"][i],
               pre["config_hashes"][p["questions"][i]["source_id"]]]
              for shard in sorted(s.COMPLETE) for i in range(3*shard,3*shard+3)]
        with patch.object(s.m,"tag",return_value=s.ORIGINAL),patch.object(s,"ORIGINAL_COMPLETED_PAIR_PINS",s.m.sha(s.m.encode(rows))):
            s.verify_preflight_pins(p,pre)
            broken=copy.deepcopy(pre);broken["hashes"][3]="changed"
            with self.assertRaises(ValueError):s.verify_preflight_pins(p,broken)
            broken=copy.deepcopy(pre);broken["config_hashes"]["s1"]="changed"
            with self.assertRaises(ValueError):s.verify_preflight_pins(p,broken)
    def test_claim_refusal_never_executes_or_writes_receipt(self):
        from pathlib import Path
        import tempfile
        with tempfile.TemporaryDirectory() as t:
            root=Path(t)
            with patch.object(s,"context"),patch.object(s,"verify_original"),patch.object(s.m,"load",return_value={}),patch.object(s.m,"validate",return_value={}),patch.object(s,"verify_preflight_pins"),patch.object(s.m,"build_check"),patch.dict("os.environ",{"GITHUB_SHA":"a"*40}),patch.object(s,"api",side_effect=ValueError("already claimed")):
                with self.assertRaises(ValueError):s.claim(root,root/"build",0)
                self.assertFalse((root/"supplement-claim.json").exists())
    def test_execute_refuses_wrong_lineage_before_provider(self):
        from pathlib import Path
        env={"GITHUB_SHA":"a"*40,"GITHUB_RUN_ID":"123"}
        with patch.dict("os.environ",env,clear=True),patch.object(s,"context"),patch.object(s.m,"load",return_value={}),patch.object(s.m,"execute_admitted_shard") as execute:
            with self.assertRaises(ValueError):s.execute(Path("/tmp/fixture"),Path("/tmp/build"),4)
            execute.assert_not_called()
if __name__=="__main__":unittest.main()
