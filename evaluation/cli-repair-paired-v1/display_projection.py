"""Bind additive v5/v6 displays; semantic support remains independently judged."""
from copy import deepcopy
from trace_audit import sanitize
import json


def is_exposed(finalization):
    return finalization['status'] in ['grounded_answer', 'qualified_partial_answer'] and finalization.get('text') is not None


def same_claim_coverage(before, after, require):
    for key in ['factual_claims', 'covered_claims', 'factual_claim_coverage', 'uncovered_propositions']:
        require(before.get(key) == after.get(key), 'projection_changed_hard_coverage:' + key)


def source_transition(hard, view, display, require):
    same_claim_coverage(hard, display, require)
    if not is_exposed(hard):
        require(display == hard, 'source_context_promoted_withheld_answer')
        require(not view['citations'] and not view['unresolved_inputs'] and not view['input_reasons'], 'blocked_source_context_not_empty')
    elif not view['citations'] and not view['unresolved_inputs']:
        require(display == hard, 'empty_source_context_changed_answer')
    else:
        require(display['status'] == 'qualified_partial_answer' and isinstance(display.get('text'), str) and bool(display['text']), 'source_context_invalid_status')
        require(hard['factual_claims'] == len(view['selected_claims']), 'source_context_claim_inventory')


def coverage_transition(view, display, require):
    base = view['base_finalization']
    same_claim_coverage(base, display, require)
    admitted = view['ledger']['plan_admitted']
    targets = view['targets']
    require(type(admitted) is bool and [t['target'] for t in targets] == [t['target'] for t in view['ledger']['targets']], 'coverage_target_inventory')
    exposed = is_exposed(base)
    complete = admitted and exposed and all(t['disposition'] == 'verified' for t in targets)
    evidence_complete = admitted and all(t['disposition'] in ['verified', 'verified_not_selected'] for t in targets)
    require(type(view['plan_answer_complete']) is bool and view['plan_answer_complete'] == complete, 'coverage_answer_completeness')
    require(type(view['plan_evidence_complete']) is bool and view['plan_evidence_complete'] == evidence_complete, 'coverage_evidence_completeness')
    if complete:
        require(display == base, 'complete_coverage_changed_answer')
        return
    # The production coverage projection adds diagnostics only. Reconstruct its
    # exact text so a blocked hard result cannot gain factual output here.
    encode = lambda value: json.dumps(value, ensure_ascii=False, separators=(',', ':'))
    text = base['text'] if exposed else ''
    if text:
        text += '\n'
    text += 'Investigation coverage (admitted plan only; not proof that the whole question is answered):'
    if not admitted:
        text += '\nNo admitted plan is available; task coverage has not been established.'
    for target in targets:
        if target['disposition'] == 'verified' and exposed:
            continue
        data = target['target']
        disposition = ('evidence_verified_but_answer_not_released' if not exposed and target['disposition'] == 'verified'
                       else encode(target['disposition']))
        text += '\n- target ' + encode(data['id']) + ' / question ' + encode(data['question']) + ' / selector ' + encode(data.get('expected_fact_key')) + ': ' + disposition
    stop = encode(view['stop_reason']) if view['stop_reason'] is not None else 'not run in this turn'
    text += '\nInvestigation stopped: ' + stop + '. Unestablished targets have no asserted value.'
    expected = deepcopy(base)
    expected['text'] = text
    if base['status'] == 'grounded_answer':
        expected['status'] = 'qualified_partial_answer'
    require(display == expected, 'coverage_invalid_display_transition')


def peel(trace, result, require):
    events = trace.get('events', [])
    require(all(e.get('sequence') == n for n, e in enumerate(events)), 'trace_sequence')
    context = result.get('answer_context')
    coverage = result.get('acquisition_coverage')
    expected = []
    if context is not None:
        expected.append(('finalization_state', 'source_context'))
    if coverage is not None:
        expected.extend([('acquisition_coverage', None), ('finalization_state', 'acquisition_coverage')])
    if not expected:
        require(result.get('output_contract','reason-natural-output-v4') == 'reason-natural-output-v4', 'missing_projection_view')
        return trace, result
    require(result['output_contract'] == ('reason-natural-output-v6' if coverage is not None else 'reason-natural-output-v5'), 'projection_output_contract')
    suffix = events[-len(expected):]
    require([(e.get('kind'), e.get('stage')) for e in suffix] == expected, 'projection_stage_order')
    prefix = events[:-len(expected)]
    require(not any(e.get('kind') == 'acquisition_coverage' or e.get('stage') in ['source_context', 'acquisition_coverage'] for e in prefix), 'duplicate_projection')
    safety = [e for e in prefix if e.get('kind') == 'finalization_state']
    require(safety and safety[-1].get('stage') == 'answer_safety', 'projection_without_safety')
    hard = safety[-1]['result']
    renders = [e for e in prefix if e.get('kind') == 'final_render_state']
    require(renders, 'projection_without_selection')
    selected = renders[-1]['candidate']['factual_claims']
    source_display = coverage['base_finalization'] if coverage is not None else result['finalization']
    if context is not None:
        require(context['policy_id'] == 'harness-bound-source-context-v1', 'source_context_policy')
        require(sanitize(context['hard_finalization']) == hard and sanitize(context['selected_claims']) == selected, 'source_context_hard_binding')
        require(suffix[0]['phase'] == 'source_context' and suffix[0]['round'] == safety[-1]['round'] and suffix[0]['result'] == sanitize(source_display), 'source_context_trace_binding')
        source_transition(context['hard_finalization'], context, source_display, require)
    else:
        require(sanitize(source_display) == hard, 'coverage_base_without_context')
    if coverage is not None:
        require(coverage['policy_id'] == 'harness-investigation-coverage-v1', 'coverage_policy')
        require(sanitize(coverage['selected_claims']) == selected, 'coverage_selection_binding')
        require(suffix[-2]['view'] == sanitize(coverage), 'coverage_trace_view_binding')
        require(suffix[-1]['phase'] == 'acquisition_coverage' and suffix[-1]['round'] == safety[-1]['round'], 'coverage_trace_round')
        require(suffix[-1]['result'] == sanitize(result['finalization']), 'coverage_trace_display_binding')
        coverage_transition(coverage, result['finalization'], require)
    # The preserved hard-state validator sees all pre-projection events with
    # their original sequence numbers. No renderer/safety event is discarded.
    core_trace = deepcopy(trace)
    core_trace['events'] = prefix
    core_result = deepcopy(result)
    core_result['finalization'] = hard
    return core_trace, core_result


def citations(result, config, corpus, require, sha):
    context = result.get('answer_context')
    configured = config.get('answer_context')
    # Config is a pinned input artifact, not a public CLI output field. The
    # private Rust answer_context_config uses serde(skip); bind each exposed
    # citation to the input registry and captured source bytes below instead.
    if configured is None:
        require(context is None, 'unconfigured_source_context')
        return []
    require(context is not None, 'missing_configured_source_context')
    artifact = result['final_outcome']['artifact']
    evidence = {e['id']: e for e in artifact['evidence']}
    require(len(evidence) == len(artifact['evidence']), 'ambiguous_citation_evidence')
    text = (result['finalization'].get('text') or '').encode()
    registry = {(s['proposition']['key'], s['proposition']['value'], s['evidence_id']): s for s in configured['sources']}
    require(len(registry) == len(configured['sources']), 'duplicate_registry_binding')
    records = []
    seen = set()
    for index, citation in enumerate(context['citations']):
        key = (citation['proposition']['key'], citation['proposition']['value'], citation['evidence_id'])
        require(key in registry and key not in seen, 'foreign_or_duplicate_source_citation')
        seen.add(key)
        binding = registry[key]
        require(all(citation[k] == binding[k] for k in ['proposition', 'evidence_id', 'source_id', 'evidence_sha256', 'url', 'source_version']), 'citation_registry_mismatch')
        eid = citation['evidence_id']
        require(eid in evidence and evidence[eid]['source'] == citation['source_id'], 'citation_without_actual_evidence')
        proposition = citation['proposition']
        require(evidence[eid]['facts'].get(proposition['key']) == proposition['value'], 'citation_evidence_fact_mismatch')
        require(any(c.get('mode') == 'grounded' and c.get('proposition') == proposition for c in context['selected_claims']), 'citation_not_selected')
        supported = [c for c in artifact['claims'] if c.get('proposition') == proposition
                     and c.get('state') in ['known', 'supported'] and eid in c.get('evidence_ids', [])]
        require(any(r.get('claim_id') == c['id'] and r.get('proposition') == proposition
                    and r.get('conclusion') == 'supported' and eid in r.get('evidence_ids', [])
                    for c in supported for r in artifact['verification_receipts']), 'citation_without_claim_receipt')
        sid = citation['source_id']
        require(sid in corpus['sources'], 'citation_source_not_in_corpus')
        source = corpus['sources'][sid]
        require(citation['quote'] == source['quote'] and citation['quote_sha256'] == sha(source['quote'].encode()), 'citation_quote_mismatch')
        require(citation['url'] == source['url'] and citation['source_version'] == source['source_version'], 'citation_source_version_mismatch')
        require(citation['quote_display_encoding'] == 'json_string', 'citation_quote_encoding')
        spans = []
        for name in ['fact', 'quote']:
            start, end = citation[name+'_start_byte'], citation[name+'_end_byte']
            require(type(start) is int and type(end) is int and 0 <= start < end <= len(text), 'citation_span_range')
            try:
                value = text[start:end].decode('utf8')
            except UnicodeDecodeError as error:
                raise ValueError('citation_span_splits_UTF8') from error
            spans.append((name, start, end, value))
        require(json.loads(spans[1][3]) == citation['quote'], 'citation_encoded_quote_mismatch')
        proposition = citation['proposition']
        expected_fact = json.dumps(proposition['key'], ensure_ascii=False)+' = '+json.dumps(proposition['value'], ensure_ascii=False)
        require(spans[0][3] == expected_fact, 'citation_fact_span_mismatch')
        require(spans[0][2] <= spans[1][1], 'citation_quote_precedes_fact')
        # Two actual attachment regions, never one span crossing unrelated text.
        # Both are tied to the same structured citation and exact captured quote.
        for name, start, end, _ in spans:
            records.append({'id':f'context-{index}-{name}', 'artifact_id':'answer',
                'start_byte':start, 'end_byte':end, 'source_id':sid,
                'source_version':citation['source_version'], 'url':citation['url'],
                'locator':source['locator'], 'literal_quote':citation['quote'],
                'structured_citation_id':f'context-{index}', 'attachment':name,
                'captured_evidence_id':citation['evidence_id']})
    return records
