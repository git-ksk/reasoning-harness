#!/usr/bin/env python3
"""Assemble only declared runtime inputs, never credentials or local home state."""
import argparse,shutil
from pathlib import Path

def main():
    p=argparse.ArgumentParser();p.add_argument('--repo',type=Path,required=True);p.add_argument('--bin-dir',type=Path,required=True);p.add_argument('--context',type=Path,required=True);a=p.parse_args()
    root=a.context/'root';root.mkdir()
    for name in ['engine073-realworld-evaluator-v2','engine073-answer-metrics-development-v1','engine07-quality-gates-v1','engine07-realworld-runner-v1','engine07-realworld-operations-v1']:
        source=a.repo/'evaluation'/name;dest=root/'repo/evaluation'/name
        shutil.copytree(source,dest,ignore=shutil.ignore_patterns('__pycache__','*.pyc'))
    target=root/'repo/.github/workflows';target.mkdir(parents=True)
    shutil.copyfile(a.repo/'.github/workflows/engine073-realworld-independent-v2.yml',target/'engine073-realworld-independent-v2.yml')
    shutil.copytree(a.bin_dir,root/'eval/bin')
if __name__=='__main__':main()
