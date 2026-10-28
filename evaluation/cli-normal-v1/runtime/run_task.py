#!/usr/bin/env python3
"""Ordinary-only entry to the preserved callback, with its own sealed admission."""
import hashlib, importlib.util, json, os, sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(1,str(HERE.parent))
import preparation_guard
from scorer import require

def require_ordinary_callback(args,env):
    require(args.track=='e2e_normal' and args.arm in ['baseline','candidate'],'ordinary_only')
    if args.model=='fixture-model':
        preparation_guard.require_loopback_preparation(args.model,env)
        return
    require('ENGINE073_LOOPBACK_BASE_URL' not in env,'live_callback_cannot_use_engineering_redirect')
    protocol=json.loads((HERE/'protocol.json').read_bytes())
    require(protocol.get('identity')=='cli-normal-real24-development-v1' and protocol.get('normal_only') is True and protocol.get('tracks')==['e2e_normal'],'ordinary_profile_required')
    require(protocol.get('state')=='DEVELOPMENT_REQUIRES_FROZEN_RECEIPT','ordinary_runtime_preparation_only')
    require(args.provider == 'mistral' and protocol['providers'][args.provider]['new_live_run_allowed'] is True,'provider_not_admitted')
    require(args.model==protocol['providers'][args.provider]['model'],'callback_unfrozen_model')
    name=env.get('CLI_REPAIR_DEVELOPMENT_RECEIPT','');digest=env.get('CLI_REPAIR_DEVELOPMENT_RECEIPT_SHA256','')
    require(bool(name) and len(digest)==64,'callback_requires_frozen_receipt')
    receipt_path=Path(name)
    require(receipt_path.is_absolute() and receipt_path.is_file(),'callback_receipt_missing')
    require(hashlib.sha256(receipt_path.read_bytes()).hexdigest()==digest,'callback_receipt_sha_mismatch')
    from development import checked_receipt,consumed_corpus
    checked_receipt(receipt_path,args.provider);consumed_corpus(args.corpus.resolve().parent)
    directory=Path.cwd().resolve();plan=json.loads((directory.parent/'plan.json').read_bytes());state=json.loads((directory.parent/'state.json').read_bytes())
    from run_lane import check_plan
    check_plan(plan,protocol)
    preparation_guard.validate_reserved_callback(args,plan,state,directory,[sys.executable,*sys.argv],receipt_path,digest)

preparation_guard.require_callback_admission=require_ordinary_callback
spec=importlib.util.spec_from_file_location('preserved_paired_task',HERE.parent/'run_task.py')
paired=importlib.util.module_from_spec(spec);spec.loader.exec_module(paired)
if __name__=='__main__':raise SystemExit(paired.main())
