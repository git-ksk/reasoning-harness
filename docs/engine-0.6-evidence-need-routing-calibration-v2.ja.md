# Engine 0.6 evidence-need routing calibration v2

Status: frozen v1 observation後のtuning successor準備済み。22-case calibration corpusは変更しない。

## v2の理由

frozen calibration v1（GitHub Actions run `35954770046`）ではprovider/runtime incompletenessなしで2つのfindingが出た。

1. v1 scorerが、Harnessに明示許可された安価なdowngradeをcorrectness failureとして誤分類した;
2. Googleがmixed-requestのsummary targetを1件over-routeした。surrounding subrequestのverification requirementとHarness内部routing controlがmodel proposalへ影響したため。

raw v1 artifactはimmutableのまま保持し、rescoreしない。詳細は[calibration v1 result](engine-0.6-evidence-need-routing-calibration-v1-result.ja.md)。

## 変更点

calibration corpus、expected label、case順序は`evidence-need-routing-calibration-v1`から一切変更しない。

versioned candidate/evaluation surfaceだけを変更する。

- runner configurationを`evidence-need-routing-live-calibration-v2`へ変更;
- correctnessをexact expected routeではなくminimum Harness-permitted materialized routeに対して採点;
- 明示許可された安価なrouteはexact-route mismatch diagnosticには残すがcorrectness/utility failureにはしない;
- expectedよりstrongなmode/acquisitionはutility missとして維持;
- model-facing requestではexact targetだけをclassification subjectにする;
- surrounding user turnはlanguage/coreference contextだけに限定し、target間でevidence requirementを移さない;
- Harness内部の`baseline_mode` / `minimum_mode` / `model_downgrade_floor` / `existing_evidence`をclassifierへ見せない。

## Frozen v2 live surface

- freeze tag: `engine-0.6-evidence-need-calibration-v2-freeze`
- workflow: `.github/workflows/engine-0.6-evidence-need-calibration-v2-live.yml`
- corpus: unchanged 22-case `evidence-need-routing-calibration-v1`
- seed: `4610600`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`
- credential: GitHub repository secretsのみ
- checksum: `fixtures/evidence-need-routing-calibration-v1/surface-v2.sha256`

同一frozen observationのworkflow rerunは拒否する。追加tuningが必要なら別の明示的versioned freeze identityを作る。

## Acceptance interpretation

v2 calibrationからsemantic freezeへ進める条件:

- 両provider armがoperationally complete: 22/22、provider failure 0;
- 両armのcorrectness-boundary violation 0;
- mixed-targetの不要external acquisition解消;
- exact-route mismatchはdiagnosticとして可視化維持;
- utilityとcorrectnessを別metricとして維持。

v2観測とcandidate semantics freezeが終わるまでindependent holdoutはauthorしない。
