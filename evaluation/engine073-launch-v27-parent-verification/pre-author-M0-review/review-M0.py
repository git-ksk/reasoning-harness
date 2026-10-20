from pathlib import Path
import json,hashlib,subprocess,sys,os
from datetime import datetime,timezone
root=Path('/Users/sawadakousuke/.codex/worktrees/engine073-source27-freeze/reasoning-harness')
d=root/'evaluation/engine073-launch-independent-v27'
out=Path('/private/tmp/engine073-source28-v27-parent-M0-review');out.mkdir()
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
m0=json.loads((d/'evaluator-freeze.json').read_text())
assert sha(d/'evaluator-freeze.json')=='aaa4b0d2494c3a89d87b3c0714847ee8d468f84295e1f868e7518398e41635bb'
assert len(m0['frozen_files'])==55 and len(m0['checked_artifacts'])==41
assert {str(x.relative_to(d)) for x in d.rglob('*') if x.is_file()}==set(m0['frozen_files'])|{'evaluator-freeze.json'}
assert all(x.stat().st_mode&0o222==0 for x in d.rglob('*') if x.is_file())
assert sha('/private/tmp/engine073-independent-v27-runtime/probe')=='b1c482ab5d20cf370868b27d0a3f62a67f1eb210f46731db7258016f56d31821'
assert m0['identity']=='engine073-launch-independent-v27-pre-author'
assert m0['candidate']['source_sha']=='26c98adbec19f9e8ea0d46f260f080d4319a8e60'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()==m0['candidate']['source_sha']
assert sha(root/'crates/reasoning-harness-core/src/evidence_relevance.rs')=='4daf64b918532a9004dea9807b8001ec743244b20b5cd1892c7f3a7ce316d1ef'
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
assert n==824 and len(prep)==5
for rel,h in prep.items():assert sha(root/rel)==h,rel
env=dict(os.environ,PYTHONDONTWRITEBYTECODE='1')
with (out/'pure-fixtures.stdout.txt').open('xb') as stdout,(out/'pure-fixtures.stderr.txt').open('xb') as stderr:
 p=subprocess.run([sys.executable,'-B','-m','unittest','test_evaluator'],cwd=d,env=env,stdout=stdout,stderr=stderr)
assert p.returncode==0
assert 'Ran 59 tests' in (out/'pure-fixtures.stderr.txt').read_text()
for rel,h in m0['frozen_files'].items():assert sha(d/rel)==h,rel
review={'reviewed_utc':datetime.now(timezone.utc).isoformat(),'stage':'PARENT_PRE_AUTHOR_M0_REVIEW','outcome':'PASS','evaluator_freeze_sha256':sha(d/'evaluator-freeze.json'),'candidate_commit':m0['candidate']['source_sha'],'source_sha256':sha(root/'crates/reasoning-harness-core/src/evidence_relevance.rs'),'probe_sha256':sha('/private/tmp/engine073-independent-v27-runtime/probe'),'old_independent_files':824,'preparation_files':5,'pure_fixture_tests':59,'pure_fixture_returncode':p.returncode,'candidate_Engine_invocations':0,'case_authorship':0,'real_cohort_scorer_processes':0,'reviewed':['strict output enums and exact identity binding','duplicate/missing/refused/malformed rows fail closed','nonzero/bool/missing process envelope and stderr fail closed','one helper call with exact pinned config','per-axis F1/precision/recall and safety FP zero','source controls exclude pre-asserted future provenance','M0 precedes authoring, M1 precedes observation, raw readonly before scorer','exclusive one-run markers forbid retries/rescores','old archives and preparation exact'],'root_full_validation':{'code':'26c98adbec19f9e8ea0d46f260f080d4319a8e60','passed':1599,'failed':0,'existing_ignored':8,'strict_clippy_fmt':'PASS','known_controls':1400,'original14':'PASS','history597':'whole byte exact'},'limits':'Cooperative filesystem/agent provenance; synthetic source gate only, no real-answer or CLI acceptance.'}
assert sha(d/'evaluator-freeze.json')=='aaa4b0d2494c3a89d87b3c0714847ee8d468f84295e1f868e7518398e41635bb'
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()==m0['candidate']['source_sha']
(out/'review.json').write_text(json.dumps(review,indent=2)+'\n')
print(json.dumps(review,indent=2))
