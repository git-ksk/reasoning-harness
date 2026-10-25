"""Inventory every mounted static callback input before any provider execution."""
import sys
from pathlib import Path
from scorer import sha
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[1]
def inventory(root,receipt,groq_recovery=None):
    folders=[Path(root),Path(receipt).resolve().parent/'development-review',Path(receipt).resolve().parent/'provider-readiness',HERE,REPO/'evaluation/engine07-realworld-runner-v1',REPO/'evaluation/engine07-realworld-operations-v1',Path('/eval/bin')]
    paths={p for folder in folders for p in folder.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    paths.update([Path(sys.executable),Path(receipt).resolve()])
    if groq_recovery is not None:paths.add(Path(groq_recovery).resolve())
    return {str(p):sha(p.read_bytes()) for p in sorted(paths)}
