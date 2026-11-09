"""Fail-closed controls for the real24 cross-shard semantic score assembler."""
import copy
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import aggregate_v6 as target
from shard_contract import EXPECTED_IDS, select_case, global_manifest, canonical
from test_shard_contract import fixture


class AggregateV6Tests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        corpus,requests=fixture()
        cls.manifest=global_manifest(corpus,requests,"a"*40)
        cls.shards=[select_case(cls.manifest,requests,c) for c in EXPECTED_IDS]
        cls.all_gold={cid:{"case_id":cid,"question":"Frozen question "+cid,
                      "label":"positive","required_groups":["g1"]}
                      for cid in EXPECTED_IDS}

    def outcome(self,cid):
        return {"case_id":cid,
                "session":{"run_id":123456+int(cid[-2:]),"source_commit":"a"*40},
                "shard":next(x for x in self.shards if x["case_id"]==cid),
                "global_manifest":self.manifest,
                "gold":self.all_gold[cid],
                "rows":[{"job_id":cid+"-"+a,"case_id":cid,"arm":a,
                         "label":"positive","tp":1,"fp":0,"fn":0,"tn":0,
                         "claim_tp":1,"claim_fp":0,"claim_fn":0,
                         "claim_denominator":1,"hard_violations":0,
                         "quality_invalid":False,"operational_error":False,
                         "source_failure_class":[],"recovered_local_decode":False,
                         "citation_evaluable":True,"factual_claims":1,
                         "supported_cited_claims":1,"citation_count":1,
                         "citation_passed_count":1,"valid_answer":True,
                         "valid_semantic_outcome":True,"task_success":True}
                        for a in ("baseline","candidate")],
                "review_attestation_sha256":"a"*64,
                "source_receipt_sha256":"b"*64,
                "first_score_sha256":"c"*64}

    def test_full_pair_grade_not_synthetic_admission(self):
        with patch.object(target,"read_pair",side_effect=lambda root,c:self.outcome(c)):
            report=target.aggregate(Path("/fake"))
        self.assertEqual(report["semantic_reviewed_jobs"],48)
        self.assertTrue(report["metrics"]["cohort_measured"])
        self.assertFalse(report["quality_go"])
        self.assertFalse(report["previous_v3_run_reused"])

    def test_v5_original_shard_cannot_be_reused_as_v6(self):
        import json
        from pathlib import Path
        import tempfile
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)/"rw-06"
            (root/"bootstrap").mkdir(parents=True)
            (root/"final").mkdir()
            # The tag identity is checked before any cross-cohort scoring.
            session={"mode":"live","execution_tag":"cli-normal-real24-development-v5-rw-06",
                     "shard_case_id":"rw-06","run_attempt":1,
                     "repository":"git-ksk/reasoning-harness"}
            raw=canonical(session)
            (root/"bootstrap/session.json").write_bytes(raw)
            (root/"final/session.json").write_bytes(raw)
            with self.assertRaisesRegex(ValueError,"wrong_live_shard_session"):
                target.read_pair(Path(temp),"rw-06")

    def test_missing_pair_fails_without_any_global_f1(self):
        def read(root,c):
            if c==EXPECTED_IDS[-1]:raise ValueError("v4_missing_decrypted_pair_artifact")
            return self.outcome(c)
        with patch.object(target,"read_pair",side_effect=read):
            with self.assertRaisesRegex(ValueError,"missing_decrypted_pair_artifact"):
                target.aggregate(Path("/fake"))

    def test_foreign_run_or_manifest_or_duplicate_pair_rejected(self):
        for mutation in ("run","manifest","pair"):
            def read(root,c):
                row=self.outcome(c)
                if c==EXPECTED_IDS[-1]:
                    if mutation=="run":row["session"]["run_id"]=123456+int(EXPECTED_IDS[0][-2:])
                    elif mutation=="manifest":row["global_manifest"]={"foreign":True}
                    else:row["shard"]=copy.deepcopy(self.shards[0])
                return row
            with self.subTest(mutation=mutation):
                with patch.object(target,"read_pair",side_effect=read):
                    with self.assertRaises(ValueError):
                        target.aggregate(Path("/fake"))

    def test_reviewer_key_cannot_be_replaced_without_original_git_operator(self):
        import json
        from shard_contract import canonical
        with tempfile.TemporaryDirectory() as tmp:
            base=Path(tmp)
            bootstrap=base/"bootstrap"
            final=base/"final"
            source="evaluation/cli-repair-paired-v1/normal_v3/ci-operator.json"
            operator={"contract":"fixture","review_policy":{"contract":"original-policy"}}
            for arm in ("baseline","candidate"):
                f=bootstrap/"bundle-files"/("source/"+arm)/source
                f.parent.mkdir(parents=True,exist_ok=True)
                f.write_bytes(canonical(operator))
            final.mkdir()
            (final/"operator.json").write_bytes(canonical(operator))
            (final/"policy.json").write_bytes(canonical(operator["review_policy"]))
            target.check_source_enrolled_operator(bootstrap,final,operator)
            replaced=copy.deepcopy(operator)
            replaced["review_policy"]={"contract":"evil-policy"}
            (final/"operator.json").write_bytes(canonical(replaced))
            (final/"policy.json").write_bytes(canonical(replaced["review_policy"]))
            with self.assertRaisesRegex(ValueError,"changed_from_git_source"):
                target.check_source_enrolled_operator(bootstrap,final,replaced)
            (final/"operator.json").write_bytes(canonical(operator))
            with self.assertRaisesRegex(ValueError,"policy_substituted"):
                target.check_source_enrolled_operator(bootstrap,final,operator)

    def test_scoring_rejects_operational_stop_from_incomplete_pair(self):
        def read(root,c):
            row=self.outcome(c)
            if c==EXPECTED_IDS[1]:row["rows"][0]["operational_error"]=True
            return row
        with patch.object(target,"read_pair",side_effect=read):
            with self.assertRaisesRegex(ValueError,"incomplete_global_cohort_no_f1"):
                target.aggregate(Path("/fake"))


if __name__=="__main__":
    unittest.main()
