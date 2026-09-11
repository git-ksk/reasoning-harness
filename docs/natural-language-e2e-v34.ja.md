# Natural-language E2E v34

## 目的

v34 は immutable な v33 の fresh held-out successor である。v33 freeze (`natural-language-e2e-v33-freeze`, `1a03f1a700766cf0c31672601b45bd15eec2cdc7`, seed `323518`) は historical evidence として固定し、rerun / rescore / reclassification を行わない。v33 の canonical release disposition は **FAIL**。Mistral paired と Groq candidate-only は PASS、Gemini は control の1件の180秒 request timeout により INCONCLUSIVE（candidate は13/13完走・operational failure 0）、Gemma は Google HTTP 500 / `provider_unavailable` により candidate 10/13 で FAIL だった。観測済み correctness-boundary violations は各rowで 0 であり、provider operational failure を reasoning regression として扱わない。

v34 candidate は `0741e2ba06618b5be37fd90b85495dbd36c0b166`（#343/#344 merge後の main）。v33 candidate からの **product-runtime resilience delta は2点のみ**：Google request timeout を 180秒から300秒へ延長し、retryable HTTP 500/502/503/504 の transient backoff を `0.5s -> 1s -> 2s` から `2s -> 5s -> 10s` へ延長する。max provider attempts は4（初回+最大3 retry）のまま。rate-limit/quota classification、429 retry、Google request-start pacing、worker数、モデル順序は変更しない。metric v13、released control `29a9e4be6273dbffeda324e15517dc64930ad315`、prompt/case semantics、correctness / authority / admission / verification / finalization boundary は変更しない。

## Fresh surface

- predecessor: `natural-language-e2e-v33-freeze` -> `1a03f1a700766cf0c31672601b45bd15eec2cdc7`
- corpus: `natural-language-e2e-v34`
- seed: `940105`
- fresh identity / source ref / fact key / marker を持つ13 synthetic cases。v33を含む全observed predecessorとのcollisionを検査する
- scoring: `natural-language-e2e-scoring-v34-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `0741e2ba06618b5be37fd90b85495dbd36c0b166`
- Cargo workspace version: `0.4.1`

## Prospective Google execution infrastructure

v33で固定した実行条件をそのまま維持する。Gemini / Gemma canonical job は GitHub Actions job layer で直列化 (`max-parallel: 1`) し、各 Google job は `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=3000` を使用する。Gemma job 内だけ eligible stateless investigation case を最大2 workerで処理するが、request start は1つの absolute shared pacer を通す。adaptive follow-up、MCP non-promotion、session case は直列のまま。paired control -> candidate 順序も変更しない。

v34で新たに許可する operational resilience は adapter 内の request timeout 300秒化と transient 5xx backoff延長だけである。semantic retry、malformed-output retry、whole-run retry、favorable case replacement、result-dependent rescheduling、scoring change は導入しない。`paired-canonical-observation-v2` は immutable v33 と同一の evaluator-only orchestration helper を保持する。

## Release discipline

v34 freeze tag / live label は deterministic pre-live validation と通常PR CIが green になるまで作成しない。freeze後は Mistral paired canonical を exactly once 実行し、PASS時だけ cross-model gate へ進む。Google各rowは独立必須、candidate operational failure は hard FAIL、`INCONCLUSIVE` は release不可、cross-model averagingは禁止。v34 の観測結果を見て設定を変更し、同じ v34 identity を再実行することも禁止する。
