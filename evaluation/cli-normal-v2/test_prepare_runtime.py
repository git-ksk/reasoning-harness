#!/usr/bin/env python3
"""Model-free controls for v2 derivation and closed live gates."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import textwrap
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
spec = importlib.util.spec_from_file_location('v2_prepare', HERE/'prepare_runtime.py')
prepare = importlib.util.module_from_spec(spec); spec.loader.exec_module(prepare)
spec = importlib.util.spec_from_file_location('v2_gate', HERE/'verify_runtime_archive.py')
gate = importlib.util.module_from_spec(spec); spec.loader.exec_module(gate)
CONFIG=json.loads((HERE/'config.json').read_bytes())
COUNTS=json.loads((HERE/'replacement-counts.json').read_bytes())

class DerivationControls(unittest.TestCase):
    def source(self, name):
        return subprocess.check_output(['git','show',prepare.PIN+':'+name],cwd=REPO)

    def test_protocol_only_changes_profile_host_and_review_scope(self):
        path='evaluation/cli-normal-v1/runtime/protocol.json'
        old=json.loads(self.source(path))
        new=json.loads(prepare.transform(path,self.source(path),COUNTS[path],CONFIG['common_cli_host']))
        self.assertEqual(new['identity'],'cli-normal-real24-development-v2')
        self.assertEqual(new['execution_tag'],'cli-normal-real24-development-v2')
        self.assertEqual(new['common_cli_host'],CONFIG['common_cli_host'])
        self.assertEqual(new['operational_review']['required_scope'],gate.SCOPE)
        old['identity']=new['identity']; old['execution_tag']=new['execution_tag']
        old['common_cli_host']=new['common_cli_host']
        old['operational_review']['required_scope']=new['operational_review']['required_scope']
        self.assertEqual(old,new)  # all model, bounds, pacing, gate and track fields preserved

    def test_changed_or_extra_replacement_is_rejected(self):
        path='evaluation/cli-normal-v1/runtime/build.py'
        data=self.source(path)
        with self.assertRaisesRegex(ValueError,'unexpected_replacement_count'):
            prepare.transform(path,data+prepare.OLD_HOST.encode(),COUNTS[path],CONFIG['common_cli_host'])
        with self.assertRaisesRegex(ValueError,'unexpected_replacement_count'):
            prepare.transform(path,data.replace(prepare.OLD_HOST.encode(),b'other'),COUNTS[path],CONFIG['common_cli_host'])

    def test_v1_verdict_cannot_admit_v2(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'review.json'
            path.write_text(json.dumps({'reviewer':gate.REVIEWER,'reviewer_kind':'agent_nonhuman',
                'scope':'cli-normal-development-runtime','result':'PASS','formal_verdict_issued':True,
                'verdict_stage':'FINAL_INDEPENDENT_OPERATIONAL_REVIEW',
                'synthetic_offline_integration_input_only':False,'not_formal_operational_verdict':False}))
            with self.assertRaisesRegex(ValueError,'final_independent_operational_review_required'):
                gate.verify_review(path,reference={})

    def test_workflows_inherit_conditions_and_fail_before_model_credentials(self):
        runtime=(REPO/'.github/workflows/cli-normal-runtime-v2.yml').read_text()
        real=(REPO/'.github/workflows/cli-normal-real24-development-v2.yml').read_text()
        self.assertIn('if: inputs.check_provider_readiness != true', runtime)
        self.assertIn('if: inputs.check_provider_readiness == true', runtime)
        self.assertIn('if: github.run_attempt == 1', real)
        self.assertFalse(CONFIG['execution_admitted'])
        self.assertEqual(runtime.count('group: reasoning-harness-mistral-live'),1)
        self.assertEqual(real.count('group: reasoning-harness-mistral-live'),1)
        for workflow, section, missing in [
            (runtime,'Load independently reviewed runtime before credentials','runtime-reference.json'),
            (real,'Verify frozen admission and prepare the exact seen24 input before credentials','receipt.json')]:
            code=workflow.split(section,1)[1].split("python3 - <<'PY'",1)[1].split('\n          PY',1)[0]
            code=textwrap.dedent(code)
            self.assertLess(workflow.index(section),workflow.index('MISTRAL_API_KEY:'))
            with tempfile.TemporaryDirectory() as tmp:
                result=subprocess.run([sys.executable,'-c',code],cwd=tmp,capture_output=True,text=True,
                                      env={'PATH':'/usr/bin:/bin'})
                self.assertNotEqual(result.returncode,0)
                self.assertIn('FileNotFoundError',result.stderr)
                self.assertIn(missing,result.stderr)
        self.assertLess(real.index('tag_gate=Path('),real.index('MISTRAL_API_KEY:'))
        self.assertLess(runtime.index('--verify-only'),runtime.index('MISTRAL_API_KEY:'))

    def test_fresh_preparation_freezes_v2_inventory_without_granting_execution(self):
        with tempfile.TemporaryDirectory() as tmp:
            out=Path(tmp)/'prepared'
            subprocess.run([sys.executable,str(HERE/'prepare_runtime.py'),'--repo',str(REPO),'--output',str(out)],check=True,capture_output=True)
            manifest=json.loads((out/'preparation.json').read_bytes())
            self.assertEqual(len(manifest['files']),104)
            self.assertEqual(manifest['profile'],'cli-normal-real24-development-v2')
            self.assertFalse(manifest['frozen_execution_receipt'])
            self.assertFalse(manifest['independent_review_complete'])
            for name in ['evaluation/cli-normal-v2/verify_runtime_archive.py', '.github/workflows/cli-normal-runtime-v2.yml',
                         '.github/workflows/cli-normal-real24-development-v2.yml']:
                self.assertEqual(manifest['files'][name],prepare.sha((REPO/name).read_bytes()))
            for name in ['build.py','development.py','runtime_proof.py','protocol.json']:
                self.assertIn(CONFIG['common_cli_host'],(out/'repo/evaluation/cli-repair-paired-v1/normal_v2'/name).read_text())
            self.assertFalse((out/'repo/evaluation/cli-repair-paired-v1/normal_v1').exists())
            self.assertFalse(manifest['v2_derivation']['execution_admitted'])
            self.assertEqual((HERE/'runtime/tag_admission.py').read_bytes(), (out/'repo/evaluation/cli-repair-paired-v1/normal_v2/tag_admission.py').read_bytes())
            self.assertEqual(manifest['v2_derivation']['source_commit'],prepare.PIN)
            self.assertEqual(manifest['v2_derivation']['generator_sha256'],prepare.sha((HERE/'prepare_runtime.py').read_bytes()))

if __name__=='__main__':
    unittest.main()
