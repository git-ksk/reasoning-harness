# Engine 0.6 evidence-target relevance calibration v14 — immutable result

状態: FAIL。first/only canonical v14 result は immutable。v14 を rerun / rescore / relabel / retag しない。

Freeze:
- commit: 1b29ad7bf752e38070a44d58fbc0300d041bbbc2
- tag: engine-0.6-evidence-relevance-calibration-v14-freeze
- canonical run: 36174639970
- run attempt: 1
- final gate: failure
- fixed core: evidence-relevance-fixed-core-v1（48 cases）

fixed calibration core は閉じたまま維持し、この結果を理由にcaseを追加しない。

## Required Mistral arm

Model: ministral-8b-latest。

Operational:
- 48/48 completed
- successful provider cases 48
- provider failures 0
- provider-attempt telemetry complete
- model calls 96
- provider attempts 96
- latency p50/p95/max: 1,294 / 2,811 / 6,717 ms

Semantic / materialization:
- proposal exact 34/48（70.83%）
- local qualification exact 27/48（56.25%）
- blocking-reason miss 1
- spurious blocking reason 2
- materialized exact 35/48（72.92%）
- wrong-target / false Relevant retention 0
- false relevance rejection 0
- expected Relevant -> Ambiguous 11
- utility miss 13

13件のdisposition missは4パターンへ集中した。
1. 明確なpositive 6件 01, 05, 07, 65, 69, 74 でfalse explicit_local_absence=present。
2. 04, 08, 11, 12 でpositive targetがprimary unresolved。55 はtarget/relationともunresolved + spurious context_gap。
3. 60 でspurious ownership_scope。
4. 45 でexpected AmbiguousがIrrelevantへ落ちた。

semantic residualは、model-authored explicit local absence と positive evidence上のtarget bindingという2つのmodel authority surfaceへ集中した。

## Required Groq arm

Model: openai/gpt-oss-120b。

Operational:
- 48/48 attempts completed
- successful provider cases 6
- failed provider cases 42
- 42件すべて typed quota
- provider-attempt telemetry complete
- assessment-timeout cascadeなし
- model calls 55
- provider attempts 55

providerは tokens per day (TPD) 200,000 tokens/dayを明示。failure開始付近のUsedは約199,579。

v14 Groq hardeningは意図通り、daily quota exhaustionをtyped Quotaとしてfail-fastし、これらのcallでtransient rate-limit retryを消費しなかった。長いprovider waitが60秒assessment timeoutへ誤変換される以前の問題は解消した。

retryを無効化したわけではない:
- transient HTTP 429はbounded provider retryを維持
- daily quota exhaustionだけをshort-window limitとしてretryしない
- Groq semantic transportはstrict raw-JSON Textのまま

成功6件はexactだったが、required operational completeness FAILのためsemantic armはnon-scorable。

## Google replication

Model: gemini-3.5-flash-lite。

Operational:
- 48/48 attempts completed
- successful provider cases 13
- failed provider cases 35
- 35件すべてrunner上は assessment_timeout
- interruption caseではrunner境界のprovider-attempt telemetry incomplete
- latency p50/p95/max: 60,001 / 60,002 / 60,002 ms

成功した最初の13件はproposal / compact guard / final materializationがすべてexactで、安全性・utility missも0。

attempt telemetryで原因は明確。GoogleはHTTP 429と20〜59秒の Retry-After を繰り返し返し、adapterは既存bounded retryを正しくscheduleしていた。一方outer runnerはprovider wait/retry sleepもshared 60秒case deadlineへ課金しており、valid retry window待機中にcaseがcancelされ assessment_timeout になった。

これはsemantic regressionではなくoperational budget boundaryの問題。

## Cross-provider conclusion

v15ではfixed coreを増やさず3点を分離して直す。
1. explicit_local_absence に残るmodel authorityを削除またはdeterministicに制約し、Harness-owned identity floorを弱めずpositive-target bindingを改善。
2. provider throttle/retry待機をsemantic execution budgetから分離。ただしbounded retryとfinite absolute operational deadlineは維持。
3. daily quotaを持つrequired providerはtiny readiness probeではなく、full canonicalのprojected token costを考慮したcapacity preflightを行う。

新case追加の根拠はない。今回のsemantic failureはすべて evidence-relevance-fixed-core-v1 の既存dimension内。

## Final decision

required top-level acceptanceはFAIL:
- required operational completeness: false
- required correctness gate: false
- required utility gate: false
- required materialization gate: false
- required qualification-safety gate: false

v14はimmutable canonical FAIL。

independent holdout authoringは禁止継続。
