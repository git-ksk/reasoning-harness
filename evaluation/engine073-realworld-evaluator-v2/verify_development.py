#!/usr/bin/env python3
import argparse,json
from pathlib import Path
from scorer import require,sha
from development import checked_receipt,PURPOSE

def main():
    p=argparse.ArgumentParser();p.add_argument('--receipt',type=Path,required=True);p.add_argument('--provider',required=True);p.add_argument('--image-archive',type=Path,required=True);p.add_argument('--actual-image-id',required=True);a=p.parse_args()
    receipt=checked_receipt(a.receipt,a.provider)
    require(sha(a.image_archive.read_bytes())==receipt['runtime_image_archive_sha256'] and a.actual_image_id==receipt['runtime_image_id'],'development_image_changed')
    print(json.dumps({'development':'VERIFIED','purpose':PURPOSE,'accuracy_acceptance':False,'provider_model_calls':0}))
if __name__=='__main__':main()
