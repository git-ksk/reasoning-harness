# Engine 0.6 evidence-need routing calibration v1 結果

Status: first canonical live calibration observation記録済み。v1 surfaceはfreeze済みで、rerunもrescoreもしない。

## Frozen identity

- Issue: #461
- freeze tag: `engine-0.6-evidence-need-calibration-v1-freeze`
- candidate commit: `8b49adf731f3fe3290b3e904d56b8216954288d9`
- GitHub Actions run: `35954770046`
- corpus: `evidence-need-routing-calibration-v1`
- cases: 22
- seed: `4610600`
- Mistral arm: `ministral-8b-latest`
- Google arm: `gemini-3.5-flash-lite`

provider credentialを読む前のpreflightはPASS。両provider armはGitHub repository secretsのcredential checkを通過し、22 caseすべてのmodel-backed observationを完了した。live job自体は両方SUCCESS。combined final gateはv1 scorerがMistralでcorrectness-boundary violationを1件報告したためFAILになった。

## Raw v1 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 22/22 | 22/22 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 17/22 (77.27%) | 21/22 (95.45%) |
| materialized-mode exact accuracy | 21/22 (95.45%) | 21/22 (95.45%) |
| acquisition exact accuracy | 21/22 (95.45%) | 21/22 (95.45%) |
| v1 reported correctness-boundary violations | 1 | 0 |
| v1 reported utility misses | 0 | 1 |
| provider attempts | 22 | 22 |
| total tokens | 7,383 | 7,826 |
| model-call latency total | 13,460 ms | 16,326 ms |

上記はfrozen v1 runnerが出力した値をそのまま記録する。以下でscorer defectと判断する値についても、v1 artifact自体は変更・再採点しない。

## Finding A — permitted downgradeをv1 scorerがfalse-positive判定

Mistralの`17_external_optional`は、fixture期待`external_optional`に対して`context_only`をproposalした。

frozen Harness decisionは以下。

- materialized mode: `context_only`
- acquisition: `context_only`
- reasons: `baseline`, `model_downgrade_applied`

policyは`context_only`までのmodel downgradeを明示許可しており、supplied contextはcompleteかつtarget-local sufficientだった。したがってunsafe skipped acquisitionではない。しかしv1 scorerはfixtureのexact expected modeより弱いmaterialized modeを一律correctness violationに分類した。

calibration結論: correctnessはexact-route期待値ではなく、Harnessが許可する最低safety floorに対して評価する。exact-route mismatchはdiagnosticとして残す。policy safety floor以上の安価なrouteはcorrectness failureではない。

frozen v1 artifactはrescoreしない。versioned successor runnerでこの区別を実装する。

## Finding B — mixed-targetの実utility over-routing

Googleの`09_mixed_summary_target`はmixed user turnだった。

- supplied articleのsummary
- 別targetとしてproductが現在利用可能かverification

分類対象のexact targetは`Summarize the supplied article.`だけで、`content_local`、context complete、target-local sufficientだった。modelは`external_required`をproposalし、Harnessはconservative baselineを維持してsummary targetにもexternal acquisitionをmaterializeした。

correctness上は安全だが、これは実utility miss。別subrequestのevidence needがtarget-local summary decisionへ漏れるという、productionで観測した問題クラスを再現している。

calibration結論: proposal requestはexact targetだけをclassification subjectにする。surrounding user turnはlanguage/coreference contextに限定し、他targetのevidence requirementを移してはならない。Harness内部のbaseline/minimum/downgrade/reuse値もmodel proposalをanchorしないよう非公開にする。

## v2 tuning requirements

次のcalibration candidateでは22-case v1 corpusは変更せず、runner/prompt/materialization evaluation surfaceだけをversion upする。

新しい明示的freeze identity前に必要な変更:

1. correctnessをexact expected routeではなくminimum Harness-permitted materialized decisionに対して採点する;
2. 不要なstronger acquisitionはcorrectnessではなくutilityとして採点する;
3. exact proposal/mode/acquisition accuracyはdiagnosticとして維持する;
4. mixed request向けtarget-local proposal instructionを強化する;
5. Harness内部baseline/minimum/downgrade/reuse値をmodel-facing classification requestへ出さない;
6. v1 artifactはすべて保持し、v1 freeze tagをrerunしない。

independent holdoutはまだauthorしない。versioned calibration successorをPASSし、candidate semanticsをfreezeしてからholdoutを作成する。
