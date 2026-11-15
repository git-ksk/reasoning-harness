#!/usr/bin/env python3
"""Stage a read-only, PRIVATE source packet for independent question authoring.

This is neither an evaluation author, a reviewer nor an executor. The packet
contains public pinned upstream source bytes and 24 blank case templates,
no gold, no reviewer signing key, no model response and no provider access.
It is never written inside the repository and is never uploaded to GitHub.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import stat
import sys

HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import stage_source_pool as source_pool
import validate_readiness as readiness
import build_corrected_pair as pair
from contracts import _open_trusted_directory

CONTRACT='engine073-531-private-independent-authoring-packet-v1'
PUBLIC_SOURCE='evaluation/engine073-531-independent24-20261116/provisional-new-sources.json'
FILES={'packet.json','source-manifest.json'}


def sha(content: bytes)->str:
    return hashlib.sha256(content).hexdigest()


def canonical(obj)->bytes:
    return json.dumps(obj,sort_keys=True,ensure_ascii=False,separators=(',',':'),allow_nan=False).encode('utf-8')+b'\n'


def write_unique(path:Path,data:bytes):
    fd=os.open(path,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
    with os.fdopen(fd,'wb') as target:
        os.fchmod(target.fileno(),0o600)
        target.write(data)
        target.flush()
        os.fsync(target.fileno())


def verify_source_bytes(source:dict,content:bytes):
    if not isinstance(content,bytes) or not 1<=len(content)<=source_pool.MAX_SOURCE_BYTES:
        raise ValueError('unexpected_source_bytes_or_size')
    if sha(content)!=source['sha256'] or len(content)!=source['bytes']:
        raise ValueError('pinned_upstream_document_content_changed')
    content.decode('utf-8','strict')


def make_packet(root:Path,manifest:dict,source_raw:bytes,payloads:list[bytes]):
    # Caller has validated source manifest trust and downloaded into bounded
    # memory before creating any file. Refuse inconsistent input on entry.
    if len(manifest['sources'])!=12 or len(payloads)!=12:
        raise ValueError('not_exactly_twelve_sources')
    for spec,payload in zip(manifest['sources'],payloads,strict=True):
        verify_source_bytes(spec,payload)
    if root.exists():
        raise FileExistsError('packet_never_overwrite_or_reuse')
    pair.original.private_new_directory(root)
    docs=root/'docs'
    cases=root/'blank-cases'
    pair.original.private_new_directory(docs)
    pair.original.private_new_directory(cases)
    indexed=[]
    case_ids=[]
    for index,(record,payload) in enumerate(zip(manifest['sources'],payloads,strict=True),1):
        name=f'upstream-{index:02d}.txt'
        write_unique(docs/name,payload)
        indexed.append({'source_id':f'source-{index:02d}',
                        'relative_path':'docs/'+name,
                        'source_sha256':record['sha256'],
                        'upstream_commit':record['commit'],
                        'official_url':record['url'],
                        'bytes':record['bytes']})
        for j in range(2):
            id=f'ih-{2*index-1+j:02d}'
            case_ids.append(id)
            blank={'contract':'engine073-531-independent-author-blank-case-v1',
                   'case_id':id,'source_id':f'source-{index:02d}',
                   'question':None,'question_sha256':None,
                   'independent_author_id':None,
                   'answer_or_label':None,
                   'status':'UNAUTHORED',
                   'not_a_frozen_case':True}
            write_unique(cases/(id+'.json'),canonical(blank))
    if case_ids!=[f'ih-{i:02d}' for i in range(1,25)]:
        raise ValueError('incorrect_blank_case_slots')
    packet={'contract':CONTRACT,
            'source_manifest_sha256':sha(source_raw),
            'source_count':12,'case_slot_count':24,
            'source_files':indexed,'blank_case_ids':case_ids,
            'independent_author_assigned':False,
            'independent_reviewer_assigned':False,
            'real_question_count':0,
            'private_gold_present':False,
            'approved':False,
            'model_calls':0,
            'formal_f1':None,
            'not_a_live_execution_authority':True}
    write_unique(root/'source-manifest.json',source_raw)
    write_unique(root/'packet.json',canonical(packet))
    return packet


def read_private_file(path:Path)->bytes:
    # Do not dereference a symbolic link or read a world-readable / replaced
    # file before authenticating its inode and private 0600 mode.
    fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK)
    try:
        st=os.fstat(fd)
        if not stat.S_ISREG(st.st_mode) or st.st_uid!=os.getuid() or stat.S_IMODE(st.st_mode)!=0o600:
            raise ValueError('packet_private_file_permission_or_identity_mismatch')
        content=b''
        while True:
            chunk=os.read(fd,65536)
            if not chunk:break
            content+=chunk
            if len(content)>source_pool.MAX_SOURCE_BYTES+65536:
                raise ValueError('private_packet_file_oversize')
        after=os.fstat(fd)
        if (st.st_dev,st.st_ino,st.st_size,st.st_mtime_ns)!=(after.st_dev,after.st_ino,after.st_size,after.st_mtime_ns):
            raise ValueError('private_packet_changed_during_read')
        return content
    finally:
        os.close(fd)


def safe_private(root:Path):
    if root.is_symlink() or not root.is_dir():
        raise ValueError('missing_or_symlinked_private_packet_root')
    # FD-based walk enforces owner/mode and rejects inherited allow ACLs.
    fd=_open_trusted_directory(root)
    os.close(fd)


def audit(repo:Path,root:Path):
    repo=repo.resolve(strict=True)
    if root!=root.resolve(strict=True):
        raise ValueError('private_packet_root_must_be_canonical')
    safe_private(root)
    trusted_raw=(repo/PUBLIC_SOURCE).read_bytes()
    manifest=json.loads(trusted_raw)
    source_pool.validate(repo,manifest)
    packet_path=root/'packet.json'
    if packet_path.is_symlink():
        raise ValueError('packet_symlink_not_allowed')
    packet_raw=read_private_file(packet_path)
    packet=json.loads(packet_raw)
    if canonical(packet)!=packet_raw or packet.get('contract')!=CONTRACT:
        raise ValueError('invalid_packet_canonical_contract')
    if packet.get('source_manifest_sha256')!=sha(trusted_raw):
        raise ValueError('source_manifest_changed_since_author_packet_created')
    if packet.get('source_files')!=[
        {'source_id':f'source-{index:02d}',
         'relative_path':f'docs/upstream-{index:02d}.txt',
         'source_sha256':r['sha256'],
         'upstream_commit':r['commit'],
         'official_url':r['url'],'bytes':r['bytes']}
        for index,r in enumerate(manifest['sources'],1)]:
        raise ValueError('packet_source_inventory_was_rewritten')
    if packet.get('blank_case_ids')!=[f'ih-{i:02d}' for i in range(1,25)] or packet.get('case_slot_count')!=24:
        raise ValueError('packet_case_slot_inventory_changed')
    if any(packet.get(k)!=v for k,v in {
        'source_count':12,'real_question_count':0,
        'independent_author_assigned':False,
        'independent_reviewer_assigned':False,
        'private_gold_present':False,'approved':False,
        'model_calls':0,'formal_f1':None,
        'not_a_live_execution_authority':True}.items()):
        raise ValueError('false_authoring_reviewer_or_execution_approval')
    if sorted(p.name for p in root.iterdir())!=sorted(FILES|{'docs','blank-cases'}):
        raise ValueError('unreviewed_extra_packet_files')
    if read_private_file(root/'source-manifest.json')!=trusted_raw:
        raise ValueError('source_manifest_inside_packet_not_trusted')
    for subdir,files in [('docs',[f'upstream-{i:02d}.txt' for i in range(1,13)]),
                         ('blank-cases',[f'ih-{i:02d}.json' for i in range(1,25)])]:
        path=root/subdir
        safe_private(path)
        if sorted(f.name for f in path.iterdir())!=sorted(files):
            raise ValueError('unexpected_packet_inner_file_inventory')
        for file in files:
            target=path/file
            meta=target.lstat()
            if not stat.S_ISREG(meta.st_mode) or stat.S_IMODE(meta.st_mode)!=0o600 or meta.st_uid!=os.getuid():
                raise ValueError('packet_private_file_permission_or_identity_mismatch')
    for index,r in enumerate(manifest['sources'],1):
        path=root/'docs'/f'upstream-{index:02d}.txt'
        verify_source_bytes(r,read_private_file(path))
    for index in range(1,25):
        path=root/'blank-cases'/f'ih-{index:02d}.json'
        raw=read_private_file(path)
        slot=json.loads(raw)
        expected={'contract':'engine073-531-independent-author-blank-case-v1',
                  'case_id':f'ih-{index:02d}','source_id':f'source-{(index+1)//2:02d}',
                  'question':None,'question_sha256':None,'independent_author_id':None,
                  'answer_or_label':None,'status':'UNAUTHORED','not_a_frozen_case':True}
        if raw!=canonical(expected) or slot!=expected:
            raise ValueError('blank_case_modified_or_independent_gold_leaked')
    for name in FILES:
        target=root/name
        meta=target.lstat()
        if not stat.S_ISREG(meta.st_mode) or stat.S_IMODE(meta.st_mode)!=0o600 or meta.st_uid!=os.getuid():
            raise ValueError('packet_private_file_permission_or_identity_mismatch')
    return {'contract':CONTRACT,'source_sha256_verified':12,'blank_slots_unchanged':24,
            'independent_questions_authored':0,'reviewer_approval':False,
            'model_calls':0,'formal_f1':None,'live_allowed':False}


def stage(repo:Path,root:Path):
    repo=repo.resolve(strict=True)
    readiness.verify(repo)
    raw=(repo/PUBLIC_SOURCE).read_bytes()
    manifest=json.loads(raw)
    source_pool.validate(repo,manifest)
    payloads=[]
    for entry in manifest['sources']:
        payload=source_pool.fetch(entry['url'],source_pool.MAX_SOURCE_BYTES)
        verify_source_bytes(entry,payload)
        payloads.append(payload)
    packet=make_packet(root,manifest,raw,payloads)
    audited=audit(repo,root)
    return {**audited,'source_manifest_sha256':packet['source_manifest_sha256']}


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--repo',required=True,type=Path)
    parser.add_argument('--packet',required=True,type=Path)
    parser.add_argument('--mode',choices=['stage','audit'],required=True)
    args=parser.parse_args()
    root=args.packet.expanduser().absolute()
    if args.mode=='stage':
        output=stage(args.repo,root)
    else:
        output=audit(args.repo,root)
    print(json.dumps(output,sort_keys=True,indent=2))

if __name__=='__main__':main()
