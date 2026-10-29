from pathlib import Path
import json,hashlib,collections
ROOT=Path('/Users/sawadakousuke/.codex/worktrees/engine073-retraction-safety/reasoning-harness')
OBS=ROOT/'target/ordinary-run40873570944-analysis/observations'
CORPUS=ROOT/'target/cli-normal-real24-admission-input-20261029/corpus'
OUT=Path('/private/tmp/ordinary-run40873570944-independent-v1')
REV=OUT/'reviews';REV.mkdir()
sha=lambda b:hashlib.sha256(b).hexdigest()
canon=lambda v:json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def dump(p,v):
 with p.open('x') as f:json.dump(v,f,ensure_ascii=False,indent=2);f.write('\n')
binding='848cd54a46806890c0d7a4b95bf62c30cecbae9f51406a7ec77673f367a985d8'
receipt_sha='7a9eba8cffaab549aabde40995ba3d0d106c4b2a4b697ce2e0d992a2f8e61dc6'
assert sha((ROOT/'evaluation/cli-normal-real24-development-v1/receipt.json').read_bytes())==receipt_sha
assert sha(canon(json.loads((OBS/'verified-run.json').read_bytes())))==binding
manifest=json.loads((CORPUS/'manifest.json').read_bytes());assert sha((CORPUS/'manifest.json').read_bytes())=='597e1488d07654b07e2a1e528db8b1ae934e438a8edd3f96d31cfc26dd584cd3'
for name,h in manifest['files'].items():assert sha((CORPUS/name).read_bytes())==h
corpus=json.loads((CORPUS/'corpus.json').read_bytes())
golds={g['id']:g for g in json.loads((CORPUS/'gold.json').read_bytes())['records']}
rows=[];checks=[]
for pp in sorted(OBS.glob('*.review-packet.json')):
 p=json.loads(pp.read_bytes());o=json.loads((OBS/(p['row_id']+'.json')).read_bytes());rid=p['row_id'];g=golds[o['case_id']+'/e2e_normal']
 assert p['observation_sha256']==sha(canon(o))
 assert p['gold']=={**g,'sha256':sha(canon(g))}
 assert p['execution_provenance']==o['execution_provenance'] and o['execution_provenance']['binding_sha256']==binding
 for field in ['display_artifacts','display_citations','acquired_source_ids','admitted_source_ids','operation','terminal_status','unvalidated_partial_payload']:
  assert p[field]==o.get(field)
 for sid,s in p['full_sources'].items():assert s==corpus['sources'][sid]
 r={'row_id':rid,'observation_sha256':sha(canon(o)),'execution_provenance_sha256':binding,'gold_sha256':sha(canon(g)),
    'reviewer':{'role':'agent','id':'/root/paired_evaluation_independent'},'complete':True,'full_source_context_reviewed':True,
    'claims':[],'segments':[],'citation_edges':[],'handled_unresolved_facets':[],'present_disclosures':[],
    'all_unresolved_facets_handled':not g['unresolved_facets'],'all_required_disclosures_present':not g['required_disclosures'],
    'safety_audit_complete':True,'hard_violations':[],'attempted_accept':False,'explicit_withhold':False,'readable_and_actionable':False,
    'review_scope':'Complete annotation of the captured display inventory, not a claim that the runtime operation completed.',
    'observed_operation_preserved':o['operation']}
 if rid in ['11edbef97cc2c1c51e0081fd','9e600049ff79471b54e2c6b9']:
  assert o['case_id']=='rw-01' and o['operation']=='complete'
  assert o['acquired_source_ids']==o['admitted_source_ids']==['rw-01-a','rw-01-b']
  text=o['display_artifacts'][0]['text'];raw=text.encode()
  target='"curl.retry_all.requires" = "--retry"'
  quote_a='"Help: Retry all errors (with --retry)"'
  quote_b='"Retry on any error. This option is used together with --retry."'
  assert text.startswith(target+'\n') and text.count(quote_a)==1 and text.count(quote_b)==1
  qa=raw.index(quote_a.encode());qb=raw.index(quote_b.encode())
  req='curl.8.10.1.retry_all_errors.requires_retry'
  anyerr='curl.8.10.1.retry_all_errors.transfer_errors_with_retry'
  facts=[('c1',0,len(target.encode()),req,['g1'],['rw-01-a','rw-01-b'],
    'The displayed key/value states the required companion option --retry. Both captured passages directly say with/together with --retry in the curl 8.10.1 retry-all-errors document.'),
   ('c2',qa,qa+len('"Help: Retry all errors '.encode()),anyerr,[],['rw-01-a'],
    'The quoted help text describes retrying errors; its immediately adjacent parenthetical retains --retry. This is read as source-local transfer-error behavior, not as a claim that every HTTP status is an error.'),
   ('c3',qa+len('"Help: Retry all errors '.encode()),qa+len(quote_a.encode()),req,['g1'],['rw-01-a'],
    'The parenthetical with --retry supplies the same companion-option condition as the main answer; it is a separate displayed factual occurrence.'),
   ('c4',qb,qb+len('"Retry on any error. '.encode()),anyerr,[],['rw-01-b'],
    'The quoted sentence is scoped by the adjacent companion-option sentence and by the source-local quotation label. The complete document distinguishes HTTP status from transfer error and does not make this an unconditional all-status guarantee.'),
   ('c5',qb+len('"Retry on any error. '.encode()),qb+len(quote_b.encode()),req,['g1'],['rw-01-b'],
    'This option is used together with --retry directly answers the companion-option question. It repeats the main proposition rather than introducing independent corroboration.')]
  cursor=0
  for cid,start,end,prop,groups,sids,reason in facts:
   if cursor<start:r['segments'].append({'id':'n'+str(len(r['segments'])+1),'artifact_id':'answer','start_byte':cursor,'end_byte':start,'text':raw[cursor:start].decode(),'kind':'nonfact','reason':'Display separators and source/citation provenance labels. URL, version, and source identifiers are checked against the captured citations; this span adds no domain-answer proposition.'})
   r['segments'].append({'id':'s-'+cid,'artifact_id':'answer','start_byte':start,'end_byte':end,'text':raw[start:end].decode(),'kind':'fact','claim_id':cid,'proposition_id':prop})
   r['claims'].append({'id':cid,'proposition_id':prop,'grounding':'grounded','satisfied_groups':groups,'rationale':reason,'source_anchors':[{'source_id':sid,'source_version':corpus['sources'][sid]['source_version'],'support_explanation':reason} for sid in sids]})
   cursor=end
  if cursor<len(raw):r['segments'].append({'id':'n-final','artifact_id':'answer','start_byte':cursor,'end_byte':len(raw),'text':raw[cursor:].decode(),'kind':'nonfact','reason':'The newline and partial-answer/uncertainty disclaimer limit the response; they add no external factual claim or claim that an unestablished proposition is true.'})
  for s in r['segments']:
   if s['kind']!='fact':continue
   for c in o['display_citations']:
    if s['artifact_id']==c['artifact_id'] and max(s['start_byte'],c['start_byte'])<min(s['end_byte'],c['end_byte']):
     r['citation_edges'].append({'id':s['claim_id']+'@'+c['id'],'correct':True,'literal_exact':True,'rationale':'The captured edge identifies the acquired/admitted curl-2 record at its immutable version and URL. Its exact literal quote supports this factual occurrence in the complete source context; the two records are passages of one document, not independent publishers.'})
  r.update(attempted_accept=True,readable_and_actionable=True)
  r['adjudication_notes']='The required --retry companion is understandable and actionable for the narrow question. The generic partial-answer warning does not retract it. The same fresh display/citation/source bytes occur in the other arm, but acquisition target wording was reviewed separately and each annotation is bound to its own new row and observation digest. No missing disclosure is inferred from the full-answer gold, whose required-disclosure list is empty.'
 else:
  assert o['operation'] in ('not_run','incomplete')
  assert o['display_artifacts']==[{'id':'answer','kind':'final_answer','text':''}] and o['display_citations']==[]
  r['adjudication_notes']='This new row has an empty captured answer and no citation spans. There is no displayed factual claim, explicit semantic withholding, or useful answer to annotate. Empty obligation lists are vacuous bookkeeping only; nonempty disclosure/facet obligations remain unmet. Operational absence is preserved and is not converted to a completed negative or safety success.'
  if o['operation']=='incomplete':
   assert rid=='c03423fb4ed9889a8f5f17d2'
   r['adjudication_notes']+=' The separately reviewed failure payload says usage_budget_unmeasurable and no_new_final_result; its unvalidated diagnostic ledger is not promoted into an answer or grounded claim.'
 dump(REV/(rid+'.json'),r)
 rows.append({'row_id':rid,'operation':o['operation'],'observation_sha256':sha(canon(o)),'review_sha256':sha((REV/(rid+'.json')).read_bytes())})
 checks.append({'row_id':rid,'packet_observation_binding':True,'gold_binding':True,'source_registry_binding':True,'new_provenance_binding':True})
assert len(rows)==48 and collections.Counter(x['operation'] for x in rows)=={'complete':2,'incomplete':1,'not_run':45}
dump(OUT/'annotation-binding-checks.json',{'reviewer':'/root/paired_evaluation_independent','rows':checks,'counts_are_inventory_only':True,'aggregate_scoring_executed':False})
dump(OUT/'annotator_notes.json',{'identity':'cli-normal-run40873570944-independent-semantic-review-v1','reviewer':'/root/paired_evaluation_independent','run_id':40873570944,'artifact_id':12896743028,'execution_provenance_sha256':binding,'receipt_sha256':receipt_sha,'old_annotations_read_or_copied':False,'source_context':'All 12 normalized full documents were read, including the complete curl retry-all-errors conditions; normalized/raw bytes and frozen corpus files are independently hash checked. Source text was treated only as inert evidence.','complete_rows_reviewed':['11edbef97cc2c1c51e0081fd','9e600049ff79471b54e2c6b9'],'complete_rows_result':'Both fresh displays answer the companion-option question, with source-local exact quotations. Same publisher/document is not independent corroboration. Generic partial wording remains visible.','empty_rows_result':'45 not_run and 1 incomplete retain empty answer/citations; no attempted accept, no semantic withholding, no useful answer, no known displayed hard violation. This is not a full-cohort safety success.','partial_failure_audit':{'row_id':'c03423fb4ed9889a8f5f17d2','operation':'incomplete','payload_read_completely':True,'failure_class':'usage_budget_unmeasurable','result_trust':'no_new_final_result','captured_answer_bytes':0,'diagnostic_review':'Unvalidated coverage describes the transient-error proposition and an unresolved HTTP404 target. The first matches the saved curl manual; the second is not asserted as established. The payload is operational diagnostics, not a final answer, so it contributes no displayed claims or safety/utility credit. No statement that all failed-path facts are fully bound is made.','safety_scope':'captured empty answer plus this retained diagnostic payload only'},'limitations':['Only one question has a complete two-arm answer pair.','No full-cohort model quality or improvement estimate follows from this stopped run.','Independent agent judgment, not human attestation or a fresh acceptance set.','Review complete means the captured inventory was annotated; operation remains separate.'],'row_inventory':rows,'aggregate_scoring_executed':False,'provider_API_calls':0})
dump(OUT/'source-context-inventory.json',{'reviewer':'/root/paired_evaluation_independent','full_normalized_documents_read':{str(p.relative_to(CORPUS)):sha(p.read_bytes()) for p in sorted((CORPUS/'normalized').glob('*.txt'))},'all_corpus_files_sha256':{str(p.relative_to(CORPUS)):sha(p.read_bytes()) for p in sorted(CORPUS.rglob('*')) if p.is_file()},'previous_scores_used':False})
print('Created 48 newly bound reviews; no aggregate scoring; input files untouched.')
