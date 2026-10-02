# Engine 0.6 Google full requalification v1

Status: frozen v4-v6 Google instabilityと3-case attempt-telemetry recovery probe後のfresh non-gating provider-operational diagnostic。

このstudyはcalibration v7のrerun / rescoreではなく、#462 relevance semanticsも変更しない。この結果だけでGoogleをrequired canonical providerへ戻さない。

## Question

Google `gemini-3.5-flash-lite` は、既にfreezeしたoperational envelopeのままevidence-relevanceの26 case request shapeをquota / provider-unavailable / timeout / circuit-abortなしで完走できるか。

## Frozen operational envelope

- model: `gemini-3.5-flash-lite`
- case deadline: 60,000 ms
- caseあたり最大model call: 2
- output token上限: 192
- Google request-start minimum interval: 6,000 ms
- inter-case delay: 6,100 ms
- adapter retryはbounded + jitter維持
- operational provider failure 2件連続でrun-level circuit open
- Google attempt telemetryはHTTP status / provider status / retry / cancellation metadataのみ記録し、request promptやresponse body本文は保存しない

## Request-shape set

frozen v7 calibrationの26 fixtureを全て`--fixture`で明示指定する。全request shapeを使うがrunner上は意図的にnon-canonicalとなる。

- v7 acceptance evidenceにはならない
- semantic labelは変更しない
- semantic metricsはdiagnosticとしてのみ保存
- provider gateはoperationalだけ

## Precommitted operational PASS

first/only frozen runが以下を全て満たした場合のみGoogleをこのstudy上でoperationally requalifiedとする。

- planned cases: 26
- completed cases: 26
- successful provider cases: 26
- failed provider cases: 0
- operational abort: なし
- runner exit: 0

HTTP 429 / `RESOURCE_EXHAUSTED`、HTTP 503 / `UNAVAILABLE`、retry、in-flight cancellation、attempt数、latency p50/p95/maxはPASS/FAILに関係なく保存する。

materialized exactness / correctness / utilityは報告するがprovider-operational verdictには混ぜない。将来Googleをrequired canonical providerへ戻す判断は別decisionとし、本結果だけでなくv4-v6 historical evidenceも合わせて扱う。

workflow rerunは禁止。FAIL / incompleteもそのままfrozen evidenceとして保持する。
