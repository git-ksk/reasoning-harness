# Natural-language E2E v33

## 目的

v33 は immutable な v32 の fresh held-out successor である。v32 freeze (`natural-language-e2e-v32-freeze`, `62a93e7edb12337edb36cb488f4cca79f49ff685`, seed `99584`) は historical evidence として固定し、rerun / rescore / reclassification を行わない。canonical release disposition は **VALID RELEASE FAIL**。Mistral paired と Groq candidate-only は PASS、Gemini / Gemma は Google quota/provider failure により operationally incomplete だった。Google 2 row の観測済み correctness-boundary violations は 0 であり、この不成立結果を reasoning regression として扱わない。

v33 candidate は `c34610863d23bb014c4b0cf49f801ab873f0f67f`（#339/#340 merge後の main）。v32 candidate からの product-runtime delta は、conflict-free な bounded structured `google.rpc.RetryInfo.retryDelay` (<=120s) を、daily/billing signal と衝突しない場合だけ既存の short-window `RateLimit` と分類する prospective Google adapter 修正のみ。explicit daily quota は必ず優先し、generic / ambiguous quota は従来どおり fail-closed とする。metric v13、released control `29a9e4be6273dbffeda324e15517dc64930ad315`、prompt/case semantics、correctness / authority / admission / verification / finalization boundary は変更しない。

## Fresh surface

- predecessor: `natural-language-e2e-v32-freeze` -> `62a93e7edb12337edb36cb488f4cca79f49ff685`
- corpus: `natural-language-e2e-v33`
- seed: `323518`
- collision check 対象を fresh identity / source ref / fact key / marker に持つ13 synthetic cases
- scoring: `natural-language-e2e-scoring-v33-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `c34610863d23bb014c4b0cf49f801ab873f0f67f`
- Cargo workspace version: `0.4.1`

## Prospective Google execution infrastructure

Google rate limit は project 単位で集計されるため、Gemini / Gemma canonical job は GitHub Actions の job layer で直列化 (`max-parallel: 1`) し、別runnerから同時発火させない。各 Google job は `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=3000` を使い、provider-side backoff 前でも request start を最大20回/分相当に抑える。immutable v32 で観測した 30 requests/minute の上限ちょうどを狙わず、window境界やjitterの余裕を残すための prospective operational policy である。

Gemma job 内だけ、eligible stateless investigation case を最大2 workerで処理できる。ただし request start は1つの absolute shared-pacer path を通して直列化し、2 worker は response latency の overlap のためだけに使う。adaptive follow-up、MCP non-promotion、session case は直列のまま。report順序は manifest case index を維持し、paired control -> candidate 順序も変更しない。

これは prospective operational control のみであり、semantic retry、malformed-output retry、whole-run retry、favorable case replacement、result-dependent rescheduling、scoring change は導入しない。

## Release discipline

v33 freeze tag / live label は deterministic pre-live validation と通常PR CIが green になるまで作成しない。freeze後は Mistral paired canonical を exactly once 実行し、PASS時だけ cross-model gate へ進む。Google各rowは独立必須、candidate operational failure は hard FAIL、`INCONCLUSIVE` は release不可、cross-model averagingは禁止。v33 の観測結果を見て設定を変更し、同じ v33 identity を再実行することも禁止する。
