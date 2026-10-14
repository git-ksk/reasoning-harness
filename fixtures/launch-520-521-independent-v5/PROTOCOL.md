# Independent v5 bounded-contract protocol

Author: `/root/launch_072_independent`, the single authorized independent evaluator. This new corpus has 50 inputs: 25 positives and 25 negatives. It was authored after fixed candidate identity without runtime implementation, root regression or prior reproduction reads. The evaluator knows its own previous results and explicit contract clarifications. No previous corpus was renamed, rescored or relabeled. Recorder/replay structure comes from v2; inputs and expectations are new.

Candidate: `c6e23e13f66503508c3055a0d0c528b38f72e920`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256: `3cfb333a44b8924511d4794f537665d723b9512f66861bfd268f47ea92fae127`.

Freeze all cases, expectations, this protocol and the formatted runner in a commit before any v5 compilation/API execution. Announce the freeze commit and corpus SHA-256 first. `corpus.json` records complete inputs/expected outcomes; `manifest.json` records source identity, corpus hash, case IDs/counts and gates; `author_corpus.py` records the independent input specification. Store pre-execution provenance in `evaluation/launch-520-521-independent-v5/freeze.json`.

The evaluation checks the existing bounded contract rather than requesting unbounded new nouns or predicates. Affirmative vocabulary stays within launched/was-launched/has-been-launched; introduced/was-introduced; went-live; rolled-out/was-rolled-out; is-(now)-generally-available/became-generally-available/has-made-target-generally-available; entered-general-availability/reached-GA. On/as-of day-May-year dates and “according to the release record” are the bounded adjuncts. Each positive has the canonical name or declared policy alias as direct subject/object in the actual affirmative statement. Cross-sentence pronoun ownership is negative, even after a naming record.

Only excerpt/fact/structured_metadata furnish factual proof. Heading/title may govern or deny but cannot prove. URL/navigation have no factual/governing authority. Fiction/hypothesis/false/denied/withdrawn/retracted contexts, including ordinary plural references, must govern their actual statements. Raw ~~ / ❌ / ≠ / ¬ / ✗ / ✘ / combining strike scope must remain intact. Explicit other-target context and clearly unrelated feature questions/feature denials in separate sentences must preserve an independent affirmative launch.

Call both public APIs directly for every frozen case: `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30`, with exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positive requires effective RequestedRelation AND materialized Relevant; negative requires neither. Every API error fails the case. Qualification is source-local only and never external hard truth.

Safety gate: zero negative RequestedRelation, zero false Relevant, zero API errors. Positive-recall gate: zero positive misses, zero API errors. Both gates and unchanged candidate source hash are required for initial PASS. Save every observation before asserting gates.

Explicit first-observation command:

```sh
LAUNCH_INDEPENDENT_V5_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v5 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v5 --jobs 2 -- --nocapture
```

Capture full stdout/stderr in `evaluation/launch-520-521-independent-v5/first-run.log` and exit status in `first-run-exit-status.txt`. Explicit first mode verifies the source hash before APIs, exclusively creates `first-run-start.json`, streams complete inputs/unmodified outputs/errors into `first-observations.jsonl`, and exclusively creates `first-summary.json` with gates and before/after source hashes. First files cannot be overwritten. Preserve any compile/runtime/partial failure and complete initial outcomes.

Normal integration execution without `LAUNCH_INDEPENDENT_V5_RECORD_FIRST=1` is side-effect-free corpus replay with frozen expectations: no filesystem writes, no dependence on existing first files, no historical candidate hash enforcement. Later development replay cannot rescore the initial observation or be called fresh.

No retry, rescore, expectation edit, runtime edit, branch change, merge, Oracle or further agent. Root owns fixes/integration. Existing earlier RED records/scope notes remain unchanged. This is offline synthetic source-local public-API relevance only, not provider, human, user, device, production or external factual-truth acceptance.
