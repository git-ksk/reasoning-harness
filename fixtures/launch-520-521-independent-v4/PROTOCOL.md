# Independent v4 frozen protocol

Author: `/root/launch_072_independent`, the same single authorized independent evaluator. Fresh 52-case corpus: 26 positive and 26 negative. Inputs were authored after receiving the new fixed identity without reading runtime implementation, root repair tests or prior reproduction material. The evaluator retains knowledge of its own earlier results and explicit scope clarifications; no earlier cases were renamed, reused or rescored. The v2 recorder/replay harness structure is reused, with new inputs/expectations and v4 artifact identity.

Candidate commit: `48ac1ba8aba655aa73d192962d7996bcc997c5e0`.
Candidate source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256: `2b32a1e4a721cb169b15420e113ad91808476380ff912c12b46c029f8f81684d`.

The case IDs, inputs and v17/v30 expected outcomes are frozen in `corpus.json`. `manifest.json` records source/corpus hashes, counts, IDs and both gates. `author_corpus.py` records the exact fresh input specification. Commit this protocol, corpus/manifest and formatted runner before any v4 compilation or API execution. Announce the freeze commit/hash before executing, and record them in `evaluation/launch-520-521-independent-v4/freeze.json`.

Every positive has the policy canonical name or Harness-declared alias owning the affirmative predicate in the same actual statement. No cross-sentence pronoun/alias creation is allowed. Cross-sentence “It” after a naming record is a frozen negative. Positive predicates are limited to launched/was-launched/has-been-launched; introduced/was-introduced; went-live; rolled-out/was-rolled-out; is-(now)-generally-available/became-generally-available/has-made-target-generally-available; entered-general-availability/reached-GA. Known on/as-of day-May-year adjuncts and the bounded “according to the release record” adjunct are included. Actor/direct-object binding is explicit and must refer to the launched product, not the introducing actor.

Only excerpt/fact/structured_metadata furnish positive factual proof. Heading/title may govern/deny but cannot prove a launch. URL/navigation cannot furnish or govern factual proof. Governing fiction/denial/retraction/hypothesis is tested with fresh source descriptions, not assumed to depend on any fixed context noun. Original markers and strike spans must preserve actual target/statement scope. Neutral prose, explicitly different targets, and clearly unrelated non-launch property statements/questions must not invalidate a separately proven launch.

All cases call both public APIs directly: `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30`, each with frozen exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positive requires RequestedRelation AND Relevant; negative requires neither. Every API error fails the case. Outputs are source-local relevance only and never externally verified hard truth.

Safety gate: zero negative RequestedRelation, zero false Relevant, zero API errors. Positive-recall gate: zero positive misses, zero API errors. Both gates plus unchanged candidate source hash are required for first-observation PASS. All observations must be saved before gate assertions.

Explicit first-observation command:

```sh
LAUNCH_INDEPENDENT_V4_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v4 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v4 --jobs 2 -- --nocapture
```

Capture stdout/stderr in `evaluation/launch-520-521-independent-v4/first-run.log`, numeric exit in `first-run-exit-status.txt`. Explicit recording mode verifies the candidate source hash before APIs, exclusively creates `first-run-start.json`, streams complete inputs and both unmodified outputs/errors to `first-observations.jsonl`, then exclusively creates `first-summary.json` with gates and before/after source hashes. Preserve every first failure, compile error and partial observation. First evidence cannot be overwritten.

Default integration-test execution is side-effect-free replay, without filesystem writes, first-file existence checks or enforcement of the historical source hash. It asserts the same frozen expectations and cannot rescore or overwrite first observations. Later development replay must remain distinct from the initial evaluation.

No changes to expectations, rescore, runtime edits, branch changes, merge, Oracle or extra agents. Root owns diagnosis/repairs. v1/v2/v3 RED originals and their separate scope notes remain unchanged. This is offline synthetic public-API/source-local relevance only, without provider, human, user, device, production or external-truth acceptance.
