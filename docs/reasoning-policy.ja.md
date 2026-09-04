# Reasoning policy と依存関係の無効化

Issue #27 は ADR-0003 の policy/invalidation トラックを実装する。policy layer が制御するのは **admissibility と escalation** であり、truth ではない。

`ReasoningPolicy` は evidence、verification receipt、hard finding、epistemic promotion、final verdict を生成できない。すでに authority を持つ state のうち許容されるものと、runtime が追加で要求できる作業だけを制約する。

## ポリシーの構成

Policy layer は通常、次の順序で明示的に合成する。

```text
global
  -> domain
  -> task/run
```

`ReasoningPolicyLayer` が持つのは汎用制約だけで、core に domain-specific source taxonomy はない。

- minimum authority class: 最も高い rank の要件を採用する。
- applicability scope: layer を積集合にする。互いに素なら configuration error。
- derived-support capability: restrictive AND。後続 layer で厳しい制約を緩められない。
- allowed resolver classes: 集合の積集合。
- evaluation `as_of` time: 後の contextual layer が上書きし、その後 qualification を再実行する。
- soft-finding escalation: escalation は作業要求であって truth の確立ではないため、後続 layer が上書きできる。

allowed resolver classes の `None` は「この layer による追加制約なし」であり、「すべての resolver を拒否」ではない。明示的な空集合は全 resolver class を拒否する。

effective policy は安定した `version_id` を持ち、source layer ID を記録する。

## ポリシーの検証

caller が `compose_reasoning_policy` を使わず直接 deserialize した場合も、public `ReasoningPolicy` value を検証する。

runtime は次を reject する。

- 空の policy version ID
- 空の source-layer ID
- harness-owned rank policy にない authority class
- 空の scope dimension
- 空の scope value set または value

これにより、構築経路に依存せず policy configuration を fail-closed に保つ。

## Evidence qualification との関係

policy は proposition evidence requirements に汎用的な temporal/scope/authority constraint を追加できる。既存の task-specific requirement が優先される。

- 明示済みの `as_of` requirement は policy context で上書きしない。
- scope は policy scope と積集合にする。
- minimum authority は強い方へ厳格化する。

typed claim/hypothesis に requirement がなく policy が evidence constraint を定義する場合、transition はその proposition key の effective requirement を作成する。その後 evidence qualification を再実行する。metadata が関連しているだけでは claim を promote しない。

## Hard authority の保持

Policy change は **new artifact snapshot** 上で行い、historical input artifact は変更しない。

- `supported` / `contradicted`: 一致する retained verification receipt が引き続き admissible であること。
- `known`: direct evidence が effective evidence qualification を満たすこと。
- `inferred`: derived support が許可され、依存 chain が有効な場合だけ derived working state として残ること。

evidence はあるが reconstructable receipt のない `supported` state は、label だけを信頼せず transition 中に downgrade する。Receipt binding は通常の verification/validation と同じ matcher を使う。

## 無効化の伝播

厳しい policy により upstream authority が inadmissible になると、`apply_reasoning_policy` は typed `PolicyInvalidation` record を出し、新しい accepted-state snapshot を構築する。対象は verification receipt、claim、inference edge、finalization である。

1. dependent inference edge を無効化し、新 snapshot から除去する。
2. dependent claim を `assumed` に downgrade する。
3. downstream inference edge へ伝播する。
4. finalization を無効化する。
5. strict acceptance policy を再評価する。

除去された edge は旧 immutable snapshot に残る。durable historical lineage は current artifact の mutation ではなく #28 `ReasoningThread` に属する。invalidation 後は policy-sensitive evidence-qualification/assumption finding も再計算する。

## Soft semantic の検出結果

calibrated `SoftJudgeObservation` は evidence 要求、deterministic verification 要求、human review 要求を advisory action として発火できる。ただし artifact を直接変更したり hard authority を作ったりはできない。結果は #22 の evidence admission / qualification / verifier boundary を通過しなければならない。

## Resolution ポリシー

`constrain_resolution_policy` ができるのは既存の #22 `GroundedResolutionPolicy` を厳しくすることだけである。resolver-class allowlist は積集合にし、required evidence authority は厳しい class に引き上げる。第二の resolver abstraction や acquisition logic は作らない。

## 決定論的ポリシーのリグレッション

`fixtures/policy/` には authority tightening、temporal re-evaluation、scope-expansion rejection、無効化された support から inference edge を通る dependency propagation の4つの provider-neutral regression scenario がある。新 snapshot から inadmissible な receipt/claim/edge が失われ、finalization が無効化され、元 artifact が不変であることを検証する。これらは corpus-v1 correctness、resolution recovery、semantic-judge calibration denominator とは別である。

## 対象外

- core における domain-specific evidence/source policy
- generic agent approval UX
- policy authority としての model confidence
- policy-generated verification receipt
- hidden chain-of-thought policy または persistence
- workflow graph orchestration

Checkpoint/resume/fork と durable event history は意図的に #28 に残す。
