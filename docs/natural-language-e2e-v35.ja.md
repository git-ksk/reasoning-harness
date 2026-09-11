# Natural-language E2E v35

## 目的

v35 は immutable な v34 の fresh held-out successor である。v34 freeze (`natural-language-e2e-v34-freeze`, `0aa45d4bfb72b445ad287e2a4f0c5b3e455a7ee2`, seed `940105`) は historical evidence として固定し、rerun / rescore / reclassification を行わない。v34 の release disposition は **FAIL**。Mistral paired と Groq candidate-only は PASS、Gemini は candidate の Google HTTP 429 quota generation failure 2件（観測された free-tier request limit 15）により hard FAIL、Gemma は candidate 13/13・operational failure 0 に対して released v0.4.1 control が Google HTTP 500 / provider-unavailable で operationally incomplete だったため INCONCLUSIVE。各rowの correctness-boundary violations は 0 だった。

v35 candidate は `dd66a4372cfb462f876ac3169ba91df8d4a7f436`（#346/#347 merge後の main）。provider runtime crates は v34 candidate `0741e2ba06618b5be37fd90b85495dbd36c0b166` から変更しない。prospective な narrow Harness/eval delta は Google canonical pacing policy のみで、shared request-start floor を 3000 ms から **6000 ms** に変更する。これにより sustained canonical start は最大10回/分となり、v34で観測した15 RPM ceilingに対して5 RPM（約33%）のrequest-count headroomを持たせる。Google quota classification、provider retry、max provider attempts、request timeout、transient-5xx backoff、worker数、モデル順序、inter-case delay、metric v13、scoring、correctness / authority / admission / verification / finalization boundary は変更しない。

## Fresh surface

- predecessor: `natural-language-e2e-v34-freeze` -> `0aa45d4bfb72b445ad287e2a4f0c5b3e455a7ee2`
- corpus: `natural-language-e2e-v35`
- seed: `776466`
- fresh identity / source ref / fact key / marker を持つ13 synthetic cases。v34を含む全observed predecessorとのcollisionを検査する
- scoring: `natural-language-e2e-scoring-v35-metric-locked-v13`
- operational bounds: `natural-language-e2e-operational-bounds-v13`
- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`
- candidate: `dd66a4372cfb462f876ac3169ba91df8d4a7f436`
- Cargo workspace version: `0.4.1`

## Prospective Google execution infrastructure

Gemini / Gemma canonical job は GitHub Actions job layer で直列 (`max-parallel: 1`) のまま。各 Google job は `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=6000` を使用し、provider retryを含む全request startを同一model job内の shared pacer に通す。Gemmaのみ eligible stateless investigation case を2 workerで処理できる。adaptive follow-up、MCP non-promotion、session/stateful case は直列のまま。paired control -> candidate の順序も変更しない。inter-case delay は3000 msのままで、provider limiterとは分離する。

repository policy と v35 workflow / manifest の配線は、credential露出前に `validate_google_canonical_pacing.py` で機械検証する。genericな人間向けquota文言からretryabilityを推測せず、ambiguous quotaはfail-closedのまま維持する。

## Release discipline

v35 freeze tag作成・live label付与は deterministic pre-live validation と通常PR CIがgreenになるまで禁止する。freeze後はMistral paired canonicalを1回だけ起動し、PASSの場合だけcross-model gateへ進む。各Google rowは独立必須、candidate operational failureはhard FAIL、INCONCLUSIVEはrelease不可、cross-model averagingは禁止。観測後にv35設定を変えて同identityをrerunしてはならない。
