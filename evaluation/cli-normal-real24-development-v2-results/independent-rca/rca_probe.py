"""Saved-byte operational RCA only. No model, observation, annotation or scorer run."""
import pathlib,json,hashlib,zipfile,sys,importlib.util,collections,dataclasses
out=pathlib.Path(__file__).resolve().parent;raw=pathlib.Path('target/ordinary-v2-live-41061954114-raw').resolve();root=raw/'artifact';runtime=out.with_name('cli-normal-v2-runtime-40928520250-review-work')/'actual-image-files/repo';paired=runtime/'evaluation/cli-repair-paired-v1';sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();read=lambda p:json.loads(p.read_bytes());before={str(p.relative_to(raw)):sha(p) for p in sorted(raw.rglob('*')) if p.is_file()}
assert sha(raw/'artifact.zip')=='b082244ee61bdef92461c08157ca09b6feda0619f3b6b627d882024641a6d30f'
with zipfile.ZipFile(raw/'artifact.zip') as z:
 names=[e.filename for e in z.infolist() if not e.is_dir()];assert len(names)==len(set(names))
 for e in z.infolist():
  p=pathlib.PurePosixPath(e.filename);assert not p.is_absolute() and '..' not in p.parts and '\\' not in e.filename
  if not e.is_dir():assert z.read(e)==(root/e.filename).read_bytes()
 assert set(names)=={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file()}
receipt=root/'development-receipt.json';assert sha(receipt)=='03b86be2bd888280e8b626ade6b638f801c4525faec164752bab71b0ac361741'
identity=read(root/'execution-identity.json');assert identity['run_id']==41061954114 and identity['execution_tag']=='cli-normal-real24-development-v2' and identity['run_attempt']==1
sys.path[:0]=[str(paired),str(runtime/'evaluation/engine07-realworld-operations-v1')]
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
source_files={str(p.relative_to(runtime)):sha(p) for p in runtime.rglob('*') if p.is_file()};assert source_files==read(pathlib.Path('evaluation/cli-normal-real24-development-v2/runtime-proof.json'))['runtime_files']
scorer=load('frozen_rca_validation_only',paired/'scorer.py');audit=load('frozen_rca_trace_audit',paired/'trace_audit.py');bridge=load('frozen_rca_operational_bridge',paired/'normal_v2/operational_bridge.py');prov=load('frozen_rca_provenance',paired/'normal_v2/provenance.py')
verified=prov.verify_extracted(root,'git-ksk/reasoning-harness',41061954114,12980216364,identity['source_sha'],sha(receipt),sha(raw/'artifact.zip'))
state=read(root/'lane/state.json');summary=read(root/'lane/summary.json');assert summary['rows']==state['rows'];counts=collections.Counter('complete' if r['complete'] else ('failed' if r['admission']['allowed'] else 'suppressed') for r in state['rows']);assert counts=={'complete':2,'failed':1,'suppressed':45};assert state['charged_reservations']==84288 and state['unknown_usage_rows']==0 and state['known_provider_attempts']==20 and state['known_total_tokens']==20293
jobs=[];failed=None
for row in state['rows']:
 if not row['admission']['allowed']:continue
 d=root/'lane'/row['id'];p=read(d/'stdout.json');t=read(d/'trace.json');events=[json.loads(line) for line in (d/'model-journal.jsonl').read_text().splitlines()]
 usage=scorer.check_model_journal(events,'mistral','ministral-8b-2512',24);audit.bind(t,events,'mistral','ministral-8b-2512');scorer.check_terminal_order(t,p['result'])
 normal=bridge.outcome_from_normal(p);base=bridge._legacy.outcome_from_normal(p);assert normal.tokens==usage['total_tokens']==p['result']['usage']['run']['total_tokens'] and normal.provider_attempts==usage['provider_attempts']==p['result']['usage']['run']['provider_attempts'];assert row['returncode']==0
 entry={'job':row['id'],'returncode':row['returncode'],'journal':usage,'HTTP_statuses':[e['status'] for e in events if e['kind']=='wire_http_response'],'frozen_bridge_outcome':dataclasses.asdict(normal),'underlying_envelope_accounting_outcome':dataclasses.asdict(base),'trace_completion':t['completion_status'],'trace_audit_and_terminal_chain_pass':True,'pre_request_refusals':sum(e['kind']=='pre_request_refusal' for e in events),'maximum_request_bytes':max(e['request_bytes'] for e in events if e['kind']=='model_request')}
 jobs.append(entry)
 if row['id']=='rw-02-candidate':
  failed=(p,t,events,entry)
assert sum(j['journal']['total_tokens'] for j in jobs)==20293 and sum(j['journal']['provider_attempts'] for j in jobs)==20
p,t,events,entry=failed;responses={e['generation']:e['response'] for e in events if e['kind']=='model_response'};r4=responses[4];r5=responses[5];assert r4['finish_reason']=='length' and r5['finish_reason']=='stop'
try:json.loads(r4['text']);raise AssertionError('first regen unexpectedly valid JSON')
except json.JSONDecodeError as err:first_error={'exception':type(err).__name__,'message':err.msg,'line':err.lineno,'column':err.colno}
fallback_request=next(e['request'] for e in events if e['kind']=='model_request' and e['generation']==5)
schema,unused=json.JSONDecoder().raw_decode(fallback_request['task'][len('JSON Schema:\n'):]);assert 'statement' in schema['$defs']['ModelCandidateClaim']['required'] and 'proposition' in schema['$defs']['ModelCandidateClaim']['properties']
fallback=json.loads(r5['text']);missing=[i for i,c in enumerate(fallback['claims']) if 'statement' not in c];assert missing==[3] and 'proposition' in fallback['claims'][3]
failure=p['result']['investigation']['generation_failure'];assert failure['failure_class']=='protocol' and failure['provider_attempts']==2 and failure['usage']['total_tokens']==4173
assert p['result']['investigation']['telemetry']['stop_reason']==p['result']['acquisition_coverage']['stop_reason']=='operational_terminal';assert 'operational_terminal' not in bridge.SAFE_STOPS
assert entry['frozen_bridge_outcome']['failure_class']=='unknown_failure' and entry['underlying_envelope_accounting_outcome']['failure_class'] is None
assert p['result']['finalization']['status']=='requires_verification' and p['result']['exposed_text']['renderer_text_exposed'] is False
regen_traces=[e for e in t['events'] if e.get('phase')=='post_investigation_regeneration' and e['kind']=='model_response'];assert len(regen_traces)==2
report={'reviewer':'/root/real24_operational_review','result':'SAVED_RAW_OPERATIONAL_CAUSE_REPRODUCED_NO_RECLASSIFICATION','run_id':41061954114,'artifact_id':12980216364,'ZIP_sha256':sha(raw/'artifact.zip'),'raw_member_count':len(names),'execution_source_sha':identity['source_sha'],'receipt_sha256':sha(receipt),'state_counts':dict(counts),'charged_reservations':state['charged_reservations'],'known_provider_attempts':20,'known_total_tokens':20293,'unknown_usage_rows':0,'latch':summary['latch'],'actual_journal_jobs':jobs,'failure_detail':{'phase':'post_investigation_regeneration','first_generation':4,'first_finish':'length','first_text_bytes':len(r4['text'].encode()),'first_strict_JSON_error':first_error,'fallback_generation':5,'fallback_finish':'stop','fallback_text_bytes':len(r5['text'].encode()),'fallback_missing_statement_claim_indices':missing,'fallback_present_optional_field':'proposition','statement_required_by_actual_fallback_request_schema':True,'CLI_recorded_failure':failure,'generation_6_final_render_HTTP200_known_usage':responses[6]['usage'],'diagnostic_terminal_status':p['result']['finalization']['status'],'renderer_text_exposed':False,'CLI_trace_completion':'completed','why_unknown_failure':'Recognized generation_failure protocol plus operational_terminal stop_reason outside SAFE_STOPS yields UNKNOWN precedence. This is unknown failure classification, not unknown usage.'},'frozen_source_files_sha256':{str(x.relative_to(runtime)):sha(x) for x in [paired/'normal_v2/operational_bridge.py',paired/'operational_bridge.py',paired/'collect.py',paired/'scorer.py',paired/'capture.py',paired/'terminal_chain.py',runtime/'evaluation/engine07-quality-gates-v1/metrics.py',runtime/'evaluation/engine073-answer-metrics-development-v1/claim_metrics.py']},'no_new_observations_annotations_score_or_F1_created':True,'check_calls':'Only source-bound journal/trace/terminal/artifact validators and pure old bridge accounting; no summarize/validate_review/score_case/score_quality or collect/bind_normal called.','raw_files_before':before}
(out/'rca-result.json').write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
assert {str(p.relative_to(raw)):sha(p) for p in raw.rglob('*') if p.is_file()}==before
print('Saved raw RCA PASS; no old outcome changed; raw files unchanged',len(before),dict(counts))
