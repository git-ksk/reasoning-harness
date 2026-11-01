import pathlib,subprocess,json,time
out=pathlib.Path(__file__).resolve().parent
frozen=pathlib.Path('target/ordinary-v2-genuine-admission-20261030/frozen').resolve()
raw=pathlib.Path('/Users/sawadakousuke/x-code/reasoning-harness/target/private-evaluation-pending/cli-normal-v2-runtime-40928520250-raw/artifact/paired-artifact')
image='sha256:d10c338c9d3df4a276456f31cbfa6a3b71540efb2bac2910c4b6a627401932b5'
old=pathlib.Path('evaluation/cli-normal-real24-development-v1/receipt.json').resolve()
info=json.loads(subprocess.check_output(['docker','image','inspect',image]))[0]
assert info['Id']==image and info['Architecture']=='amd64' and info['Os']=='linux'
cmd=['docker','run','--pull=never','--rm','--platform','linux/amd64','--network','none','--read-only','--tmpfs','/tmp','-v',str(frozen)+':/frozen:ro','-v',str(out)+':/offline','-v',str(raw)+':/image:ro','-v',str(old)+':/old-v1.json:ro',image,'python3','-B','/offline/admission_probe.py']
start=time.time()
with (out/'probe.stdout').open('xb') as stdout,(out/'probe.stderr').open('xb') as stderr:r=subprocess.run(cmd,stdout=stdout,stderr=stderr)
(out/'execution.json').write_text(json.dumps({'argv':cmd,'returncode':r.returncode,'seconds':round(time.time()-start,3),'image_id':image,'platform':'linux/amd64','real_provider_env_forwarded':False,'network':'none','read_only_root':True,'frozen_inputs_read_only':True,'model_journal_or_production_lane_mount':False},indent=2)+'\n')
print('probe returncode',r.returncode)
if r.returncode:print((out/'probe.stderr').read_text())
raise SystemExit(r.returncode)
