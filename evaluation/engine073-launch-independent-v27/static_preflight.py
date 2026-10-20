#!/usr/bin/env python3
"""Future M1 full-input STATIC_ONLY stage; not run during pre-author preparation."""
import ast
from evaluator_contract import *
POSITIVE_BASIS={'non_launch_property','foreign_event','combined_safe_context'}
NEGATIVE_BASIS={'target_denial','global_denial','nonexistence','unknown_owner','mixed_owner','conflicting_owner','missing_context'}
def validate_corpus(casefile,inputs,scope):
 cases=casefile['cases']
 if casefile.get('identity')!='engine073-launch-independent-v27' or len(cases)!=60 or not isinstance(inputs,list) or len(inputs)!=60:raise ValueError('future60_required')
 if inputs!=[c['input'] for c in cases]:raise ValueError('input_label_separation_mismatch')
 if [c['id'] for c in cases]!=[f'li27-{i:03d}' for i in range(1,61)]:raise ValueError('future_unique_ordered_ids')
 if sum(c['polarity']=='positive' for c in cases)!=30 or sum(c['polarity']=='negative' for c in cases)!=30:raise ValueError('future30_30_required')
 targets=set();pairs={}
 for c,x in zip(cases,inputs):
  if not isinstance(c['rationale'],str) or not c['rationale'].strip():raise ValueError('independent_rationale_required')
  positive=c['polarity']=='positive'
  if c['semantic_basis'] not in (POSITIVE_BASIS if positive else NEGATIVE_BASIS):raise ValueError('semantic_reason_not_raw_risk')
  if not isinstance(c.get('expected'),dict) or set(c['expected'])!={'v17_requested_relation','v30_relevant'} or not all(type(v) is bool for v in c['expected'].values()):raise ValueError('gold_axes_require_strict_booleans')
  if c['expected']!={'v17_requested_relation':positive,'v30_relevant':positive}:raise ValueError('gold_axis_mismatch')
  if set(x)!={'id','policy','candidate','proposal','raw'} or x['id']!=c['id']:raise ValueError('labels_must_not_enter_Engine')
  policy=x['policy']
  if set(policy)!={'policy_id','target_id','target_question','entity','relation','identity_requirement','assessment_budget'}:raise ValueError('policy_schema')
  if policy['assessment_budget']!=BUDGET or not all(type(v)is int for v in policy['assessment_budget'].values()):raise ValueError('nonzero_frozen_budget')
  entity=policy['entity'];name=entity['canonical_name'];targets.add(name)
  if set(entity)!={'canonical_id','canonical_name','aliases'} or entity['aliases']!=[] or entity['canonical_id']!=policy['target_id']:raise ValueError('canonical_full_name_no_alias')
  if policy['target_question']!=f'Was {name} launched?' or policy['relation']!='change_or_launch' or policy['identity_requirement']!='require_harness_anchor':raise ValueError('finite_launch_policy')
  candidate=x['candidate']
  if set(candidate)!={'evidence_id','source_id','signals'} or not candidate['signals']:raise ValueError('candidate_schema')
  signals=candidate['signals']
  if not all(set(s)=={'kind','text'} and s['kind'] in ['excerpt','heading','source_title'] and isinstance(s['text'],str) and s['text'] for s in signals):raise ValueError('source_signals')
  if sum(s=={'kind':'excerpt','text':f'{name} was launched.'} for s in signals)!=1:raise ValueError('separate_unadorned_anchor')
  if x['proposal']!={'target_binding':'exact','relation_binding':'exact'}:raise ValueError('fixed_proposal')
  raw=x['raw']
  if set(raw)!={'identity_scope','relation_scope','scope_risk'} or raw['identity_scope']!='exact_target' or raw['relation_scope']!='requested_relation' or raw['scope_risk'] not in RISK:raise ValueError('fixed_independent_raw_axes')
  if raw['scope_risk']!='none' and (positive or not isinstance(c.get('raw_risk_reason'),str) or not c['raw_risk_reason'].strip()):raise ValueError('raw_risk_only_not_semantic_gold')
  if name.endswith(' Clip') and not any(name in s['text'] for s in signals if s['text']!=f'{name} was launched.'):raise ValueError('full_Clip_context_declaration')
  pairs.setdefault(c['pair_id'],[]).append(c)
 if len(targets)!=30 or len(pairs)!=30 or any(len(v)!=2 or {c['polarity'] for c in v}!={'positive','negative'} or len({c['input']['policy']['entity']['canonical_name'] for c in v})!=1 for v in pairs.values()):raise ValueError('30_semantic_pairs')
 if scope.get('reviewed_cases')!=60 or scope.get('labels_changed_after_observation') is not False or scope.get('Engine_invocations')!=0 or scope.get('scorer_invocations')!=0:raise ValueError('pre_observation_scope_review_required')
 return cases

def main():
 m0=evaluator_integrity();preserved()
 if any((ROOT/n).exists() for n in ['manifest.json','first-invocation.started.json','first-score.started.json']):raise ValueError('static_must_precede_observation_freeze')
 cases=load(ROOT/'cases.json');inputs=load(ROOT/'inputs.json');scope=load(ROOT/'scope-review.json');author=load(ROOT/'authoring-start.json')
 validate_corpus(cases,inputs,scope);ast.parse((ROOT/'author_cases.py').read_text())
 m0sha=sha(ROOT/'evaluator-freeze.json')
 if author['evaluator_freeze_sha256']!=m0sha or cases['evaluator_freeze_sha256']!=m0sha or cases['authoring_start_sha256']!=sha(ROOT/'authoring-start.json') or not author['authorization_reference']:raise ValueError('authoring_receipt_binding')
 if not dt(m0['frozen_utc'])<=dt(author['utc'])<=dt(cases['authored_utc'])<=dt(scope['reviewed_utc']):raise ValueError('authoring_before_evaluator_freeze')
 if scope['cases_sha256']!=sha(ROOT/'cases.json') or scope['inputs_sha256']!=sha(ROOT/'inputs.json'):raise ValueError('full_scope_inputs_binding')
 result={'identity':'engine073-launch-independent-v27','stage':'STATIC_ONLY','utc':utc(),'outcome':'PASS','planned':60,'positive':30,'negative':30,'pairs':30,'evaluator_freeze_sha256':m0sha,'cases_sha256':sha(ROOT/'cases.json'),'inputs_sha256':sha(ROOT/'inputs.json'),'scope_review_sha256':sha(ROOT/'scope-review.json'),'authoring_start_sha256':sha(ROOT/'authoring-start.json'),'Engine_invocations':0,'scorer_invocations':0,'provider_calls':0,'labels_excluded':True,'budget':BUDGET}
 write_new(ROOT/'static-preflight.json',result);print(json.dumps(result,indent=2))
if __name__=='__main__':main()
