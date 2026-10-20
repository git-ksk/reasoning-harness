from pathlib import Path
from datetime import datetime,timezone
import json,hashlib,shutil,subprocess,importlib.util,sys
sys.dont_write_bytecode=True
r=Path.cwd();f=Path('/Users/sawadakousuke/.codex/worktrees/engine073-source26-freeze/reasoning-harness');src=f/'evaluation/engine073-launch-independent-v26';dest=r/'evaluation/engine073-launch-independent-v26';audit=r/'evaluation/engine073-launch-v26-parent-verification'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
anchor='8214db3607e32d707da797fb90bb4af77b1101db976b6f0d0a60f96d49cdb921';final=src/'final-artifact-manifest.json';assert sha(final)==anchor
m=json.loads(final.read_text());assert len(m['files'])==828 and m['outcome']=='FAIL' and m['decision']=='NO_GO'
for rel,h in m['files'].items():
 p=Path(rel);assert not p.is_absolute() and '..' not in p.parts and sha(f/p)==h,rel
assert sha(src/'evaluator-freeze.json')=='ad980eba8479eec16e78f5ef2c2d108ac949ed1cf4b6fe2277df4dc0e3742510'
assert sha(src/'manifest.json')=='7627332228e422c58b64890434f681066469d77ab3326a8037e9b8379508c366'
for n,a in m['checked_artifacts'].items():assert sha(a['path'])==a['sha256'] and Path(a['path']).stat().st_size==a['bytes'],n
m0=json.loads((src/'evaluator-freeze.json').read_text());m1=json.loads((src/'manifest.json').read_text())
for rel,h in m0['frozen_files'].items():assert sha(src/rel)==h and m1['frozen_files'][rel]==h,rel
for rel,h in m1['frozen_files'].items():assert sha(src/rel)==h,rel
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=f,text=True).strip()=='9082365ffecfafaa9ba0ac65ce56368ac77d4228'
assert sha(r/'crates/reasoning-harness-core/src/evidence_relevance.rs')=='4f82b2e41614394f1bd528f01c817f0e962f1c546d7212f75f15c2f9c5787999'
rows=[json.loads(s) for s in (src/'first.stdout.jsonl').read_text().splitlines()];assert len(rows)==60;byid={x['id']:x for x in rows};assert len(byid)==60
results=json.loads((src/'first-results.json').read_text());summary=json.loads((src/'first-summary.json').read_text());assert results['summary']==summary
assert summary['outcome']=='FAIL' and summary['decision']=='NO_GO' and summary['incomplete']==0 and not summary['operational_errors'] and summary['passed']==52
assert len(summary['runtime_provenance']['checks'])==13 and all(v is True for v in summary['runtime_provenance']['checks'].values())
for x in results['cases']:
 raw=byid[x['id']];assert x['raw_observation']==raw
 assert x['observed']=={'v17_requested_relation':raw['effective_v17']['relation_scope']=='requested_relation','v30_relevant':raw['v30']['disposition']=='relevant'}
inv=json.loads((src/'first-invocation.json').read_text());score_inv=json.loads((src/'first-score.invocation.json').read_text());driver=json.loads((src/'driver.invocation.json').read_text())
assert type(inv['returncode'])is int and inv['returncode']==0 and inv['execution_error'] is None and score_inv['returncode']==0 and driver['returncode']==1
assert inv['stdout_sha256']==sha(src/'first.stdout.jsonl') and inv['stderr_sha256']==sha(src/'first.stderr.txt')
assert all((src/n).stat().st_size==0 for n in ['first.stderr.txt','first-score.stderr.txt','driver.stderr.txt'])
files=sorted(p for p in src.rglob('*') if p.is_file());assert len(files)==99
assert {str(p.relative_to(f)) for p in files}=={rel for rel in m['files'] if rel.startswith('evaluation/engine073-launch-independent-v26/')}|{'evaluation/engine073-launch-independent-v26/final-artifact-manifest.json'}
for p in files:assert p.stat().st_mode&0o222==0,p
assert not dest.exists();shutil.copytree(src,dest)
for p in files:assert sha(p)==sha(dest/p.relative_to(src)),p
for rel,h in m['files'].items():assert sha(r/rel)==h,rel
old=r/'evaluation/engine073-launch-independent-v25/final-artifact-manifest.json';assert sha(old)=='f3ef8db2393aa6c458981e92099d23792856dd1e1431c5876bb51760aefab1aa'
spec=importlib.util.spec_from_file_location('parent_first_arithmetic',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
current=module.corpus_metrics(dest);assert current['status']=='MEASURED'
for axis,metrics in current['axes'].items():
 assert metrics['confusion']=={'tp':22,'fp':0,'tn':30,'fn':8}
 for key,oldkey in [('precision','precision'),('recall','recall'),('f1_positive','f1')]:assert metrics[key]==summary['axes'][axis][oldkey]
previous=json.loads((r/'evaluation/engine073-launch-first-metrics-v25-report.json').read_text());report=dict(previous,current_version='v26',final_archive_anchor=anchor,verified_archive_files=824,verified_preparation_files=5,verified_all_corpus_reference_files=829,archive_manifest_path='evaluation/engine073-launch-independent-v26/final-artifact-manifest.json',current_independent_final_review_complete=True,versions=dict(previous['versions'],v26=current),arithmetic_only='Existing corpus_metrics applied to saved first checks/labels; no Engine/scorer/helper invocation or semantic reinterpretation. Prior versions copied unchanged from v25 report.',previous_report_sha256=sha(r/'evaluation/engine073-launch-first-metrics-v25-report.json'))
(r/'evaluation/engine073-launch-first-metrics-v26-report.json').write_text(json.dumps(report,indent=2)+'\n')
review={'verified_utc':datetime.now(timezone.utc).isoformat(),'outcome':'FIRST_ARTIFACTS_VERIFIED','semantic_first_outcome':'FAIL/NO_GO','anchor':anchor,'new_files':99,'old_preserved_references':730,'all_corpus_references':829,'M0_sha256':sha(src/'evaluator-freeze.json'),'M1_sha256':sha(src/'manifest.json'),'parent_M0_review_sha256':sha(audit/'review.json'),'source_candidate':'9082365ffecfafaa9ba0ac65ce56368ac77d4228','source_sha256':sha(r/'crates/reasoning-harness-core/src/evidence_relevance.rs'),'runtime_sha256':sha('/private/tmp/engine073-independent-v26-runtime/probe'),'arithmetic':current,'provenance13_checks':'PASS','canonical_engine_processes':1,'canonical_scorer_processes':1,'canonical_helper_calls':1,'parent_additional_Engine_scorer_helper_calls':0,'parent_readonly_verification':True,'copy_bytes_all_exact':True,'retry_rescore_relabel':0,'separate_development':'Parent cfg(test) consumed eight-miss route trace is separate, not independent rerun or rescoring.','candidate_freeze_release':'Not released by this capture; root commit/push/agent notice follows.'}
(audit/'final-copy-verification.json').write_text(json.dumps(review,indent=2)+'\n')
print(json.dumps({'anchor':anchor,'new_files':99,'old_refs':730,'total_refs':829,'first':'FAIL/NO_GO','confusion':{'tp':22,'fp':0,'tn':30,'fn':8},'F1':current['axes']['v30_relevant']['f1_positive'],'readonly_copy':'PASS','arithmetic':'PASS'}))
