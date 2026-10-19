set -euo pipefail
test "$GITHUB_RUN_ATTEMPT" = 1
gh api 'repos/${{ github.repository }}/actions/runs?event=push&head_sha=${{ github.sha }}&per_page=100' > /tmp/run-reservation.json
python3 - <<'PYGUARD'
import json,os
runs=json.load(open('/tmp/run-reservation.json'))['workflow_runs']
matching=[r for r in runs if r['path']=='.github/workflows/engine073-realworld-independent-v1a.yml']
assert len(matching)==1 and str(matching[0]['id'])==os.environ['GITHUB_RUN_ID']
PYGUARD
artifact_id=$(python3 -c "import json; print(json.load(open('evaluation/engine073-realworld-M0-v1/receipt.json'))['runtime_artifact_id'])")
gh api "repos/${{ github.repository }}/actions/artifacts/$artifact_id/zip" > /tmp/runtime.zip
mkdir -p /tmp/m0-runtime /tmp/frozen /tmp/results
unzip -q /tmp/runtime.zip -d /tmp/m0-runtime
docker load -i /tmp/m0-runtime/runtime-image.tar.gz
image_id=$(docker image inspect engine073-runtime --format '{{.Id}}')
cp evaluation/engine073-realworld-M0-v1/receipt.json /tmp/frozen/M0.json
cp evaluation/engine073-realworld-M0-v1/groq-recovery.json /tmp/frozen/groq-recovery.json
cp -R evaluation/engine073-realworld-independent-v1-input /tmp/frozen/corpus
docker run --rm --read-only --network none --tmpfs /tmp -v /tmp/frozen:/frozen:ro -v /tmp/m0-runtime:/image:ro engine073-runtime python3 -B /repo/evaluation/engine073-realworld-evaluator-v1/verify_M0.py --provider '${{ matrix.provider }}' --receipt /frozen/M0.json --image-archive /image/runtime-image.tar.gz --actual-image-id "$image_id" --repo /repo
m1=$(sha256sum /tmp/frozen/corpus/manifest.json | cut -d' ' -f1)
docker run --rm --read-only --tmpfs /tmp -v /tmp/frozen:/frozen engine073-runtime python3 -B /repo/evaluation/engine073-realworld-evaluator-v1/prepare_sources.py --corpus-dir /frozen/corpus --M1-sha256 "$m1"
recovery=()
if test '${{ matrix.provider }}' = groq; then
  recovery=(--groq-recovery-file /frozen/groq-recovery.json)
fi
docker run --rm --read-only --network none --tmpfs /tmp -v /tmp/frozen:/frozen -v /tmp/results:/output engine073-runtime python3 -B /repo/evaluation/engine073-realworld-evaluator-v1/make_plan.py --M0-receipt /frozen/M0.json --corpus-dir /frozen/corpus --M1-sha256 "$m1" --provider '${{ matrix.provider }}' --output /output/plan.json "${recovery[@]}"
sudo chmod -R a-w /tmp/frozen
docker run --rm --read-only --network none --tmpfs /tmp -v /tmp/frozen:/frozen:ro engine073-runtime python3 -B -c "import sys,json;sys.path.insert(0,'/repo/evaluation/engine073-realworld-evaluator-v1');from corpus import validate,verify_provenance;verify_provenance('/frozen/corpus','/frozen/M0.json');validate('/frozen/corpus',sys.argv[1]);print(json.dumps({'post_chmod_M1':'VERIFIED','model_provider_calls':0}))" "$m1"
