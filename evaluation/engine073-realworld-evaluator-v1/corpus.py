"""Prospective independent corpus contract; no real cases are authored here."""
import hashlib,json,re
from pathlib import Path
from scorer import require,sha,keyed,check_source

FORBIDDEN_PRODUCTS={'aws','lambda','s3','python','node','nodejs','kubernetes','k8s'}
def normalize(raw,kind):
    text=raw.decode('utf8')
    if kind=='utf8-plain-whitespace-v1':return ' '.join(text.split()).encode()
    raise ValueError('unfrozen_normalization')
def validate(root,expected_manifest=None):
    root=Path(root).resolve();blob=(root/'corpus.json').read_bytes();data=json.loads(blob)
    require(data.get('identity')=='engine073-realworld-independent-v1','corpus_identity')
    cases=keyed(data['cases']);require(len(cases)==24,'independent24_required')
    require(len({c['question'] for c in cases.values()})==24,'unique_questions_required')
    docs=data['documents'];sources=data['sources'];require(isinstance(docs,dict) and isinstance(sources,dict),'source_maps_required')
    require(len(docs)>=12 and len({d['product'].lower() for d in docs.values()})>=6,'minimum12documents6products')
    for did,doc in docs.items():
        require(re.fullmatch(r'[A-Za-z0-9_-]+',did) is not None,'unsafe_document_id')
        require(doc['product'].lower() not in FORBIDDEN_PRODUCTS,'development_product_overlap')
        require(doc['url'].startswith('https://') and doc['publisher'] and doc['license_or_capture_basis'],'source_provenance_missing')
        raw=(root/doc['raw_path']).resolve();text=(root/doc['normalized_path']).resolve()
        require(raw.is_relative_to(root) and text.is_relative_to(root),'source_path_escape')
        require(sha(raw.read_bytes())==doc['raw_sha256'] and sha(text.read_bytes())==doc['normalized_sha256'],'document_hash_mismatch')
        require(normalize(raw.read_bytes(),doc['normalization'])==text.read_bytes(),'document_normalization_mismatch')
        require(type(doc['retrieved_at_unix_seconds']) is int,'retrieval_time_missing')
    for sid,s in sources.items():
        require(s['source_id']==sid and s['document_id'] in docs,'source_identity')
        doc=docs[s['document_id']];check_source(s,(root/doc['raw_path']).read_bytes(),(root/doc['normalized_path']).read_bytes())
        require(s['url']==doc['url'] and s['normalized_path']==doc['normalized_path'],'source_document_binding')
        require(s['raw_sha256']==doc['raw_sha256'] and s['normalized_sha256']==doc['normalized_sha256'],'source_document_hash_binding')
        require(s['synthetic_mutation'] is False and s['context_summary'] and s['locator'],'real_source_context_required')
        require(s['origin_group'] and isinstance(s['scope'],dict) and isinstance(s['reference_facts'],dict),'source_scope_origin_facts_required')
        require(s['retrieved_at_unix_seconds']==doc['retrieved_at_unix_seconds'],'retrieval_binding')
    for c in cases.values():
        require(set(c)=={'id','question','helper_question','source_ids','required_scope','lookup_key','question_as_of','question_scope_note'},'model_visible_case_schema')
        require(len(c['source_ids'])==2 and len(set(c['source_ids']))==2 and set(c['source_ids'])<=sources.keys(),'two_distinct_source_records_required')
        require(c['question'] and c['helper_question'] and c['lookup_key'] and c['question_as_of'] and c['question_scope_note'],'question_contract_missing')
    gold_blob=(root/'gold.json').read_bytes();gold=json.loads(gold_blob)
    require(gold['corpus_sha256']==sha(blob),'gold_corpus_binding')
    require(gold['author']['role']=='agent' and gold['source_review']['role']=='agent' and gold['source_review']['complete'] is True,'independent_source_review_required')
    records=keyed(gold['records']);expected={cid+'/'+track for cid in cases for track in ['e2e_normal','helper_opt_in']}
    require(set(records)==expected,'complete_gold_track_inventory')
    for track in ['e2e_normal','helper_opt_in']:
        selected=[v for k,v in records.items() if k.endswith('/'+track)]
        require(sum(x['label']=='positive' for x in selected)>=8 and sum(x['label']=='negative' for x in selected)>=1,'insufficient_clear_gold_classes')
    families={'compatible','conflict','scope','revision_or_history','duplicate_or_lineage','missing_or_unrelated','wrong_target','untrusted_control'}
    for track in ['e2e_normal','helper_opt_in']:
        require(families<={g['family'] for g in records.values() if g['track']==track},'required_family_coverage')
    for key,g in records.items():
        require(g['case_id'] in cases and g['track'] in ['e2e_normal','helper_opt_in'] and key==g['case_id']+'/'+g['track'],'misbound_gold_case')
        require(g['label'] in ['positive','negative','ambiguous'] and g['answerability'] in ['full','partial','none'],'gold_labels_missing')
        keyed(g['groups']);require(isinstance(g['unresolved_facets'],list) and isinstance(g['required_disclosures'],list),'gold_facets_missing')
        require(g['full_source_rationale'] and g['scope_time_origin_review'],'gold_context_review_missing')
    controls=json.loads((root/'host-policy.json').read_text());require(set(controls)==set(cases),'host_policy_inventory')
    for c in controls.values():require(type(c['controlled_reference_authorized']) is bool and c['authorization_rationale'],'host_authority_missing')
    if expected_manifest:validate_manifest(root,docs,expected_manifest)
    return data,gold,controls


def verify_provenance(root,receipt):
    root=Path(root);receipt=Path(receipt);m=json.loads(receipt.read_bytes());p=json.loads((root/'preparation.json').read_bytes())
    require(m['state']=='COMPLETE_FROZEN' and m['independent_implementation_review']['result']=='PASS','M0_not_complete_reviewed')
    require(p['M0_receipt_sha256']==sha(receipt.read_bytes()),'corpus_wrong_M0')
    require(m['semantic_reviewer_id']=='/root/engine07_realworld_independent' and p['author_id']==m['semantic_reviewer_id'],'independent_author_identity')
    require(p['author_role']=='independent_agent' and p['accuracy_cases_authored_before_M0']==0,'premature_or_nonindependent_authorship')
    require(p['authorship_started_unix_seconds']>=m['completed_unix_seconds'],'case_authorship_before_M0')
    return m


def validate_manifest(root,docs,expected_manifest):
    root=Path(root).resolve()
    m=(root/'manifest.json').read_bytes();require(sha(m)==expected_manifest,'M1_anchor_mismatch')
    inventory=json.loads(m)['files'];require(isinstance(inventory,dict) and inventory,'M1_inventory_empty')
    mandatory={'corpus.json','gold.json','host-policy.json','preparation.json'}|{d[k] for d in docs.values() for k in ['raw_path','normalized_path']}
    require(mandatory<=inventory.keys(),'M1_required_file_unbound')
    actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and p!=root/'manifest.json'}
    require(set(inventory)==actual and 'manifest.json' not in inventory,'M1_inventory_incomplete_or_extra')
    for name,digest in inventory.items():
        p=(root/name).resolve();require(p.is_relative_to(root) and sha(p.read_bytes())==digest,'M1_file_changed')
