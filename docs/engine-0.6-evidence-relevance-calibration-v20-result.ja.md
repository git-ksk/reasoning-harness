# Engine 0.6 evidence-target relevance calibration v20 — immutable result

Status: FAIL。first/only frozen v20 canonicalとしてimmutable。v20はrerun / rescore / relabel / retagしない。

Freeze:
- commit: `e9ca7cce6a60c9b69129375aeea62a60cc3b127f`
- tag: `engine-0.6-evidence-relevance-calibration-v20-freeze`
- canonical run: `36283988719`
- run attempt: `1`
- preflight: success
- final gate: failure
- fixed core: `evidence-relevance-fixed-core-v1`（48件）
- annotation protocol: `evidence-relevance-effective-qualification-v20`

この結果を理由にfrozen case / label / expected dispositionは変更しない。

## Required Mistral arm

Model: `ministral-8b-latest`。

Operational:
- 48/48完走
- provider success 48 / failure 0
- provider attempts 96/96 completed
- model calls 96
- total tokens 84,234
- active execution 56,910 ms
- provider/retry wait 0 / 0 ms
- provider-arm latchなし

Semantic/materialization:
- proposal exact: 36/48（75.00%）
- raw verifier exact: 26/48（54.17%）
- raw scope-risk miss / spurious: 10 / 1
- raw identity-scope miss: 14
- raw relation-scope miss: 14
- effective qualification: 48/48 exact
- effective risk / identity / relation miss: 0
- materialized exact: 48/48
- wrong-target Relevant retention: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 0

Mistralはfrozen v20 surfaceに対してcomplete semantic PASS。

## Required Groq arm

Model: `openai/gpt-oss-120b`。

Operational:
- successful provider cases: 39
- failed provider cases: 9
- `31_possible_successor_unconfirmed`でtyped daily quota
- confirmed quota直後にprovider arm latch
- 残り8件を抑止
- provider attempts: 79/79 completed
- model calls: 79
- latch前total tokens: 97,259
- active execution: 53,336 ms
- provider/pacing wait: 661,643 ms
- retry wait: 0 ms

quota circuitは再度設計通りで、fail-fast / 即arm latch / retry stormなし / assessment-timeout変換なし。

ただしv20ではquota前にsemantic safety missも1件露出した。
- case: `13_same_service_different_feature`
- expected proposal: `target=exact, relation=different`
- observed primary proposal: `target=exact, relation=exact`
- raw verifier: `identity=exact_target, relation=different_relation, risk=none`
- effective qualification: `identity=exact_target, relation=requested_relation, risk=none`
- expected disposition: Irrelevant
- materialized disposition: Relevant

Harness-owned effective qualifierが、deterministic requested-relation cueがない状態でもprimaryの`relation=exact`を独立verifierの`different_relation`より優先してしまい、wrong-target / false Relevant 1件につながった。

Groq successful observation上:
- effective qualification derivations: 39
- effective exact: 37
- effective relation-scope miss: 2
- materialized exact: 38/39
- wrong-target Relevant retention: 1
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 0

もう1件のeffective relation missは`25_insufficient_local_passage`。deterministic `context_gap`により最終Ambiguousは維持され、terminal safety failureではない。

## Google replication

Model: `gemini-3.5-flash-lite`。non-gating。

Operational:
- 48/48完走
- provider success 48 / failure 0
- provider attempts 96/96 completed
- model calls 96
- total tokens 86,842
- active execution 141,930 ms
- provider/pacing wait 228,023 ms
- retry wait 0 ms

Semantic/materialization:
- proposal exact: 36/48（75.00%）
- raw verifier exact: 28/48（58.33%）
- effective qualification exact: 45/48
- effective relation-scope miss: 3
- effective risk / identity miss: 0
- materialized exact: 48/48
- wrong-target Relevant retention: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 0

3件のeffective relation mismatchは従来と同じcontext-gap shape（25 / 26 / 80）で、最終Ambiguousは維持。v20はv19で残っていたGoogle case 13をIrrelevantへ正しく改善した。

## Cross-provider conclusion

v20 materialization v15はMistralとGoogleでは成立したが、required acceptanceは2つの独立理由でFAIL。
1. Groq typed daily quotaによるoperational incompleteness。
2. Groq case 13でeffective qualifierのauthority conflictからsemantic safety miss。

successor設計ではquotaよりsemantic missの方が重要。単にquota resetを待ってv20を再試行するのは不可で、v21でauthority-conflict ruleをgenericに修正する必要がある。

## Final decision

required top-level acceptanceはFAIL:
- required operational completeness: false
- required correctness gate: false
- required utility gate: required set incompleteのためaggregate final-gate false
- required materialization gate: false
- required qualification gate: false

v20はimmutable canonical FAIL。independent holdout authoringは禁止継続。
