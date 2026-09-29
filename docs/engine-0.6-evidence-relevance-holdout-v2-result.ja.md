# Engine 0.6 evidence relevance independent holdout v2 result

Status: immutable canonical **FAIL**。

- Freeze tag: `engine-0.6-evidence-relevance-holdout-v2-freeze`
- Freeze commit: `c38b5f7dbfcf8c01dcaa6f6cd7341e43d3b87325`
- Canonical run: `36533340582`、attempt 1 only
- Cases: 26（Relevant 8 / Irrelevant 10 / Ambiguous 8）
- Required provider: Mistral + Groq + Google
- Annotation protocol: `evidence-relevance-effective-qualification-v4`
- Materialization: v17
- Operational: 3 required armすべて26/26完走、provider jobはSUCCESS
- Final gate: FAIL

この結果はimmutable。rerun / rescore / relabel / expected label変更 / freeze tagの再作成・移動 / このholdout identityのPASSへの読み替えは禁止。

## Required Mistral

`ministral-8b-latest` は26/26完走。

- authority-qualified effective qualification: 25/26
- materialized exact: 25/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 1
- total tokens: 45,977
- latency p50/p95/max: 1,338 / 2,472 / 2,677 ms

terminal missは `v2h13_negative_unrelated_launch` の1件だけ。expected Irrelevant -> observed Ambiguous。proposal / raw local qualificationともrequested targetへ過剰bindしたが、canonical/alias target anchorが存在しないためHarness-owned strict identity floorがwrong-target Relevantを防いだ。

## Required Groq

`openai/gpt-oss-120b` は26/26完走し、frozen semantic / operational contractを完全に満たした。

- authority-qualified effective qualification: 26/26
- materialized exact: 26/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 0
- total tokens: 65,555
- latency p50/p95/max: 10,708 / 11,026 / 11,607 ms
- provider failure: 0
- retry wait: 0
- provider arm latch: none

## Required Google

`gemini-3.5-flash-lite` は26/26完走。

- authority-qualified effective qualification: 25/26
- materialized exact: 25/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 1
- total tokens: 47,427
- latency p50/p95/max: 6,877 / 7,655 / 7,818 ms

terminal missは同じ `v2h13_negative_unrelated_launch`。expected Irrelevant -> observed Ambiguous。Google proposalはtarget=unresolved、raw local verifierはidentityをexact_targetへ過剰bindしたため、effective qualificationはHarness-owned negative identity authorityを確立できずmaterializerがabstainした。

## 共通failed case

frozen `v2h13_negative_unrelated_launch`:

- target/relation: `Obsidian Monitor` / change_or_launch
- candidate source title: `Obsidian Trace release`
- candidate excerpt: `Obsidian Trace launches a new span waterfall view for distributed tracing.`
- expected proposal: different / exact
- expected local qualification: distinct_target / requested_relation / none
- expected disposition: Irrelevant

candidateには `Obsidian Monitor` のcanonical/alias anchorがなく、requested launch relationを持つ別のsibling-looking subject `Obsidian Trace` を一貫して明示する。Groqは最終26/26まで正しく扱えたが、Mistral / Googleは扱えなかった。

v4/v17 safety boundary自体は正しく動作した。required Harness target anchorがないためwrong-target materialをRelevantには通していない。残る問題はpositive relevance safetyではなくutility / negative-identity authority。

## Root cause

v4はholdout-v1のcomparison/context-only defectを修正した一方、strict target anchorが存在しない場合に次の2種類をgenericに分離するHarness-owned ruleを持っていない。

1. unconfirmed alias / rename / successor、omitted referent等でrequested targetかもしれない本当にunresolvedなidentity
2. visibleな別sibling/entityがsubstantive subjectで、requested relationもそのvisible subjectへ属するlocally substantive evidence

現在のdeterministic negative-identity signalはexplicit separation（`separate` / `distinct` product/service）、explicit non-mapping、comparison markerを対象とする。`v2h13` にはその明示cueがないため、Mistral / Googleのmodel over-bindingをHarness authorityで `distinct_target` へ補正できない。

successorでは **mere target-anchor absenceをIrrelevant化せずにnegative identity corroborationを強化**する必要がある。URL-only identity、identity-mapping uncertainty、truncated / omitted ownership、pronoun-only evidenceはAmbiguousのまま維持する。

## Final gate

precommitted required gate:

- required operational completeness: PASS
- required correctness hard gate: PASS
- required utility: **FAIL**
- required materialization: **FAIL**
- required qualification: **FAIL**

zero wrong-target Relevant hard gateは3 providerすべてPASS。independent holdoutがrequired semantic gateをすべて満たしていないためIssue #462はopenのまま。

## Successor direction

holdout v2はtuning surfaceにせず、rerun / relabelしない。別version successorがimmutable observationをpostmortem / regression用途でreplayすることだけ許可する。

successorはprovider / fixture / entity neutralを維持し、次を保つ。

- v23 immutable calibration regression 0
- holdout-v1 wrong-target safety repair
- holdout-v2 wrong-target Relevant 0
- URL-only identity、identity-mapping uncertainty、ownership/context gap、missing/truncated referentはAmbiguous
- `v2h13`、Obsidian名、provider名、fixture textでbranchしない

successor semanticsを先にfreezeした後でのみ、次のfresh independent holdoutをauthorする。
