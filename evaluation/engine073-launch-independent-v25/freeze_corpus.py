#!/usr/bin/env python3
"""Future post-author M1 freeze; NEVER authors cases or invokes Engine/scorer."""
from evaluator_contract import *
def main():
 m0=evaluator_integrity();preserved()
 if any((ROOT/n).exists() for n in ['manifest.json','execution.started.json','first-invocation.started.json','first-score.started.json','first-results.json']):raise ValueError('M1_first_only_barrier')
 result=load(ROOT/'static-preflight.json');cases=load(ROOT/'cases.json');author=load(ROOT/'authoring-start.json');scope=load(ROOT/'scope-review.json')
 if result['outcome']!='PASS' or result['Engine_invocations']!=0 or result['scorer_invocations']!=0:raise ValueError('static_semantic0_required')
 if result['evaluator_freeze_sha256']!=sha(ROOT/'evaluator-freeze.json') or result['cases_sha256']!=sha(ROOT/'cases.json') or result['inputs_sha256']!=sha(ROOT/'inputs.json') or result['scope_review_sha256']!=sha(ROOT/'scope-review.json') or result['authoring_start_sha256']!=sha(ROOT/'authoring-start.json'):raise ValueError('static_artifacts_changed')
 if subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip()!=SOURCE:raise ValueError('dedicated_candidate_HEAD_changed')
 frozen={str(q.relative_to(ROOT)):sha(q) for q in sorted(ROOT.rglob('*')) if q.is_file()}
 if any(frozen.get(n)!=h for n,h in m0['frozen_files'].items()):raise ValueError('pre_author_evaluator_changed')
 manifest={'identity':'engine073-launch-independent-v25','frozen_utc':utc(),'candidate':m0['candidate'],'frozen_files':frozen,'checked_artifacts':m0['checked_artifacts'],'authoring':{'evaluator_freeze_sha256':sha(ROOT/'evaluator-freeze.json'),'authoring_start_sha256':sha(ROOT/'authoring-start.json'),'parent_authorization_reference':author['authorization_reference']},'scoring':{'policy_sha256':POLICY_SHA,'helper_sha256':HELPER_SHA,'score_sha256':sha(ROOT/'score.py'),'mode':'single scorer process invokes helper once with explicit snapshot config; joint exactness diagnostic only'},'cohort':{'planned':60,'positive':30,'negative':30,'labels_outside_Engine_input':True},'first_only':{'Engine_processes':1,'scorer_processes':1,'retry_rescore_relabel':0,'raw_readonly_before_scorer':True}}
 if not dt(m0['frozen_utc'])<=dt(author['utc'])<=dt(cases['authored_utc'])<=dt(scope['reviewed_utc'])<=dt(result['utc'])<=dt(manifest['frozen_utc']):raise ValueError('M0_author_M1_time_order')
 write_new(ROOT/'manifest.json',manifest)
 for n in frozen:os.chmod(ROOT/n,0o444)
 os.chmod(ROOT/'manifest.json',0o444)
 print(json.dumps({'stage':'COMPLETE_M1_FREEZE','manifest_sha256':sha(ROOT/'manifest.json'),'Engine_invocations':0,'scorer_invocations':0}))
if __name__=='__main__':main()
