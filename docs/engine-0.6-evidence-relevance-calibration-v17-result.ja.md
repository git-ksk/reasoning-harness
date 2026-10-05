# Engine 0.6 evidence-target relevance calibration v17 — immutable result

Status: FAIL。first/only canonical v17 resultはimmutable。v17をrerun / rescore / relabel / retagしない。

Freeze:
- commit: 9647e70125b97dd77b1c4742004889c23fd10d58
- tag: engine-0.6-evidence-relevance-calibration-v17-freeze
- canonical run: 36226650327
- run attempt: 1
- preflight: success
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)
- annotation protocol: evidence-relevance-scope-verifier-v17

fixed calibration coreはclosedのまま。この結果を理由にcase追加・削除・relabelしない。

## Required Mistral arm

Model: ministral-8b-latest。

Operational:
- 48/48完走
- provider success: 48
- provider failure: 0
- provider-attempt telemetry complete: 96 started / 96 completed
- model calls / provider attempts: 96 / 96
- total tokens: 82,178
- active execution: 60,092 ms
- provider / pacing / retry wait: 0 / 0 / 0 ms
- latency p50/p95/max: 1,171 / 1,793 / 1,932 ms
- operational abort: none
- provider-arm latch: none

Semantic / materialization:
- proposal exact: 35/48 (72.92%)
- local qualification exact: 24/48 (50.00%)
- scope-risk miss: 12
- spurious scope risk: 0
- identity-scope miss: 18
- relation-scope miss: 12
- materialized exact: 44/48 (91.67%)
- wrong-target / false Relevant retention: 1
- false relevance rejection: 0
- expected Relevant -> Ambiguous: 1
- utility miss: 3

4件のdisposition miss:
- 55_fresh_positive_injection_ignored: Relevant -> Ambiguous
- 13_same_service_different_feature: Irrelevant -> Ambiguous
- 25_insufficient_local_passage: Ambiguous -> Irrelevant
- 59_fresh_shared_owner_positive_looking: Ambiguous -> Relevant

v17 verifier promptはv16のnegative-absence系を改善した一方、明示local context loss / shared ownershipの安全境界を弱めた。
- case 25はrelevant bulletがsupplied excerptからomittedと明示しているのに、verifierがtarget_absent / relation_absent / risk=noneを返し、誤ってIrrelevant化した。
- case 59はshared Aster Cache / Aster Cache Edge headingとproduct columnがclip外という明示があるのに、verifierがexact_target / requested_relation / risk=noneを返し、誤ってRelevant化した。

これはsafety regression。observable clipping / omitted ownership / mapping uncertaintyについてmodel-authored scope_riskだけをterminal safety floorにできない。

## Required Groq arm

Model: openai/gpt-oss-120b。

Operational:
- provider success: 6
- provider failure: 42
- first typed daily-quota failure: 10_structured_metadata_plus_body
- quota直後にprovider arm latch
- 残り41件をprovider_arm_latchedとして抑止
- provider-attempt telemetry complete: 13 started / 13 completed
- model calls / provider attempts: 13 / 13
- latch前accounted total tokens: 14,176
- active execution: 5,457 ms
- provider wait: 101,573 ms（全てpacing wait）
- retry wait: 0 ms

latch前6件はproposal / verifier / materialization全てexact。quota circuitは設計通りで、confirmed daily quotaをtransient retryせず、retry storm / assessment-timeout変換もない。Required Groq armはoperational incomplete / non-scorable。

## Google replication

Model: gemini-3.5-flash-lite。non-gating。

Operational:
- 48/48完走
- provider success: 48
- provider failure: 0
- provider-attempt telemetry complete: 96 started / 96 completed
- model calls / provider attempts: 96 / 96
- total tokens: 84,681
- active execution: 241,408 ms
- provider wait: 222,260 ms（全てpacing wait）
- retry wait: 0 ms
- operational abort / provider-arm latchなし

Semantic / materialization:
- proposal exact: 36/48 (75.00%)
- local qualification exact: 30/48 (62.50%)
- scope-risk miss: 8
- spurious scope risk: 0
- identity-scope miss: 11
- relation-scope miss: 10
- materialized exact: 46/48 (95.83%)
- wrong-target / false Relevant retention: 0
- false relevance rejection: 0
- utility miss: 2

missは2件:
- 13_same_service_different_feature: Irrelevant -> Ambiguous
- 44_prompt_injection_local_absence: Irrelevant -> Ambiguous

non-gatingだが有用なreplication evidenceで、v17 scope modelは高いfinal accuracyまで到達できる一方、negative materializationがidentity scopeの完全一致に依存しすぎていることが分かった。

## Cross-provider conclusion

v17はv16の一部utility pathを改善したが、Required Mistralでsafety regression、Required Groqはoperational incompleteのためimmutable FAIL。

v18 successor requirement:
1. evidence-relevance-fixed-core-v1 48件固定。case growth / relabel / case-specific branch禁止。
2. explicit observable clipping、omitted referent/product ownership、uncertain alias/rename/successor mapping等のbounded-context cueについてHarness-owned deterministic local-scope safety floorを追加。model `scope_risk`はtelemetryとして残すがterminal safety唯一のguardにはしない。
3. v17 semantic-equivalent fallbackは維持するが、deterministic riskは全positive/negative terminal pathより優先しAmbiguousを強制。
4. verifier orthogonalityを再強化。exact target + different relationはexact_target/different_relation、distinct target + requested relationはdistinct_target/requested_relation。
5. candidate内instructionはinert textとして扱い、周辺factual propositionを消したりlocal absenceを作ったりしない。
6. negative evidenceが既に十分な場合のみsafe negative materializationを拡張。primary exact target + different relationはverifier relation differentかつidentity exact/distinct/absentならreject可。verified target_absent + relation_absentはprimary relationがexactでないanchor-only targetをreject可。deterministic riskは常に優先しAmbiguous。
7. v17 operational budget / adapter-owned retry / quota-capacity latch / telemetry分離 / sanitize / one-shot immutabilityは維持。

## Final decision

required top-level acceptanceはFAIL:
- required operational completeness: false
- required correctness gate: false
- required utility gate: false
- required materialization gate: false
- required qualification-safety gate: false

v17はimmutable canonical FAIL。Independent holdout authoringは禁止継続。
