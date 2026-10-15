#!/usr/bin/env python3
"""Read-only snapshot adapter. Curated extraction is evidence data, never score labels."""
import argparse,hashlib,json,os,sys,time
from pathlib import Path

def scope(value):return {k:{'kind':'values','values':[v]} for k,v in value.items()}
def host_id(case):return 'harness:v2:captured-origin-manifest:'+case['id']
def host_facts(corpus,case):
    if 'e2e_host_facts' not in case:return {}
    name=case['id'].split('-')[1]
    groups={corpus['sources'][s]['origin_group'] for s in case['source_ids']}
    # We only know that two records of the SAME captured document are duplicates.
    docs={corpus['sources'][s]['document_id'] for s in case['source_ids']}
    assert len(docs)==1 and len(groups)==1
    actual={name+'.captured_origin_count':str(len(groups)),name+'.independent_corroboration_established':'false'}
    assert actual==case['e2e_host_facts'];return actual

def records(corpus,case):
    out=[]
    for slot,sid in enumerate(case['source_ids']):
        s=corpus['sources'][sid]
        visible={k:s[k] for k in ('quote','context_summary','document_id','url','locator','source_version','origin_group','scope','synthetic_mutation')}
        out.append({'id':f"snapshot:{case['id']}:{slot}",'source':sid,'observation':json.dumps(visible,sort_keys=True),
          'facts':s['reference_facts'],'acquisition_metadata':{'observed_at_unix_seconds':s['retrieved_at_unix_seconds'],'retrieved_at_unix_seconds':s['retrieved_at_unix_seconds'],'scope':scope(s['scope']),'claimed_authority_class':'primary' if not s['synthetic_mutation'] else 'untrusted'}})
    facts=host_facts(corpus,case)
    if facts:
        at=max(s['acquisition_metadata']['retrieved_at_unix_seconds'] for s in out)
        out.append({'id':'owner:'+case['id'],'source':host_id(case),'observation':json.dumps({'surface':'harness-owned captured snapshot manifest; no external independence inference','source_ids':case['source_ids'],'origin_groups':sorted({corpus['sources'][s]['origin_group'] for s in case['source_ids']})}),'facts':facts,'acquisition_metadata':{'observed_at_unix_seconds':at,'retrieved_at_unix_seconds':at,'scope':scope(case['required_scope']),'claimed_authority_class':'reference_host'}})
    return out

def config(corpus,case,corpus_path,journal):
    sources={sid:{'authority_class':'primary','max_age_seconds':86400,'scope':scope(corpus['sources'][sid]['scope'])} for sid in case['source_ids'] if not corpus['sources'][sid]['synthetic_mutation']}
    if host_facts(corpus,case):sources[host_id(case)]={'authority_class':'reference_host','max_age_seconds':86400,'scope':scope(case['required_scope'])}
    keys=sorted({case['lookup_key']}|{k for s in records(corpus,case) for k in s['facts']})
    admission={'authority_ranks':{'primary':20,'reference_host':20,'untrusted':0},'minimum_authority_class':'primary','required_scope':scope(case['required_scope']),'evaluation_time_unix_seconds':max(d['retrieved_at_unix_seconds'] for d in corpus['documents'].values())+1,'sources':sources}
    capabilities=[]
    for slot,record in enumerate(records(corpus,case)):
        cap={'id':'realworld-v2-source-'+str(slot),'kind':'external_command','program':sys.executable,'args':[str(Path(__file__).resolve()),'--corpus',str(Path(corpus_path).resolve()),'--case',case['id'],'--slot',str(slot),'--journal',str(Path(journal).resolve())],'read_only':True,'supported_fact_keys':sorted(set(record['facts'])|{case['lookup_key']}),'timeout_ms':5000,'max_response_bytes':65536,'admission':admission}
        capabilities.append(cap)
    return {'schema_version':'reason-config-v1','resolution':{'investigation':{'capabilities':capabilities,'max_actions':4,'max_no_progress_rounds':2,'max_rounds':3,'max_targets':3,'planner_max_tokens':256}}}

def main():
    p=argparse.ArgumentParser();p.add_argument('--corpus',type=Path,required=True);p.add_argument('--case',required=True);p.add_argument('--journal',type=Path,required=True);p.add_argument('--slot',type=int,required=True);a=p.parse_args()
    data=a.corpus.read_bytes();corpus=json.loads(data);case=next(c for c in corpus['cases'] if c['id']==a.case)
    request=json.load(sys.stdin)
    assert request.get('schema_version')=='reason-investigation-external-resolver-request-v1'
    assert request.get('adapter_id')=='investigation_external_command_v1'
    assert request.get('request',{}).get('target',{}).get('kind')=='investigation_question'
    evidence=[records(corpus,case)[a.slot]]
    response={'schema_version':'reason-external-resolver-response-v1','contribution':{'kind':'acquired_evidence','evidence':evidence},'cost':{'calls':1,'added_tokens':sum(len(e['observation'])//4 for e in evidence),'elapsed_ms':0}}
    with a.journal.open('a') as f:
        f.write(json.dumps({'at_unix':time.time(),'case':a.case,'corpus_sha256':hashlib.sha256(data).hexdigest(),'request':request,'response':response},sort_keys=True)+'\n');f.flush();os.fsync(f.fileno())
    json.dump(response,sys.stdout,separators=(',',':'))
if __name__=='__main__':main()
