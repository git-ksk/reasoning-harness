# Engine 0.6 candidate: evidence-target relevance calibration v1

Status: fresh unobserved calibration作成済み。core relevance contract、bounded live calibration runner、lexical baseline、deterministic materialization validationを実装済み。live model-backed observationはまだ実施していない。

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

## Live calibration runner

実装済みの `reason-evidence-relevance-study` runnerはexact 26-case calibration directoryへbindし、別suite/status/issue identityをrejectする。`--validate-only`はprovider call 0でcorpus/materializationをdeterministic validationする。canonical observationは`--fixture`を指定せず26 caseすべてを1回だけ評価する。

各caseでadvisory model proposalとHarness-materialized relevance assessmentを別々に記録する。さらに意図的に単純なlexical baseline、deterministic safety override、model-call数、provider attempt数、token usage、latency、fallback利用、typed operational failure classを記録する。raw model responseとcredentialは保存しない。

v1のHarness-owned assessment budgetは2 model calls、1 callあたりmax output 192 tokens、caseあたりabsolute assessment 15,000 ms。primary JSON-Schema call 1回と最大1回のbounded JSON-object fallbackだけを許可する。fallbackはmodel-call budgetを超えられず、primary+fallbackは1つのabsolute elapsed deadlineを共有する。adapter内部HTTP retryは別の`provider_attempts`として観測する。

first canonical live observationはlocal credentialではなくGitHub Actions repository secretsを使う。frozen armはMistral `ministral-8b-latest` とGoogle `gemini-3.5-flash-lite`。workflowは`.github/workflows/engine-0.6-evidence-relevance-calibration-v1-live.yml`。first-observation surfaceはcredential読込前にchecksum+freeze tagで固定し、そのfreeze identityのworkflow rerunは禁止する。

## Next sequence

1. GitHub repository secretsを使うbounded live calibration runnerを実装
2. model proposalとHarness-materialized assessmentを別記録
3. simple lexical baselineとsemantic pathを比較
4. 必要ならfresh calibrationだけでtuning
5. #462 semantics/thresholdをfreeze
6. freeze後に初めてindependent holdoutをauthor
7. first observation前にholdoutをfreeze
8. correctness / utility両gate PASS後に#462をpromote
