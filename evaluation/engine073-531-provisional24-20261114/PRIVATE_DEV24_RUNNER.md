# Controlled experimental Mistral run — issue #531 (NOT formal acceptance)

This runner is a bounded **self-authored synthetic development experiment**, not an independent real-world benchmark or a substitute for formal claim-level review. Its corpus was created by the same ChatGPT session initiating evaluation. Do not use it to assert formal F1 or approve 0.7.3.

- Pinned Core 0.6.1: `4abee90f0501f0d8fc09b453a0b45c942fbed69e`.
- Pinned corrected development Core: `7466c6f90d420a98c99713b96d6a0623e2aeef52`.
- Pinned shared CLI: `988b69f086ac4a209fe93eb10cf1fa6786977567`.
- Mistral model: `ministral-8b-2512`, macOS-15, stable Rust 1.88; initial request parity checked *before* provider calls.
- Frozen private dataset SHA256 `63da84b7176766c2fec9becf7ad631ff03b0b7c7264f4f120ac05816abd43e1c`.
- 24 x 2 arms = 48 actual CLI runs, serially, alternating arm order; 4 shards x 6 cases.
- Hard per-arm limits from prior successful V5 run: max model calls 12, max generation tokens 1024, max output tokens 4096, max total tokens 28,096, timeout 1,200 seconds. No automatic retries or replays. A CLI failure, timeout or unknown usage stops subsequent jobs.
- Absolute worst-case 48 x 28,096 = **1,348,608** input+output tokens; at the observed model list price $0.15/million each, **$0.2022912** estimated ceiling before extra fees. This is not a billing receipt; user reported about $9.98 of $10 monthly included credit on 2026-11-14. **Current admin usage/pay-as-you-go and no-overage entitlement are not independently verified**. The workflow's GET `/v1/models` only tests API key, model availability and some throughput headers; it does not validate account balance. Do not initiate billable job until this check is resolved.
- `push` on the runner branch runs **only provider-free preflight**. The explicit **one-shot live tag** `eval-531-dev24-live-20261114-one-shot` triggers billable run, and must not be pushed absent current verified free credit/no-overage protection. `github.run_attempt == 1` prevents rerun replay. Repo secret `ENGINE073_531_DEV24_20261114_JSON` stores the frozen private corpus; GitHub Secret `MISTRAL_API_KEY` is used only in live execution.
- Original private source/labels and stdout/stderr remain in the GitHub Actions runner temp directory; per-shard archive is encrypted using the public X.509 certificate in this directory, and **only `.cms` encrypted output + sanitized usage summary** is uploaded to GitHub Actions (7-day retention). Private decryption key exists **only on the user Mac** and must never be committed, printed or uploaded. Root temp directory is cleared after artifact upload.
- Decrypt an artifact locally: `openssl cms -decrypt -binary -inform DER -in encrypted-01.cms -recip reviewer-cert.pem -inkey /path/to/local/private-key.pem -out recovered.tar.gz`. The recovered archive is private and contains model output, source/label evidence and first-request plans; do not publish it.
- Even with 48/48 completed, independent review, gold provenance, unit-claim F1, citation fidelity, all quality/efficiency gates and release packaging remain separate. `formal_f1=null`, `quality_go=false`, `adoption_go=false` until separately proven.
