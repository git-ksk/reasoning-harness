"""Offline HTTP429 structural control. No network; dummy nonsecret credential."""
import contextlib,hashlib,io,json,os,runpy,urllib.error,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parent
SOURCE=ROOT/'readiness.reviewed.py'
OUT=ROOT/'http429-repaired-first'
assert hashlib.sha256(SOURCE.read_bytes()).hexdigest()=='a6bd9dc8b2875714cd6dc15f9c25e312b6b05aa6bd62bfeebf3592872d80955f'
old=dict(os.environ);old_opener=urllib.request.build_opener;attempts=[]
class Opener:
 def open(self,request,timeout):
  assert (OUT/'reservation.json').is_file() and (OUT/'request.json').is_file()
  attempts.append({'url':request.full_url,'timeout':timeout,'reservation_existed_before_mock_send':True})
  raise urllib.error.HTTPError(request.full_url,429,'mock quota exhausted',{'x-ratelimit-remaining-tokens':'0','retry-after':'60'},io.BytesIO(b'{"error":{"type":"mock_quota_exhausted"}}'))
try:
 os.environ['READINESS_OUTPUT']=str(OUT);os.environ['GROQ_API_KEY']='dummy-nonsecret-offline-only'
 urllib.request.build_opener=lambda *a,**k:Opener()
 with (ROOT/'http429-repaired-first.stdout').open('x') as stdout,(ROOT/'http429-repaired-first.stderr').open('x') as stderr:
  with contextlib.redirect_stdout(stdout),contextlib.redirect_stderr(stderr):
   try:runpy.run_path(str(SOURCE),run_name='__main__');exitcode=0
   except SystemExit as e:exitcode=e.code
 assert len(attempts)==1 and exitcode==1
 observation=json.loads((OUT/'observation.json').read_text());assert observation['status']==429 and observation['usage'] is None
 result={'scope':'offline supplied script structural control only','network_provider_model_calls':0,'mock_send_attempts':len(attempts),'script_exitcode':exitcode,'status':observation['status'],'usage':observation['usage'],'reservation_before_mock_send':True,'interpretation':'HTTP429 is retained with unknown usage and exit1; no retry or automatic quota recovery.'}
 with (ROOT/'http429-repaired-control-result.json').open('x') as f:json.dump(result,f,indent=2);f.write('\n')
 print(json.dumps(result,indent=2))
finally:
 urllib.request.build_opener=old_opener;os.environ.clear();os.environ.update(old)
