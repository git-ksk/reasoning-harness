#!/usr/bin/env python3
"""Aggregate saved first-scorer booleans only. No Engine, classifier, or rescoring."""
from pathlib import Path
import json,hashlib,os
ROOT=Path(__file__).resolve().parent
sha=lambda q:hashlib.sha256(q.read_bytes()).hexdigest()
def main():
 manifest=json.loads((ROOT/'manifest.json').read_text())
 for n,h in manifest['frozen_files'].items():assert sha(ROOT/n)==h,n
 result=json.loads((ROOT/'first-results.json').read_text());summary=json.loads((ROOT/'first-summary.json').read_text())
 assert result['summary']==summary
 assert sha(ROOT/'first.stdout.jsonl')==summary['raw']['stdout_sha256']
 valid=[x for x in result['cases'] if x['status'] in ['pass','fail'] and all(type(x['observed'][k]) is bool for k in ['v17_requested_relation','v30_relevant'])]
 complete=len(valid)==60 and len({x['id'] for x in valid})==60 and summary['incomplete']==0 and summary['operational_errors']==[]
 axes={}
 for axis,key in [('v17','v17_requested_relation'),('v30','v30_relevant')]:
  counts={k:0 for k in ['TP','FP','TN','FN']}
  for x in valid:
   gold=x['polarity']=='positive';assert x['expected'][key] is gold
   pred=x['observed'][key]
   counts['TP' if gold and pred else 'FN' if gold else 'FP' if pred else 'TN']+=1
  tp,fp,tn,fn=[counts[k] for k in ['TP','FP','TN','FN']]
  ratios={}
  for name,num,den,zero in [('Precision',tp,tp+fp,'no_predicted_positives'),('Recall',tp,tp+fn,'no_gold_positives'),('F1',2*tp,2*tp+fp+fn,'no_gold_or_predicted_positives')]:
   ratios[name]={'numerator':num,'denominator':den,'value':num/den if complete and den else None,'null_reason':None if complete and den else zero if complete else 'operationally_incomplete'}
  axes[axis]={'counts':counts if complete else None,'observed_counts_diagnostic':counts,'ratios':ratios,'observed_negative_unsafe':fp,'unobserved_cases':60-len(valid)}
 out={'identity':'engine073-launch-independent-v24','stage':'POST_FIRST_SCORER_DIAGNOSTIC_AGGREGATION_ONLY','spec_sha256':sha(ROOT/'axis-metrics-spec.json'),'aggregator_sha256':sha(ROOT/'axis_metrics.py'),'first_results_sha256':sha(ROOT/'first-results.json'),'first_summary_sha256':sha(ROOT/'first-summary.json'),'complete':complete,'planned':60,'observed_valid':len(valid),'planned_positive':30,'planned_negative':30,'axes':axes,'joint_outcome':summary['outcome'],'joint_passed':summary['passed'],'ops':{'operational_errors':summary['operational_errors'],'operational_error_count':len(summary['operational_errors']),'incomplete':summary['incomplete']},'inherited_scorer_invocations':1,'additional_Engine_or_scorer_invocations':0,'gate':'All60/60 joint source cases and operational completeness still required; no F1 threshold substitution. Not answer grounding F1.'}
 with (ROOT/'axis-metrics.json').open('x') as f:json.dump(out,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
 print(json.dumps(out,indent=2))
if __name__=='__main__':main()
