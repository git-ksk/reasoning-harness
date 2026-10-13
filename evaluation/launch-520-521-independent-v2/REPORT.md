# Engine #520/#521 independent v2 — first observation

Author: `/root/launch_072_independent`. Fresh offline synthetic corpus; authored without implementation bodies, root repairs/tests, or other corpus contents. No retries, expectation changes, runtime edits, or rescoring were performed.

**RED: 54/67 passed.** Positive recall: 23/31 passed, 8 misses. Safety: 31/36 passed, 5 false Relevant and 5 negative RequestedRelation (the same 5 cases). API errors: 0. Both gates failed. Cargo exited 101 only after all observations and aggregate gates were recorded.

Candidate: `de47d3ae4573bdfeaebae45be56c3595c06f683c`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256 before/after: `c2f119030c718af66f872f913bf697951c33eace9067cec9bbcdbd5bca0d4f17` (unchanged).
Freeze commit: `f79b400e70d8e914dd6ec7f4f65c46e4b5469d1e`, before any v2 compilation/API execution.
Corpus SHA-256: `2fd56eb40be3e4393c54e0d773502bf5a14927cbd5dace7e8456202de02f7afd` (verified again after the first run).

All inputs use exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positives require effective RequestedRelation and Relevant; negatives require neither. Authority is limited to excerpt/fact/structured_metadata factual proof. Heading/source_title can govern or deny but cannot supply factual proof. URL/navigation cannot prove or govern factual claims. Markers scoped only to another target or an unrelated statement about the same target must not negate a separately factual launch. Source-local relevance does not imply externally verified hard truth.

## Failures

Every failed output has identity_scope=exact_target and scope_risk=none. All eight positive misses have relation_scope=unresolved, disposition=ambiguous, path=conservative_fallback, and reasons=[harness_canonical_name_anchor, local_qualification_disagreement, model_ambiguous]. All five negative failures have relation_scope=requested_relation, disposition=relevant, path=model_assisted, and reasons=[harness_canonical_name_anchor, positive_target_local_binding_confirmed, model_relevant].

| ID | Label | Category | Complete signals | v17 / v30 |
| --- | --- | --- | --- | --- |
| P201 | positive | launch_day_before_may | excerpt: Brisano Vault was launched on 7 May 2081, according to the release record. | unresolved / ambiguous |
| P211 | positive | neutral_question_in_preceding_sentence | excerpt: Where is the support guide? Falvessa Bay was launched on 16 May 2091. | unresolved / ambiguous |
| P213 | positive | other_subject_withdrawn_title | source_title: Withdrawn claim about Ruvani Cove<br>fact: Volmessa Rail was launched on 14 May 2093. | unresolved / ambiguous |
| P214 | positive | other_subject_false_heading | heading: False claim about Doravi Deck<br>excerpt: Penvaro Lane entered general availability on 13 May 2094. | unresolved / ambiguous |
| P222 | positive | combining_strike_other_subject | excerpt: P̶a̶v̶n̶o̶r̶i̶ C̶h̶a̶r̶t̶ w̶a̶s̶ l̶a̶u̶n̶c̶h̶e̶d̶ o̶n̶ 3̶ M̶a̶y̶ 2̶1̶0̶2̶.̶ Desvoro Byte reached GA on 5 May 2102. | unresolved / ambiguous |
| P229 | positive | same_target_strike_on_nonlaunch_statement | excerpt: ~~Kelmora Tap includes an offline editor.~~ Kelmora Tap was launched on 22 May 2109. | unresolved / ambiguous |
| P230 | positive | same_target_cross_on_nonlaunch_statement | excerpt: ❌ Almora Plane supports XML export. Almora Plane entered general availability on 23 May 2109. | unresolved / ambiguous |
| P231 | positive | same_target_false_heading_nonlaunch_statement | heading: False claim: Jermoro Sink has an offline mode<br>fact: Jermoro Sink reached GA on 24 May 2109. | unresolved / ambiguous |
| N204 | negative | retraction_notice_governs_metadata | heading: Retraction notice<br>structured_metadata: Zelmero Jet reached GA on 16 May 2113. | requested_relation / relevant |
| N207 | negative | fictional_leading_without_release_noun | excerpt: What follows is fictional. Jevessa Rack was launched on 13 May 2116. | requested_relation / relevant |
| N209 | negative | false_correction_trailing | excerpt: Kelmessa Coin was launched on 11 May 2118. Correction: this launch report is false. | requested_relation / relevant |
| N210 | negative | denied_above_ga_reference | excerpt: Tesrano Pool reached GA on 10 May 2119. The above GA claim was denied. | requested_relation / relevant |
| N211 | negative | fictional_account_trailing_ga | excerpt: Duvessa Pipe entered general availability on 9 May 2120. This account is fictional. | requested_relation / relevant |

P222 contains original U+0336 combining strike on every non-space character of the entire other-target sentence (including its period); the following target GA sentence is unstruck. P229–P231 concern an unrelated feature assertion about the same product followed by separate factual launch/GA proof. These distinctions were frozen before execution.

## Passing controls

Positive IDs: P202, P203, P204, P205, P206, P207, P208, P209, P210, P212, P215, P216, P217, P218, P219, P220, P221, P223, P224, P225, P226, P227, P228.

Negative IDs: N201, N202, N203, N205, N206, N208, N212, N213, N214, N215, N216, N217, N218, N219, N220, N221, N222, N223, N224, N225, N226, N227, N228, N229, N230, N231, N232, N233, N234, N235, N236.

Passing positive controls include entered general availability, reached GA, day-before-May dates, alias, dotted/decimal names, neutral sentences, several explicit other-target denial/marker scopes, and URL/footer context not governing factual claims. Every original-symbol negative N213–N220 passed, including multi-sentence/newline markdown strike and combining strike. Bounded GA question/conditional/denied/misattributed/rumor counterparts N221–N228 passed. Heading/title-only, URL/footer-only, planned-launch and mere-topic controls N229–N236 also passed.

## Evidence and replay behavior

- `freeze.json`: freeze/source/corpus identities and the exact explicit recording command.
- `first-run-start.json`: first API execution identity and checkout head.
- `first-observations.jsonl`: complete unmodified input and both API outputs/errors for all 67 cases.
- `first-failures.json`: exact subset of the 13 failed observations.
- `first-summary.json`: initial aggregate gates and source identity before/after.
- `first-run.log` / `first-run-exit-status.txt`: full compiler/test output and exit 101.
- `artifact-sha256.json`: hashes of the frozen corpus/protocol/runner and saved first evidence.

Explicit first-observation mode uses exclusive creation and cannot overwrite existing evidence. Default test execution is a side-effect-free corpus replay, without source identity enforcement or filesystem writes. Existing first-result files do not prevent default replay; the current frozen RED gates will still fail against this unchanged candidate. No default replay was performed by the evaluator after the first execution.

v1 RED evidence remains separate and unchanged. Root owns implementation diagnosis and repairs. This evaluation is offline synthetic public-API relevance only; it establishes no live-provider, user, device, production, or externally verified factual-truth acceptance.
