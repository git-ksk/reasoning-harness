# Engine 0.6 Google attempt telemetry diagnostic v1 result

Status: frozen diagnostic PASS。provider-operational evidenceであり、semantic calibration PASSではない。

## Frozen identity

- issue: #462
- freeze tag: `engine-0.6-google-attempt-telemetry-diagnostic-v1`
- freeze commit: `a483bb09ae006b05dd1f56807f54c643c181575a`
- first/only Actions run: `36026307543`
- run attempt: 1
- model: `gemini-3.5-flash-lite`
- request pacing: 6,000 ms
- inter-case delay: 6,100 ms
- Harness case deadline: 60,000 ms
- cases: `01_exact_name_availability`、`02_acronym_alias`、`03_expanded_alias`

このdiagnosticはfrozen calibration v6をrerun / rescore / modifyしない。

## Provider observation

3 caseすべてがprovider attempt 1回目でHTTP 200を返して完了した。

| case | provider attempts | latency | HTTP result |
| --- | ---: | ---: | --- |
| `01_exact_name_availability` | 1 | 35,872 ms | 200 |
| `02_acronym_alias` | 1 | 28,397 ms | 200 |
| `03_expanded_alias` | 1 | 33,531 ms | 200 |

集約結果:

- planned / completed: 3 / 3
- successful / failed provider cases: 3 / 0
- provider attempts: 3
- provider-attempt telemetry incomplete observations: 0
- retry: 0
- `cancelled_in_flight`: 0
- HTTP 429: 0
- HTTP 503: 0
- typed quota / rate-limit error: 0
- typed provider-unavailable error: 0
- `RESOURCE_EXHAUSTED` / `UNAVAILABLE` 等のprovider status: なし
- latency p50 / p95 / max: 33,531 / 35,872 / 35,872 ms

workflow classifierは次を出した。

`no_quota_or_capacity_signal_observed`

## Semantic observation

このdiagnosticはnon-canonicalであり、v6をrescoreしない。参考として、完了3 caseはいずれもexpected relevance dispositionへmaterializeし、semantic missは0だった。

## 解釈

このrunでは、persistentなproject quota / rate limitがrequestをblockしている証拠は出なかった。Harness deadline前にGeminiがquota/rate-limit responseを返していれば、attempt telemetryにHTTP 429とtyped `quota` / `rate_limit`、利用可能な場合はstructured quota windowが残る設計だが、今回は1件も観測されていない。

同時に、このrunでは503 / `UNAVAILABLE`も観測されなかった。3 requestはいずれもHTTP 200だった一方、first-attempt latencyは約28〜36秒と高い。

ただし、これだけでv6時点にquota responseが絶対なかったと遡及的に断定はしない。v6にはattempt-level HTTP telemetryがなく、2件のtimeoutはadapter futureがcompleted provider resultを返す前にcancelされたためである。

それでも現在のcombined evidenceは、persistent quotaよりGoogle serving latency / capacityの変動を強く支持する。

- v5ではbounded retry後にHTTP 503 high-demandを直接観測した。
- v6では47.2秒の成功1件後、60秒outer timeoutが2件続いた。
- 今回は同じ先頭request shapeが3件ともattempt 1のHTTP 200で返ったが、28〜36秒を要した。

したがってproviderは到達可能で、現在quota blockされているとは観測されていない一方、このevaluation pathではlatency変動が依然大きい。

## Premature v7 attempt

このdiagnostic完了前にv7 calibration freezeが開始されていたため、sequencing violationを検知した時点でrun `36026148264` をcancelした。preflightは完了したが、live provider armは両方cancelされた。v7 runはacceptance evidenceとして使用せず、同じidentityでrerun / rescoreしない。

次のcanonical calibrationはfresh identityを使用する。

## Gate implication

provider tail latencyを吸収する目的だけで60秒deadlineを延長せず、provider failureをsemantic failureへ読み替えない。

次のfresh canonical successorでは、required provider選定を#462 semantic scoreとは独立したprovider-operational evidenceで行う。Googleはreplication / diagnostic evidenceとして価値があるが、v4-v6でoperational instabilityが繰り返されたため、semantically stableなcandidateの評価可否をGoogle単独のavailabilityへ依存させない。独立根拠があるstable alternate providerをrequired gateへ使い、Googleをnon-gating replicationとして残す構成は妥当である。

fresh canonical calibrationがPASSするまでindependent holdout authoringは禁止を維持する。
