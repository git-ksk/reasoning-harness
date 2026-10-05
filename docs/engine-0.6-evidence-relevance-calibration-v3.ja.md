# Engine 0.6 candidate: evidence-target relevance calibration v3

Status: frozen v2後のfresh unobserved successorをauthor済み。v1/v2はimmutableのまま保持し、rerun / rescoreしない。

## v3の理由

frozen v2 run `35992854291` はMistral / Google両armでcorrectness violation 0まで到達したが、utilityはまだFAILした。小さいmodelがunresolved bindingを破壊的な`irrelevant`へ畳む場合が残り、Googleもgeneric page 1件を安全側の`ambiguous`へ残した。

最終`relevant / irrelevant / ambiguous` policyまでmodelに持たせる責務が広すぎるため、v3ではprose tuningを続けずmodel authority自体を狭める。

## Binding proposal contract

modelは最終dispositionを返さない。

返すのはadvisoryな2つのbindingだけ。

- `target_binding`: `exact | different | unresolved`;
- `relation_binding`: `exact | different | unresolved`。

最終dispositionはHarnessがdeterministicにmaterializeする。

- `exact + exact` => `relevant`。ただし既存strict identity floorを再適用;
- どちらかが`different` => `irrelevant`;
- それ以外で`unresolved`を含む => `ambiguous`。

proposal欠落時も`ambiguous`。

これにより破壊的なreject policyはHarness-ownedのままになる。modelはtarget identity、source authority、trust、freshness、verification、truth、verdictを作れない。

## Semantic boundary

`different`は別targetまたは別requested relationだと肯定的に判断できる場合だけ。missing / partial / truncated / mixed / rename不確実 / alias不確実 / local applicability未解決は`unresolved`。

同じexact target/relationについて事実が矛盾していてもbindingは`exact + exact`のまま。contradiction / truthは後段で扱う。

strict Harness-owned identity anchorは変更しない。URLだけ、navigation/footerだけではstrict target identityを満たせない。semantic-equivalent policyもHarness-ownedの明示設定のまま。

## Fresh v3 corpus

- suite: `evidence-relevance-calibration-v3`
- issue: #462
- cases: 26
- status: `fresh_unobserved_calibration`
- production motivating incident: tuningから除外

semantic familyはv2と比較可能に保ちつつ、expected model outputを新しいtarget/relation binding contractで再authorした。

## Acceptance

canonical Mistral / Google両armで次を要求する。

- operational complete;
- wrong-target / unresolved-binding materialized `relevant`: **0**;
- utility miss: **0**;
- provider failure: **0**。

binding proposal exact accuracyはdiagnostic。deterministic safety overrideはobservableに保つ。lexical baselineもdiagnosticのみ。

## Freeze discipline

first live observation前に:

1. deterministic v3 binding materialization 26/26 PASS;
2. core/runner tests・clippy PASS;
3. exact core/runner/fixture surfaceをchecksum;
4. live workflowをcommit;
5. `engine-0.6-evidence-relevance-calibration-v3-freeze` tagでfirst/only canonical v3 observationを固定。

v3 PASS後に#462 semanticsをfreezeするまでindependent holdoutはauthorしない。
