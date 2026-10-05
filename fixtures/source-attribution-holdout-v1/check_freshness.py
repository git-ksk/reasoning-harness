import json, re
from pathlib import Path
root=Path(__file__).resolve().parents[2]
fresh=json.loads((root/'fixtures/source-attribution-holdout-v1/manifest.json').read_text())
old=[]
for path in sorted((root/'fixtures').glob('source-attribution-development-v*/manifest.json')):
    old.extend(json.loads(path.read_text())['cases'])
def strings(x):
    if isinstance(x,str): yield x
    elif isinstance(x,list):
        for y in x: yield from strings(y)
    elif isinstance(x,dict):
        for y in x.values(): yield from strings(y)
def tokens(s): return re.findall(r'[A-Za-z0-9]+',s.lower())
def windows(case,n=8):
    result=set()
    for value in strings(case):
        t=tokens(value); result.update(tuple(t[i:i+n]) for i in range(len(t)-n+1))
    return result
old_windows=set().union(*(windows(c) for c in old))
old_ids={c['id'] for c in old}; old_tasks={c['task'] for c in old}
assert len(fresh['cases'])==18
assert {k:sum(c['family']==k for c in fresh['cases']) for k in ('positive','safety','mixed')}=={'positive':6,'safety':9,'mixed':3}
assert not (old_ids & {c['id'] for c in fresh['cases']})
assert not [c['id'] for c in fresh['cases'] if c['task'] in old_tasks]
assert not [(c['id'], sorted(windows(c)&old_windows)) for c in fresh['cases'] if windows(c)&old_windows]
print('freshness PASS: 18 cases; no development ID/task/exact 8-token reuse')
