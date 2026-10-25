#!/usr/bin/env python3
"""Record bytes inside the actual read-only image, without credentials/network."""
import hashlib,json
from pathlib import Path
def inventory(root):
    return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted(root.rglob('*')) if p.is_file() and '__pycache__' not in p.parts}
if __name__=='__main__':
    print(json.dumps({'runtime_files':inventory(Path('/repo')),'binary_files':inventory(Path('/eval/bin'))},sort_keys=True))
