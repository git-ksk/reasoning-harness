# Engine 0.6 evidence relevance successor-v11 development v2

Status: successor-v11 development-v1 immutable FAIL 後の pre-observation fresh development candidate。

Historical boundary:
- holdout-v11 canonical FAIL: run 37182677114 / 4343e7939e964e91d466a0788358733d629e92f2
- development-v1 freeze: engine-0.6-evidence-relevance-successor-v11-development-v1-freeze / b97bf46f5e0badf434e5fccd8e3581fc8e77450f
- development-v1 run: 37212349800、immutable FAIL
- development-v1 result record: 861730fc0e46b96106e5001bfe9f546e03cd1525
- rerun / rescore / relabel / freeze-tag move は禁止

## v2 の目的

development-v1 では元の holdout-v11 generic-no-relation defect は修復できた。Mistral / Google とも22/22完走、authority failure 0、materialization exact 22/22、wrong-target Relevant 0、utility miss 0。

一方、両providerが `sv11d_19_risk_clipped_column` で同一の identity-axis miss を露出した。Harness target 名は文中にあるが、product column がclipされ ownership が表示されていないと明示されているため、`exact_target` authority は成立しない。frozen expectation は `unresolved`。`context_gap` と final Ambiguous は維持されていた。

したがって v2 は v16 の relation-authority 修正とは独立した ownership/identity floor の追加に限定する。

## Candidate semantics

Effective qualification v17 は v16 を継承し、effective identity が `exact_target`、scope risk が `context_gap` または `multiple`、かつ bounded candidate が product/owner column、ownership field、row owner、referent など ownership を決める情報の omit / clip / outside / missing / not shown を明示する場合だけ `exact_target -> unresolved` に降格する。

このルールは一方向のみで、`distinct_target` / `target_absent` を作らない。scope risk を解除・弱化せず、independent relation axis も変更せず、requested/different relation authority を新規生成しない。

単なる relation text の truncation で target ownership 自体が見えているケースは `exact_target` を維持する。historical replay でこの反例も固定する。

Materialization v30 は v29 full composition を維持し、v17 effective state に proposal identity/relation binding を同期して frozen v23 に渡す。stale model Exact target binding が削除済み ownership authority を再導入できない。

## Immutable replay

development-v1 live observation の v17/v30 replay で effective identity が変わるのは次の2件だけ:
- Mistral `sv11d_19_risk_clipped_column`: `exact_target -> unresolved`
- Google `sv11d_19_risk_clipped_column`: `exact_target -> unresolved`

relation scope / scope risk は不変。development-v1 の terminal disposition は1件も変更せず、両方とも Ambiguous のまま。

holdout-v10、successor-v10 development v2/v3/v4、holdout-v11、successor-v11 development-v1 の recent immutable provider observation は v30 でも frozen terminal contract を維持する。

## Fresh development-v2 surface

Suite: `evidence-relevance-successor-v11-development-v2`
Cases: 22
Protocol: effective qualification v17 + materialization v30
Fixed core: `evidence-relevance-fixed-core-successor-v11-development-v2`

内訳:
- preserve_risk 10: explicit omitted-ownership identity gap 6 + exact identity を維持すべき relation-only context gap 4
- forbid_different 4
- require_different 4
- require_requested 3
- preserve_absence 1
- final Ambiguous 14 / Irrelevant 6 / Relevant 2

observed holdout-v1-v11 / successor-v5-v11 development-v1 に対し case ID / canonical entity / task / exact signal / 8-token signal window の再利用を禁止する。

property control では preserve-risk 全ケースの deterministic risk 一致、ownership gap の adversarial ExactTarget vote 降格、relation-only context gap の ExactTarget 維持、generic control の model-only DifferentRelation 抑止、true alternate relation の conservative model output 下での回復を要求する。

## Provider policy

development-v2 observation は Mistral + Google のみ。Groq は candidate shaping から除外し、successor semantics freeze 後の fresh independent holdout でのみ復帰可能。

Intended development-v2 freeze coordinate: `engine-0.6-evidence-relevance-successor-v11-development-v2-freeze`。

exact-head CI PASS と annotated development-v2 freeze tag の一度だけの push 前に provider observation を開始してはならない。
