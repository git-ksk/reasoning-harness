# 外部情報評価 v4 — モデル横断比較

Issue #208では、freeze済み`product-external-info-v4`の同条件4-arm測定を複数model familyへ展開し、Issue #216でGroqも運用上の追試対象へ追加しました。観測後に評価条件を都合よく変えないため、v4のcorpus、target proposition、採点、evaluator semantics、MCP境界、admission policy、finalization logicはsemantic head `e324ccbff6e818d205a734f06ccc8cac4b587588`から固定しています。v4 executableで許可する差分は`GroqAdapter`のprovider wiringだけで、`scripts/validate_product_external_info_v4_provider_wiring.py`がそれ以外をfreeze済みファイルとbyte-for-byteで照合します。

## 対象モデル

- Mistral `mistral-small-latest`
- Mistral `ministral-14b-latest`
- Google-hosted `gemma-4-31b-it`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`
- Groq `qwen/qwen3.8-27b`
- Groq `openai/gpt-oss-20b`

run `34000216929`のMinistral 8B結果はv4のcanonical初回観測のまま保持し、比較表の基準行としてのみ利用する。

## 先に要点：Harnessなし → ありで何が変わったか

この評価では、Harnessなし / ありへ同じtask、target hypothesis、根拠要件、authority policy、取得済みsnapshotを渡して比較します。18件の意味評価ケースのうち、答えるべき5件と、根拠不足のままにすべき13件を分けて採点します。

| Model | 回答到達率（5件） | `unknown`維持率（13件） | 不要な棄権 | 裏付けなしgrounded claim | 根拠不足の見逃し |
| --- | ---: | ---: | ---: | ---: | ---: |
| **Ministral 8B** | **80% → 100%** | **53.8% → 100%** | **1 → 0** | **6 → 0** | **6 → 0** |
| **Gemma 4 31B** | 100% → 100% | **84.6% → 100%** | 0 → 0 | **2 → 0** | **2 → 0** |
| **Gemini 3.5 Flash-Lite** | 100% → 100% | 100% → 100% | 0 → 0 | 0 → 0 | 0 → 0 |
| **GPT-OSS 120B** | **80% → 100%** | **76.9% → 100%** | **1 → 0** | **3 → 0** | **3 → 0** |

矢印はすべて **Harnessなし → Harnessあり** です。Ministral 8BとGPT-OSS 120Bでは、安全性を改善しながら回答到達率も80%から100%へ上がりました。

| Model | Harness / raw token | Harness / raw accounted latency |
| --- | ---: | ---: |
| **Ministral 8B** | 1.234x | 0.642x |
| **Gemma 4 31B** | 0.673x | 1.159x |
| **Gemini 3.5 Flash-Lite** | 0.641x | 1.034x |
| **GPT-OSS 120B** | 0.938x | 0.860x |

このコスト値は単一runの運用観測であり、安定した速度・コストランキングではありません。

Machine-readableな主要4行のartifact:

- [Ministral 8B](observations/product-external-info-v4-mistral-ministral-8b-seed-28000-2026-09-06.json)
- [Gemma 4 31B](observations/product-external-info-v4-google-gemma-4-31b-it-seed-28000-34001534798-2026-09-06.json)
- [Gemini 3.5 Flash-Lite](observations/product-external-info-v4-google-gemini-3.5-flash-lite-seed-28000-34002172470-2026-09-06.json)
- [GPT-OSS 120B](observations/product-external-info-v4-groq-openai-gpt-oss-120b-seed-28000-34008471577-2026-09-06.json)

## 固定条件

- corpus: `product-external-info-v4`
- evaluator: `reason-product-external-info-v4`
- comparison: `matched-target-context-four-arm-v4`
- seed: `28000`
- max tokens: `1024`
- 21ケース: semantic 18 + typed operational 3
- 各model run内ではarm 3とarm 4がcaseごとに同じdecoded acquisition snapshotを共有する
- raw/Harnessへ同じtask、exact target hypothesis、evidence requirement、authority policyを渡す

v4にはfreeze後のsnapshot injection interfaceを追加しないため、live external acquisition自体はmodelごとに再実行する。したがってモデル横断では同じfrozen endpointとsemantic contractを比較するが、external response byteまで完全一致することを保証するのは各run内のarm 3/4間だけである。matrix開始直前にmodel-free acquisition preflightを必須とする。

## 報告指標

各modelについてarm 3（`raw_model_with_external`）とarm 4（`harness_with_mcp_external`）のexpected-grounded target coverage、expected-unknown preservation、false target abstention、unsupported grounded claims、missed target insufficiency、Harness unsafe-admission/authority-promotion counter、typed rejection telemetry、model token、model latency、accounted end-to-end latency、operational failureを報告する。

provider/protocol failureはsemantic scoreと分離する。観測結果を理由にv4を変更して修復してはならない。


## Groq Free Tier向けの運用拡張 — Issue #216

Groq対象も同じfreeze済みv4 corpus、seed `28000`、max-output `1024`、4-arm scoring contractを使う。GroqはOpenAI-compatible Chat Completions endpointと`GROQ_API_KEY`で接続し、model IDはadapter内のsemantic branchではなくdataとして扱う。

manual Groq laneの対象は`openai/gpt-oss-120b`、`qwen/qwen3.8-27b`、`openai/gpt-oss-20b`。Groqが現在公開しているFree Plan値は、この3モデルそれぞれ30 requests/minute、8,000 tokens/minute、1,000 requests/day、200,000 tokens/day。workflowではprovider-localに`REASON_GROQ_MIN_REQUEST_INTERVAL_MS=2100`と`REASON_GROQ_TOKENS_PER_MINUTE=8000`を設定する。adapterはrequest-startの最小間隔と直前responseの実token消費量を組み合わせ、HTTP 429では`Retry-After`/rate-limit reset headerを優先したbounded retryを行い、`REASON_GROQ_RATE_LIMIT_TELEMETRY=1`ではsecretを含まないrate-limit headerだけを診断出力できる。 Groq Structured Outputsはv4のHarness-owned schemaを書き換えず`strict:false`のbest-effort modeを使い、最終的なtyped validation/fallbackは既存Harness側に保持する。3モデルのFree Plan quotaはmodel別なのでworkflow matrixは最大3並列で実行する。

このpacing値はFree Plan用workflow設定であり、Groq全tier共通quotaの主張でもsemantic tuningでもない。prompt、output schema、fixture、acquisition、admission、verification、scoring、finalizationは変更しない。quota/rate-limitによるoperational failureはsemantic denominatorから分離する。

## 初回モデル横断観測 — 2026-09-06

GitHub Actions run `34001534798` では、v4のcase、target、scoring、evaluator semanticsを変更せず、freeze済み評価面をそのまま再利用した。

### Gemma 4 31B

`google / gemma-4-31b-it` は21ケースを完走した。

- raw + external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `11/13 = 0.8462`、unsupported grounded claims `2`、missed target insufficiency `2`。
- Harness + MCP external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `13/13 = 1.0`、false target abstention `0`、unsupported grounded claims `0`、missed target insufficiency `0`、identity-unsafe admission `0`、MCP-output authority self-promotion `0`、safety gate pass。
- rawがunsafe側へ倒れた2件は `authority-numpy-claim-cannot-self-promote-v4` と `insufficient-generic-content-no-envelope-v4`。
- raw + externalは17,244 model token、Harness + externalは11,601 token（raw比 `0.673x`）。accounted end-to-end latencyはraw 91,983 ms、Harness 106,611 ms（`1.159x`）。いずれも単発runのoperational observationであり、安定した性能順位は主張しない。

### Gemini 3.5 Flash-Lite

run `34001534798` の最初の未paced `google / gemini-3.5-flash-lite` attemptはcase 9でAI Studio free-tier request quota（`15 requests/minute`）に到達した。この未完了attemptはoperational evidenceとして保持し、semantic scoreには含めない。

run `34002172470` のprovider-only paced retryでは `REASON_GOOGLE_MIN_REQUEST_INTERVAL_MS=4500` を使用した。この設定はopt-inかつdefault無効で、request startのtimingだけを変更する。v4のrequest内容、response、fixture、scoring、admission、verification、finalizationは変更しない。paced retryは21ケースを完走した。

- raw + external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `13/13 = 1.0`、false target abstention `0`、unsupported grounded claims `0`、missed target insufficiency `0`。
- Harness + MCP external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `13/13 = 1.0`、false target abstention `0`、unsupported grounded claims `0`、missed target insufficiency `0`、identity-unsafe admission `0`、MCP-output authority self-promotion `0`、safety gate pass。
- raw + externalは17,209 model token、Harness + externalは11,030 token（raw比 `0.641x`）。accounted end-to-end latencyはraw 94,663 ms、Harness 97,917 ms（`1.034x`）。provider pacing時間はmodel-call latency counterへ完全には含まれないため、これは性能順位ではなくoperational observationとして扱う。

このmodelとfreeze済みcorpusではraw external arm自体が完全にsafeかつcompleteだったため、Harnessによるsemantic scoreの改善はなかった。一方でcoverageを落とさず同じcorrectness boundaryを維持した。

### Mistral Smallの運用上のブロッカー

`mistral / mistral-small-latest` はscored reportを生成できなかった。初回attemptと時間を空けたretryの両方で最初のcaseからHTTP 429となり、bounded backoff中も `x-ratelimit-limit-req-minute=0` / `x-ratelimit-remaining-req-minute=0` が継続した。semantic caseを1件も完了できていないため、このmodelはcross-model correctness denominatorから除外する。これはprovider rate-limit stateであり、Harness semanticsのevidenceではない。

### Ministral 14B

run `34002627232` では、freeze済みv4 contractを変更せず `mistral / ministral-14b-latest` が21ケースを完走した。

- raw + external: grounded target coverage `4/5 = 0.8`、expected-unknown preservation `9/13 = 0.6923`、false target abstention `1`、unsupported grounded claims `6`、missed target insufficiency `4`。
- Harness + MCP external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `13/13 = 1.0`、false target abstention `0`、unsupported grounded claims `0`、missed target insufficiency `0`、identity-unsafe admission `0`、MCP-output authority self-promotion `0`、safety gate pass。
- raw + externalは47,843 model token、Harness + externalは33,936 token（raw比 `0.709x`）。accounted end-to-end latencyはraw 187,730 ms、Harness 90,557 ms（`0.482x`）。単発runのoperational observationであり、安定した性能順位は主張しない。
- live response headerでは、このrun中の14Bに `30 requests/minute` と `937,500 tokens/minute` が返った。これはmodel/account固有のruntime evidenceであり、Mistral全tier共通値とは主張しない。

14Bではparameter数が増えてもraw external groundingのunsafeが単調に消えず、unsupported grounded claimは6件残った。一方Harness laneはtarget coverageを100%へ戻し、unsafe exposureを0に維持した。

## Groq初回観測 — run `34008471577`

Groq Structured Outputsをv4のHarness-owned schemaを変更しない`strict:false` best-effort modeへ切り替えた後、`openai/gpt-oss-120b` と `qwen/qwen3.8-27b` はrun `34008471577`で21ケースを完走した。`openai/gpt-oss-20b` は同runでprovider-side JSON validation 400により未完了だったため、Groq公式のbest-effort structured-output guidanceに沿って当該validation 400だけをbounded retryするadapter修正を追加した。その後main上のrun `34009503385`で20Bも21ケースを完走した。v4 corpus、prompt、output schema、scoring、admission、verification、finalization semanticsは変更していない。

### GPT-OSS 120B

- raw + external: grounded target coverage `4/5 = 0.8`、expected-unknown preservation `10/13 = 0.7692`、false target abstention `1`、unsupported grounded claims `3`、missed target insufficiency `3`。
- Harness + MCP external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `13/13 = 1.0`、false target abstention `0`、unsupported grounded claims `0`、missed target insufficiency `0`、identity-unsafe admission `0`、MCP-output authority self-promotion `0`、safety gate pass。
- raw + externalは26,897 model token、Harness + externalは25,237 token（raw比 `0.938x`）。accounted end-to-end latencyはraw 186,199 ms、Harness 160,182 ms（`0.860x`）。
- 4 arm合計のmodel tokenは97,424。これは単発runのoperational observationであり、安定した速度・コスト順位は主張しない。

### Qwen 3.8 27B

- raw + external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `12/13 = 0.9231`、false target abstention `0`、unsupported grounded claims `1`、missed target insufficiency `1`。
- Harness + MCP external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `13/13 = 1.0`、false target abstention `0`、unsupported grounded claims `0`、missed target insufficiency `0`、identity-unsafe admission `0`、MCP-output authority self-promotion `0`、safety gate pass。
- raw + externalは17,660 model token、Harness + externalは8,382 token（raw比 `0.475x`）。accounted end-to-end latencyはraw 82,842 ms、Harness 152,932 ms（`1.846x`）。
- 4 arm合計のmodel tokenは45,964。Harness laneはtokenを大きく削減した一方、この単発runではlatencyが増加したため、token効率とwall-clock速度を同一視しない。


### GPT-OSS 20B — run `34009503385`

- raw + external: grounded target coverage `4/5 = 0.8`、expected-unknown preservation `7/13 = 0.5385`、false target abstention `1`、unsupported grounded claims `6`、missed target insufficiency `6`。
- Harness + MCP external: grounded target coverage `5/5 = 1.0`、expected-unknown preservation `13/13 = 1.0`、false target abstention `0`、unsupported grounded claims `0`、missed target insufficiency `0`、identity-unsafe admission `0`、MCP-output authority self-promotion `0`、safety gate pass。
- raw + externalは25,216 model token、Harness + externalは33,019 token（raw比 `1.309x`）。accounted end-to-end latencyはraw 226,687 ms、Harness 170,651 ms（`0.753x`）。安全性とcoverageは改善した一方、token消費は約31%増え、latencyは約25%短縮した。
- 4 arm合計のmodel tokenは109,664。Harness armのmodel attemptsは25で、Raw armの21より多い。これはGroq best-effort structured-outputのbounded retryを含むoperational costであり、semantic scoreとは分離して扱う。

3モデルすべてで、Harness laneはexpected-grounded target coverageを失わず、expected-unknown preservationを100%へ戻し、unsupported grounded claimsとmissed target insufficiencyを0にした。Raw external armでは120Bに3件、Qwen 27Bに1件、20Bに6件のunsupported grounded claimが残った。model規模やfamilyだけではexternal-grounding safetyを保証できず、token/latency効果もmodelごとに異なるという既存v4傾向を補強する。
