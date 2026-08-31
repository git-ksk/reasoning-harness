# Semantic successor R4 independent evaluation

Issue #59 R4 evaluates the frozen `cross-model-selective-abstention-r3b-v1` candidate on a new observation-free holdout. This document and the holdout-v4 corpus are frozen before any R4 provider call.

## Frozen candidate

The primary R4 candidate is the provider-neutral cross-model unanimity mechanism implemented by `cross-model-selective-abstention-r3b-v1`, instantiated for this independent study with exactly two sources:

- `google:gemini-3.5-flash-lite`
- `mistral:ministral-8b-latest`

Both sources receive the same R2 `decision_note_object` semantic/materialization contract, the same fixture, token budget, and matched seed. Model identity may affect adapter mechanics only. A source disagreement is a risk signal that can only escalate the combined soft decision to `abstain`; agreement does not create truth, evidence, verification authority, hard findings, epistemic promotion, or verdict authority. Majority voting is forbidden.

The R4 measurement uses five matched seeds (`5000` through `5004`) and a 512-token output budget. `disagreement_only` is the primary candidate policy. `complete_unanimity` is reported as an operational sensitivity analysis, not substituted after observation.

## Frozen adoption gate

The candidate passes R4 only if every condition below is met without changing the contract, source set, thresholds, corpus, or labels after provider observation:

- both sources complete all 140 calls (28 fixtures x 5 trials) with 100% protocol completion;
- all five combined trials are operationally complete;
- aggregate combined precision >= 0.95 and aggregate combined recall >= 0.95;
- every trial has precision >= 0.90 and recall >= 0.90;
- aggregate ambiguous abstention >= 0.85 and every trial ambiguous abstention >= 0.80;
- aggregate decision coverage >= 0.50 and every trial decision coverage >= 0.45, preventing trivial always-abstain success;
- no labelled fixture directly oscillates between `finding` and `no_finding` across combined trial decisions;
- no provider/model-specific semantic branch is introduced and all deterministic hard/resolution safety tests remain green.

External provider unavailability or quota exhaustion leaves R4 operationally incomplete; it must not be converted into a semantic pass or failure score. A failed semantic gate rejects the candidate rather than tuning it against holdout-v4.

Passing this gate makes R3b eligible as an independently supported research successor. It does not by itself require a mandatory dual-provider production default: runtime cost, latency, provider availability, and conditional-escalation policy remain separate product/runtime decisions.

## Independent holdout-v4 freeze

`fixtures/semantic-judges-holdout-v4/` contains 28 new observation-free cases authored before the first R4 provider call: seven per diagnostic kind, with two positive, two negative, and three intentionally ambiguous cases per kind (8 positive, 8 negative, 12 ambiguous total).

Fixture IDs, request IDs, and exact request payloads must be unique relative to calibration and historical holdouts. `recorded_observations` must remain empty. Holdout-v1/v2/v3 remain historical diagnostic evidence and are not tuning data for this successor.
