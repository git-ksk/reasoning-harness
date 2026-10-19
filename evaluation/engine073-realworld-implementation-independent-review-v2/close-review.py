"""Readonly input verification and exclusive closure of limited independent review."""
from pathlib import Path
from datetime import datetime, timezone
import difflib, hashlib, json, os, stat
OUT=Path(__file__).resolve().parent
ROOT=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
INPUT=Path('/private/tmp/engine073-realworld-evaluator-review-snapshot-v2')
DELTA3=Path('/private/tmp/engine073-realworld-workflow-delta-v3')
DELTA4=Path('/private/tmp/engine073-realworld-report-delta-v4')
DELTA5=Path('/private/tmp/engine073-realworld-case-decision-delta-v5')

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def save(name,value):
    data=value.encode() if isinstance(value,str) else (json.dumps(value,indent=2,ensure_ascii=False)+'\n').encode()
    with (OUT/name).open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
def verify_map(root,mapping):
    checked={}
    for name,want in mapping.items():
        p=root/name
        assert p.is_file(),str(p)
        got=sha(p);assert got==want,(str(p),want,got)
        checked[name]=got
    return checked
now=datetime.now(timezone.utc).isoformat()
initial=json.loads((OUT/'input-hashes.json').read_text())
assert sha(INPUT/'input-manifest.json')==initial['manifest_sha256']=='0e6cfbd4413d9b62323935c1b4a3d0425dd5690954e551d42af7518db5c4d3a5'
assert json.loads((INPUT/'input-manifest.json').read_text())==initial['files']
verification={'snapshot_v2':{'manifest_sha256':sha(INPUT/'input-manifest.json'),'files':verify_map(INPUT,initial['files'])},'deltas':{}}
for label,directory,want in [('v3',DELTA3,'6676873340bb9589a273322411aeafad367ea15edd9c4dde64594452788c7998'),('v4',DELTA4,'6529d379d1e4d8b32c931953a1920070b01f28b7f9add5c8193079d9dfaa9160'),('v5',DELTA5,'048333aa09b05946b34898f779e3b18a5cab198f99e5e3826bc94f1d4afd9a3a')]:
    assert sha(directory/'manifest.json')==want,label
    data=json.loads((directory/'manifest.json').read_text())
    mapping=data.get('files',data)
    verification['deltas'][label]={'manifest_sha256':want,'files':verify_map(directory,mapping)}
    save(label+'-manifest.snapshot.json',data)

diffs={}
for name,before,after in [('v3-workflow',INPUT/'engine073-realworld-independent-v1.yml',DELTA3/'engine073-realworld-independent-v1.yml'),('v4-scorer',INPUT/'scorer.py',DELTA4/'scorer.py'),('v4-score-first',INPUT/'score_first.py',DELTA4/'score_first.py'),('v5-scorer',DELTA4/'scorer.py',DELTA5/'scorer.py'),('v5-test',INPUT/'test_scorer.py',DELTA5/'test_scorer.py')]:
    text=''.join(difflib.unified_diff(before.read_text().splitlines(True),after.read_text().splitlines(True),fromfile=str(before),tofile=str(after)))
    save(name+'.diff',text);diffs[name]={'before_sha256':sha(before),'after_sha256':sha(after)}
assert sha(DELTA3/'engine073-realworld-independent-v1.yml')==sha(DELTA4/'engine073-realworld-independent-v1.yml')

old_manifest=ROOT/'evaluation/engine073-launch-independent-v27/final-artifact-manifest.json'
assert sha(old_manifest)=='abaae2385e6c17783cc7093c10e15424b6e4ce01080e8fcdea5eab3a9e5bb439'
old=json.loads(old_manifest.read_text())
old_checked=verify_map(ROOT,old['files']);old_checked[str(old_manifest.relative_to(ROOT))]=sha(old_manifest)
assert len(old_checked)==928
prep=verify_map(ROOT,old['separate_preparation_hashes']);assert len(prep)==5
actual_source={}
for n in range(1,28):
    directory=ROOT/'evaluation'/('engine073-launch-independent-v'+str(n))
    for p in directory.rglob('*'):
        if p.is_file():actual_source[str(p.relative_to(ROOT))]=sha(p)
assert actual_source=={k:v for k,v in old_checked.items() if k.startswith('evaluation/engine073-launch-independent-v')}
old_v1=Path('/private/tmp/engine073-realworld-implementation-independent-review-v1')
old_v1_manifest=old_v1/'artifact-manifest.json'
assert sha(old_v1_manifest)=='a3d457e64271a3e286e0fbd9cf776cd62ebe6cce83a547eeb313589cc477efae'
v1=json.loads(old_v1_manifest.read_text());v1_checked=verify_map(old_v1,v1['files']);assert len(v1_checked)==64
component_inventory=Path('/private/tmp/engine073-practical-component-independent-review-v1/initial-inventory.json')
components=json.loads(component_inventory.read_text())['component_files']
component_checked={name:verify_map(ROOT/'evaluation'/name,mapping) for name,mapping in components.items()}
verification.update({'old928':old_checked,'separate_preparation5':prep,'old_source_recursive_file_count':len(actual_source),
                     'old_review_v1':{'manifest_sha256':sha(old_v1_manifest),'files':v1_checked,'including_manifest':65},
                     'old_public_components':component_checked,'reviewed_diffs':diffs,'checked_utc':now})
save('preservation-and-input-verification.json',verification)

report={
 'identity':'engine073-realworld-implementation-independent-review-v2',
 'verdict':'BOUNDED_CODE_REVIEW_PASS_WITH_EXPLICIT_V3_V4_V5_DELTAS',
 'original_snapshot_verdict':'NOT_RUNNABLE_WORKFLOW_M1_ARGV_MISSING_AND_POSITIVE_UNSUPPORTED_CASE_TP_INCONSISTENCY',
 'final_M0_or_acceptance':'NOT_COMPLETED_NOT_APPROVED_BY_THIS_REVIEW',
 'scope':'Known structural repair verification and supplied static deltas only; no new accuracy corpus or observations.',
 'independent_checks':{'supplied_structural_tests':{'tests':24,'failures':0,'errors':0,'reruns':0},
    'workflow_original_argv':{'returncode':2,'stage':'argparse before input/provider access','preserved':True},
    'v3_workflow_fix':'M1 hash computed before prepare; required --M1-sha256 supplied. Static only.',
    'v4_report_delta':'Baseline/candidate and family case/claim P/R/F1, citation null denominators and source counts added prospectively. Existing acceptance gates retained.',
    'v5_case_decision':{'function_checks':8,'pass':True,'unsupported_positive':'abstain => FN; claim recall0 and hard violations preserved','negative_and_ambiguous_unsafe':'accept => FP/hard preserved','grounded_positive':'accept/useful preserved'}},
 'fixed_known_boundaries':['factual occurrence uniqueness across displayed artifacts','response before terminal with real response schema and usage','explicit quote type and UTF8 range','citation edges cannot be orphaned','correct citation support must be admitted as well as acquired','helper displayed artifact kind matches final_answer schema','both arms complete and paired before useful gain','actual model trace request/response/usage binding','nonempty receipt and request target/source binding','strict complete M1 file inventory and hashes','partial helper raw output retained without completion promotion','provider plan and collection identity match','preHTTP wire size/reservation/single send audit guard'],
 'remaining_reproduced_code_blockers_after_supplied_deltas':[],
 'remaining_gates':[
   'Parent must compose and freeze the exact reviewed snapshot plus v3/v4/v5 bytes; this review does not validate an unprovided assembled revision.',
   'Actual Linux OCI build/image digest, runtime dependency closure and same-image M1 loading remain unverified by this reviewer.',
   'Actual paired host loopback controls, actual audit callback enforcement and approved provider/budget/quota configuration remain unverified; Groq remains stopped.',
   'Complete M0 review, post-M0 independent accuracy authorship, M1 freeze and first-only provider evaluation have not occurred. This is not real24, CLI quality, release or human acceptance.'
 ],
 'execution_accounting':{'accuracy_questions_authored':0,'Engine_accuracy_observations':0,'actual_cohort_scorer_processes':0,'provider_model_network_Oracle_calls':0,'new_agents':0,'implementation_edits':0,'Git_mutations':0},
 'preservation':{'snapshot_v2_files':28,'old_source_refs_plus_preparation':928,'preparation_files_included_in_928':5,'old_review_v1_files_including_manifest':65,'first_outputs_overwritten':False,'old_labels_scores_or_results_changed':False},
 'important_interpretation':'Original v2 case-TP issue could inflate case metrics, but hard unsupported/citation gates already prevented a final false GO. V5 corrects the prospective case metric without rescoring any accuracy observations.'
}
save('review.json',report)
save('summary.json',{'verdict':report['verdict'],'structural_tests_passed':24,'limited_v5_function_checks_passed':8,'remaining_reproduced_code_blockers':0,'actual_M0_or_realworld_acceptance_complete':False,'calls':report['execution_accounting']})
save('commands-and-runs.json',{'initial_tests':'Existing structural24-first.json/stderr retained; no rerun during closure.',
 'workflow_original':'Exact argv and rc2 retained in workflow-argv-repro.json; not re-executed after fix.',
 'case_decision':{'command':['python3','-B',str(OUT/'reproduce-case-decision.py')],'env':{'PYTHONDONTWRITEBYTECODE':'1'},'returncode':0,'kind':'pure supplied structural fixture, not scorer CLI or actual cohort'},
 'closure':{'command':['python3','-B',str(OUT/'close-review.py')],'env':{'PYTHONDONTWRITEBYTECODE':'1'},'source_edits':False}})
save('README.md','''限定独立再レビューを閉鎖しました。既知境界の修正と別提供v3/v4/v5差分を合わせたコードレビューはPASSです。元v2のworkflow必須引数欠落(rc2)と、未根拠positiveがcase TPになる反例はそのまま保存しています。v3は前者を静的に修正、v5は後者を8件の純構造対照で修正確認しました。独立実行した既存24構造試験は全PASSです。v4のbaseline/family別P/R/F1・引用分母nullの集計は記述追加で、合否gateを変更していません。\n\nこれは完全M0や本評価のPASSではありません。実Linux image/digestとdependency closure、実ホストloopback、予算・停止の実callback経路、Groq回復は独立未検証です。新精度ケース、gold、Engine精度観測、実cohort scorer、provider/model/network/Oracle呼出しは0です。親が実装を統合し完全M0を固定するまで、新24問著作や実行は開始しません。\n\n旧928参照(うちprep5)、元v1レビュー65ファイル、旧公開部品、v2全28入力とv3/v4/v5のmanifest/hashを再照合しました。原firstと既存スコアは変更していません。入力・diff・再現・原出力・レビューの全hashは自己除外artifact-manifest.jsonに保存し、全ファイルをread-only化しています。\n''')
save('receipt.json',{'identity':report['identity'],'closed_utc':now,'author':'/root/engine07_realworld_independent','scope':report['scope'],'input_manifest_sha256':initial['manifest_sha256'],'delta_manifests':{k:v['manifest_sha256'] for k,v in verification['deltas'].items()},'preserved_first_artifacts':True,'no_actual_accuracy_or_external_calls':True,'state':'CLOSED_NO_NEXT_AUTHORING_OR_EXECUTION','manifest_self_excluding':True})
for p in OUT.rglob('*'):
    if p.is_file():
        with p.open('rb') as f:os.fsync(f.fileno())
        p.chmod(0o444)
files={str(p.relative_to(OUT)):sha(p) for p in sorted(OUT.rglob('*')) if p.is_file()}
manifest={'identity':report['identity'],'closed_utc':now,'self_exclusion':'artifact-manifest.json only','files':files,'files_self_excluding':len(files),'files_including_manifest':len(files)+1,'input_manifest_sha256':initial['manifest_sha256'],'delta_manifests':{k:v['manifest_sha256'] for k,v in verification['deltas'].items()},'result':report['verdict'],'actual_accuracy_and_external_calls':0}
save('artifact-manifest.json',manifest)
(OUT/'artifact-manifest.json').chmod(0o444)
for p in OUT.rglob('*'):
    if p.is_file():assert stat.S_IMODE(p.stat().st_mode)==0o444
for k,v in files.items():assert sha(OUT/k)==v
for p in sorted((x for x in OUT.rglob('*') if x.is_dir()),reverse=True):p.chmod(0o555)
fd=os.open(OUT,os.O_RDONLY);os.fsync(fd);os.close(fd);OUT.chmod(0o555)
print(json.dumps({'path':str(OUT),'final_anchor':sha(OUT/'artifact-manifest.json'),'files_self_excluding':len(files),'files_including_manifest':len(files)+1,'old928_unchanged':True,'old_v1_review65_unchanged':True,'snapshot_and_deltas_unchanged':True,'verdict':report['verdict']},indent=2))
