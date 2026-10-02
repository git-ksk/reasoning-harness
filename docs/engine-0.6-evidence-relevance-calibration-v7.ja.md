# Engine 0.6 candidate: evidence-target relevance calibration v7

Status: frozen v6 operational FAIL後のfresh successor。v1-v6はimmutable historical evidenceとして保持する。

## Purpose

v7は#462 relevance semanticsを変更せず、canonical observationを成立させられるよう事前固定した2-provider required gateで再評価する。v6後のsemantic tuningではない。

required canonical arm:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

provider-role rationaleはv7 live observation前に `engine-0.6-evidence-relevance-provider-gate-v7.ja.md` で固定する。

## semantic/evaluation surfaceで変更しないもの

- model-facing contract: `reason-evidence-relevance-binding-proposal-v2`
- Harness materialization: `target-evidence-relevance-binding-materialization-v3`
- strict Harness-owned identity floor
- v6と同じ26 case / expected labels
- elapsed budget: case単位60,000 ms
- max model calls: 2
- max output tokens: 192
- run-level circuit: operational provider failure 2件連続
- Mistral model: `ministral-8b-latest`

## Groq operational policy

- model: `openai/gpt-oss-120b`
- inter-case delay: 2,200 ms
- adapter minimum request interval: 2,100 ms
- 既存repo workflow policyに合わせたtoken pacing: 8,000 tokens/minute
- rate-limit telemetry enabled
- provider structured-outputはbest-effort JSON Schemaのまま。Harness parser/materializerがauthorityを持つ
- runner側retryは追加しない

transport failureをproviderが返した場合はstarted provider attemptを記録し、outer Harness deadlineによるcancelとのtelemetry差を維持する。

## Fresh identity

- suite: `evidence-relevance-calibration-v7`
- issue: #462
- cases: 26
- seed: `4625607`
- status: `fresh_unobserved_calibration`
- production motivating incidentはtuningから除外

v7 corpusはcase levelでv6と同一で、suite identityのみ変更する。これはcalibration comparability用でありindependent holdoutではない。

## Acceptance

first/only frozen observationでMistral/Groq両required armが独立に以下を満たすこと。

- planned/completed: 26/26
- operational abort: none
- failed provider cases: 0
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0

proposal exact accuracy / latencyはdiagnostic。semantic release gateはmaterialized disposition。

v7がFAILした場合はimmutable FAILとして保持しrerunしない。fresh canonical calibrationがPASSするまでindependent holdout authoringは引き続き禁止。
