# Issue #531 / #613 — prospective independent 24-case acceptance, preflight only

**Status on 2026-11-16 JST: prepared source pool and corrected pair; HOLD independent model quality, billing authorization, and release.** No new formal F1, first observation, candidate GO, production CLI adoption, or user-billed provider activity is claimed.

## Why a new evaluation must not use the old native pair

The cancelled historical [Dev24 first-observation run](https://github.com/git-ksk/reasoning-harness/actions/runs/43667947455) used **0.6.1 Core `4abee90` vs unreleased 0.7.3 candidate Core `7466c6`** with the **old CLI `988b69f`**. That CLI lacks the #613 improvement. Simply rerunning it would measure the old unfixed path, and the already-used questions cannot supply independent quality evidence.

The new **provider-free paired builder** (`build_corrected_pair.py`) exports the same **corrected CLI `b85a4a0`** for both arms, replaces only each arm's native Core tree with the pinned historical Core, and applies to **both** exactly the same canonical #552 Core patch and the narrowly reviewed #613 Core read-only Reject/acquisition patch. The reviewed patch bytes are **vendored as two canonical base64-encoded files** under `patches/` and SHA256-checked independently of Git diff rendering (macOS/Linux differences). The target code is independently verified after application. The script proves all **non-Core/non-lock** files of both source trees are identical, including the CLI and provider harness. No additional Core changes are allowed. The candidate is an **unpublished 0.7.3 development candidate**; its Cargo manifest still says `0.7.2`. Version labels must not be misreported as an actual release.

On the user's Mac, both isolated native ARM64 binaries **built successfully offline**, with binary SHA256:

| Arm | Built binary SHA256 | Source revision |
| --- | --- | --- |
| 0.6.1 baseline | `ca6ebedc3201d052cc658b72ef512e87cb3411978d9c38b6cca77ffa41a80e6c` | `4abee90f0501f0d8fc09b453a0b45c942fbed69e` |
| candidate | `54319dde88d750d8ab63c9c5c2002933bfa234e70805b9a1fff5018393fff699` | `7466c6f90d420a98c99713b96d6a0623e2aeef52` |

The private build proof, source inventories, and binaries remain **only on the Mac**, under `~/Library/Application Support/reasoning-harness/issue531-independent24-20261116/paired-b85a4a0/`. No credentials/gold or binary artifacts were committed. The **already-consumed public Dev24 questions** were used *only as a non-generative paired-request identity check*: 24/24 pairs, 48 planned first requests, 24 distinct request digests, **both arms byte-identical**; **model calls 0, scores null**. This is a smoke test of the corrected candidate's build and execution-plan parity, **not a new F1**.

### Offline repeatable commands

```sh
python3 -m unittest discover \
  -s evaluation/engine073-531-independent24-20261116 -p 'test_*.py' -v
python3 evaluation/engine073-531-independent24-20261116/validate_readiness.py --repo .
python3 evaluation/engine073-531-independent24-20261116/build_corrected_pair.py \
  --repo . --output /path/to/unique-private-output --mode validate-only
# Optional, no-provider but resource-intensive native build; use a different unused output path:
python3 evaluation/engine073-531-independent24-20261116/build_corrected_pair.py \
  --repo . --output /path/to/another-private-output --mode build
# The built pair can then verify 48 first requests with already-consumed public
# cases for wiring parity only (never re-infer or re-score them):
python3 evaluation/engine073-531-independent24-20261116/verify_paired_first_requests.py \
  --repo . --build /path/to/another-private-output/pair-preflight.json \
  --output /path/to/unique-first-request-audit
```

## Newly pinned primary documents — provisional source pool

`provisional-new-sources.json` contains **12 original upstream files in five disjoint GitHub projects**: Python CPython, Node.js, The Rust Programming Language book, Kubernetes upstream docs and PostgreSQL upstream docs. The preparer used unauthenticated public GitHub metadata and **exact SHA-pinned source bytes**, saved each **whole-source SHA256** and verified original development documents were in **six different products** (curl, Git, CMake, OpenSSL, SQLite and Redis). The pool manifest itself is SHA256 `f5b814e66d715c7c7b5e413348e7a0526c98956c03079c976254518381fdf527`.

No complete copyrighted source documents, answers, questions, gold labels or raw model outputs are stored in this repository. All 12 upstream pinned links are public. Distinct sources are **necessary but not sufficient** for genuine independent assessment: the repair implementation author participated in selecting them. This pool is **not yet an independently authored question set**, and its case count is **0**.

## Frozen prospective gate and remaining blockers

`holdout-readiness.json` and `validate_readiness.py` freeze the proposed comparison requirements **before** any new model results. The expected independent assessor's work is **24 never-consumed questions**, with **16 answerable / 8 negative**, covering primary evidence, source scopes, independent vs duplicated origins, contradiction, changes, withheld/private-only facts, and tricky paraphrases. Answers need the exact final *visible* claim(s), not provider candidate text, a quotation or a model self-assessment. **An independent reviewer, not the implementation author**, must separately author/review the new cases, freeze source/question/gold and validate the scorer's integrity before the first model request. Private gold stays off GitHub and must not enter candidate prompts. The frozen quality minimums are: **case F1 ≥0.90, Precision ≥0.95, Recall ≥0.85, zero unsafe assertions on negatives, zero invalid/fabricated citations, no candidate-vs-baseline regression**, all 48 case×arm results independently verified. Timeout, truncation, unknown usage, or missing citations never convert to success. A score can be computed only on the first complete observations, not a tuned post-hoc replay.

### Billing and job isolation

No API entitlement has been verified on the current turn. The prospective Mistral model is `ministral-8b-2512`. Each CLI is bounded by **≤12 model operations, ≤28,096 tokens**, yielding 48 planned CLI calls and a **worst-case envelope 576 model operations / 1,348,608 tokens**. These are bounds, **not a guarantee of no billing**. The earlier same-account Mistral read-only gate must independently recheck *organization + API budget pay-as-you-go OFF, included budget headroom and current model* before every newly authorized shard. No model execution is permitted on an unverified or failing gate. The [#613 eight-independent-job design](../../docs/issue-613-engine073-offline-20261115.md) is a **provider-free planning reference**, not a reused real24 one-shot command. Future independent model-bearing jobs must be separately reviewed, tagged and explicitly approved, with distinct job timeouts, encrypted partial raw output, no automatic retries and no reexecution of a possibly charged in-flight attempt. **No effectful new workflow, tag or model call is added here.**

**Current blocking gates:** independent reviewer and private 24-question gold absent; source/question/gold not frozen; fresh 24-pair first-request parity not available; current same-account no-overage account read-only gate not checked; independent actual-model quality evidence and F1 absent. `quality_go=false`, `release_go=false`; PR #614 and Issue #613 should remain HOLD for Engine 0.7.3/main/CLI promotion.
