# Engine 0.6 evidence relevance holdout v8

状態: runner freeze準備のみ。successor-v7 semanticsは `engine-0.6-evidence-relevance-successor-v7-semantics-freeze` でfreeze済み。holdout-v8 corpusはまだauthorもprovider観測もしていない。

## Frozen semantic input

- successor semantics tag: `engine-0.6-evidence-relevance-successor-v7-semantics-freeze`
- tag target: `3a508bfba20386218436581dcbb69224b54948b3`
- effective qualification: v9
- materialization: v22
- historical v8/v21 predecessor functionはfrozenのまま不変

## Runner wiring

専用binaryは `crates/reasoning-harness-cli/src/bin/evidence_relevance_holdout_v8.rs` の `reason-evidence-relevance-holdout-v8-study`。

corpus authoring前にV8 profileを次で固定する。

- configuration: `evidence-relevance-live-holdout-v8`
- suite: `evidence-relevance-holdout-v8`
- annotation protocol: `evidence-relevance-effective-qualification-v9`
- fixed core: `evidence-relevance-fixed-core-v8`
- expected relative directory: `fixtures/evidence-relevance-holdout-v8`
- expected case count: 26
- checkpoint profile: holdout。complete時のみscorable

runner completeness testはV8を含む全declared profileをcoverする。runner freeze準備時点ではholdout-v8 directory自体が存在しないため、corpusがrunner wiringへ影響する余地はない。

## Runner freeze

freeze coordinate: `engine-0.6-evidence-relevance-holdout-v8-runner-freeze`。

このannotated tagをpushした後にのみholdout-v8 corpus authoringを開始する。corpusは独立authorし、holdout v1-v7およびsuccessor-v5/v6/v7の全development surfaceに対してcase ID / canonical entity / task / exact signal / exact 8-token signal n-gram overlapを0にする。corpus自体をfreezeするまでprovider観測は禁止する。
