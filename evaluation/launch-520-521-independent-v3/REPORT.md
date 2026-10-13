# Engine #520/#521 independent v3 — first observation

Author: `/root/launch_072_independent`. 56 fresh offline synthetic cases were authored after the fixed candidate identity without reading implementation, root repair tests or prior reproduction material. No v1/v2 inputs were renamed or rescored. No retry, default replay, expectation edit or runtime edit was performed.

**RED: 47/56 passed.** Safety: 27/28 passed, one false Relevant and one negative RequestedRelation (N306). Positive recall: 20/28 passed, eight misses. API errors: 0. Both gates failed. Cargo exited 101 after all 56 complete observations and aggregate gates were saved.

Candidate commit: `fdd630e1085ab3994e3397de0646741313cfd15c`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256 before/after: `63493029537f8e66762332e118090abbbce5fdf1481e22b7df57124ab12b2c32` (unchanged).
Freeze commit: `0bbd47f91a1e4daa07d4ac9b569375b8f21c9212`, before any v3 compilation/API execution.
Corpus SHA-256: `07ebc8839e742c4715f9b58611ac688184cf93faa9a6005392c5d4a3d4b50659` (verified again after first execution).

Frozen positives use the declared bounded launch/GA vocabulary and factual proof in excerpt/fact/structured_metadata. Heading/title govern or deny only; URL/navigation cannot supply or govern factual proof. Target/statement scope, actor ownership and explicit pronoun ownership were specified before execution. All cases use exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. A positive requires RequestedRelation and Relevant; a negative requires neither. These are source-local relevance expectations, not external hard-truth assertions.

## Failures

All failures returned identity_scope=exact_target and scope_risk=none. Eight positive misses returned relation_scope=unresolved, disposition=ambiguous, path=conservative_fallback, reasons=[harness_canonical_name_anchor, local_qualification_disagreement, model_ambiguous]. N306 returned relation_scope=requested_relation, disposition=relevant, path=model_assisted, reasons=[harness_canonical_name_anchor, positive_target_local_binding_confirmed, model_relevant].

| ID | Frozen label | Category | Complete signals | v17 / v30 |
| --- | --- | --- | --- | --- |
| P305 | positive | rolled_out | fact: Draleno Loop was rolled out on 21 May 2035. | unresolved / ambiguous |
| P307 | positive | is_now_generally_available | excerpt: Volriso Board is now generally available as of 23 May 2037. | unresolved / ambiguous |
| P313 | positive | decimal_rolled_out | excerpt: Zeromil 9.4 was rolled out on 29 May 2043. | unresolved / ambiguous |
| P315 | positive | same_target_feature_question_before_launch | excerpt: How many connectors does Lyrvano Loom support? Lyrvano Loom was launched on 31 May 2045. | unresolved / ambiguous |
| P318 | positive | other_target_retracted_heading_with_predicate | heading: Retracted: Sovreta Signal went live on 1 May 2048.<br>structured_metadata: Pelvano Capsule was rolled out on 3 May 2048. | unresolved / ambiguous |
| P321 | positive | same_target_withdrawn_battery_heading | heading: Withdrawn battery-life claim: Rolvano Beam lasts eight days<br>fact: Rolvano Beam went live on 6 May 2051. | unresolved / ambiguous |
| P322 | positive | rolled_out_bounded_citation_adjunct | excerpt: Velmora Parcel was rolled out on 7 May 2052, according to the release record. | unresolved / ambiguous |
| P323 | positive | clear_same_target_deictic_subject | excerpt: Ilveta Gallery is the product covered by this record. It was launched on 8 May 2053. | unresolved / ambiguous |
| N306 | negative | hypothetical_dispatch_leading | excerpt: This dispatch is hypothetical. Ovriela Pond became generally available on 19 May 2065. | requested_relation / relevant |

P323 fixes an explicit antecedent in the immediately preceding sentence: the named target is the product covered by the record, followed by “It was launched”. N319 is the wrong-antecedent counterpart and passed withholding. This explicit pronoun-ownership expectation was documented before execution. P315 is a feature question about the same target followed by a separately affirmative launch sentence. P321 is a withdrawn battery-life claim whose content differs from the affirmative launch fact.

## Passing controls

Positive IDs: P301, P302, P303, P304, P306, P308, P309, P310, P311, P312, P314, P316, P317, P319, P320, P324, P325, P326, P327, P328.

Negative IDs: N301, N302, N303, N304, N305, N307, N308, N309, N310, N311, N312, N313, N314, N315, N316, N317, N318, N319, N320, N321, N322, N323, N324, N325, N326, N327, N328.

Passing positive controls cover launched/has-been-launched, introduced, went-live, became-generally-available, entered-general-availability and reached-GA frames; actor has-made ownership; ISO/month/day/day-May dates; bounded release-record citation; alias/dotted names; several explicit other-target and same-target non-launch marker scopes. Every original-symbol negative N307–N314 passed, as did questions, conditions, denial, actor-as-target misattribution, wrong pronoun antecedent, rumor, heading-only proof and URL/footer-only proof. N306 exposes a governing hypothetical “dispatch” missed across the sentence boundary.

## Evidence

- `freeze.json` and `first-run-start.json`: freeze/source/corpus and first-execution provenance.
- `first-observations.jsonl`: complete inputs and both unchanged API outputs/errors for every case.
- `first-failures.json`: exact subset of nine failed observations.
- `first-summary.json`: immutable initial gates and source hash before/after.
- `first-run.log` and `first-run-exit-status.txt`: full compiler/test output and exit 101.
- `artifact-sha256.json`: hashes of frozen corpus/protocol/runner and first evidence.

Explicit first mode creates evidence exclusively; normal CI replay is side-effect-free and never overwrites first files or fails because they exist. No normal replay was performed by the evaluator after the first observation. Earlier v1/v2 RED records remain untouched. Root owns diagnosis and repairs; these observations alone do not establish whether an observed gap was introduced by this candidate.

Offline synthetic source-local public-API evaluation only. No provider, human, user, device, production or external factual-truth acceptance is claimed.
