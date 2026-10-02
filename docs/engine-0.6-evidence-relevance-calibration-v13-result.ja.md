# Engine 0.6 evidence-target relevance calibration v13 — immutable result

状態: FAIL。canonical result は不変。v13 は rerun / rescore / relabel / retag しない。

Freeze:
- commit: dc8ed61031a28bc156d4fa34dac0e2a9f87115fe
- tag: engine-0.6-evidence-relevance-calibration-v13-freeze
- canonical run: 36154375809
- run attempt: 1
- final gate: failure

## Required Mistral arm

Model: ministral-8b-latest。

Operational:
- 81/81 完走
- successful provider cases 81
- provider failures 0
- provider-attempt telemetry complete

Semantic / materialization:
- proposal exact 42/81 (51.85%)
- local qualification exact 28/81 (34.57%)
- blocking-cue miss 7
- spurious cue block 5
- materialized exact 61/81 (75.31%)
- wrong-target / false Relevant retention 1
- false relevance rejection 2
- expected Relevant left Ambiguous 6
- utility miss 19

v12 に対して safety regression。binary observable cue 化で一部の過剰 abstention は減ったが、必要な ambiguity まで落ち、non-Relevant case 1件を Relevant に materialize した。

v13 FAIL 後に semantic coverage で選定した fixed 48-case successor core では Mistral は 32/48 exact (66.67%)。wrong Relevant 1、false reject 2、Relevant -> Ambiguous 4、expected blocking-cue miss 5、spurious cue block 4。case を間引いても failure を隠していない。

## Required Groq arm

Model: openai/gpt-oss-120b。

canonical arm は operationally incomplete:
- completed 3/81
- successful 1
- failed 2
- case 03 後に circuit open、remaining 78
- 観測済みcaseの provider-attempt telemetry は complete

v13 の transport 変更で JsonSchema -> Text double-request failure は除去できた。今回の failure は strict-JSON Text local qualification の truncation:
- case 02: JSON string 途中で EOF、finish_reason=length
- case 03: model text なし、finish_reason=length、completion_tokens=192

つまり Groq gpt-oss-120b strict-Text path に対して local qualification の192-token completion budget が小さすぎる。successor diagnostic では Groq transport-only budget を512へ上げ、consecutive-failure circuitを無効化して全caseをattemptする。この diagnostic は noncanonical であり v13 result を変更しない。

## Google replication

Model: gemini-3.5-flash-lite。

Operational:
- 81/81 完走
- successful provider cases 81
- provider failures 0
- provider-attempt telemetry complete

Semantic / materialization:
- proposal exact 57/81 (70.37%)
- local qualification exact 38/81 (46.91%)
- blocking-cue miss 3
- spurious cue block 4
- materialized exact 65/81 (80.25%)
- wrong-target relevance retention 0
- false relevance rejection 0
- expected Relevant left Ambiguous 3
- utility miss 16

Google でも Mistral より exactness は高いが、v13 semantic contract の over-abstention と required blocking-cue miss が独立に再現した。

fixed 48-case successor core では Google は 37/48 exact (77.08%)、wrong Relevant 0、false reject 0、Relevant -> Ambiguous 2。

## Cross-model residual

fixed core で Mistral / Google 両方が miss した5件:
- 04_semantic_paraphrase
- 15_navigation_only_match
- 16_broad_landing_no_support
- 21_unknown_rename
- 32_separate_product_table

provider固有ノイズではなく model-independent design residual として扱う。

主な問題は、model-facing guard が Harness-owned fact を再判定できてしまうことと、6-field qualification task が target support / relation support / explicit absence / blocking cue を相互干渉させていること:
- Harness-owned canonical / alias anchor を model target_support が否定できる;
- uncertain rename / successor mapping を different / not_supported に harden して ambiguity を失う;
- navigation / generic page を ordinary negative evidence ではなく context-gap blocker にしやすい;
- explicit distinct structured evidence で identity-mapping cue を誤発火する;
- freshness / truth concern が target-support 判定へ漏れることがある。

## Final decision

required top-level gate はすべて false:
- operational completeness: false
- correctness: false
- utility: false
- materialization: false
- qualification safety: false

v13 は immutable canonical FAIL。independent holdout authoring は引き続き禁止。

## Successor evaluation hygiene

calibration は fresh slice 追加により 65 -> 73 -> 81 件へ増えていた。この増加はここで止める。

fixed successor calibration core:
- identity: evidence-relevance-fixed-core-v1
- 48 cases
- Relevant 14 / Irrelevant 18 / Ambiguous 16
- policy: fixed_no_new_cases
- identity mapping / ownership scope / context gap / explicit local absence / target-relation orthogonality / alias / cross-language / URL-only / injection / freshness / relation mismatch を維持

live observation 前なら successor contract / annotation は変更できるが、model miss を理由に case を in-place 追加しない。本当に新しい semantic dimension が必要な場合だけ明示的な new core version を作る。

別IDの noncanonical Groq postmortem で v13 operational failure distribution を解析してよいが、immutable v13 score は変更しない。
