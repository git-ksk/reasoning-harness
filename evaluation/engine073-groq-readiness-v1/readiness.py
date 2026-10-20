#!/usr/bin/env python3
"""One explicitly budgeted quota observation; no evaluation questions or retry."""
import hashlib,json,os,time,urllib.request,urllib.error
from decimal import Decimal
from pathlib import Path
OUT=Path(os.environ['READINESS_OUTPUT']);OUT.mkdir()
body={'model':'openai/gpt-oss-120b','messages':[{'role':'user','content':'Reply OK.'}],'temperature':0,'max_completion_tokens':64,'reasoning_effort':'low','stream':False}
reservation={'identity':'engine073-groq-readiness-20261019-v1','provider':'groq','requested_model':body['model'],'purpose':'quota/header observation only; not accuracy case','owner_reports_no_other_usage':True,'owner_reports_full_allowance':True,'owner_renewed_evaluation_request':True,'historical_depletion_record_unchanged':True,'maximum_HTTP_attempts':1,'input_token_upper_bound':512,'output_token_upper_bound':64,'currency_self_cap_USD':0.001,'created_unix_seconds':int(time.time())}
def dump(name,value):
 with (OUT/name).open('x') as f:json.dump(value,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
# The static wire body plus framing allowance is below512; output is provider capped.
# Deliberate price ceilings exceed documented0.15/0.60 USD per million tokens.
wire=json.dumps(body).encode();worst=(Decimal(512)*Decimal('1')+Decimal(64)*Decimal('2'))/Decimal(1000000)
if len(wire)+128>512 or worst>Decimal('0.001'):raise SystemExit('pre-auth reserve exceeds cap')
reservation.update(wire_UTF8_bytes=len(wire),framing_allowance=128,input_price_ceiling_USD_per_million='1',output_price_ceiling_USD_per_million='2',worst_case_reserved_USD=str(worst),budget_scope='conservative local reservation, not provider invoice guarantee')
dump('reservation.json',reservation);dump('request.json',body)
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*a,**k):return None
request=urllib.request.Request('https://api.groq.com/openai/v1/chat/completions',data=wire,headers={'Authorization':'Bearer '+os.environ['GROQ_API_KEY'],'Content-Type':'application/json'},method='POST')
allowed=['x-ratelimit-limit-requests','x-ratelimit-limit-tokens','x-ratelimit-remaining-requests','x-ratelimit-remaining-tokens','x-ratelimit-reset-requests','x-ratelimit-reset-tokens','retry-after']
try:
 try:response=urllib.request.build_opener(NoRedirect()).open(request,timeout=35)
 except urllib.error.HTTPError as e:response=e
 with response:
  raw=response.read(1024*1024+1);headers={k:response.headers[k] for k in allowed if response.headers.get(k) is not None};status=response.status
 with (OUT/'response-body.json').open('xb') as f:f.write(raw);f.flush();os.fsync(f.fileno())
 value=json.loads(raw);dump('observation.json',{'status':status,'headers':headers,'usage':value.get('usage'),'response_body_sha256':hashlib.sha256(raw).hexdigest(),'HTTP_attempts':1,'no_retry':True,'full_cohort_daily_headroom_proven':False,'actual_answer_F1':None})
 print(json.dumps({'status':status,'headers':headers,'usage':value.get('usage'),'accuracy_cases':0,'full_cohort_ready':False}))
 usage=value.get('usage',{})
 if status!=200 or type(usage.get('prompt_tokens')) is not int or type(usage.get('completion_tokens')) is not int or usage['prompt_tokens']>512 or usage['completion_tokens']>64:raise SystemExit(1)
except Exception as e:
 dump('incomplete.json',{'failure_type':type(e).__name__,'usage':'unknown_not_zero','no_retry':True});raise SystemExit(1)
