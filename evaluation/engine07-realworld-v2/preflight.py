#!/usr/bin/env python3
"""No model calls: invoke actual APIs, assert provenance/authority invariants."""
import argparse,copy,json,subprocess,tempfile
from pathlib import Path
from run import validate,verify_binaries
from score import score_helper
p=argparse.ArgumentParser();p.add_argument('--bin-dir',type=Path,required=True);a=p.parse_args()
corpus,protocol=validate();verify_binaries(a.bin_dir)
rows=[]
with tempfile.TemporaryDirectory() as tmp:
    for case in corpus['cases']:
        sources=[]
        for slot,sid in enumerate(case['source_ids']):
            s=copy.deepcopy(corpus['sources'][sid]);s['source_id']+=':slot:'+str(slot);sources.append(s)
        inp={'id':case['id'],'question':case['helper_question'],'sources':sources,'reference_authorized':case['helper_reference_review_authorized'],'advice_equivalent':True}
        path=Path(tmp)/'input.json';path.write_text(json.dumps(inp));outputs=[]
        for version in ('0.6.1','0.7.2'):
            result=subprocess.run([str(a.bin_dir.resolve()/('probe-'+version)),str(path)],capture_output=True,check=True)
            outputs.append(json.loads(result.stdout))
        row=score_helper(case,sources,*outputs,{'equivalent':True,'model_calls':0},deterministic_reference=True)
        assert not row['hard_violations'],row
        if case['helper_contract_expectation']=='review_eligible':assert row['candidate_status']=='reviewed_compatible',row
        if case['helper_contract_expectation'].startswith('must_refuse'):assert row['review_refusal'] and row['candidate_status']!='reviewed_compatible',row
        rows.append(row)
print(json.dumps({'valid':True,'model_used':False,'cases':len(rows),'review_eligible_positive_controls':sum(r['review_eligible'] for r in rows),'invalid_prebinding_diagnostic_cases':sum(r['prebinding_diagnostic_only'] for r in rows),'rows':rows},indent=2))
