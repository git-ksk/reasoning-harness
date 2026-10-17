from pathlib import Path
import hashlib,json,subprocess,datetime,shutil
r=Path.cwd();d=r/'evaluation/engine073-launch-independent-v22'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
old=r/'evaluation/engine073-launch-independent-v21/final-artifact-manifest.json'
oldanchor='9db10bc88ef24b76cd6dd360bd0b23b90b32a19dac74cbbcb7c6f6dd851ee913'
assert sha(old)==oldanchor
versions=json.loads(old.read_text())['versions']
for v,files in versions.items():
 q=r/f'evaluation/engine073-launch-independent-{v}'
 assert {str(p.relative_to(q)) for p in q.rglob('*') if p.is_file()}==set(files)|({'final-artifact-manifest.json'} if v=='v21' else set())
 for name,h in files.items():assert sha(q/name)==h,(v,name)
versions['v21']['final-artifact-manifest.json']=oldanchor
assert sum(len(files) for files in versions.values())==516
f=json.loads(Path('/private/tmp/engine073-source21-freeze.json').read_text())
assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==f['source_sha']
assert sha(r/'crates/reasoning-harness-core/src/evidence_relevance.rs')==f['source_file_sha256']
for key in ['reasoning_harness_core','serde_json']:assert sha(Path(f[key]['path']))==f[key]['sha256']
m=json.loads((d/'manifest.json').read_text());assert sha(d/'manifest.json')=='21a8b511a426f5239a7f238fff88e1fc95f8f61dfc2e69b420659058dfd28727'
assert m['candidate']['source_sha']==f['source_sha']
for name,h in m['frozen_files'].items():assert sha(d/name)==h,name
for name,item in m['checked_artifacts'].items():assert sha(Path(item['path']))==item['sha256'],name
inv=json.loads((d/'first-invocation.json').read_text());assert inv['returncode']==0 and not inv['execution_error'] and inv['provider_calls']==inv['retries']==0
assert inv['pre_invocation_identity']['head']==f['source_sha']
assert inv['pre_invocation_identity']['frozen_files']==m['frozen_files']
for stream in ['stdout','stderr']:assert sha(d/f'first.{stream}.jsonl' if stream=='stdout' else d/'first.stderr.txt')==inv[f'{stream}_sha256']
s=json.loads((d/'first-summary.json').read_text());assert s==json.loads((d/'first-results.json').read_text())['summary']
assert s['candidate_source_sha']==f['source_sha'] and s['passed']==30 and s['failed']==30 and s['positive_passed']==0 and s['negative_passed']==30
assert s['planned_cases']==s['observed_cases']==60 and s['provider_calls']==s['retries']==s['incomplete']==0 and not s['operational_errors']
assert sha(d/'first.stdout.jsonl')==s['raw']['stdout_sha256'];assert sha(d/'first-invocation.json')==s['raw']['invocation_sha256']
assert not (d/'final-artifact-manifest.json').exists() and not (d/'receipt.json').exists()
versions['v22']={str(p.relative_to(d)):sha(p) for p in sorted(d.rglob('*')) if p.is_file()}
dest=r/'evaluation/engine073-launch-v22-parent-closure';dest.mkdir(exist_ok=False)
archive={'identity':'engine073-launch-v22-parent-closure','reviewer':'implementation parent; independent author unavailable after first result due usage limit','independent_final_review_complete':False,'predecessor_final_anchor':oldanchor,'versions':versions}
(dest/'archive-manifest.json').write_text(json.dumps(archive,indent=2)+'\n')
receipt={'identity':archive['identity'],'verified_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'archive_manifest_sha256':sha(dest/'archive-manifest.json'),'old516_unchanged':True,'v22_files':len(versions['v22']),'all_archive_files':sum(len(x) for x in versions.values()),'first_summary':s,'source':f,'source_core_serde_head_unchanged':True,'independent_first_run_complete':True,'independent_final_save_review_complete':False,'interruption':'Authorized evaluator reported first FAIL, then tool status errored at usage limit before receipt/README/final recursive manifest. Parent preserved all existing bytes and verified frozen/run/source identities.','parent_freeze_release':True,'release_reason':'Independent evaluator is stopped; parent archive closure preserves FAIL and interruption. No independent PASS/final completion is claimed.','Engine_scorer_provider_model_calls':0,'retry_relabel_original_artifact_edits':0,'merge_release_cli_adoption':'NO_GO'}
(dest/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
(dest/'README.md').write_text('''# V22 first result: FAIL; independent final review interrupted

The authorized independent evaluator authored/froze60 new cases and executed Engine once/scorer once. FirstFAIL30/60:positive0/30, negative30/30, unsafe0, positive misses30, operational/incomplete0. It reported the first result, then hit its usage limit before saving its receipt, README and final recursive manifest. Those missing steps are **not completed by the independent evaluator**.

The implementation parent performed hash/inventory closure only: old516 files unchanged; all existingv22 files preserved byte-for-byte; frozen author/cases/inputs/probe/scorer/recorder, first raw/result/summary and source/core/serde/HEAD identities verified. The parent-authored archive-manifest/receipt are outside the independent directory. They do not impersonate an independent final report or convert FAIL to acceptance. No Engine/scorer rerun, relabel or provider/model/Oracle calls. Parent released the freeze only after the evaluator stopped and this closure was saved. All original first results remain immutable; future semantic changes require a new pin and independent evaluation when available.

Source gate, merge/release and CLI adoption remain NO-GO. API F1/precision/recall/coverage are derivative first-observation metrics, separate from new real24 final-answer quality (0/24 unmeasured).
''')
shutil.copyfile('/private/tmp/engine073-parent-close-interrupted-v22.py',dest/'verify-and-close.py')
(dest/'artifact-sha256.json').write_text(json.dumps({p.name:sha(p) for p in sorted(dest.iterdir()) if p.is_file()},indent=2)+'\n')
print(json.dumps(receipt,indent=2))
