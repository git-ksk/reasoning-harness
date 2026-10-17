from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,shutil,subprocess,importlib.util,sys
r=Path.cwd();s=Path('/Users/sawadakousuke/.codex/worktrees/engine073-source25-freeze/reasoning-harness');src=s/'evaluation/engine073-launch-independent-v25';dest=r/'evaluation/engine073-launch-independent-v25';anchor=sys.argv[1];sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not dest.exists();assert sha(src/'final-artifact-manifest.json')==anchor
assert sha(src/'evaluator-freeze.json')=='b172872d29b19e4308bf86bbce99b570f8d0198a087973def644f679fcb75ac6'
assert all(p.stat().st_mode&0o222==0 for p in src.rglob('*') if p.is_file())
assert sha(s/'crates/reasoning-harness-core/src/evidence_relevance.rs')=='9d8d3fa48a67f197d32282314813d992a374b9f54a6b079c39498b9a67f9207f'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=s,text=True).strip()=='95053007bba54c728eee428837b22eb9d52a4f61'
subprocess.run(['git','diff','--exit-code','9505300','--','crates','Cargo.toml','Cargo.lock'],check=True,stdout=subprocess.PIPE)
spec=importlib.util.spec_from_file_location('first_metrics',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
m.verified_versions(r,'v24','120f3c9e780512244c890635095069b100593ef8486300fba72390fcd2ae617a')
def verify_flat(repo):
 f=repo/'evaluation/engine073-launch-independent-v25/final-artifact-manifest.json';assert sha(f)==anchor
 idx=json.loads(f.read_text());files=idx['files'];selfrel=str(f.relative_to(repo));assert idx['self_excluded']==selfrel
 expected=set(idx['source_inventory']);prep=set(idx['preparation_inventory']);assert len(expected)==724 and len(prep)==5 and set(files)==expected|prep and len(files)==729
 actual=set()
 for version in range(1,26):
  q=repo/f'evaluation/engine073-launch-independent-v{version}'
  actual.update(str(x.relative_to(repo)) for x in q.rglob('*') if x.is_file())
 assert actual==expected|{selfrel} and len(actual)==725
 for rel,digest in files.items():
  rp=Path(rel);assert not rp.is_absolute() and '..' not in rp.parts
  assert sha(repo/rel)==digest,rel
 for a in idx['checked_artifacts'].values():assert sha(Path(a['path']))==a['sha256']
 return idx
verify_flat(s)
shutil.copytree(src,dest)
assert {str(p.relative_to(src)):sha(p) for p in src.rglob('*') if p.is_file()}=={str(p.relative_to(dest)):sha(p) for p in dest.rglob('*') if p.is_file()}
verify_flat(r)
report=m.build_report(r,'v24','120f3c9e780512244c890635095069b100593ef8486300fba72390fcd2ae617a')
report.update(current_version='v25',final_archive_anchor=anchor,verified_archive_files=725,verified_all_references_including_preparation=730,archive_manifest_path='evaluation/engine073-launch-independent-v25/final-artifact-manifest.json',manifest_schema='v25 flat repository-relative files + explicit source/preparation inventories; anchored verified before first-metric arithmetic')
report['versions']['v25']=m.corpus_metrics(dest)
(r/'evaluation/engine073-launch-first-metrics-v25-report.json').write_text(json.dumps(report,indent=2)+'\n')
summary=json.loads((dest/'first-summary.json').read_text());current=report['versions']['v25']
proof={'identity':'engine073-independent-v25-parent-verification','utc':datetime.now(timezone.utc).isoformat(),'source_sha':'95053007bba54c728eee428837b22eb9d52a4f61','source_file_sha256':sha(r/'crates/reasoning-harness-core/src/evidence_relevance.rs'),'evaluator_freeze_sha256':sha(dest/'evaluator-freeze.json'),'final_archive_anchor':anchor,'verified_archive_files':report['verified_archive_files'],'copied_files':len([p for p in dest.rglob('*') if p.is_file()]),'byte_identical_copy':True,'old632_plus_preparation5_unchanged':True,'first_outcome':summary['outcome'],'gate':summary.get('gate'),'original_first_summary_sha256':sha(dest/'first-summary.json'),'original_first_results_sha256':sha(dest/'first-results.json'),'parent_metrics_scope':'Read-only arithmetic from original first labels/observations; no new Engine/scorer/helper/canonical decision','Engine_scorer_provider_model_Oracle_calls':0,'real24':0,'release_cli_adoption':'NO_GO'}
if current['status']=='MEASURED':
 assert summary['observed_cases']==60 and summary['incomplete']==0 and not summary['operational_errors']
 for axis,a in current['axes'].items():
  g=summary['axes'][axis];assert a['confusion']=={k:g[k] for k in ('tp','fp','tn','fn')}
  assert a['precision']==g['precision'] and a['recall']==g['recall'] and a['f1_positive']==g['f1']
 proof['axes']=current['axes']
proofdir=r/'evaluation/engine073-launch-v25-parent-verification';assert not proofdir.exists();proofdir.mkdir()
(proofdir/'review.json').write_text(json.dumps(proof,indent=2)+'\n')
for srcfile,dst in [('/private/tmp/engine073-source25-preauthor-parent-review.json','preauthor-parent-review.json'),('/private/tmp/engine073-source25-cleanup.json','own-prototype-cleanup.json'),('/private/tmp/engine073-source25-doc-ci-final.json','prior-documentation-ci.json')]:shutil.copyfile(srcfile,proofdir/dst)
(proofdir/'artifact-sha256.json').write_text(json.dumps({p.name:sha(p) for p in sorted(proofdir.iterdir()) if p.is_file()},indent=2)+'\n')
for p in proofdir.iterdir():p.chmod(0o444)
print(json.dumps(proof,indent=2))
