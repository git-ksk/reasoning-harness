#!/usr/bin/env python3
"""Export patched paired cores with one CLI, recovery patch and audit bounds."""
import argparse, hashlib, io, json, os, re, shutil, subprocess, tarfile
from pathlib import Path

HERE=Path(__file__).resolve().parent
HOST='73bbd18173b4dfcd6664a26241de0385f2eb2e3e'
ARMS={'baseline':'4abee90f0501f0d8fc09b453a0b45c942fbed69e','candidate':'ab3c6322921c8c738ec35bd018345917b5afa538'}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def run(args,cwd=None,env=None):return subprocess.check_output(args,cwd=cwd,env=env)
def extract(data,path):
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        for m in archive.getmembers():
            if m.name.startswith('/') or '..' in Path(m.name).parts or not (m.isfile() or m.isdir()):raise ValueError('unsafe source archive')
        archive.extractall(path)
def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--work-dir',type=Path,required=True);p.add_argument('--fetch-dependencies',action='store_true');a=p.parse_args()
    repo=a.repo.resolve();work=a.work_dir.resolve();work.mkdir();bins=work/'bin';bins.mkdir()
    archive=run(['git','archive',HOST],repo)
    proof={'host_commit':HOST,'scope':'same CLI/provider host and PR552 recovery opt-in patch on both historical core trees; not untouched distribution binaries','rustc':run(['rustc','-vV']).decode(),'build_script_sha256':sha(__file__),'audit_module_sha256':sha(HERE/'audit.rs'),'registry_source_sha256':sha(HERE/'registry.rs'),'arms':{}}
    for arm,pin in ARMS.items():
        source=work/('source-'+arm);source.mkdir();extract(archive,source)
        shutil.rmtree(source/'crates/reasoning-harness-core')
        extract(run(['git','archive',pin,'crates/reasoning-harness-core'],repo),source)
        core_root=source/'crates/reasoning-harness-core'
        original_core={str(p.relative_to(core_root)):sha(p) for p in core_root.rglob('*') if p.is_file()}
        original_resolution=sha(source/'crates/reasoning-harness-core/src/resolution.rs')
        common_patch=run(['git','diff','2f6f88fd1a8b721ef11bcda4b7a1fb00e5691886','2c4fea29ac9ae7c0447766cd0d83200ae01ea9f1','--','crates/reasoning-harness-core/src/resolution.rs'],repo)
        subprocess.run(['patch','-p1','--dry-run'],cwd=source,input=common_patch,check=True)
        subprocess.run(['patch','-p1'],cwd=source,input=common_patch,check=True)
        patched_resolution=sha(source/'crates/reasoning-harness-core/src/resolution.rs')
        assert patched_resolution != original_resolution, 'common patch did not apply'
        assert 'pub fn run_with_pending_investigation' in (source/'crates/reasoning-harness-core/src/resolution.rs').read_text()
        patched_core={str(p.relative_to(core_root)):sha(p) for p in core_root.rglob('*') if p.is_file()}
        assert original_core.keys()==patched_core.keys()
        assert [p for p in original_core if original_core[p]!=patched_core[p]]==['src/resolution.rs']
        lock=source/'Cargo.lock';s=lock.read_text()
        original_version=re.search(r'^version = "([^"]+)"',run(['git','show',HOST+':crates/reasoning-harness-core/Cargo.toml'],repo).decode(),re.M).group(1)
        arm_version=re.search(r'^version = "([^"]+)"',(source/'crates/reasoning-harness-core/Cargo.toml').read_text(),re.M).group(1)
        old='name = "reasoning-harness-core"\nversion = "'+original_version+'"'
        assert s.count(old)==1
        lock.write_text(s.replace(old,'name = "reasoning-harness-core"\nversion = "'+arm_version+'"'))
        cli=source/'crates/reasoning-harness-cli';providers=source/'crates/reasoning-harness-providers';src=providers/'src'
        with (cli/'Cargo.toml').open('a') as f:f.write('\n[[bin]]\nname = "engine073-untrusted-adviser"\npath = "src/bin/engine073_untrusted_adviser.rs"\n')
        with (cli/'Cargo.toml').open('a') as f:f.write('\n[[bin]]\nname = "engine073-audit-probe"\npath = "src/bin/engine073_audit_probe.rs"\n')
        shutil.copyfile(HERE/'adviser.rs',cli/'src/bin/engine073_untrusted_adviser.rs')
        shutil.copyfile(HERE/'audit_probe.rs',cli/'src/bin/engine073_audit_probe.rs')
        shutil.copyfile(HERE/'audit.rs',src/'evaluation_audit.rs')
        with (src/'lib.rs').open('a') as f:f.write('\nmod evaluation_audit;\n')
        patched={}
        for file,provider in [('mistral.rs','mistral'),('gemma.rs','google'),('groq.rs','groq')]:
            path=src/file;before=path.read_text();old='Box::pin(self.generate_inner(request))';assert before.count(old)==1
            new='''Box::pin(async move {
                let id = crate::evaluation_audit::before("PROVIDER", &self.model, &request)?;
                let result = self.generate_inner(request).await;
                crate::evaluation_audit::after(id, &result)?;
                result
            })'''.replace('PROVIDER',provider)
            after=before.replace(old,new)
            if provider=='groq':
                marker='impl GroqAdapter {'
                assert after.count(marker)==1
                method='''
    /// Evaluation-only offline fixture; real task construction is unchanged.
    pub fn evaluation_loopback(api_key: &str, model: &str, base: &str) -> Result<Self, ModelError> {
        assert!(base.starts_with("http://127.0.0.1:"));
        Self::with_base_url_timeout_and_pacing(api_key, model, base, DEFAULT_TIMEOUT, Duration::ZERO, None)
    }
'''
                after=after.replace(marker,marker+method)
            marker='            let send = self'
            assert after.count(marker)==1
            after=after.replace(marker,f'            crate::evaluation_audit::wire("{provider}", &body)?;\n'+marker)
            marker='let status = response.status();'
            after=after.replace(marker,marker+f'\n            crate::evaluation_audit::wire_response("{provider}", status.as_u16(), response.headers())?;',1)
            # No retry can start after a failed HTTP attempt. Existing CLI-level
            # structured compatibility fallbacks remain bounded by audit24.
            after=re.sub(r'(const MAX_RATE_LIMIT_RETRIES: usize = )\d+;',r'\g<1>0;',after)
            after=re.sub(r'(const MAX_PROVIDER_ATTEMPTS: u32 = )\d+;',r'\g<1>1;',after)
            after=re.sub(r'(const MAX_STRUCTURED_OUTPUT_RETRIES: usize = )\d+;',r'\g<1>0;',after)
            # Model-start pacing is owned by the identical audit wrapper. Avoid
            # success-header sleep being mistaken for a failed paid generation.
            if provider=='mistral':after=after.replace('const MAX_SUCCESS_PACING_DELAY: Duration = Duration::from_secs(90);','const MAX_SUCCESS_PACING_DELAY: Duration = Duration::ZERO;')
            # Redirect only inert fixture-model controls through numeric loopback.
            # Missing override follows the unchanged live constructor path.
            if provider in ['mistral','google']:
                constructor='    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self, ModelError> {\n'
                assert after.count(constructor)==1
                hook=constructor+'        let api_key=api_key.into(); let model=model.into();\n        if let Some(base)=crate::evaluation_audit::loopback_base(&api_key,&model)? { return Self::with_base_url(api_key,model,&base); }\n'
                after=after.replace(constructor,hook)
            else:
                start=after.index('    pub fn from_api_key_and_env(');end=after.index('    pub fn new(',start)
                part=after[start:end]
                marker='        let min_request_interval = '
                assert part.count(marker)==1 and part.count('            DEFAULT_BASE_URL,')==1
                part=part.replace(marker,'        let api_key=api_key.into(); let model=model.into();\n        let engineering_base=crate::evaluation_audit::loopback_base(&api_key,&model)?;\n'+marker)
                part=part.replace('            DEFAULT_BASE_URL,','            engineering_base.as_deref().unwrap_or(DEFAULT_BASE_URL),')
                after=after[:start]+part+after[end:]
            path.write_text(after);patched[file]={'original_sha256':hashlib.sha256(before.encode()).hexdigest(),'patched_sha256':sha(path)}
        target=work/('target-'+arm);env=os.environ.copy();env.update(CARGO_TARGET_DIR=str(target),CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0')
        if a.fetch_dependencies:subprocess.run(['cargo','fetch','--locked'],cwd=source,env=env,check=True)
        cmd=['cargo','build','--locked','--offline','-p','reasoning-harness-cli','--bin','reason','--bin','engine073-untrusted-adviser','--bin','engine073-audit-probe','--jobs','2','--message-format=json-render-diagnostics']
        with (work/(arm+'-build.jsonl')).open('xb') as out,(work/(arm+'-build.stderr')).open('xb') as err:r=subprocess.run(cmd,cwd=source,env=env,stdout=out,stderr=err)
        assert r.returncode==0,(arm,work/(arm+'-build.stderr'))
        artifacts={}
        for line in (work/(arm+'-build.jsonl')).read_bytes().splitlines():
            msg=json.loads(line)
            if msg.get('reason')=='compiler-artifact':
                for f in msg['filenames']:
                    if f.endswith('.rlib'):artifacts[msg['target']['name']]=Path(f)
        core,serde=artifacts['reasoning_harness_core'],artifacts['serde_json'];probe=bins/('source-host-'+arm)
        cmd=['rustc','--edition=2024',str(HERE/'source_host.rs'),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-L','dependency='+str(serde.parent),'-o',str(probe)]
        if arm=='candidate':cmd+=['--cfg','engine07']
        compile_result=subprocess.run(cmd,cwd=source,capture_output=True)
        (work/(arm+'-source-host-compile.stderr')).write_bytes(compile_result.stderr);assert compile_result.returncode==0
        source_host_cmd=cmd
        registry_cmd=['rustc','--edition=2024',str(HERE/'registry.rs'),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'--extern','sha2='+str(artifacts['sha2']),'-L','dependency='+str(serde.parent),'-o',str(bins/('registry-'+arm))]
        compiled_registry=subprocess.run(registry_cmd,cwd=source,capture_output=True)
        (work/(arm+'-registry-compile.stderr')).write_bytes(compiled_registry.stderr);assert compiled_registry.returncode==0
        shutil.copyfile(target/'debug/reason',bins/('reason-'+arm));(bins/('reason-'+arm)).chmod(0o755)
        shutil.copyfile(target/'debug/engine073-untrusted-adviser',bins/('adviser-'+arm));(bins/('adviser-'+arm)).chmod(0o755)
        shutil.copyfile(target/'debug/engine073-audit-probe',bins/('audit-probe-'+arm));(bins/('audit-probe-'+arm)).chmod(0o755)
        files={str(p.relative_to(source)):sha(p) for p in sorted(source.rglob('*')) if p.is_file()}
        # target is a sibling, so the source inventory contains no build outputs.
        original_main=run(['git','show',HOST+':crates/reasoning-harness-cli/src/main.rs'],repo)
        assert sha(cli/'src/main.rs')==hashlib.sha256(original_main).hexdigest()
        proof['arms'][arm]={'engine_commit':pin,'common_resolution_patch_sha256':hashlib.sha256(common_patch).hexdigest(),'original_resolution_sha256':original_resolution,'patched_resolution_sha256':patched_resolution,'core_tree':run(['git','rev-parse',pin+':crates/reasoning-harness-core'],repo).decode().strip(),
            'relevance_sha256':sha(source/'crates/reasoning-harness-core/src/evidence_relevance.rs'),'cli_main_sha256':sha(cli/'src/main.rs'),'cli_package_version':re.search(r'^version = "([^"]+)"',(cli/'Cargo.toml').read_text(),re.M).group(1),
            'provider_patch':patched,'source_files':files,'original_core_files':original_core,'patched_core_files':patched_core,'binaries':{n:sha(bins/(n+'-'+arm)) for n in ['reason','adviser','source-host','audit-probe','registry']},
            'source_host_compile_argv':source_host_cmd,'registry_compile_argv':registry_cmd,'build_profile':'dev debug0 incremental0 normal non-test'}
        print('built',arm,flush=True)
    assert proof['arms']['baseline']['provider_patch']==proof['arms']['candidate']['provider_patch']
    (bins/'identity.json').write_text(json.dumps(proof,indent=2)+'\n')
if __name__=='__main__':main()
