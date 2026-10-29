from pathlib import Path
import hashlib,json,sys,importlib.util
sha=lambda b:hashlib.sha256(b).hexdigest()
canon=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
receipt_path=Path('/receipt.json');receipt=json.loads(receipt_path.read_bytes())
assert sha(receipt_path.read_bytes())=='7a9eba8cffaab549aabde40995ba3d0d106c4b2a4b697ce2e0d992a2f8e61dc6'
repo=Path('/repo');normal=repo/'evaluation/cli-repair-paired-v1/normal_v1';parent=normal.parent
for name,digest in receipt['files'].items():assert sha((repo/name).read_bytes())==digest,name
sys.path[:0]=[str(normal),str(parent)]
def load(name,path):
 spec=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
scorer=load('frozen_semantic_scorer',normal/'scorer.py')
parent_scorer=load('frozen_parent_scorer',parent/'scorer.py')
driver=load('frozen_first_driver_validation_only',normal/'score_development_first.py')
deps=parent_scorer.frozen_dependencies(repo)
corpus_root=Path('/corpus');corpus=json.loads((corpus_root/'corpus.json').read_bytes());gold={g['id']:g for g in json.loads((corpus_root/'gold.json').read_bytes())['records']}
manifest=json.loads((corpus_root/'manifest.json').read_bytes());assert sha((corpus_root/'manifest.json').read_bytes())==receipt['consumed_M1_sha256']
for name,h in manifest['files'].items():assert sha((corpus_root/name).read_bytes())==h
for sid,s in corpus['sources'].items():
 d=corpus['documents'][s['document_id']]
 scorer.check_source(s,(corpus_root/d['raw_path']).read_bytes(),(corpus_root/s['normalized_path']).read_bytes())
obs=Path('/observations');reviews=Path('/reviews');binding=sha(canon(json.loads((obs/'verified-run.json').read_bytes())))
assert binding=='848cd54a46806890c0d7a4b95bf62c30cecbae9f51406a7ec77673f367a985d8'
packets=sorted(obs.glob('*.review-packet.json'));assert len(packets)==48
assert {p.name for p in reviews.iterdir()}=={p.name.replace('.review-packet','') for p in packets}
results=[]
for pp in packets:
 packet=json.loads(pp.read_bytes());rid=packet['row_id'];o=json.loads((obs/(rid+'.json')).read_bytes());r=json.loads((reviews/(rid+'.json')).read_bytes());g=gold[o['case_id']+'/e2e_normal'];record={**g,'sha256':sha(canon(g))}
 assert r['observed_operation_preserved']==o['operation']
 try:
  driver.verify_review_binding(r,o,binding,'/root/paired_evaluation_independent')
  # This frozen function internally returns row arithmetic. Discard its entire
  # return value; never print, store, inspect or aggregate metrics here.
  scorer.validate_review(record,o,r,corpus['sources'],deps)
  results.append({'row_id':rid,'valid':True})
 except Exception as e:results.append({'row_id':rid,'valid':False,'error':type(e).__name__+':'+str(e)})
print(json.dumps({'identity':'cli-normal-run40873570944-semantic-validation-only-v1','runtime_files_checked':len(receipt['files']),'execution_provenance_sha256':binding,'receipt_sha256':sha(receipt_path.read_bytes()),'normal_scorer_sha256':sha((normal/'scorer.py').read_bytes()),'parent_scorer_sha256':sha((parent/'scorer.py').read_bytes()),'arithmetic_sha256':sha((repo/'evaluation/engine073-answer-metrics-development-v1/claim_metrics.py').read_bytes()),'display_binding_sha256':sha((repo/'evaluation/engine073-answer-metrics-development-v1/display_binding.py').read_bytes()),'reviewer':'/root/paired_evaluation_independent','rows':results,'valid':all(r['valid'] for r in results),'row_arithmetic_return_values_discarded':True,'aggregate_scoring_executed':False,'scorer_main_called':False,'network':'none','secrets_passed':False},indent=2))
raise SystemExit(0 if all(r['valid'] for r in results) else 1)
