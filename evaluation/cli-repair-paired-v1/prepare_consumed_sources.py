#!/usr/bin/env python3
"""Materialize immutable public source bytes and verify before any credentials."""
import urllib.parse
import argparse,hashlib,json,re,urllib.request
from pathlib import Path
from corpus import normalize
from development import INPUT_M1,consumed_corpus

def main():
    p=argparse.ArgumentParser();p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--M1-sha256',required=True);a=p.parse_args();root=a.corpus_dir.resolve()
    assert a.M1_sha256==INPUT_M1 and hashlib.sha256((root/'manifest.json').read_bytes()).hexdigest()==INPUT_M1
    corpus=json.loads((root/'corpus.json').read_text())
    for doc in corpus['documents'].values():
        raw=(root/doc['raw_path']).resolve();text=(root/doc['normalized_path']).resolve()
        if not raw.is_relative_to(root) or not text.is_relative_to(root):raise ValueError('source path escape')
        if not raw.exists():
            url=doc['immutable_capture_url'];parts=urllib.parse.urlparse(url)
            if parts.scheme!='https' or parts.hostname!='raw.githubusercontent.com' or not re.search(r'/[0-9a-f]{40}/',parts.path):raise ValueError('immutable public capture URL required')
            with urllib.request.urlopen(url,timeout=30) as response:
                if urllib.parse.urlparse(response.url).hostname!='raw.githubusercontent.com':raise ValueError('source redirect outside fixed host')
                data=response.read(4*1024*1024+1)
            if len(data)>4*1024*1024 or hashlib.sha256(data).hexdigest()!=doc['raw_sha256']:raise ValueError('captured source bytes changed')
            raw.parent.mkdir(parents=True,exist_ok=True)
            with raw.open('xb') as f:f.write(data)
        if not text.exists():
            text.parent.mkdir(parents=True,exist_ok=True)
            with text.open('xb') as f:f.write(normalize(raw.read_bytes(),doc['normalization']))
    consumed_corpus(root);print(json.dumps({'full_sources_verified':len(corpus['documents']),'model_provider_calls':0}))
if __name__=='__main__':main()
