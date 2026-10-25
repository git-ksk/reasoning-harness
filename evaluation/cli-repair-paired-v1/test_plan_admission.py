"""Precredential plan tamper controls; no questions, Engine or provider calls."""
import contextlib,copy,io,json,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import development,make_development_plan,plan_inputs,run_lane
from scorer import sha

class PlanAdmission(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name).resolve();self.corpus={'cases':[{'id':'engineering-id-'+str(n)} for n in range(24)]}
        self.receipt={'consumed_M1_sha256':development.INPUT_M1};self.artifacts={'/inert-static-input':'0'*64}
        self.protocol=json.loads((Path(__file__).parent/'protocol.json').read_bytes())
        # Exercise the inherited plan validator separately from the preparation
        # entry guard, which verify_local.py checks without these fixture mocks.
        self.protocol['state']='UNIT_PLAN_VALIDATION_ONLY'
        output=self.root/'plan.json'
        argv=['make_development_plan.py','--receipt',str(self.root/'receipt.json'),'--corpus-dir',str(self.root),'--provider','mistral','--output',str(output)]
        with patch.object(sys,'argv',argv),patch.object(make_development_plan,'checked_receipt',return_value=self.receipt),patch.object(make_development_plan,'consumed_corpus',return_value=self.corpus),patch.object(make_development_plan,'inventory',return_value=self.artifacts),contextlib.redirect_stdout(io.StringIO()):
            make_development_plan.main()
        self.plan=json.loads(output.read_bytes())
    def check(self,plan):
        with patch.object(development,'checked_receipt',return_value=self.receipt),patch.object(development,'consumed_corpus',return_value=self.corpus),patch.object(plan_inputs,'inventory',return_value=self.artifacts):
            return run_lane.check_plan(plan,self.protocol)
    def test_generated_plan_passes(self):
        self.assertEqual(self.check(self.plan),'0.2184192')
    def test_tampered_plan_rejected_before_scheduler(self):
        mutations=[('M1',lambda p:p.update(M1_sha256='f'*64)),
            ('normal_timeout',lambda p:p['jobs'][0].update(timeout_seconds=600)),
            ('helper_timeout',lambda p:p['jobs'][2].update(timeout_seconds=360)),
            ('grace',lambda p:p['jobs'][0].update(cancel_grace_seconds=5)),
            ('corpus_path',lambda p:p['jobs'][0]['argv'].__setitem__(p['jobs'][0]['argv'].index('--corpus')+1,'/other/corpus.json')),
            ('duplicate_flag',lambda p:p['jobs'][0]['argv'].extend(['--bin-dir','/other/bin'])),
            ('case_order',lambda p:p['case_ids'].reverse()),
            ('job_order',lambda p:p['jobs'].__setitem__(slice(0,2),list(reversed(p['jobs'][:2])))),
            ('omitted_input',lambda p:p.update(artifacts={})),
            ('token_reservation',lambda p:p.update(token_budget=1)),
            ('currency_self_cap',lambda p:p.update(currency_self_cap_USD=100))]
        for name,mutate in mutations:
            with self.subTest(name=name):
                plan=copy.deepcopy(self.plan);mutate(plan)
                with self.assertRaises(ValueError):self.check(plan)
    def test_consumed_corpus_extra_file_rejected(self):
        root=self.root/'input';root.mkdir();data={'identity':'engine073-realworld-independent-v1',**self.corpus}
        (root/'corpus.json').write_text(json.dumps(data));manifest=json.dumps({'files':{'corpus.json':sha((root/'corpus.json').read_bytes())}}).encode();(root/'manifest.json').write_bytes(manifest)
        with patch.object(development,'INPUT_M1',sha(manifest)):
            self.assertEqual(len(development.consumed_corpus(root)['cases']),24)
            (root/'unbound').write_text('inert extra bytes')
            with self.assertRaisesRegex(ValueError,'consumed_inventory_changed'):development.consumed_corpus(root)
if __name__=='__main__':unittest.main()
