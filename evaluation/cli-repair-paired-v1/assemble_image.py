#!/usr/bin/env python3
"""Copy the complete declared runtime inventory, never local home or credentials."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil

def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    p=argparse.ArgumentParser()
    p.add_argument('--repo',type=Path,required=True)
    p.add_argument('--bin-dir',type=Path,required=True)
    p.add_argument('--context',type=Path,required=True)
    a=p.parse_args();root=a.context/'root';root.mkdir()
    inventory=json.loads((a.repo.parent/'preparation.json').read_bytes())['files']
    for name,expected in inventory.items():
        relative=Path(name)
        assert not relative.is_absolute() and '..' not in relative.parts
        source=a.repo/relative
        assert not source.is_symlink() and source.is_file() and digest(source)==expected
        target=root/'repo'/relative;target.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(source,target)
    assert {str(p.relative_to(root/'repo')):digest(p) for p in (root/'repo').rglob('*') if p.is_file()}==inventory
    identity=json.loads((a.bin_dir/'identity.json').read_bytes())
    bins={'identity.json':digest(a.bin_dir/'identity.json')}
    for arm in ['baseline','candidate']:
        assert set(identity['arms'][arm]['binaries'])=={'reason','adviser','source-host','audit-probe','registry'}
        bins.update({name+'-'+arm:value for name,value in identity['arms'][arm]['binaries'].items()})
    target=root/'eval/bin';target.mkdir(parents=True)
    for name,expected in bins.items():
        source=a.bin_dir/name
        assert not source.is_symlink() and digest(source)==expected
        shutil.copy2(source,target/name)

if __name__=='__main__':main()
