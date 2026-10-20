"""Transport-lineage controls only; no questions, models or metric execution."""
import copy,sys,unittest
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.append(str(HERE.parent/'engine073-realworld-evaluator-v2'))
import capture
from test_review_boundaries import normal_fixture

def fixture():
    args=normal_fixture();args[3]['resolution']['investigation']['capabilities'][0]['supported_fact_keys']=['value','secondary.value'];return args

def duplicate(args):
    event=copy.deepcopy(args[2][0]);event['request']['request']['id']='investigation:second-target:cap';event['request']['request']['target']['target_id']='second-target';args[2].append(event)
    admission=copy.deepcopy(args[1]['events'][0]);admission['target_id']='second-target';admission['admitted_evidence_ids']=[];args[1]['events'].insert(1,admission)
    for n,e in enumerate(args[1]['events']):e['sequence']=n

class CaptureRepair(unittest.TestCase):
    def test_exact_reacquisition_is_bound_once(self):
        args=fixture();duplicate(args);out=capture.bind_normal(*args);self.assertEqual(out['admitted_source_ids'],['s']);self.assertEqual(len(out['admission_routes']),1)
    def test_conflicting_reacquisition_is_rejected(self):
        args=fixture();duplicate(args);args[2][1]['response']['contribution']['evidence'][0]['facts']['value']='changed'
        with self.assertRaisesRegex(ValueError,'ambiguous_acquired_evidence'):capture.bind_normal(*args)
    def test_advertised_secondary_key_is_allowed(self):
        args=fixture();args[2][0]['request']['request']['target']['expected_fact_key']='secondary.value';self.assertEqual(capture.bind_normal(*args)['admitted_source_ids'],['s'])
    def test_unadvertised_key_is_rejected(self):
        args=fixture();args[2][0]['request']['request']['target']['expected_fact_key']='unadvertised.value'
        with self.assertRaisesRegex(ValueError,'acquisition_unsupported_fact_key'):capture.bind_normal(*args)
if __name__=='__main__':unittest.main()
