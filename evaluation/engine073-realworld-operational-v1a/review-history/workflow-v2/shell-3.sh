set -euo pipefail
test -f /tmp/results/lane/plan.json || exit 0
docker run --rm --read-only --network none --tmpfs /tmp -v /tmp/frozen:/frozen:ro -v /tmp/results:/output engine073-runtime python3 -B /repo/evaluation/engine073-realworld-evaluator-v1/collect.py --corpus-dir /frozen/corpus --provider '${{ matrix.provider }}' --lane /output/lane --output /output/observations
