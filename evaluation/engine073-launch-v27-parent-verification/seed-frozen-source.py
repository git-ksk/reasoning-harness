from pathlib import Path
import json,hashlib,subprocess,shutil,os
r=Path.cwd();f=Path('/Users/sawadakousuke/.codex/worktrees/engine073-source27-freeze/reasoning-harness');pin=json.loads(Path('/private/tmp/engine073-source28-root-pin.json').read_text());sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=f,text=True).strip()==pin['code_commit'];assert sha(f/'crates/reasoning-harness-core/src/evidence_relevance.rs')==pin['source_sha256']
d=r/'evaluation/engine073-launch-development-v26-local-references';summary=json.loads((d/'summary.json').read_text());assert summary['known_with_v26_and_supplemental']==1400 and summary['development_inputs']==2011 and summary['v26']['joint_passed']==60
runtime=Path('/private/tmp/engine073-independent-v27-runtime');runtime.mkdir();exe=Path('/private/tmp/engine073-source28-actual-26c98ad-seen');shutil.copyfile(exe,runtime/'probe');shutil.copyfile(r/'evaluation/engine073-launch-independent-v26/probe.rs',runtime/'probe.rs');shutil.copyfile('/private/tmp/engine073-source28-actual-build.jsonl',runtime/'cargo-build.jsonl')
files={str(p.relative_to(r)):sha(p) for p in sorted(d.iterdir()) if p.is_file()};assert len(files)==30
for rel,h in files.items():
 p=f/rel;p.parent.mkdir(parents=True,exist_ok=True);assert not p.exists();shutil.copyfile(r/rel,p);assert sha(p)==h;os.chmod(p,0o444)
receipt={'source_commit':pin['code_commit'],'source_file_sha256':pin['source_sha256'],'normal_regression_summary_sha256':sha(d/'summary.json'),'known_controls':1400,'original_audit_separate':14,'historical_whole_output_byte_equal':597,'known_scope':'Tested #520 source only: all known1400 and original14 pass,historical597 whole bytes equal; known_unfixed_critical_defects[] only in this tested scope. Not generalization,real24 or project acceptance.','Engine_processes':27,'inputs':2011,'runtime_probe':{'sha256':sha(runtime/'probe'),'probe_source_sha256':sha(runtime/'probe.rs'),'cargo_metadata_sha256':sha(runtime/'cargo-build.jsonl')},'normal_identity':summary['identity'],'files':files,'scope':'Frozen source-only normal controls captured before later root full-validation packaging. Copied bodies are hash-only for independent author.','new_v27_authorship_or_engine_runs':0,'full_validation_prerequisite':'Pending root final2 full suite. Strict workspace/all-target Clippy/fmt pass at the same source. First sandbox attempt stopped at5 existing localhost binds; source unchanged,corrected environment running. No authoring until parent full completion,M0 review and explicit next notice.'}
parent=f/'evaluation/engine073-launch-independent-v27-parent-controls';parent.mkdir();(parent/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');shutil.copyfile(parent/'receipt.json',runtime/'identity.json')
audit=r/'evaluation/engine073-launch-v27-parent-verification/frozen-parent-reference-snapshot';audit.mkdir(parents=True);shutil.copyfile(parent/'receipt.json',audit/'receipt.json')
for rel,h in files.items():
 p=audit/'files'/rel;p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(r/rel,p);assert sha(p)==h
for p in runtime.iterdir():
 with p.open('rb') as handle:os.fsync(handle.fileno())
 os.chmod(p,0o555 if p.name=='probe' else 0o444)
with (parent/'receipt.json').open('rb') as handle:os.fsync(handle.fileno())
os.chmod(parent/'receipt.json',0o444)
print(json.dumps({'candidate':pin,'runtime':str(runtime/'probe'),'runtime_sha256':sha(runtime/'probe'),'receipt_sha256':sha(parent/'receipt.json'),'parent_reference_files':len(files),'parent_receipt':str(parent/'receipt.json'),'old_corpus_references':829}))
