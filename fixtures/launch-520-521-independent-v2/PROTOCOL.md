# Independent v2 protocol — frozen before execution

Author: `/root/launch_072_independent`, the same independently authorized evaluator as v1. This corpus is newly authored: no v1 cases were renamed, reused, relabeled or rescored. Root implementation bodies, root repairs/tests, prior reproduction corpora, and other independent corpora were not consulted. The evaluator knows the previous v1 results from its own first run, and received the public contract clarification/reminder described below.

Candidate commit: `de47d3ae4573bdfeaebae45be56c3595c06f683c`.
Candidate source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256: `c2f119030c718af66f872f913bf697951c33eace9067cec9bbcdbd5bca0d4f17`.

67 fresh cases: 31 positives, 36 negatives. The case IDs, inputs and per-case expected v17/v30 outcomes are frozen in `corpus.json`. `manifest.json` records counts, IDs, corpus SHA-256 and both gates. `author_corpus.py` records the exact new authorship specification.

Authority contract: only `excerpt`, `fact` and `structured_metadata` can furnish affirmative factual proof. `heading` and `source_title` may govern/deny factual proof but cannot provide it themselves. `canonical_url` and `navigation_or_footer` cannot furnish factual proof or govern a factual assertion. Affirmative source-local recognition covers launch, entered general availability, reached GA, aliases, dotted/decimal names and day-before-May dates. Governing fictional/hypothetical/false/denied/withdrawn/retracted context and original negation/strikethrough markers must be preserved. Conditional, question, denied, rumor and wrong-target counterparts withhold. Scope applies to both target and statement: markers explicitly belonging to another target, or to an unrelated non-launch statement about the same target, must not invalidate a separately affirmative launch claim.

All cases use exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. The test calls the public `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30` APIs directly. Positives require effective RequestedRelation AND Relevant. Negatives require neither RequestedRelation nor Relevant. Any API error fails the case.

Safety gate: zero negative RequestedRelation, zero false Relevant, zero API errors. Positive-recall gate: zero positive misses, zero API errors. Both gates and an unchanged candidate source hash are required for first-observation PASS. No source-local output is interpreted as externally verified hard truth.

The corpus, manifest, this protocol and formatted API runner must be committed before any v2 compilation/API execution. Record and announce that freeze commit before starting. The explicit first-observation command is:

```sh
LAUNCH_INDEPENDENT_V2_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v2 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v2 --jobs 2 -- --nocapture
```

Capture stdout/stderr and exit status in `evaluation/launch-520-521-independent-v2/first-run.log` and `first-run-exit-status.txt`. `freeze.json` records the announced freeze/source/corpus identity before execution. Explicit first-observation mode checks candidate source SHA-256 before API calls, exclusively creates `first-run-start.json`, streams all complete inputs and unmodified outputs/errors to `first-observations.jsonl`, then exclusively creates `first-summary.json` with aggregate gates and source hashes before/after. It records all cases before asserting failures. Preserve compiler failures, partial outputs and all first failures.

Normal integration-test execution WITHOUT `LAUNCH_INDEPENDENT_V2_RECORD_FIRST=1` replays the frozen corpus, asserts its gates, and prints results without any filesystem writes or dependence on existing observation files. Normal replay does not enforce the historical candidate hash, allowing future candidates to be tested. It never changes first evidence. The initial execution is the explicit first-observation command; later diagnostics/replays are distinct runs and cannot be labeled fresh first observations.

Do not edit expected outcomes, rescore the initial result, modify runtime, change branches, merge, use Oracle, or spawn additional agents. Root owns repairs/integration. v1 RED evidence and its separate post-run P04 clarification remain preserved. This protocol establishes offline synthetic API relevance only; no live-provider, user, device, production or external factual-truth acceptance is claimed.
