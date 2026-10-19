"""Hash-only OCI layers/binaries; extract only declared public evaluator code, no Engine source."""
from pathlib import Path,PurePosixPath
from decimal import Decimal
import hashlib,json,io,os,tarfile,difflib
OUT=Path(__file__).resolve().parent
BUNDLE=Path('/private/tmp/engine073-realworld-M0-runtime-39206178872')
DRAFT=Path('/private/tmp/engine073-realworld-M0-v1-staging/receipt.draft.json')
ROOT=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
def sha(data):return hashlib.sha256(data).hexdigest()
def save(name,data):
 data=data if isinstance(data,bytes) else (json.dumps(data,indent=2,ensure_ascii=False)+'\n').encode()
 p=OUT/name;p.parent.mkdir(parents=True,exist_ok=True)
 with p.open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
archive=BUNDLE/'runtime-image.tar.gz';archive_sha=sha(archive.read_bytes());assert archive_sha=='69c2886e78b7e358bbb0d758bfd48c9a330521dc6ddd1a047f48496208d6229b'
draft_bytes=DRAFT.read_bytes();assert sha(draft_bytes)=='199898c1d54b70d4d77605849ed5cc7e0abd95b668c00c113f7a7f0f61cd70b3';m=json.loads(draft_bytes)
save('M0-receipt.reviewed-draft.json',draft_bytes)
recovery_bytes=(DRAFT.parent/'groq-recovery.json').read_bytes();recovery=json.loads(recovery_bytes);assert sha(recovery_bytes)==m['required_provider_readiness']['groq']['recovery_sha256'];save('groq-recovery.reviewed.json',recovery_bytes)
repo_hashes={};repo_bytes={};bin_hashes={};bin_bytes={};layers=[]
with tarfile.open(archive) as outer:
 manifest=json.loads(outer.extractfile('manifest.json').read())[0];index=json.loads(outer.extractfile('index.json').read())
 config_raw=outer.extractfile(manifest['Config']).read();config=json.loads(config_raw)
 assert 'sha256:'+sha(config_raw)==m['runtime_image_id']=='sha256:067ec40415561e7dff0aefabb4fc5dfb003eef201bcccdfb5fd497d7fae26937'
 for layer in manifest['Layers']:
  blob=outer.extractfile(layer).read();digest=sha(blob);assert digest==PurePosixPath(layer).name;layers.append('sha256:'+digest)
  with tarfile.open(fileobj=io.BytesIO(blob)) as inner:
   for member in inner:
    name=str(PurePosixPath(member.name.removeprefix('./')))
    assert not name.startswith('/') and '..' not in PurePosixPath(name).parts
    if '/.wh.' in name:
     prefix=name.split('/.wh.')[0]+'/'
     for table in [repo_hashes,repo_bytes,bin_hashes,bin_bytes]:
      for key in list(table):
       if key.startswith(prefix):del table[key]
     continue
    if member.isfile() and (name.startswith('repo/') or name.startswith('eval/bin/')):
     data=inner.extractfile(member).read()
     if name.startswith('repo/'):
      rel=name[5:];repo_hashes[rel]=sha(data);repo_bytes[rel]=data
     else:
      rel=name[len('eval/bin/'):];bin_hashes[rel]=sha(data)
      if rel=='identity.json':bin_bytes[rel]=data
assert layers==config['rootfs']['diff_ids']
assert len(repo_hashes)==63 and repo_hashes==m['files']
canonical=sha(json.dumps(repo_hashes,sort_keys=True,separators=(',',':')).encode());assert canonical==m['runtime_file_inventory_canonical_sha256']=='9be785406c62e80d19c11543717fdc40aec358757e38358dd9af523e9ceb09b7'
assert json.loads(Path('/private/tmp/engine073-M0-v2-runtime-files.json').read_text())==repo_hashes
identity=json.loads(bin_bytes['identity.json']);assert sha(bin_bytes['identity.json'])==m['paired_host_identity_sha256']=='143c24b700bbdae258628ee016b80bc3c43cc1d1784f7dbe0f04defac47dc5f5'
assert sha((BUNDLE/'paired-host-identity.json').read_bytes())==sha(bin_bytes['identity.json'])
expected_bins={'identity.json':sha(bin_bytes['identity.json'])}
for arm,v in identity['arms'].items():
 for name,want in v['binaries'].items():expected_bins[name+'-'+arm]=want
assert bin_hashes==expected_bins
old=json.loads((OUT/'runtime-identity-first.json').read_text());assert old['binary_hashes']=={k:v for k,v in bin_hashes.items() if k!='identity.json'}
assert identity['host_commit']==m['common_cli_host_commit']=='ec6e55ff95b8364f9a5d09b8bbfe0ecc0c9d19d7'
assert identity['arms']['baseline']['engine_commit']==m['baseline_source_commit']=='4abee90f0501f0d8fc09b453a0b45c942fbed69e'
assert identity['arms']['candidate']['engine_commit']==m['candidate_source_commit']=='ab3c6322921c8c738ec35bd018345917b5afa538'
assert identity['arms']['candidate']['relevance_sha256']==m['candidate_core_source_sha256']
old_inspect=json.loads(Path('/private/tmp/engine073-realworld-M0-runtime-39205043793/engine073-realworld-M0-runtime-39205043793/image-inspect.json').read_text())[0]
assert layers[:5]==old_inspect['RootFS']['Layers'][:5] and layers[6:]==old_inspect['RootFS']['Layers'][6:]
assert config['config']['Env']==old_inspect['Config']['Env'] and config['config']['Cmd']==old_inspect['Config']['Cmd']
changed={k:{'previous':old['repo_files'].get(k),'v2':v} for k,v in repo_hashes.items() if old['repo_files'].get(k)!=v}
assert set(changed)=={'evaluation/engine073-realworld-evaluator-v1/'+x for x in ['PROTOCOL.md','make_plan.py','run_lane.py','scorer.py','verify_M0.py']}|{'.github/workflows/engine073-realworld-independent-v1.yml'}
for rel,data in repo_bytes.items():
 assert sha((ROOT/rel).read_bytes())==sha(data)
 save('runtime-code-v2/repo/'+rel,data)
# Preserve artifact logs/inventory by hash; supplied CI results remain distinguished.
artifact_files={str(p.relative_to(BUNDLE)):{'sha256':sha(p.read_bytes()),'bytes':p.stat().st_size} for p in sorted(BUNDLE.rglob('*')) if p.is_file()};save('v2-runtime-input-inventory.json',artifact_files)
control=json.loads((BUNDLE/'runtime-controls/loopback/summary.json').read_text());assert control['all_pass'] and len(control['controls'])==18
assert 'Ran 24 tests' in (BUNDLE/'runtime-controls/scorer.stderr').read_text() and (BUNDLE/'runtime-controls/scorer.stderr').read_text().rstrip().endswith('OK')
# External quota raw binding only, no new request.
raw_root=ROOT/'evaluation/engine073-realworld-M0-v1'
for rel,want in recovery['raw_files'].items():assert sha((raw_root/rel).read_bytes())==want
rows=[json.loads(x) for x in (raw_root/'groq-readiness-v2-raw/model.jsonl').read_text().splitlines()]
assert [r['kind'] for r in rows]==['model_request','wire_request','wire_http_response','model_response']
wire=rows[2];response=rows[3]['response'];assert wire['status']==200 and response['usage']=={'input_tokens':246,'output_tokens':32,'total_tokens':278} and response['provider_attempts']==1
assert wire['rate_headers']['x-ratelimit-limit-tokens']=='8000' and wire['rate_headers']['x-ratelimit-limit-requests']=='1000'
reserved=48*(24000+4096)+24*(4096+384);assert reserved==1456128==recovery['full_provider_cohort_reserved_tokens'];assert reserved>recovery['documented_TPD']
assert m['required_provider_readiness']['mistral']['new_live_run_allowed'] is True and m['required_provider_readiness']['google']['new_live_run_allowed'] is True and m['required_provider_readiness']['groq']['new_live_run_allowed'] is False
assert recovery['new_live_run_allowed'] is False and recovery['daily_reserve_capacity_verified'] is False and recovery['actual_answer_f1'] is None
save('runtime-v2-offline-verification.json',{'pass':True,'archive_sha256':archive_sha,'config_digest':m['runtime_image_id'],'OCI_manifest_digest':index['manifests'][0]['digest'],'runtime_repo_file_count':63,'runtime_canonical_sha256':canonical,'binary_count':8,'binary_hashes_identical_to_independently_executed_v1':True,'base_interpreter_libraries_layers_unchanged':True,'changed_repo_files':changed,'supplied_CI_v2_structural_tests':24,'supplied_CI_v2_loopback_controls':18,'independent_prior_actual_controls':'Same 8 binaries, audit, callback host, inherited runner and base runtime; 24 tests,18 loopback and PID1/descendant containment. New whole image not independently Docker-executed due approval-service restriction.','quota_raw_bound':True,'Groq_observed_usage':response['usage'],'Groq_observed_TPM':8000,'Groq_observed_RPD':1000,'Groq_documented_TPD_source':'External recovery record, not independently web-queried','whole_cohort_reserve':reserved,'Groq_deferred':True,'actual_accuracy_authorship':0,'external_calls_by_this_review':0})
print(json.dumps({'pass':True,'runtime_files':63,'canonical_sha256':canonical,'binaries':8,'changed_files':list(changed),'Groq_deferred':True},indent=2))
