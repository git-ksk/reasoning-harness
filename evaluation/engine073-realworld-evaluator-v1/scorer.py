"""Complete-cohort arithmetic after external, source-bound semantic adjudication.

The scorer never infers support from words, model confidence or gold equality.
External annotation remains a declared independent-agent judgment, not human truth.
"""
from collections import Counter, defaultdict
import hashlib
import importlib.util
import json
from pathlib import Path
import statistics
import sys

def require(ok,reason):
    if not ok: raise ValueError(reason)
def sha(data): return hashlib.sha256(data).hexdigest()
def integer(n): return type(n) is int and n>=0
def keyed(items,key='id'):
    require(isinstance(items,list),'list_required');out={}
    for item in items:
        require(isinstance(item,dict) and isinstance(item.get(key),str) and item[key] and item[key] not in out,'missing_or_duplicate_id')
        out[item[key]]=item
    return out
def load_module(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
def frozen_dependencies(repo):
    old=repo/'evaluation/engine073-answer-metrics-development-v1'
    sys.path.insert(0,str(old))
    return (load_module('claim_arithmetic',old/'claim_metrics.py'),
            load_module('display_binding',old/'display_binding.py'),
            load_module('case_quality',repo/'evaluation/engine07-quality-gates-v1/metrics.py'))

def check_model_journal(events,provider,model,max_generations):
    """Full ModelResponse is required; empty metadata never proves completion."""
    require(isinstance(events,list) and events,'model_journal_missing')
    pending=None;responses=[];requests=[];total_in=total_out=attempts=0;wire=False;http=False
    for event in events:
        require(isinstance(event,dict),'invalid_model_event')
        if event.get('kind')=='model_request':
            require(pending is None,'concurrent_or_uncompleted_generation')
            n=event.get('generation');require(type(n) is int and n==len(requests)+1,'model_generation_order')
            require(event.get('provider')==provider and event.get('model')==model,'wrong_provider_model')
            req=event.get('request');require(isinstance(req,dict) and isinstance(req.get('task'),str),'invalid_model_request')
            require(type(req.get('max_tokens')) is int and 0<req['max_tokens']<=1024,'invalid_generation_output_bound')
            pending=n;requests.append(event);wire=False;http=False
        elif event.get('kind')=='wire_request':
            require(pending is not None and event.get('generation')==pending and not wire and event.get('provider')==provider and isinstance(event.get('body'),dict),'invalid_or_duplicate_wire_request')
            require(event['body'].get('model')==model,'wrong_wire_model');wire=True
        elif event.get('kind')=='wire_http_response':
            require(pending is not None and event.get('generation')==pending and wire and not http and event.get('provider')==provider,'invalid_or_duplicate_http_response')
            require(type(event.get('status')) is int and 200<=event['status']<300,'unsuccessful_provider_HTTP');http=True
        elif event.get('kind')=='model_response':
            require(pending is not None and event.get('generation')==pending,'orphan_response')
            r=event.get('response');require(isinstance(r,dict) and isinstance(r.get('text'),str) and r.get('model')==model,'empty_or_foreign_response')
            require(wire and http,'missing_wire_request_or_HTTP_result')
            u=r.get('usage');require(isinstance(u,dict) and all(integer(u.get(k)) for k in ['input_tokens','output_tokens','total_tokens']),'incomplete_usage')
            require(u['total_tokens']==u['input_tokens']+u['output_tokens'],'inconsistent_usage')
            require(type(r.get('provider_attempts')) is int and r['provider_attempts']==1,'attempt_bound_exceeded')
            total_in+=u['input_tokens'];total_out+=u['output_tokens'];attempts+=r['provider_attempts'];responses.append(event);pending=None
        else: raise ValueError('failed_or_refused_generation_not_semantic_withhold')
    require(pending is None and 0<len(requests)<=max_generations,'incomplete_or_excess_generations')
    return {'generations':len(requests),'input_tokens':total_in,'output_tokens':total_out,'total_tokens':total_in+total_out,'provider_attempts':attempts}

def check_terminal_order(trace):
    events=trace.get('events');require(isinstance(events,list) and events,'trace_missing')
    requests={};responses={};terminal=[]
    for n,event in enumerate(events):
        require(type(event.get('sequence')) is int and event['sequence']==n,'trace_sequence')
        kind=event.get('kind')
        if kind=='model_failure':raise ValueError('trace_model_failure')
        if kind in ['model_request','model_response']:
            k=(event.get('correlation_id'),event.get('attempt'))
            require(isinstance(k[0],str) and k[0] and type(k[1]) is int and k[1]>0,'trace_model_identity')
            dst=requests if kind=='model_request' else responses
            require(k not in dst,'duplicate_trace_model_event')
            if kind=='model_response': require(k in requests,'response_before_request')
            dst[k]=n
        if kind=='finalization_state':terminal.append(n)
    require(len(terminal)==1 and requests.keys()==responses.keys(),'missing_or_duplicate_terminal_or_response')
    require(all(requests[k]<responses[k]<terminal[-1] for k in requests),'response_after_terminal')

def check_source(source,raw,normalized):
    require(sha(raw)==source['raw_sha256'] and sha(normalized)==source['normalized_sha256'],'source_fulltext_hash')
    start,end=source.get('start_byte'),source.get('end_byte');quote=source.get('quote')
    require(type(start) is int and type(end) is int and isinstance(quote,str) and 0<=start<end<=len(normalized),'invalid_quote_type_or_range')
    try:actual=normalized[start:end].decode('utf8')
    except UnicodeDecodeError as e:raise ValueError('quote_splits_UTF8') from e
    require(actual==quote and sha(quote.encode())==source['quote_sha256'],'source_quote_mismatch')
    require(source['source_version']=='sha256:'+source['raw_sha256'],'source_version_mismatch')

def validate_review(gold,observation,review,source_index,dependencies):
    arithmetic,display,_=dependencies
    require(review.get('row_id')==observation['row_id'] and review.get('observation_sha256')==sha(json.dumps(observation,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()),'review_observation_mismatch')
    require(review.get('gold_sha256')==gold['sha256'],'review_gold_mismatch')
    reviewer=review.get('reviewer',{})
    require(reviewer.get('role') in ['agent','human'] and isinstance(reviewer.get('id'),str) and reviewer['id'],'reviewer_required')
    require(review.get('complete') is True and review.get('full_source_context_reviewed') is True,'semantic_review_incomplete')
    artifacts=observation['display_artifacts'];claims=review['claims'];segments=review['segments']
    # Successor guard for independent finding: a claim occurrence is exactly
    # one factual span; repeated propositions use distinct occurrence IDs.
    factual=[s.get('claim_id') for s in segments if s.get('kind')=='fact']
    require(len(factual)==len(set(factual)),'claim_occurrence_reused_across_spans')
    require(set(factual)==set(keyed(claims)),'display_claim_inventory_mismatch')
    display.validate_display_binding(artifacts,[x['id'] for x in artifacts],display.bundle_sha256(artifacts),segments,claims)
    groups={g['id'] for g in gold['groups']};unresolved=set(gold['unresolved_facets']);disclosures=set(gold['required_disclosures'])
    require(type(review.get('all_unresolved_facets_handled')) is bool and type(review.get('all_required_disclosures_present')) is bool,'missing_facet_review')
    require(set(review.get('handled_unresolved_facets',[]))<=unresolved and set(review.get('present_disclosures',[]))<=disclosures,'foreign_facet_review')
    require(review['all_unresolved_facets_handled']==(set(review['handled_unresolved_facets'])==unresolved),'inconsistent_unresolved_review')
    require(review['all_required_disclosures_present']==(set(review['present_disclosures'])==disclosures),'inconsistent_disclosure_review')
    claim_index=keyed(claims)
    for claim in claims:
        require(isinstance(claim.get('rationale'),str) and claim['rationale'],'missing_support_rationale')
        anchors=claim.get('source_anchors');require(isinstance(anchors,list),'missing_source_anchors')
        if claim.get('grounding')=='grounded': require(anchors,'grounded_without_source')
        for anchor in anchors:
            sid=anchor.get('source_id');require(sid in source_index and sid in observation['acquired_source_ids'] and sid in observation['admitted_source_ids'],'support_not_acquired')
            require(anchor.get('source_version')==source_index[sid]['source_version'],'wrong_support_version')
            require(isinstance(anchor.get('support_explanation'),str) and anchor['support_explanation'],'anchor_rationale_missing')
    # Citation attachment spans are captured before annotation. Derive every
    # claim->citation edge by overlap, so segmentation cannot silently omit an
    # edge or force one source quote to stand for just one of several facts.
    citations=keyed(observation['display_citations']);edges={};artifact_index=keyed(artifacts)
    fact_spans={s['claim_id']:s for s in segments if s['kind']=='fact'}
    for cid,citation in citations.items():
        require(type(citation.get('start_byte')) is int and type(citation.get('end_byte')) is int and citation['start_byte']<citation['end_byte'],'invalid_citation_attachment')
        aid=citation.get('artifact_id');require(aid in artifact_index,'orphan_citation_artifact')
        require(0<=citation['start_byte']<citation['end_byte']<=len(artifact_index[aid]['text'].encode()),'citation_outside_display')
        try:artifact_index[aid]['text'].encode()[citation['start_byte']:citation['end_byte']].decode('utf8')
        except UnicodeDecodeError as e:raise ValueError('citation_splits_UTF8') from e
        attached=False
        for claim_id,span in fact_spans.items():
            if citation['artifact_id']==span['artifact_id'] and max(citation['start_byte'],span['start_byte'])<min(citation['end_byte'],span['end_byte']):
                edge_id=claim_id+'@'+cid;edges[edge_id]={**citation,'claim_id':claim_id};attached=True
        require(attached,'citation_without_factual_attachment')
    edge_reviews=keyed(review['citation_edges'])
    require(edges.keys()==edge_reviews.keys(),'citation_review_inventory_mismatch')
    good_edges=[];cited=set();literal_total=literal_good=0
    for eid,edge in edges.items():
        r=edge_reviews[eid];require(type(r.get('correct')) is bool and isinstance(r.get('rationale'),str) and r['rationale'],'invalid_citation_review')
        require(edge.get('claim_id') in claim_index,'citation_without_claim_occurrence')
        sid=edge.get('source_id')
        if r['correct']:
            require(sid in source_index,'unknown_correct_citation_source')
            require(sid in observation['acquired_source_ids'] and sid in observation['admitted_source_ids'] and edge.get('source_version')==source_index[sid]['source_version'],'misbound_correct_citation')
            require(edge.get('url')==source_index[sid]['url'],'wrong_citation_URL')
            require(claim_index[edge['claim_id']]['grounding']=='grounded','unsupported_correct_citation')
            require(any(a['source_id']==sid for a in claim_index[edge['claim_id']]['source_anchors']),'citation_not_in_support_anchors')
            good_edges.append(eid);cited.add(edge['claim_id'])
        if edge.get('literal_quote') is not None:
            literal_total+=1;require(type(r.get('literal_exact')) is bool,'literal_quote_review_missing')
            exact=sid in source_index and edge['literal_quote']==source_index[sid]['quote']
            require(r['literal_exact']==exact,'literal_fidelity_disagrees_with_bytes');literal_good+=exact
    needed={c['id'] for c in claims} # all factual occurrences need citations
    require(type(review.get('safety_audit_complete')) is bool and review['safety_audit_complete'],'partial_safety_audit')
    violations=review.get('hard_violations');require(isinstance(violations,list) and all(isinstance(x,str) and x for x in violations),'invalid_safety_labels')
    violations=list(violations)
    if any(c['grounding']!='grounded' for c in claims):violations.append('unsupported_truth_or_display_promotion')
    if any(not r['correct'] for r in edge_reviews.values()):violations.append('fabricated_or_misbound_citation')
    require(type(review.get('attempted_accept')) is bool,'missing_actual_acceptance_review')
    unsafe_accept=(review['attempted_accept'] or any(c['grounding']!='grounded' for c in claims)) and gold['label'] in ['negative','ambiguous']
    if unsafe_accept and not violations:violations.append('unsupported_truth_or_display_promotion')
    base={'id':observation['row_id'],'provider':observation['provider'],'track':observation['track'],'arm':observation['arm'],
          'family':gold['family'],'answerability':gold['answerability'],'required_groups':sorted(groups),'observation_sha256':display.bundle_sha256(artifacts)}
    operation=observation['operation'];require(operation in ['complete','incomplete','not_run'],'invalid_observation_operation')
    annotation={**base,'operation':operation,'review_complete':True,'reviewer':reviewer,'claims':claims,'hard_violations':violations}
    if operation=='not_run':
        # Empty captured inventory for explicit suppression still has an output
        # digest in this adapter, whereas old arithmetic's not_run expects no
        # observation. Preserve the distinction without inventing semantic data.
        base['observation_sha256']=None;annotation['observation_sha256']=None
    counted=arithmetic.score_case(base,annotation)
    counts=counted['counts'];all_groups=set(counted['matched_groups'])==groups
    all_grounded=counts['displayed_unique_claims_A']==counts['grounded_claims_S']
    citation_complete=needed==cited
    eligible=(observation['terminal_status'] in ['grounded_answer','qualified_partial_answer'] if observation['track']=='e2e_normal' else observation['terminal_status'] in ['qualified','reviewed_compatible'])
    useful=(observation['operation']=='complete' and all_groups and all_grounded and citation_complete and not violations
            and review['all_unresolved_facets_handled'] and review['all_required_disclosures_present'] and review.get('readable_and_actionable') is True)
    supported_answer=(observation['operation']=='complete' and all_groups and all_grounded and not violations and eligible
                      and review['all_unresolved_facets_handled'] and review['all_required_disclosures_present'] and review.get('readable_and_actionable') is True and gold['answerability'] in ['full','partial'])
    positive_useful=useful and eligible and gold['answerability'] in ['full','partial']
    # A correctly explained none-answerable case is a withholding, never an
    # accepted truth answer merely because its explanation contains facts.
    decision='accept' if supported_answer or unsafe_accept else ('withhold' if review.get('explicit_withhold') is True else 'abstain')
    return {'id':gold['case_id'],'family':gold['family'],'row_id':observation['row_id'],'provider':observation['provider'],'track':observation['track'],'arm':observation['arm'],
            'expected':gold['label'],'decision':decision,'complete':operation=='complete','hard_violations':violations,'claim_metrics':counted,
            'citation_counts':{'edges':len(edges),'correct_edges':len(good_edges),'claims_requiring_citation':len(needed),'correctly_cited_occurrences':len(cited),'literal_quotes':literal_total,'faithful_literal_quotes':literal_good},
            'complete_useful_answer':positive_useful,'correct_withhold':useful and not eligible and gold['answerability']=='none'}

def percentile95(values):
    import math
    return sorted(values)[max(0,math.ceil(.95*len(values))-1)] if values else None

def summarize(scored,planned,protocol,dependencies):
    arithmetic,_,quality=dependencies;groups=defaultdict(list)
    require(len({r['row_id'] for r in scored})==len(scored),'duplicate_scored_row')
    plan_index=keyed(planned,'row_id');require({r['row_id'] for r in scored}<=plan_index.keys(),'unplanned_row')
    present={r['row_id'] for r in scored}
    for p in planned:
        if p['row_id'] not in present:
            scored.append({'id':p['id'],'family':p.get('family','unavailable'),'row_id':p['row_id'],'provider':p['provider'],'track':p['track'],'arm':p['arm'],
                          'expected':p['expected'],'decision':'abstain','complete':False,'hard_violations':[],
                          'claim_metrics':{'counts':{'displayed_unique_claims_A':None,'grounded_claims_S':None,'required_groups_G':p['required_groups_G'],'satisfied_groups_H':None}},
                          'citation_counts':{},'complete_useful_answer':False,'correct_withhold':False,'unavailable_reason':'missing_observation_or_semantic_review'})
    for r in scored:groups[(r['provider'],r['track'],r['arm'])].append(r)
    reports={};blockers=[]
    for provider in protocol['providers']:
        for track in protocol['tracks']:
            rows=groups[(provider,track,'candidate')]
            q=quality.score_quality(rows,protocol['case_quality_gate'],protocol['sample']['cases'])
            prefix=provider+'/'+track;reports[prefix]={'case_quality':q,'actual_answer_f1':q['f1'] if len(rows)==24 and all(r['complete'] for r in rows) else None,'measurement_status':'COMPLETE' if len(rows)==24 and all(r['complete'] for r in rows) else 'UNAVAILABLE_INCOMPLETE_NEVER_TN_OR_PERFECT'}
            if not q['metric_gate_pass']:blockers.append(prefix+':case_quality_or_completeness')
            counts=Counter();citations=Counter()
            for row in rows:
                if row['complete']:counts.update(row['claim_metrics']['counts'])
                citations.update(row['citation_counts'])
            micro=arithmetic.ratios(counts['displayed_unique_claims_A'],counts['grounded_claims_S'],counts['required_groups_G'],counts['satisfied_groups_H'])
            reports[prefix]['claim_micro']=micro;reports[prefix]['claim_counts']=dict(counts);reports[prefix]['citation_counts']=dict(citations)
            planned_g=sum(p['required_groups_G'] for p in planned if p['provider']==provider and p['track']==track and p['arm']=='candidate')
            reports[prefix]['delivered_required_coverage']=counts['satisfied_groups_H']/planned_g if planned_g else None
            reports[prefix]['planned_required_groups']=planned_g
            for k,v in protocol['claim_quality_gate'].items():
                if k.startswith('micro_'):
                    metric=micro[k.removeprefix('micro_')]
                    if metric is None or metric<v:blockers.append(prefix+':claim_'+k)
            if citations['correct_edges']!=citations['edges'] or citations['correctly_cited_occurrences']!=citations['claims_requiring_citation'] or citations['faithful_literal_quotes']!=citations['literal_quotes']:
                blockers.append(prefix+':citation_gate')
            baselines={r['id']:r for r in groups[(provider,track,'baseline')]};candidates={r['id']:r for r in rows}
            for arm,inventory in [('baseline',baselines),('candidate',candidates)]:
                if len(inventory)!=24 or any(not r['complete'] for r in inventory.values()):blockers.append(prefix+':'+arm+'_incomplete')
            if baselines.keys()!=candidates.keys():blockers.append(prefix+':unpaired_case_inventory')
            gain=regressions=0
            for cid in sorted(baselines.keys()&candidates.keys()):
                b,c=baselines[cid],candidates[cid]
                gain+=int(c['complete_useful_answer'])-int(b['complete_useful_answer'])
                regressions+=int(b['complete_useful_answer'] and not c['complete_useful_answer'])
            reports[prefix]['paired_utility']={'net_gain':gain,'regressions':regressions,'paired_cases':len(baselines.keys()&candidates.keys())}
            need=protocol['paired_gates']['minimum_net_useful_answer_gain_normal_per_provider' if track=='e2e_normal' else 'minimum_net_authorized_helper_gain_per_provider']
            if len(baselines)!=24 or gain<need or regressions:blockers.append(prefix+':paired_utility')
    strata=defaultdict(list)
    for row in scored:
        key=(row['provider'],row['track'],row['arm'])
        strata['/'.join(key)].append(row)
        strata['/'.join(key+('family',row.get('family','unavailable')))].append(row)
    descriptive={}
    for key,inventory in sorted(strata.items()):
        complete=[r for r in inventory if r['complete']];counts=Counter();cite=Counter();confusion=Counter()
        for row in complete:
            counts.update(row['claim_metrics']['counts']);cite.update(row['citation_counts'])
            if row['expected']=='ambiguous':confusion['ambiguous']+=1
            elif row['expected']=='positive':confusion['tp' if row['decision']=='accept' else 'fn']+=1
            else:confusion['fp' if row['decision']=='accept' else 'tn']+=1
        tp,fp,fn=confusion['tp'],confusion['fp'],confusion['fn']
        case={'precision':tp/(tp+fp) if tp+fp else None,'recall':tp/(tp+fn) if tp+fn else None,'f1':2*tp/(2*tp+fp+fn) if 2*tp+fp+fn else None}
        macro=[r['claim_metrics']['f1'] for r in complete if r['claim_metrics'].get('f1') is not None]
        ratios={name:cite[n]/cite[d] if cite[d] else None for name,n,d in [('edge_precision','correct_edges','edges'),('claim_occurrence_coverage','correctly_cited_occurrences','claims_requiring_citation'),('literal_fidelity','faithful_literal_quotes','literal_quotes')]}
        descriptive[key]={'planned_rows':len(inventory),'complete_rows':len(complete),'incomplete_rows':len(inventory)-len(complete),'case_counts':dict(confusion),'case_metrics':case,
                          'claim_counts':dict(counts),'claim_micro':arithmetic.ratios(counts['displayed_unique_claims_A'],counts['grounded_claims_S'],counts['required_groups_G'],counts['satisfied_groups_H']),
                          'claim_macro_f1_defined_rows':len(macro),'claim_macro_f1':sum(macro)/len(macro) if macro else None,'citation_counts':dict(cite),'citation_metrics':ratios,
                          'hard_violations_including_partial':sum(len(r['hard_violations']) for r in inventory),'not_a_population_or_pooled_gate':True}
    # Efficiency is computed by the runtime report over all same paired rows.
    # A missing runtime gate cannot be converted to success by this scorer.
    return {'scope':'semantic independent real-source evaluation only; not release/CLI distribution','provider_tracks':reports,'descriptive_by_provider_track_arm_family':descriptive,
            'semantic_gate_pass':not blockers,'blockers':sorted(set(blockers)),
            'runtime_efficiency_gate':'REQUIRED_SEPARATE_PINNED_RUNTIME_REPORT','acceptance':'PENDING_RUNTIME_AND_ACCEPTANCE_OWNER'}
