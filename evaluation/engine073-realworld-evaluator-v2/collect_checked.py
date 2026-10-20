#!/usr/bin/env python3
"""Root-in-container collection: absent/inaccessible plan is a typed failure."""
import argparse,json,sys
from pathlib import Path
import collect

def main():
    p=argparse.ArgumentParser();p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--provider',required=True);p.add_argument('--lane',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    # Do not use exists()/shell test as a successful skip: opening must succeed.
    plan=json.loads((a.lane/'plan.json').read_bytes())
    if plan.get('provider')!=a.provider:raise ValueError('collector_provider_mismatch')
    sys.argv=[sys.argv[0],'--corpus-dir',str(a.corpus_dir),'--provider',a.provider,'--lane',str(a.lane),'--output',str(a.output)]
    collect.main()
if __name__=='__main__':main()
