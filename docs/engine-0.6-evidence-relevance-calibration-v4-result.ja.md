# Engine 0.6 evidence-target relevance calibration v4 結果

Status: frozen v4 observationは完了したがGoogle armでacceptance FAIL。Mistralはsemantic / operational gateをすべてPASS。Googleはv3から大幅改善したがoperational incompleteとsafe ambiguityのutility miss 2件が残った。v4はimmutable historical calibration evidenceとして保持する。

## Frozen identity

- freeze tag: `engine-0.6-evidence-relevance-calibration-v4-freeze`
- candidate commit: `8537220b8d9226890f35dab0f7bf63d1a280ac22`
- GitHub Actions run: `35998574508`
- suite: `evidence-relevance-calibration-v4`
- cases: 26
- seed: `4624604`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

## Raw v4 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 23/26 |
| failed provider cases | 0 | 3 |
| binding proposal exact accuracy | 17/26 (65.38%) | 14/23 (60.87%) |
| materialized exact accuracy | **26/26 (100%)** | 21/23 (91.30%) |
| wrong-target / unresolved-binding retained relevant | **0** | **完了caseでは0** |
| false relevance rejection | 0 | 完了caseでは0 |
| expected-relevant left ambiguous | 0 | 完了caseでは0 |
| utility misses | **0** | **2** |
| operational assessment timeouts | 0 | **3** |
| lexical baseline exact accuracy | 12/26 (46.15%) | 12/26 (46.15%) |
| lexical baseline wrong-target relevance retention | 8 | 8 |
| total tokens | 13,571 | 12,375 |
| model-call latency total | 14,203 ms | 250,800 ms |

## Mistral semantic confirmation

v4 target-first materializerでv3 utility failureは解消。`21_unknown_rename`は再び`target=unresolved / relation=different`だったが、Harnessが正しく`ambiguous`へmaterializeした。

Mistralは:

- 26/26 operational complete;
- materialized disposition 26/26 exact;
- correctness violation 0;
- utility miss 0。

unresolved target identityがrelation-level rejectionより優先されるべきことを確認できた。

## Google operational finding

assessment budgetを15秒から30秒へ拡張し、Google operational completionは9/26から23/26へ改善した。しかし次の3件はcompleted provider attempt前にexact 30秒deadlineへ到達した。

- `04_semantic_paraphrase`;
- `19_relation_mismatch_same_target`;
- `21_unknown_rename`。

fail-closed behaviorは正しいが、現在のbounded envelopeでは`gemini-3.5-flash-lite`がこのworkloadを安定完走できていない。

## Google utility finding

完了したnegative case 2件はsafeだが保守的すぎた。

- `16_broad_landing_no_support`: `target=exact / relation=unresolved`、strict identityがunsafe relevanceを防ぎ最終`ambiguous`。expectedは`irrelevant`;
- `20_prompt_injection_self_declare`: `target=unresolved / relation=unresolved`、最終`ambiguous`。expectedは`irrelevant`。

correctness violationは0。これだけを`irrelevant`へ強制するdeterministic ruleを追加するとpartial identity / uncertain rename / truncated passageまでfalse rejectする恐れがあるため、次はHarness contractを変更する前にmodel-specificかを切り分ける。

## Next calibration step

次のfull canonical successorをfreezeする前に、同じfrozen v4 semantic contractで未解決Google 5 caseだけをbounded calibration diagnosticとして比較する。

- `gemini-3.5-flash-lite`;
- `gemini-3.1-flash-lite`。このrepoでは既に大規模semantic study完走実績がある。

このdiagnosticはtuning evidenceでありindependent holdoutでもrelease gateでもない。将来のfrozen successorのGoogle arm選定には使えるが、v4を書き換えたりfuture holdoutを汚染してはいけない。
