#!/usr/bin/env python3
"""Static input-contract stage only: no Engine/provider/model invocation."""
from pathlib import Path
import ast,hashlib,json
ROOT=Path(__file__).resolve().parent

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    data=json.loads((ROOT/'cases.json').read_text());cases=data['cases'];inputs=json.loads((ROOT/'inputs.json').read_text())
    budget={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
    assert len(cases)==60 and len({c['id'] for c in cases})==60 and len({c['pair_id'] for c in cases})==30
    assert sum(c['polarity']=='positive' for c in cases)==30 and sum(c['polarity']=='negative' for c in cases)==30
    assert inputs==[c['input'] for c in cases]
    targets=set()
    for case,inp in zip(cases,inputs):
        assert set(inp)=={'id','policy','candidate','proposal','raw'}
        assert inp['id']==case['id']
        assert case['expected']=={'v17_requested_relation':case['polarity']=='positive','v30_relevant':case['polarity']=='positive'}
        p=inp['policy']
        assert set(p)=={'policy_id','target_id','target_question','entity','relation','identity_requirement','assessment_budget'}
        assert p['assessment_budget']==budget
        assert all(isinstance(x,int) and not isinstance(x,bool) and x>0 for x in p['assessment_budget'].values())
        assert p['relation']=='change_or_launch' and p['identity_requirement']=='require_harness_anchor'
        assert set(p['entity'])=={'canonical_id','canonical_name','aliases'} and p['entity']['canonical_id']==p['target_id'] and p['entity']['aliases']==[]
        target=p['entity']['canonical_name'];targets.add(target)
        assert p['target_question']==f'Was {target} launched on 14 March 2023?'
        candidate=inp['candidate'];assert set(candidate)=={'evidence_id','source_id','signals'}
        signals=candidate['signals'];assert signals and all(set(s)=={'kind','text'} and s['text'] and s['kind'] in ('excerpt','heading','source_title') for s in signals)
        assert sum(s=={'kind':'excerpt','text':f'{target} was launched on 14 March 2023.'} for s in signals)==1
        assert inp['proposal']=={'target_binding':'exact','relation_binding':'exact'}
        assert inp['raw']=={'identity_scope':'exact_target','relation_scope':'requested_relation','scope_risk':'none'}
    assert len(targets)==30
    assert sha(ROOT/'score.py')=='9a9941bf7ab3ea8ac22e1c851940197fdf3ef65ffb975c1b6fc2ddaa0130f4e3'
    assert sha(ROOT/'probe.rs')=='18d24664f5851f2bf63e9e440c49e3500468a5f8bb183e71abfd122c1d89ea8d'
    for name in ('author_cases.py','score.py','first-run.py','static_preflight.py'):ast.parse((ROOT/name).read_text())
    predecessors=json.loads((ROOT/'predecessor-artifact-hashes.json').read_text())
    for version,files in predecessors.items():
        d=ROOT.parent/('engine073-launch-independent-'+version)
        assert {p.name for p in d.iterdir() if p.is_file()}==set(files)
        assert all(sha(d/name)==digest for name,digest in files.items())
    result={'identity':'engine073-launch-independent-v3','stage':'STATIC_ONLY','outcome':'PASS','planned_cases':60,'positive':30,'negative':30,'matched_pairs':30,'distinct_canonical_targets':30,'required_fields_valid':True,'budget':budget,'one_unadorned_target_subject_affirmative_per_case':True,'labels_excluded_from_engine_inputs':True,'scorer_and_probe_inherited_unchanged':True,'v1_v2_all_artifacts_unchanged':True,'semantic_observations':0,'engine_invocations':0,'provider_calls':0}
    with (ROOT/'static-preflight.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps(result,indent=2))
if __name__=='__main__':main()
