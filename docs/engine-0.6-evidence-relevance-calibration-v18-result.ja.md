# Engine 0.6 evidence-target relevance calibration v18 — immutable result

Status: FAIL。first/only canonical v18 resultはimmutable。v18をrerun / rescore / relabel / retagしない。

Freeze:
- commit: 8abb0f9b8d9b7e8a6859e6c791e7e600cdd54e9c
- tag: engine-0.6-evidence-relevance-calibration-v18-freeze
- canonical run: 36237860382
- run attempt: 1
- preflight: success
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)
- annotation protocol: evidence-relevance-scope-verifier-v18

fixed calibration coreはclosedのまま。この結果を理由にcase追加・削除・relabelしない。

## Required Mistral arm

Model: ministral-8b-latest。

Operational:
- 48/48完走
- provider success: 48
- provider failure: 0
- provider attempts: 96 started / 96 completed
- model calls: 96
- total tokens: 84,233
- active execution: 63,592 ms
- provider/retry wait: 0 / 0 ms
- provider-arm latch: none
- operational abort: none

Semantic/materialization:
- proposal exact: 37/48 (77.08%)
- local qualification exact: 26/48 (54.17%)
- materialized exact: 47/48 (97.92%)
- wrong-target / false Relevant retention: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 1
- deterministic guard override: 16

v18 deterministic local-risk floorによりv17のsafety regressionは解消した。特に`59_fresh_shared_owner_positive_looking`はprovider outputがpositive寄りでもAmbiguousにmaterializeし、unsafe Relevantは0になった。

唯一のdisposition missは`14_sibling_product_overlap`: expected Irrelevant -> Ambiguous。observed primaryは`target_binding=unresolved, relation_binding=different`、independent verifierは`identity_scope=distinct_target, relation_scope=different_relation, scope_risk=none`。v13はprimary target axisがunresolvedのためterminal target-negativeを許可しなかった。correctness failureではなくconservative utility miss。

## Required Groq arm

Model: openai/gpt-oss-120b。

Operational:
- provider success: 13
- provider failure: 35
- quota前13 successful observationはproposal / qualification / materializationすべて13/13 exact
- `74_v13_exact_target_same_relation_no_cue`のlocal qualification中にtyped daily-quota failure
- confirmed quota直後にprovider arm latch
- 残り34件を抑止
- provider attempts: 28 started / 28 completed
- model calls: 28
- latch前total tokens: 32,572
- active execution: 12,971 ms
- provider wait: 231,724 ms（全てpacing wait）
- retry wait: 0 ms

runtime quota circuitは設計通り。retry storm / timeout変換なし。Required Groq armはoperational incomplete / non-scorableのため、13件が全exactでもv18 PASS不可。

## Google replication

Model: gemini-3.5-flash-lite。non-gating。

Operational:
- provider success: 47
- provider failure: 1
- `56_fresh_exact_exact_shared_row_abstain`でlocal-qualification semantic execution timeout 1件
- provider-arm latchなし
- provider attempts: 97 started / 96 completed

successful observation上のSemantic/materialization:
- proposal exact: 38/47 (80.85%)
- local qualification exact: run記録上28/48 expected case
- materialized exact: 42/47 (89.36%)
- wrong-target / false Relevant retention: 0
- false relevance rejection: 0
- Relevant -> Ambiguous: 0
- utility miss: 5

Disposition missは13, 15, 16, 36, 44で、全てexpected Irrelevant -> Ambiguous。v18 safety floorがprovider横断でconservativeに動いている一方、negative disagreementはmodel-sensitive。

## Cross-provider conclusion

v18はRequired Mistralをv17 44/48から47/48へ改善し、unsafe Relevantも0へ復帰。Groqもquota前13 successful observationは完全一致。Required semantic residualはconservative negative disagreement 1件まで絞れた。

v19 successorは最小変更とする:
1. primary proposal v5 / local verifier v8は変更しない;
2. deterministic local-risk floorも変更しない;
3. Harness-owned target-negative terminal ruleを1つ追加: deterministic riskなし + verifier `scope_risk=none` + verifier `identity_scope=distinct_target` + primary `target_binding != exact`なら、primary targetがunresolvedでもIrrelevantへmaterialize可能;
4. primary target exact、verifier riskあり、deterministic local riskありでは絶対に適用しない;
5. scored 48件のsemanticsは変更せず、新boundaryはunscored property testだけ追加;
6. v18 operational budget / retry / latch / telemetry / sanitizationは維持。

fixed-core監査では`primary target != exact + verifier distinct_target + risk none`を満たすexpected Ambiguousは0件で、expected instanceは全てIrrelevant。このためscored label変更なしでv19 ruleを導入できる。

## Final decision

required top-level acceptanceはFAIL:
- required operational completeness: false（Groq quota latch）
- required correctness gate: final-gate aggregateではrequired operational completeness未達のためfalse
- required utility gate: false（Mistral utility miss 1 + Groq incomplete）
- required materialization gate: false
- required qualification gate: false

v18はimmutable canonical FAIL。Independent holdout authoringは禁止継続。
