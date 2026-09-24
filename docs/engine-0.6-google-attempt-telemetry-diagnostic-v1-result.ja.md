# Engine 0.6 Google attempt telemetry diagnostic v1 result

Status: provider observation diagnosticとしてfrozen PASS。calibration v6のrerun / rescoreではない。

## Frozen identity

- issue: #462
- branch: `diag/462-google-attempt-telemetry-v1`
- freeze tag: `engine-0.6-google-attempt-telemetry-diagnostic-v1`
- freeze commit: `a483bb09ae006b05dd1f56807f54c643c181575a`
- first/only Actions run: `36026307543`
- run attempt: 1
- provider/model: Google `gemini-3.5-flash-lite`
- probe case: `01_exact_name_availability` / `02_acronym_alias` / `03_expanded_alias`
- Harness case deadline: 60,000 ms
- Google request-start pacing: 6,000 ms

## Result

3 caseすべてoperational success。

- planned / completed: 3 / 3
- successful / failed provider cases: 3 / 0
- provider attempts: 合計3、各call 1 attempt
- provider-attempt telemetry incomplete observation: 0
- retry: 0
- in-flight cancellation: 0
- HTTP status: 各callでheaders + completed responseの200のみ
- HTTP 429: 0
- HTTP 503: 0
- `RESOURCE_EXHAUSTED`: 0
- `UNAVAILABLE`: 0
- quota-window evidence: なし
- typed provider error evidence: なし

HTTP headers到達latency:

1. call 1: 35,871 ms
2. call 2: 28,397 ms
3. call 3: 33,530 ms

runner case latencyは35,872 / 28,397 / 33,531 ms。3 probeともmaterialized dispositionはexact。

frozen summary classificationは `no_quota_or_capacity_signal_observed`、`provider_operational=true`。

## 解釈

今回の観測から、v6の反復failureをactive quota / rate limitが原因とする証拠は得られなかった。v6で2件60秒assessment timeoutとなった同じearly request shapeが、その後は全てfirst attemptのHTTP 200で完了し、retryもquota signalもなかった。

v6時点ではouter cancellation前のattempt単位HTTP statusを記録していなかったため、v6中にhidden transient quotaが一切なかったとretroactiveに証明するものではない。ただし以下を合わせると、quota exhaustionよりintermittent serving-tail / capacity variabilityの説明が強い。

- v5では4 provider attempt後に明示的HTTP 503 high-demandを観測した。
- v6では47秒台のsuccess後、2 requestが60秒Harness deadlineまでcompleted adapter resultを返さなかった。
- 今回は同じearly request shapeが約28-36秒でHTTP 200、retry 0、quota signal 0で完了した。

3-case healthy probeだけでGoogleをrequired 26-case canonical providerへ再認定はしない。今回確定できるのはquota疑いの切り分けと、設定・semantic変更なしでもGoogle pathが回復し得ることまで。

## Consequence

この結果を理由に60秒semantic-assessment deadlineを延ばしたり、v6 operational hardeningを戻したりしない。bounded jitter、fail-fast load shedding、attempt telemetry、tail-latency reportingは維持する。

Googleはreplication / operational requalificationには利用できるが、required-provider gateはquota問題を仮定したり#462 semantic outcomeへfitさせたりせず、独立operational evidenceで決める。
