# Engine 0.6 evidence-target relevance calibration v16 — immutable result

Status: FAIL。first/only canonical v16 resultはimmutable。v16をrerun / rescore / relabel / retagしない。

Freeze:
- commit: 98cc85d495d7e9dba49389f5cf15505088609fa4
- tag: engine-0.6-evidence-relevance-calibration-v16-freeze
- canonical run: 36221674316
- run attempt: 1
- preflight: success
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1 (48 cases)
- annotation protocol: evidence-relevance-scope-verifier-v16

fixed calibration coreはclosedのまま。この結果を理由にcaseを追加しない。

## Required Mistral arm

Model: ministral-8b-latest。

Operational:
- 48/48完走
- provider success: 48
- provider failure: 0
- provider-attempt telemetry complete: 96 started / 96 completed
- model calls / provider attempts: 96 / 96
- total tokens: 71,385
- operational abort: none
- provider-arm latch: none

Semantic / materialization:
- proposal exact: 36/48 (75.00%)
- local qualification exact: 25/48 (52.08%)
- scope-risk miss: 8
- spurious scope risk: 4
- identity-scope miss: 13
- relation-scope miss: 9
- materialized exact: 42/48 (87.50%)
- wrong-target / false Relevant retention: 0
- false relevance rejection: 0
- expected Relevant -> Ambiguous: 2
- utility miss: 6

6件のdisposition miss:
- Relevant -> Ambiguous: 04_semantic_paraphrase, 55_fresh_positive_injection_ignored
- Irrelevant -> Ambiguous: 15_navigation_only_match, 16_broad_landing_no_support, 29_explicit_no_target_information, 44_prompt_injection_local_absence

v15のcorrectness failureはv16で解消し、wrong-target Relevantは0。残りはconservative utility missのみ。

generic residualは2系統:
1. `allow_semantic_equivalent` でprimary target axisがunresolvedの場合、independent verifierがexact target/relation・risk noneでもbounded positive fallbackがない（case 04 shape）。
2. local verifierが`context_gap`を過剰使用し、navigation/footer identityを過大評価する。generic page、explicit local absence、明確なdifferent substantive owner、無視すべきprompt injection textが、usable negative scope evidenceではなくmissing contextとして扱われる場合がある。

## Required Groq arm

Model: openai/gpt-oss-120b。

Operational:
- provider success: 4
- provider failure: 44
- 07_url_omits_target_termsでtyped daily-quota failure
- quota直後にprovider arm latch
- 残り43件を抑止
- latch前4件はproposal / verifier / materializationすべてexact
- wrong-target Relevant: 0
- latch前utility miss: 0

runtime quota policyは設計通り。retry stormやtimeout変換なしで停止した。armはoperational incomplete / non-scorable。同じdaily windowでquota exhaustionを実測した直後なので、その既知のexhausted windowでsuccessor canonicalを意図的に消費しない。

## Google replication

Model: gemini-3.5-flash-lite。non-gating。

- provider success: 0
- early typed rate-limit 2件でcorrelated-capacity latch
- 残り46件を抑止
- non-scorable
- circuit動作は設計通り

semantic conclusionは出さない。

## Cross-provider conclusion

v16はv15から安全性・utilityとも大幅改善:
- Mistral materialized exact: 33/48 -> 42/48
- wrong-target Relevant retention: 1 -> 0
- utility miss: 14 -> 6

次successorはv16のagreement-based terminal policyを維持し、bounded generic changeのみ行う:
1. explicit `allow_semantic_equivalent` policyに限定し、primary relation exact + verifier identity/relation exact + scope risk noneの場合だけpositive fallbackを許可。strict identityのunresolved primaryはAmbiguous維持。
2. verifierを明確化し、candidate instructionはcontext lossではなくignore、navigation/footer target mentionはexact target ownershipを作らない、complete generic/broad unitとexplicit local absenceは`target_absent` / `relation_absent`、different substantive ownerはHarness targetがnavigationにしかなくても`distinct_target` + risk noneとする。
3. fixed 48件は変更せず、新policy boundaryはunscored structural/property controlだけ追加。
4. v16 operational budget / retry ownership / telemetry / sanitization / quota latch / correlated-capacity latchは維持。

v16はimmutable canonical FAIL。Independent holdout authoringは禁止継続。
