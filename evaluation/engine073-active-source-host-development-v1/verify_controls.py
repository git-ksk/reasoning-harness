#!/usr/bin/env python3
"""Inspect engineering capture bytes. This is not an answer-quality scorer."""
import hashlib
import json
from pathlib import Path
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parent
REPO = Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def read(name): return json.loads((ROOT/name).read_text())

before, after, source = read('identity-before.json'), read('identity-after.json'), read('input.json')['source']
assert before['input_hashes'] == after['input_hashes']
for p, digest in before['input_hashes'].items(): assert sha(p) == digest
for arm, libs in before['libraries'].items():
    for name in ('core','serde'):
        assert sha(libs[name]) == libs[name+'_sha256'] == after['library_hashes'][arm][name+'_sha256']

arms = {}
for arm, version in [('baseline','0.6.1'),('candidate','0.7.2')]:
    gate, renderer = read(arm+'-node-control.stdout'), read(arm+'-renderer-control.stdout')
    assert gate['engine_version'] == renderer['engine_version'] == version
    assert [event['sequence'] for event in gate['events']] == [0,1,2,3]
    assert gate['events'][0]['quote'] == source['quote']
    assert gate['events'][1]['candidate']['signals'] == [{'kind':'excerpt','text':source['quote']}]
    assert gate['events'][1]['output'] == {'identity_scope':'exact_target','relation_scope':'unresolved','scope_risk':'none'}
    assert gate['events'][2]['output']['disposition'] == 'ambiguous'
    assert gate['direct_host_api_calls'] == {'derive_effective_evidence_local_qualification_v17':1,'materialize_evidence_relevance_v30':1}
    assert gate['finalization']['status'] == 'unresolved' and not gate['finalization']['citations']
    assert 'text' not in gate['finalization'] and gate['display_artifacts'] == []
    assert gate['events'][3]['output'] == gate['finalization']
    assert renderer['manual_source_binding_control'] and renderer['direct_relevance_api_calls'] == 0
    expected_text = (f"According to {source['source_id']}: {source['quote']} "
                     f"[source:{source['source_id']}; evidence:{source['evidence_id']}; binding:isolated-source-binding]")
    assert renderer['finalization']['text'] == expected_text
    assert renderer['finalization']['status'] == 'qualified'
    assert renderer['finalization']['claim_ids'] == ['isolated-source-claim']
    assert renderer['finalization']['citations'] == [{
        'binding_id':'isolated-source-binding','claim_id':'isolated-source-claim','evidence_id':source['evidence_id'],
        'locator':{'section':source['locator']},'retrieved_at_unix_seconds':source['retrieved_at_unix_seconds'],
        'source_id':source['source_id'],'source_url':source['url'],'source_version':source['source_version']}]
    for output in [gate,renderer]:
        assert output['truth_claims'] == output['verification_receipts'] == output['model_provider_calls'] == 0
        assert output['actual_answer_f1'] is None and output['acceptance'] == 'NOT_EVALUATED' and not output['normal_cli_changed']
    for kind in ['node-control','renderer-control']:
        assert read(arm+'-'+kind+'.invocation.json')['exit_code'] == 0
        assert (ROOT/(arm+'-'+kind+'.stderr')).read_bytes() == b''
    bad = read(arm+'-quote-mismatch.invocation.json')
    assert bad['exit_code'] != 0 and (ROOT/(arm+'-quote-mismatch.stdout')).read_bytes() == b''
    assert 'quote differs from retained full source' in (ROOT/(arm+'-quote-mismatch.stderr')).read_text()
    arms[arm] = {'engine_package_version':version,'core_commit':before['libraries'][arm]['commit'],
                 'source_sha256':before['libraries'][arm]['source_sha256'],
                 'relevance_pipeline':{'v17_relation':'unresolved','v30_disposition':'ambiguous','final_status':'unresolved',
                                       'displayed_citations':0,'direct_host_calls_per_API':1},
                 'isolated_exact_quote_renderer':{'manual_source_binding':True,'relevance_calls':0,'final_status':'qualified',
                                                 'structured_citations':1,'text_has_exact_source_evidence_binding':True},
                 'mismatched_quote_rejected_before_Engine_calls':True}
assert read('baseline-node-control.stdout')['finalization'] == read('candidate-node-control.stdout')['finalization']
assert read('baseline-renderer-control.stdout')['finalization'] == read('candidate-renderer-control.stdout')['finalization']
anchor = REPO/'evaluation/engine073-launch-independent-v27/final-artifact-manifest.json'
assert sha(anchor) == 'abaae2385e6c17783cc7093c10e15424b6e4ce01080e8fcdea5eab3a9e5bb439'
originals = json.loads(anchor.read_text())['files']; assert len(originals) == 927
for name,digest in originals.items(): assert sha(REPO/name)==digest,name
old = REPO/'evaluation/engine073-answer-metrics-development-v1'
old_index = json.loads((old/'artifact-sha256.json').read_text())
for name,digest in old_index.items(): assert sha(old/name)==digest,name
result = {'identity':'engine073-active-source-host-development-v1','verified_utc':datetime.now(timezone.utc).isoformat(),
          'purpose':'unscored offline engineering controls; same existing captured Node question, not fresh cases',
          'arms':arms,'same_finalizations_across_arms':True,'host_processes':6,'Engine_executing_processes':4,'relevance_pipeline_processes':2,
          'renderer_isolation_processes':2,'quote_guard_negative_processes_before_API':2,
          'direct_host_v17_calls':2,'direct_host_v30_calls':2,'nested_internal_API_calls_not_counted':True,
          'model_provider_network_Oracle_calls':0,'fresh_synthetic_or_real_authorship':0,'real24_completed':0,
          'actual_answer_f1':None,'acceptance':'NOT_EVALUATED','normal_cli_changed':False,'production_source_changed':False,
          'all928_original_references_exact':True,'old_claim_display_component_unchanged':True,
          'private_source_fulltext_published':False,'current_candidate_discriminating_quality_or_performance_evidence':False,
          'limitations':['host-controlled qualification/proposal, not a model response',
                         'retained definition question does not measure launch-repair discrimination',
                         'paired build profiles are identified, not a latency or cost comparison',
                         'active relevance pipeline withheld; isolated renderer is not a relevance success',
                         'no automatic scope/time semantic judgment or publisher authenticity attestation',
                         'ordinary CLI integration, semantic/citation adjudication, complete M0/budgets and independent review remain pending']}
with (ROOT/'validation.json').open('x') as f: json.dump(result,f,ensure_ascii=False,indent=2); f.write('\n')
print(json.dumps({'structural_validation':'PASS','pipeline':'AMBIGUOUS_WITHHELD_BOTH_ARMS',
                  'renderer':'ISOLATED_EXACT_SOURCE_LOCAL_CITATION_BOTH_ARMS','actual_answer_f1':None,
                  'all928_original_references_exact':True,'acceptance':'NOT_EVALUATED'}))
