#!/usr/bin/env python3
"""Hash/copy first-result closure only. No Engine, scorer or helper invocation."""
from pathlib import Path
from collections import Counter,defaultdict
import sys,os,json,subprocess
sys.path.insert(0,str(Path(__file__).resolve().parent))
import evaluator_contract as c
P=c.ROOT;R=c.REPO;IDENTITY='engine073-launch-independent-v27'
M0_SHA='aaa4b0d2494c3a89d87b3c0714847ee8d468f84295e1f868e7518398e41635bb'
M1_SHA='fb8a350de68ee6dfc61e4d440411a96eb217b54a4fdec7bfe5a6ebf02709d82c'
OLD_ANCHOR='8214db3607e32d707da797fb90bb4af77b1101db976b6f0d0a60f96d49cdb921'
FIRST_NAMES=['driver.started.json','driver.stdout.txt','driver.stderr.txt','driver.invocation.json','execution.started.json','first-run.stdout.txt','first-run.stderr.txt','first-invocation.started.json','first-invocation.json','first.stdout.jsonl','first.stderr.txt','first-score.started.json','first-score.stdout.txt','first-score.stderr.txt','first-score.invocation.json','first-results.json','first-summary.json']
NEW_NAMES=['closure-preparation-error.json','first-preservation-hashes.json','risk-diagnostics.json','observation-review.json','README-results.md','closure-input-hashes.json','closure-receipt.json','final-artifact-manifest.json']
assert not any((P/n).exists() for n in NEW_NAMES),'existing_closure_no_overwrite'
assert c.sha(P/'evaluator-freeze.json')==M0_SHA and c.sha(P/'manifest.json')==M1_SHA
m0=c.evaluator_integrity();m1=c.load(P/'manifest.json')
assert c.preserved()==(824,5)
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()==c.SOURCE
assert all(c.sha(P/n)==h for n,h in m1['frozen_files'].items()) and c.checked_artifacts(m1['checked_artifacts'])
first_hashes={n:c.sha(P/n) for n in FIRST_NAMES}
s=c.load(P/'first-summary.json');results=c.load(P/'first-results.json');rows=results['cases']
cases={x['id']:x for x in c.load(P/'cases.json')['cases']}
scope=c.load(P/'scope-review.json');scope_pairs={x['pair_id']:x for x in scope['pair_reviews']}
assert len(rows)==60 and len(cases)==60 and len({x['id'] for x in rows})==60 and s==results['summary']
inv=c.load(P/'first-invocation.json');score_inv=c.load(P/'first-score.invocation.json');driver=c.load(P/'driver.invocation.json')
assert inv['state']=='finished' and type(inv['returncode']) is int and inv['returncode']==0 and inv['execution_error'] is None
assert type(score_inv['returncode']) is int and score_inv['returncode']==0 and score_inv['scorer_invocations']==1
assert type(driver['returncode']) is int and driver['returncode']==1 and driver['first_execution_invocations']==1
assert all((P/n).stat().st_size==0 for n in ['driver.stderr.txt','first-run.stderr.txt','first.stderr.txt','first-score.stderr.txt'])
assert s['outcome']=='FAIL' and s['decision']=='NO_GO' and s['incomplete']==0 and s['operational_errors']==[]
assert all(s['runtime_provenance']['checks'].values()) and len(s['runtime_provenance']['checks'])==13
assert s['observed_cases']==60 and s['passed']==54 and s['failed']==6
for a in s['axes'].values():assert {k:a[k] for k in ['tp','fp','tn','fn']}=={'tp':24,'fp':0,'tn':30,'fn':6}
def new_json(n,d):c.write_new(P/n,d)
def new_text(n,t):
 with (P/n).open('x') as f:f.write(t);f.flush();os.fsync(f.fileno())
def desc(n):return {'sha256':c.sha(P/n),'bytes':(P/n).stat().st_size}
new_json('closure-preparation-error.json',{'identity':IDENTITY,'recorded_utc':c.utc(),'phase':'post-first closure script creation only','failed_shell_exitcode':1,'stderr_excerpt':'File "<stdin>", line 77\nSyntaxError: leading zeros in decimal integer literals are not permitted; use an 0o prefix for octal integers','cause':'An outer raw triple-single-quoted script string was terminated by an inner README triple-single-quoted string; the wrapper failed to parse before writing any file.','effects':'No closure file, Engine call, scorer call or helper call was made by the failed preparation command; original first bytes remain intact.','repair':'Created closure-save.py with a single literal shell heredoc under noclobber; closure persistence only, not semantic retry.'})
new_json('first-preservation-hashes.json',{'identity':IDENTITY,'recorded_utc':c.utc(),'first_saved_files':first_hashes,'count':len(first_hashes),'scope':'Original first bytes only; no rerun, rescore or label change.'})
literal=[{'id':x['id'],'pair_id':x['pair_id'],'polarity':x['polarity'],'complete':x['complete'],'status':x['status'],'exact_risk_diagnostic':x['exact_risk_diagnostic']} for x in rows]
new_json('risk-diagnostics.json',{'identity':IDENTITY,'recorded_utc':c.utc(),'source_first_results_sha256':first_hashes['first-results.json'],'rows':literal,'interpretation':'Literal tuples copied from original first scorer output. Supplied raw and effective qualification are separate axes. Enum names alone do not establish internal risk retention/loss or algorithmic causes.'})
def counted(items):
 z=Counter(json.dumps(v,sort_keys=True) for v in items)
 return [{'value':json.loads(k),'count':v} for k,v in sorted(z.items())]
by_pol={}
for pol in ['positive','negative']:
 a=[x for x in rows if x['polarity']==pol]
 by_pol[pol]={'cases':len(a),'status_counts':dict(Counter(x['status'] for x in a)),'raw_tuples':counted(x['exact_risk_diagnostic']['raw_provided_tuple'] for x in a),'effective_v17_tuples':counted(x['exact_risk_diagnostic']['effective_v17_tuple'] for x in a),'v29_dispositions':dict(Counter(x['exact_risk_diagnostic']['v29_disposition'] for x in a)),'v30_dispositions':dict(Counter(x['exact_risk_diagnostic']['v30_disposition'] for x in a))}
basis_status=defaultdict(Counter);failed=[]
for x in rows:
 g=cases[x['id']];basis_status[g['semantic_basis']][x['status']]+=1
 if x['status']!='pass':
  pair=scope_pairs[x['pair_id']];assert g['rationale']==pair[x['polarity']+'_rationale']
  failed.append({'id':x['id'],'pair_id':x['pair_id'],'polarity':x['polarity'],'axis':x['axis'],'semantic_basis':g['semantic_basis'],'original_pre_observation_rationale':g['rationale'],'original_expected':g['expected'],'original_input_signals':g['input']['candidate']['signals'],'scope_review_pair':pair,'first_observed_booleans':x['observed'],'complete':x['complete'],'status':x['status'],'exact_risk_diagnostic':x['exact_risk_diagnostic'],'v30_reasons':x['v30_reasons'],'checks':x['checks']})
review={'identity':IDENTITY,'reviewed_utc':c.utc(),'reviewer':'same sole authorized independent evaluator; not separate human adjudication','first_summary_sha256':first_hashes['first-summary.json'],'first_results_sha256':first_hashes['first-results.json'],'original_outcome':s['outcome'],'original_decision':s['decision'],'original_gate':s['gate'],'axes_copied_without_rescoring':s['axes'],'joint_exactness_diagnostic':{k:s[k] for k in ['passed','failed','positive_passed','negative_passed','incomplete']},'unsafe_false_positives_by_axis':{k:v['fp'] for k,v in s['axes'].items()},'operational_errors':s['operational_errors'],'literal_tuple_counts_by_polarity':by_pol,'semantic_basis_status_counts':{k:dict(v) for k,v in sorted(basis_status.items())},'nonpassing_cases':failed,'scope_preflight_correspondence':{'scope_review_sha256':c.sha(P/'scope-review.json'),'static_preflight_sha256':c.sha(P/'static-preflight.json'),'M1_sha256':M1_SHA,'all_original_rationales_match_pre_observation_pair_review':True,'pre_M1_corrections':scope['pre_M1_corrections'],'scope_review_Engine_calls':scope['Engine_invocations'],'scope_review_scorer_calls':scope['scorer_invocations'],'scope_static_pass_is_not_candidate_pass':True},'observation_limits':['All6 missed positives were supplied exact_target/requested_relation/none and returned effective exact_target/unresolved/none with v30 ambiguous. This is an output observation, not an internal-cause conclusion.','Four missed positives concern complete non-launch properties; one a complete foreign event; one a combination of separately safe finite roles. Original rationale/labels are retained unchanged.','The literal reason strings model_ambiguous/model_relevant and path model_assisted are API outputs with supplied raw values; no live model/provider was invoked.','This synthetic cohort does not measure real answers, ordinary CLI behavior, separate human agreement or arbitrary language correctness. Different authored cohorts do not support an improvement-rate claim.','Parent known1400/original14/history597/full1599 are pinned parent-reported controls, not evaluator replay. Source-scoped critical=[] does not complete #531/#540/CLI/release/Groq gates.'],'retry':0,'rescore':0,'relabel':0,'extra_Engine_scorer_helper_invocations':0}
new_json('observation-review.json',review)
new_text('README-results.md',f'''# v27 first-result closure

**FAIL / NO_GO** under the frozen #547 future source gate. Each API returned TP24 / FP0 / TN30 / FN6, precision1.0, recall0.8 and F1 0.8888888889. Both recall and F1 are below their precommitted floors (P>=.95/R>=.85/F1>=.90; FP0). Joint exactness54/60 is diagnostic only: positive24/30, negative30/30. Unsafe false positives0 per API, operational errors0 and incomplete0. No threshold, input, gold or first result was changed.

Candidate `{c.SOURCE}`, sourceSHA `{m0['checked_artifacts']['candidate_source']['sha256']}`, runtimeSHA `{m0['checked_artifacts']['probe_executable']['sha256']}`, public package0.7.2. M0 `{M0_SHA}` was reviewed before authoring. M1 `{M1_SHA}` pinned all60 inputs, independent gold/rationales, evaluator code and actual dependencies before observation. Scope/static checks used0 Engine/scorer invocations; they did not assert candidate success. All59 pure evaluator fixtures are earlier preparation evidence, not semantic observations.

The frozen driver made the first Engine invocation once (one process/60 inputs), one real scorer process and its one explicit-config helper call. Engine/recorder/scorer returncodes are0; driver rc1 is the semantic gateFAIL. Driver, recorder, Engine and scorer stderr are empty. All13 runtime-provenance checks are true. Original started/finished markers, stdout, stderr, score, booleans and complete failure records remain intact. Helper-call count follows the verified frozen single-call scorer success path; no separate runtime counter is claimed. Retries/rescores/relabels0; provider/model/network/Oracle0; Groq remains stopped.

The6 missed positives are `li27-005/025/029/045/049/053`: all supplied raw exact_target/requested_relation/none and returned effective exact_target/unresolved/none plus v30 ambiguous. Four correct a complete non-launch property, one corrects a uniquely foreign event and one contains three separately safe roles according to the unchanged pre-observation rationale. `observation-review.json` preserves original scope rationale and actual observations side by side. Literal tuples in `risk-diagnostics.json` do not establish internal component retention/loss or rule causes. No Engine implementation, current tests, parent diagnostics/development bodies or old cohort bodies were read for this evaluation.

All v1-v26 source824 files and separate preparation5 remain byte-identical (old anchor `{OLD_ANCHOR}`). The parent source-only receipt and its30 copies, runtime4 files and all41 checked artifacts are preserved by hash. Known1400/1400, original14/14, history597 whole-output equality and full1599/0/8 are parent-reported pinned controls, not independent reruns. Previous initial FAILs, labels and raw output remain unchanged.

A post-first closure-wrapper quoting error occurred before any file creation or execution. It is preserved in `closure-preparation-error.json`; its repair did not retry Engine, scorer or helper, and does not change the original semantic/operational counts.

This is a finite synthetic source API qualification, not #531 real24, normal CLI, live provider, human evaluation, Groq recovery or release acceptance. No cross-cohort improvement claim or arbitrary-language guarantee is made. Final manifest is self-excluding and separates source inventory, preparation5 and checked external artifacts. All final artifacts are fsynced/read-only. Candidate freeze remains held pending parent final copy/hash/commit/safety acknowledgment. The evaluator stops here; no next identity is authored or executed.
''')
input_names=['evaluator-freeze.json','manifest.json','authoring-start.json','parent-M0-review.opaque.json','author_cases.py','cases.json','inputs.json','scope-review.json','static-preflight.json','source_gate_policy.json','source_gate_helper.py','score.py','evaluator_contract.py','first-run.py','execute_first.py','probe.rs','risk-diagnostics.json','observation-review.json','README-results.md','closure-save.py','closure-preparation-error.json']+FIRST_NAMES
new_json('closure-input-hashes.json',{'identity':IDENTITY,'recorded_utc':c.utc(),'files':{n:desc(n) for n in dict.fromkeys(input_names)},'actual_checked_artifacts':m0['checked_artifacts'],'old_source_files_verified':824,'separate_preparation_files_verified':5,'parent_references_verified':30,'runtime_files_verified':4,'source_HEAD_verified':c.SOURCE,'no_Engine_scorer_helper_execution_by_closure':True})
new_json('closure-receipt.json',{'identity':IDENTITY,'closed_utc':c.utc(),'outcome':s['outcome'],'decision':s['decision'],'candidate':m0['candidate'],'M0_sha256':M0_SHA,'M1_sha256':M1_SHA,'policy_sha256':c.POLICY_SHA,'helper_sha256':c.HELPER_SHA,'scorer_sha256':c.sha(P/'score.py'),'axes_copied_from_first_summary':s['axes'],'joint_diagnostic':{'passed':s['passed'],'failed':s['failed']},'unsafe_false_positives_by_axis':{k:v['fp'] for k,v in s['axes'].items()},'operational_failures':0,'incomplete':s['incomplete'],'Engine_processes':1,'Engine_input_count':60,'real_scorer_processes':1,'helper_calls':1,'helper_calls_evidence':'one successful real scorer invocation and verified frozen single helper-call path; not an independent runtime counter','returncodes':{'Engine':inv['returncode'],'recorder':0,'scorer':score_inv['returncode'],'driver':driver['returncode']},'driver_nonzero_reason':'Frozen semantic gate FAIL, not operational failure','stderr_bytes':{n:(P/n).stat().st_size for n in ['driver.stderr.txt','first-run.stderr.txt','first.stderr.txt','first-score.stderr.txt']},'provenance_checks':s['runtime_provenance']['checks'],'first_preserved_files':first_hashes,'old_anchor':OLD_ANCHOR,'old_source_files':824,'separate_preparation_files':5,'parent_reference_files':30,'runtime_files':4,'checked_artifacts':41,'pre_M1_scope_corrections':scope['pre_M1_corrections'],'retry':0,'rescore':0,'relabel':0,'provider_model_network_Oracle_calls':0,'Groq':'stopped','compile_Cargo_sharedtarget_Git_source_root_prep_mutations_extra_agents_real24_authorship':0,'closure_operation_source_sha256':c.sha(P/'closure-save.py'),'closure_preparation_error_sha256':c.sha(P/'closure-preparation-error.json'),'candidate_freeze':'HELD: parent final copy/hash/commit/safety acknowledgment required before release','next_identity':'not authorized and not started','acceptance_boundary':'Finite #520 synthetic source APIs only; no real24/normalCLI/provider/human/release acceptance.'})
assert all(c.sha(P/n)==h for n,h in first_hashes.items())
assert c.evaluator_integrity() and c.preserved()==(824,5)
assert all(c.sha(P/n)==h for n,h in m1['frozen_files'].items()) and c.checked_artifacts(m0['checked_artifacts'])
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip()==c.SOURCE
old=c.load(P/'predecessor-artifact-hashes.json');prep=c.load(P/'preserved-preparation-hashes.json')
old_source={str((P.parent/('engine073-launch-independent-'+v)/n).relative_to(R)):h for v,fs in old.items() for n,h in fs.items()}
current={str(f.relative_to(P)):c.sha(f) for f in sorted(P.rglob('*')) if f.is_file()}
new_source={str((P/n).relative_to(R)):h for n,h in current.items()}
files={**old_source,**new_source,**prep};assert len(files)==len(old_source)+len(new_source)+5
parent_refs={a['path']:a['sha256'] for k,a in m0['checked_artifacts'].items() if k.startswith('parent_reference_')};assert len(parent_refs)==30
final={'identity':IDENTITY,'closed_utc':c.utc(),'state':'CLOSED_FIRST_FAIL_NO_GO','self_exclusion':'This final-artifact-manifest.json is excluded from its own file maps; its SHA256 is the externally reported anchor.','candidate':m0['candidate'],'M0_sha256':M0_SHA,'M1_sha256':M1_SHA,'old_anchor':OLD_ANCHOR,'old_source_files':824,'separate_preparation_files':5,'old_reference_files':829,'current_files_self_excluding':len(current),'current_files_including_final_manifest':len(current)+1,'source_files_self_excluding_final_manifest':len(old_source)+len(new_source),'source_files_including_final_manifest':len(old_source)+len(new_source)+1,'corpus_references_self_excluding_final_manifest':len(files),'corpus_references_including_final_manifest':len(files)+1,'files':files,'current_files':current,'separate_preparation_hashes':prep,'checked_artifacts':m0['checked_artifacts'],'parent_reference_hashes':parent_refs,'first_preserved_files':first_hashes,'outcome':s['outcome'],'decision':s['decision'],'axes_copied_from_first_summary':s['axes'],'joint_exactness_is_diagnostic':True,'joint_passed':s['passed'],'joint_failed':s['failed'],'incomplete':s['incomplete'],'operational_errors':s['operational_errors'],'unsafe_false_positives_by_axis':{k:v['fp'] for k,v in s['axes'].items()},'Engine_processes':1,'Engine_inputs':60,'scorer_processes':1,'helper_calls':1,'retry_rescore_relabel':0,'provider_model_network_Oracle_calls':0,'candidate_freeze':'HELD pending parent final copy/hash/commit/safety acknowledgment','next_evaluation_started':False}
new_json('final-artifact-manifest.json',final)
for f in P.rglob('*'):
 if f.is_file():
  with f.open('rb') as h:os.fsync(h.fileno())
  if f.stat().st_mode&0o222:os.chmod(f,0o444)
for d in sorted([q for q in P.rglob('*') if q.is_dir()]+[P],key=lambda x:len(x.parts),reverse=True):
 fd=os.open(d,os.O_RDONLY)
 try:os.fsync(fd)
 finally:os.close(fd)
assert all(c.sha(R/n)==h for n,h in final['files'].items())
assert {str(f.relative_to(P)) for f in P.rglob('*') if f.is_file()}==set(current)|{'final-artifact-manifest.json'}
assert all(not(f.stat().st_mode&0o222) for f in P.rglob('*') if f.is_file())
assert all(c.sha(P/n)==h for n,h in first_hashes.items())
assert c.evaluator_integrity() and c.preserved()==(824,5) and c.checked_artifacts(m0['checked_artifacts'])
assert all(c.sha(P/n)==h for n,h in m1['frozen_files'].items())
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=R,text=True).strip();assert head==c.SOURCE
allpaths={str(R/n) for n in final['files']}|{str(P/'final-artifact-manifest.json')}|{a['path'] for a in final['checked_artifacts'].values()}
print(json.dumps({'state':final['state'],'final_anchor':c.sha(P/'final-artifact-manifest.json'),'new_files_including_manifest':len(current)+1,'old_source_files':824,'source_files_self_excluding_manifest':final['source_files_self_excluding_final_manifest'],'source_files_including_manifest':final['source_files_including_final_manifest'],'prep_files':5,'corpus_references_self_excluding_manifest':len(files),'corpus_references_including_manifest':len(files)+1,'checked_artifacts':len(m0['checked_artifacts']),'unique_total_paths_including_manifest_and_external_artifacts':len(allpaths),'parent30_runtime4_old824_prep5_M0_M1_first17_unchanged':True,'HEAD':head,'source_sha256':m0['checked_artifacts']['candidate_source']['sha256'],'runtime_sha256':m0['checked_artifacts']['probe_executable']['sha256'],'axes':s['axes'],'incomplete':s['incomplete'],'ops':len(s['operational_errors']),'freeze':'HELD_PENDING_PARENT_ACK','next_evaluation_started':False},indent=2))
