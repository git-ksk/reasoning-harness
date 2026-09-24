# Engine 0.6 evidence-need routing calibration v3 結果

Status: frozen v3 observationは成功。両provider armがoperational / correctness / utility gateをすべてPASSした。#461 candidate semanticsはここでfreezeし、別途authorするindependent holdout観測前の追加calibration tuningは禁止する。

## Frozen identity

- freeze tag: `engine-0.6-evidence-need-calibration-v3-freeze`
- candidate commit: `34494f498b2be6534fe01627de64486c86604965`
- GitHub Actions run: `35957170730`
- corpus: 変更なしの `evidence-need-routing-calibration-v1`
- cases: 22
- seed: `4610600`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

preflight、両live job、combined final gateはGitHub repository secretsを利用してすべて成功した。

## Raw v3 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 22/22 | 22/22 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 20/22 (90.91%) | 21/22 (95.45%) |
| materialized-mode exact accuracy | 21/22 (95.45%) | 22/22 (100%) |
| acquisition exact accuracy | 21/22 (95.45%) | 22/22 (100%) |
| correctness-boundary violations | **0** | **0** |
| utility misses | **0** | **0** |
| provider attempts | 22 | 22 |
| total tokens | 10,517 | 10,972 |
| model-call latency total | 16,425 ms | 17,089 ms |

## Diagnostic mismatch

Mistral `17_external_optional` はexact expected `external_optional`ではなく`context_only`をproposalした。ただしHarness-owned policyは`context_only`までのdowngradeを明示許可し、supplied contextはcompleteかつsufficient、minimum permitted acquisitionもcontext-onlyである。よってcorrectness violationでもutility missでもない。

`22_ambiguous_target_conservative`では両providerのproposalがexact expectationと異なったが、Harness-owned ambiguous-target floorが`external_required`をmaterializeし、最終mode/acquisitionは両armとも正しい。

これは意図したarchitectureを示す。model routing proposalはadvisoryであり、authoritative acquisition boundaryはHarness-owned floorとdowngrade permissionが決める。

## v3 authority-boundary確認

v2で見つかった「untrusted modelが`trusted_verification_required`を新規作成できる」問題はv3でcloseした。

- model-facing enumにtrusted verificationを含めるのはHarness policyがtrusted verificationを明示要求する場合だけ;
- Harness-owned flagがfalseなら手動trusted proposalも無視;
- ordinary `external_required`までのescalationはadvisoryとして維挍;
- explicit verification / current state / target kind / trusted verification floorはHarness-ownedのまま。

v3 observationではmodel proposalによるtrusted authority classの新規作成は発生していない。

## Freeze decision

#461 calibration phaseは完了。

independent holdout authoring前に次のcandidate semanticsをfreezeする。

- target-local evidence-need decision;
- typed target kindとhard floor;
- explicit verification / current-state / trusted-verification floor;
- bounded model downgrade permission;
- modelがtrusted authorityを作れないこと;
- context completenessとtarget-local sufficiencyの分離;
- evidence needとacquisition/reuseの分離;
- mixed-target independence;
- follow-up recomputation;
- replay-safe serializable decision state;
- minimum Harness-permitted routeに対するcorrectness scoring;
- avoidable stronger acquisitionに対するutility scoring。

今後holdout observationを見てからcalibration fixture、expected label、policy threshold、model-facing contract、materialization rule、scorer semanticsを変更してはならない。semantic changeが必要なら新しいversioned research identityと新しいindependent evaluationを作る。

## Next gate

このfreeze後にfresh independent holdoutをauthorする。holdoutはfirst model-backed observation前に別checksumとfreeze tagで固定し、calibration prompt/caseを再利用しない。

hard gateは同じ。

- unsafe skipped acquisition = 0;
- context authority laundering = 0;
- explicit verification downgrade = 0;
- current-state downgrade = 0;
- trusted-verification downgrade / model-created trusted authority = 0;
- invalid existing-evidence reuse = 0;
- mixed-target whole-turn over-routing = 0;
- replayed external side effect = 0;
- provider failureはsemantic failureと分離。

#461をindependent acceptedとみなすにはcorrectness / utilityの両gateをPASSする必要がある。
