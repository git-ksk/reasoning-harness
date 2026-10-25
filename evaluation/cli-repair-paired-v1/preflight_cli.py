#!/usr/bin/env python3
"""Same compiled CLI normal/error/helper paths using inert loopback HTTP only."""
import argparse,json,os,subprocess,threading,hashlib,sys
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path
from scorer import check_model_journal,check_terminal_order,sha
from capture import bind_normal
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]/'evaluation/engine07-realworld-operations-v1'))
from bridge import outcome_from_normal

class Handler(BaseHTTPRequestHandler):
    requests=[];provider='mistral';mode='success'
    def log_message(self,*args):pass
    def do_POST(self):
        body=json.loads(self.rfile.read(int(self.headers['Content-Length'])));self.requests.append(body)
        if self.mode=='acquisition':
            sequence=[
                {'claims':[{'id':'c1','statement':'Engineering value needs evidence.','proposed_state':'unknown'}],'inferences':[]},
                {'targets':[{'id':'target-1','question':'What is the engineering value?'}]},
                {'action':'acquire','target_id':'target-1','capability_id':'independent-v2-source-0'},
                {'claims':[{'id':'c1','statement':'Engineering value is four.','proposed_state':'supported','evidence_ids':['snapshot:engineering-01:0'],'proposition':{'key':'unit.value','value':'4'}}],'inferences':[]},
                {'text':'Engineering value is four.','factual_claims':[{'mode':'grounded','proposition':{'key':'unit.value','value':'4'}}]}]
            text=json.dumps(sequence[len(self.requests)-1])
        elif self.mode=='helper':text=json.dumps({'equivalent':True})
        elif len(self.requests)==1:
            text=json.dumps({'claims':[{'id':'c1','statement':'Engineering value is four.','proposed_state':'known' if self.mode=='success' else 'unknown','proposition':{'key':'unit.value','value':'4'},'evidence_ids':['cli-fact-0'] if self.mode=='success' else []}],'inferences':[]})
        else:text=json.dumps({'text':'Engineering value is four.','factual_claims':[{'mode':'grounded','proposition':{'key':'unit.value','value':'4'}}]})
        if self.provider=='google':
            value={'id':'loopback-cli','model':'fixture-model','status':'completed','steps':[{'type':'model_output','content':[{'type':'text','text':text}]}],'usage':{'total_input_tokens':12,'total_output_tokens':8,'total_tokens':20}}
        else:value={'model':'fixture-model','choices':[{'message':{'content':text},'finish_reason':'stop'}],'usage':{'prompt_tokens':12,'completion_tokens':8,'total_tokens':20}}
        blob=json.dumps(value).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(blob)));self.end_headers();self.wfile.write(blob)

def run_case(arm,provider,mode,bins,root):
    out=root/(arm+'-'+provider+'-'+mode);out.mkdir(mode=0o700)
    Handler.requests=[];Handler.provider=provider;Handler.mode=mode
    server=ThreadingHTTPServer(('127.0.0.1',0),Handler);thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
    env=os.environ.copy()
    for key in ['MISTRAL_API_KEY','GEMINI_API_KEY','GOOGLE_API_KEY','GROQ_API_KEY','NVIDIA_API_KEY']:env.pop(key,None)
    env.update(MISTRAL_API_KEY='inert-loopback-only',GEMINI_API_KEY='inert-loopback-only',GROQ_API_KEY='inert-loopback-only',ENGINE073_LOOPBACK_BASE_URL=f'http://127.0.0.1:{server.server_port}/',ENGINE073_MODEL_JOURNAL=str(out/'model-journal.jsonl'),ENGINE073_GENERATION_CAP='24',REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS='0',REASON_GROQ_STRUCTURED_OUTPUT_RETRIES='0',REASON_HOME=str(out/'private-reason-home'))
    if mode in ['helper','acquisition']:
        sources=[]
        for n,text in enumerate(['The engineering value is four.','Four is the engineering value.']):
            p=out/f'source{n}.txt';p.write_text(text);digest=hashlib.sha256(p.read_bytes()).hexdigest()
            sources.append({'source_id':f's{n}','quote':text,'normalized_path':str(p),'start_byte':0,'end_byte':len(text.encode()),'source_version':'sha256:'+digest,'url':f'https://example.invalid/engineering/{n}','locator':'engineering-only','retrieved_at_unix_seconds':100,'scope':{'product':'engineering'},'reference_facts':{'unit.value':'4'},'raw_sha256':digest,'document_id':f'd{n}','context_summary':'Inert transport control','origin_group':'engineering','synthetic_mutation':False})
        corpus={'cases':[{'id':'engineering-01','question':'What is the engineering value?','helper_question':'Do the engineering descriptions assert the same value?','source_ids':['s0','s1'],'lookup_key':'unit.value','required_scope':{'product':'engineering'}}],'documents':{'d0':{'retrieved_at_unix_seconds':100},'d1':{'retrieved_at_unix_seconds':100}},'sources':{s['source_id']:s for s in sources}}
        (out/'corpus.json').write_text(json.dumps(corpus));(out/'host-policy.json').write_text(json.dumps({'engineering-01':{'controlled_reference_authorized':True}}))
        cmd=[sys.executable,str(HERE/'run_task.py'),'--provider',provider,'--model','fixture-model','--corpus',str(out/'corpus.json'),'--host-policy',str(out/'host-policy.json'),'--case','engineering-01','--track','helper_opt_in' if mode=='helper' else 'e2e_normal','--arm','helper' if mode=='helper' else arm,'--bin-dir',str(bins)]
    else:
        cmd=[str(bins/('reason-'+arm)),'What is the engineering value?','--provider',provider,'--model','fixture-model','--no-config','--format','json','--max-tokens','1024','--max-model-calls','12' if mode=='success' else '1','--max-output-tokens','4096','--max-total-tokens','28096','--diagnostic-trace',str(out/'trace.json')]
        if mode=='success':cmd+=['--fact','unit.value=4','--hypothesis','unit.value=4']
    try:
        p=subprocess.run(cmd,cwd=out,env=env,input=b'',capture_output=True,timeout=90)
        (out/'stdout.json').write_bytes(p.stdout);(out/'stderr.txt').write_bytes(p.stderr);(out/'requests.json').write_text(json.dumps(Handler.requests,indent=2)+'\n');(out/'invocation.json').write_text(json.dumps({'argv':cmd,'returncode':p.returncode},indent=2)+'\n')
        value=json.loads(p.stdout);events=[json.loads(x) for x in (out/'model-journal.jsonl').read_text().splitlines()]
        usage=check_model_journal(events,provider,'fixture-model',24)
        if mode=='acquisition':
            assert p.returncode==0 and len(Handler.requests)==5,value
            model_requests=[event for event in events if event['kind']=='model_request']
            assert model_requests[1]['request']['max_tokens']==json.loads((HERE/'protocol.json').read_text())['normal_bound']['investigation_planner_output_tokens']
            trace=json.loads((out/'trace.json').read_text());corpus['_capture_sha256']=sha((out/'corpus.json').read_bytes());corpus['_expected_case_id']='engineering-01'
            journal=[json.loads(x) for x in (out/'resolver.jsonl').read_text().splitlines()]
            captured=bind_normal(value,trace,journal,json.loads((out/'config.json').read_text()),corpus,events,provider,'fixture-model')
            assert captured['terminal_status']=='qualified_partial_answer' and len(captured['display_citations'])==2 and captured['acquired_source_ids']==['s0'] and captured['admitted_source_ids']==['s0'],captured
            (out/'capture.json').write_text(json.dumps(captured,indent=2)+'\n')
        elif mode=='helper':
            assert p.returncode==0 and len(Handler.requests)==1,value
            for side in ['baseline','candidate']:assert value[side]['artifact_unchanged'] and value[side]['replay_equal'] and not value[side]['claims_promoted'] and not value[side]['unauthorized_reviewed'],value
        else:
            result=value['result'];trace=json.loads((out/'trace.json').read_text());observed=outcome_from_normal(value)
            assert observed.tokens==usage['total_tokens'] and observed.provider_attempts==usage['provider_attempts'],(value,usage)
            if mode=='success':
                assert p.returncode==0 and result['finalization']['status']=='grounded_answer',value
                last=check_terminal_order(trace,result);assert last['result']==result['finalization']
                assert len(Handler.requests)==2 and observed.failure_class is None,value
            else:
                assert p.returncode!=0 and result['failure']['failure_class']=='usage_budget_exceeded',value
                partial=result['partial_diagnostics'];assert partial['complete'] is False and partial['usage_is_lower_bound'] is False and partial['trace_written'] is True,value
                assert len(Handler.requests)==1 and trace['completion_status']=='partial' and not any(e['kind']=='finalization_state' for e in trace['events']),trace
        return {'arm':arm,'provider':provider,'mode':mode,'pass':True,'loopback_requests':len(Handler.requests),'external_provider_requests':0,'usage':usage}
    finally:server.shutdown();server.server_close();thread.join()

def main():
    p=argparse.ArgumentParser();p.add_argument('--bin-dir',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir()
    rows=[]
    for arm in ['baseline','candidate']:
        for provider in ['mistral','google','groq']:
            for mode in ['success','budget-stop']:rows.append(run_case(arm,provider,mode,a.bin_dir.resolve(),a.output.resolve()))
    for arm in ['baseline','candidate']:rows.append(run_case(arm,'google','acquisition',a.bin_dir.resolve(),a.output.resolve()))
    # Helper replays both actual Engine hosts in a single invocation.
    for provider in ['mistral','google','groq']:rows.append(run_case('paired',provider,'helper',a.bin_dir.resolve(),a.output.resolve()))
    (a.output/'summary.json').write_text(json.dumps({'all_pass':True,'controls':rows,'accuracy_cases':0,'external_provider_requests':0},indent=2)+'\n')
    print(json.dumps({'all_pass':True,'actual_CLI_normal_and_budget_controls':12,'paired_helper_controls':3,'actual_acquisition_binding_controls':2,'external_provider_requests':0,'accuracy_cases':0}))
if __name__=='__main__':main()
