#!/usr/bin/env python3
"""Static full-input contract check only, no Engine/provider imports or calls."""
from pathlib import Path
import ast,hashlib,json,os
ROOT=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 cases=json.loads((ROOT/'cases.json').read_text())['cases'];inputs=json.loads((ROOT/'inputs.json').read_text());targets=set()
 assert len(cases)==60 and len({c['id'] for c in cases})==60 and len({c['pair_id'] for c in cases})==30
 assert sum(c['polarity']=='positive' for c in cases)==30 and sum(c['polarity']=='negative' for c in cases)==30
 assert inputs==[c['input'] for c in cases]
 for c,x in zip(cases,inputs):
  assert set(x)=={'id','policy','candidate','proposal','raw'} and x['id']==c['id']
  assert c['expected']=={'v17_requested_relation':c['polarity']=='positive','v30_relevant':c['polarity']=='positive'}
  p=x['policy'];assert set(p)=={'policy_id','target_id','target_question','entity','relation','identity_requirement','assessment_budget'}
  assert p['assessment_budget']=={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000} and all(type(v)is int and v>0 for v in p['assessment_budget'].values())
  assert p['relation']=='change_or_launch' and p['identity_requirement']=='require_harness_anchor'
  assert set(p['entity'])=={'canonical_id','canonical_name','aliases'} and p['entity']['canonical_id']==p['target_id'] and p['entity']['aliases']==[]
  t=p['entity']['canonical_name'];targets.add(t);assert p['target_question']==f'Was {t} launched?'
  assert set(x['candidate'])=={'evidence_id','source_id','signals'}
  ss=x['candidate']['signals'];assert ss and all(set(s)=={'kind','text'} and s['text'] and s['kind'] in ['excerpt','heading','source_title'] for s in ss)
  assert sum(s=={'kind':'excerpt','text':f'{t} was launched.'} for s in ss)==1
  assert x['proposal']=={'target_binding':'exact','relation_binding':'exact'}
  risk='context_gap' if c['id'] in ['li19-030','li19-050'] else 'none'
  assert x['raw']=={'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':risk}
  if risk!='none':assert c['polarity']=='negative'
  if t.endswith(' Clip'):assert any(t in s['text'] for s in ss if s['text']!=f'{t} was launched.')
 assert len(targets)==30
 for n,h in [('probe.rs','18d24664f5851f2bf63e9e440c49e3500468a5f8bb183e71abfd122c1d89ea8d'),('score.py','9a9941bf7ab3ea8ac22e1c851940197fdf3ef65ffb975c1b6fc2ddaa0130f4e3'),('first-run.py','b7e521fb087822acc054aec8c12c08a81d42bae678fd3f3cea4cad57625e8635')]:assert sha(ROOT/n)==h
 for n in ['author_cases.py','static_preflight.py','score.py','first-run.py']:ast.parse((ROOT/n).read_text())
 old=json.loads((ROOT/'predecessor-artifact-hashes.json').read_text())
 for v,fs in old.items():
  d=ROOT.parent/('engine073-launch-independent-'+v);assert {q.name for q in d.iterdir() if q.is_file()}==set(fs)
  for n,h in fs.items():assert sha(d/n)==h
 result={'identity':'engine073-launch-independent-v19','stage':'STATIC_ONLY','outcome':'PASS','planned':60,'positive':30,'semantic_negative':30,'pairs':30,'targets':30,'required_fields_valid':True,'labels_excluded':True,'budget':{'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000},'inherited_probe_scorer_recorder_identical':True,'old_files_unchanged':402,'v11_v17_NOT_EVALUATED_unchanged':True,'raw_context_gap':['li19-030','li19-050'],'pure_raw_risk_joint_negatives':0,'semantic_observations':0,'Engine_invocations':0,'provider_network_calls':0}
 with (ROOT/'static-preflight.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
 print(json.dumps(result,indent=2))
if __name__=='__main__':main()
