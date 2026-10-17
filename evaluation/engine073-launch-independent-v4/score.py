#!/usr/bin/env python3
"""One immutable first-result evaluation; never invokes Engine or providers."""
import hashlib, json
from pathlib import Path
ROOT = Path(__file__).resolve().parent

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()

def main():
    manifest = json.loads((ROOT / 'manifest.json').read_text())
    for name, expected in manifest['frozen_files'].items():
        if sha(ROOT / name) != expected: raise RuntimeError('Frozen file mismatch: ' + name)
    invocation = json.loads((ROOT / 'first-invocation.json').read_text())
    cases = json.loads((ROOT / 'cases.json').read_text())['cases']
    expected_ids = {c['id'] for c in cases}
    rows, errors = {}, []
    for index, line in enumerate((ROOT / 'first.stdout.jsonl').read_text().splitlines(), 1):
        try:
            row = json.loads(line)
            if not isinstance(row, dict): raise ValueError('row not object')
            cid = row.get('id')
            if not isinstance(cid, str) or cid not in expected_ids: raise ValueError('foreign/missing id')
            if cid in rows: raise ValueError('duplicate id ' + cid)
            if row.get('engine_version') != manifest['candidate']['package_version']: raise ValueError('unexpected engine_version')
            relation = row['effective_v17']['relation_scope']
            disposition = row['v30']['disposition']
            if relation not in ('requested_relation', 'different_relation', 'relation_absent', 'unresolved'): raise ValueError('unexpected relation scope')
            if disposition not in ('relevant', 'irrelevant', 'ambiguous'): raise ValueError('unexpected disposition')
            if not isinstance(row.get('v29'), dict): raise ValueError('missing v29 diagnostic')
            rows[cid] = row
        except (ValueError, TypeError, KeyError) as exc:
            errors.append({'line': index, 'error': str(exc)})
    missing = sorted(expected_ids - set(rows))
    if missing: errors.append({'missing_ids': missing})
    if invocation.get('state') != 'finished' or invocation.get('returncode') != 0:
        errors.append({'invocation': invocation.get('state'), 'returncode': invocation.get('returncode')})
    stderr_bytes = (ROOT / 'first.stderr.txt').stat().st_size
    if stderr_bytes: errors.append({'stderr_bytes': stderr_bytes})
    scored=[]
    for case in cases:
        row=rows.get(case['id'])
        if row is None:
            scored.append({'id':case['id'], 'pair_id':case['pair_id'], 'polarity':case['polarity'], 'axis':case['axis'], 'status':'incomplete'})
            continue
        observed={'v17_requested_relation':row['effective_v17']['relation_scope']=='requested_relation','v30_relevant':row['v30']['disposition']=='relevant'}
        checks={key:observed[key]==value for key,value in case['expected'].items()}
        scored.append({'id':case['id'],'pair_id':case['pair_id'],'polarity':case['polarity'],'axis':case['axis'],'expected':case['expected'],'observed':observed,'effective_relation_scope':row['effective_v17']['relation_scope'],'v30_disposition':row['v30']['disposition'],'v30_reasons':row['v30'].get('reasons'),'checks':checks,'status':'pass' if all(checks.values()) else 'fail'})
    negatives=[r for r in scored if r['polarity']=='negative' and r['status']!='incomplete']
    positives=[r for r in scored if r['polarity']=='positive' and r['status']!='incomplete']
    summary={'identity':manifest['identity'],'candidate_source_sha':manifest['candidate']['source_sha'],'manifest_sha256':sha(ROOT/'manifest.json'),'outcome':'PASS' if not errors and all(r['status']=='pass' for r in scored) else 'FAIL','planned_cases':len(cases),'observed_cases':len(rows),'passed':sum(r['status']=='pass' for r in scored),'failed':sum(r['status']=='fail' for r in scored),'incomplete':sum(r['status']=='incomplete' for r in scored),'positive_passed':sum(r['status']=='pass' for r in positives),'negative_passed':sum(r['status']=='pass' for r in negatives),'negative_v17_requested_relation_violations':sum(r['observed']['v17_requested_relation'] for r in negatives),'negative_v30_relevant_violations':sum(r['observed']['v30_relevant'] for r in negatives),'positive_v17_withheld':sum(not r['observed']['v17_requested_relation'] for r in positives),'positive_v30_withheld':sum(not r['observed']['v30_relevant'] for r in positives),'operational_errors':errors,'raw':{'stdout_sha256':sha(ROOT/'first.stdout.jsonl'),'stderr_sha256':sha(ROOT/'first.stderr.txt'),'invocation_sha256':sha(ROOT/'first-invocation.json')},'provider_calls':0,'retries':0,'scope':'Independent synthetic candidate API check only; not #531 real-world or release/CLI acceptance.'}
    with (ROOT/'first-results.json').open('x') as f: json.dump({'summary':summary,'cases':scored},f,indent=2);f.write('\n')
    with (ROOT/'first-summary.json').open('x') as f: json.dump(summary,f,indent=2);f.write('\n')
    print(json.dumps(summary,indent=2))

if __name__ == '__main__': main()
