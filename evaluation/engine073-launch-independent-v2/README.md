# Independent synthetic launch-context check v2

**First semantic result: FAIL, 50/60 expected outcomes.** All 60 rows were observed; the probe exited 0 with empty stderr and no operational errors. Positives passed 21/30 and negatives passed 29/30. One negative retained both v17 `requested_relation` and v30 `relevant`; nine positives became v17 `unresolved` / v30 `ambiguous`.

This is the explicitly authorized operational repair of v1, whose all-zero assessment budget produced `InvalidAssessmentBudget` before any semantic row. V1 remains entirely unchanged, including its operational FAIL and original inputs/labels/results. V2 retains the same 60 semantic cases, expectations and byte-identical `score.py`; only assessment budget `max_model_attempts=2, max_tokens=192, max_elapsed_ms=90000`, the new identity and repair provenance changed. It is not a newly authored holdout. `static-preflight.json` records the required-field/count/nonzero-budget checks and exact semantic/label comparison to v1. One distinct synthetic API-contract sentinel passed before the 60-case first invocation; it is excluded from these counts.

Candidate source remains `b6c1598655a826b3eb5d3527d613dd50ed8b9bbc`, with source-file hash `8acd799ae90a1407320b8899a437e0d0db3c2104a60a90db138fd76dd601a6f8`. Package version `0.7.2` is unchanged before release and is not the candidate identity. The manifest, all inputs/scorer/probe files, source SHA/hash, linked core rlib, serde_json rlib and probe executable were verified immediately before both the separate sentinel and the 60-case invocation. Core rlib hash is `16b7b8e5c33b73eed5670c2eea5cf7f8725a165fc84cf71f03ec7270ce17874c`. The build was parent-supplied; the evaluator did not rebuild or inspect production implementation.

The precommitted gate requires every positive to retain requested launch and Relevant, and every negative to lose requested launch and remain non-Relevant. The single safety miss is `i073-024`: `Larchwharf Capsule was launched in August 2024. No launch of Larchwharf Capsule took place.` The explicit same-target non-event did not suppress the optimistic launch qualification. The nine positive misses concern clear distinct-target or distinct-property negation/retraction/withdrawal, as itemized below. These are observed disagreements against the evaluator's frozen controlled labels; no causal explanation from implementation inspection is claimed.

| Case | Polarity | Context axis | v17 | v30 |
| --- | --- | --- | --- | --- |
| i073-001 | positive | trailing_explicit_retraction | unresolved | ambiguous |
| i073-021 | positive | leading_never_happened | unresolved | ambiguous |
| i073-024 | negative | explicit_no_event | requested_relation | relevant |
| i073-027 | positive | leading_present_denial | unresolved | ambiguous |
| i073-031 | positive | heading_claim_withdrawn | unresolved | ambiguous |
| i073-045 | positive | possessive_reference_withdrawn | unresolved | ambiguous |
| i073-047 | positive | report_scope_launch_vs_price | unresolved | ambiguous |
| i073-049 | positive | heading_unscoped_reference | unresolved | ambiguous |
| i073-051 | positive | title_unscoped_reference | unresolved | ambiguous |
| i073-053 | positive | publisher_present_denial | unresolved | ambiguous |

Failed-case input contexts, preserved from the frozen corpus:

- `i073-001`: [excerpt] Asterloom Lantern was launched in August 2024. We retract our claim that Moonvine Spindle was launched.
- `i073-021`: [excerpt] The launch of Moonvine Spindle never happened. Kestrelmere Sundial was launched in August 2024.
- `i073-024`: [excerpt] Larchwharf Capsule was launched in August 2024. No launch of Larchwharf Capsule took place.
- `i073-027`: [excerpt] We deny that Nectarford Relay is made from titanium. Nectarford Relay was launched in August 2024.
- `i073-031`: [heading] Pinehollow Tablet packaging claim withdrawn / [excerpt] Pinehollow Tablet was launched in August 2024.
- `i073-045`: [excerpt] Willowroost Arbor was launched in August 2024. Its packaging claim is withdrawn.
- `i073-047`: [excerpt] Xylemshore Map was launched in August 2024. The Xylemshore Map pricing report is retracted.
- `i073-049`: [heading] We retract the packaging statement / [excerpt] Yarrowport Cradle was launched in August 2024.
- `i073-051`: [source_title] Packaging claim withdrawn / [excerpt] Zinniaford Wheel was launched in August 2024.
- `i073-053`: [excerpt] Amberthicket Globe was launched in August 2024. The publisher denies that Moonvine Spindle launched.

The same case labels and scoring rules were fixed before any valid semantic observation and were never changed after results. V2 used one separate sentinel, one 60-case invocation, and one application of the precommitted scorer. No retry, semantic prompt/model invocation, relabeling or rescoring occurred. First stdout/stderr, started/finished invocation identity guards, results and summaries are preserved. After this first semantic observation the set is consumed; future use is regression/development evidence, not a fresh independent set.

This is a narrow synthetic public-core-API source-fix check for #520. It is not #531's real primary-source 24-question evaluation, general release acceptance, CLI adoption, provider performance, document acquisition or final-answer E2E. Only the candidate was run, so no measured baseline improvement/regression size is claimed. The 60 scenarios form 30 matched pairs using 30 fictional target names; they are not 60 independent real documents. Optimistic binding/raw qualification deliberately tests safety correction, not model extraction/binding accuracy. No preparation/future-release or unrelated feature claim is generalized. Labels are evaluator-curated controlled expectations, not human-approved or universal truth. Reasons named `model_relevant`/`model_ambiguous` are serialized core reason labels here; no model was called.

Only authorized generic probe and necessary public type definitions were inspected. Engine implementation, existing tests, old cases/fixtures/corpora and previous result outputs were not read for authoring. No Oracle, additional agent, provider API, cargo/shared-target change, Git mutation or commit occurred. Writes were confined to the two newly authorized evaluation directories and dedicated temporary probe paths. `v1-artifact-hashes.json` and the final both-version artifact manifest verify preservation; each pre-run manifest records candidate/source/corpus/probe/scoring/author hashes.
