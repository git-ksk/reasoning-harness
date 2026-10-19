"""Bind real host outputs to source/admission bytes before semantic annotation."""
import json
from pathlib import Path
from trace_audit import bind as bind_trace_audit
from scorer import require,sha,keyed,check_model_journal,check_terminal_order

def bind_normal(payload,trace,journal,config,corpus,model_events,provider,model):
    result=payload.get('result',{});require(result.get('output_contract')=='reason-natural-output-v4' and 'failure' not in result,'normal_run_incomplete')
    require(result['configuration']['provider']==provider and result['configuration']['model']==model,'runtime_identity_mismatch')
    pinned=json.loads((Path(__file__).parent/'protocol.json').read_text())['normal_bound']['safety_runtime']
    require(result.get('safety_runtime')==pinned,'unfrozen_safety_runtime')
    check_terminal_order(trace,result)
    usage=check_model_journal(model_events,provider,model,24)
    bind_trace_audit(trace,model_events,provider,model)
    require(payload.get('schema_version')=='reason-cli-output-v1' and payload.get('command')=='ask','CLI_envelope_contract')
    cases=[c for c in corpus['cases'] if c['id']==corpus['_expected_case_id']]
    require(len(cases)==1 and result.get('task')==cases[0]['question'],'acquisition_question_mismatch')
    case=cases[0]
    logical=result.get('usage',{}).get('run',{}).get('model_calls')
    require(type(logical) is int and 0<logical<=12,'logical_operation_budget')
    usage['logical_model_operations']=logical
    terminals=[e for e in trace['events'] if e['kind']=='finalization_state']
    grounded=[e for e in trace['events'] if e['kind']=='grounding_state']
    pre_terminal_grounded=[e for e in grounded if e['sequence']<terminals[-1]['sequence']]
    require(terminals[-1].get('result')==result['finalization'],'actual_display_terminal_mismatch')
    require(grounded and grounded[-1].get('outcome')==result['final_outcome'],'artifact_terminal_mismatch')
    artifact=result['final_outcome']['artifact'];evidence=keyed(artifact['evidence']);claims=keyed(artifact['claims']);receipts=keyed(artifact['verification_receipts'])
    acquired={};journal_requests={}
    for event in journal:
        require(event.get('corpus_sha256')==corpus['_capture_sha256'],'acquisition_wrong_corpus')
        require(event.get('case')==case['id'],'mixed_acquisition_cases')
        require(event['request'].get('schema_version')=='reason-investigation-external-resolver-request-v1' and event['response'].get('schema_version')=='reason-external-resolver-response-v1' and event['response'].get('contribution',{}).get('kind')=='acquired_evidence','resolver_contract')
        request=event['request']['request'];rid=request['id'];require(rid not in journal_requests,'duplicate_acquisition_request')
        journal_requests[rid]=event
        for record in event['response']['contribution']['evidence']:
            require(record['id'] not in acquired,'ambiguous_acquired_evidence')
            sid=record['source'];require(sid in corpus['sources'] and sid in case['source_ids'],'unknown_acquired_source')
            source=corpus['sources'][sid];visible=json.loads(record['observation'])
            for k in ['quote','context_summary','document_id','url','locator','source_version','origin_group','scope','synthetic_mutation']:
                require(visible.get(k)==source[k],'changed_acquired_source_field:'+k)
            require(record['facts']==source['reference_facts'],'changed_host_curated_facts')
            acquired[record['id']]=record
    caps=keyed(config['resolution']['investigation']['capabilities']);admitted={}
    for event in trace['events']:
        if event['kind']!='evidence_admission':continue
        rid='investigation:'+event['target_id']+':'+event['capability_id']
        require(rid in journal_requests and event['capability_id'] in caps,'admission_without_same_target_acquisition')
        policy=caps[event['capability_id']]['admission'];acquisition=journal_requests[rid]
        request=acquisition['request']['request']
        require(request.get('target',{}).get('target_id')==event['target_id'],'acquisition_wrong_target')
        require(request.get('target',{}).get('expected_fact_key')==case['lookup_key'],'acquisition_wrong_fact_key')
        task_scope={k:{'kind':'values','values':[v]} for k,v in case['required_scope'].items()}
        require(policy['required_scope']==task_scope,'task_admission_scope_mismatch')
        require(event['sequence']<pre_terminal_grounded[-1]['sequence']<terminals[-1]['sequence'],'admission_after_finalization')
        available=keyed(acquisition['response']['contribution']['evidence'])
        for eid in event['admitted_evidence_ids']:
            require(eid not in admitted and eid in available and eid in evidence,'unacquired_admission')
            a,b=available[eid],evidence[eid]
            require(all(a[k]==b[k] for k in ['id','source','observation','facts']),'admission_content_changed')
            source=corpus['sources'][a['source']];owner=policy['sources'].get(a['source']);require(owner is not None,'source_not_authorized_by_host')
            scope={k:{'kind':'values','values':[v]} for k,v in source['scope'].items()}
            require(owner['scope']==scope and all(scope.get(k)==v for k,v in policy['required_scope'].items()),'admitted_wrong_scope')
            require(policy['authority_ranks'][owner['authority_class']]>=policy['authority_ranks'][policy['minimum_authority_class']],'authority_not_host_approved')
            at=source['retrieved_at_unix_seconds'];expected={'scope':scope,'provenance_class':owner['authority_class'],
                  'temporal':{'effective_from_unix_seconds':at,'effective_until_unix_seconds':at+owner['max_age_seconds']}}
            require(b['metadata']==expected,'admitted_metadata_changed')
            meta=a.get('acquisition_metadata',{});require(meta.get('scope')==scope and meta.get('observed_at_unix_seconds')==at and meta.get('retrieved_at_unix_seconds')==at,'acquisition_metadata_changed')
            require(at<=policy['evaluation_time_unix_seconds']<at+owner['max_age_seconds'],'capture_freshness_policy_mismatch')
            admitted[eid]={'source_id':a['source'],'admission_sequence':event['sequence'],'metadata':b['metadata']}
    require(set(admitted)==set(evidence),'artifact_has_unadmitted_evidence')
    for receipt in receipts.values():
        require(receipt.get('claim_id') in claims and receipt.get('proposition')==claims[receipt['claim_id']].get('proposition'),'orphan_receipt')
        require(all(eid in admitted for eid in receipt.get('evidence_ids',[])),'receipt_unadmitted_source')
        if receipt.get('conclusion')=='supported':
            require(receipt.get('verifier') in ['structured_fact_equality','qualified_structured_fact_equality'],'unsupported_receipt_verifier')
            ids=receipt.get('evidence_ids');require(isinstance(ids,list) and ids and len(set(ids))==len(ids),'empty_or_duplicate_receipt_evidence')
            require(set(ids)<=set(claims[receipt['claim_id']]['evidence_ids']),'receipt_not_claim_evidence')
            p=receipt['proposition'];require(all(evidence[eid]['facts'].get(p['key'])==p['value'] for eid in receipt['evidence_ids']),'supported_receipt_fact_mismatch')
    for claim in claims.values():
        if claim.get('state') in ['known','supported']:
            require(claim.get('evidence_ids') and all(eid in admitted for eid in claim['evidence_ids']),'supported_claim_without_admission')
            require(any(r.get('claim_id')==claim['id'] and r.get('conclusion')=='supported' for r in receipts.values()),'supported_claim_without_receipt')
    text=result['finalization'].get('text')
    require(text is None or isinstance(text,str),'nontext_terminal')
    # Fixed canonical API surface exposes no citation objects. Do not turn
    # internal evidence IDs into displayed citations or manufacture a sentence.
    return {'display_artifacts':[{'id':'answer','kind':'final_answer','text':text or ''}], 'display_citations':[],
            'acquired_source_ids':sorted({e['source'] for e in acquired.values()}),
            'admitted_source_ids':sorted({e['source_id'] for e in admitted.values()}),'admission_routes':admitted,
            'internal_claims':list(claims.values()),'verification_receipts':list(receipts.values()),'usage':usage,
            'terminal_status':result['finalization']['status'],'ordinary_feature_effect':'not inferred from dependency version'}

def bind_helper(result,corpus,provider,model):
    require(result.get('operational_failure') is None,'helper_run_incomplete')
    usage=check_model_journal(result['model_events'],provider,model,1)
    outputs={}
    for arm in ['baseline','candidate']:
        raw=result[arm];require(raw['artifact_unchanged'] and raw['replay_equal'] and not raw['claims_promoted'] and not raw['unauthorized_reviewed'],'helper_authority_or_replay_violation')
        final=raw['original'] if arm=='baseline' else raw['view']['target_answers'][0]['original']
        text=final.get('text') or '';citations=[]
        # The paired host emits the Engine's actual source-local canonical text
        # and structured citations. Attach each citation to its own source line.
        slots={}
        source_ids=result['source_ids'];sources=[corpus['sources'][sid] for sid in source_ids]
        def tail(n):return f"[source:{source_ids[n]}; evidence:e:{n}; binding:b:{n}]"
        if sources[0]['quote']==sources[1]['quote']:
            prefix="According to "+', '.join(sorted(set(source_ids)))+": "
            expected=prefix+sources[0]['quote']+' '+tail(0)+' '+tail(1)
            slots={0:len(prefix),1:len(prefix)}
        else:
            pieces=[];cursor=0
            for n,s in enumerate(sources):
                prefix=f"According to {source_ids[n]}: ";slots[n]=cursor+len(prefix)
                piece=prefix+s['quote']+' '+tail(n);pieces.append(piece);cursor+=len(piece)+1
            expected=' '.join(pieces)
        require(text==expected,'helper_canonical_source_text_mismatch')
        for n,c in enumerate(final.get('citations',[])):
            sid=c['source_id'];require(sid in corpus['sources'],'helper_source_missing')
            slot=int(c['binding_id'].split(':')[1]);require(slot in [0,1] and source_ids[slot]==sid,'helper_wrong_binding_source')
            source=corpus['sources'][sid];quote=source['quote'];where=slots[slot]
            citations.append({'id':'citation-'+str(n),'artifact_id':'answer','start_byte':len(text[:where].encode()),'end_byte':len(text[:where+len(quote)].encode()),
                'source_id':sid,'source_version':c.get('source_version'),'url':c.get('source_url'),'locator':c.get('locator'),'literal_quote':quote,
                'captured_engine_claim_id':c['claim_id'],'binding_id':c['binding_id']})
        outputs[arm]={'display_artifacts':[{'id':'answer','kind':'final_answer','text':text}],'display_citations':citations,
                      'acquired_source_ids':result['source_ids'],'admitted_source_ids':result['source_ids'],
                      'usage':usage if arm=='candidate' else {'generations':0,'input_tokens':0,'output_tokens':0,'total_tokens':0,'provider_attempts':0},
                      'terminal_status':raw['status'],'shared_advice_source':'one live untrusted adviser, replayed identically for Engine isolation',
                      'host_authority_scope':'independent-agent prebound controlled-source review; not human attestation or truth'}
    return outputs
