"""Pre-observation author review only; retain original bytes and labels before M1."""
from pathlib import Path
import json,hashlib,difflib,os,time
N=Path(__file__).resolve().parent;R=Path('/private/tmp/engine073-realworld-independent-v1-input');history=N/'pre-freeze-history/initial';history.mkdir(parents=True)
old={name:(R/name).read_bytes() for name in ['corpus.json','gold.json','host-policy.json','preparation.json']}
for name,data in old.items():
 with (history/name).open('xb') as f:f.write(data);f.flush();os.fsync(f.fileno())
corpus=json.loads(old['corpus.json']);gold=json.loads(old['gold.json']);prep=json.loads(old['preparation.json'])
neutral={
'curl-1':'curl8.10.1 --retry manual: transient errors, zero default retries, backoff and Retry-After history.',
'curl-2':'curl8.10.1 --retry-all-errors manual: companion option, redirected-IO cautions and HTTP-status versus transfer-error distinction.',
'git-1':'Git2.47.0 check-ignore manual: default/index options, ignore output and exit statuses.',
'git-2':'Git2.47.0 check-attr manual: per-path attribute states, text/NUL output and illustrative .gitattributes examples.',
'cmake-1':'CMake3.31.0 return manual: function/file/directory contexts, CMP0140, PROPAGATE history and block examples.',
'cmake-2':'CMake3.31.0 break command: foreach/while syntax and reference to continue.',
'openssl-1':'OpenSSL3.0.15 SSL_get_verify_result: last verification error, session reuse and absent-peer-certificate caveat.',
'openssl-2':'OpenSSL3.0.15 verification flags and callbacks: preserve client/server and built-in/custom verification contexts.',
'sqlite-1':'SQLite3.47.2 source README: mirror/Fossil names, Unix/Windows builds, source layout and integrity-check caveats.',
'sqlite-2':'SQLite3.47.2 Unix-build recipe: tested2024-10-11, assuming Tcl9.0, configure/targets and debug instructions.',
'redis-1':'Redis7.2.5 quick-start/build README: allocator/platform options, command examples and descriptive internals.',
'redis-2':'Redis7.2.5 repository security-policy snapshot: version table, backport/disclosure and reporting procedures.'}
for s in corpus['sources'].values():s['context_summary']=neutral[s['document_id']]
# Witness lists are precise source alternatives, not an assertion that every
# passage supports every group in a two-fact answer.
assign={'rw-02':[[0],[1]],'rw-03':[[0],[1]],'rw-06':[[0],[1]],'rw-10':[[0],[1]],'rw-11':[[0],[1]],'rw-13':[[0],[1]],'rw-17':[[0],[0,1]],'rw-20':[[1],[1]],'rw-21':[[0,1],[0]]}
case_index={c['id']:c for c in corpus['cases']}
for g in gold['records']:
 if g['track']=='e2e_normal' and g['case_id'] in assign:
  for group,witnesses in zip(g['groups'],assign[g['case_id']]):group['supporting_source_ids']=[case_index[g['case_id']]['source_ids'][i] for i in witnesses]
 if g['id']=='rw-19/e2e_normal':g['groups']=g['groups'][:1] # unasked interpreter prerequisite is a boundary, not extra required content
# Disclosure adjudication is semantic; it does not demand invented boilerplate.
gold['disclosure_interpretation']='Judge meaning in the complete display. A correctly bounded description of a command/example need not recite an exact non-execution phrase if it makes no execution/result claim. Scope disclosures may be satisfied by an explicitly scoped answer and do not require an unasked exception paragraph. Missing-target/current/live uncertainty must actually remain unresolved. No disclosure permits unsupported assertions.'
newblob=(json.dumps(corpus,ensure_ascii=False,indent=2)+'\n').encode();gold['corpus_sha256']=hashlib.sha256(newblob).hexdigest()
prep['source_context_review_completed_unix_seconds']=int(time.time());prep['pre_freeze_scope_cleanup']='Neutral per-document context replaces target-specific absence/equivalence hints; group witness mapping clarified; an unasked extra prerequisite requirement removed while preserving all24 questions, labels, answerability and families. All original bytes/static checks retained outside corpus; Engine/model/actual scorer0.'
new={'corpus.json':newblob,'gold.json':(json.dumps(gold,ensure_ascii=False,indent=2)+'\n').encode(),'host-policy.json':old['host-policy.json'],'preparation.json':(json.dumps(prep,ensure_ascii=False,indent=2)+'\n').encode()}
oldgold=json.loads(old['gold.json']);assert [(g['id'],g['label'],g['answerability'],g['family']) for g in oldgold['records']]==[(g['id'],g['label'],g['answerability'],g['family']) for g in gold['records']]
assert json.loads(old['corpus.json'])['cases']==corpus['cases']
for name,data in new.items():
 diff=''.join(difflib.unified_diff(old[name].decode().splitlines(True),data.decode().splitlines(True),fromfile='initial/'+name,tofile='final/'+name))
 with (N/('pre-freeze-'+name+'.diff')).open('x') as f:f.write(diff)
 with (R/name).open('wb') as f:f.write(data);f.flush();os.fsync(f.fileno())
with (N/'pre-freeze-scope-cleanup.json').open('x') as f:
 json.dump({'pre_M1':True,'observations':0,'labels_answerability_questions_families_unchanged':True,'original_bytes_preserved':True,'reason':'Avoid gold-like target-specific hints in visible context; provide accurate per-group witnesses and avoid an unasked extra fact requirement. All are author/schema/scope review before any semantic observation, not outcome-dependent repair.','source_raw_normalized_unchanged':True,'old':{k:hashlib.sha256(v).hexdigest() for k,v in old.items()},'new':{k:hashlib.sha256(v).hexdigest() for k,v in new.items()}},f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
print('Pre-freeze scope cleanup preserved original bytes and all48 labels; observations0.')
