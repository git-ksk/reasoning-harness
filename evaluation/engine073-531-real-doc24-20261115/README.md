# Issue #531 — Public upstream documents: 24 paired development cases

**Scope:** Development-only comparison of old Engine **0.6.1** with the **0.7.3 candidate**, using a fixed native CLI and Mistral model. **This is not a blind holdout or a formal release-quality F1 evaluation.** These 24 questions were already consumed by prior October 2026 development experiments. Unlike the previous fictional device-launch test, these materials originate in real pinned upstream product manuals.

## Sources and provenance

- **24 questions** cover **12 original upstream files** from curl, Git, CMake, OpenSSL, SQLite and Redis. Each URL is pinned to a specific 40-character Git commit, and has a SHA256 fingerprint of the complete upstream file.
- All **12 complete upstream files** were downloaded again and matched their frozen SHA256; every stored quotation was found in its original document after normalizing whitespace. Results are recorded in [source-audit.json](source-audit.json). Source/version/quote/case metadata is in [cases.json](cases.json) and individual original [configs/](configs/) files.
- Questions include multi-source comparisons, caveats, different source scopes, missing information, hypothetical private resources and potentially misleading context. Statements about a *private repo* or *live Redis invoice key* are **hypothetical test questions**, not real user data.
- No passwords, credentials, actual account data, prior model answers, evaluation ground-truth labels, or reviewer keys are included. **Original semantic gold** stays off the repository and off the Actions runner. This benchmark has public evaluation inputs: the goal is regression diagnosis, not a blinded performance claim.
- Original public cases come from the previous preserved evaluation request set (SHA256 `d53410e21cf899d562d63d72d22c392a89955f79cda00990cf8bd316cf44365d`). The public manifest SHA256 is `8b910bc4ec497a4bc768cc6c68465c4d2bb8bd6886addfe985c55340265fb542`; this checksum and every source config checksum are required by the runner. An earlier private transport bundle (unused by this PR) had SHA256 `02506cade31f5f0ddd82fd345350f05c08460d1dc3d96682b1a6f2c7f451a6a6`.

## Paired fixed versions

| Layer | Pinned source |
| --- | --- |
| Baseline Core, Engine 0.6.1 | `4abee90f0501f0d8fc09b453a0b45c942fbed69e` |
| Candidate Core, 0.7.3 (not released) | `7466c6f90d420a98c99713b96d6a0623e2aeef52` |
| Shared CLI, v0.5.4 source | `988b69f086ac4a209fe93eb10cf1fa6786977567` |
| Provider/model | Mistral / `ministral-8b-2512` |

Both arms use the exact same question, public source configuration, model, seed and budgets. All **24 pairs of first provider requests** matched byte-for-byte in a Mac-side provider-free preflight; this was also identical to the first-request hashes obtained previously using the private transport format. The GitHub CI branch path builds the native pair and **re-verifies all 24 pairs without making any model requests**. It does not import or recalculate historical benchmark scores.

## Commands and restrictions

To reproduce with the exact pinned native pair, run the following without an API key:

```sh
python3 evaluation/dev-lite/issue531_real24_live.py \
  --repo . \
  --build /path/to/verified-pinned-two-arm-build.json \
  --cases evaluation/engine073-531-real-doc24-20261115/cases.json \
  --output /path/to/new-private-output-directory \
  --mode preflight

python3 evaluation/dev-lite/test_issue531_real24_live.py
python3 evaluation/dev-lite/test_issue531_real24_billing_gate.py
```

**Live inference is not enabled by pushing to this branch.** It is only authorized via the distinct, unused one-shot tag `eval-531-real-doc24-live-20261115-one-shot`, *and only* if a fresh human-confirmed Mistral account/billing check, disabled pay-as-you-go, remaining free allowance, credential/model access and other fail-closed gates pass. The publicly committed source documents are not Secrets. **The Mistral API key must remain a GitHub Actions Secret.**

The prospective maximum is 24 tasks × 2 Engines = 48 CLI runs, at most 12 internal model calls and 28,096 tokens per CLI run, no retry, 4 serial shards, encrypted model raw outputs and seven-day artifact retention. The extreme per-run cap gives **1,348,608 tokens** in total; estimated at a hypothetical symmetric $0.15 per million input/output tokens this is **$0.2022912**, not verified billing or a free quota guarantee. Actual tokens and provider attempts must be measured and reported separately.

**Adoption policy:** Results on these reused, public documents are development diagnostics only. No valid formal independent F1, exhaustive claim audit, or approval to merge the corrected Core into `main` or publish Engine 0.7.3 follows from this run. Refer to #531 for release gating and independent holdout requirements. The source evaluation runner is in a stacked Draft PR on top of #611 and is not itself a release commit.
