"""Separate seen-corpus diagnostic admission; never an accuracy/M0 receipt."""
import json
from pathlib import Path
from scorer import require,sha

INPUT_M1='597e1488d07654b07e2a1e528db8b1ae934e438a8edd3f96d31cfc26dd584cd3'
PURPOSE='CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE'
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[1]

def checked_receipt(path,provider):
    require(provider in ['mistral','google'],'Groq_deferred_for_development')
    receipt=json.loads(Path(path).read_bytes())
    require(receipt.get('identity')=='engine073-realworld-v2-development-freeze-v1' and receipt.get('state')=='DEVELOPMENT_FROZEN','development_not_frozen')
    require(receipt.get('purpose')==PURPOSE and receipt.get('fresh_accuracy_cases')==0 and receipt.get('accuracy_acceptance') is False,'development_scope_mismatch')
    require(receipt.get('consumed_M1_sha256')==INPUT_M1,'wrong_consumed_corpus')
    require(receipt['required_provider_readiness'][provider]['new_live_run_allowed'] is True,'provider_not_admitted')
    review=receipt.get('independent_operational_review',{})
    require(review.get('result')=='PASS' and review.get('reviewer')=='/root/engine07_realworld_independent','development_review_missing')
    require(sha((Path(path).parent/'development-review/artifact-manifest.json').read_bytes())==review['manifest_sha256'],'development_review_manifest_changed')
    files={str(p.relative_to(REPO)):sha(p.read_bytes()) for p in REPO.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    require(files==receipt['files'],'development_runtime_changed')
    require(sha(json.dumps(files,sort_keys=True,separators=(',',':')).encode())==review['reviewed_runtime_files_sha256'],'runtime_not_reviewed')
    require(sha((HERE/'protocol.json').read_bytes())==receipt['protocol_sha256'],'development_protocol_changed')
    proof=Path('/eval/bin/identity.json');identity=json.loads(proof.read_bytes())
    require(sha(proof.read_bytes())==receipt['paired_host_identity_sha256'],'development_binary_identity_changed')
    require(identity['host_commit']=='8525330ea3cadb897c50d6688228c4895558d3dc','development_host_changed')
    expected={'baseline':'4abee90f0501f0d8fc09b453a0b45c942fbed69e','candidate':'ab3c6322921c8c738ec35bd018345917b5afa538'}
    for arm,pin in expected.items():
        record=identity['arms'][arm];require(record['engine_commit']==pin and record['cli_package_version']=='0.5.4','development_core_or_version_changed')
        for name,digest in record['binaries'].items():require(sha((proof.parent/(name+'-'+arm)).read_bytes())==digest,'development_binary_changed')
    return receipt

def consumed_corpus(root):
    root=Path(root).resolve();m=(root/'manifest.json').read_bytes()
    require(sha(m)==INPUT_M1,'consumed_manifest_changed')
    inventory=json.loads(m)['files']
    actual={str(p.relative_to(root)) for p in root.rglob('*') if p.is_file() and p!=root/'manifest.json'}
    require(set(inventory)==actual and 'manifest.json' not in inventory,'consumed_inventory_changed')
    for name,digest in inventory.items():
        p=(root/name).resolve();require(p.is_relative_to(root) and sha(p.read_bytes())==digest,'consumed_input_changed')
    data=json.loads((root/'corpus.json').read_bytes())
    require(data['identity']=='engine073-realworld-independent-v1' and len(data['cases'])==24,'not_original_consumed24')
    return data
