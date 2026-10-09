# Engine 0.7.0 — Source reconciliation development v1 (#490)

[日本語](engine-0.7-source-reconciliation-v1.ja.md) | English

**Status: development candidate validated deterministically; no release, no independent live provider acceptance.** The published Engine remains 0.6.1, and Reason CLI remains 0.5.4.

## Measured motivation and scope

[#487's frozen Engine 0.6.1 baseline](engine-0.7-baseline-v1-result.md) measured 29/29 predicted deterministic behaviors, including a specifically reproducible avoidable source-local Conflict: two exact source quotations for the *same target* convey equivalent beta status but differ as strings. Existing `source_attribution.rs` marks all distinct string values for a target as Conflict, including this safe conservative case. **This is not an unsupported Known/Supported factual-verdict error.**

This v1 candidate adds an **opt-in, separately versioned metadata view** for an explicit, genuinely **host-reviewed** source-local equivalence pair. It does **not** change `SourceAttributionState`, `refresh_conflict_states`, `validate_source_attribution_state`, `finalize_source_attributed_answer`, historical artifacts, source citations, or the existing hard-verification/finalization pipeline.

## Host-only API and trust boundary

| Item | Contract |
| --- | --- |
| Capture | `capture_source_review_anchor(artifact, claim_id)` snapshots the exact original target policy, source-local claim, bound source binding and complete admitted evidence including observation, scope, time and provenance metadata |
| Authority | `TrustedSourceReviewAuthority::new(policy_id)` must be invoked **only by the trusted host** after authenticating the named human/oracle reviewing policy; no model/source input may create or select this capability |
| Human/oracle review | The host inspects **both exact source quotes** and establishes they convey a compatible statement **in context**; only then may it call `record_trusted_source_equivalence(...)` to persist a versioned review record |
| Reconciliation | `reconcile_source_attributed_answer(artifact, target_ids, Some(&authority), &reviews)` revalidates each review against the complete current artifact and the separately supplied host capability |
| Output | `SourceReconciliationView` includes a `status` of `reviewed_compatible`, `qualified`, `conflict`, or `unresolved` plus the **entire original** `SourceAttributionFinalization`, reviewed targets and remaining conflicts |
| Wire IDs | `harness-source-compatibility-trusted-review-v1` and `harness-source-reconciliation-view-v1`, separate from all source-attribution v1 identities |

**The library does not implement automatic semantic equivalence.** It cannot verify that a trusted reviewer made a correct linguistic judgment; authorizing incorrect human/model-derived reviews would compromise the *reviewer's* semantic assertion. This trust capability must **never** be exposed to the model, MCP read-only result, third-party document, browser text, or a client-editable review file. A stored review alone is not authorization: the host must independently restore its trusted policy on every replay.

Even a valid review **does not change either original Conflict flag or the legacy result's Conflict status**. The new view may be `reviewed_compatible`, but `view.original.status` remains `conflict` and its text still contains **both separately cited exact quotes**. This avoids promoting external truth, misquoting one source as the other, inventing a new combined statement, or silently removing disagreement from historical sessions. No provider/model calls or acquisition take place in the overlay.

## Default refusal and controls

- No review, unknown equivalence, or incomplete **pairwise** review coverage for a target => `conflict`. Three-source transitive chains are **not** promoted to full equivalence.
- Unauthenticated policy ID, mismatched review contract, duplicate/reused pair, wrong target, hard-verification target, tampered source observation/binding/version after replay, or duplicate target selection => reject.
- Only exact-quote claims with **two distinct** source passages qualify for this narrow candidate. Transformed paraphrases are deliberately not taken as independent trusted review receipts.
- Differing Harness-owned evidence scope, temporal/authority metadata, or explicit source version => reject compatibility, even when retrieved later. Never infer source independence or supersession.
- Explicit lexical opposition, negation, changed numbers, missing/present future/conditional modality, past-time changes and narrowed/universal scope markers => refuse even if a reviewer attempts approval. These hard guards are **not** general-purpose semantics; the trusted reviewer remains responsible for all other meaning and temporal applicability.
- The view is bounded: at most 16 claims per reviewed target and 1,024 review records per request; overlarge target groups stay conservative. It never performs external I/O, provider retry or session mutation.

## Frozen development evidence

| Field | Exact identity |
| --- | --- |
| Pre-implementation frozen specification | `engine-0.7-reconciliation-dev-v1-spec-freeze` → `426b70195623d1b4d7196c5d6a1ab4a2272741c6` |
| Fixture SHA-256 | `8d50e04a24d09a2620827b15639a89c396b100dfa5d104baa69b3b9357190714` |
| Input | `fixtures/engine-0.7-source-reconciliation-development-v1/manifest.json` |
| Rust tests | `crates/reasoning-harness-core/tests/source_reconciliation_development_v1.rs` |
| Report | `evaluation/engine-0.7-reconciliation-dev-v1-result.json` |
| Integrity and report parser | `scripts/engine-0.7-reconciliation-dev-v1-report.py` and `scripts/test_engine_0_7_reconciliation_dev_v1_report.py` |

The frozen surface predicted **22** deterministic development outcomes, and all **22/22** matched. Counts: **4 reviewed-compatible**, **9 retained Conflicts**, **7 fail-closed errors**, **2 unchanged qualified**. Both original source quotations/citations survive in compatible scenarios. Zero hard factual promotions, model calls, provider attempts, external acquisition calls or mutations. Additional *post-freeze* regression tests cover model/reviewer impersonation, replay tampering, review order invariance, conditional/future/past modality and scope/quantity strengthening; they are **not** retroactively inserted into the frozen 22-case denominator.

Reproduction:

```bash
cargo test --locked -p reasoning-harness-core \
  --test source_reconciliation_development_v1 -- \
  --test-threads=1 --nocapture > /tmp/engine-0.7-reconciliation-dev-v1.log 2>&1
python3 scripts/engine-0.7-reconciliation-dev-v1-report.py \
  --log /tmp/engine-0.7-reconciliation-dev-v1.log \
  --output /tmp/engine-0.7-reconciliation-dev-v1-result.json
cmp evaluation/engine-0.7-reconciliation-dev-v1-result.json \
  /tmp/engine-0.7-reconciliation-dev-v1-result.json
```

CI retrieves the frozen tag by exact SHA, compares frozen fixture bytes, and runs all report-integrity/forgery checks. The first observed development result is preserved; further changes to semantics require a *new* candidate identity and evidence, not a rescore.

## Remaining acceptance — #490 is NOT finished

1. **Connect an authentically trusted reviewer/oracle workflow**, not model self-attested equivalence. Measure whether approval is operationally viable and whether source-local compatibility genuinely improves useful terminal output. No automatic reviewer policy can be delegated to provider model output.
2. Precommit and freeze candidate + evaluator, then author an **independently fresh** holdout without reused entities, quote windows or examples; compare identical-input Engine 0.6.1 with candidate separately on **Mistral, Google and Groq**. Require at least one net useful answer gain per arm and all hard safety violations zero, including true opposition/conflict preservation.
3. Do not silently change the v1 source-attribution/CLI machine envelope or old persisted sessions. Any future CLI adoption and Engine 0.7 source release require [#492](https://github.com/git-ksk/reasoning-harness/issues/492) exact-head CI/release gates and separate tags. No version bump in this PR.

The new API is intentionally **development-only/opt-in** and is not connected to Reason CLI commands or automatic model/summarization flow.
