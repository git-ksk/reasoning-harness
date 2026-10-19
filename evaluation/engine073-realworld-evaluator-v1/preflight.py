#!/usr/bin/env python3
"""Actual loopback HTTP provider-library controls, with zero external requests."""
import argparse,json,os,subprocess,threading
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from pathlib import Path

class Handler(BaseHTTPRequestHandler):
    requests=[];mode='ok';provider='mistral'
    def log_message(self,*args):pass
    def do_POST(self):
        body=json.loads(self.rfile.read(int(self.headers['Content-Length'])));self.requests.append(body)
        if self.provider=='google':
            value={'id':'loopback-fixture','model':'fixture-model','status':'completed','steps':[{'type':'model_output','content':[{'type':'text','text':'fixture'}]}],
                'usage':{'total_input_tokens':12,'total_output_tokens':8,'total_tokens':20}}
        else:value={'model':'fixture-model','choices':[{'message':{'content':'fixture'},'finish_reason':'stop'}],
                    'usage':{'prompt_tokens':12,'completion_tokens':8,'total_tokens':20}}
        if self.mode=='missing-usage':value.pop('usage')
        blob=json.dumps(value).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(blob)));self.end_headers();self.wfile.write(blob)
def main():
    p=argparse.ArgumentParser();p.add_argument('--bin-dir',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();a.output.mkdir()
    summaries=[]
    for arm in ['baseline','candidate']:
        for provider in ['mistral','google','groq']:
            for mode in ['ok','oversize','missing-usage']:
                directory=a.output/(arm+'-'+provider+'-'+mode);directory.mkdir();Handler.requests=[];Handler.mode=mode;Handler.provider=provider
                server=ThreadingHTTPServer(('127.0.0.1',0),Handler);thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
                try:
                    env=os.environ.copy()
                    for k in ['MISTRAL_API_KEY','GEMINI_API_KEY','GROQ_API_KEY']:env.pop(k,None)
                    env.update(ENGINE073_MODEL_JOURNAL=str(directory/'model.jsonl'),ENGINE073_GENERATION_CAP='12',REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS='0',REASON_GROQ_STRUCTURED_OUTPUT_RETRIES='0')
                    cmd=[str(a.bin_dir/('audit-probe-'+arm)),provider,'http://127.0.0.1:'+str(server.server_port)+'/',mode]
                    r=subprocess.run(cmd,env=env,capture_output=True,timeout=30)
                    (directory/'stdout.json').write_bytes(r.stdout);(directory/'stderr.txt').write_bytes(r.stderr)
                    (directory/'requests.json').write_text(json.dumps(Handler.requests,indent=2)+'\n')
                    assert r.returncode==0,r.stderr.decode();value=json.loads(r.stdout)
                    assert value['first_ok']==(mode=='ok'),value
                    assert len(Handler.requests)==(0 if mode=='oversize' else 1),Handler.requests
                    if mode=='missing-usage':assert value['second_ok'] is False,value
                    summaries.append({'arm':arm,'provider':provider,'mode':mode,'pass':True,'loopback_requests':len(Handler.requests),'external_provider_requests':0})
                finally:server.shutdown();server.server_close();thread.join()
    (a.output/'summary.json').write_text(json.dumps({'controls':summaries,'all_pass':True,'external_provider_requests':0,'accuracy_cases':0},indent=2)+'\n')
    print(json.dumps({'all_pass':True,'loopback_controls':len(summaries),'external_provider_requests':0,'accuracy_cases':0}))
if __name__=='__main__':main()
