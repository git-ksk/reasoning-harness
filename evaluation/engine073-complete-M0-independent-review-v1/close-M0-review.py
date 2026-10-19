from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,stat
OUT=Path(__file__).resolve().parent
ROOT=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(name,value):
 data=value.encode() if isinstance(value,str) else (json.dumps(value,indent=2,ensure_ascii=False)+'\n').encode()
 with (OUT/name).open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
def verify(root,files):
 checked={}
 for name,want in files.items():
  got=sha(root/name);assert got==want,(str(root/name),got,want);checked[name]=got
 return checked
oldmanifest=ROOT/'evaluation/engine073-launch-independent-v27/final-artifact-manifest.json'
assert sha(oldmanifest)=='abaae2385e6c17783cc7093c10e15424b6e4ce01080e8fcdea5eab3a9e5bb439'
old=json.loads(oldmanifest.read_text());oldrefs=verify(ROOT,old['files']);oldrefs[str(oldmanifest.relative_to(ROOT))]=sha(oldmanifest);assert len(oldrefs)==928
prep=verify(ROOT,old['separate_preparation_hashes']);assert len(prep)==5
prior={}
for directory,want,count in [('/private/tmp/engine073-realworld-implementation-independent-review-v1','a3d457e64271a3e286e0fbd9cf776cd62ebe6cce83a547eeb313589cc477efae',65),('/private/tmp/engine073-realworld-implementation-independent-review-v2','77a757f6d4fd03095649c39c0388260abf2b0dbbd8499136d1c5d4b04f94cade',28),('/private/tmp/engine073-groq-readiness-independent-review-v1','9430973f4cfdb46ff43fd64ef47b9ec78bbc55d80d1ff201806e02cfe3079117',16)]:
 p=Path(directory);m=p/'artifact-manifest.json';assert sha(m)==want;d=json.loads(m.read_text());checked=verify(p,d['files']);assert len(checked)+1==count;prior[directory]={'manifest_sha256':want,'files_including_manifest':count,'all_bytes_unchanged':True}
oldbundle=Path('/private/tmp/engine073-realworld-M0-runtime-39205043793/engine073-realworld-M0-runtime-39205043793')
oldinventory=json.loads((OUT/'initial-runtime-inventory.json').read_text());verify(oldbundle,{k:v['sha256'] for k,v in oldinventory.items()})
newbundle=Path('/private/tmp/engine073-realworld-M0-runtime-39206178872');newinventory=json.loads((OUT/'v2-runtime-input-inventory.json').read_text());verify(newbundle,{k:v['sha256'] for k,v in newinventory.items()})
quota=json.loads((OUT/'initial-quota-input-hashes.json').read_text());assert all(sha(Path(path))==want for path,want in quota.items())
draft=json.loads((OUT/'M0-receipt.reviewed-draft.json').read_text());assert sha(OUT/'M0-receipt.reviewed-draft.json')=='199898c1d54b70d4d77605849ed5cc7e0abd95b668c00c113f7a7f0f61cd70b3'
assert sha(Path('/private/tmp/engine073-realworld-M0-v1-staging/receipt.draft.json'))==sha(OUT/'M0-receipt.reviewed-draft.json')
assert sha(OUT/'groq-recovery.reviewed.json')==draft['required_provider_readiness']['groq']['recovery_sha256']=='4dc4ac84781e3694ccdcfeca4554f16358620a32445e51ed08995455d6b42aed'
runtime=json.loads((OUT/'runtime-v2-offline-verification.json').read_text());assert runtime['pass'] is True
boundaries=json.loads((OUT/'provider-readiness-boundaries.json').read_text());assert boundaries['all_pass'] is True
pid1=json.loads((OUT/'pid1-containment-result.json').read_text());assert pid1['pass'] is True
assert 'Ran 24 tests' in (OUT/'v2-pure-host-structural24.stderr').read_text() and (OUT/'v2-pure-host-structural24.stderr').read_text().rstrip().endswith('OK')
assert 'Ran 24 tests' in (OUT/'runtime-structural24.stderr').read_text() and (OUT/'runtime-structural24.stderr').read_text().rstrip().endswith('OK')
fs=json.loads((OUT/'runtime-filesystem-hash-comparison.json').read_text());assert fs['old_file_or_link_count']==fs['new_file_or_link_count']==5934 and not fs['base_changed']
save('preservation-verification.json',{'old928_including_prep5':oldrefs,'preparation5':prep,'prior_closed_reviews':prior,'old_runtime_bundle_unchanged':True,'new_runtime_bundle_unchanged':True,'quota_v2_workflow_and_metadata_unchanged':True,'M0_draft_and_recovery_unchanged':True})
now=datetime.now(timezone.utc).isoformat()
report={
 'identity':'engine073-complete-M0-independent-review-v1','closed_utc':now,
 'result':'PASS',
 'result_scope':'Independent software/runtime/protocol M0 implementation review for the fixed 9bdddd2 runtime, with Mistral/Google execution allowed and Groq explicitly deferred. This is not model accuracy, full-three-provider acceptance, CLI distribution or release approval.',
 'reviewer':'/root/engine07_realworld_independent',
 'reviewed_M0_draft_sha256':sha(OUT/'M0-receipt.reviewed-draft.json'),
 'reviewed_runtime_files_sha256':draft['runtime_file_inventory_canonical_sha256'],
 'runtime':{'execution_source_commit':draft['execution_source_commit'],'artifact_run':draft['runtime_build_run'],'artifact_id':draft['runtime_artifact_id'],'archive_sha256':draft['runtime_image_archive_sha256'],'config_digest':draft['runtime_image_id'],'OCI_manifest_digest':runtime['OCI_manifest_digest'],'runtime_repo_files':63,'paired_identity_sha256':draft['paired_host_identity_sha256'],'real_non_test_binary_count':8,'same_CLI_host':'ec6e55ff95b8364f9a5d09b8bbfe0ecc0c9d19d7','CLI_version_both':'reason 0.5.5','baseline_engine_commit':draft['baseline_source_commit'],'candidate_engine_commit':draft['candidate_source_commit'],'same_audit_provider_patches':True},
 'independent_evidence':{
   'previous_fixed_image_actual_Linux_execution':{'structural_tests':24,'loopback_controls':18,'loopback_HTTP_requests':12,'external_provider_requests':0,'platform':'Colima Linux with amd64 QEMU emulation on arm64; fixed OCI content. Not a native GitHub repeat.'},
   'actual_PID1_containment':pid1,
   'new_image_evidence_binding':{'offline_OCI_config_layers_and_all_repo_inventory':True,'full_effective_file_link_mode_comparison_count_each':5934,'only_six_declared_repo_files_changed':True,'all_interpreter_library_binary_callback_and_inherited_runner_bytes_identical':True,'new_image_not_independently_Docker_executed':True,'reason':'Approval-service availability constraint; used expressly authorized sandbox tar parsing and exact byte-equivalence evidence instead. No bypass.'},
   'new_code_host_pure_structural_tests':{'tests':24,'pass':True,'runtime':'host Python, not a new-image execution'},
   'new_provider_metadata_arithmetic_boundaries':{'checks':5,'pass':True,'draft_rejected_as_complete_M0':True,'wrong_provider_readiness_not_borrowed':True,'wrong_model_rejected':True,'deferred96_not_semantic_TN_or_defined_actual_F1':True}},
 'supplied_CI_evidence_not_independent_reexecution':{'new_image_run':39206178872,'CI_structural_tests':24,'current_image_CI_loopback_controls':18,'complete_logs_hash_bound':True},
 'Groq_quota_v2_review':{'workflow_sha256':quota['/private/tmp/engine073-groq-actual-adapter-readiness-v2.yml'],'runtime_metadata_sha256':quota['/private/tmp/engine073-groq-readiness-v2-runtime.json'],'one_actual_adapter_generation_attempt':True,'precredential_local_currency_reservation_USD':'0.004864','self_cap_USD':'0.005','input_cap':4096,'output_cap':384,'outer_timeout_seconds':110,'no_retry_redirect_tier_or_model_switch':True,'readiness_raw_run':39205645436,'observed_input_tokens':246,'observed_output_tokens':32,'observed_total_tokens':278,'observed_TPM':8000,'observed_RPD':1000,'declared_external_TPD':200000,'TPD_verification_scope':'External recovery document; no new web inquiry by reviewer. Observed TPM itself is below unchanged100000 recovery gate.','full_cohort_token_reserve':1456128,'new_live_run_allowed':False,'prior_nonJSON_edge1010_usage':'unknown, retained, never zero/refunded/retried'},
 'frozen_provider_readiness':draft['required_provider_readiness'],
 'global_acceptance':'NO_GO: Groq deferred, no real24 authored/executed or actual-answer F1 measured',
 'permitted_post_review_receipt_finalization_only':[
   'state DRAFT_AWAITING_INDEPENDENT_REVIEW -> COMPLETE_FROZEN',
   'completed_unix_seconds null -> actual integer completion timestamp, before independent accuracy authorship',
   'independent_implementation_review.result PENDING -> PASS and manifest_sha256 null -> this review final manifest hash',
   'runtime_controls wording clarified as CI_structural_tests24/current_image_CI_loopback_controls18 and independent_previous_runtime_structural_tests24/loopback18, with same-byte evidence reference; no assertion of independent whole-new-image execution'
 ],
 'unchanged_finalization_requirements':['runtime63 inventory canonical9be785406c62e80d19c11543717fdc40aec358757e38358dd9af523e9ceb09b7','all source/runtime/binary/protocol/profile/budget hashes','mistral=true/google=true/groq=false with recovery4dc4ac84781e3694ccdcfeca4554f16358620a32445e51ed08995455d6b42aed','all-three-provider gate unchanged; deferred remains96 planned/0 observed/null actual F1/overall NO_GO','fresh authorship only after the parent fixes the completed receipt and separately explicitly authorizes it; M1 before first-only execution'],
 'remaining_implementation_blockers_for_this_software_M0_scope':[],
 'limits':['This review proves fixed implementation/runtime bindings and observed structural stop/budget behavior, not semantic answer accuracy or future provider service availability.','Price ceilings are frozen local accounting assumptions, not an invoice guarantee.','External time/usage/quota failure during actual first evaluation remains incomplete/NO_GO, never semantic success.','The first Colima orchestration stopped after both test subprocesses succeeded because VM tmp was not host-shared; recovered their same first raw outputs without rerunning tests.','The first offline layer-identity assertion stopped because layer packaging digests changed; full5934 file/link/mode comparison then established equal functional dependencies. Original code/errors retained; no accuracy observation/rescoring occurred.','Cooperative artifact freeze and container PID namespace containment were reviewed; no adversarial host writer or killed-whole-Docker-daemon guarantee is claimed.'],
 'execution_accounting_by_reviewer':{'new_accuracy_questions_or_gold':0,'Engine_accuracy_observations':0,'actual_cohort_scorer_processes':0,'external_provider_model_network_Oracle_calls':0,'Git_source_or_existing_artifact_changes':0,'new_agents':0},
 'preservation':{'old_references_including_manifest':928,'preparation_files_included_in928':5,'previous_reviews_unchanged':prior,'all_initial_failures_and_raw_preserved':True}
}
save('review.json',report)
save('summary.json',{'result':'PASS','scope':report['result_scope'],'reviewed_runtime_files_sha256':report['reviewed_runtime_files_sha256'],'new_runtime_archive_sha256':draft['runtime_image_archive_sha256'],'Mistral_allowed':True,'Google_allowed':True,'Groq_allowed':False,'global_acceptance':'NO_GO','actual_answer_F1':None,'new_accuracy_authored':0,'external_calls_by_reviewer':0,'prior_actual_independent_controls':'24 structural +18 loopback +PID1 containment','new_code_pure_host_tests':24,'pure_provider_boundaries':5})
save('README.md','''固定9bdddd2のsoftware/runtime/protocol M0実装レビューはPASSです。Mistral/Googleだけ実行許可、Groq延期を前提とします。実回答F1・3社受入・CLI配布・releaseのPASSではありません。新accuracy著作・gold・Engine精度観測・実cohort採点・外部API/Oracle呼出しは0です。\n\n新imageをsandbox内で解析し、archive/config・63 runtime files・8 actual non-test binaries・paired host identityを照合しました。旧/新imageの全5934 file/link/mode比較では、差は宣言された6repoファイルだけで、interpreter/libs/callback/runner/binaryは同一です。旧固定imageで独立24構造試験・18loopbackを実行、実runner PID1をSIGKILLし、子/孫のheartbeat停止、inflight予約保持、欠測を成功扱いしないこと、同output再起動拒否を確認しました。ローカルはColima/QEMU条件です。新whole imageの独立Docker再実行はしておらず、同bytes証明と新image CI18/24を区別して記録しました。新codeのhost純24試験と5 metadata/算術境界もPASSです。\n\nGroq実SDK readinessは外部原記録にbindし、1attempt・usage278・TPM8000を確認。全量予約1456128は記録TPD200000に収まらず、Groqは認証前deferred、96 planned/0 complete/F1 null、全3社GOはNO_GOです。旧edge1010/unknown usageは保持し返金/再試行扱いにしていません。\n\n親は草稿のstate/time/review manifest参照と独立/CI証拠の説明だけを後充填してfinal receiptを固定できます。その他のhash/設定/readinessは固定のままです。新資料著作は、そのfinal receiptの固定と別の明示許可後にのみ開始します。原928参照（prep5含む）と既存reviewを照合し、全first/失敗と自己除外manifestをread-onlyで閉鎖しました。\n''')
save('receipt.json',{'identity':report['identity'],'result':'PASS','closed_utc':now,'reviewed_M0_draft_sha256':report['reviewed_M0_draft_sha256'],'reviewed_runtime_files_sha256':report['reviewed_runtime_files_sha256'],'new_runtime_archive_sha256':draft['runtime_image_archive_sha256'],'provider_execution_scope':{'mistral':True,'google':True,'groq':False},'full_three_provider_acceptance':'NO_GO','actual_answer_F1':None,'new_accuracy_authored':0,'state':'REVIEW_CLOSED_AWAIT_FINAL_PARENT_RECEIPT_AND_EXPLICIT_AUTHORING_AUTHORIZATION'})
for p in OUT.rglob('*'):
 if p.is_file():
  with p.open('rb') as f:os.fsync(f.fileno())
  p.chmod(0o444)
files={str(p.relative_to(OUT)):sha(p) for p in sorted(OUT.rglob('*')) if p.is_file()}
manifest={'identity':report['identity'],'closed_utc':now,'self_exclusion':'artifact-manifest.json only','files':files,'files_self_excluding':len(files),'files_including_manifest':len(files)+1,'result':'PASS','scope':report['result_scope'],'reviewed_runtime_files_sha256':report['reviewed_runtime_files_sha256'],'new_runtime_archive_sha256':draft['runtime_image_archive_sha256'],'old928_unchanged':True,'new_accuracy_authored':0,'external_calls_by_reviewer':0}
save('artifact-manifest.json',manifest);(OUT/'artifact-manifest.json').chmod(0o444)
for name,want in files.items():assert sha(OUT/name)==want
for p in OUT.rglob('*'):
 if p.is_file():assert stat.S_IMODE(p.stat().st_mode)==0o444
for p in sorted([x for x in OUT.rglob('*') if x.is_dir()],reverse=True):p.chmod(0o555)
fd=os.open(OUT,os.O_RDONLY);os.fsync(fd);os.close(fd);OUT.chmod(0o555)
print(json.dumps({'path':str(OUT),'files_self_excluding':len(files),'files_including_manifest':len(files)+1,'manifest_sha256':sha(OUT/'artifact-manifest.json'),'result':'PASS','reviewed_runtime_files_sha256':report['reviewed_runtime_files_sha256'],'Groq_deferred':True,'global_acceptance':'NO_GO','new_accuracy_authored':0},indent=2))
