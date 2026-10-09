# Engine 0.7.0 — Fresh independent three-provider acceptance v1 (#492)

[日本語](engine-0.7-independent-v1-runner-freeze.ja.md)

**Status: evaluator, protocol and scoring freeze candidate. The fresh independent holdout must not exist before the evaluator has been merged and immutably tagged.**

## Precommitted candidate and safety gate

The candidate source commit is `4cae9326bcc3e210c3250f05772d84f00e41b645`. The Engine 0.6.1 baseline source finalizer is unchanged. Compare the original `finalize_source_attributed_answer` and the additive `reconcile_source_attributed_targets` on the same target, exact source text, scope and version.

Pinned providers and model IDs: Mistral `ministral-8b-2512`, Google `gemini-3.5-flash-lite` and Groq `openai/gpt-oss-120b`. Each must complete the same 12-case never-before-observed corpus, including at least four compatible cases and six negative/conflicting/unknown/context-mismatched targets. These targets are measured individually; global original Conflict, every original citation and literal source text remain unchanged.

A model may suggest equivalent versus non-equivalent wording with no more than **one additional advisory call per exact target** (no provider substitution). It is **never** allowed to authenticate its own equivalence judgment. Only a separately precommitted, controlled evaluation-oracle label may authorize a compatibility review. That controlled label is not a claim of external factual truth. The provider-attempt cap is two per target, with zero internal retry wait and deterministic 10.5-second external pacing. Strict JSON parsing, exact target/source binding and no synthetic source-acquisition are mandatory.

Acceptance requires **at least one net useful source-grounded answer gain per provider**, zero fabricated/erased/misbound citations, zero uncontrolled review/unknown origin truth/temporal promotion, zero known/verified truth promotion, unchanged legacy v1 Conflict and prose, deterministic replay, zero replay side effects and zero operational failures. Record provider attempts, model calls, input/output tokens and error types separately; quota/transport failures never count as epistemic uncertainty.

## Freeze and observation sequence

1. Merge the protocol `evaluation/engine-0.7-independent-v1-protocol.json`, Rust runner, scoring/freshness verifiers, and CI. Create/push the immutable `engine-0.7-independent-v1-runner-freeze` tag.
2. **Only after that tag**, author new independently disjoint cases and quotes. The frozen freshness validator rejects reused case/source/target identities and overlapping eight-token passages. Commit and tag `engine-0.7-independent-v1-corpus-freeze`.
3. The corpus tag automatically triggers three independent credential-isolated provider jobs. Results use create-new files and append-only flushed checkpoints, so even a failed or partial first run is immutable.
4. Apply the **unchanged, previously frozen** cross-provider scorecard. Any missing provider, failure or hard violation blocks Engine 0.7.0. Preserve first FAIL; use a successor identity instead of editing history.

No published CLI/Engine versions change at this step. Additional session/migration/no-Rust consumer and release checks in [#492](https://github.com/git-ksk/reasoning-harness/issues/492) remain mandatory.