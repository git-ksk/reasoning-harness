from pathlib import Path
import json,hashlib,shutil,re,subprocess,gzip
r=Path.cwd();pin=json.loads(Path('/private/tmp/engine073-source29-root-pin.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==pin['code_commit']
assert sha(r/'crates/reasoning-harness-core/src/evidence_relevance.rs')==pin['source_sha256']
loc=r/'evaluation/engine073-launch-development-v27-bounded-properties';assert json.loads((loc/'summary.json').read_text())['v27']['joint_passed']==60
counts=[]
for name in ['clippy','fmt','workspace','providers','fixtures','archive','quality','source-policy','claim-metrics']:
 p=Path('/private/tmp')/f'engine073-source29-final1-verified-{name}.log';assert p.exists(),name;shutil.copyfile(p,loc/(name+'.log'))
 if name in ['workspace','providers']:counts.extend(tuple(map(int,x)) for x in re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;',p.read_text()))
assert counts
actual=tuple(sum(row[i] for row in counts) for i in range(3));assert actual==(1605,0,8),actual
assert (loc/'fixtures.log').read_bytes()==(r/'evaluation/engine073-launch-development-v26-local-references/fixtures.log').read_bytes()
for n in ['clippy','fmt','workspace','providers','fixtures','archive','quality','source-policy','claim-metrics']:
 assert 'test result: FAILED.' not in (loc/(n+'.log')).read_text()
summary=json.loads((loc/'summary.json').read_text());summary['whole_validation']={'passed':1605,'failed':0,'existing_ignored':8,'workspace_strict_clippy_fmt':'PASS','fixtures_byte_equal_previous':True,'archive':'PASS','quality_tests':10,'source_policy_tests':10,'claim_display_component_tests':32};summary['scope']='Consumed development/source compatibility only; not fresh independent, real answers, release or CLI acceptance.';(loc/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
shutil.copyfile('/private/tmp/engine073-source29-measure-actual.py',loc/'measure-actual.py');shutil.copyfile('/private/tmp/engine073-source29-final1-validate.py',loc/'validate.py');shutil.copyfile('/private/tmp/engine073-source29-actual-build.jsonl',loc/'cargo-build.jsonl');shutil.copyfile('/private/tmp/engine073-source29-actual-build.stderr',loc/'cargo-build.stderr')
pro=r/'evaluation/engine073-launch-development-v27-bounded-prototypes';assert not pro.exists();pro.mkdir()
for label,path in [('repair','/private/tmp/engine073-source29-v27-repair'),('normal-stage01','/private/tmp/engine073-source29-stage01'),('known-stage01','/private/tmp/engine073-source29-known-stage01')]:
 out=pro/label;out.mkdir()
 for p in Path(path).iterdir():
  if p.is_file() and p.name not in ['probe']:
   shutil.copyfile(p,out/p.name)
for n in ['add-tests','stage01','stage01-before-scope','measure','known']:
 p=Path('/private/tmp')/f'engine073-source29-{n}.py';shutil.copyfile(p,pro/p.name)
(pro/'README.md').write_text('''# Consumed v27 repair development

Original first v27 at26c remainsF1.888889/NO_GO. Six new public-behavior groups first fail (367PASS/6FAIL). One shared semantic stage repairs complete physical/printed types,optional heading article,same-role quoted owner,finite coordinated role references and bounded property coverage;foreign apposition owners must match. Stage01 source2e9207177d9f1e91b8c031322dbbf83c2b374d8847e123e135dcca727b0214de:core373PASS,strict core/all-targetsClippyPASS,normal consumedv27 eachTP30/FP0/TN30/FN0,F1=1.0. Prior normal1400 controls pass with this identical source. Original labels/scorer/raw are untouched. Setup edits failed in memory before any source write,then were scoped to exact function/test anchors;no semantic first rerun.

No further new synthetic holdout authored. Development perfection/count is not the acceptance criterion. Future independent F1/P/R and safety remain separate;next focus is actual-document/ordinaryCLI traces and complete semantic scoring. No Engine release or CLI adoption. Binaries/caches are omitted;compiler/library/executable digests remain provenance only.
''')
(loc/'README.md').write_text('''# Fixed source repair and limited actual-root verification

CodeCODE/source2e9207177d9f1e91b8c031322dbbf83c2b374d8847e123e135dcca727b0214de;package0.7.2 unchanged. Same consumedv27 firstF1.888889→development1.000 (eachTP30/FP0/TN30/FN0),not fresh acceptance. Actual normal-root outputs equal private normal artifact;v29 all60 match original first. Original audit14 passes,historical597 outputs byte-identical. Root measurement3processes/671inputs;prior1400 controls already measured once with the identical source/normal artifact are referenced,not redundantly rerun. This is source/behavior evidence,not practical answerF1.

Full workspace1605PASS/0FAIL/8existing ignored,strict workspace/all-targetsClippy/fmtPASS,fixtures byte-equal previous,archivePASS;quality10/source-policy10/claim-display32PASS. Existing local OAuth mocks run with the required localhost permission;no live provider/model/Oracle/Groq call. Source/HEAD/library/input/executable hashes remain pinned throughout. Frozen v27 firstFAIL and928corpus references are unchanged. New synthetic60 author/repair loop is stopped for now;see evaluation-focus. Real24=0/24,actual-answer F1 unmeasured,complete realM0 and independent component review pending,release/CLI adoptionNO_GO.
'''.replace('CODE',pin['code_commit']))
for d in [loc,pro]:
 (d/'artifact-sha256.json').write_text(json.dumps({str(p.relative_to(d)):sha(p) for p in sorted(d.rglob('*')) if p.is_file() and p.name!='artifact-sha256.json'},indent=2)+'\n')
# Refresh latest status while retaining original independent history.
ja='''修正候補 `CODE` は、同じ観測済みv27の開発評価で両API TP30/FP0/TN30/FN0、P/R/F1=1.000。元の独立v27初回F1 .888889/Recall .8/NO_GOは変更しない。完全な印字属性・同roleのowner/複数参照・括弧内の属性限定を共通処理で修正した。修正した不具合と安全性境界の退行を確認し、全体1605PASS/0FAIL/既存ignore8・strict Clippy/fmt、元14・歴史597byte互換を保存。過去1400は同じsourceの通常成果物で一度確認し、rootでは必要な60/14/597照合に絞った。件数・開発満点を完成度にせず、次の合成60件は作らない。次は実資料・通常CLIのfeature-active経路/traceと完全採点器・数値予算へ重点を戻す。主張算術/表示binding部品32検証は準備のみ、独立部品レビューは利用上限で未完了。全928原参照は不変。実24問0/24・実回答F1未測定・Groq停止、sourceマージ/Engine公開/CLI採用NO_GO。[方針](engine073-evaluation-focus.ja.md)、[保存済み修正](../evaluation/engine073-launch-development-v27-bounded-properties/README.md)、[原独立判定](engine073-v27-decision.ja.md)。'''.replace('CODE',pin['code_commit'][:7])
en='''Repair candidate `CODE` passes the same consumedv27 development set:eachAPI TP30/FP0/TN30/FN0,P/R/F1=1.000. Original independentv27 F1.888889/Recall.8/NO_GO remains unchanged. Shared complete printed types,same-role owner/coordinated references and bounded property coverage are repaired. Repaired-behavior/safety regressions checked;workspace1605PASS/0FAIL/8ignored,strictClippy/fmt,original14 andhistory597 byte compatibility saved. Prior1400 run once on identical source through a normal artifact;root verification limited to60/14/597 rather than repeated wholesale. Count/development perfection is not readiness. No next synthetic60 is authored;prioritize actual-document/ordinaryCLI feature-active paths,traces,complete scoring and numeric budgets. Claim/display32 is preparation only;independent component review incomplete due usage limit. All928original references immutable. Real24=0/24,actual-answer F1 unmeasured,Groq stopped,source merge/release/CLI adoptionNO_GO. See[focus](engine073-evaluation-focus.md),[saved repair](../evaluation/engine073-launch-development-v27-bounded-properties/README.md),[first decision](engine073-v27-decision.md).'''.replace('CODE',pin['code_commit'][:7])
for base in ['project-status','product-roadmap','engine-0.7.3-roadmap','engine-073-research-informed-repair']:
 for lang,text,prefix in [('ja',ja,'候補 `26c98ad`'),('en',en,'Fresh independent v27 at candidate `26c98ad`')]:
  p=r/'docs'/(base+('.ja' if lang=='ja' else '')+'.md');parts=p.read_text().split('\n\n');hits=[i for i,x in enumerate(parts) if x.startswith(prefix)];assert len(hits)==1,(p,hits);parts[hits[0]]=text;s='\n\n'.join(parts)
  s=s.replace('26c98adの独立v27初回はF1 .888889/FP0/Recall .8、NO_GO。既知1454/1460の6保留を修正し新候補で回帰確認。全体1599/CI19成功は26cの別検証。','ab3c632の消費済みv27開発F1=1、元独立v27 .888889/NO_GO不変。全体1605成功。次の合成60追加は止め、実資料/通常CLIの評価器と経路へ重点移行。')
  s=s.replace('26c98ad independentv27 firstF1.888889/FP0/Recall.8 NO_GO;known1454/1460;repair6 misses and verifynewcandidate;workspace1599/CI19 are26c checks;','ab3c632 consumedv27 developmentF1=1;originalv27 first.888889/NO_GO unchanged;workspace1605PASS;stop freshsynthetic60 loop and prioritize actual-document/ordinaryCLI scoring/path;')
  p.write_text(s)
for lang,text in [('ja',ja),('en',en)]:
 p=r/'docs'/('engine073-v27-decision'+('.ja' if lang=='ja' else '')+'.md');p.open('a').write('\n## '+('保存した開発修正と評価の重点' if lang=='ja' else 'Saved development repair and evaluation focus')+'\n\n'+text+'\n')
print(json.dumps({'code':pin['code_commit'],'workspace':actual,'current_development_f1':1.0,'original_independent_f1':.8888888888888888,'next_synthetic_authorship':0,'actual_answer_F1':'UNMEASURED'}))
