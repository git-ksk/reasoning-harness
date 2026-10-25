#!/usr/bin/env python3
"""Compare actual core serializers using the existing inert preflight corpus."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
from resolver import records, scope


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--corpus', type=Path, required=True)
    parser.add_argument('--bin-dir', type=Path, default=Path('/eval/bin'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    corpus = json.loads(args.corpus.read_bytes())
    case = corpus['cases'][0]
    snapshots = []
    for record in records(corpus, case):
        at = record['acquisition_metadata']['retrieved_at_unix_seconds']
        snapshots.append({key: record[key] for key in ['id', 'source', 'observation', 'facts']} | {'metadata': {
            'scope': scope(case['required_scope']), 'provenance_class': 'primary',
            'temporal': {'effective_from_unix_seconds': at, 'effective_until_unix_seconds': at + 86400}}})
    payload = json.dumps(snapshots).encode()
    outputs = {arm: subprocess.check_output([str(args.bin_dir / ('registry-' + arm))], input=payload)
               for arm in ['baseline', 'candidate']}
    assert outputs['baseline'] == outputs['candidate']
    registry = json.loads(outputs['baseline'])
    assert len(registry['sources']) == 2 and registry['unresolved_inputs'] == []
    proof = {'pass': True, 'sources': 2, 'identical_output': True,
             'input_sha256': hashlib.sha256(payload).hexdigest(),
             'registry_sha256': hashlib.sha256(outputs['baseline']).hexdigest(),
             'external_provider_requests': 0, 'new_accuracy_cases': 0}
    with args.output.open('x') as handle:
        json.dump(proof, handle, indent=2)
        handle.write('\n')
    print(json.dumps(proof))


if __name__ == '__main__':
    main()
