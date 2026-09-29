# Engine 0.6 evidence relevance independent holdout v1 result

Status: immutable canonical **FAIL**。

- Freeze tag: `engine-0.6-evidence-relevance-holdout-v1-freeze`
- Freeze commit: `5fda8675e642ee98de57e2c4d69e34a5622fa485`
- Canonical run: `36495389012`、attempt 1 only
- Cases: 26（Relevant 8 / Irrelevant 10 / Ambiguous 8）
- Required provider: Mistral + Groq + Google
- Annotation protocol: `evidence-relevance-effective-qualification-v23`
- Materialization: v16
- Operational: 3 required armすべて26/26完走、provider jobはSUCCESS
- Final gate: FAIL

この結果はimmutable。rerun / rescore / relabel / expected label変更 / freeze tagの再作成・移動 / このholdout identityのPASSへの読み替えは禁止。

## Required Mistral

`ministral-8b-latest` は26/26完走。

- authority-qualified effective qualification: 24/26
- materialized exact: 24/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 2
- total tokens: 45,544
- latency p50/p95/max: 1,265 / 2,077 / 4,142 ms

terminal miss 2件はいずれも安全側。

- `h10_negative_sibling_same_relation`: expected Irrelevant -> observed Ambiguous
- `h13_negative_unrelated_announcement`: expected Irrelevant -> observed Ambiguous

wrong-target materialをRelevantへ通したケースはない。

## Required Groq

`openai/gpt-oss-120b` は26/26完走。

- authority-qualified effective qualification: 26/26
- materialized exact: 26/26
- wrong-target relevance retention: 0
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 0
- total tokens: 65,244
- latency p50/p95/max: 10,720 / 11,348 / 11,586 ms
- provider-arm latch / retry waitなし

frozen semantic / operational contractを完全に満たした。

## Required Google

`gemini-3.5-flash-lite` は26/26完走。

- authority-qualified effective qualification: 25/26
- materialized exact: 24/26
- wrong-target relevance retention: **1**
- false relevance rejection: 0
- Relevant left Ambiguous: 0
- utility miss: 1
- total tokens: 47,067
- latency p50/p95/max: 6,760 / 8,028 / 11,862 ms

correctness hard gateを落とした決定的ケースは `h14_negative_comparison_only_mention`。

frozen case:

- requested target/relation: `Umber DB` / pricing
- candidate title: `Violet DB pricing`
- candidate excerpt: `Unlike Umber DB, Violet DB charges by provisioned shard-hour. This page lists only Violet DB prices.`
- expected proposal: target=different / relation=exact
- expected local qualification: distinct_target / requested_relation / none
- expected disposition: Irrelevant

Google observed:

- proposal: exact / exact
- raw local qualification: distinct_target / requested_relation / none
- effective local qualification: exact_target / requested_relation / none
- final disposition: **Relevant**
- reasonに `harness_canonical_name_anchor`、`positive_target_local_binding_confirmed`、`model_relevant`

raw verifierはdistinct targetを正しく識別していた。失敗を作ったのはHarness-owned compositionで、比較/context内のliteral canonical-name occurrenceをpositive target identityとして強く扱い、raw `distinct_target` を上書きした。

`h13_negative_unrelated_announcement` はwrong-target Relevantには進まず、安全側Ambiguousに留まった。

## Root cause

effective qualification v1では、`distinct_target` がHarness name anchorより優先されるのは、deterministic distinctness signalに加えてadvisory proposalもtarget=differentへ一致した場合だけだった。proposalがtarget=exactならこの枝を通らず、次の `has_harness_anchor` がidentityを `exact_target` へ昇格した。

effective qualification v2/v3はrelation-axis conflictを修正したが、このidentity-axis precedence defectは未処理だった。materialization v16はその後exact target + requested relationを受け、positive pathへ進んだ。

generic safety gapは次の通り。

> candidate内にcanonical target nameが出現することはidentity metadataであり、そのcandidateのsubstantive propositionがtarget所有である証明ではない。

requested relationの語が存在することもtarget ownership evidenceではない。

## Final gate

precommitted required gate:

- required operational completeness: PASS
- required correctness: **FAIL**
- required utility: **FAIL**
- required materialization: **FAIL**
- required qualification: **FAIL**

#462 acceptanceにはindependent frozen holdoutでwrong-target relevance admission 0が必要なため、Issueはopenのまま維持する。

## Successor postmortem replay

immutable v1 observationをrescore / rewriteしない。別versionのsuccessor semanticsとしてeffective qualification v4 + materialization v17をrecorded observationへ適用し、regression / postmortem用途でのみ確認する。

immutable canonical artifact由来のreplay fixture:

- `fixtures/evidence-relevance-holdout-successor-v2/v23-observation-replay.json`
- `fixtures/evidence-relevance-holdout-successor-v2/holdout-v1-observation-replay.json`

現在のsuccessor replay:

- calibration v23: 48 x 3 = 144 provider observationすべてexpected authority/materializationを維持、regression 0
- holdout v1 Groq: 26/26 exact維持
- holdout v1 Mistral: 24/26 exact維持、wrong-target Relevant 0維持
- holdout v1 Google: 25/26 exactへ改善し、frozen h14 observationはsuccessor semantics上だけRelevant -> Irrelevantへ修正、wrong-target Relevantはreplay上0
- 3 providerともRelevant utility missは0維持

これはsuccessor semanticsのevidenceであり、holdout v1をPASSへ変更するものではない。

詳細は [successor v2 design](engine-0.6-evidence-relevance-holdout-successor-v2.ja.md)。
