"""One-per-run tag admission transport, no GitHub calls."""
import copy
import unittest

import tag_gate
from shard_contract import canonical,sha


class TagGateTests(unittest.TestCase):
    def env(self):
        return {
            "GITHUB_REPOSITORY":"git-ksk/reasoning-harness",
            "GITHUB_SHA":"a"*40,
            "GITHUB_REF":"refs/tags/cli-normal-real24-development-v4-rw-01",
            "GITHUB_EVENT_NAME":"push","GITHUB_RUN_ATTEMPT":"1",
            "GITHUB_RUN_ID":"101"
        }

    def history(self):
        return {"repository_created_date_utc":"2026-11-01",
                "checked_through_date_utc":"2026-11-08",
                "windows":[{"start":"2026-11-01","end":"2026-11-08",
                     "pages":[{"total_count":1,"workflow_runs":[{
                         "event":"push","head_sha":"a"*40,"head_branch":tag_gate.TAG+"-rw-01",
                         "id":101,"created_at":"2026-11-08T10:00:00Z",
                         "path":tag_gate.WORKFLOW,"run_attempt":1}]}]}]}

    def test_one_history_first_tag_can_be_reused_only_by_exact_same_run(self):
        proof=tag_gate.make_proof(self.env(),self.history())
        self.assertEqual(tag_gate.verify_proof(proof,self.env()),proof)
        self.assertEqual(sha(canonical(proof)),sha(canonical(proof)))
        for key,value in (("GITHUB_SHA","b"*40),("GITHUB_RUN_ID","102")):
            bad=dict(self.env(),**{key:value})
            with self.assertRaisesRegex(ValueError,"identity_changed"):
                tag_gate.verify_proof(proof,bad)
        bad=copy.deepcopy(proof)
        bad["admission"]["tag_runs"][0]["head_sha"]="b"*40
        with self.assertRaisesRegex(ValueError,"missing_original_first_run"):
            tag_gate.verify_proof(bad,self.env())

    def test_replays_or_missing_history_never_admit(self):
        extra=copy.deepcopy(self.history())
        extra["windows"][0]["pages"][0]["workflow_runs"].append({
            **extra["windows"][0]["pages"][0]["workflow_runs"][0],
            "id":202})
        extra["windows"][0]["pages"][0]["total_count"]=2
        with self.assertRaisesRegex(ValueError,"tag_must_have_one_development_run"):
            tag_gate.make_proof(self.env(),extra)
        wrong_case=dict(self.env(),GITHUB_REF="refs/tags/cli-normal-real24-development-v4-rw-02")
        with self.assertRaises(ValueError):
            tag_gate.make_proof(wrong_case,self.history())
        wrong_case=dict(self.env(),GITHUB_REF="refs/tags/cli-normal-real24-development-v4-rw-25")
        with self.assertRaisesRegex(ValueError,"wrong_actual_push"):
            tag_gate.make_proof(wrong_case,self.history())
        wrong=dict(self.env(),GITHUB_RUN_ATTEMPT="2")
        with self.assertRaisesRegex(ValueError,"wrong_actual_push"):
            tag_gate.make_proof(wrong,self.history())
        with self.assertRaises(ValueError):
            tag_gate.make_proof(self.env(),{"windows":[]})


if __name__=="__main__":
    unittest.main()
