"""Validate the actual CLI renderer/recovery/safety state machine."""
from collections import defaultdict
from copy import deepcopy

def check(trace, result, require):
    from trace_audit import sanitize
    events=trace.get('events');require(isinstance(events,list) and events,'trace_missing')
    requests={};responses={};terminals=[]
    for n,event in enumerate(events):
        require(type(event.get('sequence')) is int and event['sequence']==n,'trace_sequence')
        kind=event.get('kind')
        if kind=='model_failure':raise ValueError('trace_model_failure')
        if kind in ['model_request','model_response']:
            key=(event.get('correlation_id'),event.get('attempt'))
            require(isinstance(key[0],str) and key[0] and type(key[1]) is int and key[1]>0,'trace_model_identity')
            dst=requests if kind=='model_request' else responses
            require(key not in dst,'duplicate_trace_model_event')
            if kind=='model_response':require(key in requests,'response_before_request')
            dst[key]=n
        if kind=='finalization_state':terminals.append(event)
    require(terminals and requests.keys()==responses.keys(),'missing_terminal_or_response')
    last=terminals[-1]['sequence']
    require(all(requests[k]<responses[k]<last for k in requests),'response_after_terminal')
    require(trace.get('completion_status')=='completed','trace_not_completed')
    stages=['renderer_candidate','canonical_verified_target','canonical_verified_target_partial',
            'renderer_downgrade_recovery','canonical_verified_target_reject_partial','answer_safety']
    rounds=defaultdict(list)
    for event in terminals:
        n=event.get('round')
        require(type(n) is int and 1<=n<=2 and event.get('phase')=='finalization','invalid_finalization_round')
        stage=event.get('stage');require(stage in stages,'unknown_finalization_stage')
        final_value=event.get('result');require(isinstance(final_value,dict) and isinstance(final_value.get('status'),str),'invalid_finalization_result')
        require(final_value.get('text') is None or isinstance(final_value['text'],str),'nontext_terminal')
        rounds[n].append(event)
    require(list(rounds)==list(range(1,len(rounds)+1)),'finalization_round_order')
    observed_safety=[e.get('observation') for e in events if e.get('kind')=='answer_safety_state']
    require(observed_safety==sanitize(result.get('safety_observations',[])),'trace_output_safety_inventory_mismatch')
    runtime=result.get('safety_runtime');require(isinstance(runtime,dict) and runtime.get('profile'),'safety_runtime_missing')
    previous_end=-1;previous_terminal=None;previous_artifact=None;consumed_renders=set()
    for n,chain in rounds.items():
        names=[e['stage'] for e in chain];positions=[stages.index(x) for x in names]
        require(names[0]=='renderer_candidate' and names[-1]=='answer_safety','missing_renderer_or_safety_terminal')
        require(positions==sorted(set(positions)),'duplicate_or_reordered_finalization_stage')
        require(chain[0]['sequence']>previous_end,'interleaved_finalization_rounds')
        require(all(e.get('round')==n for e in chain),'mixed_finalization_rounds')
        grounding=[e for e in events if previous_end<e['sequence']<chain[0]['sequence'] and e.get('kind')=='grounding_state']
        require(grounding,'missing_round_grounding')
        artifact=grounding[-1]['outcome']['artifact']
        if previous_terminal is not None:
            require(previous_terminal['result']['status']=='requires_verification','retry_after_nonverification_terminal')
            retries=[e for e in grounding if e.get('phase')=='finalization_retry_resolution' and e.get('round')==n-1]
            require(len(grounding)==len(retries)==1 and retries[0]['outcome']['artifact']!=previous_artifact,'retry_without_changed_grounding')
        before=[e for e in events if previous_end<e['sequence']<chain[0]['sequence'] and e.get('kind')=='final_render_state' and e.get('round')==n]
        require(len(before)==1 and before[-1].get('source') in ['model','canonical_fallback'],'renderer_candidate_without_render')
        initial_render=before[0];consumed_renders.add(initial_render['sequence'])
        if initial_render['source']=='model':
            responses_for_render=[e for e in events[:initial_render['sequence']] if e.get('kind')=='model_response' and e.get('phase')=='final_render' and e.get('round')==n]
            require(responses_for_render,'renderer_without_actual_response')
            value=responses_for_render[-1].get('response',{}).get('value',{})
            require(initial_render['candidate']=={'text':value.get('text'),'factual_claims':value.get('factual_claims',[])},'render_not_actual_model_response')
        # Every recovery has an actual renderer event in its exact interval.
        for prior,event in zip(chain,chain[1:]):
            interval=events[prior['sequence']+1:event['sequence']]
            if event['stage']=='answer_safety':
                require(all(e.get('kind') not in ['final_render_state','grounding_state','evidence_admission'] for e in interval),'state_changed_during_answer_safety')
                safety=[e for e in interval if e.get('kind')=='answer_safety_state']
                require(all(e.get('round')==n and e.get('phase')=='answer_safety' and e['observation'].get('render_round')==n for e in safety),'safety_wrong_render_round')
                require(all(e['observation'].get('observation',{}).get('runtime')==sanitize(runtime) for e in safety),'safety_runtime_mismatch')
                renders=[e for e in events[:prior['sequence']] if e.get('kind')=='final_render_state' and e.get('round')==n]
                require(renders,'safety_without_rendered_candidate')
                targets=[]
                if runtime['profile']!='baseline' and prior['result']['status'] in ['grounded_answer','qualified_partial_answer']:
                    for claim in renders[-1]['candidate'].get('factual_claims',[]):
                        if claim.get('mode')=='grounded' and claim.get('proposition') not in targets:targets.append(claim.get('proposition'))
                require([e['observation']['observation'].get('target') for e in safety]==targets,'safety_target_inventory_mismatch')
                expected=deepcopy(prior['result']);blocked=[]
                for e in safety:
                    value=e['observation']['observation'];disposition=value.get('disposition')
                    require(disposition in ['preserve_baseline','force_verification'],'unknown_safety_disposition')
                    if disposition=='force_verification':blocked.append(value['target'])
                if blocked:
                    for target in blocked:
                        if target not in expected['uncovered_propositions']:expected['uncovered_propositions'].append(target)
                    expected.update(status='requires_verification',text=None)
                require(event['result']==expected,'safety_final_result_mismatch')
            else:
                matches=[e for e in interval if e.get('kind')=='final_render_state' and e.get('round')==n and e.get('source')==event['stage']]
                require(len(matches)==1,'recovery_without_matching_render');consumed_renders.add(matches[0]['sequence'])
                prior_status=prior['result']['status'];verdict=grounding[-1]['outcome'].get('verdict')
                if event['stage'] in ['canonical_verified_target','canonical_verified_target_partial']:
                    require(prior_status in ['unresolved','requires_verification'],'canonical_recovery_precondition')
                    require(verdict==('accept' if event['stage']=='canonical_verified_target' else 'unknown'),'canonical_recovery_verdict')
                elif event['stage']=='renderer_downgrade_recovery':
                    prior_renders=[e for e in events[:prior['sequence']] if e.get('kind')=='final_render_state' and e.get('round')==n]
                    require(prior_status=='qualified_partial_answer' and verdict in ['accept','unknown'] and any(c.get('mode')=='uncertain' for c in prior_renders[-1]['candidate']['factual_claims']),'downgrade_recovery_precondition')
                else:require(verdict=='reject','reject_partial_recovery_precondition')
        previous_end=chain[-1]['sequence'];previous_terminal=chain[-1];previous_artifact=artifact
    all_renders=[e for e in events if e.get('kind')=='final_render_state']
    require({e['sequence'] for e in all_renders}==consumed_renders,'orphan_or_duplicate_render')
    for render in all_renders:
        candidate=render.get('candidate')
        require(render.get('phase')=='final_render' and isinstance(candidate,dict) and isinstance(candidate.get('text'),str) and isinstance(candidate.get('factual_claims'),list),'invalid_render_schema')
        for claim in candidate['factual_claims']:
            proposition=claim.get('proposition')
            require(claim.get('mode') in ['grounded','uncertain'] and isinstance(proposition,dict) and all(isinstance(proposition.get(k),str) and proposition[k] for k in ['key','value']),'invalid_render_claim_schema')
    tail=events[last+1:]
    if tail:
        require(len(tail)==1 and tail[0].get('kind')=='grounding_state' and tail[0].get('phase')=='finalization_retry_resolution'
                and tail[0].get('round')==len(rounds) and previous_terminal['result']['status']=='requires_verification'
                and tail[0]['outcome']['artifact']==previous_artifact,'events_after_final_safety_terminal')
    return terminals[-1]
