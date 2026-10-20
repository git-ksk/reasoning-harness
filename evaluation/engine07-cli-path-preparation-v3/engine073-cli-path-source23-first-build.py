from pathlib import Path
import subprocess,json,hashlib,tarfile,io,os,sys,importlib.util,shutil,re
REPO=Path('/Users/sawadakousuke/x-code/reasoning-harness'); PREP=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
HOST='ec6e55ff95b8364f9a5d09b8bbfe0ecc0c9d19d7';ARMS={'baseline':'4abee90f0501f0d8fc09b453a0b45c942fbed69e','candidate':'01dc78b6952ed77cd20001323f6ab032eeec361e'}
WORK=Path('/private/tmp/engine073-cli-path-source23-preparation');WORK.mkdir(exist_ok=False);(WORK/'bin').mkdir()
def git(*args):return subprocess.check_output(['git',*args],cwd=REPO)
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def extract(data,path):
 with tarfile.open(fileobj=io.BytesIO(data)) as t:
  assert all(not (m.name.startswith('/') or '..' in Path(m.name).parts or m.issym() or m.islnk()) for m in t.getmembers());t.extractall(path)
# Compile-time replacement of the provider's complete generation body; there is
# no runtime fallback to any HTTP client. CLI main and common host stay identical.
fixture_rs=r'''
fn offline_path_response(request: &ModelRequest, model: &str) -> Result<ModelResponse, ModelError> {
    use std::io::Write;
    static INDEX: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let path=std::env::var("ENGINE07_OFFLINE_TRANSCRIPT").map_err(|_| ModelError::new(ModelErrorKind::Protocol,"offline transcript required"))?;
    let data=std::fs::read(path).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"offline transcript unreadable"))?;
    let values: Vec<serde_json::Value>=serde_json::from_slice(&data).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"offline transcript invalid"))?;
    let index=INDEX.fetch_add(1,std::sync::atomic::Ordering::SeqCst);
    let value=values.get(index).ok_or_else(||ModelError::new(ModelErrorKind::Protocol,"offline transcript exhausted"))?;
    let path=std::env::var("ENGINE07_OFFLINE_MODEL_LOG").map_err(|_|ModelError::new(ModelErrorKind::Protocol,"offline request log required"))?;
    let mut log=std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"offline request log unreadable"))?;
    writeln!(log,"{}",serde_json::json!({"index":index,"request":request,"response":value})).map_err(|_|ModelError::new(ModelErrorKind::Protocol,"offline request log failed"))?;
    log.sync_all().map_err(|_|ModelError::new(ModelErrorKind::Protocol,"offline request log fsync failed"))?;
    Ok(ModelResponse {text:serde_json::to_string(value).unwrap(),model:model.to_owned(),usage:reasoning_harness_core::ModelUsage{input_tokens:Some(20),output_tokens:Some(10),total_tokens:Some(30)},provider_attempts:1,finish_reason:Some("stop".into())})
}
'''
def replace_body(s,needle,body):
 a=s.index(needle);a=s.index('{',a)+1;depth=1;b=a
 while depth:
  if s[b]=='{':depth+=1
  if s[b]=='}':depth-=1
  b+=1
 return s[:a]+'\n'+body+'\n    '+s[b-1:]
def instrument(s):
 inserts=[];names=[]
 for m in re.finditer(r'^pub fn ([A-Za-z0-9_]+)\(',s,re.M):
  # Public relevance APIs have ordinary signatures; locate body after closing
  # parameter list, without treating braces inside parameter types as a body.
  depth=1;i=m.end()
  while depth:
   if s[i]=='(':depth+=1
   if s[i]==')':depth-=1
   i+=1
  start=s.index('{',i)+1;name=m[1];names.append(name)
  code='\n    if let Ok(path)=std::env::var("ENGINE07_OFFLINE_API_LOG") { use std::io::Write; let mut log=std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap(); writeln!(log,"'+name+'").unwrap(); log.sync_all().unwrap(); }\n'
  inserts.append((start,code))
 for start,code in reversed(inserts):s=s[:start]+code+s[start:]
 return s,names
v3=PREP/'evaluation/engine07-realworld-v3';spec=importlib.util.spec_from_file_location('fixed_resolver',v3/'resolver.py');resolver=importlib.util.module_from_spec(spec);spec.loader.exec_module(resolver)
corpus=json.loads((v3/'corpus.json').read_text());case=next(c for c in corpus['cases'] if c['id']=='rw03-node-positive')
# These are NEW controlled responses, not a replay of the unavailable original
# provider response bodies and not a new real-source canonical acceptance set.
transcript=[
 {'claims':[{'id':'node-buffer','statement':'The Buffer subtype requires verification.','proposed_state':'unknown','proposition':{'key':'node.buffer.uint8array','value':'true'},'evidence_ids':[]}],'inferences':[]},
 {'targets':[{'id':'realworld-v3-source-0','question':'Are Buffer instances Uint8Array instances?','expected_fact_key':'node.buffer.uint8array'}]},
 {'action':'acquire','target_id':'realworld-v3-source-0','capability_id':'realworld-v3-source-1'},
 {'claims':[{'id':'node-buffer','statement':'Buffer instances are Uint8Array instances.','proposed_state':'supported','proposition':{'key':'node.buffer.uint8array','value':'true'},'evidence_ids':['snapshot:rw03-node-positive:1']}],'inferences':[]},
 {'text':'Buffer instances are Uint8Array instances.','factual_claims':[{'mode':'grounded','proposition':{'key':'node.buffer.uint8array','value':'true'}}]}
]
(WORK/'controlled-transcript.json').write_text(json.dumps(transcript,indent=2)+'\n')
proof={'schema':'engine07-offline-cli-path-preparation-v1','host_commit':HOST,'engine_commits':ARMS,'provider_calls':0,'original_response_replay':False,'real_source_acceptance_complete':False,'arms':{}}
for arm,pin in ARMS.items():
 source=WORK/('source-'+arm);source.mkdir();extract(git('archive',HOST),source)
 if arm=='candidate':
  shutil.rmtree(source/'crates/reasoning-harness-core');extract(git('archive',pin,'crates/reasoning-harness-core'),source)
  lock=source/'Cargo.lock';old='name = "reasoning-harness-core"\nversion = "0.6.1"';t=lock.read_text();assert t.count(old)==1;lock.write_text(t.replace(old,'name = "reasoning-harness-core"\nversion = "0.7.2"'))
 main=source/'crates/reasoning-harness-cli/src/main.rs';main_original=git('show',HOST+':crates/reasoning-harness-cli/src/main.rs');assert main.read_bytes()==main_original
 provider=source/'crates/reasoning-harness-providers/src/mistral.rs';original=provider.read_text();s=replace_body(original,'async fn generate_inner(', '        offline_path_response(&request, &self.model)');provider.write_text(s+'\n'+fixture_rs)
 relevance=source/'crates/reasoning-harness-core/src/evidence_relevance.rs';original_hash=sha(relevance);s,apis=instrument(relevance.read_text());relevance.write_text(s)
 target=Path('/private/tmp/engine073-cli-path-source23-targets')/arm;env=os.environ.copy();env.update(CARGO_TARGET_DIR=str(target),CARGO_INCREMENTAL='0',CARGO_PROFILE_DEV_DEBUG='0')
 with (WORK/(arm+'-build.jsonl')).open('xb') as out,(WORK/(arm+'-build.stderr')).open('xb') as err:
  p=subprocess.run(['cargo','build','--locked','--offline','-p','reasoning-harness-cli','--bin','reason','--jobs','2','--message-format=json'],cwd=source,env=env,stdout=out,stderr=err)
 if p.returncode:raise RuntimeError(arm+' offline build failed; first logs retained')
 binary=WORK/'bin'/('reason-'+arm);shutil.copyfile(target/'debug/reason',binary);binary.chmod(0o755)
 proof['arms'][arm]={'source_relevance_sha256':original_hash,'instrumented_relevance_sha256':sha(relevance),'instrumented_public_apis':apis,'main_sha256':sha(main),'provider_fixture_sha256':sha(provider),'binary_sha256':sha(binary),'lock_sha256':sha(source/'Cargo.lock')}
 print('built '+arm,flush=True)
 (WORK/'build-proof.json').write_text(json.dumps(proof,indent=2)+'\n')
# Run a new offline controlled pair with identical transcript and CLI argv except
# isolated output paths. This never reexecutes or scores the old v3 evaluator.
for arm in ARMS:
 directory=WORK/('run-'+arm);directory.mkdir();cfg=resolver.config(corpus,case,v3/'corpus.json',directory/'resolver.jsonl');(directory/'config.json').write_text(json.dumps(cfg,indent=2)+'\n')
 env=os.environ.copy();env.update(MISTRAL_API_KEY='offline-fixture-not-a-credential',REASON_HOME=str(directory/'reason-home'),ENGINE07_OFFLINE_TRANSCRIPT=str(WORK/'controlled-transcript.json'),ENGINE07_OFFLINE_MODEL_LOG=str(directory/'model.jsonl'),ENGINE07_OFFLINE_API_LOG=str(directory/'relevance-api.log'))
 cmd=[str(WORK/'bin'/('reason-'+arm)),case['question'],'--provider','mistral','--model','ministral-8b-2512','--seed','531003','--max-tokens','1024','--max-model-calls','12','--max-output-tokens','4096','--max-total-tokens','28096','--config',str(directory/'config.json'),'--format','json','--diagnostic-trace',str(directory/'trace.json')]
 (directory/'invocation.json').write_text(json.dumps({'argv':cmd,'offline_only':True},indent=2)+'\n')
 with (directory/'stdout.json').open('xb') as out,(directory/'stderr.txt').open('xb') as err:p=subprocess.run(cmd,cwd=directory,env=env,stdout=out,stderr=err,timeout=60)
 (directory/'exit.json').write_text(json.dumps({'returncode':p.returncode})+'\n');assert p.returncode==0,arm
 print('ran '+arm,flush=True)
print('offline pair complete; not a canonical/live semantic score',flush=True)
