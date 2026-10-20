"""Public primary documentation only; no credentials, models or evaluation calls."""
from pathlib import Path
from datetime import datetime,timezone
import concurrent.futures,hashlib,json,os,time,urllib.request
NOTES=Path(__file__).resolve().parent
CANDIDATES=[('curl','curl/curl','curl-8_10_1',['docs/cmdline-opts/retry.md','docs/cmdline-opts/retry-all-errors.md']),('git','git/git','v2.47.0',['Documentation/git-check-ignore.txt','Documentation/git-check-attr.txt']),('cmake','Kitware/CMake','v3.31.0',['Help/command/return.rst','Help/command/break.rst']),('openssl','openssl/openssl','openssl-3.0.15',['doc/man3/SSL_get_verify_result.pod','doc/man3/SSL_get_verify_mode.pod']),('sqlite','sqlite/sqlite','version-3.47.2',['README.md','doc/compile-for-unix.md']),('redis','redis/redis','7.2.5',['README.md','SECURITY.md'])]
def get(url):
 req=urllib.request.Request(url,headers={'User-Agent':'source-bound-independent-evaluation-document-capture/1'})
 with urllib.request.urlopen(req,timeout=30) as r:
  raw=r.read(4*1024*1024+1);assert len(raw)<=4*1024*1024;return raw,r.url,dict(r.headers),int(time.time())
def capture(item):
 product,repo,ref,paths=item;result={'product':product,'repository':repo,'ref':ref,'attempts':[]}
 try:
  meta,url,headers,at=get('https://api.github.com/repos/'+repo+'/commits/'+ref);commit=json.loads(meta)['sha'];assert len(commit)==40
  result.update(commit=commit,ref_resolved_at_unix_seconds=at)
  for n,path in enumerate(paths):
   capture_url='https://raw.githubusercontent.com/'+repo+'/'+commit+'/'+path
   try:
    raw,final,headers,at=get(capture_url);assert final==capture_url
    name=product+'-'+str(n+1)+'.raw';dest=NOTES/'acquired'/name;dest.parent.mkdir(exist_ok=True)
    with dest.open('xb') as f:f.write(raw);f.flush();os.fsync(f.fileno())
    result['attempts'].append({'path':path,'capture_url':capture_url,'final_url':final,'retrieved_at_unix_seconds':at,'sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'saved_path':str(dest),'HTTP_status':200})
   except Exception as e:result['attempts'].append({'path':path,'capture_url':capture_url,'failure_type':type(e).__name__,'error':str(e)})
 except Exception as e:result.update(failure_type=type(e).__name__,error=str(e))
 return result
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:results=list(pool.map(capture,CANDIDATES))
with (NOTES/'acquisition-authorized.json').open('x') as f:json.dump(results,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
print(json.dumps(results,indent=2))
