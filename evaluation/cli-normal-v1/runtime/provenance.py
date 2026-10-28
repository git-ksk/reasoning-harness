"""Read back one Actions artifact and bind it to the first admitted run.

GitHub API and the separately reviewed runtime are the trust boundary; this is
not a signature and cannot prove that administrators never deleted history.
The functions are also exercised with inert metadata/ZIP fixtures offline.
"""
import hashlib, json, os, re, stat, subprocess, zipfile
from pathlib import Path, PurePosixPath

REPOSITORY='git-ksk/reasoning-harness'
TAG='cli-normal-real24-development-v1'
WORKFLOW='.github/workflows/cli-normal-real24-development.yml'
M1='597e1488d07654b07e2a1e528db8b1ae934e438a8edd3f96d31cfc26dd584cd3'
MAX_FILE=64*1024*1024
MAX_TOTAL=1024*1024*1024
MAX_ENTRIES=5000
MAX_ZIP=512*1024*1024

def require(ok,message):
    if not ok:raise ValueError(message)
def sha(data):return hashlib.sha256(data).hexdigest()
def read(path):return json.loads(Path(path).read_bytes())
def github_json(path):
    return json.loads(subprocess.check_output(['gh','api','--hostname','github.com',path],timeout=60))

def safe_extract(zip_path,output):
    root=Path(output).resolve();root.mkdir()
    names=set();total=0
    with zipfile.ZipFile(zip_path) as archive:
        entries=archive.infolist();require(len(entries)<=MAX_ENTRIES,'archive_too_many_entries')
        for entry in entries:
            name=entry.filename
            require(isinstance(name,str) and name and '\\' not in name and '\x00' not in name,'archive_invalid_name')
            path=PurePosixPath(name)
            require(not path.is_absolute() and all(x not in ('','.','..') for x in name.rstrip('/').split('/')),'archive_unsafe_path')
            normalized=str(path)
            require(normalized not in names,'archive_duplicate_path');names.add(normalized)
            mode=(entry.external_attr>>16)&0xffff
            require(not stat.S_ISLNK(mode) and (mode==0 or stat.S_IFMT(mode) in (0,stat.S_IFREG,stat.S_IFDIR)),'archive_special_file')
            require(not entry.flag_bits&1 and entry.file_size<=MAX_FILE,'archive_file_too_large_or_encrypted')
            total+=entry.file_size;require(total<=MAX_TOTAL,'archive_total_too_large')
            if entry.file_size>1024*1024:
                require(entry.compress_size>0 and entry.file_size<=entry.compress_size*1000,'archive_excessive_expansion')
            dest=root.joinpath(*path.parts);require(dest.resolve().is_relative_to(root),'archive_path_escape')
            if entry.is_dir():dest.mkdir(parents=True,exist_ok=True);continue
            dest.parent.mkdir(parents=True,exist_ok=True)
            with archive.open(entry) as src,dest.open('xb') as out:
                count=0
                while True:
                    chunk=src.read(1024*1024)
                    if not chunk:break
                    count+=len(chunk);require(count<=entry.file_size,'archive_size_mismatch');out.write(chunk)
                require(count==entry.file_size,'archive_size_mismatch')
    return root

def verify_metadata(repo,run_id,artifact_id,run,artifact,zip_bytes):
    require(repo==REPOSITORY and type(run_id) is int and run_id>0 and type(artifact_id) is int and artifact_id>0,'invalid_exact_identity')
    require(run.get('id')==run_id and run.get('run_attempt')==1 and run.get('event')=='push'
            and run.get('head_branch')==TAG and run.get('path')==WORKFLOW and run.get('status')=='completed', 'run_identity_mismatch')
    head=run.get('head_sha');require(isinstance(head,str) and re.fullmatch('[0-9a-f]{40}',head) is not None,'run_head_invalid')
    require(run.get('repository',{}).get('full_name')==repo,'run_repository_mismatch')
    require(type(run['repository'].get('id')) is int and run['repository']['id']>0,'run_repository_id_missing')
    require(run.get('head_repository',{}).get('full_name')==repo and run['head_repository'].get('id')==run['repository']['id'],'run_head_repository_mismatch')
    wr=artifact.get('workflow_run',{})
    require(artifact.get('id')==artifact_id and artifact.get('name')==f'cli-normal-real24-mistral-{run_id}'
            and artifact.get('expired') is False and wr.get('id')==run_id and wr.get('head_sha')==head
            and wr.get('head_branch')==TAG and wr.get('repository_id')==run['repository'].get('id')
            and wr.get('head_repository_id')==run['repository'].get('id'),'artifact_identity_mismatch')
    require(type(artifact.get('size_in_bytes')) is int and 0<artifact['size_in_bytes']<=MAX_ZIP and len(zip_bytes)<=MAX_ZIP,'artifact_archive_too_large')
    digest=artifact.get('digest');require(isinstance(digest,str) and digest=='sha256:'+sha(zip_bytes),'artifact_zip_digest_mismatch')
    return head

def verify_artifact_list(run_id,artifact_id,listing):
    require(isinstance(listing,dict) and type(listing.get('total_count')) is int
        and isinstance(listing.get('artifacts'),list)
        and listing['total_count']==len(listing['artifacts']) and listing['total_count']<100,
        'artifact_list_incomplete')
    expected=f'cli-normal-real24-mistral-{run_id}'
    matches=[item for item in listing['artifacts'] if item.get('name')==expected]
    require(len(matches)==1 and matches[0].get('id')==artifact_id,'artifact_not_unique_for_run')

def verify_journal(lane,state,rows):
    paths=sorted(lane.glob('event-*.json'))
    require(paths and [p.name for p in paths]==[f'event-{n:06}.json' for n in range(1,len(paths)+1)],'lane_journal_sequence')
    events=[read(path) for path in paths]
    require(all(e.get('state',{}).get('sequence')==i for i,e in enumerate(events,1)),'lane_journal_state_sequence')
    def check(index,name,count,charged,inflight,current):
        require(index<len(events),'lane_journal_incomplete')
        event=events[index];snapshot=event.get('state',{});guard=snapshot.get('guard',{})
        require(event.get('event')==name and snapshot.get('rows')==rows[:count]
            and snapshot.get('charged_reservations')==charged and guard.get('inflight') is inflight
            and snapshot.get('current_reserve')==current,'lane_journal_transition')
        return guard
    check(0,'created',0,0,False,None)
    index=1;charged=0;stopped=False
    for count,row in enumerate(rows):
        admission=row.get('admission');require(isinstance(admission,dict) and type(admission.get('allowed')) is bool,'lane_invalid_admission')
        require(type(row.get('complete')) is bool,'lane_invalid_completion')
        if admission['allowed']:
            require(not stopped,'lane_started_after_stop')
            charged+=28096
            guard=check(index,'reserved_inflight',count,charged,True,28096);index+=1
            require(guard.get('latch') is None,'lane_reserved_after_latch')
            guard=check(index,'observed',count+1,charged,False,None);index+=1
            if not row['complete']:
                require(guard.get('latch') is not None,'lane_failure_not_latched');stopped=True
            else:
                outcome=row.get('outcome',{})
                require(row.get('operational_failure') is None and outcome.get('failure_class') is None
                    and type(outcome.get('tokens')) is int and outcome['tokens']>=0
                    and type(outcome.get('provider_attempts')) is int and outcome['provider_attempts']>=0
                    and row.get('returncode')==0,'lane_complete_without_valid_outcome')
        else:
            require(not row['complete'],'lane_suppressed_marked_complete');stopped=True
            guard=check(index,'suppressed',count+1,charged,False,None);index+=1
            require(guard.get('latch') is not None and admission.get('reason')==guard['latch'],'lane_suppression_without_latch')
    check(index,'finished',len(rows),charged,False,None);index+=1
    require(index==len(events) and events[-1]['state']==state,'lane_not_durably_finished')

def verify_extracted(root,repo,run_id,artifact_id,head,receipt_sha,zip_sha):
    root=Path(root);identity=read(root/'execution-identity.json');reservation=read(root/'run-reservation.json')
    receipt=(root/'development-receipt.json').read_bytes();plan=read(root/'plan.json');lane=root/'lane';lane_plan=read(lane/'plan.json');state=read(lane/'state.json')
    require(identity=={'schema':'cli-normal-execution-identity-v1','repository':repo,'execution_tag':TAG,
        'run_id':run_id,'run_attempt':1,'source_sha':head,'receipt_sha256':receipt_sha,
        'reservation_sha256':sha((root/'run-reservation.json').read_bytes())},'execution_identity_mismatch')
    require(sha(receipt)==receipt_sha,'receipt_hash_mismatch')
    m0=json.loads(receipt)
    require(m0.get('identity')=='cli-normal-v1-development-freeze-v1' and m0.get('state')=='DEVELOPMENT_FROZEN'
        and m0.get('execution_tag')==TAG and m0.get('consumed_M1_sha256')==M1,'wrong_frozen_receipt')
    workflow_path=Path(__file__).resolve().parents[3]/WORKFLOW
    workflow_sha=sha(workflow_path.read_bytes())
    require(m0.get('execution_workflow_sha256')==workflow_sha
        and m0.get('files',{}).get(WORKFLOW)==workflow_sha,'receipt_workflow_mismatch')
    require(reservation.get('repo')==repo and reservation.get('source_sha')==head and reservation.get('tag')==TAG
        and reservation.get('run_id')==run_id,'run_reservation_mismatch')
    matching=[r for r in reservation.get('tag_runs',[]) if r.get('path')==WORKFLOW]
    require(len(matching)==1 and matching[0].get('id')==run_id and matching[0].get('head_sha')==head
        and matching[0].get('run_attempt')==1,'reservation_not_unique_first_run')
    require(plan==lane_plan and plan.get('provider')=='mistral' and plan.get('M1_sha256')==M1
        and plan.get('development_receipt_file')=='/frozen/development.json'
        and plan.get('artifacts',{}).get('/frozen/development.json')==receipt_sha,'lane_plan_or_receipt_mismatch')
    require(plan.get('model')==m0.get('provider_profiles',{}).get('mistral',{}).get('model')
        and plan.get('currency_self_cap_USD')==2.5 and plan.get('corpus_root')=='/frozen/corpus'
        and plan.get('purpose')=='CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE'
        and plan.get('callback_contract')=='audited-aggregate-callback-v1'
        and plan.get('artifacts',{}).get('/eval/bin/identity.json')==m0.get('paired_host_identity_sha256'), 'lane_profile_mismatch')
    jobs=plan.get('jobs');rows=state.get('rows')
    require(isinstance(jobs,list) and len(jobs)==48 and isinstance(rows,list) and len(rows)==48,'lane_incomplete')
    cases=plan.get('case_ids');require(isinstance(cases,list) and len(cases)==24 and all(isinstance(c,str) and c for c in cases) and len(set(cases))==24,'lane_case_inventory_mismatch')
    expected=[(cid,arm) for i,cid in enumerate(cases) for arm in (['baseline','candidate'] if i%2==0 else ['candidate','baseline'])]
    bounds={'max_generations':24,'max_input_tokens_per_generation':12512,'max_output_tokens_per_generation':1024,
            'max_attempts_per_generation':1,'aggregate_input_tokens':24000,'aggregate_output_tokens':4096}
    require(plan.get('token_budget')==48*(24000+4096),'lane_reservation_mismatch')
    require(state.get('provider')=='mistral' and state.get('model')==plan.get('model')
        and state.get('schema')=='engine07-durable-arm-v1' and not state.get('guard',{}).get('inflight'),'lane_state_identity')
    require([x.get('id') for x in rows]==[x.get('id') for x in jobs],'lane_rows_not_plan_order')
    for job,row,(case,arm) in zip(jobs,rows,expected):
        require(job.get('contract')=='normal' and job.get('arm') in ('baseline','candidate')
            and row.get('arm')==job['arm']==arm and row.get('contract')=='normal'
            and job.get('id')==case+'-'+arm and job.get('budget')==bounds
            and (job.get('timeout_seconds'),job.get('cancel_grace_seconds'))==(360,1),'lane_wrong_job')
        argv=job.get('argv');require(isinstance(argv,list) and len(argv)==18 and argv[1:]==[
            '/repo/evaluation/cli-repair-paired-v1/normal_v1/run_task.py','--provider','mistral','--model',plan['model'],
            '--corpus','/frozen/corpus/corpus.json','--host-policy','/frozen/corpus/host-policy.json',
            '--case',case,'--track','e2e_normal','--arm',arm,'--bin-dir','/eval/bin']
            and isinstance(argv[0],str) and argv[0].startswith('/'),'lane_callback_mismatch')
        directory=lane/job['id'];require(read(directory/'row.json')==row,'lane_row_changed')
        for field,name in [('stdout_sha256','stdout.json'),('stderr_sha256','stderr.txt')]:
            if field in row:require(sha((directory/name).read_bytes())==row[field],'lane_raw_changed')
    verify_journal(lane,state,rows)
    files={str(p.relative_to(root)):sha(p.read_bytes()) for p in sorted(root.rglob('*')) if p.is_file()}
    return {'schema':'cli-normal-verified-run-v1','repository':repo,'execution_tag':TAG,'run_id':run_id,
        'run_attempt':1,'artifact_id':artifact_id,'source_sha':head,'receipt_sha256':receipt_sha,
        'reservation_sha256':identity['reservation_sha256'],'artifact_zip_sha256':zip_sha,
        'plan_sha256':sha((root/'plan.json').read_bytes()),'state_sha256':sha((lane/'state.json').read_bytes()),
        'artifact_files':files}

def fetch_verified(repo,run_id,artifact_id,receipt_path,work_dir):
    require(repo==REPOSITORY and type(run_id) is int and run_id>0 and type(artifact_id) is int and artifact_id>0,'invalid_exact_identity')
    run=github_json(f'repos/{repo}/actions/runs/{run_id}')
    artifact=github_json(f'repos/{repo}/actions/artifacts/{artifact_id}')
    listing=github_json(f'repos/{repo}/actions/runs/{run_id}/artifacts?per_page=100')
    verify_artifact_list(run_id,artifact_id,listing)
    require(type(artifact.get('size_in_bytes')) is int and 0<artifact['size_in_bytes']<=MAX_ZIP,'artifact_archive_too_large')
    work=Path(work_dir);work.mkdir();archive=work/'artifact.zip'
    with archive.open('xb') as f:subprocess.run(['gh','api','--hostname','github.com',f'repos/{repo}/actions/artifacts/{artifact_id}/zip'],stdout=f,check=True,timeout=120)
    require(archive.stat().st_size<=MAX_ZIP,'artifact_archive_too_large')
    data=archive.read_bytes();head=verify_metadata(repo,run_id,artifact_id,run,artifact,data)
    root=safe_extract(archive,work/'artifact')
    provenance=verify_extracted(root,repo,run_id,artifact_id,head,sha(Path(receipt_path).read_bytes()),sha(data))
    return root,provenance
