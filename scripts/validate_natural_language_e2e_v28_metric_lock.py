#!/usr/bin/env python3
import ast
import json
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
PREV='natural-language-e2e-v27-freeze'
PREV_RUNNER='scripts/natural_language_e2e_v27.py'
PREV_ACCEPTANCE='scripts/natural_language_e2e_v27_acceptance.py'
PREV_PAIR='scripts/validate_natural_language_e2e_v27_pair.py'
CURRENT_RUNNER=ROOT/'scripts/natural_language_e2e_v28.py'
CURRENT_ACCEPTANCE=ROOT/'scripts/natural_language_e2e_v28_acceptance.py'
CURRENT_PAIR=ROOT/'scripts/validate_natural_language_e2e_v28_pair.py'

ALLOWED_CHANGED_RUNNER_FUNCTIONS={'validate_corpus','score_investigation','aggregate','main'}
EXPECTED_NEW_RUNNER_FUNCTIONS={'continuation_budget_allows'}
ALLOWED_CHANGED_ACCEPTANCE_FUNCTIONS={'validate_report','compare_pair','metrics','main'}
V12_SCORE_RETURN_EXCLUSIONS={
    'continuation_eligible','trigger_target_id','trigger_exact_target_bound',
    'mechanism_conformant','mechanism_classification','mechanism_followup_status',
    'downstream_followup_useful','followup_opportunity','useful_followup',
}
V12_AGGREGATE_RETURN_EXCLUSIONS={
    'continuation_eligible_cases','continuation_ineligible_trigger_cases',
    'mechanism_denominator','mechanism_conformant_cases','mechanism_conformance_rate',
    'mechanism_classification','downstream_useful_followup_cases','downstream_useful_followup_rate',
}
PRESERVED_SCORE_ASSIGNMENTS={
    'target_recalled','relevant','selected','action_observations','irrelevant','tool_selected',
    'trigger_exposed','avoidable_followup_stall','false_abstention','missed_insuff',
    'correctness','typed_operational_action_failures','generation_failure_observed',
    'generation_failure_class',
}
EXPECTED_CONTINUATION_HELPER='''
def continuation_budget_allows(actions, trigger_index, policy):
    if trigger_index is None or not isinstance(policy,dict): return False
    max_actions=int(policy.get('max_actions') or 0); max_no_progress=int(policy.get('max_no_progress_rounds') or 0)
    if max_actions<=0 or max_no_progress<=0: return False
    through=list(actions[:trigger_index+1])
    if len(through)>=max_actions: return False
    no_progress=0
    for action in through:
        progress=int(action.get('admitted_evidence') or 0)>0 or bool(action.get('verification_progress'))
        no_progress=0 if progress else no_progress+1
    return no_progress < max_no_progress
'''
EXPECTED_CANDIDATE_GATE="if role=='candidate' and eligible>0 and a.get('mechanism_conformance_rate')!=1.0: reasons.append('#249 candidate mechanism conformance must be 1.0 when legally executable')"

class LockError(Exception): pass

def git_show(path):
    cp=subprocess.run(['git','show',f'{PREV}:{path}'],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    if cp.returncode!=0:
        raise LockError(cp.stderr.strip() or f'cannot read {PREV}:{path}')
    return cp.stdout

def function_map(text):
    return {n.name:n for n in ast.parse(text).body if isinstance(n,(ast.FunctionDef,ast.AsyncFunctionDef))}

def ast_id(node):
    return ast.dump(node,include_attributes=False)

def return_dict(fn):
    matches=[n.value for n in ast.walk(fn) if isinstance(n,ast.Return) and isinstance(n.value,ast.Dict)]
    if not matches: raise LockError(f'{fn.name}: return dict missing')
    return max(matches,key=lambda d: len(d.keys))

def dict_entries(fn):
    d=return_dict(fn); out={}
    for k,v in zip(d.keys,d.values):
        if isinstance(k,ast.Constant) and isinstance(k.value,str): out[k.value]=v
    return out

def assignments(fn,name):
    out=[]
    for n in ast.walk(fn):
        if not isinstance(n,(ast.Assign,ast.AnnAssign)): continue
        targets=n.targets if isinstance(n,ast.Assign) else [n.target]
        names=[]
        for t in targets:
            if isinstance(t,ast.Name): names.append(t.id)
            elif isinstance(t,(ast.Tuple,ast.List)):
                names.extend(x.id for x in t.elts if isinstance(x,ast.Name))
        if name in names: out.append(ast_id(n))
    return sorted(out)

def normalized_if_ids(fn):
    class Norm(ast.NodeTransformer):
        def visit_Constant(self,node):
            if isinstance(node.value,str):
                s=node.value
                s=s.replace('reason-natural-language-e2e-v27','reason-natural-language-e2e-vXX')
                s=s.replace('reason-natural-language-e2e-v28','reason-natural-language-e2e-vXX')
                s=s.replace('natural-language-e2e-v27','natural-language-e2e-vXX')
                s=s.replace('natural-language-e2e-v28','natural-language-e2e-vXX')
                s=s.replace('natural-language-e2e-scoring-v27-metric-locked-v11','<SCORING_ID>')
                s=s.replace('natural-language-e2e-scoring-v28-metric-locked-v12','<SCORING_ID>')
                return ast.copy_location(ast.Constant(s),node)
            if node.value in (94000,95100): return ast.copy_location(ast.Constant('<SEED>'),node)
            return node
    ids=[]
    for n in fn.body:
        if isinstance(n,ast.If):
            text=ast_id(n)
            if 'mechanism conformance' in text: continue
            ids.append(ast_id(Norm().visit(ast.fix_missing_locations(n))))
    return ids

def module_assignment(text,name):
    for n in ast.parse(text).body:
        if isinstance(n,ast.Assign) and len(n.targets)==1 and isinstance(n.targets[0],ast.Name) and n.targets[0].id==name:
            return ast.literal_eval(n.value)
    raise LockError(f'module assignment {name} missing')

def main():
    prev_runner_text=git_show(PREV_RUNNER); cur_runner_text=CURRENT_RUNNER.read_text()
    prev=function_map(prev_runner_text); cur=function_map(cur_runner_text)
    removed=sorted(set(prev)-set(cur))
    new=sorted(set(cur)-set(prev))
    changed=sorted(name for name in set(prev)&set(cur) if ast_id(prev[name])!=ast_id(cur[name]))
    unexpected_changed=sorted(set(changed)-ALLOWED_CHANGED_RUNNER_FUNCTIONS)
    helper_exact=(new==sorted(EXPECTED_NEW_RUNNER_FUNCTIONS) and ast_id(cur['continuation_budget_allows'])==ast_id(function_map(EXPECTED_CONTINUATION_HELPER)['continuation_budget_allows']))

    prev_score=dict_entries(prev['score_investigation']); cur_score=dict_entries(cur['score_investigation'])
    preserved_score_keys=sorted(set(prev_score)&set(cur_score)-V12_SCORE_RETURN_EXCLUSIONS)
    score_return_diffs=[k for k in preserved_score_keys if ast_id(prev_score[k])!=ast_id(cur_score[k])]
    score_assignment_diffs=[k for k in sorted(PRESERVED_SCORE_ASSIGNMENTS) if assignments(prev['score_investigation'],k)!=assignments(cur['score_investigation'],k)]

    prev_agg=dict_entries(prev['aggregate']); cur_agg=dict_entries(cur['aggregate'])
    preserved_agg_keys=sorted(set(prev_agg)&set(cur_agg)-V12_AGGREGATE_RETURN_EXCLUSIONS)
    aggregate_return_diffs=[k for k in preserved_agg_keys if ast_id(prev_agg[k])!=ast_id(cur_agg[k])]
    expected_new_aggregate_keys={'continuation_eligible_cases','continuation_ineligible_trigger_cases'}
    aggregate_new_keys=set(cur_agg)-set(prev_agg)

    prev_acc_text=git_show(PREV_ACCEPTANCE); cur_acc_text=CURRENT_ACCEPTANCE.read_text()
    pa=function_map(prev_acc_text); ca=function_map(cur_acc_text)
    acc_removed=sorted(set(pa)-set(ca)); acc_new=sorted(set(ca)-set(pa))
    acc_changed=sorted(name for name in set(pa)&set(ca) if ast_id(pa[name])!=ast_id(ca[name]))
    acc_unexpected=sorted(set(acc_changed)-ALLOWED_CHANGED_ACCEPTANCE_FUNCTIONS)
    compare_pair_if_equal=(normalized_if_ids(pa['compare_pair'])==normalized_if_ids(ca['compare_pair']))
    zero_equal=(module_assignment(prev_acc_text,'ZERO')==module_assignment(cur_acc_text,'ZERO'))
    candidate_gate_exact=(EXPECTED_CANDIDATE_GATE in cur_acc_text)
    control_gate_absent=("role=='control' and eligible" not in cur_acc_text and '#249 mechanism conformance must be 1.0 when exposed' not in cur_acc_text)

    prev_pair=function_map(git_show(PREV_PAIR)); cur_pair=function_map(CURRENT_PAIR.read_text())
    pair_scrub_equal=(ast_id(prev_pair['scrub'])==ast_id(cur_pair['scrub']))

    valid=all([
        not removed,
        new==sorted(EXPECTED_NEW_RUNNER_FUNCTIONS),
        not unexpected_changed,
        helper_exact,
        not score_return_diffs,
        not score_assignment_diffs,
        aggregate_new_keys==expected_new_aggregate_keys,
        not aggregate_return_diffs,
        not acc_removed,
        not acc_new,
        not acc_unexpected,
        compare_pair_if_equal,
        zero_equal,
        candidate_gate_exact,
        control_gate_absent,
        pair_scrub_equal,
    ])
    out={
      'schema_version':'natural-language-e2e-v28-metric-lock-validation-v2',
      'valid':valid,
      'locked_from':'natural-language-e2e-v11',
      'approved_metric_revision':'v12',
      'predecessor_freeze':PREV,
      'runner_changed_functions':changed,
      'runner_unexpected_changed_functions':unexpected_changed,
      'runner_new_functions':new,
      'continuation_budget_helper_exact':helper_exact,
      'preserved_score_return_keys_checked':len(preserved_score_keys),
      'preserved_score_return_diffs':score_return_diffs,
      'preserved_score_assignment_diffs':score_assignment_diffs,
      'preserved_aggregate_return_keys_checked':len(preserved_agg_keys),
      'preserved_aggregate_return_diffs':aggregate_return_diffs,
      'aggregate_new_keys':sorted(aggregate_new_keys),
      'acceptance_changed_functions':acc_changed,
      'acceptance_unexpected_changed_functions':acc_unexpected,
      'paired_utility_gate_if_ast_equal_to_v27':compare_pair_if_equal,
      'hard_zero_safety_fields_equal_to_v27':zero_equal,
      'candidate_only_mechanism_gate_exact':candidate_gate_exact,
      'released_control_mechanism_gate_removed':control_gate_absent,
      'pair_scrub_semantics_ast_equal_to_v27':pair_scrub_equal,
      'v12_scope':'only legal #249 continuation opportunity/conformance measurement and candidate-only hard mechanism gate; v11 recall/tool/stall/trigger/correctness/safety formulas remain locked',
    }
    print(json.dumps(out,indent=2,sort_keys=True))
    if not valid: raise SystemExit(2)

if __name__=='__main__': main()
