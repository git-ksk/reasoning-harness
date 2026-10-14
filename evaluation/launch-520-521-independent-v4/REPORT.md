# Engine #520/#521 independent v4 — first observation

Author: `/root/launch_072_independent`. 52 new offline synthetic inputs were authored after fixed candidate identity, without runtime/root repair test reads. Every positive uses a same-statement policy canonical name or declared alias with declared named-subject/direct-object launch vocabulary. Cross-sentence pronoun-only proof is explicitly negative. No retries, default replay, expectation edits, rescoring or runtime changes were performed.

**RED: 46/52 passed.** Safety: 25/26 passed; one false Relevant and one negative RequestedRelation (N408). Positive recall: 21/26 passed; five misses. API errors: 0. Both gates failed. Cargo exited 101 after saving every observation and aggregate gates.

Candidate: `48ac1ba8aba655aa73d192962d7996bcc997c5e0`.
Source: `crates/reasoning-harness-core/src/evidence_relevance.rs`.
Source SHA-256 before/after: `2b32a1e4a721cb169b15420e113ad91808476380ff912c12b46c029f8f81684d` (unchanged).
Freeze commit: `5f9af188ccc63d3ad558f45d237c0d920adb9858`, before v4 compilation/API execution.
Corpus SHA-256: `9fc73d42ad202af5416f365e5d938b6a690bb2181081b68a703ec2c9d28cb917` (verified after first execution).

Only excerpt/fact/structured_metadata provide factual proof. Heading/title govern or deny, never provide positive proof. URL/navigation cannot prove or govern facts. Scope belongs to the actual target and statement; unrelated non-launch properties/questions and different targets must not invalidate an affirmative launch. All API inputs include exact/exact proposal and optimistic exact-target/requested-relation/no-risk raw qualification. Positive requires RequestedRelation AND Relevant; negative requires neither. This evaluates source-local relevance, not external hard truth.

## Failures

All failures have identity_scope=exact_target and scope_risk=none. The five positive misses returned unresolved/ambiguous/conservative_fallback with reasons=[harness_canonical_name_anchor, local_qualification_disagreement, model_ambiguous]. N408 returned requested_relation/relevant/model_assisted with reasons=[harness_canonical_name_anchor, positive_target_local_binding_confirmed, model_relevant].

| ID | Frozen label | Category | Complete signals | v17 / v30 |
| --- | --- | --- | --- | --- |
| P404 | positive | introduced_neutral_leading_adjunct | excerpt: In the published chronology, Carvilo Horizon was introduced on 15 May 2153. | unresolved / ambiguous |
| P405 | positive | introduced_direct_object | fact: Berori Devices introduced Galventa Portal on 16 May 2154. | unresolved / ambiguous |
| P415 | positive | launched_direct_object | excerpt: Alvori Instruments launched Morliva Dial on 26 May 2164. | unresolved / ambiguous |
| P416 | positive | same_target_why_feature_question | excerpt: Why does Helvona Finch use a dial? Helvona Finch was introduced on 27 May 2165. | unresolved / ambiguous |
| P417 | positive | same_target_false_material_property | excerpt: The statement that Telriva Kite uses magnesium is false. Telriva Kite went live on 28 May 2166. | unresolved / ambiguous |
| N408 | negative | fictional_plural_entries | heading: These entries are fictional<br>fact: Yarona Chart became generally available on 14 May 2187. | requested_relation / relevant |

P405/P415 place the exact target in the direct-object position of introduced/launched. The named actor is not the target. P404 adds neutral published-chronology prose before a direct named-subject predicate. P416 asks a clearly non-launch property question about the same target before a separate introduction assertion. P417 denies a material property, not the target launch. All distinctions were frozen before execution. These findings do not by themselves establish whether a gap was newly introduced by this candidate.

## Passing controls

Positive IDs: P401, P402, P403, P406, P407, P408, P409, P410, P411, P412, P413, P414, P418, P419, P420, P421, P422, P423, P424, P425, P426.

Negative IDs: N401, N402, N403, N404, N405, N406, N407, N409, N410, N411, N412, N413, N414, N415, N416, N417, N418, N419, N420, N421, N422, N423, N424, N425, N426.

Passing positives cover active/passive/has-been launch; went-live; active/passive rollout; is/now/became generally-available; actor has-made direct object; entered-general-availability and reached-GA; on/as-of dates; declared aliases/dotted-decimal names; record adjunct; explicit other-target contexts; closed same-target property strike; and URL/navigation lacking governing authority. The feature-question heading P425 also preserved affirmative proof.

All original-symbol negatives N409–N416 passed, including alias under logical-not, original cross/not-equal symbols, multi-sentence strike, combining strike, and unclosed markdown strike. Conditional, question, denied, rumor, introducing-actor target, cross-sentence pronoun, heading/title-only and URL/footer-only cases N417–N426 all withheld correctly. Most fresh generic context nouns N401–N407 passed; plural deictic “These entries are fictional” in N408 remained unsafe.

## Evidence and replay

- `freeze.json`, `first-run-start.json`: frozen and initial execution provenance.
- `first-observations.jsonl`: complete input and both unmodified API outputs/errors for all52 cases.
- `first-failures.json`: exact subset of six failed first observations.
- `first-summary.json`: immutable initial gates and candidate source identity before/after.
- `first-run.log`, `first-run-exit-status.txt`: full compiler/test output and exit101.
- `artifact-sha256.json`: hashes of frozen corpus/protocol/runner and initial evidence.

Explicit first mode exclusively creates evidence. Default replay performs no filesystem writes and does not depend on first-file existence. No default replay was performed after the initial observation by this evaluator. Earlier v1/v2/v3 RED records and scope notes remain unchanged. Root owns repairs/integration.

Offline synthetic source-local public-API evaluation only. No provider, human, user, device, production or external factual-truth acceptance is claimed.
