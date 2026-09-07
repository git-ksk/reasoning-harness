# Natural-language E2E v11 — trigger 条件付き v0.4.1 successor

Issue #254 では、frozen v10 の Mistral 観測で v0.4.1 / Issue #249 continuation の手前にある planner trigger miss が見つかったため、fresh な full-product successor として `natural-language-e2e-v11` を定義する。

## historical boundary

v9 / v10 は immutable な historical observation として保持し、rerun、rescore、fixture repair、tuning data 化をしない。

- v9: 専用 Umber lane は `cache -> typed no_result` まで到達したが registry へ continuation できなかった。
- v10 canonical Mistral run `34125135760`: 11/11 case 完了、correctness-boundary violation 0、operational failure 0。ただし専用 Cobalt lane は target を recall した一方で action を1回も実行せず、`planner_calls=4`, `actions=0`, `round_budget` で終了した。そのため #249 trigger は未到達で、live mechanism effect は inconclusive だった。

評価対象 product は次の exact released coordinate のまま固定する。

- tag: `v0.4.1`
- commit: `29a9e4be6273dbffeda324e15517dc64930ad315`
- natural output contract: `reason-natural-output-v4`
- MCP adapter: `mcp_readonly_v3`

v11 のために production semantics は変更しない。

## controlled live intervention lane を作らない理由

released `reason` CLI には investigation action や pre-populated `InvestigationState` を外部から注入する supported input がない。measurement 専用 injector を追加すると exact released-v0.4.1 product observation ではなくなり、すでに #249 closeout で使った deterministic unit regression の言い換えになりやすい。

そのため v11 は observational のまま、独立した fresh trigger case を複数 freeze して測る。

## corpus

従来どおり whole-product E2E family を維持し、follow-up surface だけ複数化する。

- mechanically unique な grounded acquisition 1件
- ambiguous tool-selection 1件
- stale evidence rejection 1件
- scope rejection 1件
- fresh observational `cache -> no_result -> registry` follow-up 3件
- authority rejection 1件
- source-identity rejection 1件
- pinned read-only GitHub MCP generic-content non-promotion 1件
- session add / correct / resume-fork 3件

合計13件。investigation 10件 + session 3件。

case ID、task string、target key、fresh marker、config source ref、base seed は observed predecessor v1-v10 と mechanically disjoint にする。

## follow-up を3軸に分離する

3つの follow-up case を1個のbinary gateとして扱わない。

### 1. trigger reachability / planner utility

分母は freeze 済み follow-up 3件すべて。

最初に実行された relevant capability が configured cache で、その action が typed `no_result` を返した場合だけ `trigger_exposed=true` とする。

`trigger_reachability_rate = trigger_exposed_cases / 3`。

trigger miss は planner utility の観測値であり、#249 mechanism failure ではない。また observation 自体を non-scorable にしない。

### 2. conditional #249 mechanism conformance

分母は trigger-exposed case のみ。

trigger 後の直後の action が configured exact-target registry で、`harness_no_result_followup_selections == 1` により Harness-owned continuation が観測された場合、mechanism-conformant とする。

total `planner_calls == 1` は要求しない。cache 前の invalid stochastic attempt は trigger reachability 側のデータであり、registry 後の downstream work も post-`no_result` selector が planner を呼んだかどうかと混同しないためである。

trigger-exposed が0件なら mechanism classification は `inconclusive`。success / failure にしない。1件以上なら exact numerator / denominator を descriptive に報告する。

### 3. downstream utility

registry を Harness が選んだだけでは utility success としない。registry が admitted evidence / verification progress を生んだか、最終 target が grounded になったかを別々に記録する。

## measurement-validity semantics

Issue #247 が要求した分離を v11 で適用する。

- **measurement observability**: frozen study が実行され、全 case report が生成されたか
- **path exposure**: MCP / no-result trigger が実際に exercise されたか
- **hard correctness**: unsupported exposed/structured claim、missed insufficiency、identity unsafe admission など authority-boundary failure
- **operational completeness**: process / typed-action / generation failure
- **planner utility**: target recall、tool selection、trigger reachability、false abstention
- **conditional mechanism conformance**: trigger-exposed case のみ
- **downstream utility**: useful evidence / verification と grounded recovery

utility success、MCP exposure、trigger reachability、#249 mechanism success は `measurement_validity_passed` の前提にしない。これらは測定対象の product outcome である。

一方 hard correctness または operational completeness が崩れた場合は live workflow の report gate を fail させ、artifact を保存したまま rerun しない。

## frozen provider policy

- provider: `mistral`
- model: `ministral-8b-latest`
- base seed: `57000`
- max tokens: `1024`
- inter-case delay: `1500 ms`

最初の live-case launch が canonical。live-case launch 前の純粋な infrastructure failure だけ retry 可。live boundary に入った後は v11 identity で rerun / tuning しない。

## pre-observation status

v11 の live observation はまだ実施していない。corpus、evaluator、workflow、tests、checksums、documentation を credentials 利用前に freeze する。
