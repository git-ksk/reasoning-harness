from pathlib import Path
from datetime import datetime,timezone
import json,hashlib,shutil,subprocess,importlib.util,sys,os
sys.dont_write_bytecode=True
r=Path.cwd();f=Path('/Users/sawadakousuke/.codex/worktrees/engine073-source27-freeze/reasoning-harness');src=f/'evaluation/engine073-launch-independent-v27';dest=r/'evaluation/engine073-launch-independent-v27';audit=r/'evaluation/engine073-launch-v27-parent-verification'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
anchor=sys.argv[1];assert len(anchor)==64
final=src/'final-artifact-manifest.json';assert sha(final)==anchor
m=json.loads(final.read_text());assert m['identity']=='engine073-launch-independent-v27'
assert m['old_source_files']==824 and m['separate_preparation_files']==5
assert len(m['files'])==m['corpus_references_self_excluding_final_manifest']
assert m['corpus_references_including_final_manifest']==len(m['files'])+1
for rel,h in m['files'].items():
 p=Path(rel);assert not p.is_absolute() and '..' not in p.parts and sha(f/p)==h,rel
assert sha(src/'evaluator-freeze.json')=='aaa4b0d2494c3a89d87b3c0714847ee8d468f84295e1f868e7518398e41635bb'
assert sha(src/'manifest.json')==m['M1_sha256']
for n,a in m['checked_artifacts'].items():assert sha(a['path'])==a['sha256'] and Path(a['path']).stat().st_size==a['bytes'],n
m0=json.loads((src/'evaluator-freeze.json').read_text());m1=json.loads((src/'manifest.json').read_text())
for rel,h in m0['frozen_files'].items():assert sha(src/rel)==h and m1['frozen_files'][rel]==h,rel
for rel,h in m1['frozen_files'].items():assert sha(src/rel)==h,rel
code='26c98adbec19f9e8ea0d46f260f080d4319a8e60';source='4daf64b918532a9004dea9807b8001ec743244b20b5cd1892c7f3a7ce316d1ef'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=f,text=True).strip()==code
assert sha(r/'crates/reasoning-harness-core/src/evidence_relevance.rs')==source
rows=[json.loads(s) for s in (src/'first.stdout.jsonl').read_text().splitlines()];assert len(rows)==60
byid={x['id']:x for x in rows};assert len(byid)==60
results=json.loads((src/'first-results.json').read_text());summary=json.loads((src/'first-summary.json').read_text());assert results['summary']==summary
assert summary['incomplete']==0 and not summary['operational_errors']
assert summary['outcome']==m['outcome'] and summary['decision']==m['decision']
assert len(summary['runtime_provenance']['checks'])==13 and all(v is True for v in summary['runtime_provenance']['checks'].values())
for x in results['cases']:
 raw=byid[x['id']];assert x['raw_observation']==raw
 assert x['observed']=={'v17_requested_relation':raw['effective_v17']['relation_scope']=='requested_relation','v30_relevant':raw['v30']['disposition']=='relevant'}
inv=json.loads((src/'first-invocation.json').read_text());score_inv=json.loads((src/'first-score.invocation.json').read_text());driver=json.loads((src/'driver.invocation.json').read_text())
assert type(inv['returncode'])is int and inv['returncode']==0 and inv['execution_error'] is None and score_inv['returncode']==0 and driver['returncode']==(0 if summary['decision']=='GO' else 1)
assert inv['stdout_sha256']==sha(src/'first.stdout.jsonl') and inv['stderr_sha256']==sha(src/'first.stderr.txt')
assert all((src/n).stat().st_size==0 for n in ['first.stderr.txt','first-score.stderr.txt','driver.stderr.txt'])
files=sorted(p for p in src.rglob('*') if p.is_file());new=len(files);total=829+new
assert total==m['corpus_references_including_final_manifest']
assert {str(p.relative_to(f)) for p in files}=={rel for rel in m['files'] if rel.startswith('evaluation/engine073-launch-independent-v27/')}|{'evaluation/engine073-launch-independent-v27/final-artifact-manifest.json'}
for p in files:assert p.stat().st_mode&0o222==0,p
assert not dest.exists();shutil.copytree(src,dest)
for p in files:assert sha(p)==sha(dest/p.relative_to(src)),p
for rel,h in m['files'].items():assert sha(r/rel)==h,rel
assert sha(r/'evaluation/engine073-launch-independent-v26/final-artifact-manifest.json')=='8214db3607e32d707da797fb90bb4af77b1101db976b6f0d0a60f96d49cdb921'
spec=importlib.util.spec_from_file_location('parent_first_arithmetic',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
current=module.corpus_metrics(dest);assert current['status']=='MEASURED'
for axis,metrics in current['axes'].items():
 assert metrics['confusion']=={k:summary['axes'][axis][k] for k in ['tp','fp','tn','fn']}
 for key,oldkey in [('precision','precision'),('recall','recall'),('f1_positive','f1')]:assert metrics[key]==summary['axes'][axis][oldkey]
previous_path=r/'evaluation/engine073-launch-first-metrics-v26-report.json';previous=json.loads(previous_path.read_text())
report=dict(previous,current_version='v27',final_archive_anchor=anchor,verified_archive_files=824+new,verified_preparation_files=5,verified_all_corpus_reference_files=total,archive_manifest_path='evaluation/engine073-launch-independent-v27/final-artifact-manifest.json',current_independent_final_review_complete=True,versions=dict(previous['versions'],v27=current),arithmetic_only='Existing corpus_metrics applied to saved first checks/labels; no Engine/scorer/helper invocation or semantic reinterpretation. Prior versions copied unchanged from v26 report.',previous_report_sha256=sha(previous_path))
with (r/'evaluation/engine073-launch-first-metrics-v27-report.json').open('x') as fp:fp.write(json.dumps(report,indent=2)+'\n')
review={'verified_utc':datetime.now(timezone.utc).isoformat(),'outcome':'FIRST_ARTIFACTS_VERIFIED','semantic_first_outcome':summary['outcome']+'/'+summary['decision'],'anchor':anchor,'new_files':new,'old_preserved_references':829,'all_corpus_references':total,'M0_sha256':sha(src/'evaluator-freeze.json'),'M1_sha256':sha(src/'manifest.json'),'parent_M0_review_sha256':sha(audit/'pre-author-M0-review/review.json'),'source_candidate':code,'source_sha256':source,'runtime_sha256':sha('/private/tmp/engine073-independent-v27-runtime/probe'),'arithmetic':current,'provenance13_checks':'PASS','canonical_engine_processes':1,'canonical_scorer_processes':1,'canonical_helper_calls':1,'parent_additional_Engine_scorer_helper_calls':0,'parent_readonly_verification':True,'copy_bytes_all_exact':True,'retry_rescore_relabel':0,'scope':'Synthetic source gate only. Real24/actual answer F1/CLI/release gates remain separate.','candidate_freeze_release':'Not released by this capture; root commit/push/agent notice follows.'}
with (audit/'final-copy-verification.json').open('x') as fp:fp.write(json.dumps(review,indent=2)+'\n')
shutil.copyfile(__file__,audit/'capture-v27.py')
print(json.dumps({'anchor':anchor,'new_files':new,'old_refs':829,'total_refs':total,'first':review['semantic_first_outcome'],'axes':summary['axes'],'readonly_copy':'PASS','arithmetic':'PASS'}))
