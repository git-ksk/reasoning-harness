# Independent v6 bounded statement-scope protocol

Author: `/root/launch_072_independent`, the same single authorized independent evaluator. 50 freshly authored inputs, 25 positive and 25 negative. Authoring began after the fixed identity was supplied without implementation, root regression or prior reproduction reads. Earlier observations and explicit contract clarifications are known to this evaluator, but cases were not renamed, relabeled, reused or rescored. The v2 recorder/replay structure is reused; the input corpus, expectations and artifact identities are new.

Candidate commit: `0cf240469f5b81ccc93ecb9d8dec53e85f3cff8b`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
SHA-256: `e0b4f91248ee6d4746835ecc19a9df8549b3de95733db754978b3de6f9826434`.

Freeze/commit the corpus, expected outcomes, manifest, this protocol and formatted runner before v6 compilation or API execution. Announce freeze commit and corpus SHA-256 first; save pre-execution provenance as `evaluation/launch-520-521-independent-v6/freeze.json`. `corpus.json` has every ID/input/expected v17/v30 outcome. `manifest.json` records source/corpus identity, counts, IDs and safety/recall gates. `author_corpus.py` records the fresh author specification.

Every affirmative case must have canonical name or policy-declared alias as direct subject/object in the same actual statement. No cross-sentence pronoun/coreference ownership creation. Positive predicates stay within launched/was-launched/has-been-launched, introduced/was-introduced, went-live, rolled-out/was-rolled-out, is-(now)-generally-available/became-generally-available/has-made-target-generally-available, entered-general-availability/reached-GA. Known on/as-of day-May-year dates and the bounded “according to the release record” adjunct are used.

The v5 contract also states that unrelated non-launch attributes and their adjacent singular references stay local: e.g. “That claim was denied” after a feature assertion must not deny a separate named launch. Broad report/passage negation remains governing even if it is next to a feature statement. The new corpus independently checks both sides and the differing adjacency of launch versus feature statements. These context references never create launch ownership. Raw original symbols and strike scope, exact different-target ownership, title/heading authority and URL/navigation authority are included.

Only excerpt/fact/structured_metadata furnish positive factual proof. Heading/title can govern or deny but cannot prove; URL/navigation cannot prove or govern. Source-local relevance never claims external hard truth. Call both `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30` directly with each frozen exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification.

Positive expected: RequestedRelation AND Relevant. Negative expected: neither RequestedRelation nor Relevant. Any API error fails its case. Safety gate: zero negative RequestedRelation, zero false Relevant, zero API errors. Positive-recall gate: zero positive misses, zero API errors. Both gates plus unchanged source identity are required for initial PASS. Save all outcomes before asserting gate failures.

First-observation command:

```sh
LAUNCH_INDEPENDENT_V6_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v6 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v6 --jobs 2 -- --nocapture
```

Save complete stdout/stderr as `evaluation/launch-520-521-independent-v6/first-run.log` and numeric exit as `first-run-exit-status.txt`. Explicit first mode checks fixed source hash before APIs, exclusively creates `first-run-start.json`, streams complete unmodified input/output/error per case into `first-observations.jsonl`, then exclusively creates `first-summary.json` with aggregate gates and source hashes before/after. Preserve any compile failure, partial observation or complete failure. First records cannot be overwritten.

Default integration execution without `LAUNCH_INDEPENDENT_V6_RECORD_FIRST=1` replays frozen expectations side-effect-free: no filesystem writes, no dependence on first-file existence and no enforcement of historical candidate source hash. Later development replays cannot rescore or replace the initial result or be described as fresh evaluations.

No retries, rescoring, expectation edits, runtime changes, branch changes, merge, Oracle or extra agents. Root owns implementation work. Earlier first evidence and separate scope notes remain immutable. This evaluation is offline synthetic source-local public-API relevance only, with no provider, human, user, device, production or external factual-truth acceptance.
