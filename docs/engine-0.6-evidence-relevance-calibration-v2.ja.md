# Engine 0.6 candidate: evidence-target relevance calibration v2

Status: frozen v1後のfresh unobserved successorを作成。v1はimmutableのまま保持し、rerun / rescoreしない。

## v2の理由

frozen v1 run `35991268202` はMistral / Google両armともoperational completeだったがsemantic gateはFAILした。v2は次の一般的findingだけを扱う。

1. model guidanceでaffirmative irrelevanceとunresolved applicability/bindingの境界が弱く、ambiguity caseを不要に`irrelevant`へ落としていた;
2. v1 `24_conflicting_sections` はexact target/relationに対するfactual contradictionであり、relevance後段の責務だったためwrong-target relevance failureとして不適切だった。

## Successor changes

v2で変更するのはこのcalibration-facing semantic boundaryだけ。

advisory decision ruleを次のように明確化する。

- `relevant`: exact targetとrequested relationについてdownstream considerationへ残すのに十分;
- `irrelevant`: 別targetまたは別requested relationだと肯定的に判断できる;
- `ambiguous`: identity / relation binding / applicabilityを確立できない。partial/truncated passage、rename/alias不確実、mixed-product unresolved binding、local support欠落などを含む;
- missing / insufficient local informationだけではirrelevantにしない;
- same target/relationのfactual contradictionはirrelevanceではなく、contradiction/truthは後段で扱う。

v2ではdeterministic identity floor、authority boundary、source/provenance、freshness、verification、finalizationは変更しない。

## Fresh v2 corpus

- suite: `evidence-relevance-calibration-v2`
- issue: #462
- cases: 26
- status: `fresh_unobserved_calibration`
- production motivating incident: tuningから除外

v2 corpusはfrozen v1のsemantic coverageを維持しつつ、case 24だけを本当のrelation-binding ambiguityへ置き換える。Cedar Vault / Cedar Vault Classic共有regional tableにWest rowがあるが、supplied local excerptではどちらのproductへ適用されるか確立できない形にする。

v1 truth-conflict fixtureはv1 freeze/tagとv1 result docにのみhistorical evidenceとして残す。

## Acceptance

canonical Mistral / Google両armで次を要求する。

- operational complete;
- wrong-target / unresolved-binding materialized `relevant`: **0**;
- utility miss: **0**;
- provider failure: **0**。

proposal exact accuracyはdiagnostic。Harness-owned deterministic safety overrideは許容し、必ずobservableにする。

simple lexical baselineはdiagnosticのみ。semantic/cross-lingualではfalse miss、same-name wrong-featureではover-retentionする比較基準として扱う。

## Freeze discipline

first live observation前に:

1. deterministic v2 materialization 26/26 PASS;
2. runner/core clippy・tests PASS;
3. exact core/runner/fixture surfaceをchecksum;
4. live workflowをcommit;
5. `engine-0.6-evidence-relevance-calibration-v2-freeze` tagでfirst/only canonical v2 observationを固定。

v2 calibration acceptance PASS後に#462 semanticsをfreezeするまでindependent holdoutはauthorしない。
