#!/usr/bin/env python3
"""Inert fixture: callback -> real CLI -> durable runner -> collector.

This deliberately does NOT create or impersonate a real execution receipt.
The fixture-model callback uses the existing numeric-loopback-only admission;
check_plan is replaced in-process solely for this 3-job synthetic plan. Thus
this tests wiring and restart/latch behavior, not real receipt admission,
readiness, complete 48-job plans, semantic quality, or accuracy acceptance.
"""
import argparse
import hashlib
import json
import os
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest.mock import patch

HERE=Path(__file__).resolve().parent
sys.path.insert(1,str(HERE.parent))
import run_lane

if Path(sys.modules[run_lane.outcome_from_normal.__module__].__file__).resolve() != HERE/'operational_bridge.py':
    raise RuntimeError('fixture_must_use_normal_profile_bridge')

class FixtureHandler(BaseHTTPRequestHandler):
    requests=[]
    def log_message(self,*args):pass
    def do_POST(self):
        request=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        self.requests.append(request)
        n=len(self.requests)
        if n>5:
            value={'message':'quota exhausted in inert fixture'}
            status=429
        else:
            sequence=[
                {'claims':[{'id':'c1','statement':'Engineering value needs evidence.','proposed_state':'unknown'}],'inferences':[]},
                {'targets':[{'id':'target-1','question':'What is the engineering value?'}]},
                {'action':'acquire','target_id':'target-1','capability_id':'independent-v2-source-0'},
                {'claims':[{'id':'c1','statement':'Engineering value is four.','proposed_state':'supported','evidence_ids':['snapshot:engineering-01:0'],'proposition':{'key':'unit.value','value':'4'}}],'inferences':[]},
                {'text':'Engineering value is four.','factual_claims':[{'mode':'grounded','proposition':{'key':'unit.value','value':'4'}}]},
            ]
            value={'model':'fixture-model','choices':[{'message':{'content':json.dumps(sequence[n-1])},'finish_reason':'stop'}],
                   'usage':{'prompt_tokens':12,'completion_tokens':8,'total_tokens':20}}
            status=200
        blob=json.dumps(value).encode()
        self.send_response(status);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(blob)))
        self.end_headers();self.wfile.write(blob)

def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def dump(path,obj):Path(path).write_text(json.dumps(obj,indent=2)+'\n')

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bin-dir',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    a=parser.parse_args();bins=a.bin_dir.resolve();root=a.output.resolve();root.mkdir(mode=0o700)
    required=['reason-baseline','reason-candidate','registry-baseline']
    for name in required:
        if not (bins/name).is_file():raise ValueError('instrumented_fixture_binary_missing:'+name)
    corpus_root=root/'fixture-corpus';corpus_root.mkdir()
    sources=[]
    for n,text in enumerate(['The engineering value is four.','Four is the engineering value.']):
        f=corpus_root/f'source{n}.txt';f.write_text(text);digest=sha(f)
        sources.append({'source_id':f's{n}','quote':text,'normalized_path':str(f),'start_byte':0,'end_byte':len(text.encode()),
            'source_version':'sha256:'+digest,'url':f'https://example.invalid/engineering/{n}','locator':'engineering-only',
            'retrieved_at_unix_seconds':100,'scope':{'product':'engineering'},'reference_facts':{'unit.value':'4'},
            'raw_sha256':digest,'document_id':f'd{n}','context_summary':'Inert fixture only','origin_group':'engineering','synthetic_mutation':False})
    cases=[{'id':f'engineering-{n:02}','question':'What is the engineering value?',
            'helper_question':'Unused fixture helper','source_ids':['s0','s1'],
            'lookup_key':'unit.value','required_scope':{'product':'engineering'}} for n in range(1,4)]
    corpus={'cases':cases,'documents':{'d0':{'retrieved_at_unix_seconds':100},'d1':{'retrieved_at_unix_seconds':100}},
            'sources':{s['source_id']:s for s in sources}}
    dump(corpus_root/'corpus.json',corpus)
    dump(corpus_root/'host-policy.json',{})
    dump(corpus_root/'gold.json',{'records':{c['id']+'/e2e_normal':{'id':c['id'],'label':'positive','family':'synthetic'} for c in cases}})
    # Not a fake PASS or a prospective execution receipt; never leave this fixture directory.
    receipt=root/'not-a-live-receipt.json';dump(receipt,{'fixture_only':True,'grants_live_execution':False})
    budget={'max_generations':24,'max_input_tokens_per_generation':12512,'max_output_tokens_per_generation':1024,
            'max_attempts_per_generation':1,'aggregate_input_tokens':24000,'aggregate_output_tokens':4096}
    jobs=[]
    for case,arm in zip(cases,['baseline','candidate','baseline']):
        argv=[sys.executable,str(HERE/'run_task.py'),'--provider','mistral','--model','fixture-model',
              '--corpus',str(corpus_root/'corpus.json'),'--host-policy',str(corpus_root/'host-policy.json'),
              '--case',case['id'],'--track','e2e_normal','--arm',arm,'--bin-dir',str(bins)]
        jobs.append({'id':case['id']+'-'+arm,'arm':arm,'contract':'normal','timeout_seconds':120,'cancel_grace_seconds':1,'budget':budget,'argv':argv})
    artifacts={str(p):sha(p) for p in [Path(sys.executable),HERE/'run_task.py',receipt,corpus_root/'corpus.json',
                                     corpus_root/'host-policy.json',corpus_root/'gold.json',*list(corpus_root.glob('source*.txt')),
                                     *[bins/name for name in required]]}
    plan={'schema':'engine07-operational-plan-v1','purpose':'CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE',
          'provider':'mistral','model':'fixture-model','token_budget':3*(24000+4096),
          'artifacts':artifacts,'jobs':jobs,'development_receipt_file':str(receipt),'M1_sha256':'f'*64}
    protocol=json.loads((HERE/'protocol.json').read_text())
    FixtureHandler.requests=[]
    server=ThreadingHTTPServer(('127.0.0.1',0),FixtureHandler)
    thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
    try:
        # Minimal allowlisted environment avoids inherited credentials or proxies.
        env={'PATH':os.defpath,'LANG':'C.UTF-8','MISTRAL_API_KEY':'inert-loopback-only',
             'ENGINE073_LOOPBACK_BASE_URL':f'http://127.0.0.1:{server.server_port}/',
             'CLI_REPAIR_REGISTRY_BIN':str(bins/'registry-baseline')}
        lane=root/'lane'
        with patch.dict(os.environ,env,clear=True),patch.object(run_lane,'check_plan',return_value='0'):
            runner=run_lane.AuditedRunner(plan,lane,protocol)
            result=runner.run()
            try:run_lane.AuditedRunner(plan,lane,protocol)
            except FileExistsError:pass
            else:raise AssertionError('existing lane must not restart')
        assert result['complete'] is False and len(result['rows'])==3,result
        assert result['rows'][0]['complete'] is True,result['rows'][0]
        assert result['rows'][1]['complete'] is False,result['rows'][1]
        assert result['rows'][1]['outcome']['failure_class']=='quota',result['rows'][1]
        assert result['rows'][2]['admission']['allowed'] is False,result['rows'][2]
        assert len(FixtureHandler.requests)==6, len(FixtureHandler.requests)
        assert (lane/'event-000002.json').is_file() and (lane/'state.json').is_file()
        state=json.loads((lane/'state.json').read_text())
        assert state['guard']['latch']=='typed_quota' and not state['guard']['inflight'],state
        assert state['charged_reservations']==2*(24000+4096),state
        first=lane/jobs[0]['id'];events=[json.loads(x) for x in (first/'model-journal.jsonl').read_text().splitlines()]
        assert sum(e.get('kind')=='model_response' for e in events)==5
        command=[sys.executable,str(HERE.parent/'collect_checked.py'),'--corpus-dir',str(corpus_root),
                 '--provider','mistral','--lane',str(lane),'--output',str(root/'observations')]
        collected=subprocess.run(command,capture_output=True,env={'PATH':os.defpath,'LANG':'C.UTF-8'},timeout=60)
        if collected.returncode:raise AssertionError('collector failed: '+collected.stderr.decode(errors='replace'))
        index=json.loads((root/'observations/observation-index.json').read_text())
        assert index['observations']==3,index
        assert not index['binding_failures'],index
        observations=[json.loads(p.read_text()) for p in (root/'observations').glob('*.json') if p.name not in ['observation-index.json'] and not p.name.endswith('.review-packet.json')]
        assert sorted(x['operation'] for x in observations)==['complete','incomplete','not_run'],observations
        report={'scope':'SYNTHETIC_LOOPBACK_ONLY_NOT_REAL_ADMISSION','fixture_plan_jobs':3,'complete':1,'failed':1,'suppressed':1,
                'loopback_requests':len(FixtureHandler.requests),'external_provider_requests':0,'models_used':False,
                'precredential_guard_stubbed_for_fixture':True,'real_receipt_admission_tested':False,'f1_measured':False,
                'durable_restart_refused':True,'collected_observations':index['observations'],'binding_failures':index['binding_failures']}
        dump(root/'summary.json',report);print(json.dumps(report))
    finally:
        server.shutdown();server.server_close();thread.join()
if __name__=='__main__':main()
