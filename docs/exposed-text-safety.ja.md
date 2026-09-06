# 表示文章の安全性

Issue #210 は、Reasoning Harness が検証した structured factual claim と、自然言語product pathで実際にユーザーへ表示する文章の間にあったgapを閉じる。

## 現在のpolicy

machine identity: `harness-canonical-exposed-text-v1`。

renderer互換性のため `FinalAnswerCandidate` は引き続き `text` と `factual_claims` の両方を持つが、authorityは同じではない。

1. modelが生成した `text` はadvisoryに限定する。
2. `factual_claims` は既存のgrounded/uncertain ruleでfinal verified artifactと照合する。
3. finalizationが回答を公開できる場合、実際の表示文章はaccepted factual claimとそのmodeからHarness codeが構築する。
4. target-local recoveryの文章もtyped verified stateからHarnessが構築する。
5. rendererの `text` を `GroundedAnswer` / `QualifiedPartialAnswer` へそのままコピーしない。

これにより、rendererがclaim宣言を省略する、本文だけで矛盾する、本文だけに追加factを書く、本文だけcertaintyを強める、といった挙動はcorrectness保証対象の表示文章を変更できない。model rendererにauthorityを与えるものではなく、evidence admission、qualification、verification、answer-safety、global verdict、target-local recoveryのsemanticsも緩めない。

## Product telemetryとwire contract

自然文JSON出力は `reason-natural-output-v4` を使い、次を含む。

```json
"exposed_text": {
  "policy_id": "harness-canonical-exposed-text-v1",
  "renderer_text_exposed": false
}
```

`finalization.factual_claim_coverage` はstructured claim coverageの指標として維持する。`exposed_text` observationはユーザー表示文の生成policyを別に記録し、structured coverageだけを arbitrary renderer prose の安全性指標として扱わない。

## 互換性とmigration

`reason-natural-output-v2` では、verified structured claimとrenderer `text` が食い違ってもrenderer textが公開され得た。rendererの自然なprose presentationに依存していたconsumerは、v3で同じ表現を前提にしてはいけない。supported exposed answerは `finalization.text`、inspect可能なcorrectness surfaceはstructured artifact/claimとして扱う。

canonical textの具体的な文言はpresentationであり、将来は明示的にversionされたpolicyのもとで変更し得る。ただしauthority ruleはfail-closedを維持する。

## 再現性とrollback note

#210以前の挙動はmain commit `3a601c8` と `reason-natural-output-v2` で再現できる。このbaselineはhistorical reproduction専用である。model renderer textをgrounded exposed outputへ戻すruntime flagは意図的に提供しない。戻すとP0 correctness gapを再導入するためである。

将来rich proseを再導入する場合は、新しいexposed-text policy identityを与え、公開するすべてのfactual assertionをHarness-owned verified stateへ機械的にbindingできることをadoption前に示さなければならない。

historical Stage-C、RSD2、`product-external-info-v1/v2/v3/v4` の観測結果は書き換えない。各観測がexposed-text safetyを明示的に測定していない限り、`unsupported grounded claims` はstructured-claim metricとして扱う。
