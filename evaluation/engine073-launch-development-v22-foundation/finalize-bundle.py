from pathlib import Path
import json,hashlib,shutil,re,subprocess
r=Path.cwd();d=r/'evaluation/engine073-launch-development-v22-foundation';s=json.loads((d/'summary.json').read_text());f=json.loads(Path('/private/tmp/engine073-source22-freeze.json').read_text())
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==s['identity']['source_sha']==f['source_sha'];assert sha(r/'crates/reasoning-harness-core/src/evidence_relevance.rs')==s['identity']['source_file_sha256']==f['source_file_sha256']
for key in ['reasoning_harness_core','serde_json']:assert sha(Path(f[key]['path']))==f[key]['sha256']
for name in ['clippy','fmt','workspace','providers','fixtures','archive']:shutil.copyfile(f'/private/tmp/engine073-foundation22-conflict-verified-{name}.log',d/(name+'.log'))
for src,dst in [('engine073-source22-freeze.json','source-freeze.json'),('engine073-source22-build-artifacts.jsonl','cargo-build-artifacts.jsonl'),('engine073-source22-build.log','build.log'),('engine073-foundation22-measure-pinned.py','measure-pinned-source.py'),('engine073-foundation22-measure-pinned.log','measurement.log'),('engine073-foundation22-repair.py','foundation-repair.py'),('engine073-foundation22-conflict-fix.py','competing-owner-repair.py'),('engine073-foundation22-conflict-focused.log','focused-pass.log'),('engine073-finalize-foundation22.py','finalize-bundle.py')]:shutil.copyfile(Path('/private/tmp')/src,d/dst)
rows=[]
for name in ['workspace','providers']:
 text=(d/(name+'.log')).read_text();assert 'FAILED' not in text;rows+=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',text)
passed=sum(int(x[0]) for x in rows);ignored=sum(int(x[2]) for x in rows);assert passed==1564 and ignored==8,(passed,ignored)
assert 'Finished' in (d/'clippy.log').read_text() and 'error:' not in (d/'clippy.log').read_text();assert not (d/'fmt.log').read_bytes();fixtures=json.loads((d/'fixtures.log').read_text());assert fixtures['corpus']['stratification']['cases']==20;archive=json.loads((d/'archive.log').read_text());assert archive['archive_verified'] and archive['files_with_exact_sha256']==13
assert len(s['seen_regressions'])==18 and sum(x['passed'] for x in s['seen_regressions'].values())==1080
a=s['v22']['after_development_axes'];assert a['v30_relevant']['confusion']==dict(tp=9,fp=0,tn=30,fn=21) and a['v17_requested_relation']['confusion']==dict(tp=10,fp=0,tn=30,fn=20)
assert s['v22']['historical_v29_all60_equal'];inv=json.loads((d/'development-invocations.json').read_text());assert len(inv)==21 and all(x['returncode']==0 and x['stderr_empty'] for x in inv)
validation={'workspace':{'passed':passed,'failed':0,'ignored':ignored},'strict_clippy':'PASS','fmt':'PASS','fixtures':20,'original_archive_files':13,'source_sha':f['source_sha'],'source_file_sha256':f['source_file_sha256'],'full_current_acceptance':'FAIL positive misses remain','fresh_independent_after_this_semantic_change':'NOT_RUN; resolve21 consumed misses before authoring another corpus','merge_release_cli_adoption':'NO_GO','provider_model_Oracle_calls':0}
(d/'validation-summary.json').write_text(json.dumps(validation,indent=2)+'\n')
text=f'''# Limited foundation repair after independentv22

Pinned actual source `{f['source_sha']}`, source SHA256 `{f['source_file_sha256']}`; actual core `{f['reasoning_harness_core']['sha256']}`. Correct compiler-artifact metadata/top/deps rlib equality, source equal HEAD, identity saved **before** development invocation and checked afterward. Original independentv22 firstFAIL30/60 and all551 archived files remain unchanged. This is consumed same-input development, not fresh acceptance.

Complete claim/assertion/statement wrappers preserve property scope only if their full inner proposition passes existing checks; arbitrary/nested/unknown/event/global bodies remain blocked. Equivalent article/unowned forms require a unique same-role foreign owner. The [first repair](../engine073-launch-development-v22-foundation-rejected/README.md) raised F1 but produced unsafeFP1 because a second owner declaration was skipped; it was rejected. The final repair includes explicit role-specific competing owner statements in the uniqueness check, with target/different-foreign/unknown conflicts and same-owner/split-role controls. It does not erase denials or introduce a generic graph/public API.

Same60 frozenv22 inputs/labels, baseline7431333 first output versus currentc20c513 development output:

| API / stage | TP | FP | TN | FN | Precision | Recall | F1 positive | API coverage |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| v30 baseline first | 0 | 0 | 30 | 30 | undefined | 0 | 0 | 0 |
| v30 current development | 9 | 0 | 30 | 21 | 1.0000 | 0.3000 | 0.4615 | 0.1500 |
| v17 baseline first | 0 | 0 | 30 | 30 | undefined | 0 | 0 | 0 |
| v17 current development | 10 | 0 | 30 | 20 | 1.0000 | 0.3333 | 0.5000 | 0.1667 |

Both current specificities1.0; v30 balanced accuracy/accuracy0.65. One visible v17 relation still has blocking effective context_gap, so v30 remains ambiguous. API coverage is not CLI answer rate. Precision1.0 describes only9/10 positive predictions here, not general safety. No cross-corpus improvement claim, no original first rescore, and no independent PASS. Current joint39/60 stillFAIL with21 positive misses. Scorer invocations0;21 development public-API probe processes (1751 inputs including historical/audit/replays), provider/model/Oracle calls0. Historicalv29 objects match all60 baseline observations.

Validation: old1080/1080 consumed regressions; original14/14;597 historical complete outputs byte-identical;workspace1564 PASS/0 FAIL/8 existing ignored;strict Clippy/fmt;20 fixtures;13 immutable original archive files. Two new test groups cover wrapper scope and owner-reference/competition boundaries. The first unsafe repair/partial superseded validation, root fixture correction and temporary diagnostic compile failure stay archived separately.

Next: diagnose remaining21 on current source, then bounded physical/document predicates (has/prints/uses and measured values), printed literals with outside property/owner proof, foreign event quote/apposition and the canonical Clip false-gap case. Each needs positive/negative controls without owner/type transfer. Keep this fixed input as development regression, measure F1 andFP after each change, preserve old first results; new semantic source freeze precedes another genuinely new independent evaluation. Full real24=0/24, adoption1/6; runner live callback/dependency/numeric-budget and ordinaryCLI usefulness remain open. No merge, Engine release or CLI adoption.
'''
(d/'README.md').write_text(text)
(d/'artifact-sha256.json').write_text(json.dumps({p.name:sha(p) for p in sorted(d.iterdir()) if p.is_file()},indent=2)+'\n')
print(json.dumps(validation,indent=2))
