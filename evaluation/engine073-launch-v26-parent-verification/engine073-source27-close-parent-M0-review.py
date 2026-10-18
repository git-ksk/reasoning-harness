from pathlib import Path
import json,hashlib,subprocess,sys,os
from datetime import datetime,timezone
root=Path('/Users/sawadakousuke/.codex/worktrees/engine073-source26-freeze/reasoning-harness')
d=root/'evaluation/engine073-launch-independent-v26'
out=Path('/private/tmp/engine073-source27-v26-parent-M0-review');out.mkdir(exist_ok=True)
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
m0=json.loads((d/'evaluator-freeze.json').read_text())
assert m0['identity']=='engine073-launch-independent-v26-pre-author'
assert m0['candidate']['source_sha']=='9082365ffecfafaa9ba0ac65ce56368ac77d4228'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()==m0['candidate']['source_sha']
assert sha(root/'crates/reasoning-harness-core/src/evidence_relevance.rs')=='4f82b2e41614394f1bd528f01c817f0e962f1c546d7212f75f15c2f9c5787999'
for rel,h in m0['frozen_files'].items():assert sha(d/rel)==h,rel
for name,a in m0['checked_artifacts'].items():
 p=Path(a['path']);assert sha(p)==a['sha256'] and p.stat().st_size==a['bytes'],name
assert sha(d/'source_gate_policy.json')=='3a6d37e3fbf134f4a49677fdcd7de8b1b625739988a40e1c1e78b94978099767'
assert sha(d/'source_gate_helper.py')=='00f6971900864ee9e89e4db6029d8196fabd0a4b6ab28e343f19506417735c6f'
for n in ['cases.json','inputs.json','author_cases.py','authoring-start.json','manifest.json','execution.started.json','first-invocation.started.json','first-score.started.json','first-results.json','first-summary.json']:
 assert not (d/n).exists(),n
old=json.loads((d/'predecessor-artifact-hashes.json').read_text());n=0
for version,files in old.items():
 p=root/'evaluation'/('engine073-launch-independent-'+version)
 assert {str(x.relative_to(p)) for x in p.rglob('*') if x.is_file()}==set(files),version
 for rel,h in files.items():assert sha(p/rel)==h,(version,rel)
 n+=len(files)
prep=json.loads((d/'preserved-preparation-hashes.json').read_text())
assert n==725 and len(prep)==5
for rel,h in prep.items():assert sha(root/rel)==h,rel
env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
assert (out/'pure-fixtures.stdout.txt').read_bytes()==b''
assert 'Ran 59 tests' in (out/'pure-fixtures.stderr.txt').read_text() and (out/'pure-fixtures.stderr.txt').read_text().rstrip().endswith('OK')
for rel,h in m0['frozen_files'].items():assert sha(d/rel)==h,rel
review={'reviewed_utc':datetime.now(timezone.utc).isoformat(),'stage':'PARENT_PRE_AUTHOR_M0_REVIEW','outcome':'PASS','evaluator_freeze_sha256':sha(d/'evaluator-freeze.json'),'candidate_commit':m0['candidate']['source_sha'],'source_sha256':sha(root/'crates/reasoning-harness-core/src/evidence_relevance.rs'),'probe_sha256':sha('/private/tmp/engine073-independent-v26-runtime/probe'),'old_independent_files':725,'preparation_files':5,'pure_fixture_tests':59,'pure_fixture_returncode':0,'review_record_repair':'attempt01 records formatting failure after PASS; existing pure logs retained, no fixture rerun','candidate_Engine_invocations':0,'case_authorship':0,'real_cohort_scorer_processes':0,'reviewed':['strict output enums and exact identity binding','duplicate/missing/refused/malformed rows fail closed','nonzero/bool/missing process envelope and stderr fail closed','one helper call with exact pinned config','per-axis F1/precision/recall and safety FP zero','source controls exclude pre-asserted future provenance','M0 precedes authoring, M1 precedes observation, raw readonly before scorer','exclusive one-run markers forbid retries/rescores','old archives and preparation exact'],'root_full_validation':{'code':'9082365ffecfafaa9ba0ac65ce56368ac77d4228','passed':1594,'failed':0,'existing_ignored':8,'strict_clippy_fmt':'PASS','known_controls':1340,'original14':'PASS','history597':'whole byte exact'},'limits':'Cooperative filesystem/agent provenance; synthetic source gate only, no real-answer or CLI acceptance.'}
(out/'review.json').write_text(json.dumps(review,indent=2)+'\n')
print(json.dumps(review,indent=2))
