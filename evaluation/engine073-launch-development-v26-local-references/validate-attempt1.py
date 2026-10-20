from pathlib import Path
import subprocess,os,json,hashlib
r=Path.cwd();env=dict(os.environ,CARGO_TARGET_DIR='/private/tmp/engine073-validation-target',CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',CARGO_PROFILE_TEST_DEBUG='0',PYTHONDONTWRITEBYTECODE='1')
pin=json.loads(Path('/private/tmp/engine073-source28-root-pin.json').read_text())
def verify_pin():
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip()==pin['code_commit']
 assert hashlib.sha256((r/'crates/reasoning-harness-core/src/evidence_relevance.rs').read_bytes()).hexdigest()==pin['source_sha256']
verify_pin()
commands=[('clippy',['cargo','clippy','--locked','--offline','--workspace','--all-targets','--jobs','2','--','-D','warnings']),('fmt',['cargo','fmt','--all','--check']),('workspace',['cargo','test','--locked','--offline','--workspace','--exclude','reasoning-harness-providers','--jobs','2']),('providers',['cargo','test','--locked','--offline','-p','reasoning-harness-providers','--jobs','2','--','--test-threads=1']),('fixtures',['cargo','run','-q','--locked','--offline','-p','reasoning-harness-cli','--','eval','fixtures','--format','json']),('archive',['python3','scripts/verify_engine_0_7_independent_v1_archive.py']),('quality',['python3','-m','unittest','discover','-s','evaluation/engine07-quality-gates-v1','-v']),('source-policy',['python3','-m','unittest','discover','-s','evaluation/engine073-source-gate-v1','-v'])]
for name,cmd in commands:
 verify_pin()
 p=Path('/private/tmp')/f'engine073-source28-final1-verified-{name}.log'
 with p.open('x') as f: res=subprocess.run(cmd,cwd=r,env=env,stdout=f,stderr=subprocess.STDOUT)
 print(name,res.returncode,flush=True)
 if res.returncode:print(p.read_text()[-6000:],flush=True);raise SystemExit(res.returncode)

verify_pin()
