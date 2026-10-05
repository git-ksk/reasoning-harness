# Engine 0.6 candidate: evidence-target relevance calibration v5

Status: frozen v4 と bounded Google model diagnostic の後継となる fresh/unobserved calibration。v1-v4 は immutable historical evidence として保持し、rerun/rescore しない。

## v5 が必要な理由

frozen v4 run `35998574508` では Mistral が semantic/operational gate を完全通過した一方、Google は Harness の 30,000 ms assessment deadline で operationally incomplete だった。

- Mistral: 26/26 operational、materialized disposition 26/26 exact、correctness 0、utility 0。
- Google: 23/26 operational。3 case が約 30,001 ms で typed `assessment_timeout`。
- Google 完了23 caseの latency median は 1,175 ms、最大は 29,703 ms。
- 後続の5-case Google model diagnostic run `36000374933` は v4 semantics と30,000 ms budgetを維持。 `gemini-3.5-flash-lite` は5/5完走、provider retry 0、最長caseは27,389 msだった。`gemini-3.1-flash-lite` はoperationally weakerだったためcanonical modelへ変更しない。

したがって30秒deadlineが観測tail latencyに近すぎることが主なoperational gapであり、semantic contract、materializer、canonical model、pacing、adapter retry policyを変更する根拠はない。

## v5 の変更

calibration policyとして変更するのは1点だけ。

- case単位 assessment elapsed budget: **60,000 ms**

以下は不変。

- model-facing contract: `reason-evidence-relevance-binding-proposal-v2`
- Harness materialization: `target-evidence-relevance-binding-materialization-v3`
- max model calls: 2
- max output tokens: 192
- Google canonical: `gemini-3.5-flash-lite`
- Mistral canonical: `ministral-8b-latest`
- Actions上のGoogle request-start pacing: 6,000 ms、inter-case delay 6,100 ms
- provider adapterのbounded retry/backoff
- 26 semantic families と expected proposal/disposition

Google adapterはprovider attempt最大4回。短期rate-limit retryはRetry-Afterまたは10/20/40秒、transient 5xxは2/5/10秒でbounded。60秒のHarness deadlineはadapterの最悪retry envelope全体を隠すための値ではなく、実測latencyのheadroomと限定的retry recoveryだけを許し、継続的provider instabilityはoperational failureとして表面化させる。

## Fresh identity

- suite: `evidence-relevance-calibration-v5`
- issue: #462
- cases: 26
- seed: `4625605`
- status: `fresh_unobserved_calibration`
- production motivating incident: tuningから除外

v5はcalibration comparability維持のためv4 semantic corpusを引き継ぐ。manifest上のpolicy deltaは `max_elapsed_ms: 60000` のみで、expected binding/dispositionは変更しない。

## Acceptance

canonical両armで以下を必須とする。

- successful provider cases: 26/26
- failed provider cases: 0
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0

proposal exact accuracyはdiagnostic。release gateのsemantic判定はHarness-materialized dispositionを主とする。

frozen v5 workflowはrerunを拒否する。first observationがFAILならv5はimmutable failed evidenceとして保持し、次はv6等のfresh successor identityを作る。

v5 PASSかつ#462 semantic implementation freeze前にindependent holdoutをauthorしない。
