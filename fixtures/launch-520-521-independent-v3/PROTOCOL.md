# Independent v3 protocol — before first execution

Author: `/root/launch_072_independent`, the same authorized independent evaluator. The corpus contains 56 fresh cases, evenly split into 28 positives and 28 negatives. It was authored after receiving the new fixed candidate identity without reading root implementation, repair tests or prior reproduction material. The evaluator knows its own prior v1/v2 results, but did not rename, resample, relabel or rescore their cases. The recorder/replay harness structure is reused from v2; the inputs and authored expectations are new.

Candidate commit: `fdd630e1085ab3994e3397de0646741313cfd15c`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
SHA-256: `63493029537f8e66762332e118090abbbce5fdf1481e22b7df57124ab12b2c32`.

`corpus.json` freezes each ID, input, positive/negative label and expected v17/v30 outcome. `manifest.json` records the candidate identity, corpus SHA-256, IDs, counts and safety/recall gates. `author_corpus.py` records the exact fresh author specification. Commit the corpus, manifest, formatted runner and this protocol before compiling or invoking any v3 API. Announce the freeze commit and hash before execution; store that provenance in `evaluation/launch-520-521-independent-v3/freeze.json`.

The bounded positive vocabulary is limited to target-owned was/has-been launched, introduced, went live, rolled out and generally-available frames (including actor has-made, entered general availability and reached GA). Dates include ISO, month/day/year and day-May-year. The bounded citation adjunct “according to the release record” is included. Actual factual proof must appear in `excerpt`, `fact` or `structured_metadata`. `heading`/`source_title` may govern or deny but never supply positive factual proof; URL/navigation neither prove nor govern factual claims.

Negation, questions, conditions, fiction, retraction and original markers apply to the actual target and statement. Explicitly different targets or unrelated non-launch statements about the same target must not invalidate a separately factual launch. Actor/object ownership and explicit pronoun ownership are evaluated as source-local binding: a product covered by the source's record may be referred to as “It”; an introducing actor or an explicitly different antecedent is not the launched target. Qualification does not claim external hard truth.

All cases pass exact/exact binding proposal and optimistic exact-target/requested-relation/no-risk raw qualification to both public APIs: `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30`. Positive expected: RequestedRelation AND Relevant. Negative expected: neither RequestedRelation nor Relevant. Any API error fails the case.

Safety gate: zero negative RequestedRelation and zero false Relevant, with zero API errors. Positive-recall gate: zero positive misses, with zero API errors. Both gates and unchanged candidate source SHA-256 are required for first-observation PASS. Complete outputs must be recorded before asserting failures.

Explicit first-observation command:

```sh
LAUNCH_INDEPENDENT_V3_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v3 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v3 --jobs 2 -- --nocapture
```

Capture complete stdout/stderr into `evaluation/launch-520-521-independent-v3/first-run.log`, and numeric exit status into `first-run-exit-status.txt`. Explicit first mode checks the fixed source hash before API calls, exclusively creates `first-run-start.json`, streams every complete input and both unmodified outputs/errors into `first-observations.jsonl`, then exclusively creates `first-summary.json` with aggregate gates and source identity before/after. Preserve compile/runtime failures, partial observations and all initial results. First files cannot be overwritten.

Normal test execution without `LAUNCH_INDEPENDENT_V3_RECORD_FIRST=1` is a side-effect-free CI replay: no filesystem writes, no dependence on first-observation file existence and no enforcement of the historical candidate source hash. It asserts the same frozen expectations. Initial results remain immutable even if later development replay passes.

No retry may be called fresh; no expected outcomes may change or be rescored. No runtime changes, branch changes, merge, Oracle or extra agents. Root owns implementation diagnosis and repairs. Preserve v1/v2 RED records unchanged. This establishes offline synthetic public-API/source-local relevance only, not provider, human, user, device, production or external truth acceptance.
