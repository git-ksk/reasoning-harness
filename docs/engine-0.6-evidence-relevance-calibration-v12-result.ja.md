# Engine 0.6 evidence-target relevance calibration v12 — immutable result

状態: FAIL。この結果は不変とする。v12 は再実行・再採点・再ラベル・再タグ付けしない。

Freeze:
- candidate/tag commit: 9534ae382640c154790abf1cc07c57d1065519b6
- tag: engine-0.6-evidence-relevance-calibration-v12-freeze
- canonical GitHub Actions run: 36148322898
- run attempt: 1
- final-gate conclusion: failure

## Required Mistral arm

Model: ministral-8b-latest。

Operationally complete:
- successful provider cases: 73/73
- provider failures: 0
- qualification invoked: 73/73
- provider-attempt telemetry: complete

Safety:
- qualification risk misses: 0
- wrong-target relevance retention: 0
- false relevance rejections: 0

Utility/materialization は未達:
- proposal exact: 35/73 (47.95%)
- local qualification exact: 28/73 (38.36%)
- qualification spurious risk blocks: 15
- materialized exact: 55/73 (75.34%)
- expected Relevant left Ambiguous: 1
- utility misses: 18

ケース単位では disposition miss 18 件中 17 件が expected Irrelevant -> Ambiguous、1 件が expected Relevant -> Ambiguous。18 件中 16 件で relation_binding が frozen expectation と不一致だった。残課題は risk overblocking だけではなく、target identity / relation kind / blocking-risk cue がまだ十分に直交していないこと。

また静的 specification audit で、v12 の frozen label 自体にも protocol 不整合を確認した。例として case 28/29 は candidate が「product-specific / target-specific information がない」と明示している一方、expected explicit_local_absence は absent になっている。v12 prompt はこの種の局所的明示を present と定義している。v12 の score は不変のまま保持し、successor では observation 前に annotation protocol を固定する。

## Required Groq arm

Model: openai/gpt-oss-120b。

Operationally incomplete / non-scorable:
- planned/completed: 73/3
- successful provider cases: 1
- failed provider cases: 2
- 03_expanded_alias 後に consecutive operational failure 2 件で abort
- remaining cases: 70
- failed 2 件は provider-attempt telemetry incomplete
- latency p50/p95/max: 60,000 / 60,001 / 60,001 ms

v12 で v11 の schema-generation retry loop は除去できたが、別の pacing failure が露出した。structured request は HTTP 400 でも provider capacity を消費し得る一方、ローカル token pacer は成功レスポンスの usage しか記録しない。その直後の strict Text fallback が HTTP 429 を受け、retry-after が shared 60-second case deadline を超えたため recovery 前に assessment timeout となった。

これは transport の operational issue であり、semantic gate を緩める理由にはしない。

## Google replication

Model: gemini-3.5-flash-lite。non-gating。

Operationally incomplete:
- planned/completed: 73/37
- successful provider cases: 35
- failed provider cases: 2
- 37_fresh_sibling_overlap 後に consecutive failure 2 件で abort
- remaining cases: 36

abort 前の diagnostic:
- proposal exact: 23/35 (65.71%)
- local qualification exact: 14/37 (37.84%)
- qualification risk misses: 1
- qualification spurious risk blocks: 5
- materialized exact: 27/35 (77.14%)
- wrong-target relevance retention: 1
- expected Relevant left Ambiguous: 1
- utility misses: 7

未完走のため score は diagnostic のみ。

## Final decision

required top-level gate はすべて false:
- operational completeness: false
- correctness: false
- utility: false
- materialization: false
- qualification: false

v12 は immutable canonical FAIL。再実行・再採点・再ラベル・replacement tag・fixture の後付け修正は禁止。independent holdout authoring は引き続き禁止。

## v13 successor requirements

1. **Annotation/protocol consistency**: model field を atomic proposition として定義し、expected label を observation 前の reviewable rule から生成する。v12 の historical labels は変更しない。
2. **Orthogonal semantics**: target identity と relation kind を独立判定する。別 sibling target でも requested relation 自体は exact になり得る。
3. **Observable blocking cues**: materializer v7 では Present / Unresolved が同じ blocking consequence を持つため、successor では「具体的な local blocking cue が存在するか」を主に判定し、generic open-world uncertainty を blocker にしない。
4. **Groq transport pacing**: usage telemetry がない structured-output failure の直後に 60 秒 budget 内で fallback を連打しない。provider-specific format/pacing を precommit する。

v13 design の研究アンカー:
- Weir et al., EMNLP 2024, *Enhancing Systematic Decompositional Natural Language Inference Using Informal Logic*
- Chen et al., Findings ACL 2023, *PropSegmEnt*
- Srinivasan et al., Findings ACL 2024, *Selective “Selective Prediction”*
- Xu et al., Findings ACL 2025, *Do Language Models Mirror Human Confidence?*
- Geng et al., 2025, *Generating Structured Outputs from Language Models: Benchmark and Studies*
