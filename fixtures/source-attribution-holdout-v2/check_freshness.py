#!/usr/bin/env python3
import json, re
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
CUR=ROOT/'fixtures/source-attribution-holdout-v2/manifest.json'
PRIOR=sorted((ROOT/'fixtures').glob('source-attribution-development-v*/manifest.json'))+[ROOT/'fixtures/source-attribution-holdout-v1/manifest.json']
ENTITIES=['Copper Finch','Delta Mosaic','Echo Harbor','Fable Circuit','Glacier Note','Harbor Mint','Ivory Relay','Koi Matrix','Lantern Dock','Meadow Arc','Northwind Page','Opal Cache','Opal Search','Prairie Desk','Quill Export','River Slate','Silver Gate','Tundra Plan','Umber Trace']

def texts(m):
    out=[]
    for c in m['cases']:
        out += [c.get('task',''), c.get('frozen_statement',''), c.get('hard_verified_text','')]
        out += [t.get('target_question','') for t in c.get('targets',[])]
        for s in c.get('sources',[]): out += [s.get('observation',''),s.get('excerpt','')]
    return [x for x in out if x]

def toks(s): return re.findall(r"[A-Za-z0-9]+(?:[-'][A-Za-z0-9]+)?",s.lower())
def ngrams(s,n=8):
    t=toks(s); return {' '.join(t[i:i+n]) for i in range(max(0,len(t)-n+1))}

cur=json.loads(CUR.read_text())
assert len(cur['cases'])==18
fam={k:sum(1 for c in cur['cases'] if c['family']==k) for k in ['positive','safety','mixed']}
assert fam=={'positive':6,'safety':9,'mixed':3}, fam
prior=[json.loads(p.read_text()) for p in PRIOR]
prior_ids={c['id'] for m in prior for c in m['cases']}
cur_ids=[c['id'] for c in cur['cases']]
assert len(cur_ids)==len(set(cur_ids)) and not (set(cur_ids)&prior_ids)
prior_tasks={c['task'] for m in prior for c in m['cases']}
assert not ({c['task'] for c in cur['cases']} & prior_tasks)
prior_source_ids={s['source_id'] for m in prior for c in m['cases'] for s in c.get('sources',[])}
cur_source_ids={s['source_id'] for c in cur['cases'] for s in c.get('sources',[])}
assert not (cur_source_ids & prior_source_ids)
prior_exact=set(texts({'cases':[c for m in prior for c in m['cases']]}))
cur_exact=set(texts(cur))
assert not (cur_exact & prior_exact)
prior_ng=set()
for m in prior:
    for t in texts(m): prior_ng |= ngrams(t)
cur_ng=set()
for t in texts(cur): cur_ng |= ngrams(t)
overlap=cur_ng & prior_ng
assert not overlap, f'exact 8-token overlap: {sorted(overlap)[:5]}'
prior_blob='\n'.join(texts({'cases':[c for m in prior for c in m['cases']]})).lower()
for entity in ENTITIES:
    assert entity.lower() not in prior_blob, f'entity reused: {entity}'
print(f'PASS: {len(cur_ids)} fresh cases; families={fam}; prior_manifests={len(PRIOR)}; exact_8_token_overlap=0; entity_reuse=0')
