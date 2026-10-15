#!/usr/bin/env python3
"""Build identical released CLI/provider host with two immutable Engine source trees."""
import argparse, hashlib, io, json, os, shutil, subprocess, tarfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
HOST='ec6e55ff95b8364f9a5d09b8bbfe0ecc0c9d19d7'
ARMS={'0.6.1':'4abee90f0501f0d8fc09b453a0b45c942fbed69e','0.7.2':'296690a86e049b75ee693ae5032a71504366137c'}
def sha(data):return hashlib.sha256(data).hexdigest()
def run(args,cwd=None,env=None):return subprocess.run(args,cwd=cwd,env=env,check=True,capture_output=True).stdout
def extract(data,path):
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        for m in archive.getmembers():
            if m.name.startswith('/') or '..' in Path(m.name).parts or m.issym() or m.islnk():raise ValueError('unsafe source archive')
        archive.extractall(path)
def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--work-dir',type=Path,required=True);p.add_argument('--target-cache',type=Path);p.add_argument('--fetch-dependencies',action='store_true');a=p.parse_args()
    repo=a.repo.resolve();work=a.work_dir.resolve();work.mkdir();out=work/'bin';out.mkdir()
    for tag,pin in [('reason-v0.5.5',HOST)]+[(f'engine-v{v}',c) for v,c in ARMS.items()]:
        assert run(['git','rev-parse',tag+'^{}'],repo).decode().strip()==pin
    assert not run(['git','diff','engine-v0.6.1',HOST,'--','crates/reasoning-harness-core'],repo)
    archive=run(['git','archive',HOST],repo)
    frozen_probe=(HERE/'probe.rs').read_bytes();frozen_adviser=(HERE/'adviser.rs').read_bytes()
    proof={'host_tag':'reason-v0.5.5','host_commit':HOST,'probe_sha256':sha(frozen_probe),'adviser_sha256':sha(frozen_adviser),'build_script_sha256':sha(Path(__file__).read_bytes()),'rustc':run(['rustc','-vV']).decode(),'arms':{}}
    (work/'probe.rs').write_bytes(frozen_probe)
    for version,pin in ARMS.items():
        source=work/('source-'+version);source.mkdir();extract(archive,source)
        if version=='0.7.2':
            core=source/'crates/reasoning-harness-core';assert work in core.parents;shutil.rmtree(core)
            extract(run(['git','archive',pin,'crates/reasoning-harness-core'],repo),source)
            lock=source/'Cargo.lock';t=lock.read_text();old='name = "reasoning-harness-core"\nversion = "0.6.1"';assert t.count(old)==1
            lock.write_text(t.replace(old,'name = "reasoning-harness-core"\nversion = "0.7.2"'))
        manifest=source/'crates/reasoning-harness-cli/Cargo.toml'
        with manifest.open('a') as f:f.write('\n[[bin]]\nname = "reason-engine07-realworld-adviser"\npath = "src/bin/engine07_realworld_adviser.rs"\n')
        (source/'crates/reasoning-harness-cli/src/bin/engine07_realworld_adviser.rs').write_bytes(frozen_adviser)
        target=(a.target_cache/('target-'+version)).resolve() if a.target_cache else work/('target-'+version)
        env=os.environ.copy();env.update(CARGO_TARGET_DIR=str(target),CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0',REASON_HOME=str(work/'reason-home'))
        lock_sha=sha((source/'Cargo.lock').read_bytes())
        if a.fetch_dependencies:run(['cargo','fetch','--locked'],source,env)
        args=['cargo','build','--locked','--offline','-p','reasoning-harness-cli','--bin','reason','--jobs','2','--message-format=json']
        if version=='0.6.1':args+=['--bin','reason-engine07-realworld-adviser']
        print('building common CLI0.5.5 host / Engine'+version,flush=True)
        with (work/f'build-{version}.jsonl').open('xb') as stdout,(work/f'build-{version}.stderr').open('xb') as stderr:
            process=subprocess.run(args,cwd=source,env=env,stdout=stdout,stderr=stderr)
        process.check_returncode();assert sha((source/'Cargo.lock').read_bytes())==lock_sha
        artifacts={}
        for line in (work/f'build-{version}.jsonl').read_bytes().splitlines():
            m=json.loads(line)
            if m.get('reason')=='compiler-artifact':
                for name in m['filenames']:
                    if name.endswith('.rlib'):artifacts[m['target']['name']]=Path(name)
        core,serde=artifacts['reasoning_harness_core'],artifacts['serde_json']
        probe=out/('probe-'+version)
        args=['rustc','--edition=2024',str(work/'probe.rs'),'--extern','reasoning_harness_core='+str(core),'--extern','serde_json='+str(serde),'-L','dependency='+str(serde.parent),'-o',str(probe)]
        if version=='0.7.2':args+=['--cfg','engine07']
        run(args,source)
        binary=out/('reason-'+version);shutil.copyfile(target/'debug/reason',binary);binary.chmod(0o755)
        if version=='0.6.1':
            adviser=out/'adviser';shutil.copyfile(target/'debug/reason-engine07-realworld-adviser',adviser);adviser.chmod(0o755)
        doctor=subprocess.run([str(binary),'doctor','--format','json'],cwd=work,env=env,capture_output=True)
        public=json.loads(doctor.stdout)['result'];assert public['versions']=={'cli':'0.5.5','engine':version};assert not public['live_check']
        core_tree=run(['git','rev-parse',pin+':crates/reasoning-harness-core'],repo).decode().strip()
        main_original=run(['git','show',HOST+':crates/reasoning-harness-cli/src/main.rs'],repo)
        assert sha((source/'crates/reasoning-harness-cli/src/main.rs').read_bytes())==sha(main_original)
        proof['arms'][version]={'engine_commit':pin,'engine_tree':core_tree,'engine_manifest_sha256':sha((source/'crates/reasoning-harness-core/Cargo.toml').read_bytes()),'engine_relevance_sha256':sha((source/'crates/reasoning-harness-core/src/evidence_relevance.rs').read_bytes()),'host_lock_sha256':lock_sha,'cli_main_sha256':sha(main_original),'versions':public['versions'],'doctor_live_check':False,'binary_sha256':sha(binary.read_bytes()),'probe_binary_sha256':sha(probe.read_bytes())}
        print('built and version verified Engine'+version,flush=True)
    proof['adviser_binary_sha256']=sha((out/'adviser').read_bytes())
    (out/'identity.json').write_text(json.dumps(proof,indent=2)+'\n')
if __name__=='__main__':main()
