#!/usr/bin/env python3
"""One metadata GET per provider; no generation, retries or quota guarantee."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import time
import urllib.error
import urllib.request

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None

def verify(review_path, proof_path):
    review = json.loads(review_path.read_bytes())
    proof = json.loads(proof_path.read_bytes())
    protocol = json.loads((HERE/'protocol.json').read_bytes())
    policy = protocol['operational_review']
    assert review['result'] == 'PASS' and review['scope'] == policy['required_scope']
    assert review['reviewer'] == policy['reviewer_id'] and review['reviewer_kind'] == 'agent_nonhuman'
    assert review['runtime_proof_sha256'] == digest(proof_path)
    for key in ['source_commit','paired_host_identity_sha256','runtime_image_id','runtime_image_archive_sha256']:
        assert review[key] == proof[key]
    files = {str(p.relative_to(REPO)): digest(p) for p in REPO.rglob('*')
             if p.is_file() and '__pycache__' not in p.parts}
    assert files == proof['runtime_files']
    assert review['reviewed_runtime_files_sha256'] == hashlib.sha256(json.dumps(files,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    return protocol, proof

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--provider', choices=['mistral','google'], required=True)
    parser.add_argument('--review', type=Path, required=True)
    parser.add_argument('--runtime-proof', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--verify-only', action='store_true')
    args = parser.parse_args()
    protocol, proof = verify(args.review, args.runtime_proof)
    if args.verify_only:
        print('Exact operational review and runtime verified; no provider request.')
        return
    assert args.output is not None
    args.output.parent.mkdir(parents=True,exist_ok=True)
    # Reserve the exclusive record before reading credentials or opening HTTP.
    with args.output.open('x') as output:
        started = int(time.time())
        record = {'provider':args.provider,'model':protocol['providers'][args.provider]['model'],
                  'source_commit':proof['source_commit'],'runtime_proof_sha256':digest(args.runtime_proof),
                  'operational_review_sha256':digest(args.review),'checked_at_unix_seconds':started,
                  'valid_until_unix_seconds':started+86400,'model_generations':0,'maximum_metadata_requests':1,
                  'metadata_requests_attempted':0,'authentication_and_model_available':False,
                  'quota_headroom':'UNKNOWN','full_cohort_completion_guaranteed':False,
                  'http_status':None,'response_rate_limits':{},'error_class':None}
        output.write(json.dumps(record,indent=2)+'\n');output.flush();os.fsync(output.fileno())
        provider = args.provider
        model = record['model']
        key = os.environ.get('MISTRAL_API_KEY' if provider=='mistral' else 'GEMINI_API_KEY','')
        if not key:
            record['error_class']='missing_credential'
        else:
            url = ('https://api.mistral.ai/v1/models' if provider=='mistral'
                   else 'https://generativelanguage.googleapis.com/v1beta/models/'+model)
            headers = {'Authorization':'Bearer '+key} if provider=='mistral' else {'x-goog-api-key':key}
            request = urllib.request.Request(url,headers=headers,method='GET')
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}),NoRedirect())
            record['metadata_requests_attempted']=1
            # Persist attempted state before network; interruptions cannot imply no request.
            output.seek(0);output.truncate();output.write(json.dumps(record,indent=2)+'\n');output.flush();os.fsync(output.fileno())
            try:
                with opener.open(request,timeout=30) as response:
                    record['http_status']=response.status
                    raw=response.read(1024*1024+1)
                    if len(raw)>1024*1024:
                        raise ValueError('metadata_response_too_large')
                    data=json.loads(raw)
                    available = (any(item.get('id')==model and item.get('capabilities',{}).get('completion_chat') is True for item in data.get('data',[]))
                                 if provider=='mistral' else data.get('name')=='models/'+model and 'generateContent' in data.get('supportedGenerationMethods',[]))
                    record['authentication_and_model_available']=response.status==200 and available
                    for name in ['x-ratelimit-limit-requests','x-ratelimit-remaining-requests','x-ratelimit-limit-tokens','x-ratelimit-remaining-tokens']:
                        value=response.headers.get(name)
                        if value is not None and value.isdecimal():record['response_rate_limits'][name]=int(value)
            except urllib.error.HTTPError as error:
                record['http_status']=error.code;record['error_class']='http_error'
            except (urllib.error.URLError,TimeoutError,OSError,ValueError,KeyError,TypeError):
                record['error_class']='metadata_transport_or_schema_error'
        output.seek(0);output.truncate();output.write(json.dumps(record,indent=2)+'\n');output.flush();os.fsync(output.fileno())
    print(json.dumps({k:record[k] for k in ['provider','http_status','authentication_and_model_available','quota_headroom','model_generations']}))
    if not record['authentication_and_model_available']:raise SystemExit(1)

if __name__=='__main__':main()
