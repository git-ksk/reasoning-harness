#!/usr/bin/env python3
"""Check frozen first observations without models or rescoring final answers."""
import argparse,hashlib,json,sys
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--raw-root',type=Path,required=True);p.add_argument('--frozen-suite',type=Path,required=True);p.add_argument('--publication',action='store_true');a=p.parse_args()
index=json.loads((Path(__file__).parent/'index.json').read_text())
sys.path.insert(0,str(a.frozen_suite.resolve()))
from run import validate
from score import summarize
corpus,protocol=validate();expected_ids={c['id'] for c in corpus['cases']};results=[]
for provider in ('mistral','google','groq'):
 root=a.raw_root/provider
 def read(name):return json.loads((root/name).read_text())
 identity=read('execution-identity.json')
 assert identity['git_commit']==index['freeze_commit'] and str(identity['run_id'])==str(index['run_id']) and str(identity['attempt'])=='1' and identity['provider']==provider
 assert identity['manifest_sha256']==hashlib.sha256((a.frozen_suite/'manifest.sha256').read_bytes()).hexdigest()
 for name in ('corpus.json','protocol.json','manifest.sha256'):assert (root/name).read_bytes()==(a.frozen_suite/name).read_bytes()
 redactions=read('publication-redactions.json') if a.publication and (root/'publication-redactions.json').exists() else {}
 assert len(redactions)==(index['providers'][provider]['publication_redacted_files'] if a.publication else 0)
 journal=[json.loads(line) for line in (root/'invocations.jsonl').read_text().splitlines()];starts={};finished=set();hash_checks=0;redacted_checks=0
 for e in journal:
  directory=e['directory'];assert (root/directory).resolve().is_relative_to(root.resolve())
  if e['event']=='START':assert directory not in starts;starts[directory]=e
  elif e['event']=='FINISH':
   assert directory in starts and directory not in finished;finished.add(directory)
   for name in ('stdout.json','stderr.txt'):
    relative=directory+'/'+name;actual=hashlib.sha256((root/relative).read_bytes()).hexdigest();expected=e['stdout_sha256' if name=='stdout.json' else 'stderr_sha256']
    if relative in redactions:
     entry=redactions[relative];assert entry['rule']=='provider_account_identifier' and entry['original_sha256']==expected and entry['published_sha256']==actual;redacted_checks+=1
    else:assert actual==expected
    hash_checks+=1
  else:raise ValueError('unexpected journal event')
 assert set(starts)==finished,'partial invocation; do not call complete'
 rows=read('normal-rows.json');helper=read('helper-rows.json');summary=read('summary.json')
 assert len(rows)==48 and len(helper)==24 and {r['id'] for r in rows}==expected_ids and {r['id'] for r in helper}==expected_ids
 for row in rows:assert read(row['id']+'/normal-'+row['engine']+'/score.json')==row
 for row in helper:assert read(row['id']+'/helper-0.7.2/score.json')==row
 assert summarize(provider,rows,helper)==summary==index['providers'][provider]['summary']
 results.append({'provider':provider,'raw_file_hash_checks':hash_checks,'publication_redaction_hash_checks':redacted_checks,'invocations':len(finished),'normal_rows':len(rows),'helper_rows':len(helper),'summary_matches_frozen_scorer':True})
print(json.dumps({'verified':True,'publication_derivative':a.publication,'model_calls_performed':0,'results':results},indent=2))
