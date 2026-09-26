# Engine 0.6 candidate: evidence-target relevance calibration v8

Status: frozen v7 FAIL/incompleteと、独立freeze済みidentity-ambiguity / Google operational diagnostic後のfresh semantic successor calibration。

## Successor semantics

binding proposal v2は維持し、Harness materialization policy v4を導入する。

v3から変えるsemanticはnegative target identityだけ。

- `target=exact` はv3のrelation rule維持
- `target=unresolved` は `ambiguous` 維持
- `target=different` だけでは `irrelevant` を確定しない
- primaryが `target=different` の時だけone-sided negative-target confirmationを追加
- `confirmed_distinct_entity`: supplied local materialが別entity/productへsubstantive contentを明示的にbindできる場合のみ `irrelevant` を許可
- `confirmed_target_absent`: supplied local materialにtarget-specific local contentが無いことを確立できる場合のみ `irrelevant` を許可
- `not_confirmed` / confirmation欠落は `ambiguous`
- timeout / protocol failureはsemantic rejectionへ変換せずtyped operational failure

confirmation modelはalias / provenance / truth / authority / freshness / verification / final relevanceを作れない。

## Freeze前evidence

fresh identity ambiguity diagnostic v1 run `36029430165` は独立authorした12 synthetic case x matched seed 3回をMistral/Groqで測定。primary bindingはMistralでexpected-unresolvedを11回、Groqで1回false `different`へ寄せた。一方one-sided candidateは両providerでfalse confirmation 0、explicit-different confirmation miss 0、gated disposition 36/36。

Google full requalification v1 run `36078211994` はhistorical 26-case request shapeを26/26完走し、全call first attempt / HTTP 200、retry/cancellation/429/503 0、latency p50/p95/max 695/843/950 ms。Googleはoperationally requalifiedしたがv4-v6 historical tail/capacity instabilityは履歴として残す。

## Fresh calibration corpus

v8はsynthetic 32 case。

- 比較可能性のためhistorical 26 caseを維持
- explicit not-a-rename distinctness、generic target absence、explicit target absence、unknown alias、unknown successor、structured distinctnessのfresh 6 caseを追加
- expected negative-target confirmationはexpected primary target bindingが`different`のcaseだけfreeze
- production motivating product名/内容は除外

case elapsed budgetは60,000 ms維持。primary bindingとnegative-target confirmationは各stageごとにstructured-output model call最大2回のbounded budgetを持つが、全stageで同じ60秒case envelopeを共有する。confirmation outputは96 tokens上限。

## Provider roles

required semantic arm:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

non-gating full replication arm:

- Google `gemini-3.5-flash-lite`

Googleも同じ32 case / expected labelを使い、結果を保存・比較するがv8 release gateには含めない。Google attempt-level HTTP telemetryは維持。

## Acceptance

required各armでfirst/only frozen observationが以下すべてを満たす。

- planned/completed 32/32
- operational abortなし
- failed provider case 0
- wrong-target / unsafe relevance admission 0
- false relevance rejection 0
- expected-relevant left ambiguous 0
- utility miss 0
- expected confirmation caseでnegative-target confirmation exact 100%

proposal exact accuracyはdiagnostic。semantic release gateはmaterialized disposition。

Google replicationは同じsemantic metricsとoperational/attempt telemetryを報告するが、required gateを救済も失敗もさせない。

v8 FAIL時はimmutable FAILとしてrerunしない。fresh canonical calibration PASSまではindependent holdout authoring禁止。
