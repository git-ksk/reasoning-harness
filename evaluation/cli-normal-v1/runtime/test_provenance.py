"""Offline artifact provenance controls: no API, credential or model calls."""
import concurrent.futures, copy, hashlib, importlib.util, io, json, os, shutil, subprocess, sys, tempfile, unittest, zipfile
from pathlib import Path
from unittest.mock import patch
import collect_normal, provenance
spec=importlib.util.spec_from_file_location('ordinary_score_for_test',Path(__file__).resolve().parent/'score_development_first.py')
score_development_first=importlib.util.module_from_spec(spec);spec.loader.exec_module(score_development_first)

class ArtifactProvenance(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
        self.head='a'*40;self.runid=12345;self.aid=67890;self.repo=provenance.REPOSITORY
        self.run={'id':self.runid,'run_attempt':1,'event':'push','head_branch':provenance.TAG,'path':provenance.WORKFLOW,
                  'status':'completed','head_sha':self.head,'repository':{'id':99,'full_name':self.repo},'head_repository':{'id':99,'full_name':self.repo}}
        self.zip=b'inert zip bytes'
        self.artifact={'id':self.aid,'name':f'cli-normal-real24-mistral-{self.runid}','expired':False,'size_in_bytes':len(self.zip),
                       'digest':'sha256:'+provenance.sha(self.zip),'workflow_run':{'id':self.runid,'repository_id':99,'head_repository_id':99,'head_branch':provenance.TAG,'head_sha':self.head}}
    def test_github_metadata_requires_one_exact_run_and_artifact(self):
        self.assertEqual(provenance.verify_metadata(self.repo,self.runid,self.aid,self.run,self.artifact,self.zip),self.head)
        listing={'total_count':1,'artifacts':[{'id':self.aid,'name':self.artifact['name']}]}
        provenance.verify_artifact_list(self.runid,self.aid,listing)
        for changed in [{'total_count':2,'artifacts':listing['artifacts']},
                        {'total_count':2,'artifacts':listing['artifacts']+[{'id':self.aid+1,'name':self.artifact['name']}]},
                        {'total_count':1,'artifacts':[{'id':self.aid+1,'name':self.artifact['name']}]}]:
            with self.assertRaises(ValueError):provenance.verify_artifact_list(self.runid,self.aid,changed)
        changes=[('run_id',lambda r,a:r.update(id=9)),('attempt',lambda r,a:r.update(run_attempt=2)),('head',lambda r,a:r.update(head_sha='b'*40)),
                 ('tag',lambda r,a:r.update(head_branch='old-v2')),('path',lambda r,a:r.update(path='other.yml')),('not_finished',lambda r,a:r.update(status='in_progress')),
                 ('artifact_id',lambda r,a:a.update(id=1)),('artifact_name',lambda r,a:a.update(name='old-v2')),
                 ('artifact_run',lambda r,a:a['workflow_run'].update(id=1)),('artifact_repo',lambda r,a:a['workflow_run'].update(repository_id=1)),
                 ('digest_missing',lambda r,a:a.pop('digest')),('zip_changed',lambda r,a:a.update(digest='sha256:'+'0'*64))]
        for name,change in changes:
            run=copy.deepcopy(self.run);artifact=copy.deepcopy(self.artifact);change(run,artifact)
            with self.subTest(name=name),self.assertRaises(ValueError):provenance.verify_metadata(self.repo,self.runid,self.aid,run,artifact,self.zip)
    def test_archive_rejects_unsafe_or_duplicate_and_accepts_normal(self):
        def make(entries):
            blob=io.BytesIO()
            with zipfile.ZipFile(blob,'w') as z:
                for name,body in entries:z.writestr(name,body)
            return blob.getvalue()
        def check(data,n):
            path=self.root/f'{n}.zip';path.write_bytes(data)
            return provenance.safe_extract(path,self.root/f'out-{n}')
        self.assertEqual((check(make([('lane/state.json','{}')]),'safe')/'lane/state.json').read_text(),'{}')
        invalid=[('../escape','x'),('/abs','x'),('a/../b','x'),('a\\b','x')]
        for i,item in enumerate(invalid):
            with self.subTest(item=item),self.assertRaises(ValueError):check(make([item]),str(i))
        with self.assertRaises(ValueError):check(make([('same','a'),('same','b')]),'duplicate')
        link=zipfile.ZipInfo('link');link.create_system=3;link.external_attr=(0o120777<<16)
        with self.assertRaises(ValueError):check(make([(link,'target')]),'link')
        with patch.object(provenance,'MAX_FILE',2),self.assertRaises(ValueError):check(make([('big','123')]),'size')
    def fixture(self):
        root=self.root/'artifact';root.mkdir();lane=root/'lane';lane.mkdir()
        receipt={'identity':'cli-normal-v1-development-freeze-v1','state':'DEVELOPMENT_FROZEN','execution_tag':provenance.TAG,
                 'consumed_M1_sha256':provenance.M1,'paired_host_identity_sha256':'e'*64,
                 'provider_profiles':{'mistral':{'model':'model'}}}
        workflow_sha=provenance.sha((Path(provenance.__file__).resolve().parents[3]/provenance.WORKFLOW).read_bytes())
        receipt.update(execution_workflow_sha256=workflow_sha,files={provenance.WORKFLOW:workflow_sha})
        def dump(p,obj):p.write_text(json.dumps(obj,sort_keys=True)+'\n')
        dump(root/'development-receipt.json',receipt);rh=provenance.sha((root/'development-receipt.json').read_bytes())
        reservation={'repo':self.repo,'source_sha':self.head,'tag':provenance.TAG,'run_id':self.runid,
                     'tag_runs':[{'path':provenance.WORKFLOW,'id':self.runid,'head_sha':self.head,'run_attempt':1}]}
        dump(root/'run-reservation.json',reservation)
        ident={'schema':'cli-normal-execution-identity-v1','repository':self.repo,'execution_tag':provenance.TAG,'run_id':self.runid,'run_attempt':1,
               'source_sha':self.head,'receipt_sha256':rh,'reservation_sha256':provenance.sha((root/'run-reservation.json').read_bytes())}
        dump(root/'execution-identity.json',ident)
        cases=[f'inert-{i:02}' for i in range(24)];jobs=[];rows=[]
        budget={'max_generations':24,'max_input_tokens_per_generation':12512,'max_output_tokens_per_generation':1024,'max_attempts_per_generation':1,'aggregate_input_tokens':24000,'aggregate_output_tokens':4096}
        for index,cid in enumerate(cases):
            for arm in (['baseline','candidate'] if index%2==0 else ['candidate','baseline']):
                jid=cid+'-'+arm;argv=['/usr/local/bin/python','/repo/evaluation/cli-repair-paired-v1/normal_v1/run_task.py','--provider','mistral','--model','model','--corpus','/frozen/corpus/corpus.json','--host-policy','/frozen/corpus/host-policy.json','--case',cid,'--track','e2e_normal','--arm',arm,'--bin-dir','/eval/bin']
                jobs.append({'id':jid,'arm':arm,'contract':'normal','argv':argv,'timeout_seconds':360,'cancel_grace_seconds':1,'budget':budget})
                row={'id':jid,'arm':arm,'contract':'normal','complete':False,'admission':{'allowed':False,'reason':'typed_quota'},'operational_failure':'typed_quota'};rows.append(row)
                (lane/jid).mkdir()
        first=rows[0];first.update(complete=True,admission={'allowed':True,'reason':None},operational_failure=None,
                                   returncode=0,outcome={'failure_class':None,'tokens':17,'provider_attempts':1})
        second=rows[1];second.update(admission={'allowed':True,'reason':None},operational_failure='quota',
                                     returncode=1,outcome={'failure_class':'quota','tokens':None,'provider_attempts':1})
        for row in (first,second):
            d=lane/row['id'];(d/'stdout.json').write_text('{}');(d/'stderr.txt').write_text('')
            row['stdout_sha256']=provenance.sha((d/'stdout.json').read_bytes());row['stderr_sha256']=provenance.sha((d/'stderr.txt').read_bytes())
        for row in rows:dump(lane/row['id']/'row.json',row)
        plan={'provider':'mistral','model':'model','M1_sha256':provenance.M1,'development_receipt_file':'/frozen/development.json',
              'artifacts':{'/frozen/development.json':rh,'/eval/bin/identity.json':'e'*64},'currency_self_cap_USD':2.5,
              'corpus_root':'/frozen/corpus','purpose':'CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE',
              'callback_contract':'audited-aggregate-callback-v1','token_budget':48*28096,'jobs':jobs,'case_ids':cases}
        dump(root/'plan.json',plan);dump(lane/'plan.json',plan)
        events=[]
        def event(name,count,charged,inflight,current,latch):
            state={'schema':'engine07-durable-arm-v1','sequence':len(events)+1,'provider':'mistral','model':'model',
                   'guard':{'inflight':inflight,'latch':latch},'rows':copy.deepcopy(rows[:count]),
                   'charged_reservations':charged,'current_reserve':current}
            events.append({'event':name,'state':state})
        event('created',0,0,False,None,None)
        event('reserved_inflight',0,28096,True,28096,None);event('observed',1,28096,False,None,None)
        event('reserved_inflight',1,56192,True,28096,None);event('observed',2,56192,False,None,'typed_quota')
        for count in range(3,49):event('suppressed',count,56192,False,None,'typed_quota')
        event('finished',48,56192,False,None,'typed_quota')
        for item in events:dump(lane/f"event-{item['state']['sequence']:06}.json",item)
        dump(lane/'state.json',events[-1]['state'])
        return root,rh,cases
    def test_run_artifact_receipt_lane_binding_rejects_tampering(self):
        root,rh,_=self.fixture()
        verified=provenance.verify_extracted(root,self.repo,self.runid,self.aid,self.head,rh,'d'*64)
        self.assertEqual(verified['artifact_id'],self.aid)
        state=json.loads((root/'lane/state.json').read_bytes());impossible=copy.deepcopy(state['rows'])
        impossible[-1]['admission']={'allowed':True,'reason':None};impossible[-1]['complete']=True
        with self.assertRaisesRegex(ValueError,'lane_started_after_stop'):
            provenance.verify_journal(root/'lane',state,impossible)
        for name,fn in [
            ('receipt',lambda: (root/'development-receipt.json').write_text('{}')),
            ('reservation',lambda: (root/'run-reservation.json').write_text('{}')),
            ('plan',lambda: (root/'plan.json').write_text('{}')),
            ('state',lambda: (root/'lane/state.json').write_text('{}')),
            ('row',lambda: next((root/'lane').glob('inert-*/row.json')).write_text('{}')),
            ('raw',lambda: next((root/'lane').glob('inert-*/stdout.json')).write_text('{"old_run":true}'))]:
            with self.subTest(name=name):
                files={p:p.read_bytes() for p in root.rglob('*') if p.is_file()};fn()
                with self.assertRaises((ValueError,KeyError)):provenance.verify_extracted(root,self.repo,self.runid,self.aid,self.head,rh,'d'*64)
                for p,data in files.items():p.write_bytes(data)
        with self.assertRaises(ValueError):provenance.verify_extracted(root,self.repo,self.runid,self.aid,'b'*40,rh,'d'*64)
    def test_end_to_end_readback_uses_one_fixed_artifact_and_receipt(self):
        root,rh,_=self.fixture();blob=io.BytesIO()
        with zipfile.ZipFile(blob,'w',compression=zipfile.ZIP_DEFLATED) as z:
            for path in sorted(root.rglob('*')):
                if path.is_file():z.writestr(str(path.relative_to(root)),path.read_bytes())
        data=blob.getvalue();metadata=copy.deepcopy(self.artifact)
        metadata.update(size_in_bytes=len(data),digest='sha256:'+provenance.sha(data))
        listing={'total_count':1,'artifacts':[{'id':self.aid,'name':metadata['name']}]}
        calls=[]
        def api(path):
            calls.append(path)
            if path.endswith('/artifacts?per_page=100'):return listing
            if path.endswith('/runs/'+str(self.runid)):return self.run
            if path.endswith('/artifacts/'+str(self.aid)):return metadata
            raise AssertionError('unexpected API path')
        def download(args,stdout,check,timeout):
            self.assertEqual(args,['gh','api','--hostname','github.com',f'repos/{self.repo}/actions/artifacts/{self.aid}/zip'])
            stdout.write(data)
        with patch.object(provenance,'github_json',side_effect=api),patch.object(provenance.subprocess,'run',side_effect=download):
            extracted,binding=provenance.fetch_verified(self.repo,self.runid,self.aid,root/'development-receipt.json',self.root/'readback')
        self.assertTrue((extracted/'lane/state.json').is_file())
        self.assertEqual(binding['receipt_sha256'],rh);self.assertEqual(binding['artifact_zip_sha256'],provenance.sha(data))
        self.assertEqual(len(calls),3)
    def test_new_id_binds_run_and_rejects_relabelled_old_observation(self):
        root,rh,cases=self.fixture();verified=provenance.verify_extracted(root,self.repo,self.runid,self.aid,self.head,rh,'d'*64)
        binding=collect_normal.binding_hash(verified);old=provenance.sha((provenance.M1+'|mistral|'+cases[0]+'|e2e_normal|baseline').encode())[:24]
        new=collect_normal.new_id(binding,old);self.assertNotEqual(new,old)
        other=dict(verified);other['artifact_id']+=1
        self.assertNotEqual(new,collect_normal.new_id(collect_normal.binding_hash(other),old))
        corpus=self.root/'corpus';corpus.mkdir()
        (corpus/'corpus.json').write_text(json.dumps({'cases':[{'id':cid,'source_ids':[]} for cid in cases],'sources':{}}))
        (corpus/'gold.json').write_text(json.dumps({'records':{cid+'/e2e_normal':{'id':cid} for cid in cases}}))
        with patch.object(collect_normal,'consumed_corpus',return_value={'cases':[{'id':cid} for cid in cases]}):
            collect_normal.reconstruct(root,verified,corpus,self.root/'observations')
        derived=json.loads((self.root/'observations'/(new+'.json')).read_bytes())
        self.assertEqual(derived['operation'],'incomplete');self.assertEqual(derived['execution_provenance']['binding_sha256'],binding)
        candidate=self.root/'candidate';shutil.copytree(self.root/'observations',candidate)
        (candidate/'verified-run.json').write_text(json.dumps(verified))
        score_development_first.verify_observation_files(candidate,self.root/'observations',verified)
        # Merely renaming an old row and inserting an asserted provenance cannot
        # equal deterministic reconstruction from the current ZIP's raw lane.
        forged=dict(derived);forged['unvalidated_partial_payload']={'old_run':'pretend'}
        (candidate/(new+'.json')).write_text(json.dumps(forged))
        with self.assertRaisesRegex(ValueError,'observations_not_from_exact_run_artifact'):
            score_development_first.verify_observation_files(candidate,self.root/'observations',verified)
        review={'row_id':new,'observation_sha256':provenance.sha(collect_normal.canonical(derived)),
                'execution_provenance_sha256':binding,'reviewer':{'role':'agent','id':'independent-fixture'}}
        score_development_first.verify_review_binding(review,derived,binding,'independent-fixture')
        for field,value in [('row_id',old),('observation_sha256','f'*64),('execution_provenance_sha256','0'*64)]:
            stale=dict(review);stale[field]=value
            with self.subTest(field=field),self.assertRaisesRegex(ValueError,'review_not_for_exact_artifact'):
                score_development_first.verify_review_binding(stale,derived,binding,'independent-fixture')
    def test_first_score_reservation_is_atomic_and_not_output_scoped(self):
        lock=self.root/'locks';lock.mkdir()
        def run(_):
            try:score_development_first.reserve_first_score(123,'a'*64,456);return 'reserved'
            except FileExistsError:return 'denied'
        with patch.object(score_development_first,'FIRST_SCORE_ROOT',lock):
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:results=list(pool.map(run,range(2)))
        self.assertEqual(sorted(results),['denied','reserved'])
        with patch.object(score_development_first,'FIRST_SCORE_ROOT',lock),self.assertRaises(FileExistsError):
            score_development_first.reserve_first_score(123,'a'*64,999)
    def test_metadata_checks_survive_python_optimized_mode(self):
        program='import provenance; provenance.verify_metadata("other/repo",1,2,{}, {},b"x")'
        result=subprocess.run([sys.executable,'-O','-c',program],capture_output=True,env={'PATH':os.defpath,'PYTHONPATH':str(Path(__file__).parent)})
        self.assertNotEqual(result.returncode,0);self.assertIn(b'invalid_exact_identity',result.stderr)
if __name__=='__main__':unittest.main()
