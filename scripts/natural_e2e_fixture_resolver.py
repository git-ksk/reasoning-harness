#!/usr/bin/env python3
import argparse, json, sys

p=argparse.ArgumentParser()
p.add_argument('--mode', choices=['evidence','no-result'], default='evidence')
p.add_argument('--source', required=True)
p.add_argument('--fact-key')
p.add_argument('--fact-value')
p.add_argument('--observed-at', type=int, default=1999999950)
p.add_argument('--retrieved-at', type=int, default=1999999960)
p.add_argument('--authority', default='primary')
p.add_argument('--scope', default='prod')
a=p.parse_args()
try:
    req=json.load(sys.stdin)
except Exception:
    sys.exit(2)
if req.get('schema_version') != 'reason-investigation-external-resolver-request-v1' or req.get('adapter_id') != 'investigation_external_command_v1':
    sys.exit(3)
if req.get('request',{}).get('target',{}).get('kind') != 'investigation_question':
    sys.exit(4)
if a.mode == 'no-result':
    out={'schema_version':'reason-external-resolver-response-v1','contribution':{'kind':'no_result'},'cost':{'calls':1,'added_tokens':0,'elapsed_ms':0}}
else:
    if not a.fact_key or a.fact_value is None:
        sys.exit(5)
    ev={
      'id':f'e2e:{a.source}:{a.fact_key}', 'source':a.source,
      'observation':f'{a.fact_key}={a.fact_value}', 'facts':{a.fact_key:a.fact_value},
      'acquisition_metadata':{
        'observed_at_unix_seconds':a.observed_at,
        'retrieved_at_unix_seconds':a.retrieved_at,
        'scope':{'ecosystem':{'kind':'values','values':[a.scope]}},
        'claimed_authority_class':a.authority,
      }
    }
    out={'schema_version':'reason-external-resolver-response-v1','contribution':{'kind':'acquired_evidence','evidence':[ev]},'cost':{'calls':1,'added_tokens':0,'elapsed_ms':0}}
json.dump(out, sys.stdout, separators=(',',':'))
