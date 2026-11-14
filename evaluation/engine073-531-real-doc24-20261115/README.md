# #531 — 24 previously consumed **real upstream documents** / Dev V5 comparison

**State at preparation: 0 provider model calls; provider-free paired preparation only.**

This comparison uses exactly **24 questions** over **12 actual upstream documents** from pinned curl, Git, CMake, OpenSSL, SQLite and Redis Git source commits. This contrasts sharply with the self-authored fictitious 24-case Dev cohort from #611. These 24 questions themselves were **previously consumed in Oct 2026** and are therefore **not new independent blind holdout cases**. The old real-world corpus labels and any old model outputs are **not supplied to the new runtime**. Any resulting score is only a **development diagnosis**.

- Common CLI source `988b69f086ac4a209fe93eb10cf1fa6786977567`.
- Old Engine Core `4abee90f0501f0d8fc09b453a0b45c942fbed69e` (0.6.1); corrected Engine 0.7.3 candidate Core `7466c6f90d420a98c99713b96d6a0623e2aeef52`.
- Same native CLI code, provider `mistral`, model `ministral-8b-2512`, seed and per-task budget for both arms, 24 paired first requests checked *before* inference.
- Frozen private bundle SHA256 `02506cade31f5f0ddd82fd345350f05c08460d1dc3d96682b1a6f2c7f451a6a6` and original `requests.json` SHA256 `d53410e21cf899d562d63d72d22c392a89955f79cda00990cf8bd316cf44365d`. Its 24 original source config hashes are checked individually. The 12 publicly reachable upstream original files were independently re-downloaded by SHA256, **12/12 exact source hashes**; all their original source quotes are found after whitespace normalization. No changed URLs or reconstructed evidence.
- Original local semantic gold SHA256 `27f202d941f606d5b9e32f43f05583e59e8d42a42b2ec51ab0d68c06217b1cc1`, **kept off Actions**. Its previous score is not imported to this experiment. The raw 24 config bundle exists only on the user's Mac and in an Actions repository secret; never stored in this PR.
- Live is permitted **only** via the designated unused single-action tag `eval-531-real-doc24-live-20261115-one-shot` after current same-account authenticated Mistral free headroom and pay-as-you-go disabled have been confirmed **within 30 minutes** and provisioned under a distinct attestation secret. Branch-push and manual dispatch are *provider-free*; a missing or expired attestation fails closed.
- Up to 48 real paired CLI runs, 12 calls and 28,096 total tokens per arm at maximum, and serial shards of 6 cases (12 CLI runs), no automated retry, encrypted raw output, 7-day artifact retention. The absolute theoretical tokens are 1,348,608; at **assumed** $0.15 / 1M combined token billing, estimated maximum $0.2022912, **not** a verified invoice or allowance balance.
- Do **not** start the new tag before reviewing the first-request parity CI and fresh budget status. Run release/CLI adoption gates separately: no independent holdout, independent reviewer, multi-claim scoring, or release authority comes from this experiment.

Run offline test controls:

```sh
python3 evaluation/dev-lite/test_issue531_real24_billing_gate.py
python3 evaluation/dev-lite/test_issue531_real24_live.py
```

The evaluation runner and Actions workflow are in this stacked development PR on top of the pending #611 runner. Do not merge the corrected Core into main, publish Engine 0.7.3, or silently claim formal F1 based on this pilot.
