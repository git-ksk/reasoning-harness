# Independent v8 local/global authority protocol

Author: `/root/launch_072_independent`, the single authorized independent evaluator. Fresh 50-case corpus with 25 positives/25 negatives, authored after fixed identity without implementation, root regression or prior reproduction reads. The evaluator knows earlier observations and public contract clarifications, but no known earlier input is reused, relabeled or rescored. The v2 recorder/replay structure is reused; the authored inputs, expectations and artifact identities are new.

Candidate: `bcf65d9b2f0edb88c05780e4322a633c720d8a8a`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256: `69f3e3116ae0a887cf5b83401d2355485edc5dd877a983c94587c3f1cbb00322`.

Before any v8 compilation/API execution, commit the corpus/expectations, manifest, this protocol and formatted runner. Announce freeze commit and corpus SHA-256 first; store pre-execution provenance in `evaluation/launch-520-521-independent-v8/freeze.json`. `corpus.json` records every input and expected v17/v30 outcome; `manifest.json` contains IDs/counts/hashes/gates; `author_corpus.py` records the fresh authorship specification.

Positive proof requires the policy canonical name or declared alias directly owning the predicate as subject/object in the same statement. Coreference never adds ownership. Declared predicates only: launched/was-launched/has-been-launched; introduced/was-introduced; went-live; rolled-out/was-rolled-out; is-(now)-generally-available/became-generally-available/has-made-target-generally-available; entered-general-availability/reached-GA. On/as-of day-May-year and bounded “according to the release record” adjuncts are used. No new launch predicates or positive coreference expectations are introduced.

A negative/fictional heading with a supported direct-subject launch frame clearly about another product is local to that product. Whole report/passage or “all following” negation remains governing even if a different named launch also appears. The corpus freezes both sides: local foreign passive/active headline statements as positives, and all-following/entire-report disclaimers with a foreign name as negatives. Non-launch property statements/questions, adjacent singular references, original markers/strike spans, target ownership and authority floors remain part of the same bounded contract.

Only excerpt/fact/structured_metadata supply factual proof. Heading/title govern/deny only; URL/navigation have no factual/governing authority. Directly invoke `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30` for each frozen exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positive requires RequestedRelation AND Relevant; negative requires neither. Every API error fails. This qualification is source-local only and does not establish external hard truth.

Safety gate: zero negative RequestedRelation, zero false Relevant, zero API errors. Positive-recall gate: zero positive misses, zero API errors. Both gates plus unchanged source identity are required for initial PASS. Save all complete case observations before asserting gates.

First-observation command:

```sh
LAUNCH_INDEPENDENT_V8_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v8 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v8 --jobs 2 -- --nocapture
```

Store stdout/stderr as `evaluation/launch-520-521-independent-v8/first-run.log`, numeric exit as `first-run-exit-status.txt`. Explicit first mode verifies source hash before APIs, exclusively creates `first-run-start.json`, streams complete unmodified input/output/error into `first-observations.jsonl`, then exclusively creates `first-summary.json` with gates and source hashes before/after. Preserve compile errors, partial observations and full first outcomes. First evidence cannot be overwritten.

Normal execution without `LAUNCH_INDEPENDENT_V8_RECORD_FIRST=1` is side-effect-free frozen-corpus replay: no filesystem writes, no dependence on first-file existence, no historical source hash enforcement. Later development replay cannot rescore or replace the initial observation or be called fresh.

Authoring was interrupted by an automatic approval-review usage limit before the runner/protocol/freeze operation executed. On resumption the candidate hash was rechecked, the saved corpus hash/expectations were unchanged, and no v8 first-observation files or runner existed. Resumption completes the same unexecuted evaluation, not a retry or replacement corpus.

No retries, rescoring, expectation edits, runtime changes, branch changes, merge, Oracle or additional agents. Root owns implementation/integration. Existing first records and scope notes remain immutable. Offline synthetic source-local public-API relevance only; no provider, human, user, device, production or external factual-truth acceptance.
