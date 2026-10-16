import os,signal,time,json,sys
from pathlib import Path
child=os.fork()
if child==0:
 signal.signal(signal.SIGINT,signal.SIG_IGN)
 while True:
  Path('heartbeat.txt').write_text(str(time.monotonic()))
  time.sleep(0.02)
Path('descendant.pid').write_text(str(child))
def done(sig,frame):
 print(json.dumps({'result':{'status':'failed','failure':{'failure_class':'cancelled'},'partial_diagnostics':{'complete':False,'usage_is_lower_bound':True}}}),flush=True)
 raise SystemExit(1)
signal.signal(signal.SIGINT,done)
while True: time.sleep(0.02)
