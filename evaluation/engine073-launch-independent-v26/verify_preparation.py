#!/usr/bin/env python3
"""Read-only v26 M0 verifier; no candidate/cohort/scorer execution."""
import ast
from evaluator_contract import *
def main():
 old,pres=preserved();a=load(ROOT/'fixed-artifacts.json');runtime=load(ROOT/'runtime-artifact-hashes.json')['files'];parents=load(ROOT/'parent-receipt-file-hashes.json')
 if not checked_artifacts(a):raise ValueError('checked_artifact_mismatch')
 if subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()!=SOURCE:raise ValueError('candidate_HEAD_mismatch')
 if any(sha(REPO/n)!=h for n,h in parents.items()):raise ValueError('parent36_reference_changed')
 if any(sha(n)!=h for n,h in runtime.items()):raise ValueError('runtime_changed')
 if {str(p) for p in Path('/private/tmp/engine073-independent-v26-runtime').iterdir() if p.is_file()}!=set(runtime):raise ValueError('runtime_inventory_changed')
 for n in ['author_cases.py','authoring-start.json','cases.json','inputs.json','labels.json','manifest.json','execution.started.json','first-invocation.started.json','first-score.started.json','first-results.json','first-summary.json']:
  if (ROOT/n).exists():raise ValueError('pre_author_phase_violation:'+n)
 for p in ROOT.glob('*.py'):ast.parse(p.read_text())
 m0=evaluator_integrity() if (ROOT/'evaluator-freeze.json').exists() else None
 print(json.dumps({'stage':'PRE_AUTHOR_READ_ONLY_VERIFY','old_source_files':old,'separate_preparation_files':pres,'parent_reference_files':len(parents),'runtime_files':len(runtime),'checked_artifacts':len(a),'M0_sha256':sha(ROOT/'evaluator-freeze.json') if m0 else None,'candidate_source':SOURCE,'Engine_invocations':0,'real_cohort_scorer_processes':0,'authored_cases':0,'all_hash_inventory_checks':'PASS'},indent=2))
if __name__=='__main__':main()
