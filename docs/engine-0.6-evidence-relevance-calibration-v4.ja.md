# Engine 0.6 candidate: evidence-target relevance calibration v4

Status: frozen v3後のfresh unobserved successor。v1-v3はimmutableのまま保持する。

## v4の理由

frozen v3 run `35996093336` で残件が2種類に分離できた。

1. uncertain rename caseでMistralが`target=unresolved / relation=different`を返し、v3 materializerがrelation-level `different`を優先してunresolved target identityを破壊的にrejectした;
2. GoogleはHarness-owned 15秒assessment deadlineにより17件がcompleted provider attempt前にtimeoutし、9/26しか完了しなかった。

## v4 semantic change

binding proposal contractは`reason-evidence-relevance-binding-proposal-v2`を維持する。

final Harness-owned materialization policyは`target-evidence-relevance-binding-materialization-v3`へadvanceし、hierarchyを次に変更する。

1. target `different` => `irrelevant`;
2. target `unresolved` => `ambiguous`;
3. target `exact` + relation `different` => `irrelevant`;
4. target `exact` + relation `unresolved` => `ambiguous`;
5. target `exact` + relation `exact` => `relevant`。既存deterministic identity floorはそのまま。

これによりrelation-level rejectionがunresolved target identityを上書きできなくなる。

## v4 operational change

explicit assessment elapsed budgetを15,000msから30,000msへ拡張する。model-call上限2、max output 192 tokensは変更しない。assessmentは引き続きboundedでtimeout時はfail-closed。

これはv3のrerunではなく、Harness-owned operational budgetを変更した新しいfrozen evaluation identity。

## Fresh v4 corpus

- suite: `evidence-relevance-calibration-v4`
- issue: #462
- cases: 26
- status: `fresh_unobserved_calibration`
- production motivating incident: tuningから除外

semantic case setはv3と比較可能なまま。expected model bindingは変更せず、materializer precedenceとexplicit elapsed budgetだけを変更する。

## Acceptance

canonical Mistral / Google両armで次を要求する。

- operational complete;
- wrong-target / unresolved-binding materialized `relevant`: **0**;
- utility miss: **0**;
- provider failure: **0**。

proposal exact accuracyはdiagnostic。

v4 PASS後に#462 semanticsをfreezeするまでindependent holdoutはauthorしない。
