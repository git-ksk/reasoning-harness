# Engine 0.6 evidence relevance independent holdout v9 result

Status: immutable canonical FAIL。

- Freeze tag: `engine-0.6-evidence-relevance-holdout-v9-freeze`
- Freeze commit: `1b50e5c54cdff4bd850bb0b1c58cfc66ce46d8bf`
- Canonical run: `37037497486` attempt 1 only
- Cases: 26 (Relevant 8 / Irrelevant 10 / Ambiguous 8)
- Required providers: Mistral + Google + Groq
- Effective qualification: v11
- Materialization: v23
- Final gate: FAIL

この結果は immutable。rerun / rescore / relabel / freeze tag の移動・再作成 / PASS への読み替えは禁止する。

## Provider results

Mistral / `ministral-8b-latest`:
- operational 26/26、provider failure 0
- effective authority qualification exact 24/26
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- relevant left Ambiguous 0
- utility miss 1
- relation-scope miss: `v9h18_negative_distinct_availability_lookalike`、`v9h26_ambiguous_exact_target_numeric_observation`
- terminal materialization miss: `v9h26` expected Ambiguous -> Irrelevant

Groq / `openai/gpt-oss-120b`:
- operational 26/26、provider failure 0
- effective authority qualification exact 24/26
- materialized exact 25/26
- wrong-target Relevant 0
- utility miss 1
- Mistral と同じ2件の relation-scope miss、同じ `v9h26` materialization miss

Google / `gemini-3.5-flash-lite`:
- semantic observation 25/26
- operational failure 1件: `v9h07_positive_alias_availability`
- failure class: timeout。local qualification が 60,000 ms semantic execution budget を超過
- effective authority qualification は24 recorded match、semantic relation miss 1件 + derivation欠落1件
- materialized exact 24/25 successful provider cases
- wrong-target Relevant 0
- `v9h26` は expected Ambiguous に対し Irrelevant

Google timeout は operational evidence のみで、semantic evidence には使わない。

## Contract-adjudication finding

semantic gate FAIL を理由に frozen v11/v23 を変更しない。

holdout-v9 の2つの expectation が、holdout authoring 前に既に freeze されていた relation contract から drift していた。

1. `v9h18_negative_distinct_availability_lookalike` は `unresolved` を期待したが、pre-freeze successor-v9 development の同型 `sv9d_10_availability_feature_support_non_frame` は `different_relation` / Irrelevant を期待していた。

2. `v9h26_ambiguous_exact_target_numeric_observation` は `unresolved` / Ambiguous を期待したが、pre-freeze successor-v9 development の exact-target 同型 `sv9d_19_exact_target_model_only_limit` は `different_relation` / Irrelevant を固定していた。関連する `sv9d_07_limit_numeric_observation` も `different_relation` を期待していた。

`v9h26` で semantic observation を得た3 provider は全て frozen-contract behavior を返した。Mistral / Groq は `v9h18` でも frozen-contract behavior。Google は同件で v11 により abstain 済み。

canonical result 後に negative relation authority を全面対称化する v12/v24 candidate をローカル検証したが、過去 holdout / development の正当な `different_relation` を多数 regression させたため不採用・未commit。frozen v11/v23 は byte-for-byte unchanged。

## Final gate interpretation

公式 canonical result:
- operational completeness: Google timeout により FAIL
- correctness hard gate: required arm incomplete のため canonical workflow 上 FAIL
- qualification: FAIL
- materialization: FAIL
- utility: FAIL

研究上の解釈:
- completed arm で wrong-target Relevant 0
- 反復した semantic miss は v11/v23 defect ではなく、frozen pre-holdout contract に対する holdout expectation drift
- Google timeout は独立した operational failure
- 次は semantic retune ではなく unchanged v11/v23 で fresh evaluation generation を行う

## Successor evaluation direction

次の independent acceptance surface は unchanged frozen v11/v23 を使う holdout v10 とする。

corpus authoring 前に:
- existing successor-v9 semantics tag に固定した dedicated holdout-v10 runner を freeze
- non-frame control が `different_relation` から `unresolved` へ勝手に変わらないよう、holdout relation expectation と frozen development contract の authoring review を追加
- semantic failure と operational failure を分離
- Google timeout は operational robustness input として扱い、semantic label signal には使わない
- holdout v1-v9 と全 prior development surface に対する case/entity/task/signal/8-token overlap 0 を維持

Canonical holdout v9 は immutable FAIL のまま、evaluation-contract gap の証拠として保持する。
