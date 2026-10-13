# Independent v7 bounded grammar acceptance protocol

Author: `/root/launch_072_independent`, the same single authorized evaluator. 50 fresh offline synthetic inputs: 25 positives and 25 negatives. Authored after fixed candidate identity without implementation, root regression or prior reproduction reads. The evaluator knows its earlier results and explicit public contract updates, but does not reuse known inputs or rescore earlier evaluations. The generic recorder/replay structure is reused from v2; input corpus, expectations and artifact identities are new.

Candidate commit: `19a8faaa582c1d4af7a68972c494b2e29f72d80e`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256: `1e6293feab2c5eb7428a4427c7de9dc2d635a01fe9de14036db64085ffdde315`.

Before any v7 compilation/API call, commit the corpus, expected outcomes, manifest, this protocol and formatted runner, then announce the freeze commit and corpus hash. Record pre-execution provenance in `evaluation/launch-520-521-independent-v7/freeze.json`. `corpus.json` contains complete case inputs and v17/v30 expectations; `manifest.json` contains source/corpus identity, counts, IDs and gates; `author_corpus.py` is the independent authorship specification.

The objective is corrective acceptance of existing bounded local grammar, not unbounded expansion of context nouns or release predicates. Every positive uses the policy canonical name or declared alias as direct subject/object in the actual launch statement. Supported predicates only: launched/was-launched/has-been-launched, introduced/was-introduced, went-live, rolled-out/was-rolled-out, is-(now)-generally-available/became-generally-available/has-made-target-generally-available, entered-general-availability/reached-GA. On/as-of day-May-year and bounded “according to the release record” adjuncts are used. Coreference never creates positive ownership; pronoun-only launch in another sentence is negative.

Known feature predicates and has-a/an properties are separate non-launch statements; their adjacent singular references remain local to those attributes. Broad report/passage disclaimers remain governing, including when a feature statement is adjacent. Explicit different-target contexts and closed marks on unrelated non-launch features cannot invalidate separately proven launch. Original ~~ / ❌ / ≠ / ¬ / ✗ / ✘ / combining strike scope must remain intact. Only excerpt/fact/structured_metadata supply factual proof; heading/title may govern/deny but cannot prove; URL/navigation have no factual/governing authority.

Each case directly calls `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30` with exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positive requires RequestedRelation AND Relevant; negative requires neither. Any API error fails. Source-local qualification does not establish external hard truth.

Safety gate: zero negative RequestedRelation, zero false Relevant, zero API errors. Positive-recall gate: zero positive misses, zero API errors. Both gates plus unchanged candidate source hash are required for first-observation PASS. Record every complete input/output/error before gate assertions.

First-observation command:

```sh
LAUNCH_INDEPENDENT_V7_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v7 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v7 --jobs 2 -- --nocapture
```

Store stdout/stderr in `evaluation/launch-520-521-independent-v7/first-run.log`, numeric exit in `first-run-exit-status.txt`. Explicit first mode verifies fixed source hash before APIs, exclusively creates `first-run-start.json`, streams complete unmodified per-case input and both outputs/errors into `first-observations.jsonl`, then exclusively creates `first-summary.json` with gates and source hashes before/after. Preserve compile errors, partial observations and full initial results. First evidence cannot be overwritten.

Default integration execution without `LAUNCH_INDEPENDENT_V7_RECORD_FIRST=1` is side-effect-free replay: no filesystem writes, no dependency on first-file existence, no historical candidate hash enforcement. It asserts the same frozen expectations. Later replay cannot rescore or replace the initial observation or be called fresh.

No retries, rescoring, expectation edits, runtime changes, branch changes, merge, Oracle or additional agents. Root owns diagnosis/repairs. Earlier first records and scope notes remain immutable. This is offline synthetic source-local public-API relevance only, with no provider, human, user, device, production or external factual-truth acceptance.
