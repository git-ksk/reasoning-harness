set -euo pipefail
mkdir -p /tmp/results
python3 - <<'PYREADY'
import json,os,hashlib
from pathlib import Path
digest=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
amendment=json.load(open('evaluation/engine073-realworld-operational-v1a/receipt.json'))
assert amendment['identity']=='engine073-realworld-independent-v1a-operational-continuation'
assert amendment['original_run_id']==39248903567 and amendment['original_provider_calls']==amendment['original_accuracy_observations']==0
assert amendment['semantic_or_profile_changes'] is False and amendment['gold_or_questions_changed'] is False
assert amendment['independent_review']['result']=='PASS' and amendment['independent_review']['reviewer']=='/root/engine07_realworld_independent'
assert digest('evaluation/engine073-realworld-operational-v1a/independent-review/artifact-manifest.json')==amendment['independent_review']['manifest_sha256']
assert digest('.github/workflows/engine073-realworld-independent-v1a.yml')==amendment['workflow_sha256']
assert digest('evaluation/engine073-realworld-first-v1-operational-fail/manifest.json')==amendment['original_first_manifest_sha256']
assert digest('evaluation/engine073-realworld-M0-v1/receipt.json')==amendment['M0_receipt_sha256']
assert digest('evaluation/engine073-realworld-independent-v1-input/manifest.json')==amendment['M1_sha256']
m=json.load(open('evaluation/engine073-realworld-M0-v1/receipt.json'))
provider='${{ matrix.provider }}';r=m['required_provider_readiness'][provider]
allowed=r['new_live_run_allowed'] is True
with open(os.environ['GITHUB_OUTPUT'],'a') as f:f.write('allowed='+str(allowed).lower()+'\n')
if not allowed:
    record={'provider':provider,'state':'DEFERRED_BEFORE_CREDENTIALS','reason':r['reason'],'planned_observations':96,'complete_observations':0,'actual_answer_f1':None,'provider_attempts':0,'full_three_provider_acceptance':'NO_GO'}
    with open('/tmp/results/provider-deferred.json','x') as f:json.dump(record,f,indent=2)
PYREADY
