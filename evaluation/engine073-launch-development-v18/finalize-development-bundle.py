from pathlib import Path
import subprocess,json,hashlib,shutil,re
r=Path.cwd();b=r/'evaluation/engine073-launch-development-v18';s=json.loads((b/'probe-summary.json').read_text());freeze=json.loads(Path('/private/tmp/engine073-source18-freeze.json').read_text())
assert subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==s['source_sha']==freeze['source_sha']
assert hashlib.sha256((r/'crates/reasoning-harness-core/src/evidence_relevance.rs').read_bytes()).hexdigest()==s['source_file_sha256']==freeze['source_file_sha256']
assert not (b/'summary.json').exists()
for name in ['workspace','providers','fmt','archive','clippy','build']:
 shutil.copyfile('/private/tmp/engine073-successor18-'+name+'.log',b/(name+'.log'))
for src,dest in [('first-focused.log','development-initial-red.log'),('attribute-focused.log','development-attribute-red.log'),('helper-diagnostics.log','development-helper-diagnostics.log'),('followup-focused.log','development-followup-red.log'),('literal-focused.log','development-literal-after.log'),('final-focused.log','focused-after.log'),('fixtures.json','fixtures.json')]:
 shutil.copyfile('/private/tmp/engine073-successor18-'+src,b/dest)
for src,dest in [('/private/tmp/engine073-typed-compound-probe.json','root-compound-inputs.json'),('/private/tmp/engine073-source18-freeze.json','source-freeze.json'),('/private/tmp/engine073-verify-source18.py','verify-pinned-source.py'),('/private/tmp/engine073-development-helper-diagnostics18.json','development-helper-inputs.json')]:
 shutil.copyfile(src,b/dest)
shutil.copyfile('/private/tmp/engine073-unknown-owner-development-v2/inputs.json',b/'parent-unknown-owner-inputs.json')
for p in ['/private/tmp/engine073-reporting-modifier-fix.py','/private/tmp/engine073-attribute-clause-fix.py','/private/tmp/engine073-attribute-followup-fix.py','/private/tmp/engine073-literal-object-fix.py','/private/tmp/engine073-attribute-boundary-finalize.py']:
 shutil.copyfile(p,b/Path(p).name)
rows=[]
for name in ['workspace','providers']:
 t=(b/(name+'.log')).read_text();rows+=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',t);assert 'FAILED' not in t
passed=sum(int(x[0]) for x in rows);ignored=sum(int(x[2]) for x in rows);assert passed==1541 and ignored==8,(passed,ignored)
assert 'Finished' in (b/'clippy.log').read_text() and 'error:' not in (b/'clippy.log').read_text() and not (b/'fmt.log').read_bytes()
assert not Path('/private/tmp/engine073-successor18-fixtures.stderr').read_bytes()
fixtures=json.loads((b/'fixtures.json').read_text()); assert fixtures['corpus']['stratification']['cases']==20
archive=json.loads((b/'archive.log').read_text()); assert archive['archive_verified'] and archive['files_with_exact_sha256']==13
shutil.copyfile('/private/tmp/engine073-finalize18.py', b/'finalize-development-bundle.py')
assert len(s['known_seen_regression'])==15 and all(x['passed']==60 for x in s['known_seen_regression'].values())
assert '59 passed; 0 failed' in (b/'focused-after.log').read_text()
assert not re.search(r'engine073_private_attribute_diagnostics|eprintln!\("ATTR|eprintln!\("scope', (r/'crates/reasoning-harness-core/src/evidence_relevance.rs').read_text())
s.update(focused_tests=77,workspace={'passed':passed,'failed':0,'ignored':ignored},clippy='PASS',fmt='PASS',fixtures_cases=20,original_archive_files=13,root_compound_boundaries={'cases':4,'withheld':4},parent_unknown_owner_boundary={'cases':4,'withheld':4},cli_adoption='NO_GO',full_real_source_acceptance_complete=False)
(b/'summary.json').write_text(json.dumps(s,indent=2)+'\n')
(b/'README.md').write_text(f'''# Engine 0.7.3 successor18 — development verification

Pinned source `{s['source_sha']}`, evidence_relevance.rs SHA256 `{s['source_file_sha256']}`. Package metadata remains0.7.2 until a separate release promotion. Provider/model API calls0.

Independent v18 first FAIL45/60 is immutable: one unsafe negative li18-038 and14 positive misses at source258667b. No labels, raw output, scorer or initial observation are replaced. This repair consumes that corpus as development coverage. Fresh independent v19 is separate and is not counted here.

The unsafe case combined a typed property denial with a positive reporting shell containing an inner target nonexistence denial. The adverse-clause grammar now preserves the inner scope across explicit reporting modifiers and nested reporting. Non-launch assertions and fully declared multi-role/compound objects retain their own types. Mixed property/foreign corrections check every object independently using original owner spelling. Parentheses and active correction clauses retain distinct adverse scopes. A printed quoted sentence is literal lettering only with a complete outside property/target binding; a target event proposition cannot be retyped by an apposition. No unknown owner or global/target event denial is discarded. A trusted target's literal `Both` cannot be stripped to manufacture a foreign owner.

Intermediate initial RED, attribute RED (including one seen-v12 regression), follow-up RED and temporary private-helper diagnostic output are saved. The private diagnostic module was removed from final production/test source; it is not fresh independent acceptance. Final boundary groups cover nested reporting, literal field/event separation, mixed/compound/parenthetical scopes and target-name binding.

- Seen v2–v10/v12–v16/v18:900/900 pass.
- Original audit14/14; all597 complete historical API v16/v29 outputs byte-identical.
- Related77 tests; locked workspace1541 PASS/0 FAIL/8 existing ignored.
- Strict Clippy/fmt,20 fixtures and13 original archive files pass.
- Parent compound4/4 and unknown-owner4/4 withheld.

All prior first observations, including v11/v17 NOT_EVALUATED, remain unchanged. Different-corpus scores do not measure improvement. Bounded grammar success does not establish arbitrary-language/coreference, factual truth, Known/Supported promotion or normal CLI/provider utility. Full tests use a separate target while normal source/core/serde/HEAD stay fixed for independent evaluation. Unchanged probes and create-new outputs are pinned in `verify-pinned-source.py`; a new execution needs a new identity.

Safe runnerPR541 is merged after final CI5/5,47 offline controls and six bounded independent re-review boundaries. #540 live callback/dependency/numeric budget acceptance remains open. Paired CLI evidence uses olderdevelopment5edd. #531 fresh real24/provider/human acceptance0/24, adoption1/6 and Groq stop remain unchanged. No Engine release or CLI adoption occurs here.
''')
hashes={str(p.relative_to(b)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(b.rglob('*')) if p.is_file() and p.name!='artifact-sha256.json'}
(b/'artifact-sha256.json').write_text(json.dumps({'source_sha':s['source_sha'],'source_file_sha256':s['source_file_sha256'],'files':hashes},indent=2)+'\n')
print(json.dumps(s,indent=2))
