#!/usr/bin/env python3
import ast
import json
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]
PREV='natural-language-e2e-v26-freeze'
CURRENT_RUNNER=ROOT/'scripts/natural_language_e2e_v27.py'
PREV_RUNNER='scripts/natural_language_e2e_v26.py'
LOCKED_RUNNER_FUNCTIONS=[
    'final_artifact','parse_exposed','artifact_supports','exposed_metrics','generation_costs',
    'collect_rejections','collect_operational_attempt_failures','score_investigation',
    'session_artifact','session_events','checkpoint_snapshot','fork_source_snapshot',
    'fork_checkpoint_state_matches','thread_event_payload','thread_event_kind','thread_event_change_kind',
    'aggregate','evaluate_report_gates',
]

class LockError(Exception): pass

def git_show(path):
    cp=subprocess.run(['git','show',f'{PREV}:{path}'],cwd=ROOT,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    if cp.returncode!=0: raise LockError(cp.stderr.strip() or f'cannot read {PREV}:{path}')
    return cp.stdout

def normalize(text):
    for token in ('v26','v27'): text=text.replace(token,'vXX')
    for token in ('V26','V27'): text=text.replace(token,'VXX')
    for token in ('93000','94000'): text=text.replace(token,'<SEED>')
    for token in ('2107af7942dd3cb7dac5fc30603359095f4a2709','c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7'):
        text=text.replace(token,'<CANDIDATE_COMMIT>')
    return text

def functions(text):
    tree=ast.parse(text)
    return {node.name:node for node in tree.body if isinstance(node,(ast.FunctionDef,ast.AsyncFunctionDef))}

def ast_identity(node):
    return ast.dump(node,include_attributes=False)

class SessionRetryNormalizer(ast.NodeTransformer):
    def visit_Call(self,node):
        self.generic_visit(node)
        if isinstance(node.func,ast.Name) and node.func.id=='attach_operational_retry' and node.args:
            return node.args[0]
        if isinstance(node.func,ast.Name) and node.func.id=='run_json_with_operational_retry':
            node.func=ast.Name(id='run_json',ctx=ast.Load())
            if len(node.args)>=4 and isinstance(node.args[-1],ast.Constant) and node.args[-1].value=='session_start':
                node.args=node.args[:-1]
        return node
    def visit_Assign(self,node):
        self.generic_visit(node)
        if len(node.targets)==1 and isinstance(node.targets[0],ast.Tuple):
            elts=node.targets[0].elts
            if len(elts)==4 and isinstance(elts[-1],ast.Name) and elts[-1].id=='start_retry':
                node.targets[0].elts=elts[:-1]
        return node

def normalized_session_identity(node):
    node=SessionRetryNormalizer().visit(ast.fix_missing_locations(node))
    return ast_identity(node)

def compare_normalized_file(current,previous):
    return normalize(Path(current).read_text())==normalize(git_show(previous))

def main():
    previous=functions(git_show(PREV_RUNNER)); current=functions(CURRENT_RUNNER.read_text())
    runner_diffs=[]
    for name in LOCKED_RUNNER_FUNCTIONS:
        if name not in previous or name not in current: runner_diffs.append(f'{name}:missing'); continue
        if ast_identity(previous[name])!=ast_identity(current[name]): runner_diffs.append(name)
    if 'run_session_case' not in previous or 'run_session_case' not in current:
        session_equal=False
    else:
        session_equal=ast_identity(previous['run_session_case'])==ast_identity(current['run_session_case'])

    file_checks={
      'acceptance_normalized_equal':compare_normalized_file(ROOT/'scripts/natural_language_e2e_v27_acceptance.py','scripts/natural_language_e2e_v26_acceptance.py'),
      'pair_validator_normalized_equal':compare_normalized_file(ROOT/'scripts/validate_natural_language_e2e_v27_pair.py','scripts/validate_natural_language_e2e_v26_pair.py'),
      'acceptance_tests_normalized_equal':compare_normalized_file(ROOT/'scripts/test_natural_language_e2e_v27_acceptance.py','scripts/test_natural_language_e2e_v26_acceptance.py'),
    }
    valid=not runner_diffs and session_equal and all(file_checks.values())
    out={
      'schema_version':'natural-language-e2e-v27-metric-lock-validation-v1',
      'valid':valid,
      'locked_from':'natural-language-e2e-v11',
      'predecessor_freeze':PREV,
      'runner_locked_function_diff_count':len(runner_diffs),
      'runner_locked_function_diffs':runner_diffs,
      'session_scoring_normalized_equal':session_equal,
      'session_scoring_exact_equal_to_v26':session_equal,
      **file_checks,
      'driver_addition':'v27 keeps the frozen v26 operational-only retry driver outside locked scoring semantics',
    }
    print(json.dumps(out,indent=2,sort_keys=True))
    if not valid: raise SystemExit(2)

if __name__=='__main__': main()
