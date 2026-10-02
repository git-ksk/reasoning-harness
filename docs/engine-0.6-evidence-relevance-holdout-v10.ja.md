# Engine 0.6 evidence relevance holdout v10

Status: runner-freeze preparation only。semantic contract は engine-0.6-evidence-relevance-successor-v9-semantics-freeze の frozen v11 + v23 をそのまま使う。holdout-v10 corpus はまだ author / observe しない。

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

runner freeze 時点では holdout-v10 directory を作らない。

## Evaluation-contract guard

corpus freeze 前に relation expectation を frozen successor-v9 development contract と明示照合する。pre-freeze で different_relation と固定された non-frame control を、holdout の都合で unresolved へ変更しない。

canonical v9 label は編集しない。この guard は fresh surface のみ対象。

## Runner freeze

freeze coordinate: engine-0.6-evidence-relevance-holdout-v10-runner-freeze。

この tag push 後にのみ fresh 26-case corpus を author する。holdout v1-v9 と全 prior successor development surface に対して case/entity/task/signal/exact-8-token reuse 0 を要求する。corpus 自体の freeze 前は provider observation 禁止。
