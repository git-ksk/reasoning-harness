#!/usr/bin/env python3
"""New offline development capture only; original observations remain immutable."""
import argparse,json,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
FROZEN=HERE.parent/'engine073-realworld-evaluator-v2'
sys.path.append(str(FROZEN))
from scorer import require,sha
from development import consumed_corpus,INPUT_M1,PURPOSE
import collect

def main():
    p=argparse.ArgumentParser();p.add_argument('--corpus-dir',type=Path,required=True);p.add_argument('--provider',choices=['mistral','google'],required=True);p.add_argument('--raw-root',type=Path,required=True);p.add_argument('--raw-manifest',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    for name,digest in json.loads((HERE/'dependency-sha256.json').read_bytes()).items():require(sha((FROZEN/name).read_bytes())==digest,'frozen_dependency_changed')
    require(Path(collect.bind_normal.__code__.co_filename).resolve()==HERE/'capture.py','wrong_capture_implementation')
    pins={'mistral':'9c49fb2b38e5f234f8c1b147c1ae6725b1d636f316b16ec434c64a222a245f55','google':'64e346484deaa3a2c72a037fb69e48035956e0463a3e8096bfcd6903bede09d7'}
    require(sha(a.raw_manifest.read_bytes())==pins[a.provider],'wrong_original_raw_manifest')
    manifest=json.loads(a.raw_manifest.read_bytes());actual={str(p.relative_to(a.raw_root)):sha(p.read_bytes()) for p in a.raw_root.rglob('*') if p.is_file()}
    require(actual==manifest['files'],'original_raw_changed')
    consumed_corpus(a.corpus_dir)
    plan=json.loads((a.raw_root/'lane/plan.json').read_bytes())
    require(plan['provider']==a.provider and plan['M1_sha256']==INPUT_M1 and plan['purpose']==PURPOSE,'wrong_development_lane')
    sys.argv=[sys.argv[0],'--corpus-dir',str(a.corpus_dir),'--provider',a.provider,'--lane',str(a.raw_root/'lane'),'--output',str(a.output)]
    collect.main()
    lineage={'identity':'engine073-realworld-development-capture-repair-v1','purpose':'CONSUMED_DIAGNOSTIC_NOT_ACCURACY_ACCEPTANCE','provider':a.provider,'raw_manifest_sha256':pins[a.provider],'original_capture_index_sha256':sha((a.raw_root/'observations/observation-index.json').read_bytes()),'new_model_or_Engine_calls':0,'semantic_score':'NOT_SCORED'}
    with (a.output/('capture-lineage-'+a.provider+'.json')).open('x') as f:json.dump(lineage,f,indent=2);f.write('\n')
if __name__=='__main__':main()
