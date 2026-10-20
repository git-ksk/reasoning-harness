#!/usr/bin/env python3
"""Offline engineering controls; no quality labels, scorer or provider execution."""
from pathlib import Path
import copy
import hashlib
import json
import subprocess
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parent
REPO = Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
BASE = Path('/private/tmp/engine073-active-source-host-baseline')
BASE_SHA = '4abee90f0501f0d8fc09b453a0b45c942fbed69e'
SOURCE_SHA = 'ab3c6322921c8c738ec35bd018345917b5afa538'
SOURCE_HASH = '2e9207177d9f1e91b8c031322dbbf83c2b374d8847e123e135dcca727b0214de'

def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def write(p, v):
    with p.open('x') as f: json.dump(v, f, indent=2, ensure_ascii=False); f.write('\n')

def execute(name, argv, cwd=ROOT):
    p = subprocess.run(argv, cwd=cwd, capture_output=True)
    (ROOT/(name+'.stdout')).write_bytes(p.stdout)
    (ROOT/(name+'.stderr')).write_bytes(p.stderr)
    write(ROOT/(name+'.invocation.json'), {'argv':list(map(str,argv)), 'cwd':str(cwd), 'exit_code':p.returncode,
          'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(), 'stderr_sha256':hashlib.sha256(p.stderr).hexdigest()})
    return p

input_path = ROOT/'input.json'
data = json.loads(input_path.read_text())
source = Path(data['source']['normalized_path'])
raw = source.with_suffix('.html')
host = ROOT/'host.rs'
prod = REPO/'crates/reasoning-harness-core/src/evidence_relevance.rs'
before = {str(p):sha(p) for p in [input_path,source,raw,host,prod]}
assert before[str(source)] == '8252ff58d1701eed6b80a20a183eef855dfde7b1b3b11bd9ffb2ec2c5125a4dc'
assert before[str(raw)] == '23f92fc924708065b4998712ee080dda049e9fd2b4e16558b396d68694d6fa47'
assert before[str(prod)] == SOURCE_HASH
old = json.loads((REPO/'evaluation/engine073-launch-development-v27-bounded-properties/identity-before.json').read_text())
for key in ('core','serde'): assert sha(old[key]['path']) == old[key]['sha256']
build_messages = [json.loads(l) for l in (ROOT/'baseline-build.jsonl').read_text().splitlines()]
assert build_messages[-1] == {'reason':'build-finished','success':True}
def lib(name):
    found = [f for x in build_messages if x.get('reason')=='compiler-artifact' and x['target']['name']==name for f in x['filenames'] if f.endswith('.rlib')]
    assert len(found)==1,(name,found)
    return found[0]
libraries = {
 'baseline':{'core':lib('reasoning_harness_core'),'serde':lib('serde_json'),'commit':BASE_SHA,
             'source_sha256':sha(BASE/'crates/reasoning-harness-core/src/evidence_relevance.rs')},
 'candidate':{'core':old['core']['path'],'serde':old['serde']['path'],'commit':SOURCE_SHA,'source_sha256':SOURCE_HASH}
}
write(ROOT/'identity-before.json',{'utc':datetime.now(timezone.utc).isoformat(),'input_hashes':before,
 'libraries':{arm:{**v,'core_sha256':sha(v['core']),'serde_sha256':sha(v['serde'])} for arm,v in libraries.items()},
 'baseline_build':{'argv':['cargo','build','--locked','--offline','-p','reasoning-harness-core','--lib','--message-format=json-render-diagnostics'],
                   'cwd':str(BASE),'exit_code':0,'stdout_sha256':sha(ROOT/'baseline-build.jsonl'),'stderr_sha256':sha(ROOT/'baseline-build.stderr')},
 'rustc_version':subprocess.check_output(['rustc','--version','--verbose'],text=True)})
outputs={}
for arm, libs in libraries.items():
    binary = ROOT/('host-'+arm)
    cmd = ['rustc','--edition=2024',str(host),'-L','dependency='+str(Path(libs['serde']).parent),
           '--extern','reasoning_harness_core='+libs['core'],'--extern','serde_json='+libs['serde'],'-o',str(binary)]
    compiled = execute(arm+'-compile',cmd)
    assert compiled.returncode==0,compiled.stderr.decode()
    p = execute(arm+'-node-control',[str(binary),str(input_path)])
    assert p.returncode==0,p.stderr.decode()
    outputs[arm]=json.loads(p.stdout)
    negative = copy.deepcopy(data)
    negative['run_id']+='-quote-mismatch'
    negative['source']['quote']+=' CHANGED'
    bad_path = ROOT/(arm+'-mismatch-input.json')
    write(bad_path,negative)
    bad = execute(arm+'-quote-mismatch',[str(binary),str(bad_path)])
    assert bad.returncode!=0 and bad.stdout==b'' and b'quote differs from retained full source' in bad.stderr
for path, expected in before.items(): assert sha(path)==expected,('input changed',path)
for libs in libraries.values():
    assert Path(libs['core']).is_file() and Path(libs['serde']).is_file()
write(ROOT/'identity-after.json',{'utc':datetime.now(timezone.utc).isoformat(),'input_hashes':{p:sha(p) for p in before},
 'binaries':{arm:{'sha256':sha(ROOT/('host-'+arm))} for arm in libraries},
 'library_hashes':{arm:{'core_sha256':sha(v['core']),'serde_sha256':sha(v['serde'])} for arm,v in libraries.items()}})
print(json.dumps({'arms':{arm:{'engine_version':v['engine_version'],'finalization':v['finalization'],
 'direct_host_api_calls':v['direct_host_api_calls'],'truth_claims':v['truth_claims'],'verification_receipts':v['verification_receipts']} for arm,v in outputs.items()},'model_provider_calls':0}))
