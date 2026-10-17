"""Frozen v25 evaluator contract; no Engine/provider execution on import."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,os,subprocess
ROOT=Path(__file__).resolve().parent
REPO=ROOT.parent.parent
SOURCE='95053007bba54c728eee428837b22eb9d52a4f61'
VERSION='0.7.2'
POLICY_SHA='3a6d37e3fbf134f4a49677fdcd7de8b1b625739988a40e1c1e78b94978099767'
HELPER_SHA='00f6971900864ee9e89e4db6029d8196fabd0a4b6ab28e343f19506417735c6f'
IDENTITY={'exact_target','distinct_target','target_absent','unresolved'}
RELATION={'requested_relation','different_relation','relation_absent','unresolved'}
RISK={'none','identity_mapping','ownership_scope','context_gap','multiple'}
DISPOSITION={'relevant','irrelevant','ambiguous'}
BUDGET={'max_model_attempts':2,'max_tokens':192,'max_elapsed_ms':90000}
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def utc():return datetime.now(timezone.utc).isoformat()
def dt(value):return datetime.fromisoformat(value)
def unique_object(pairs):
 d={}
 for k,v in pairs:
  if k in d:raise ValueError('duplicate_json_key:'+k)
  d[k]=v
 return d
def strict_json(text):
 return json.loads(text,object_pairs_hook=unique_object,parse_constant=lambda x:(_ for _ in ()).throw(ValueError('nonfinite_json')))
def load(p):return strict_json(Path(p).read_text())
def write_new(path,data):
 with Path(path).open('x') as f:json.dump(data,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def checked_artifacts(artifacts):
 return bool(artifacts) and all(isinstance(a,dict) and Path(a['path']).is_absolute() and sha(a['path'])==a['sha256'] and Path(a['path']).stat().st_size==a['bytes'] for a in artifacts.values())
def preserved(root=ROOT):
 old=load(root/'predecessor-artifact-hashes.json');total=0
 for v,fs in old.items():
  d=root.parent/('engine073-launch-independent-'+v)
  if {str(q.relative_to(d)) for q in d.rglob('*') if q.is_file()}!=set(fs):raise ValueError('old_inventory_changed:'+v)
  for n,h in fs.items():
   if sha(d/n)!=h:raise ValueError('old_hash_changed:'+v+'/'+n)
  total+=len(fs)
 if total!=632:raise ValueError('old632_required')
 prep=load(root/'preserved-preparation-hashes.json')
 if len(prep)!=5 or any(sha(root.parent.parent/n)!=h for n,h in prep.items()):raise ValueError('preparation_changed')
 return total,len(prep)
def evaluator_integrity(root=ROOT):
 freeze=load(root/'evaluator-freeze.json')
 if freeze['identity']!='engine073-launch-independent-v25-pre-author':raise ValueError('wrong_M0_identity')
 if any(sha(root/n)!=h for n,h in freeze['frozen_files'].items()):raise ValueError('M0_frozen_file_changed')
 if not checked_artifacts(freeze['checked_artifacts']):raise ValueError('M0_artifact_changed')
 if freeze['candidate']['source_sha']!=SOURCE or freeze['candidate']['package_version']!=VERSION:raise ValueError('wrong_M0_candidate')
 return freeze
