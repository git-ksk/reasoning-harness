#!/usr/bin/env python3
"""Exactly one separate API-contract sentinel; not a holdout case or provider call."""
import hashlib, json, os, subprocess, time
from datetime import datetime, timezone
from pathlib import Path
ROOT = Path(__file__).resolve().parent

def sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def utc(): return datetime.now(timezone.utc).isoformat()
def write_new(name, data):
    with (ROOT/name).open('x') as f:
        json.dump(data,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def main():
    manifest=json.loads((ROOT/'manifest.json').read_text())
    for name, expected in manifest['frozen_files'].items():
        if sha(ROOT/name)!=expected: raise RuntimeError('Frozen file mismatch: '+name)
    repo=ROOT.parent.parent
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
    if head!=manifest['candidate']['source_sha']:raise RuntimeError('Candidate HEAD mismatch')
    checked={}
    for name,artifact in manifest['checked_artifacts'].items():
        observed=sha(artifact['path'])
        if observed!=artifact['sha256']:raise RuntimeError('Artifact mismatch: '+name)
        checked[name]={'path':artifact['path'],'sha256':observed}
    # No source text is emitted or inspected; only its bytes are hashed.
    guard={'verified_utc':utc(),'head':head,'artifacts':checked,'frozen_files':manifest['frozen_files'],'manifest_sha256':sha(ROOT/'manifest.json')}
    command=[manifest['checked_artifacts']['probe_executable']['path'],str(ROOT/'preflight-input.json')]
    start=time.monotonic()
    write_new('preflight-invocation.started.json',{'state':'started','utc':utc(),'command':command,'pre_invocation_identity':guard,'provider_calls':0,'retry':0})
    with (ROOT/'preflight.stdout.jsonl').open('xb') as stdout, (ROOT/'preflight.stderr.txt').open('xb') as stderr:
        returncode=None;error=None
        try:
            completed=subprocess.run(command,stdout=stdout,stderr=stderr,timeout=60,check=False)
            returncode=completed.returncode
        except subprocess.TimeoutExpired:
            error='probe_timeout_60_seconds'
        except OSError as exc:
            error=type(exc).__name__
        finally:
            stdout.flush();stderr.flush();os.fsync(stdout.fileno());os.fsync(stderr.fileno())
    write_new('preflight-invocation.json',{'state':'finished','utc':utc(),'command':command,'pre_invocation_identity':guard,'returncode':returncode,'execution_error':error,'elapsed_seconds':time.monotonic()-start,'provider_calls':0,'retries':0,'stdout_sha256':sha(ROOT/'preflight.stdout.jsonl'),'stderr_sha256':sha(ROOT/'preflight.stderr.txt')})
    rows=(ROOT/'preflight.stdout.jsonl').read_text().splitlines()
    schema_ok=False
    if returncode==0 and error is None and len(rows)==1 and (ROOT/'preflight.stderr.txt').stat().st_size==0:
        try:
            row=json.loads(rows[0])
            schema_ok=(row.get('id')=='api-contract-sentinel-v2' and row.get('engine_version')==manifest['candidate']['package_version'] and row['effective_v17']['relation_scope'] in ('requested_relation','different_relation','relation_absent','unresolved') and row['v30']['disposition'] in ('relevant','irrelevant','ambiguous') and isinstance(row['v29'],dict))
        except (TypeError,ValueError,KeyError):schema_ok=False
    write_new('preflight-summary.json',{'identity':manifest['identity'],'outcome':'PASS' if schema_ok else 'FAIL','scope':'One distinct API-contract sentinel, no holdout inference','returncode':returncode,'execution_error':error,'rows':len(rows),'provider_calls':0,'retries':0})
    print(json.dumps({'preflight':'PASS' if schema_ok else 'FAIL','returncode':returncode,'rows':len(rows)}))
if __name__=='__main__':main()
