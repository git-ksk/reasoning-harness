"""Issue 616 provider-free public integrity tests; no private gold in shared CI."""
import copy,hashlib,json,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import validate_independent24 as val

class Independent24Public(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pub,cls.raw=val.read_json(val.HERE/val.PUBLIC)
        cls.lock,_=val.read_json(val.HERE/val.LOCK)
        cls.source,cls.srcraw=val.read_json(val.HERE/val.SOURCE)
        cls.dev,_=val.read_json(val.DEV)
    def attempt(self,pub=None,lock=None,source=None,raw=None,sourceraw=None,dev=None):
        return val.verify_public_data(pub if pub is not None else self.pub,
            lock if lock is not None else self.lock,
            source if source is not None else self.source,
            raw if raw is not None else self.raw,
            sourceraw if sourceraw is not None else self.srcraw,
            dev if dev is not None else self.dev)
    def test_real_public_24(self):
        result=self.attempt()
        self.assertEqual(result['public_case_count'],24)
        self.assertEqual(result['dev_exact_overlap'],0)
    def test_no_answer_label_field_leak(self):
        p=copy.deepcopy(self.pub);p['cases'][0]['classification']='answerable'
        with self.assertRaisesRegex(ValueError,'public_case_extra_field_or_leak'):
            self.attempt(pub=p)
    def test_question_text_tamper(self):
        p=copy.deepcopy(self.pub);p['cases'][0]['question']+=' changed'
        with self.assertRaisesRegex(ValueError,'question_hash_drift'):self.attempt(pub=p)
    def test_wrong_source_mapping(self):
        p=copy.deepcopy(self.pub);p['cases'][0]['source_id']='source-03'
        with self.assertRaisesRegex(ValueError,'id_or_per_source_count_wrong'):self.attempt(pub=p)
    def test_source_sha_change(self):
        p=copy.deepcopy(self.pub);p['cases'][1]['source_sha256']='0'*64
        with self.assertRaisesRegex(ValueError,'official_source_provenance_drift'):self.attempt(pub=p)
    def test_public_manifest_wrong_digest(self):
        p=copy.deepcopy(self.pub);p['source_manifest_sha256']='0'*64
        with self.assertRaisesRegex(ValueError,'public_source_hash_drift'):self.attempt(pub=p)
    def test_forged_review(self):
        p=copy.deepcopy(self.pub);p['reviewer_approved']=True
        with self.assertRaisesRegex(ValueError,'public_false_authority'):self.attempt(pub=p)
    def test_author_lock_premature_go(self):
        l=copy.deepcopy(self.lock);l['ready_to_execute']=True
        with self.assertRaisesRegex(ValueError,'premature_go'):self.attempt(lock=l)
    def test_forged_signature(self):
        l=copy.deepcopy(self.lock);l['reviewer_signature']='self-signed'
        with self.assertRaisesRegex(ValueError,'forged_review_or_score'):self.attempt(lock=l)
    def test_changed_gold_hash(self):
        l=copy.deepcopy(self.lock);l['private_gold_sha256']='invalid'
        with self.assertRaisesRegex(ValueError,'gold_sha_missing'):self.attempt(lock=l)
    def test_threshold_downgrade(self):
        l=copy.deepcopy(self.lock);l['case_f1_min']=0.1
        with self.assertRaisesRegex(ValueError,'pre_registered_threshold_drift'):self.attempt(lock=l)
    def test_old_core_commit_swap(self):
        l=copy.deepcopy(self.lock);l['baseline_core_commit']='f'*40
        with self.assertRaisesRegex(ValueError,'core_cli_commit_drift'):self.attempt(lock=l)
    def test_wrong_final_scoring_rule(self):
        l=copy.deepcopy(self.lock);l['scoring_policy']='score on internal model proposal'
        with self.assertRaisesRegex(ValueError,'unsafe_scoring_policy'):self.attempt(lock=l)
    def test_case_removed(self):
        p=copy.deepcopy(self.pub);p['cases'].pop()
        with self.assertRaisesRegex(ValueError,'not_24_cases'):self.attempt(pub=p)
    def test_old_dev_case_exact_reuse(self):
        p=copy.deepcopy(self.pub);q=self.dev['cases'][0]['question']
        p['cases'][0]['question']=q
        p['cases'][0]['question_sha256']=val.sha(q.encode())
        with self.assertRaisesRegex(ValueError,'dev_or_internal_exact_question_reuse'):self.attempt(pub=p)
    def test_model_override(self):
        l=copy.deepcopy(self.lock);l['model']='new-model-not-frozen'
        with self.assertRaisesRegex(ValueError,'sampling_protocol_changed'):self.attempt(lock=l)

if __name__=='__main__':unittest.main()
