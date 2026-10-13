# Independent v9 bounded launch acceptance protocol

Author: `/root/launch_072_independent`, the same single authorized independent evaluator. Fresh 50 inputs, evenly split 25 positive/25 negative. Authoring follows fixed identity without implementation/root regression/prior reproduction reads. Earlier observations and public contract clarifications are known, but no earlier input is reused, relabeled or rescored. The generic v2 recorder/replay structure is reused; inputs, expected outcomes and artifact identity are new.

Candidate: `f70d1ada8bd56767af4256c82122b01eb60a9d23`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
SHA-256: `6b32d628cf731977b0f637b99646485d7f57d8fc3d9f9fa62b9a5432d09a34a6`.

Before any v9 compilation/API call, commit the corpus, expectations, manifest, this protocol and formatted runner; announce freeze commit/hash first. Store pre-execution provenance at `evaluation/launch-520-521-independent-v9/freeze.json`. `corpus.json` contains full cases/expected v17/v30 outcomes, `manifest.json` contains IDs/counts/hashes/gates, and `author_corpus.py` records fresh authorship.

Every positive has a same-statement canonical name or policy-declared alias directly owning the predicate as subject/object. No coreference creates ownership. Supported predicates only: launched/was-launched/has-been-launched; introduced/was-introduced; went-live; rolled-out/was-rolled-out; is-(now)-generally-available/became-generally-available/has-made-target-generally-available; entered-general-availability/reached-GA. Known on/as-of day-May-year and bounded “according to the release record” adjuncts are used. No new predicate or positive coreference capability is expected.

Original negation, fiction and retraction govern their actual scope. Clearly different targets and explicitly unrelated property claims/questions remain local. Plural release-statement/retraction references and whole report/passage disclaimers must not shrink to only the last different target. Corpus controls cover date-first direct-object launch, plural multi-product retraction, all-following false statements, local foreign supported-subject headlines, property locality, raw marks and ownership/authority floors.

Only excerpt/fact/structured_metadata provide factual proof. Heading/title may govern/deny but cannot prove; URL/navigation have no factual/governing authority. Directly invoke `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30` with each exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positive requires RequestedRelation AND Relevant; negative requires neither. API errors fail cases. This is source-local relevance only, not external hard truth.

Safety gate: zero negative RequestedRelation, zero false Relevant, zero API errors. Positive-recall gate: zero positive misses, zero API errors. Both gates plus unchanged candidate source hash are needed for initial PASS. Save all complete observations before gate assertions.

First command:

```sh
LAUNCH_INDEPENDENT_V9_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v9 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v9 --jobs 2 -- --nocapture
```

Save stdout/stderr as `evaluation/launch-520-521-independent-v9/first-run.log`, numeric exit as `first-run-exit-status.txt`. Explicit first mode verifies fixed source hash before APIs, exclusively creates `first-run-start.json`, streams complete unmodified inputs/outputs/errors to `first-observations.jsonl`, then exclusively creates `first-summary.json` with gates and source hashes before/after. Preserve compile errors, partial observations and all initial results. First files cannot be overwritten.

Default execution without `LAUNCH_INDEPENDENT_V9_RECORD_FIRST=1` replays frozen expectations side-effect-free: no filesystem writes, no dependence on first-file existence and no historical source hash enforcement. Later replay cannot rescore/replace the initial observation or be called fresh.

No retries, rescoring, expectation edits, runtime changes, branch changes, merge, Oracle or further agents. Root owns implementation/integration. Earlier first evidence/scope notes remain immutable. Offline synthetic source-local public-API relevance only; no provider, human, user, device, production or external factual-truth acceptance.
