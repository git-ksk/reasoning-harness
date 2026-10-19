"""Frozen corpus contract and observed native ModelRequest serialization shape; zero Engine/scorer calls."""
from pathlib import Path
import copy,hashlib,json,os,sys
N=Path(__file__).resolve().parent;R=Path('/private/tmp/engine073-realworld-independent-v1-input');S=Path('/private/tmp/engine073-complete-M0-independent-review-v1/runtime-code-v2/repo/evaluation/engine073-realworld-evaluator-v1')
sys.path.insert(0,str(S));from corpus import validate,verify_provenance
suffix=sys.argv[1];data,gold,policy=validate(R);verify_provenance(R,Path('/private/tmp/engine073-realworld-M0-v1-staging/receipt.json'))
raw=Path('/private/tmp/engine073-groq-actual-adapter-readiness-39205645436/engine073-groq-actual-adapter-readiness-39205645436/readiness/model.jsonl');template=json.loads(raw.read_text().splitlines()[0])['request']
checks=[]
for n,c in enumerate(data['cases']):
 visible=[{k:data['sources'][s][k] for k in ['quote','context_summary','scope','source_version','url','locator','origin_group','synthetic_mutation']} for s in c['source_ids']]
 packet={'question':c['helper_question'],'sources':visible};task=json.dumps(packet,ensure_ascii=False,separators=(',',':'),sort_keys=True)
 req=copy.deepcopy(template);req['task']=task;req['random_seed']=49200+n
 size=len(json.dumps(req,ensure_ascii=False,separators=(',',':'),sort_keys=True).encode())
 checks.append({'case_id':c['id'],'visible_packet_UTF8_bytes':len(task.encode()),'native_shape_ModelRequest_UTF8_bytes':size,'reserved_input_upper_bound':size+512,'within3584_request':size<=3584,'reference_facts_visible_to_helper':False,'gold_visible_to_models':False})
result={'scope':'STATIC_ONLY; no candidate Engine/model/scorer observation','schema_and_provenance_pass':True,'cases':24,'documents':len(data['documents']),'products':len({x['product'] for x in data['documents'].values()}),'checks':checks,'all_request_budgets_pass':all(c['within3584_request'] for c in checks),'native_request_shape_source_sha256':hashlib.sha256(raw.read_bytes()).hexdigest(),'gold_counts':{t:{lab:sum(g['track']==t and g['label']==lab for g in gold['records']) for lab in ['positive','negative','ambiguous']} for t in ['e2e_normal','helper_opt_in']},'family_counts':{t:{fam:sum(g['track']==t and g['family']==fam for g in gold['records']) for fam in sorted({g['family'] for g in gold['records']})} for t in ['e2e_normal','helper_opt_in']},'Engine_model_provider_actual_scorer_Oracle_calls':0}
with (N/('static-preflight-'+suffix+'.json')).open('x') as f:json.dump(result,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
print(json.dumps({'schema_and_provenance_pass':True,'all_request_budgets_pass':result['all_request_budgets_pass'],'max_request_bytes':max(c['native_shape_ModelRequest_UTF8_bytes'] for c in checks),'over_budget':[(c['case_id'],c['native_shape_ModelRequest_UTF8_bytes']) for c in checks if not c['within3584_request']],'gold_counts':result['gold_counts']},indent=2))
