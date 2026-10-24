#!/usr/bin/env python3
"""Recheck saved inert CLI outputs, without HTTP, scoring or relabelling."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--observations', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    sys.path.insert(0, str(args.runtime.resolve()))
    from capture import bind_normal, bind_helper
    from scorer import check_model_journal, check_terminal_order
    from preparation_guard import require_loopback_preparation
    from trace_audit import sanitize
    from run_lane import check_plan
    from development import checked_receipt

    root = args.observations.resolve()
    rows = []
    acquisition_inputs = []
    paths = sorted((root / 'mac-preflight-first').glob('*'))
    paths += sorted((root / 'mac-preflight-continuation').glob('*'))
    for directory in paths:
        if not directory.is_dir():
            continue
        arm, provider, mode = directory.name.split('-', 2)
        read = lambda name: json.loads((directory / name).read_bytes())
        lines = lambda name: [json.loads(line) for line in (directory / name).read_text().splitlines()]
        payload = read('stdout.json')
        events = lines('model-journal.jsonl')
        usage = check_model_journal(events, provider, 'fixture-model', 24)
        invocation = read('invocation.json')
        count = len(read('requests.json'))
        if mode == 'success':
            assert invocation['returncode'] == 0 and count == 2
            final = check_terminal_order(read('trace.json'), payload['result'])
            assert final['result'] == payload['result']['finalization']
            assert final['result']['status'] == 'grounded_answer'
        elif mode == 'budget-stop':
            result = payload['result']
            assert invocation['returncode'] != 0 and count == 1
            assert result['failure']['failure_class'] == 'usage_budget_exceeded'
            assert result['partial_diagnostics']['complete'] is False
            assert result['partial_diagnostics']['usage_is_lower_bound'] is False
            trace = read('trace.json')
            assert trace['completion_status'] == 'partial'
            assert not any(e['kind'] == 'finalization_state' for e in trace['events'])
        elif mode == 'acquisition':
            assert invocation['returncode'] == 0 and count == 5
            corpus = read('corpus.json')
            corpus.update(_capture_sha256=digest(directory / 'corpus.json'), _expected_case_id='engineering-01')
            inputs = [payload, read('trace.json'), lines('resolver.jsonl'), read('config.json'),
                      corpus, events, provider, 'fixture-model']
            captured = bind_normal(*inputs)
            assert captured['terminal_status'] == 'qualified_partial_answer'
            assert captured['acquired_source_ids'] == captured['admitted_source_ids'] == ['s0']
            assert len(captured['display_citations']) == 2
            assert {c['attachment'] for c in captured['display_citations']} == {'fact', 'quote'}
            acquisition_inputs.append(inputs)
        elif mode == 'helper':
            assert invocation['returncode'] == 0 and count == 1
            captured = bind_helper(payload, read('corpus.json'), provider, 'fixture-model')
            assert set(captured) == {'baseline', 'candidate'}
        else:
            raise ValueError('unexpected control')
        rows.append({'arm': arm, 'provider': provider, 'mode': mode,
                     'pass': True, 'loopback_requests': count, 'usage': usage})
    assert len(rows) == 17 and len(acquisition_inputs) == 2

    mutations = []
    for index, original in enumerate(acquisition_inputs):
        for name in ['citation_url', 'unacquired_citation', 'quote_span', 'missing_safety',
                     'request_fact_key', 'request_question', 'changed_acquired_record',
                     'source_promotes_withheld', 'coverage_promotes_withheld']:
            inputs = copy.deepcopy(original)
            result, trace, journal = inputs[0]['result'], inputs[1], inputs[2]
            citation = result['answer_context']['citations'][0]
            if name == 'citation_url':
                citation['url'] += '/tampered'
            elif name == 'unacquired_citation':
                binding = inputs[3]['answer_context']['sources'][1]
                for key in ['proposition', 'evidence_id', 'source_id', 'evidence_sha256', 'url', 'source_version']:
                    citation[key] = binding[key]
            elif name == 'quote_span':
                citation['quote_start_byte'] += 1
            elif name == 'missing_safety':
                trace['events'] = [e for e in trace['events'] if e.get('stage') != 'answer_safety']
                for sequence, event in enumerate(trace['events']):
                    event['sequence'] = sequence
            elif name == 'request_fact_key':
                journal[0]['request']['request']['target']['expected_fact_key'] = 'other.key'
            elif name == 'request_question':
                journal[0]['request']['request']['target']['question'] = 'different target'
            elif name == 'changed_acquired_record':
                journal[0]['response']['contribution']['evidence'][0]['facts']['unit.value'] = '5'
            else:
                context = result['answer_context']
                target = context['selected_claims'][0]['proposition']
                hard = copy.deepcopy(context['hard_finalization'])
                hard.update(status='requires_verification', text=None, uncovered_propositions=[target])
                context['hard_finalization'] = hard
                coverage = result['acquisition_coverage']
                for observation in result['safety_observations']:
                    observation['observation']['disposition'] = 'force_verification'
                for event in trace['events']:
                    if event['kind'] == 'answer_safety_state':
                        event['observation']['observation']['disposition'] = 'force_verification'
                    if event.get('stage') == 'answer_safety':
                        event['result'] = sanitize(hard)
                if name == 'source_promotes_withheld':
                    coverage['base_finalization']['uncovered_propositions'] = [target]
                    result['finalization']['uncovered_propositions'] = [target]
                else:
                    context.update(citations=[], unresolved_inputs=[], input_reasons={})
                    coverage['base_finalization'] = hard
                    coverage['plan_answer_complete'] = False
                    result['finalization']['uncovered_propositions'] = [target]
                for event in trace['events']:
                    if event.get('stage') == 'source_context':
                        event['result'] = sanitize(coverage['base_finalization'])
                    elif event['kind'] == 'acquisition_coverage':
                        event['view'] = sanitize(coverage)
                    elif event.get('stage') == 'acquisition_coverage':
                        event['result'] = sanitize(result['finalization'])
            try:
                bind_normal(*inputs)
            except ValueError as error:
                mutations.append({'arm_index': index, 'mutation': name, 'rejected': str(error)})
            else:
                raise AssertionError('mutation accepted: ' + name)

    env = {'ENGINE073_LOOPBACK_BASE_URL': 'http://127.0.0.1:1234/', 'GEMINI_API_KEY': 'inert-loopback-only'}
    require_loopback_preparation('fixture-model', env)
    for model, changed in [('real-model', env), ('fixture-model', {}),
                           ('fixture-model', {**env, 'GEMINI_API_KEY': 'not-an-API-key'}),
                           ('fixture-model', {**env, 'ENGINE073_LOOPBACK_BASE_URL': 'https://example.invalid/'})]:
        try:
            require_loopback_preparation(model, changed)
        except ValueError:
            pass
        else:
            raise AssertionError('preparation admitted non-inert callback')
    for callback in [lambda: check_plan({}, {'state': 'PREPARATION_NO_LIVE_ALLOWED'}),
                     lambda: checked_receipt(Path('/nonexistent-receipt'), 'google')]:
        try:
            callback()
        except ValueError as error:
            assert str(error) == 'paired_runtime_preparation_only'
        else:
            raise AssertionError('preparation admitted execution')

    output = {'state': 'OFFLINE_ENGINEERING_CONTROLS_PASS_NOT_ACCEPTANCE', 'controls': rows,
              'saved_output_mutation_checks': mutations, 'preparation_guard_controls': 7,
              'new_accuracy_cases': 0, 'external_provider_requests': 0, 'F1_recomputed': False,
              'independent_review_complete': False, 'Linux_containment_validated': False,
              'verification_script_sha256': digest(Path(__file__)),
              'original_consumed_score_sha256': digest(root.parent / 'engine073-realworld-v2-development/first-consumed-score-v1/report.json')}
    assert output['original_consumed_score_sha256'] == 'bfe04912745e2236a8d1995e0166f57aa6cd396e988229fc168aafc2dea37051'
    with args.output.open('x') as handle:
        json.dump(output, handle, indent=2)
        handle.write('\n')
    print(json.dumps({key: output[key] for key in ['state', 'new_accuracy_cases', 'external_provider_requests', 'F1_recomputed']}))


if __name__ == '__main__':
    main()
