#!/usr/bin/env python3
"""Static contract check only. No Engine imports, execution, or outputs."""
import ast,hashlib,json,os
from pathlib import Path
ROOT=Path(__file__).resolve().parent
sha=lambda q:hashlib.sha256(q.read_bytes()).hexdigest()
def main():
 cases=json.loads((ROOT/'cases.json').read_text())['cases'];inputs=json.loads((ROOT/'inputs.json').read_text())
 assert len(cases)==60 and len({c['id'] for c in cases})==60 and len({c['pair_id'] for c in cases})==30
 assert sum(c['polarity']=='positive' for c in cases)==30 and sum(c['polarity']=='negative' for c in cases)==30
 assert inputs==[c['input'] for c in cases]
 targets=set();gaps=[]
 for c,x in zip(cases,inputs):
  assert set(x)=={'id','policy','candidate','proposal','raw'} and x['id']==c['id']
  assert c['expected']=={'v17_requested_relation':c['polarity']=='positive','v30_relevant':c['polarity']=='positive'}
  p=x['policy'];assert set(p)=={'policy_id','target_id','target_question','entity','relation','identity_requirement','assessment_budget'}
  assert p['assessment_budget']=={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
  assert all(isinstance(v,int) and not isinstance(v,bool) and v>0 for v in p['assessment_budget'].values())
  assert p['relation']=='change_or_launch' and p['identity_requirement']=='require_harness_anchor'
  assert set(p['entity'])=={'canonical_id','canonical_name','aliases'} and p['entity']['canonical_id']==p['target_id'] and p['entity']['aliases']==[]
  t=p['entity']['canonical_name'];targets.add(t);assert p['target_question']==f'Was {t} launched?'
  assert set(x['candidate'])=={'evidence_id','source_id','signals'}
  ss=x['candidate']['signals'];assert all(set(s)=={'kind','text'} and s['text'] and s['kind'] in ('excerpt','heading','source_title') for s in ss)
  assert sum(s=={'kind':'excerpt','text':f'{t} was launched.'} for s in ss)==1
  assert x['proposal']=={'target_binding':'exact','relation_binding':'exact'}
  risk='context_gap' if c['id'] in ['li15-026','li15-042'] else 'none'
  assert x['raw']=={'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':risk}
  if risk!='none':assert c['polarity']=='negative';gaps.append(c['id'])
  if t.endswith(' Clip'):assert any(t in s['text'] for s in ss if s['text']!=f'{t} was launched.')
 assert len(targets)==30
 assert sha(ROOT/'score.py')=='9a9941bf7ab3ea8ac22e1c851940197fdf3ef65ffb975c1b6fc2ddaa0130f4e3'
 assert sha(ROOT/'probe.rs')=='18d24664f5851f2bf63e9e440c49e3500468a5f8bb183e71abfd122c1d89ea8d'
 assert sha(ROOT/'first-run.py')=='b7e521fb087822acc054aec8c12c08a81d42bae678fd3f3cea4cad57625e8635'
 for n in ['author_cases.py','static_preflight.py','score.py','first-run.py']:ast.parse((ROOT/n).read_text())
 old=json.loads((ROOT/'predecessor-artifact-hashes.json').read_text())
 for v,fs in old.items():
  d=ROOT.parent/('engine073-launch-independent-'+v)
  assert {p.name for p in d.iterdir() if p.is_file()}==set(fs)
  assert all(sha(d/n)==h for n,h in fs.items())
 result={'identity':'engine073-launch-independent-v15','stage':'STATIC_ONLY','outcome':'PASS','planned_cases':60,'positive':30,'negative':30,'matched_pairs':30,'canonical_targets':30,'old_artifacts_verified':290,'complete_inputs_and_required_fields':True,'budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},'labels_excluded':True,'probe_scorer_recorder_byte_identical':True,'actual_context_gap_cases':gaps,'pure_risk_joint_negatives':0,'semantic_observations':0,'engine_invocations':0,'provider_calls':0}
 with (ROOT/'static-preflight.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
