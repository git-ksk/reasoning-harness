from pathlib import Path
import hashlib,json,re,shutil,subprocess
r=Path.cwd();b=r/'evaluation/engine073-launch-development-v19-repair';s=json.loads((b/'probe-summary.json').read_text());freeze=json.loads(Path('/private/tmp/engine073-source19-freeze.json').read_text());assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==s['source_sha']==freeze['source_sha']
assert hashlib.sha256((r/'crates/reasoning-harness-core/src/evidence_relevance.rs').read_bytes()).hexdigest()==s['source_file_sha256']==freeze['source_file_sha256'];assert not (b/'summary.json').exists()
for name in ['workspace','providers','fmt','archive','clippy','build']:
 shutil.copyfile('/private/tmp/engine073-successor19-'+name+'.log',b/(name+'.log'))
for src,dest in [('baseline-red.log','development-baseline-red.log'),('first-focused.log','development-first-repair-red.log'),('second-focused.log','development-repair-after.log'),('boundaries.log','focused-after.log'),('fixtures.json','fixtures.json')]:shutil.copyfile('/private/tmp/engine073-successor19-'+src,b/dest)
for src,dest in [('/private/tmp/engine073-source19-freeze.json','source-freeze.json'),('/private/tmp/engine073-verify-source19.py','verify-pinned-source.py'),('/private/tmp/engine073-finalize19-repair.py','finalize-development-bundle.py'),('/private/tmp/engine073-structural-reference-fix19.py','structural-reference-repair.py'),('/private/tmp/engine073-bound-reference-followup19.py','bound-reference-followup.py'),('/private/tmp/engine073-boundaries19.rs','new-boundaries.rs'),('/private/tmp/engine073-typed-compound-probe.json','root-compound-inputs.json'),('/private/tmp/engine073-unknown-owner-development-v2/inputs.json','parent-unknown-owner-inputs.json')]:shutil.copyfile(src,b/dest)
rows=[]
for name in ['workspace','providers']:
 t=(b/(name+'.log')).read_text();assert 'FAILED' not in t;rows+=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',t)
passed=sum(int(x[0]) for x in rows);ignored=sum(int(x[2]) for x in rows);assert passed==1547 and ignored==8,(passed,ignored)
assert '65 passed; 0 failed' in (b/'focused-after.log').read_text()
assert 'Finished' in (b/'clippy.log').read_text() and 'error:' not in (b/'clippy.log').read_text() and not (b/'fmt.log').read_bytes()
assert not Path('/private/tmp/engine073-successor19-fixtures.stderr').read_bytes();assert json.loads((b/'fixtures.json').read_text())['corpus']['stratification']['cases']==20
archive=json.loads((b/'archive.log').read_text());assert archive['archive_verified'] and archive['files_with_exact_sha256']==13
assert len(s['known_seen_regression'])==16 and all(x['passed']==60 for x in s['known_seen_regression'].values())
assert 'record_launch_development_trace_v17' not in (r/'crates/reasoning-harness-core/src/evidence_relevance.rs').read_text()
s.update(focused_tests=83,workspace={'passed':passed,'failed':0,'ignored':ignored},clippy='PASS',fmt='PASS',fixtures_cases=20,original_archive_files=13,root_compound_boundaries={'cases':4,'withheld':4},parent_unknown_owner_boundary={'cases':4,'withheld':4},cli_adoption='NO_GO',full_real_source_acceptance_complete=False)
(b/'summary.json').write_text(json.dumps(s,indent=2)+'\n')
(b/'README.md').write_text(f'''# Successor19 repair — development verification

Pinned source `{s['source_sha']}`; evidence_relevance.rs SHA256 `{s['source_file_sha256']}`. Package metadata remains0.7.2. No provider/model API calls.

Independentv19 firstFAIL49/60 at f3d9ad1 remains unchanged, including11 positive misses and negative30/30 withheld. The [initial surface triage](../engine073-launch-development-v19/README.md), [archive-only contrast diagnosis](../engine073-launch-research-diagnostics-v1/README.md) and [temporary actual-rule trace](../engine073-launch-trace-development-v1/README.md) remain separate preserved identities. This bundle is the subsequent source repair, not replacement first observations.

The trace showed typed-reference-not-excluded5, target-adverse-clause3 and unresolved-adverse-scope3. The repair binds property-specific references to adjacent complete property propositions or their exact named roles. Complete owner-qualified material propositions support previously unenumerated components without a new component-name list. Explicit named foreign-event references do not bind to unrelated generic typed properties; explicit role/owner conflicts still block. Coordinated-role objects remain together until an actual next predicate, including elided reporting verbs. Printed literal fields and complete nominal property definitions keep their outside bindings; actual target/global denials are independently checked.

The baseline replay reproduces all11 positive failures. The first repair RED also exposed old reporting-conjunction regressions in v13 and prior boundary groups; the following predicate repair restores them. All intermediate RED is retained. Five new development boundary groups cover custom-component property references, every coordinated role/owner flip, named foreign vs explicit roles, literal vs actual denials and property self-definitions vs event/global values. The temporary trace module/hooks were removed and exact base source restored before this semantic repair.

- Consumed v2–v10/v12–v16/v18/v19:960/960 pass.
- Related83 tests; locked workspace1547 PASS/0 FAIL/8 existing ignored.
- Original audit14/14; all597 complete historical v16/v29 outputs byte-identical.
- Strict Clippy/fmt,20 fixtures,13 immutable original archive files pass.
- Parent compound4/4 and unknown-owner4/4 withheld.

Fresh independentv20 is separate and pending. Normal source/core/serde/HEAD stay fixed while full checks use a separate target. The unchanged public probes and new output identity are recorded in `verify-pinned-source.py`. This development coverage is not arbitrary-language/ownership/coreference, property truth, Known/Supported, normal CLI/provider utility or real24 acceptance. Different-corpus rates are not measured improvement. No additional generic public claim graph/API or model calls are introduced by this repair.

Safe runnerPR541 is merged with bounded offline/independent checks; #540 live callback/dependency/numeric budgets remain open. #531 fresh real24=0/24, CLI adoption1/6 and Groq stopped remain unchanged. No Engine release or CLI adoption occurs here. Research-derived next steps and their status are documented in [the repair plan](../../docs/engine-073-research-informed-repair.md).
''')
h={str(p.relative_to(b)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(b.rglob('*')) if p.is_file() and p.name!='artifact-sha256.json'}
(b/'artifact-sha256.json').write_text(json.dumps({'source_sha':s['source_sha'],'source_file_sha256':s['source_file_sha256'],'files':h},indent=2)+'\n');print(json.dumps(s,indent=2))
