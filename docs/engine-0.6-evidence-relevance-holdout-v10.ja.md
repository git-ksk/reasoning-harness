# Engine 0.6 evidence relevance holdout v10

Status: fresh corpus prepared / still unobserved。semantic contract は engine-0.6-evidence-relevance-successor-v9-semantics-freeze の frozen v11 + v23 をそのまま使う。holdout-v10 corpus と live workflow は準備済みだが、provider observation はまだ行っていない。

## Why v10 exists

Canonical holdout v9 は immutable FAIL。事後 adjudication で、既に frozen だった successor-v9 development contract から drift した evaluation-surface expectation 2件と、独立した Google operational timeout 1件を確認した。そのため semantic implementation は retune せず、unchanged semantics で fresh evaluation generation を作る。

## Frozen semantic input

- semantics tag: engine-0.6-evidence-relevance-successor-v9-semantics-freeze
- semantics commit: 3e49a9fd2f7827b7c707516d2150e2f0f16e9e17
- effective qualification: v11
- materialization: v23
- issue binding: #468
- canonical holdout v9 は immutable FAIL のまま

## Runner wiring

Dedicated binary: reason-evidence-relevance-holdout-v10-study。

V10 profile:
- configuration: evidence-relevance-live-holdout-v10
- suite: evidence-relevance-holdout-v10
- annotation protocol: evidence-relevance-effective-qualification-v11
- fixed core: evidence-relevance-fixed-core-v10
- expected directory: fixtures/evidence-relevance-holdout-v10
- expected cases: 26
- issue: 468
- 全case観測時のみ complete holdout

runner freeze 時点では holdout-v10 directory は存在せず、runner-freeze tag push 後にのみ fresh corpus を author した。

## Evaluation-contract guard

corpus freeze 前に relation expectation を frozen successor-v9 development contract と明示照合する。pre-freeze で different_relation と固定された non-frame control を、holdout の都合で unresolved へ変更しない。

canonical v9 label は編集しない。この guard は fresh surface のみ対象。

acceptance の relation-authority contract は v11 freeze 時と同じ selective contract とする。

- require_requested: effective relation scope は requested_relation 必須
- forbid_requested: effective relation scope が requested_relation であってはならない
- preserve_risk: deterministic scope risk を exact に維持

identity scope / scope risk は全26caseで exact 必須。materialized disposition も全26caseで exact 必須。forbid_requested case での different_relation と保守的な relation_absent / unresolved の差は diagnostic とし、acceptance failure にはしない。v11 はこれら advisory negative label を Harness-owned exact authority として freeze していないため。

v10 の固定分布は require_requested 15 / forbid_requested 6 / preserve_risk 5。

## Runner freeze

freeze coordinate: engine-0.6-evidence-relevance-holdout-v10-runner-freeze。

runner-freeze tag push 後に fresh 26-case corpus を author 済み。holdout v1-v9 と全 prior successor development surface に対して case/entity/task/signal/exact-8-token reuse 0。corpus freeze と Groq admission floor 成立前は provider observation 禁止。per-case semantic active budget は90秒、absolute case deadline は120秒のまま。
