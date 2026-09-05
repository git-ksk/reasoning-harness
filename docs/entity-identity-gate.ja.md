# エンティティ同一性ゲート

## 状態

Issue #193、#195、#196 で行った MCP 外部情報／entity identity 研究は、trusted-context candidate v12 の採用可能性を確認するところまで完了した。凍結した candidate semantics は `d1db067e6efe6033656b8e7c3315a9fe322c015d` である。独立して凍結した Issue #196 の one-shot holdout は、最初の1回だけの観測で 16/16 に合格し、semantic false decision、context-unverified fact admission、planner/tool/operational/budget failure はすべて 0 だった。

この結果は historical evidence であり、holdout は再実行・再利用・tuning しない。Issue #197 の adoption branch は holdout evaluator ではなく、凍結 candidate semantics そのものから分岐している。

## authority boundary

model / planner は untrusted である。planner が選べるのは Harness が公開した bounded next action だけで、truth、entity identity sufficiency、evidence admission、terminal correctness は決められない。

Harness が所有する境界は次のとおり。

- candidate list は plausibility evidence にすぎず、fact admission の authority ではない。
- no-context の entity fact は、non-disambiguation の Wikipedia top と resolved Wikibase item が一致し、cross-source corroboration rank が 1 の場合だけ admission できる。
- trusted identity context がある場合、bare query の rank-1 agreement だけでは admission しない。Harness が `surface, context` の canonical query を作り、planner は `follow_suggested_query` または `stop` だけを選ぶ。
- trusted-context admission には canonical query の実行に加えて、Wikidata label/description または Wikipedia top title による deterministic context compatibility が必要である。
- trusted-context coordinate では、search recall が足りない場合に限り、non-disambiguation Wikipedia top の exact Wikibase QID を Wikidata へ直接照合できる。ただし direct-QID bridge 自体は authority ではなく、同じ context compatibility gate を通過する必要がある。
- no-context query では direct-QID bridge を使用せず、rank-1 rule を緩めない。
- adapter が返す suggested query は observation にすぎない。実行可能なのは `harness_trusted_identity_context` origin を持つ Harness-owned suggestion だけである。
- malformed、unavailable、duplicate planner action は external HTTP より前に reject する。
- ambiguity を解消する trusted context がなければ NIL / `unknown` を保持する。

## stable adoption materialization

Issue #197 では research transformer chain を通常実行時に再適用せず、v12 を `crates/reasoning-harness-cli/src/bin/mcp_identity_gate_benchmark.rs` に一度だけ materialize した。凍結した materialized identity は次のとおり。

- candidate semantics: `d1db067e6efe6033656b8e7c3315a9fe322c015d`
- materialized benchmark SHA-256: `d36bc7e0df7e423c96a8a7a3b7aa7846471d985e6dfffca9928b398e5557ff9f`
- adapter SHA-256: `c8fb8f76a4e4abcd5b89161d98508e0146f9d329e8831073cd0f69530e8e7098`

`fixtures/mcp-identity-context-v12-adoption.sha256` と adoption CI はこの identity を直接検証する。research transformer は provenance と再現性のために残すが、adopted source の通常 test path は transformer を実行しない。

## 変更規律

identity sufficiency、query policy、planner authority、evidence admission、budget、stop rule、terminal safety を変更する場合、それは adoption cleanup ではなく新しい semantic research である。新しい issue、fresh dev split、新しい独立 holdout を用意しなければならない。

#193、#195、#196 の frozen holdout はいずれも historical evidence のまま保持し、今後の tuning material にはしない。
