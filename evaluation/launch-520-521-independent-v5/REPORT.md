# Engine #520/#521 independent v5 — first observation

Author: `/root/launch_072_independent`. Fresh bounded 50-case offline synthetic corpus, authored after fixed candidate identity without reading runtime implementation/root tests. Every positive uses same-statement policy canonical name or declared alias and declared launch/GA frames. No ambiguous cross-sentence pronoun is used as affirmative proof. No retry, default replay, expectation edit, rescore or runtime change was performed.

**RED: 49/50 passed.** Safety passed: 25/25 negatives withheld, zero false Relevant, zero negative RequestedRelation. Positive recall failed: 24/25 positives passed, one miss (P519). API errors: 0. Cargo exited 101 after all50 observations and aggregate gates were saved.

Candidate: `c6e23e13f66503508c3055a0d0c528b38f72e920`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256 before/after: `3cfb333a44b8924511d4794f537665d723b9512f66861bfd268f47ea92fae127` (unchanged).
Freeze commit: `c4cf4b525e9a4e67251d0b47390d8f2795461b96`, before v5 compilation/API execution.
Corpus SHA-256: `ce46e02fb492aa78c1b5dbd1bc9ebe37ae70af08539c6256e92895754c4bb498` (reverified after first execution).

All cases use exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positives require RequestedRelation and Relevant; negatives require neither. Heading/title govern/deny only; URL/navigation have no authority; factual proof is excerpt/fact/structured_metadata. Relevance is source-local and does not establish external hard truth.

## Sole failure: P519

Input excerpt: “Mernova Badge supports face unlock. That feature claim is false. Mernova Badge went live on 26 May 2228.”

Frozen expected: RequestedRelation and Relevant. Observed v17: identity_scope=exact_target, relation_scope=unresolved, scope_risk=none. Observed v30: disposition=ambiguous, path=conservative_fallback, reasons=[harness_canonical_name_anchor, local_qualification_disagreement, model_ambiguous].

The canonical target directly owns “went live” in the actual affirmative statement. “That feature claim” refers to the preceding feature assertion; it is not used to bridge launch ownership or create an alias. The intended boundary is a non-launch feature denial in separate sentences followed by independent named launch proof. This distinction and expected positive were frozen before the first execution. The result remains a miss, not rescored.

## Passing controls

Positive IDs: P501, P502, P503, P504, P505, P506, P507, P508, P509, P510, P511, P512, P513, P514, P515, P516, P517, P518, P520, P521, P522, P523, P524, P525.

Negative IDs: N501, N502, N503, N504, N505, N506, N507, N508, N509, N510, N511, N512, N513, N514, N515, N516, N517, N518, N519, N520, N521, N522, N523, N524, N525.

All declared affirmative predicate families passed outside P519, including actor/direct-object launched/introduced, has-been-launched, active/passive rollout, is/now/became generally-available, actor has-made, entered-general-availability and reached-GA. Alias/dotted/decimal ownership, on/as-of dates, bounded record adjunct, separate feature questions, direct feature negation, closed same-target feature strike, other-target context and URL/navigation lacking authority passed.

All negative controls passed: fiction/hypothesis/false/denied/withdrawn/retracted context, plural references, original ~~ / ❌ / ≠ / ¬ / ✗ / ✘ / combining strike, marked-target versus unstruck other product, conditional/question/rumor, introducing actor, wrong product, cross-sentence pronoun-only proof, title-only proof and URL/navigation-only proof.

## Evidence

- `freeze.json` / `first-run-start.json`: frozen and first-execution provenance.
- `first-observations.jsonl`: complete input and both unmodified API outputs/errors for all50 cases.
- `first-failures.json`: exact one-failure subset.
- `first-summary.json`: immutable initial gates and before/after source identity.
- `first-run.log` / `first-run-exit-status.txt`: complete compiler/test output and exit101.
- `artifact-sha256.json`: hashes of frozen corpus/protocol/runner and saved first evidence.

Explicit recording mode exclusively creates first files. Default replay performs no filesystem writes or dependence on file existence. No default replay followed the initial observation in this evaluator task. All earlier initial results/scope notes remain unchanged. Root owns diagnosis/repairs, and these observations alone do not establish whether the miss was introduced by this candidate.

The initial raw evidence was committed in `04bebac2205c8fb6bb0e97c869e9a8c1ee95f005`. A syntax error in report-generation Python prevented the derived report/failure subset/hash files from being created in that commit; this was corrected without invoking the APIs, rerunning the corpus, or modifying initial files.

Offline synthetic source-local public-API relevance only; no provider, human, user, device, production or external factual-truth acceptance.
