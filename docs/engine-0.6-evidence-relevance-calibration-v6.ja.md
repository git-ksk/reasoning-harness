# Engine 0.6 candidate: evidence-target relevance calibration v6

Status: frozen v5 operational FAILと6-case Google recovery smoke FAILを受けたfresh successor identity。v1-v5およびrecovery diagnosticはimmutable historical evidenceとしてrerun / rescoreしない。

## v6が必要な理由

frozen v5 run `36008648993` ではsemantic regressionは観測されなかった一方、Google servingが大きく不安定だった。

- Mistral: 26/26 operational、materialized exact、correctness / utility miss 0。
- Google: 11/26のみoperational。14 caseがHarness 60,000 ms assessment timeout、1 caseがprovider attempt 4回後に明示的HTTP 503 high-demand。
- Google完了11 caseはmaterialized 11/11 exact。

続くfrozen recovery smoke run `36018360038` もv5 surfaceを変更せず6 caseを再観測したが4/6 operationalに留まり、`21_unknown_rename` と `26_url_only_identity` が再度60,000 ms timeoutとなった。単純な再試行・サンプル拡大だけでv6へ進む根拠は得られなかった。

*The Tail at Scale*、Google SRE overload/cascading-failure guidance、Gemini error guidance、AWS retry/jitter guidance、LLM serving evaluationの先行事例に基づき、semantic qualityとprovider reliabilityを分離し、retry amplificationをboundedにする。

## v6 operational hardening

v6ではrelevance semanticsを変更せず、provider/run reliabilityとtelemetryのみを変更する。

### Google provider retry jitter

既存のbounded attempt policyを維持する。

- maximum provider attempts: 4
- transient 5xx base schedule: 2 / 5 / 10秒
- rate-limit fallback base schedule: 10 / 20 / 40秒
- 明示的 `Retry-After` はそのまま優先

fallback waitはdeterministic delayから、各base delayの50%-100%範囲のbounded equal jitterへ変更する。attempt数や上限時間を増やさず、同期retry amplificationを抑える。

### Run-level operational circuit

runner側にはretry loopを追加しない。代わりに **operational provider failureが2件連続** した時点でrun-level circuitを開き、その後のmodel requestを停止する。

対象failure class:

- `assessment_timeout`
- `credentials`
- `provider`
- `provider_unavailable`
- `quota`
- `rate_limit`
- `timeout`
- `transport`

成功またはprovider以外のresponseで連続failure streakはresetする。途中abortしたcanonical runはoperationally incompleteのままFAILとし、失敗caseを隠す目的には使わない。

### Telemetry

v6では追加で以下を記録する。

- planned / completed cases
- structured operational-abort reason / remaining cases
- case単位provider-attempt telemetryがcompleteかどうか
- provider-attempt telemetry incomplete observation数
- total latency + p50 / p95 / max latency
- successful-case p50 / p95 / max latency

Harness outer timeoutでは、HTTP attempt開始後にadapterがattempt数を返す前にcancelされる可能性があるため、provider-attempt telemetryをincompleteとして明示する。

### Visible workflow gating

canonical workflowはraw artifactを必ず保存する一方、最終gateは両canonical armが26/26 operational、run-level abortなし、semantic correctness / utility gateすべてPASSでなければFAILする。artifact保存stepのgreenをacceptanceとは扱わない。

## semantic/evaluation surfaceで変更しないもの

- model-facing contract: `reason-evidence-relevance-binding-proposal-v2`
- Harness materialization: `target-evidence-relevance-binding-materialization-v3`
- strict Harness-owned identity floor
- case単位elapsed budget: 60,000 ms
- max model calls: 2
- max output tokens: 192
- Google canonical model: `gemini-3.5-flash-lite`
- Mistral canonical model: `ministral-8b-latest`
- Google request-start pacing: 6,000 ms + inter-case 6,100 ms
- 26 semantic caseおよびexpected proposal/disposition labelsすべて

hedged duplicate request、runner側の二重retry、90/120秒へのblanket deadline拡張は行わない。

## Fresh successor identity

- suite: `evidence-relevance-calibration-v6`
- issue: #462
- cases: 26
- seed: `4625606`
- status: `fresh_unobserved_calibration`
- production motivating incident: tuningから除外

corpusはcalibration comparabilityのため再利用する。independent holdoutではない。canonical calibrationがPASSするまでholdout authoringは行わない。

## Acceptance

first/only frozen v6 observationで、Mistral/Google両armが以下すべてを満たすこと。

- planned cases: 26
- completed cases: 26
- operational abort: none
- successful provider cases: 26/26
- failed provider cases: 0
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0

proposal exact accuracyおよびlatency統計はdiagnostic。semantic release gateはHarness-materialized dispositionとする。

first frozen v6 observationがFAILした場合、v6はimmutable failed evidenceとして保持しrerunしない。
