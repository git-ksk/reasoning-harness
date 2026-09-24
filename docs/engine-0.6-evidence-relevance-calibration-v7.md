# Engine 0.6 candidate: evidence-target relevance calibration v7

Status: fresh successor after frozen v6 operational failure. v1-v6 remain immutable historical evidence.

## Purpose

v7 tests the unchanged #462 relevance semantics under a precommitted two-provider required gate that can produce a valid canonical observation. It does not tune semantics after v6.

Required canonical arms:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

The provider-role rationale is frozen in `engine-0.6-evidence-relevance-provider-gate-v7.md` before any v7 live observation.

## Unchanged semantic/evaluation surface

- model-facing contract: `reason-evidence-relevance-binding-proposal-v2`
- Harness materialization: `target-evidence-relevance-binding-materialization-v3`
- strict Harness-owned identity floor
- 26 cases and all expected labels unchanged from v6
- elapsed budget: 60,000 ms per case
- max model calls: 2
- max output tokens: 192
- run-level circuit: two consecutive operational provider failures
- Mistral model: `ministral-8b-latest`

## Groq operational policy

- model: `openai/gpt-oss-120b`
- inter-case delay: 2,200 ms
- adapter minimum request interval: 2,100 ms
- token pacing ceiling used by the established repository workflow policy: 8,000 tokens/minute
- rate-limit telemetry enabled
- provider structured-output mode remains best-effort JSON Schema; Harness parsing/materialization remains authoritative
- runner adds no retry layer

The adapter also records a started provider attempt on transport failure so attempt telemetry remains complete when the provider returns the failure rather than being cancelled by the outer Harness deadline.

## Fresh identity

- suite: `evidence-relevance-calibration-v7`
- issue: #462
- cases: 26
- seed: `4625607`
- status: `fresh_unobserved_calibration`
- production motivating incident excluded from tuning

The v7 corpus is byte-equivalent at the case level to v6; only the suite identity changes. This is calibration comparability, not an independent holdout.

## Acceptance

Both Mistral and Groq required arms must independently satisfy on the first/only frozen observation:

- planned/completed: 26/26
- operational abort: none
- failed provider cases: 0
- wrong-target / unsafe relevance admission: 0
- false relevance rejection: 0
- expected-relevant left ambiguous: 0
- utility miss: 0

Proposal exact accuracy and latency are diagnostic. Materialized disposition is the semantic release gate.

If v7 fails, it remains immutable FAIL and is not rerun. Independent holdout authoring remains blocked until a fresh canonical calibration passes.
