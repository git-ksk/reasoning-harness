# Engine 0.6 Google attempt telemetry diagnostic v1

Status: frozen v6 FAIL後のfresh / non-canonical provider diagnostic。

このdiagnosticはv6をrescoreせず、v6をPASSへ変更するものでもない。v7のprovider gate判断より前に、Googleの失敗がquota / rate-limit、provider capacity、またはHTTP response未着のtail latencyのどれとして観測されるかだけを切り分ける。

## Frozen question

v6でcase 01が47.2秒で成功し、case 02/03が60秒Harness timeoutになった同じ先頭3 requestについて、Harness deadlineへ達する前にGoogle adapter内部では何が起きているか。

## Cases

provider-attempt telemetryとcase identityを曖昧にしないため、各caseを別processで実行する。

- `01_exact_name_availability`
- `02_acronym_alias`
- `03_expanded_alias`

process間には7秒のgapを置く。modelは`gemini-3.5-flash-lite`、Harness case budgetは60,000 msのまま、semantic contractとexpected labelも変更しない。

## Telemetry

`REASON_GOOGLE_ATTEMPT_TELEMETRY_PATH` を指定すると、次のoperational metadataだけをappend-only JSONLで保存する。

- attempt start
- HTTP headers受信時のstatus
- response body解析後のtyped provider error class
- structured quota window
- provider status / high-demand message / quota ID / RetryInfoなどのbounded detail
- 安全なrate-limit header
- retry delay
- Harness deadlineによりadapter futureがdropされた場合の`cancelled_in_flight`

API key、prompt、system instruction、candidate evidence、model response本文はtelemetryへ保存しない。

## Interpretation

- HTTP `429` / `RESOURCE_EXHAUSTED`、`rate_limit`、`quota` は直接のquota/rate-limit evidence。
- HTTP `503` / `provider_unavailable` は直接のprovider-capacity evidence。
- `cancelled_in_flight` までにHTTP error statusが無ければ、そのattemptについてHarness deadline前にadapterがHTTP responseを受け取れなかったことを示す。providerが最終的に何を返したかまでは断定しない。
- signalが混在した場合はmixedとして保持し、semantic failureへ変換しない。

workflowはfirst/onlyで、rerunを拒否する。
