#!/usr/bin/env python3
import argparse,json,sys
from pathlib import Path
sys.path.insert(1,str(Path(__file__).resolve().parent.parent))
from development import checked_receipt,PURPOSE
from image_identity import verify_loaded_image

def main():
    p=argparse.ArgumentParser();p.add_argument('--receipt',type=Path,required=True);p.add_argument('--provider',required=True);p.add_argument('--image-archive',type=Path,required=True);p.add_argument('--actual-image-id',required=True);a=p.parse_args()
    receipt=checked_receipt(a.receipt,a.provider)
    verify_loaded_image(a.image_archive,receipt['runtime_image_archive_sha256'],receipt['runtime_image_id'],a.actual_image_id)
    print(json.dumps({'development':'VERIFIED','purpose':PURPOSE,'accuracy_acceptance':False,'provider_model_calls':0,'actual_image_id':a.actual_image_id}))
if __name__=='__main__':main()
