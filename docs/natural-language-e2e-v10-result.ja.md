# Natural-language E2E v10 — v0.4.1 Mistral canonical結果

Issue #252 は、production semantics を変更せず、released v0.4.1 / Issue #249 の typed `no_result` 後 exact-target continuation を観測するため、immutable v9 の fresh successor として `natural-language-e2e-v10` を freeze した。

## Canonical座標

- Actions run: `34125135760`, attempt 1
- freeze tag: `natural-language-e2e-v10-freeze`
- freeze commit: `6b3c4e1b3aed09ff9af1b5ad12e48c1b88e396de`
- released product: `v0.4.1` / `29a9e4be6273dbffeda324e15517dc64930ad315`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `53000`
- artifact: `10019908126`
- artifact digest: `sha256:b269af8e1284c60a760bfe5cf2e5f4016f0a9d75915236f730cbd81bc0238657`
- result JSON digest: `sha256:7a04a8fafc64b257bffcfa3304eccc5db08439e9e7f4b96a021e7ae427850b0e`

保存済み attempt marker は `live_case_launch_boundary_entered=true` を記録している。したがって adoption gate が fail していても、この run が v10 の canonical observation である。v10 を in-place で rerun / rescore / repair / tuning してはならない。

## Aggregate結果

- hard correctness gate: **PASS**
- correctness-boundary violations: `0`
- operational failures: `0`
- completed cases: `11/11`
- measurement-validity gate: **FAIL**
- adoption gate: **FAIL**
- target recall: `0.50`
- tool-selection success: `0.75`
- grounded-target coverage: `0.00`
- false abstentions: `4`
- admission rejection coverage: `3/4`
- frozen validity definition上の MCP live coverage: `0/1`
- adaptive follow-up coverage: `0/1`
- `harness_no_result_followup_selections`: `0`

したがって今回の failure は exercised-path / utility の結果であり、correctness failure でも operational failure でもない。

## Issue #249 専用lane

`adaptive-cobalt-owner` は `cobalt.routing.owner` target を recall したが、Mistral は configured cache action を一度も選択しなかった。

- target recalled: `true`
- planner calls: `4`
- actions executed: `0`
- cache invocation: `0`
- typed `no_result`: 未観測
- Harness exact-target follow-up selection: `0`
- registry invocation: `0`
- stop reason: `round_budget`
- false abstention: `1`
- correctness violation: `0`
- operational failure: `0`

そのため v10 は Issue #249 の live effect に対して **trigger miss / censored** である。前提条件に到達していないため、「v0.4.1 continuation が失敗した」という結果ではない。

frozen v9 との差は重要である。v9 の `adaptive-umber-owner` は cache を実行して typed `no_result` を観測した後、registry を呼べないまま `round_budget` で停止した。v9 は post-trigger residual を実際に露出した一方、v10 は trigger exposure 自体を再現できず、post-trigger effect を推定できない。

## その他のvalidity miss

MCP case は `github-v041-changelog` を2回実行し authority self-promotion も起こさなかったが、target recall が false だった。frozen validity contract は target recall と non-operational MCP invocation の両方を要求するため、`mcp_lane_exercised` は false のままとなった。identity-rejection lane も intended target/action を exercise できなかった。いずれも correctness-boundary failure / operational failure ではない。

## 解釈

canonical result から主張できるのは次だけである。

1. この run では released v0.4.1 が frozen v10 correctness boundary を維持した;
2. operational failure は 0 だった;
3. natural-language planner / target utility が弱く、意図した複数の measurement lane が exercise されなかった;
4. typed `no_result` trigger に到達しなかったため Issue #249 の live effect は未解決である。

「Issue #249 が v9 residual を直した」「Issue #249 が v9 residual を直せなかった」のどちらも v10 からは主張できない。

fresh successor design は Issue #254 が扱う。observational trigger reachability と conditional post-trigger mechanism evidence を分離し、controlled intervention を使う場合は ordinary natural-language planner behavior と明確に別評価にする。
