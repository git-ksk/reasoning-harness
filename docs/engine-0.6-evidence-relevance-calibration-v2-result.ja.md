# Engine 0.6 evidence-target relevance calibration v2 結果

Status: frozen v2 observationはoperational completeで、両provider armともcorrectness gateをPASSした。ただしutilityが残ったため#462 semanticsはまだfreezeせず、independent holdoutもauthorしない。

## Frozen identity

- freeze tag: `engine-0.6-evidence-relevance-calibration-v2-freeze`
- candidate commit: `e7ffdbe27572d7113410b5f5187a84570d3dd4a4`
- GitHub Actions run: `35992854291`
- suite: `evidence-relevance-calibration-v2`
- cases: 26
- seed: `4622602`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

preflightと両live provider armはSUCCESS。combined final gateはutilityだけでFAILした。

## Raw v2 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 26/26 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 22/26 (84.62%) | 24/26 (92.31%) |
| materialized exact accuracy | 23/26 (88.46%) | 25/26 (96.15%) |
| wrong-target / unresolved-binding retained relevant | **0** | **0** |
| expected-relevant false rejection | 0 | 0 |
| expected-relevant left ambiguous | 0 | 0 |
| utility misses | **3** | **1** |
| ambiguous dispositions | 3 | 7 |
| deterministic safety overrides | 0 | 2 |
| lexical baseline exact accuracy | 12/26 (46.15%) | 12/26 (46.15%) |
| lexical baseline wrong-target relevance retention | 8 | 8 |
| lexical baseline expected-relevant misses | 2 | 2 |
| model calls | 26 | 26 |
| provider attempts | 26 | 26 |
| total tokens | 11,707 | 12,221 |
| model-call latency total | 15,573 ms | 21,895 ms |

v2はv1 correctness failureを解消し、ambiguity handlingも大幅改善したが、zero-utility-miss acceptanceには未到達。

## Mistralの残りutility miss

Mistralは次のexpected `ambiguous`をまだ`irrelevant`へ落とす。

- `21_unknown_rename`: 別名のsourceだがtargetを置換したか未解決;
- `22_partial_identity`: partial product identityをexact targetへbindできない;
- `25_insufficient_local_passage`: exact targetはあるがlocal release-note bulletがsupplied passageから欠落。

v2 promptはこれらをambiguousと明記したが、小さいmodelではsemantic uncertaintyをdestructive rejectionへ潰すことが残る。

## Googleの残りutility miss

Googleは`16_broad_landing_no_support`を`ambiguous`とした。frozen labelは`irrelevant`で、generic cloud-services landing pageにtarget-specific local supportがないcase。

安全側だが、calibration上は不要なcandidate retention。

## v3 design conclusion

prose-only tuningを続けると、微妙な3-way final policy decisionをmodelへ持たせ続けることになる。v3ではrelevance decision自体のmodel-owned authorityを減らす。

model-facing contractをtyped advisory bindingへ分解する。

- target binding: exact / different / unresolved;
- requested-relation binding: exact / different / unresolved。

最終dispositionはHarnessがdeterministicにmaterializeする。

- exact target + exact relation => existing deterministic identity floorを満たす場合に`relevant`;
- affirmative different targetまたはdifferent relation => `irrelevant`;
- unresolved bindingが1つでもあれば`ambiguous`。

modelはsemantic matchingに使うが、最終的なretention/rejection policyはHarness-ownedに戻す。factual contradictionは別責務でtarget/relation bindingを変更しない。

v2 observationはimmutable。変更後にrerun / rescoreせず、v3はnew proposal/materialization contract identity、new calibration identity/freeze、first/only canonical observationを要求する。
