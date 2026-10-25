"""Separate seen-corpus diagnostic admission; never an accuracy/M0 receipt."""
import json,time
from pathlib import Path
from scorer import require,sha

INPUT_M1='597e1488d07654b07e2a1e528db8b1ae934e438a8edd3f96d31cfc26dd584cd3'
PURPOSE='CONSUMED_REAL24_ENGINEERING_DIAGNOSTIC_NOT_ACCEPTANCE'
HERE=Path(__file__).resolve().parent
REPO=HERE.parents[1]

def checked_binaries(identity,bin_dir):
    require(identity.get('host_commit')=='73bbd18173b4dfcd6664a26241de0385f2eb2e3e','development_host_changed')
    expected={'baseline':'4abee90f0501f0d8fc09b453a0b45c942fbed69e','candidate':'ab3c6322921c8c738ec35bd018345917b5afa538'}
    require(set(identity.get('arms',{}))==set(expected),'development_arm_inventory_changed')
    for arm,pin in expected.items():
        record=identity['arms'][arm]
        require(record['engine_commit']==pin and record['cli_package_version']=='0.5.4','development_core_or_version_changed')
        require(set(record.get('binaries',{}))=={'reason','adviser','source-host','audit-probe','registry'},'development_binary_inventory_incomplete')
        for name,digest in record['binaries'].items():
            require(sha((bin_dir/(name+'-'+arm)).read_bytes())==digest,'development_binary_changed')

OPERATIONAL_REVIEWER='chatgpt-gpt6-technical-review-20261025'

def require_operational_review_identity(protocol,review,receipt):
    policy=protocol.get('operational_review',{})
    require(policy.get('reviewer_id')==OPERATIONAL_REVIEWER
            and policy.get('reviewer_kind')=='agent_nonhuman'
            and policy.get('required_scope')=='cli-repair-paired-development-runtime'
            and policy.get('required_result')=='PASS', 'development_operational_reviewer_policy_changed')
    require(review.get('reviewer')==OPERATIONAL_REVIEWER
            and review.get('reviewer_kind')=='agent_nonhuman'
            and review.get('result')=='PASS'
            and review.get('scope')==policy['required_scope'],'development_full_operational_review_missing')
    # The previous bounded review must never be promoted to full operational PASS.
    require(review.get('source_commit')==receipt.get('source_commit')
            and review.get('runtime_proof_sha256')==receipt.get('runtime_proof_sha256'),
            'operational_review_not_for_source_or_proof')

def require_operational_verdict(protocol,review,verdict,receipt):
    require_operational_review_identity(protocol,review,receipt)
    require(verdict.get('result')=='PASS'
            and verdict.get('reviewer')==review['reviewer']
            and verdict.get('reviewer_kind')=='agent_nonhuman'
            and verdict.get('scope')==protocol['operational_review']['required_scope'],
            'development_review_scope_mismatch')
    for key in ['source_commit','runtime_proof_sha256','paired_host_identity_sha256',
                'runtime_image_id','runtime_image_archive_sha256']:
        require(verdict.get(key)==receipt.get(key),'review_not_for_runtime:'+key)

def checked_receipt(path,provider):
    protocol=json.loads((HERE/'protocol.json').read_text())
    require(protocol['state']=='DEVELOPMENT_REQUIRES_FROZEN_RECEIPT','paired_runtime_preparation_only')
    require(provider in ['mistral','google'],'Groq_deferred_for_development')
    require(protocol['providers'][provider]['new_live_run_allowed'] is True,'provider_profile_deferred')
    path=Path(path).resolve()
    require(path.is_file(),'development_receipt_missing')
    receipt=json.loads(Path(path).read_bytes())
    require(receipt.get('identity')=='cli-repair-paired-v1-development-freeze-v1' and receipt.get('state')=='DEVELOPMENT_FROZEN','development_not_frozen')
    require(receipt.get('purpose')==PURPOSE and receipt.get('fresh_accuracy_cases')==0 and receipt.get('accuracy_acceptance') is False,'development_scope_mismatch')
    require(receipt.get('consumed_M1_sha256')==INPUT_M1,'wrong_consumed_corpus')
    require(receipt.get('execution_tag')==protocol['execution_tag'],'wrong_development_execution')
    require(receipt.get('semantic_reviewer_id')=='/root/paired_evaluation_independent','wrong_semantic_reviewer')
    require(receipt.get('provider_profiles')==protocol['providers'],'provider_profile_changed')
    require(receipt['required_provider_readiness'][provider]['new_live_run_allowed'] is True,'provider_not_admitted')
    require(time.time()<=receipt['required_provider_readiness'][provider]['valid_until_unix_seconds'],'provider_readiness_expired')
    review=receipt.get('independent_operational_review',{})
    require_operational_review_identity(protocol,review,receipt)
    require(sha((Path(path).parent/'development-review/artifact-manifest.json').read_bytes())==review['manifest_sha256'],'development_review_manifest_changed')
    verdict_path=path.parent/'development-review/review.json'
    require(sha(verdict_path.read_bytes())==review['verdict_sha256'],'development_review_verdict_changed')
    verdict=json.loads(verdict_path.read_bytes())
    require_operational_verdict(protocol,review,verdict,receipt)
    files={str(p.relative_to(REPO)):sha(p.read_bytes()) for p in REPO.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    require(files==receipt['files'],'development_runtime_changed')
    require(sha(json.dumps(files,sort_keys=True,separators=(',',':')).encode())==review['reviewed_runtime_files_sha256'],'runtime_not_reviewed')
    require(verdict.get('reviewed_runtime_files_sha256')==review['reviewed_runtime_files_sha256'],'review_not_for_runtime')
    require(sha((HERE/'protocol.json').read_bytes())==receipt['protocol_sha256'],'development_protocol_changed')
    proof=Path('/eval/bin/identity.json');identity=json.loads(proof.read_bytes())
    require(sha(proof.read_bytes())==receipt['paired_host_identity_sha256'],'development_binary_identity_changed')
    checked_binaries(identity,proof.parent)
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
