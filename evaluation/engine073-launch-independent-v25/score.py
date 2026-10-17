#!/usr/bin/env python3
"""v25 ONE real scorer process. Strict observation parsing + pinned policy helper.

Pure fixture functions may be unit-tested before authorship. Never run main on
historical cohorts. Joint exactness remains diagnostic; new policy decides gate.
"""
import importlib.util,sys
from evaluator_contract import *
BASE_KEYS={'known_regressions_total','known_regressions_passed','original_audit_total','original_audit_passed','historical_byte_equivalent','known_unfixed_critical_defects'}
def config_and_helper():
 if sha(ROOT/'source_gate_policy.json')!=POLICY_SHA or sha(ROOT/'source_gate_helper.py')!=HELPER_SHA:raise ValueError('pinned_policy_or_helper_changed')
 config=load(ROOT/'source_gate_policy.json')
 spec=importlib.util.spec_from_file_location('v25_frozen_helper',ROOT/'source_gate_helper.py');helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
 return config,helper

def provenance(bundle):
 """Pure checks over separately loaded runtime hashes/markers; no trusted future flags."""
 try:
  m0,m,before,after,author,start=bundle['m0'],bundle['manifest'],bundle['before'],bundle['after'],bundle['author_start'],bundle['score_start']
  frozen=m['frozen_files'];guard=before['pre_invocation_identity'];after_guard=after['pre_invocation_identity']
  expected=[m['checked_artifacts']['probe_executable']['path'],str(ROOT/'inputs.json')]
  checks={
   'candidate_identity':m0['candidate']['source_sha']==m['candidate']['source_sha']==bundle['current_head']==SOURCE and m['candidate']['package_version']==VERSION,
   'same_checked_artifacts':m['checked_artifacts']==m0['checked_artifacts'] and bundle['artifacts_match'] is True,
   'M0_evaluator_hashes':bundle['M0_files_match'] is True and frozen.get('evaluator-freeze.json')==bundle['M0_sha256'] and all(frozen.get(n)==h for n,h in m0['frozen_files'].items()),
   'M1_all_frozen_hashes':bundle['M1_files_match'] is True,
   'gold_and_input_pinned':all(n in frozen for n in ['cases.json','inputs.json','author_cases.py','authoring-start.json','scope-review.json','static-preflight.json','protocol.json']),
   'authorship_after_M0':author['evaluator_freeze_sha256']==bundle['M0_sha256']==m['authoring']['evaluator_freeze_sha256']==bundle['cases_header']['evaluator_freeze_sha256'] and bool(author['authorization_reference']) and dt(m0['frozen_utc'])<=dt(author['utc'])<=dt(bundle['cases_header']['authored_utc'])<=dt(m['frozen_utc']),
   'author_start_bound':m['authoring']['authoring_start_sha256']==frozen.get('authoring-start.json')==bundle['author_start_sha256']==bundle['cases_header']['authoring_start_sha256'],
   'first_pre_invocation_manifest':guard['manifest_sha256']==after_guard['manifest_sha256']==bundle['manifest_sha256'] and guard['frozen_files']==after_guard['frozen_files']==frozen and guard['head']==after_guard['head']==SOURCE,
   'first_pre_invocation_artifacts':all(guard['artifacts'][n]['path']==a['path'] and guard['artifacts'][n]['sha256']==a['sha256'] and after_guard['artifacts'][n]==guard['artifacts'][n] for n,a in m['checked_artifacts'].items()),
   'first_command_and_markers':before['state']=='started' and after['state']=='finished' and before['command']==after['command']==expected,
   'first_raw_hashes':after['stdout_sha256']==bundle['raw_sha256'] and after['stderr_sha256']==bundle['stderr_sha256'] and bundle['raw_readonly'] is True,
   'first_order':dt(m['frozen_utc'])<=dt(before['utc'])<=dt(after['utc'])<=dt(start['utc']),
   'first_score_marker':start['state']=='started' and start['identity']=='engine073-launch-independent-v25' and start['scorer_sha256']==bundle['scorer_sha256'] and start['manifest_sha256']==bundle['manifest_sha256'],
  }
 except (KeyError,TypeError,ValueError):
  checks={'malformed_runtime_provenance':False}
 all_good=bool(checks) and all(v is True for v in checks.values())
 return {'checks':checks,'labels_frozen_before_observation':all_good,'candidate_frozen_before_observation':all_good,'first_observation_immutable':all_good,'boundary':'Cooperative operator freeze/hash/marker ordering, not proof against hostile filesystem writer or semantic proof of independent authorship.'}

def runtime_bundle(manifest,score_start):
 m0=load(ROOT/'evaluator-freeze.json');before=load(ROOT/'first-invocation.started.json');after=load(ROOT/'first-invocation.json');casefile=load(ROOT/'cases.json')
 return {'m0':m0,'manifest':manifest,'before':before,'after':after,'author_start':load(ROOT/'authoring-start.json'),'score_start':score_start,'cases_header':casefile,'current_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip(),'artifacts_match':checked_artifacts(manifest['checked_artifacts']),'M0_files_match':all(sha(ROOT/n)==h for n,h in m0['frozen_files'].items()),'M1_files_match':all(sha(ROOT/n)==h for n,h in manifest['frozen_files'].items()),'M0_sha256':sha(ROOT/'evaluator-freeze.json'),'manifest_sha256':sha(ROOT/'manifest.json'),'author_start_sha256':sha(ROOT/'authoring-start.json'),'raw_sha256':sha(ROOT/'first.stdout.jsonl'),'stderr_sha256':sha(ROOT/'first.stderr.txt'),'scorer_sha256':sha(ROOT/'score.py'),'raw_readonly':all((ROOT/n).stat().st_mode&0o222==0 for n in ['first.stdout.jsonl','first.stderr.txt','first-invocation.started.json','first-invocation.json'])}

def validate_gold(cases):
 if not isinstance(cases,list) or len(cases)!=60:raise ValueError('exact60_planned_required')
 ids=set()
 for c in cases:
  if not isinstance(c,dict) or not isinstance(c.get('id'),str) or not c['id'] or c['id'] in ids:raise ValueError('invalid_or_duplicate_gold_id')
  ids.add(c['id'])
  if c.get('polarity') not in ['positive','negative']:raise ValueError('missing_independent_gold')
  truth=c['polarity']=='positive'
  expected=c.get('expected')
  if not isinstance(expected,dict) or set(expected)!={'v17_requested_relation','v30_relevant'} or not all(type(v) is bool for v in expected.values()):raise ValueError('gold_axes_require_strict_booleans')
  if expected!={'v17_requested_relation':truth,'v30_relevant':truth}:raise ValueError('gold_axis_mismatch')
 if sum(c['polarity']=='positive' for c in cases)!=30:raise ValueError('balanced30_30_required')

def validate_row(row,case):
 if not isinstance(row,dict) or set(row)!={'id','engine_version','effective_v17','v29','v30'}:raise ValueError('invalid_probe_row_schema')
 if row['engine_version']!=VERSION:raise ValueError('unexpected_engine_version')
 q=row['effective_v17']
 if not isinstance(q,dict) or set(q)!={'identity_scope','relation_scope','scope_risk'} or q['identity_scope'] not in IDENTITY or q['relation_scope'] not in RELATION or q['scope_risk'] not in RISK:raise ValueError('invalid_effective_qualification')
 inp=case['input'];bindings={'policy_id':inp['policy']['policy_id'],'target_id':inp['policy']['target_id'],'evidence_id':inp['candidate']['evidence_id'],'source_id':inp['candidate']['source_id']}
 for version in ['v29','v30']:
  value=row[version]
  if not isinstance(value,dict) or value.get('disposition') not in DISPOSITION:raise ValueError('invalid_'+version+'_disposition')
  if any(value.get(k)!=v for k,v in bindings.items()):raise ValueError('misbound_'+version+'_identity')
  if not all(isinstance(value.get(k),str) and value[k] for k in ['contract_id','materialization_policy_id','path']):raise ValueError('invalid_'+version+'_contract')
  if not isinstance(value.get('reasons'),list) or not all(isinstance(x,str) for x in value['reasons']):raise ValueError('invalid_'+version+'_reasons')

def exact_risk_diagnostic(case,row):
 """Literal input/output tuples only; no inferred internal component state."""
 return {'raw_provided_tuple':case['input'].get('raw'),'effective_v17_tuple':row['effective_v17'] if row else None,'v29_disposition':row['v29']['disposition'] if row else None,'v30_disposition':row['v30']['disposition'] if row else None,'interpretation':'Exact observed tuples; enum values alone do not establish internal risk-component retention/loss.'}

def assess(cases,raw_text,invocation,stderr_bytes,base,proof,config,helper):
 """ONE helper call. Diagnostic joint matches never override the new policy."""
 validate_gold(cases)
 if set(base)!=BASE_KEYS:raise ValueError('parent_regressions_must_exclude_future_provenance')
 byid={c['id']:c for c in cases};rows={};errors=[]
 for line_number,line in enumerate(raw_text.splitlines(),1):
  try:
   row=strict_json(line);cid=row.get('id') if isinstance(row,dict) else None
   if not isinstance(cid,str) or cid not in byid:raise ValueError('foreign_or_missing_id')
   if cid in rows:raise ValueError('duplicate_observation_id')
   validate_row(row,byid[cid]);rows[cid]=row
  except (ValueError,TypeError,KeyError) as exc:errors.append({'line':line_number,'error':str(exc)})
 missing=sorted(set(byid)-set(rows))
 if missing:errors.append({'missing_ids':missing})
 invocation_ok=invocation.get('state')=='finished' and invocation.get('returncode')==0 and invocation.get('execution_error') is None
 if not invocation_ok:errors.append({'invocation_state':invocation.get('state'),'returncode':invocation.get('returncode'),'execution_error':invocation.get('execution_error')})
 if stderr_bytes:errors.append({'stderr_bytes':stderr_bytes})
 if not proof.get('checks') or not all(x is True for x in proof['checks'].values()):errors.append({'runtime_provenance_failed':proof.get('checks')})
 scored=[];helper_rows=[]
 for c in cases:
  row=rows.get(c['id']);complete=row is not None and invocation_ok and stderr_bytes==0
  observed={'v17_requested_relation':row['effective_v17']['relation_scope']=='requested_relation','v30_relevant':row['v30']['disposition']=='relevant'} if row else None
  helper_rows.append({'id':c['id'],'gold':c['polarity'],'complete':complete,**(observed or {'v17_requested_relation':False,'v30_relevant':False})})
  saved={'id':c['id'],'pair_id':c['pair_id'],'polarity':c['polarity'],'axis':c['axis'],'expected':c['expected'],'complete':complete,'status':'incomplete' if not complete else 'pass' if observed==c['expected'] else 'fail','observed':observed,'raw_observation':row,'exact_risk_diagnostic':exact_risk_diagnostic(c,row)}
  if row:saved.update(effective_relation_scope=row['effective_v17']['relation_scope'],v30_disposition=row['v30']['disposition'],v30_reasons=row['v30']['reasons'],checks={k:observed[k]==v for k,v in c['expected'].items()})
  scored.append(saved)
 controls=dict(base,operational_failures=len(errors),**{k:proof.get(k) is True for k in ['labels_frozen_before_observation','candidate_frozen_before_observation','first_observation_immutable']})
 gate=helper.score(helper_rows,controls,config=config)
 accepted=gate['gate_pass'] and not errors
 summary={'identity':'engine073-launch-independent-v25','candidate_source_sha':SOURCE,'engine_version':VERSION,'outcome':'PASS' if accepted else 'FAIL','decision':'GO' if accepted else 'NO_GO','gate':gate,'axes':gate['axes'],'planned_cases':60,'observed_cases':len(rows),'passed':sum(x['status']=='pass' for x in scored),'failed':sum(x['status']=='fail' for x in scored),'incomplete':sum(not x['complete'] for x in scored),'joint_exactness_is_diagnostic':True,'strict_joint_outcome':'PASS' if not errors and all(x['status']=='pass' for x in scored) else 'FAIL','positive_passed':sum(x['status']=='pass' for x in scored if x['polarity']=='positive'),'negative_passed':sum(x['status']=='pass' for x in scored if x['polarity']=='negative'),'operational_errors':errors,'controls':controls,'parent_regressions':'parent-reported pinned source qualification only; future provenance separately verified','runtime_provenance':proof,'policy_sha256':POLICY_SHA,'helper_sha256':HELPER_SHA,'provider_calls':0,'retries':0,'rescores':0,'scope':'Future #520 synthetic source gate only; no historical rescore/real24/CLI/release approval'}
 return {'summary':summary,'cases':scored,'helper_rows':helper_rows}

def main():
 if any((ROOT/n).exists() for n in ['first-results.json','first-summary.json','first-score.started.json']):raise RuntimeError('first_only_scorer_restart_refused')
 start={'identity':'engine073-launch-independent-v25','state':'started','utc':utc(),'scorer_sha256':sha(ROOT/'score.py'),'manifest_sha256':sha(ROOT/'manifest.json')}
 write_new(ROOT/'first-score.started.json',start)
 try:
  evaluator_integrity();manifest=load(ROOT/'manifest.json');config,helper=config_and_helper();proof=provenance(runtime_bundle(manifest,start));parent=load(ROOT/'parent-controls-snapshot.json')
  if parent['source_sha']!=SOURCE or parent['opaque_reference_sha256']!=sha(ROOT/'parent-controls.opaque.json'):raise ValueError('parent_source_controls_binding')
  cases=load(ROOT/'cases.json')['cases'];invocation=load(ROOT/'first-invocation.json')
  scored=assess(cases,(ROOT/'first.stdout.jsonl').read_bytes().decode('utf-8'),invocation,(ROOT/'first.stderr.txt').stat().st_size,parent['reported_controls'],proof,config,helper)
  scored['summary'].update(manifest_sha256=sha(ROOT/'manifest.json'),evaluator_freeze_sha256=sha(ROOT/'evaluator-freeze.json'),raw={'stdout_sha256':sha(ROOT/'first.stdout.jsonl'),'stderr_sha256':sha(ROOT/'first.stderr.txt'),'invocation_sha256':sha(ROOT/'first-invocation.json')})
 except Exception as exc:
  scored={'summary':{'identity':'engine073-launch-independent-v25','outcome':'FAIL','decision':'NO_GO','gate':{'gate_pass':False,'blockers':['evaluator_or_runtime_integrity_error']},'planned_cases':60,'observed_cases':None,'passed':None,'failed':None,'incomplete':60,'operational_errors':[{'type':type(exc).__name__,'error':str(exc)}],'provider_calls':0,'retries':0,'rescores':0},'cases':[]}
 write_new(ROOT/'first-results.json',scored);write_new(ROOT/'first-summary.json',scored['summary']);print(json.dumps(scored['summary'],indent=2))
 return 0
if __name__=='__main__':raise SystemExit(main())
