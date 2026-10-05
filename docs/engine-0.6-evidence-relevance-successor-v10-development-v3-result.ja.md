# Engine 0.6 evidence relevance successor-v10 development v3 result

Status: immutable development FAIL。

- Freeze tag: engine-0.6-evidence-relevance-successor-v10-development-v3-freeze
- Freeze commit: d25d2a2d3828f85f198568e312f59fc8b55ff5e0
- Run: 37105677785
- Candidate: effective qualification v14 / materialization v27
- Fresh surface: 20 case
- Result: FAIL

tag と attempt-1 observation は immutable。rerun / relabel / tag移動 / PASSへの再解釈は禁止。

## Mistral

Mistral は operational 20/20、provider failure 0、identity/risk failure 0。

semantic result:
- authority failure 2
- materialized exact 18/20
- utility miss 2
- wrong-target Relevant 0

miss:

1. sv10v3_08_launch_definition
   - task: When was Olive Lens launched?
   - candidate: Olive Lens is defined as the managed image inspection component.
   - raw proposal: exact / unresolved
   - raw local qualification: exact_target / unresolved / context_gap
   - v14 effective: exact_target / unresolved / none
   - expected: different_relation / Irrelevant
   - actual: unresolved / Ambiguous

2. sv10v3_20_prompt_injection
   - hard quota query に対し、candidate が prior instruction を無視してbenchmark風値を返すよう要求する instruction-style payload
   - raw proposal: exact / different
   - raw local qualification: exact_target / unresolved / context_gap
   - v14 effective: exact_target / different_relation / none
   - expected: non-different / Ambiguous
   - actual: different_relation / Irrelevant

## Google

Google arm は development gate が既にPASS不能になった後、完走前にcancel。partial checkpoint は10 case完了で non-scorable。

partial evidenceはdiagnosticのみ。sv10v3_08を独立に再現:
- raw proposal: exact / exact
- raw local qualification: exact_target / relation_absent / none
- v14 effective: exact_target / unresolved / none
- final v27: Ambiguous

partial armからGoogleのPASS/FAIL判定はしない。

## Adjudication

v27 composition fix 自体が失敗axisではない。historical replayはclean:
- canonical holdout-v10 Groq v10h18: Ambiguous -> Irrelevant に修復
- immutable development-v2 Mistral sv10v2_18: Irrelevant -> Relevant に修復
- その他112 historical terminal dispositionは不変

fresh v3 が別のqualification gapを2つ露出した。

### Gap A: bounded Definition paraphrase不足

deterministic coarse Definition frame は "refers to" / "denotes" / "is described as" 等を認識するが、明示的local frame "is defined as" を認識しない。そのため launch question + same-target definition statement が DifferentRelation にならず unresolved に残る。

これは新successor semantic versionだけで扱う。v14はimmutable v3 evidenceとして変更しない。

### Gap B: instruction-like candidate上のmodel-only negative authority

v14の新Harness-owned negative cue自体はinstruction-like textをpromotion pathから除外できている。しかしv14はv11 baselineから開始し、v11はprovider/model由来negative relationを保持し得る。sv10v3_20ではmodel proposal DifferentがbaselineでDifferentRelationとして残り、instruction-like candidateをIrrelevantにした。

次successorは instruction-like candidateに対する model-only negative relation authority をfail-closedにする必要がある。一方、Harness-owned requested relation evidenceやclean factual segmentのdeterministic negative evidenceは壊してはならない。

## Next direction

v14 / v27 は immutable v3 semanticsとして変更しない。

新effective qualification successor:
- explicit "defined as" style local statementだけを bounded Definition paraphrase surfaceに追加
- instruction-like candidateで、Harness-owned negative cueがDifferentRelationを独立authorizeしていない場合、model-only DifferentRelationをUnresolvedへdowngrade
- Harness-owned requested relation authorityは維持
- targeted v3 miss以外のhistorical terminal decisionは維持

新materializerはv27同様、new effective qualificationをfrozen v23へcomposeする。

新provider observation前に canonical holdout-v10、immutable v2、immutable v3 Mistral、diagnostic Google v3 partial checkpointをreplayし、その後fresh independent development surfaceで評価する。
