# Issue #616: independent24 author draft — source-backed and sealed, **HOLD**

**2026-11-16 JST.** This is an author-prepared *candidate holdout*, not independent reviewer acceptance, a formal Engine 0.7.3 F1, a provider billing authorization, or permission to release/merge. Existing source-packet templates and sources are intentionally left unchanged.

## New evaluation material (separated by trust boundary)

| Material | Location | Visibility |
| --- | --- | --- |
| Questions (24, two per pinned source) | `independent24-public-cases.json` | public: question, source id, exact official URL, upstream commit and full-file SHA256; NO answerability labels or expected facts |
| Author draft freeze | `independent24-author-freeze.json` | public: immutable SHA256 references, Core/CLI/model/seed/budgets, strict gates; explicitly NOT approved |
| Author Gold v2 | `~/Library/Application Support/reasoning-harness/issue531-independent24-20261116/independent-author-draft-v1/gold-24-v2.json` | private Mac-only 0600, parent 0700; never commit/log/upload |
| Original docs + blank slots | `.../independent-author-source-packet-v1/` | immutable original Mac-only packet; source bytes and 24 blanks not edited |
| Handoff state | `independent24-review-pending.json` | public: no reviewer, no signature, GO false |
| Offline auditor | `validate_independent24.py` and `test_*independent24.py` | public code, no model, API key, payment, output scoring or new workflow |

**Public question file SHA256:** `be20db67a4cf0545bde79234ec9a4429b8854bf97d42a4fdd410b8789b9208e6`.

**Mac-only private Gold v2 SHA256:** `08220bbb3a9c4cb23c4212c8a4dbdc1fd26f65746551115c927f98ded6e0663f`. This is a **draft content fingerprint**, not a reviewer signature or verified independence claim.

**Original official source inventory SHA256:** `f5b814e66d715c7c7b5e413348e7a0526c98956c03079c976254518381fdf527`. Full-source bytes 12/12 verified in strict packet audit. The 12 source files cover CPython (3), Node.js (3), Rust Book (2), Kubernetes (2), and PostgreSQL (2). Compared with the consumed Dev24's upstream source audit, **zero URL/SHA overlap**; compared with all 24 consumed Dev24 questions, zero exact matches and a maximum 5-token-shingle Jaccard similarity of 0.0. Source disjointness is not a substitute for independent *human* judgment.

The draft has **16 answerable / 8 negative** labels *only in private Gold*, with 54 individually scored facts, 70 exact quoted original evidence spans, original file sha/commit/url, line numbers, version and applicability, allowable paraphrases, disallowed claims, and per-negative withholding reasons. Negatives probe absent runtime context, missing performance evidence, timing, version mismatches, another product's scheduler, unverifiable cluster provenance, a contradictory Deployment constraint, and absent PostgreSQL live configuration. No new model request was made.

## Fixed evaluation envelope and unresolved gates

- Core 0.6.1: `4abee90f0501f0d8fc09b453a0b45c942fbed69e`; Core candidate: `7466c6f90d420a98c99713b96d6a0623e2aeef52`; common corrected CLI: `b85a4a0b2ab03397b361a8620d7a97dd26b2eb36`. Candidate is not an Engine 0.7.3 release.
- Provider/model: Mistral / `ministral-8b-2512`. Intended first-request random seed = `49200 + zero-based case index`; max 1,024 output tokens for the first candidate request; `temperature` and `top_p` were not explicitly supplied in prior provider-free first-request plans. This is a *planned sampling contract* derived from prior plan metadata: **new-case 48-request parity has not been demonstrated** and must be re-proven before use. Maximum per CLI: 12 model calls / 28,096 tokens. Exactly 24 cases × 2 arms = 48 first observations required.
- Frozen release gates: case F1 ≥0.90, precision ≥0.95, recall ≥0.85; zero unsafe claims on negatives; zero invented/invalid citations; no Core candidate regression relative to baseline. Grade only externally displayed final answers, not internal candidate text, retrieved source quotations or scorer claims. Unknown, missing, timed-out or incomplete outcomes NEVER count as positive or safe passes.
- **No independent reviewer has been enrolled or approved**. Same assistant session authored and self-checked Gold; it cannot act as its own second reviewer. The original source pool was selected by the correction implementer; different authorship/independence identity must be verified externally, not inferred from file hashes.
- **No new-case first-request plans/configs have been promoted**. Existing Dev24 configs contain harness-owned factual propositions and MUST NOT be copied verbatim or injected into new prompt inputs: this could disclose Gold and compromise the holdout. New configs/runner are an independently reviewed, separate future preflight. Public case manifest is not an executable live run.
- Current read-only **same-Mistral-account** organization pay-as-you-go OFF, API-budget pay-as-you-go OFF, remaining included balance and model availability were not checked here. No API key, billing settings, live workflow, tag, old cancelled run replay, or 48 observations were used.

## Offline verification

On the Mac, from the repository root:

```sh
python3 evaluation/engine073-531-independent24-20261116/reviewer_handoff.py \
  --repo . --mode audit \
  --packet "$HOME/Library/Application Support/reasoning-harness/issue531-independent24-20261116/independent-author-source-packet-v1"

python3 evaluation/engine073-531-independent24-20261116/validate_independent24.py --repo .

python3 evaluation/engine073-531-independent24-20261116/validate_independent24.py \
  --repo . \
  --gold "$HOME/Library/Application Support/reasoning-harness/issue531-independent24-20261116/independent-author-draft-v1/gold-24-v2.json"

REASON_616_PRIVATE_GOLD="$HOME/Library/Application Support/reasoning-harness/issue531-independent24-20261116/independent-author-draft-v1/gold-24-v2.json" \
  python3 -m unittest discover -s evaluation/engine073-531-independent24-20261116 -p 'test_*.py' -q
```

The repository CI runs only the public integrity check and public/offline tests; no private packet or Gold is uploaded, echoed or required in CI. A small Python 3.9 compatibility fix replaces `zip(...,strict=True)` in the original packet builder, where both array lengths are already separately checked, avoiding the local 3.9.6 test failure without weakening validation.

## Exact conditions for independent reviewer handoff / GO

An **independent authenticated second reviewer** must inspect the full 12 sources, 24 questions and Mac private Gold in a trusted local setting and check each fact/quote, scope, version, negative absence/contradiction, question originality and answer-label leakage. Reviewer must document provenance, sampling balance and scorer rules; issue a *separate*, verifiable approved receipt/signature bound to the immutable public question SHA, private Gold SHA, all three Core/CLI commits, model and scorer. Neither author nor implementation executor can self-approve. Any correction creates a **new** frozen revision and invalidates any old review.

Only after reviewer acceptance: create Gold-free immutable model input configs, verify 24 × 2 **fresh** first-request parity (no provider calls), prove no score/label leak, authorize a new 8-shard one-shot with bounded independent jobs, read-only fresh same-account no-overage billing proof and an explicit separate execution approval. Do not reuse or replay the cancelled historical run. Only the 48 first final-visible answers are scored, and unknown outcomes block GO. PR #614/#615 stay Draft and unmerged; #531/#613/#616 remain open; Engine 0.7.3/main/normal CLI remain **HOLD**.
