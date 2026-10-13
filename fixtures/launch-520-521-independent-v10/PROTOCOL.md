# Independent v10 bounded adverse-scope acceptance protocol

Author: `/root/launch_072_independent`, the same single authorized independent evaluator. Fresh 50 inputs: 25 positive/25 negative. Authored after fixed identity without implementation/root regression/prior reproduction reads. Earlier observations and explicit public contract updates are known, but earlier inputs are not reused, relabeled or rescored. The generic v2 recorder/replay structure is reused; the corpus/expectations/artifact identities are new.

Candidate: `2f12d4c61546e60d31026d9b691c9eb369c37957`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256: `5343e722618721e58ad6a24b12c55c1272eb3c581bb3ada26b24d8596dfebbf5`.

Before any v10 compilation/API call, commit corpus/expectations, manifest, this protocol and formatted runner. Announce freeze commit/hash first; record provenance in `evaluation/launch-520-521-independent-v10/freeze.json`. `corpus.json` has complete inputs and expected v17/v30 outcomes; `manifest.json` has IDs/counts/source/corpus hashes and gates; `author_corpus.py` records independent authoring.

Every positive has same-statement canonical name or policy-declared alias as direct subject/object of a supported predicate. Coreference cannot create positive ownership. Supported vocabulary only: launched/was-launched/has-been-launched; introduced/was-introduced; went-live; rolled-out/was-rolled-out; is-(now)-generally-available/became-generally-available/has-made-target-generally-available; entered-general-availability/reached-GA. Known on/as-of day-May-year and bounded “according to the release record” adjuncts are used. No new release predicate or positive coreference is requested.

Known negative/fictional/retraction cues whose scope is unresolved must fail closed regardless of the reference word. The new controls include a false subset of claims, an unresolved retracted antecedent and one fictional member of a collection: none safely confirms the policy target's launch. Broad report/passage and plural-statement disclaimers remain governing. Explicitly separate foreign supported-subject launch or clearly unrelated non-launch property scope stays local. This tests the existing adverse-cue contract without requesting new launch vocabulary.

Only excerpt/fact/structured_metadata supply factual proof. Heading/title may govern/deny but never prove. URL/navigation cannot supply factual/governing authority. Original strike/negation symbols, owner mismatch, questions/conditions/rumor and authority floors remain controls. Directly call `derive_effective_evidence_local_qualification_v17` and `materialize_evidence_relevance_v30` with each frozen exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positive needs RequestedRelation AND Relevant; negative needs neither. API errors fail cases. Source-local qualification does not establish external hard truth.

Safety gate: zero negative RequestedRelation, zero false Relevant, zero API errors. Positive-recall gate: zero positive misses, zero API errors. Both gates plus unchanged source hash are required for first PASS. Save every complete observation before asserting gates.

First command:

```sh
LAUNCH_INDEPENDENT_V10_RECORD_FIRST=1 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=target/engine-072-independent-v10 cargo test -p reasoning-harness-core --test launch_context_markers_independent_v10 --jobs 2 -- --nocapture
```

Save stdout/stderr as `evaluation/launch-520-521-independent-v10/first-run.log`, numeric exit as `first-run-exit-status.txt`. Explicit first mode verifies fixed source hash before APIs, exclusively creates `first-run-start.json`, streams complete unmodified input/output/error into `first-observations.jsonl`, then exclusively creates `first-summary.json` with gates and source hashes before/after. Preserve any compile error, partial observation or full initial result. First files cannot be overwritten.

Default execution without `LAUNCH_INDEPENDENT_V10_RECORD_FIRST=1` replays frozen expectations side-effect-free: no filesystem writes, no dependence on first-file existence, no historical source hash enforcement. Later replay cannot rescore/replace the initial observation or be called fresh.

No retries, rescoring, expectation edits, runtime changes, branch changes, merge, Oracle or extra agents. Root owns implementation/integration. All earlier first records, including v9 GREEN and earlier RED/scope notes, remain immutable. Offline synthetic source-local public-API relevance only; no provider, human, user, device, production or external factual-truth acceptance.
