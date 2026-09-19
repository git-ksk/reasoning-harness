# Action materialization v1 — measurement surface失敗結果

Issue #283の最初のfresh adoption holdoutは`action-materialization-v1`としてfreezeし実行しましたが、**product比較として有効な結果にはなりませんでした**。両provider・両coordinateで、semantic trialがcompleteになる前に同じevaluation fixture defectが発生しました。

## Freeze座標

- freeze tag: `action-materialization-v1-freeze`
- freeze commit: `75d61b891ebd13f7ba43b3e4351baadf38bebb40`
- control product commit: `94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e`
- candidate product commit: `7a91d272af1bab0a97bf80ed7bba027ff253d50a`
- corpus: `action-materialization-v1`
- evaluator: `reason-action-materialization-v1`
- scoring: `action-materialization-scoring-v1`
- canonical Actions run: `35428076586`
- Mistral job: `105857443683`
- Google job: `105857443732`
- primary seed: `97211`〜`97215`
- observed all-k group: `k=5`

surface、evaluator、scoring identity、provider/model座標、seed、success predicate、coordinate順序、k groupはfirst provider call前にfreezeしました。semantic rerunはしていません。

## 観測

両jobともfrozen live attempt自体は実行されましたが、paired trialがすべてoperationally incompleteとなりacceptanceはfailureでした。一方でhard correctness boundaryは維持されています。

| provider/model | control complete | candidate complete | correctness violation control | correctness violation candidate |
| --- | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` | 0/5 | 0/5 | 0 | 0 |
| Google `gemini-3.5-flash-lite` | 0/5 | 0/5 | 0 | 0 |

failureはgrounded same-key-sibling caseに限定され、stale/unknown caseは正常completeでした。grounded caseでは、両provider・両coordinate・全trialで`malformed_output`がちょうど1件記録されました。

fixture failure前には#283のpath差自体は観測できています。

- controlはsame-key sibling caseでlegacy executable-action model pathを通過;
- candidateは`reason-investigation-intent-v1`を通過;
- candidateは`target-intent-materialization-v1`でHarness materializationを実行;
- candidateの新pathではlegacy executable-action planner callは0;
- intent rejectionは0;
- same-key sibling target identityは別identityのまま維持;
- correctness-boundary violationは0。

ただしこれらはdiagnostic path evidenceであり、不完全runをadoption resultには昇格しません。

## Root cause

v1 fixture resolverのevidence IDは次の固定式でした。

```text
e2e:<source>:<fact_key>
```

別identityのsame-key sibling targetが同じsource/factを取得すると、2つ目のtargetでも1つ目と同じevidence IDが返ります。

通常のresolution engineは、current inputにすでに存在するevidence IDが再度取得された場合、そのattemptを`MalformedOutput`として拒否します。これはduplicate-ID safetyとして正しい挙動です。問題はcontrol/candidate productではなくevaluation fixture identityにありました。

両providerで同じpatternを再現したため、provider固有挙動や#283 candidate regressionではなく、provider-neutralなfixture defectと判断します。

## Disposition

`action-materialization-v1`はfailed measurement evidenceとして保存し、repair/retryしません。

successorは`action-materialization-v2`です。

- control/candidate product commitはv1と完全に同じ;
- v1 caseを再利用せずfresh case / seedを使用;
- exact canonical target IDをevidence IDへ含めるtarget-aware専用fixture resolverを使用;
- 別sibling target IDでは別evidence IDになることをpreflight;
- 同じtarget IDの再取得では同じIDになるidempotencyもpreflight;
- correctness / operational / path separationのacceptance semanticsは維持。

v1 failureを理由にproduct codeは変更していません。

## 保存したmachine report

- [Mistral raw machine report](observations/action-materialization-v1-mistral-run-35428076586-2026-09-19.json), SHA-256 `2b07140a7d5fcb675f14de269ee6ffc4c772842d51fca250f9afc6770c340081`
- [Google raw machine report](observations/action-materialization-v1-google-run-35428076586-2026-09-19.json), SHA-256 `00dfbda6dc447f1833119b5819c2ad26dbbe16e3dba76e56c91c04cf8af2590b`
