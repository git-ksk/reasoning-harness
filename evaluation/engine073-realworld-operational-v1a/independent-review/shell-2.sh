set -euo pipefail
name=engine073-v1a-first-${{ matrix.provider }}-${{ github.run_id }}
trap 'docker rm -f "$name" >/dev/null 2>&1 || true' EXIT INT TERM
timeout --signal=INT --kill-after=5s 200m docker run --name "$name" --read-only --tmpfs /tmp -v /tmp/frozen:/frozen:ro -v /tmp/results:/output -e MISTRAL_API_KEY -e GEMINI_API_KEY -e GROQ_API_KEY engine073-runtime python3 -B /repo/evaluation/engine073-realworld-evaluator-v1/run_lane.py --plan /output/plan.json --output /output/lane --protocol /repo/evaluation/engine073-realworld-evaluator-v1/protocol.json
