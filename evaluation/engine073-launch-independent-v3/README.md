# Independent synthetic launch-context check v3

**First semantic result: FAIL, 39/60 precommitted outcomes matched.** All 60 outputs were observed, exit status was 0, stderr was empty, and there were no operational errors. Positives passed 11/30 and negatives passed 28/30. Two negatives retained both v17 `requested_relation` and v30 `relevant`; 19 positives became v17 `unresolved` and v30 `ambiguous`.

Candidate: `d8c777656688339f915a0f80a0b8af422c50ee7d`. Source-file SHA256: `ef8a6539d79f73cfb6b3b5b9ff89b8b5d75cb0e43f5bf0f692f8f0de21830c22`. Actual linked core rlib SHA256: `950248fe0e0514e10dfd432404c1a2755f4e0b68eaf367e5f5dae7552d301a39`. Package version is still `0.7.2`; source SHA identifies the unreleased candidate. Immediately before invocation the recorder verified actual HEAD, hash-only source bytes, rlib, serde_json, dedicated probe executable, complete inputs, corpus/labels, scorer, protocol and every frozen authored file. The parent supplied the completed build; no independent rebuild or implementation inspection was performed.

V3 is a newly authored 60-scenario synthetic set with 30 matched pairs, 30 fictional targets and new discourse/control constructions. It is not a surface name replacement of v2. The evaluator knew v2 and records that exposure: v2 is seen development data, not a fresh holdout. V1's operational FAIL and v2's semantic FAIL are unchanged. Their complete pre-v3 artifact hashes are preserved and reverified after this run. No root implementation, existing root tests, old v10 or other corpora/fixtures were inspected.

The scope clarification was incorporated before freeze. Every case supplies a separate unadorned Excerpt with its canonical target as the direct subject of `was launched on 14 March 2023`. Entry/statement numbers occur only in surrounding plain-text context; no new affirmative launch grammar, trusted reference AST or general coreference feature is assumed. Positive controls explicitly concern another product/property or neutral record context, so their expectation does not depend solely on resolving a number. Five explicitly unresolved launch-retraction references were precommitted as negative withhold cases. `pre-freeze-scope-review.json` and `protocol.json` explain these choices. Expectations are offline controlled evaluator labels and do not approve the unrelated property as a product fact.

Stages are separate. `static-preflight.json` is STATIC_ONLY PASS with 0 Engine invocations/semantic observations; it verifies all 60 required-field inputs, budget 2/192/90000, label exclusion, supported direct affirmative signals, unchanged inherited scorer/probe and unchanged predecessors. There was no v3 semantic sentinel. After freeze, one probe process made the first 60-case observations through the deterministic public v17/v29/v30 functions. The byte-identical precommitted scorer was then applied once. No retry, rescoring, post-result relabeling or result overwrite occurred.

Two unsafe negative observations:

- `li3-020` (no_event_log_annotation): v17 `requested_relation`, v30 `relevant`. Complete signals: [excerpt] Jadegrove Speaker was launched on 14 March 2023. / [excerpt] Current record: Launch assertion recorded for Jadegrove Speaker.
Log annotation: No launch for Jadegrove Speaker occurred.
- `li3-038` (three_entries_one_unspecified_withdrawal): v17 `requested_relation`, v30 `relevant`. Complete signals: [excerpt] Sorrelvale Receiver was launched on 14 March 2023. / [excerpt] Three launch entries are listed in this order.
First: Launch assertion recorded for Sorrelvale Receiver.
Second: Pondstone Anchor was launched on 14 March 2023.
Third: Riverglass Node was launched on 14 March 2023.
Author note: I withdraw one launch claim in this list; this notice does not say which one.

The first negative explicitly says that no launch for the same named target occurred. The second explicitly says that one of three launch claims is withdrawn and that the notice does not identify which one. Both violate the frozen withholding expectation. These observations justify the narrow gate FAIL without requiring any implementation-level causal claim.

Positive disagreements are listed below. Every row had its own separate direct target-subject affirmative signal. The unrelated property/owner is explicit in the contextual correction; these labels do not require accepting that property itself or trusting a numeric reference mapping.

| Case | Context axis | v17 | v30 |
| --- | --- | --- | --- |
| li3-001 | lettered_statement_scope | unresolved | ambiguous |
| li3-005 | second_excerpt_amendment | unresolved | ambiguous |
| li3-007 | title_applies_to_object | unresolved | ambiguous |
| li3-009 | heading_statement_object | unresolved | ambiguous |
| li3-011 | direct_assertion_with_back_reference | unresolved | ambiguous |
| li3-015 | did_not_occur_event_scope | unresolved | ambiguous |
| li3-017 | never_occurred_event_scope | unresolved | ambiguous |
| li3-021 | leading_today_author_denial | unresolved | ambiguous |
| li3-023 | two_entries_exclusive_named_withdrawal | unresolved | ambiguous |
| li3-025 | cross_signal_entry_identifier | unresolved | ambiguous |
| li3-031 | publisher_withdraws_claim_type | unresolved | ambiguous |
| li3-033 | heading_current_denial | unresolved | ambiguous |
| li3-037 | three_entries_one_unspecified_withdrawal | unresolved | ambiguous |
| li3-043 | heading_missing_referent | unresolved | ambiguous |
| li3-045 | single_register_claim_reference | unresolved | ambiguous |
| li3-049 | title_heading_joint_scope | unresolved | ambiguous |
| li3-053 | former_latter_with_explicit_claim_type | unresolved | ambiguous |
| li3-055 | reader_question_publisher_response_chain | unresolved | ambiguous |
| li3-057 | passive_hereby_author_withdrawal | unresolved | ambiguous |

Examples include withdrawal of a carrying-strap claim, withdrawal of an oxygen-sensor specification, denial of a photoshoot event, and a named different product's launch withdrawal explicitly limited to that product alone. All 19 disagreement inputs and expected/observed fields remain available in `cases.json` and `first-results.json`; nothing was relabeled after observation. Exact serialized core reasons are saved as raw output. Reason names such as `model_relevant` and `model_ambiguous` do not mean a model ran in this probe.

This is a bounded synthetic #520 source-fix confirmation, separate from #531's new real primary-source 24-question evaluation and from release/CLI adoption. It measures neither acquired documents, final answers, providers, quota/headroom, alias reasoning, preparation/future release, new launch predicates, arbitrary reference ASTs nor approval of unrelated property facts. Only the candidate was run; no baseline delta is measured, and v2/v3 percentages cannot be compared as an improvement/regression measurement because their cases and candidates differ. The 60 scenarios are 30 paired controlled constructions, not 60 independent real documents or an exhaustive language sample. After this first semantic observation v3 is consumed; future runs are regression/development evidence.

Provider API calls, Oracle calls, additional agents, cargo/shared-target changes, Git mutations and commits were all zero. Repository writes were confined to this new v3 directory. Rustc wrote a dedicated probe under `/private/tmp/engine073-launch-independent-v3-probe`. First stdout/stderr and started/finished invocation records remain intact. `manifest.json` records the pre-run candidate/source/rlib/probe/scorer/input hashes; `receipt.json` and `final-artifact-manifest.json` record the post-run artifact hashes and predecessor preservation.
