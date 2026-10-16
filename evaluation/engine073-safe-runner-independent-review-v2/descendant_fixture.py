"""Offline fork fixture: signal-only, no provider imports or sockets."""
import argparse,json,os,signal,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--mode');p.add_argument('--provider');p.add_argument('--model');a=p.parse_args()
child=os.fork()
if child==0:
 signal.signal(signal.SIGINT,signal.SIG_IGN)
 Path('descendant.pid').write_text(str(os.getpid()))
 while True:
  with Path('heartbeat.txt').open('a') as f:f.write('alive\n');f.flush();os.fsync(f.fileno())
  time.sleep(.01)
while not Path('heartbeat.txt').exists():time.sleep(.005)
def done(sig,frame):
 print(json.dumps({'result':{'status':'failed','failure':{'failure_class':'cancelled'},'partial_diagnostics':{'complete':False,'usage_is_lower_bound':True}}}),flush=True)
 raise SystemExit(1)
if a.mode=='cancel':
 signal.signal(signal.SIGINT,done)
 while True:time.sleep(.01)
print(json.dumps({'result':{'output_contract':'reason-natural-output-v4','usage':{'run':{'tracked_from_session_start':True,'total_tokens':10,'provider_attempts':1}}}}),flush=True)
