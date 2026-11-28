"""Rebuild the exact measured 0.6.1 composition and its three-file successor.
No model access; old runs and artifacts remain immutable."""
import argparse
import importlib.util
import hashlib
import json
import os
import platform
from pathlib import Path
import shutil
import subprocess

CLI = "7cf783d2169cd7882832fd780070ab7280ad63c0"
ARMS = {"baseline":"7401d7c761f6d48c27fc27f9998307dd899c73a7",
        "candidate":"6ba4a836a728fafeaf124aa4f7b37b02c14cda14"}
CHANGED = {"crates/reasoning-harness-cli/src/main.rs",
           "crates/reasoning-harness-core/src/generation.rs",
           "crates/reasoning-harness-core/src/investigation.rs"}
HERE = Path(__file__).resolve().parent
COMMON_PATCH_SHA = '02a08c73ea25e09ff5355ee4e5ee4cab991f01da71e36c0878e61957e32f3f24'
COMMON_PATCH = HERE/"google-accounted-503.patch"

def build(repo, output, compile_binaries):
    repo, output = Path(repo).resolve(), Path(output).resolve()
    spec = importlib.util.spec_from_file_location("frozen_pair_builder",
        HERE.parent/"engine073-531-independent24-20261116/build_completeness_pair.py")
    f=importlib.util.module_from_spec(spec);spec.loader.exec_module(f)
    original=f.original
    for pin in [CLI,*ARMS.values()]: f.require_commit(repo,pin)
    original.require(hashlib.sha256(COMMON_PATCH.read_bytes()).hexdigest()==COMMON_PATCH_SHA,"common_patch_pin")
    original.private_new_directory(output)
    if compile_binaries: original.private_new_directory(output/"bin")
    proof={"runner_os":platform.system(),"runner_arch":platform.machine(),"cli_commit":CLI,"built":compile_binaries,"provider_generation_authorized":False,
           "source_identical_except_declared_changes":False,"changed_files":sorted(CHANGED),"common_patch_sha256":COMMON_PATCH_SHA,"arms":{}}
    for arm,pin in ARMS.items():
        source=output/("source-"+arm);original.private_new_directory(source)
        original.export(repo,pin,(*original.SOURCE_FILES,*original.SOURCE_DIRS),source)
        subprocess.run(["git","apply","--check",str(COMMON_PATCH)],cwd=source,check=True)
        subprocess.run(["git","apply",str(COMMON_PATCH)],cwd=source,check=True)
        inventory={n:f.sha((source/n).read_bytes()) for n in original.source_inventory(source)}
        if compile_binaries:
            env={k:v for k,v in os.environ.items() if not k.endswith("_API_KEY")}
            env.update(CARGO_TARGET_DIR=os.environ.get("R2_LOCAL_BUILD_CACHE",str(output/"target")),
                       CARGO_INCREMENTAL="0",CARGO_PROFILE_DEV_DEBUG="0")
            with (output/(arm+"-build.log")).open("xb") as log:
                subprocess.run(["cargo","build","--locked","--offline","-p","reasoning-harness-cli","--bin","reason","--jobs","2"],
                    cwd=source,env=env,stdout=log,stderr=subprocess.STDOUT,check=True)
            dst=output/"bin"/("reason-"+arm)
            shutil.copyfile(Path(env["CARGO_TARGET_DIR"])/"debug/reason",dst);dst.chmod(0o700)
        proof["arms"][arm]={"engine_commit":pin,"source_file_sha256":inventory,
            "binary":str(dst) if compile_binaries else None,"binary_sha256":f.sha(dst.read_bytes()) if compile_binaries else None}
    a,b=(proof["arms"][arm]["source_file_sha256"] for arm in ARMS)
    original.require(a.keys()==b.keys() and {k for k in a if a[k]!=b[k]}==CHANGED,"runtime_scope")
    proof["source_identical_except_declared_changes"]=True
    p=output/"pair-preflight.json";p.write_text(json.dumps(proof,sort_keys=True)+"\n");p.chmod(0o600)
    return {"built":compile_binaries,"changed_files":sorted(CHANGED),"provider_calls":0}

if __name__=="__main__":
    p=argparse.ArgumentParser();p.add_argument("--repo",type=Path,required=True);p.add_argument("--output",type=Path,required=True);p.add_argument("--mode",choices=("validate-only","build"),default="validate-only")
    a=p.parse_args();print(json.dumps(build(a.repo,a.output,a.mode=="build"),sort_keys=True))
