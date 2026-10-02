# Engine 0.6 evidence relevance holdout v9

Status: runner-freeze preparation only。successor-v9 semantics は `engine-0.6-evidence-relevance-successor-v9-semantics-freeze` で frozen。holdout-v9 corpus はまだ author / observe していない。

## Frozen semantic input

- successor semantics tag: `engine-0.6-evidence-relevance-successor-v9-semantics-freeze`
- tag target: `3e49a9fd2f7827b7c707516d2150e2f0f16e9e17`
- effective qualification: v11
- materialization: v23
- historical v9/v22 predecessor functionsは frozen / unchanged

## Runner wiring

dedicated binary は `crates/reasoning-harness-cli/src/bin/evidence_relevance_holdout_v9.rs` の `reason-evidence-relevance-holdout-v9-study`。

corpus authoring 前に V9 profile を以下で固定する。

- configuration: `evidence-relevance-live-holdout-v9`
- suite: `evidence-relevance-holdout-v9`
- annotation protocol: `evidence-relevance-effective-qualification-v11`
- fixed core: `evidence-relevance-fixed-core-v9`
- expected relative directory: `fixtures/evidence-relevance-holdout-v9`
- expected case count: 26
- checkpoint profile: holdout / complete 時のみ scorable
- effective qualification: v11
- materialization: v23

runner completeness test は V9 を含む全 profile を coverage する。runner-freeze preparation 時点では `fixtures/evidence-relevance-holdout-v9` directory 自体を作成しないため、fresh corpus が runner wiring に影響することはない。

## Runner freeze

最初の runner freeze `engine-0.6-evidence-relevance-holdout-v9-runner-freeze` / `17ea49bc6b332bb985971b661830c80665ad5dd4` は、provider 観測前の validate-only 監査で v8 由来の `issue == 462` binding を引き継いでいることが判明した。tag は immutable evidence として保持し、移動・削除しない。この v1 runner では #468 corpus を受理できないため acceptance runner としては使用しない。

修正版は V9 のみ `issue == 468`、historical V1-V8 / development profiles は `issue == 462` を明示する。新しい freeze coordinate は `engine-0.6-evidence-relevance-holdout-v9-runner-freeze-v2`。

この v2 annotated tag を push した後にのみ holdout-v9 corpus authoring を開始する。corpus は holdout v1-v8 と successor-v5/v6/v7/v8/v9 development surface に対して case ID / canonical entity / task / exact signal / exact 8-token candidate-signal n-gram overlap 0 の fresh independent surface とする。corpus 自体を freeze するまでは provider observation を禁止する。
