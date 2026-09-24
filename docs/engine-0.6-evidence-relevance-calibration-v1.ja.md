# Engine 0.6 candidate: evidence-target relevance calibration v1

Status: fresh unobserved calibration作成済み。core relevance contractとdeterministic materialization testはPASS。live model-backed observationはまだ実施していない。

## Identity

- issue: #462
- suite: `evidence-relevance-calibration-v1`
- status: `fresh_unobserved_calibration`
- corpus: `fixtures/evidence-relevance-calibration-v1/manifest.json`
- cases: 26
- production motivating incident: tuningから除外

## Contract under test

modelが提案できるのは`relevant` / `irrelevant` / `ambiguous`だけ。Harness-owned policyがtarget identity、alias、relation kind、strict-vs-semantic identity requirement、assessment budgetを固定する。materializationはunsafeなmodel-relevantをblockできるが、evidence authorityは作れない。

strict entity identityではcanonical/alias anchorをcontent-bearing material内に要求する。canonical URLやnavigation/footer一致は観測するがself-authorizeできない。policyがsemantic equivalenceを明示許可する場合のみ、non-lexical paraphraseやcross-lingual relevanceをadvisory semantic pathでaccept可能。

model output欠落時はdeterministically `ambiguous`へfallbackし、`relevant`にはしない。

## Calibration families

Positive: exact product name、acronym/expanded alias、semantic paraphrase、titleにidentity/bodyにrelation、section跨ぎ、target termを含まないURL、日本語target+英語source、英語target+日本語alias、structured metadata+body、stale-but-relevant、multi-section support。

Negative: same service別feature、sibling product、navigation/footerだけtarget mention、local supportの無いbroad landing、unrelated announcement、comparison-only mention、same entity別relation、relevance/trustをself-declareするprompt injection。

Ambiguous: unknown rename、partial identity、mixed multi-product material、conflicting section、insufficient local passage、URL-only identity。

## Acceptance metrics

別々にreportする。

- wrong-target relevance retention — hard gate 0
- expected-relevant materialのfalse rejection
- expected-relevant materialがambiguousに残る率
- ambiguous disposition rate
- model proposal exact accuracy
- materialized disposition exact accuracy
- deterministic safety override
- model calls / provider attempts / tokens / latency
- provider/model operational failureとsemantic failureの分離
- positive semantic/cross-lingual caseでsimple lexical-overlap baselineとの比較

always-relevant policyはcorrectness FAIL。always-irrelevant/ambiguous policyはutility PASS不可。

## Current deterministic validation

- core relevance unit: 14 PASS
- calibration manifest materialization: 26/26 exact expected disposition
- production motivating productはcorpusに含まない
- negative-family expected caseはrelevantへmaterializeされない
- core clippy `-D warnings`: PASS
- `git diff --check`: PASS

## Next sequence

1. GitHub repository secretsを使うbounded live calibration runnerを実装
2. model proposalとHarness-materialized assessmentを別記録
3. simple lexical baselineとsemantic pathを比較
4. 必要ならfresh calibrationだけでtuning
5. #462 semantics/thresholdをfreeze
6. freeze後に初めてindependent holdoutをauthor
7. first observation前にholdoutをfreeze
8. correctness / utility両gate PASS後に#462をpromote
