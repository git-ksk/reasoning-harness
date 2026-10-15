#!/usr/bin/env python3
"""Verify public first-observation archives with zero model/API calls."""
import hashlib,json,re,subprocess,sys,tempfile,zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
EXPECTED={'v2':('107c05dc3f2ffe20ea2116db84a69e57e85a1389',38656022911), 'v3':('7432710f61d747595cb8498aa2dea9e44d2ff7fb',38659305389)}
secret=re.compile(rb'(?:gsk_[A-Za-z0-9]{20,}|AIza[A-Za-z0-9_-]{30,}|org_[A-Za-z0-9_-]{8,})')
for version,(commit,run) in EXPECTED.items():
 result=ROOT/f'evaluation/engine07-realworld-{version}-results'
 index=json.loads((result/'index.json').read_text())
 assert index['freeze_commit']==commit and index['run_id']==run and index['run_attempt']==1 and index['adoption']=='NO_GO'
 assert set(index['providers'])=={'mistral','google','groq'}
 with tempfile.TemporaryDirectory(prefix='engine07-first-results-') as directory:
  base=Path(directory)
  for provider,entry in index['providers'].items():
   archive=(result/entry['published_archive']).resolve();assert archive.is_relative_to(result.resolve())
   assert hashlib.sha256(archive.read_bytes()).hexdigest()==entry['published_zip_sha256']
   dest=base/provider;dest.mkdir()
   with zipfile.ZipFile(archive) as z:
    assert len(z.namelist())==len(set(z.namelist()))
    for name in z.namelist():
     target=(dest/name).resolve();assert target.is_relative_to(dest.resolve())
     data=z.read(name);assert not secret.search(data),'credential/account-like publication material'
    z.extractall(dest)
  command=[sys.executable,str(result/'verify.py'),'--raw-root',str(base),'--frozen-suite',str(ROOT/f'evaluation/engine07-realworld-{version}'),'--publication']
  checked=json.loads(subprocess.check_output(command,text=True,cwd=ROOT))
  assert checked==json.loads((result/'publication-verification.json').read_text())
  original=json.loads((result/'original-verification.json').read_text());assert original['verified'] and original['model_calls_performed']==0 and not original['publication_derivative']
  print(json.dumps({'version':version,'public_derivatives_verified':True,'normal_rows':144,'helper_rows':72,'model_calls_performed':0}))
