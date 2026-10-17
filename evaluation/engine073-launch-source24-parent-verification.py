from pathlib import Path
import hashlib,json,sys,subprocess,importlib.util,stat
r=Path.cwd();anchor=sys.argv[1]
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
spec=importlib.util.spec_from_file_location('metrics',r/'evaluation/engine073-launch-first-metrics-v1/metrics.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
report=m.build_report(r,'v24',anchor)
assert report['current_independent_final_review_complete']
current=r/'evaluation/engine073-launch-independent-v24';final=json.loads((current/'final-artifact-manifest.json').read_text())
prep=final['preserved_preparation_files'];assert len(prep)==5
for rel,h in prep.items():assert sha(r/rel)==h
f=json.loads(Path('/private/tmp/engine073-source24-freeze.json').read_text());assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==f['source_sha']=='3ccda6b4263eb5210477660b11f6926ec0186f4e'
source=r/'crates/reasoning-harness-core/src/evidence_relevance.rs';assert sha(source)==f['source_file_sha256']=='85a64b506f8c47828a4a3296c5d07c13e921c59b528ea27e10ec4993c5f636e2'
for k in ['reasoning_harness_core','serde_json']:assert sha(Path(f[k]['path']))==f[k]['sha256']
files=[p for p in current.rglob('*') if p.is_file()];assert all(not p.stat().st_mode & (stat.S_IWUSR|stat.S_IWGRP|stat.S_IWOTH) for p in files)
v=report['versions']['v24'];assert v['source_sha']==f['source_sha']
manifest=json.loads((current/'manifest.json').read_text());assert manifest['candidate']['source_sha']==f['source_sha']
record={'source_sha':f['source_sha'],'source_file_sha256':sha(source),'candidate_artifacts_unchanged':True,'final_archive_anchor':anchor,'verified_source_files':report['verified_archive_files'],'separate_preparation_files':len(prep),'all_referenced_files':report['verified_archive_files']+len(prep),'new_files_readonly':len(files),'first_metrics':v,'parent_verification_Engine_scorer_provider_calls':0,'first_retry_rescore_relabel':0,'scope':'read-only verification and arithmetic on original first outputs; not new Engine/scorer run or practical acceptance'}
for name,data in [('engine073-launch-first-metrics-v24-report.json',report),('engine073-launch-v24-parent-final-verification.json',record)]:
 p=r/'evaluation'/name
 with p.open('x') as fp:fp.write(json.dumps(data,indent=2)+'\n')
print(json.dumps(record,indent=2))
