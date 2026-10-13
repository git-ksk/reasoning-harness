# Engine #520/#521 independent v1 — initial observation

Author: `/root/launch_072_independent`. Fresh offline synthetic corpus; no implementation bodies or prior corpus contents were read during authoring. No retries, runtime changes, expectation changes, or rescoring were performed.

**RED: 56/66 cases passed.** Safety: 36/42 passed, 6 false Relevant and 6 negative RequestedRelation (the same 6 cases). Positive recall: 20/24 passed, 4 misses. API errors: 0. Both gates failed. Cargo exited 101 after recording all observations and asserting the gates.

Candidate commit: `049be6542d95539e275d9a44a5bac89835b43834`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256 before/after: `c9d18f43633665191b18af5c5d79cc535b7914be66e91651ea4788bd18a41736` (unchanged).
Freeze commit: `1feccdf4f500d5585219d9f1b0e1187c407de536`, before the first compile/API execution.
Corpus SHA-256: `7303baa54b2a64176bbd7192d640d0dc3f263c146c886f80fabb36dcf9d9c833`.

Frozen expectations: a positive needs effective RequestedRelation and Relevant; a negative needs neither. All inputs use exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. These are source-local relevance outcomes, not externally verified truth claims.

## Failures

All failures returned identity_scope=exact_target and scope_risk=none. All six negative failures returned relation_scope=requested_relation, disposition=relevant, path=model_assisted and reasons=[harness_canonical_name_anchor, positive_target_local_binding_confirmed, model_relevant]. All four positive misses returned relation_scope=unresolved, disposition=ambiguous, path=conservative_fallback and reasons=[harness_canonical_name_anchor, local_qualification_disagreement, model_ambiguous].

| ID | Frozen label | Category | Complete signal text | v17 / v30 |
| --- | --- | --- | --- | --- |
| P04 | positive | affirmative_heading | heading: Tavren Cloud was launched on 11 July 2033.<br>excerpt: The release includes a collaboration workspace. | unresolved / ambiguous |
| P14 | positive | other_target_denial_before | excerpt: Osven Pilot was not launched in May. Leriva Forge was launched on 21 May 2043. | unresolved / ambiguous |
| P21 | positive | ga_full_phrase | excerpt: Talvori Kit entered general availability on 28 December 2050. | unresolved / ambiguous |
| P22 | positive | ga_short_form | excerpt: Cenora Grid reached GA on 29 January 2051. | unresolved / ambiguous |
| N03 | negative | false_heading | heading: False claim<br>excerpt: Tovari Desk was launched on 5 June 2033. | requested_relation / relevant |
| N04 | negative | denied_heading | heading: Denied launch claim<br>excerpt: Ralven Port was launched on 6 July 2034. | requested_relation / relevant |
| N06 | negative | withdrawn_heading | heading: Withdrawn release statement<br>fact: Nelvori Edge was launched on 8 September 2036. | requested_relation / relevant |
| N08 | negative | fictional_trailing_sentence | excerpt: Koravi Tide was launched on 10 November 2038. This release story is fictional. | requested_relation / relevant |
| N16 | negative | retracted_trailing_sentence | excerpt: Tervani Wire was launched on 18 July 2046. The publisher retracted this release statement. | requested_relation / relevant |
| N18 | negative | false_title_metadata | source_title: False launch report<br>structured_metadata: Cavrani Leaf was launched on 20 September 2048. | requested_relation / relevant |

P04 deliberately treats an explicit affirmative launch/date assertion in a source heading as a legitimate positive; this expectation was fixed before execution and is retained. P21 and P22 deliberately cover ordinary GA predicates “entered general availability” and “reached GA”. P14 tests a denial whose explicit subject is another product in the preceding sentence.

## Successes

Positive IDs passing: P01, P02, P03, P05, P06, P07, P08, P09, P10, P11, P12, P13, P15, P16, P17, P18, P19, P20, P23, P24.

Negative IDs passing: N01, N02, N05, N07, N09, N10, N11, N12, N13, N14, N15, N17, N19, N20, N21, N22, N23, N24, N25, N26, N27, N28, N29, N30, N31, N32, N33, N34, N35, N36, N37, N38, N39, N40, N41, N42.

The successful controls include ordinary launch, “became generally available”, aliases, dotted/decimal names, neutral leading/trailing prose, explicit unrelated-target marks/strikes/hypotheticals/retractions, and URL/navigation not vetoing a factual assertion. All original-symbol controls N27–N38 passed, including ~~ across multiple sentences, ❌, ≠, ¬, ✗, ✘, and combining stroke forms. URL/navigation-only, wrong-target, conditional, rumor, and question negatives passed. Context handling is incomplete in the specific forms listed above.

## Evidence and execution

- `freeze.json`: pre-execution provenance and exact low-disk command.
- `first-run-start.json`: first API execution identity and checkout head.
- `first-observations.jsonl`: complete input, v17 output/error, v30 output/error, and frozen verdict for every case.
- `first-failures.json`: exact unmodified subset of the 10 failed observations.
- `first-summary.json`: aggregate initial gates and source hash before/after.
- `first-run.log` and `first-run-exit-status.txt`: complete compiler/test output and exit 101.
- `artifact-sha256.json`: hashes of frozen corpus/protocol/runner and initial evidence.

The runner uses exclusive file creation and intentionally refuses to replace any first-run evidence. Re-running this integration test in the same checkout will refuse the existing observation files. Root should account for this single-use evidence runner when arranging later workspace validation; the evaluator has left the frozen runner unchanged.

This establishes an offline synthetic public-API result only. It does not establish live-provider, user, device, production, or external factual-truth acceptance.
