# Engine 0.6 evidence-target relevance calibration v3 結果

Status: frozen v3 observationは完了したがacceptanceはFAIL。Mistralはoperational completeでcorrectness violation 0、utility miss 1。Googleは完了したcaseではsemantic exactだったが、Harness-owned 15秒assessment deadlineにより17件がprovider応答完了前にtimeoutしoperational incomplete。本結果はimmutable historical calibration evidenceとして保持する。

## Frozen identity

- freeze tag: `engine-0.6-evidence-relevance-calibration-v3-freeze`
- candidate commit: `53402a19eb979b53c7ddaa80afceffcdfbee8fbe`
- GitHub Actions run: `35996093336`
- suite: `evidence-relevance-calibration-v3`
- cases: 26
- seed: `4623603`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

preflightではexact frozen checksum、validate-only contract、fmt、clippy、deterministic v3 materialization、runner testがすべてPASS。両live jobはfirst canonical observationを保存した。combined gateはMistral utility mismatch 1件とGoogle operational incompleteによりFAIL。

## Raw v3 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 9/26 |
| failed provider cases | 0 | 17 |
| binding proposal exact accuracy | 17/26 (65.38%) | 7/9 (77.78%) |
| materialized exact accuracy | 25/26 (96.15%) | 9/9 (100%) |
| wrong-target / unresolved-binding retained relevant | **0** | **完了caseでは0** |
| false relevance rejection | 0 | 完了caseでは0 |
| expected-relevant left ambiguous | 0 | 完了caseでは0 |
| utility misses | **1** | 完了caseでは0 |
| operational assessment timeouts | 0 | **17** |
| lexical baseline exact accuracy | 12/26 (46.15%) | 12/26 (46.15%) |
| lexical baseline wrong-target relevance retention | 8 | 8 |
| model calls | 26 | 26 |
| provider attempts | 26 | 9 |
| total tokens | 13,571 | 4,896 |
| model-call latency total | 18,439 ms | 307,129 ms |

Googleは17件がprovider response前に失敗したためcomplete canonical semantic armとしてはscoreしない。

## Semantic finding: binding precedence

Mistral `21_unknown_rename` は:

- target binding: `unresolved`;
- relation binding: `different`

を返した。frozen v3 materializerはどちらかが`different`なら最終`irrelevant`としていたためutility missになった。

target identity自体がunresolvedなら、relation bindingだけでrequested targetにirrelevantだと破壊的に確定してはいけない。final policyは次のhierarchyが妥当。

1. target `different` => irrelevant;
2. target `unresolved` => ambiguous;
3. target `exact` + relation `different` => irrelevant;
4. target `exact` + relation `unresolved` => ambiguous;
5. target `exact` + relation `exact` => relevant。ただしdeterministic identity floorを再適用。

これによりaffirmative wrong-target rejectionは維持しつつ、target identity未解決のmaterialをsecondary relation判定で破棄しない。

## Operational finding: 15秒deadline

Googleは9/26のみ完了。17件すべて約15,000msでtyped `assessment_timeout`、`provider_attempts = 0`だった。completed provider attemptが記録される前にHarness deadlineがprovider futureを終了した形。

完了したGoogle 9件はmaterialized 9/9 exactなので、Google semantic regressionを示す結果ではない。

15秒budgetはboundedだが今回のprovider/model観測には狭すぎた。successorではfinite Harness-owned deadline、2 model-call上限、operational/semantic failure分離を維持したままelapsed budgetを拡張できる。frozen v3自体はrerunしない。

## v4 requirements

successorは新しいfreeze identityを使い、v3をrewrite/rerunしない。変更は次の2点に限定する。

1. unresolved target identityがrelation-level rejectionより優先されるfinal binding materialization hierarchy;
2. observed provider latency envelopeに対応できる、引き続きboundedなassessment elapsed budget。

proposal contractはv2 binding contractを維持可能。ただしmaterialization semanticsが変わるためHarness materialization policy identityはadvanceする。

両canonical provider armでcorrectness / utility / operational completenessをPASSするまでindependent holdoutはauthorしない。
