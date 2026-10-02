# Engine 0.6 evidence-target relevance calibration v5 result

Status: frozen FAIL。rerun / rescore禁止。

## Frozen identity

- issue: #462
- branch: feat/462-evidence-target-relevance
- freeze tag: engine-0.6-evidence-relevance-calibration-v5-freeze
- freeze commit: a6cdb5f6a6c67e7f1c7a0f514e649a461b2c0030
- first/only Actions run: 36008648993
- run attempt: 1
- suite: evidence-relevance-calibration-v5
- semantic contract: binding proposal v2 + target-first materialization v3
- case単位budget: 2 model calls / 192 output tokens / elapsed 60,000 ms

このrunはrerunしていない。後続successorがPASSしてもv5はimmutable historical evidenceとしてFAILのまま保持する。

## Mistral canonical arm

model: ministral-8b-latest

- operational: 26/26
- provider failure: 0
- materialized exact: 26/26
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0
- provider attempts: 26
- total latency: 20,327 ms

proposal exactは17/26。proposal exactnessはdiagnosticであり、semantic gateはHarness-materialized dispositionを主とする。

## Google canonical arm

model: gemini-3.5-flash-lite

- operational: 11/26
- provider failure: 15
- 完了caseのmaterialized exact: 11/11
- 完了caseのwrong-target / unsafe relevance admission: 0
- 完了caseのfalse relevance rejection: 0
- 完了caseのexpected-relevant left ambiguous: 0
- 完了caseのutility miss: 0
- failure class: assessment_timeout 14、provider_unavailable 1
- total observed latency: 1,261,376 ms
- 成功case latency median: 30,628 ms
- 成功case latency max: 49,319 ms

typed provider_unavailable は 10_structured_metadata_plus_body で発生し、Google provider attempt 4回・51,181 ms後にHTTP 503となった。provider messageはhigh demand状態を明示していた。ほか14 caseはadapterが完了結果を返す前に60,000 msのHarness deadlineへ到達したため、outer observation上は provider_attempts = 0 となる。これはHTTP request自体が開始されていないことを意味しない。

failure patternはsemantic regressionではない。Googleの完了caseはすべてexpected dispositionへmaterializeできている。

## 解釈

v5ではGoogle 3.5のtail latencyがv4の30秒envelopeを実際に超え、成功case自体が最大49.3秒だった。一方、providerが明示的にhigh demandを返して断続的に不安定だったため、単純にHarness deadlineをさらに延長する根拠にはならない。

v5だけを根拠に90/120秒へのblanket budget拡張、攻撃的materializer、model switch、unbounded retry追加は行わない。

v6をauthorする前に、v5 frozen semantic surfaceを変更しない小規模Google recovery diagnosticでtransient provider-capacity incidentか継続的不安定かを切り分ける。

v5がPASSしていないためindependent holdout authoringは引き続き禁止。
