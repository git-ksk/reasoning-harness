# モデル横断セマンティック判定器の適合性

この文書は、advisory semantic judge の model implementation 間における portability を追跡する。モデルの順位付けは行わず、model agreement を truth とみなすこともない。

## 権限境界

live semantic output はすべて信頼されない observation のままである。model-backed `SoftJudgeOutput` は verification receipt、hard finding、verdict、trusted evidence、epistemic promotion、final-answer authority のいずれも作成できない。provider/protocol failure は operational failure のままであり、`no_finding` にはならない。未完了 trial は semantic denominator の外に置く。hidden chain of thought は保存も採点もしない。

## `soft-semantic-v3` ホールドアウト v2 の行列

Holdout-v2 は凍結済みで、すでに観測されている。以下は v3 portability の診断にのみ使い、successor contract の tuning target にはしない。

| Model | Operational / protocol | Precision | Recall | Coverage | Ambiguous abstention | Fallback | Main conformance signal |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| `ministral-8b-latest` | 140/140, 5/5 complete | 1.000 | 1.000 | 0.700 | 0.933 | 0.371 | v3 の uncertainty boundary への観測上最も強い準拠 |
| `mistral-small-latest` | 140/140, 5/5 complete | 0.982 | 1.000 | 1.000 | 0.000 | 0.071 | systematic over-assertion / abstention collapse |
| `gemini-3.1-flash-lite` | 140/140, 5/5 complete | 1.000 | 1.000 | 0.800 | 0.622 | 0.000 | protocol issue を伴わない中程度の保守性 |
| `ministral-14b-latest` | 135/140; same protocol failure in all five trials | n/a | n/a | n/a | n/a | n/a | finding object を伴う repeated non-finding decision |
| `nvidia/nemotron-3.5-lightning-30b-a3b` | initial holdout result invalid for semantic interpretation; calibration after reasoning minimization reached 14/18 | n/a | n/a | n/a | n/a | 0.929 on successful calibration calls | finding bias plus typed-output/schema-completeness failures |

該当 run ID は `33318380199` (Ministral 8B)、`33319598306` (Mistral Small)、`33318691626` (Gemini 3.1 Flash-Lite)、`33318689080` (Ministral 14B)、`33321109608` (initial Nemotron holdout) である。Nemotron の初期 0/140 は semantic score ではない。content-free diagnostics で reasoning-token truncation が判明したためである。provider-neutral reasoning minimization 後の run `33340942700` では length-truncation の交絡は除かれたが、semantic/protocol failure は残った。`research/51-bounded-reasoning` の bounded-reasoning experiment は悪化したため、意図的に merge していない。

この matrix は contract portability の evidence であり、scalar capability ordering ではない。1行だけを改善する変更は presumptively model overfit とする。harness の改善は model/provider family をまたいで一般化すべきである。

## `soft-semantic-v4` 後継版

Issue #53 は v3 の semantic intent を保ったまま representation を簡素化する。

Global rule:

- `finding`: supplied context が requested diagnostic concern を肯定的に支持する。
- `no_finding`: supplied context が concern を肯定的に解決または否定する。
- `abstain`: binding、scope、applicability、authority、evidence adequacy のいずれかが unresolved、mixed、partial で、どちらの結論も十分に支持されない。

Diagnostic-kind text は requested concern のみ定義し、kind ごとの別個の three-way policy は繰り返さない。Causal-gap semantics は correlation-only、direction のない temporal/mechanism-only、explicit confounding、undistinguished viable reverse direction を affirmative gap evidence とし、不完全・partial・scoped な evidence だけでは gap を確立しない。

Model-facing schema は validation を弱めず厳格化した。Structured output は discriminated union である。

- `finding` は typed finding を要求する。
- `no_finding` は finding object を許可しない。
- `abstain` は finding object を許可しない。

parsed model DTO は既存の internal `SoftJudgeOutput` に変換され、その後も既存の exact kind/target validation が実行される。public soft/hard authority surface は変わらない。

## 凍結 v4 互換性基準

holdout-v3 に対する v4 provider run の前に、次の基準を固定した。

model が `conformant` となるのは、5つすべての holdout-v3 trial が完了し、protocol conformance が 100%、aggregate precision/recall がそれぞれ少なくとも 0.95、各 complete trial の precision/recall が少なくとも 0.90、aggregate ambiguous abstention が少なくとも 0.80、各 complete trial が少なくとも 0.70、かつ labelled fixture が complete trial 間で `finding` と `no_finding` の間を直接 oscillate しない場合だけである。

`usable_with_limitations` は、5 trial 完了、protocol conformance 100%、aggregate precision/recall >= 0.90、aggregate ambiguous abstention >= 0.50、意図的に ambiguous な case が family-wide に一つの assertive decision へ collapse しない場合だけである。provider availability により完全な測定ができない場合、semantic tier は未割当とし semantic failure score を捏造しない。

Fallback dependence は報告するが semantic gate ではない。provider ごとに JSON-Schema capability が異なるためである。

v4 を portability improvement として採用するのは、異なる provider family から少なくとも2つの conformant model があり、さらに1 model が conformant または usable with limitations で、model/provider 固有の semantic branch を導入せず、deterministic hard/resolution safety gate が green の場合だけである。

## 独立ホールドアウト v3 の凍結

`fixtures/semantic-judges-holdout-v3/` は v4 用の independent observation-free corpus である。v4 provider measurement 前に凍結され、diagnostic kind ごとに7 fixture、合計28 fixture（positive 8、negative 8、intentionally ambiguous 12）を含む。source `recorded_observations` は空のままである。

holdout-v1/v2 は historical/diagnostic corpus のままにする。holdout-v3 で v4 result を観測した後、material な contract/schema change には新しい configuration identity と別の independently frozen holdout が必要であり、holdout-v3 を in place で tuning してはならない。

## `soft-semantic-v4` ホールドアウト v3 の結果

independent matrix は merged commit `3774e4f19db9da11cfd2ea065792b78b53b0c9dd`、model ごとに5 sequential trial、256 output tokens、provider-safe fixture concurrency を用いた。frozen compatibility threshold は観測後に変更していない。

| Model | Run | Operational / protocol | Precision | Recall | Coverage | Ambiguous abstention | Fallback | Frozen tier |
| --- | ---: | --- | ---: | ---: | ---: | ---: | ---: | --- |
| `ministral-8b-latest` | `33342332130` | 140/140, 5/5 complete | 0.889 | 1.000 | 0.714 | 0.667 | 0.429 | non-conformant |
| `mistral-small-latest` | `33342547879` | 140/140, 5/5 complete | 1.000 | 1.000 | 1.000 | 0.000 | 0.050 | non-conformant |
| `gemini-3.1-flash-lite` | `33342334655` | 140/140, 5/5 complete | 1.000 | 1.000 | 0.821 | 0.417 | 0.000 | non-conformant |
| `ministral-14b-latest` | `33342335857` | 140/140, 5/5 complete | 0.800 | 1.000 | 0.786 | 0.500 | 0.543 | non-conformant |
| `nvidia/nemotron-3.5-lightning-30b-a3b` | `33342337031` | 71/140 success, 69 protocol failures, 0/5 complete | n/a | n/a | n/a | n/a | 1.000 on successes | non-conformant |

adoption gate は decisive に fail した。conformant model は0、usable-with-limitations model も0であり、successor は採用しない。

### 失敗した後継版で確認できたこと

- three-way semantics の簡素化は、model 固有の interpretation noise だけを減らすのではなく、provider family をまたぐ uncertainty calibration を弱めた。
- Mistral Small の v3 abstention collapse は kind をまたいで変わらず、8B/Gemini/14B も intentionally ambiguous な unsupported-premise と causal-scope case で assertive になりすぎた。
- stricter discriminated schema により Ministral 14B の実際の protocol property は改善した。v3 の repeated non-finding-plus-finding violation は消え、140 call すべてが protocol-valid になった。ただしこれは protocol result であり、簡素化した semantic wording の evidence ではない。
- Nemotron では truncation confound は除去されたが、69 protocol failure と成功 call 全件での `finding` という重大な incompatibility が残った。
- 一部の Mistral failure は structured note が conflict ではなく agreement を意味的に記述していても decision mapping を誤った。従って問題は model knowledge や task comprehension だけには還元できない。

### ランタイム上の判断

Issue #55 により、従来特徴づけ済みの `soft-semantic-v3` model request/schema behavior を runtime baseline に戻す。v4 commit、holdout-v3 fixture、run ID、この result は immutable research history として残す。観測済み holdout-v3 case から v4 wording を tuning してはならない。

将来の successor は semantic wording と protocol/schema experiment を calibration corpus 上で分離し、material な configuration change の provider measurement 前に新しい holdout-v4 を凍結しなければならない。
