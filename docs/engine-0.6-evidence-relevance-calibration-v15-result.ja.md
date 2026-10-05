# Engine 0.6 evidence-target relevance calibration v15 — immutable result

Status: FAIL。first/only canonical v15 resultはimmutable。v15をrerun / rescore / relabel / retagしない。

Freeze:
- commit: 80aba233aa375fa88d4515d68c586f9f0c5f02bb
- tag: engine-0.6-evidence-relevance-calibration-v15-freeze
- canonical run: 36217988625
- run attempt: 1
- preflight: success
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)
- annotation protocol: evidence-relevance-binding-verifier-v15

fixed calibration coreはclosedのまま。この結果を理由にcaseを追加しない。

## Required Mistral arm

Model: ministral-8b-latest。

Operational:
- 48/48完走
- provider success 48
- provider failure 0
- provider-attempt telemetry complete: 96 started / 96 completed
- model calls / provider attempts: 96 / 96
- total tokens: 67,667
- active execution: 53,152 ms
- provider wait / pacing wait / retry wait: 0 / 0 / 0 ms
- latency p50/p95/max: 1,037 / 1,589 / 1,926 ms

Semantic / materialization:
- proposal exact: 33/48 (68.75%)
- local qualification exact: 23/48 (47.92%)
- blocking-reason miss: 1
- spurious blocking reason: 13
- binding-confirmation miss: 14
- spurious binding confirmation: 1
- materialized exact: 33/48 (68.75%)
- wrong-target / false Relevant retention: 1
- false relevance rejection: 0
- expected Relevant -> Ambiguous: 3
- utility miss: 14

Disposition failureは次のshapeへ集中した:
- expected Relevant -> Ambiguous: 04_semantic_paraphrase, 55_fresh_positive_injection_ignored, 63_fresh_stale_contradiction_relevant
- expected Irrelevant -> Ambiguous: 13_same_service_different_feature, 14_sibling_product_overlap, 16_broad_landing_no_support, 17_unrelated_announcement, 18_comparison_only_mention, 19_relation_mismatch_same_target, 29_explicit_no_target_information, 50_fresh_sibling_local_scope, 60_fresh_comparison_sibling, 76_v13_sibling_different_relation_no_cue, 81_v13_exact_target_other_relation
- correctness failure: 59_fresh_shared_owner_positive_looking が expected Ambiguous から Relevant へ誤materialize

最重要のcorrectness failureはcase 59。primaryはunresolved/exact、independent verifierは blocking_reason=none + confirmed_target_relation を返し、materialization v10がこのverifier outputとHarness identity floorを使ってpositive rescueした。candidateはlocal ownershipが曖昧であり、wrong target/relation ownership bindingをRelevantとして通した。したがって現行contractではmodel-authored positive confirmation単独にunresolved primaryの救済authorityを持たせられない。

主要utility failureはverifierのover-abstention。Mistralはexpected blocker=noneの10件でcontext_gapを出し、identity_mapping / multipleの一部もcontext_gapへ潰した。そのためprimary target/relation axisが正しいnegative caseでもIrrelevantへ到達できないケースが多発した。

## Required Groq arm

Model: openai/gpt-oss-120b。

Operational:
- planned/completed observation: 48/48
- provider success: 43
- provider failure: 5
- 45_url_only_identity_fresh でtyped daily-quota failure 1件
- quota検出直後にprovider arm latch
- 残り4件をprovider_arm_latchedとして抑止
- provider attempts complete: 88 started / 88 completed
- model calls / provider attempts: 88 / 88
- latch前accounted total tokens: 90,148
- active execution: 49,606 ms
- provider wait: 725,682 ms（全てpacing wait）
- retry wait: 0 ms
- successful latency p50/p95/max: 17,740 / 18,364 / 18,532 ms

runtime quota policyは意図通り動作した。confirmed daily quotaをtransient short-window rate limitとしてretryせず、即時arm latchして確実に失敗するcallを抑止した。retry stormやassessment timeoutへの誤変換はない。この運用挙動はv16でも維持する。

quota latch前のsemantic evidenceだけでもacceptance不足:
- proposal exact accuracy: 83.72%（successful observation上）
- local qualification exact: run記録上26/48 expected case
- spurious blocking reason: 5
- binding-confirmation miss: 13
- materialized exact: successful observation中36
- wrong-target / false Relevant retention: 0
- false relevance rejection: 0
- expected Relevant -> Ambiguous: 1
- utility miss: 7

successful observationのdisposition missは04, 13, 14, 15, 16, 50, 81。したがってGroq daily capacityが残っていたとしてもv15はsemantic successorが必要だった。

## Google replication

Model: gemini-3.5-flash-lite。non-gating arm。

Operational:
- provider success: 0
- case 01はlocal qualificationまで進みtyped rate_limit
- case 02は30,000 ms retry wait後に2件目のtyped rate_limit
- 2-consecutive capacity failure circuitでarm latch
- 残り46件を抑止
- provider attempts: 4 started / 4 completed
- provider attempts incomplete observation: 0
- active execution: 1,335 ms
- provider wait: 35,287 ms
- pacing wait: 5,287 ms
- retry wait: 30,000 ms

replicationはnon-scorable。semantic conclusionは出さない。correlated capacity failureを早期停止するoperational circuitは設計通り動作した。

## Cross-provider conclusion

v15はquotaだけのFAILではない。Required Mistralが48/48 operational完走した上でcorrectness / utility / materialization / qualification-safety gateを独立にFAILした。Groqもquota latch前にsemantic missを示した。

v16 successorは次をconstraintとする:
1. evidence-relevance-fixed-core-v1を48件固定。case growth、v15 relabel、case-specific branchは禁止。
2. unresolved primaryへのpositive rescue authorityを廃止。verifier positive単独からRelevantを生成しない。
3. free-form binding_confirmationを廃止し、local identity scope / relation scope / concrete scope-riskを直交fieldへ分解する。modelにconflated confirmationを直接authorさせず、Harnessがdeterministicに導出する。
4. primary binding promptをHarness canonical/alias ownership、cross-language identity、stale-but-relevant、distributed same-target section、semantic-equivalent policy、untrusted candidate instructionについて明確化する。一方でshared ownership / clipped contextはunresolvedを維持する。
5. primary axisとverifier axisのagreementからRelevant / Irrelevant / Ambiguousを導出する。concrete identity/ownership/context riskは常にfail-closed。
6. v15のoperational budget、retry ownership、telemetry、sanitization、quota latch、correlated-capacity latch、manual TPD start gateなしの方針は独立operational evidenceがない限り変更しない。

## Final decision

required top-level acceptanceはFAIL:
- required operational completeness: false
- required correctness gate: false
- required utility gate: false
- required materialization gate: false
- required qualification-safety gate: false

v15はimmutable canonical FAIL。Independent holdout authoringは禁止継続。
