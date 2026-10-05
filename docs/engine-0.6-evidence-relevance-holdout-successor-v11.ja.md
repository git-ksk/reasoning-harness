# Engine 0.6 evidence relevance holdout successor v11

Status: immutable holdout-v11 FAIL、development-v1 FAIL、development-v2 PASS 後の pre-freeze successor。candidate semantics は effective qualification v17 + materialization v30。successor-v11 semantics-freeze tag 作成前の fresh acceptance-holdout authoring を禁止する。

## Historical boundary

Canonical holdout-v11 run 37182677114 は `engine-0.6-evidence-relevance-holdout-v11-freeze` / `4343e7939e964e91d466a0788358733d629e92f2` の immutable FAIL のまま維持する。Mistral / Google は PASS、Groq のみ `v11h21_ambiguous_generic_no_relation` で、classifiable alternate relation のない generic target-owned documentation を relation_absent/Ambiguous から different_relation/Irrelevant に harden した。rerun / rescore / relabel / tag movement は禁止する。

Development-v1 run 37212349800 は `engine-0.6-evidence-relevance-successor-v11-development-v1-freeze` / `b97bf46f5e0badf434e5fccd8e3581fc8e77450f` の immutable FAIL。元の negative-relation defect は修復したが、explicit omitted ownership に Harness-owned identity boundary を露出した。

Development-v2 run 37215152913 は `engine-0.6-evidence-relevance-successor-v11-development-v2-freeze` / `db38a7eb525a07208bfeb467cfa4d9d8a8a59f51` の immutable PASS で、`1bc44a16c601d285a8e383c9ad4b66da8c4059df` に結果記録済み。これは development evidence のみで `holdout_acceptance_evidence=false`。

## Frozen candidate semantics

Effective qualification v17 は v16 symmetric negative-authority hardening を維持する。Harness-owned affirmative alternate-relation evidence がない exact target に対し、model output だけで DifferentRelation を生成できない。

さらに v17 は、既に ContextGap / Multiple risk が型付けされ、owner/product column、ownership、row owner、referent が omitted / clipped / missing / hidden 等で利用不能と明示された local evidence に限り、one-sided ownership/identity floor を適用する。この限定状態では ExactTarget を Unresolved に降格する。DistinctTarget / TargetAbsent は生成せず、blocking scope risk は解除せず、relation authority も生成しない。

Materialization v30 は v29/v23 composition を維持しつつ、advisory proposal の target/relation binding を v17 effective state に同期する。これにより stale model Exact binding が v17 で除去した target ownership を復活させることを防ぐ。

## Development-v2 evidence

fresh 22-case development-v2 surface は provider observation 前に author され、predeclared historical surface に対する case ID / canonical entity / task / exact-signal / 8-token-window reuse を除外している。

One-shot run 37215152913 は Mistral `ministral-8b-latest` + Google `gemini-3.5-flash-lite` のみを使用。両providerとも22/22完走、provider / authority / identity-risk / materialization failure 0、final materialization exact 22/22、wrong-target Relevant 0、utility miss 0。Groq は未観測で fresh independent acceptance まで温存する。

v1 omitted-ownership defect は修復され、paired relation-only context-gap control に広い identity demotion は発生していない。保守的な relation-label 差分は non-terminal で frozen authority contract を満たす。

## Pre-freeze audit

Development-evidence commit `1bc44a16c601d285a8e383c9ad4b66da8c4059df` 時点で:
- exact-head GitHub CI 8/8 PASS;
- branch HEAD == origin、working tree clean;
- development-v2 freeze tag は `db38a7eb525a07208bfeb467cfa4d9d8a8a59f51` を維持;
- immutable holdout-v11 / development-v1 freeze coordinate は不変;
- recorded Google / summary artifact は canonical run 37215152913 artifact と byte-identical;
- development-v2 surface checksum は observed freeze commit 上の candidate files を維持し、roadmap は immutable PASS 記録時の追記だけが変化している。

Semantics-freeze commit で追加可能なのは freeze evidence/documentation のみ。implementation code / tests / runners / workflows / observed development surface は変更しない。

Intended freeze coordinate: `engine-0.6-evidence-relevance-successor-v11-semantics-freeze`。

annotated tag push 後にのみ next-holdout 専用 runner を準備・freezeする。runner freeze 後にのみ fresh independent acceptance corpus を author する。acceptance は Mistral + Google + Groq を required provider に復帰させ、observed development case を再利用しない。
