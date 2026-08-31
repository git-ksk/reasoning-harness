# Semantic successor R4 independent evaluation

Issue #59 R4 evaluates the frozen `cross-model-selective-abstention-r3b-v1` candidate on a new observation-free holdout. This document and the holdout-v4 corpus are frozen before any R4 provider call.

## Frozen candidate

The primary R4 candidate is the provider-neutral cross-model unanimity mechanism implemented by `cross-model-selective-abstention-r3b-v1`, instantiated for this independent study with exactly two sources:

- `google:gemini-3.5-flash-lite`
- `mistral:ministral-8b-latest`

Both sources receive the same R2 `decision_note_object` semantic/materialization contract, the same fixture, token budget, and matched seed. Model identity may affect adapter mechanics only. A source disagreement is a risk signal that can only escalate the combined soft decision to `abstain`; agreement does not create truth, evidence, verification authority, hard findings, epistemic promotion, or verdict authority. Majority voting is forbidden.

The R4 measurement uses five matched seeds (`5000` through `5004`) and a 512-token output budget. `disagreement_only` is the primary candidate policy. `complete_unanimity` is reported as an operational sensitivity analysis, not substituted after observation.

## Frozen adoption gate

The canonical gate is the strict conjunction of the two pre-observation Issue #59 declarations. It was reconciled before any R4 provider run and cannot be weakened after observation.

The candidate passes R4 only if every condition below is met without changing the contract, source set, thresholds, corpus, or labels after provider observation:

- both sources complete all 140 calls (28 fixtures x 5 trials) with 100% protocol completion and all five combined trials complete;
- aggregate combined precision >= 0.95 and aggregate combined recall >= 0.95;
- every trial has precision >= 0.90 and recall >= 0.90;
- aggregate ambiguous abstention >= 0.85 and every trial ambiguous abstention >= 0.80;
- aggregate overall decision coverage >= 0.50 and every trial decision coverage >= 0.45;
- aggregate clear-case coverage over positive+negative fixtures >= 0.90 and every trial clear-case coverage >= 0.85;
- no positive/negative fixture produces both assertive polarities (`finding` and `no_finding`) across successful source/seed probes, and combined trial decisions do not oscillate between those polarities;
- disagreement may only preserve a unanimous soft decision or escalate to `abstain`; agreement or vote count cannot create truth, trusted evidence, verification receipts, hard findings, epistemic promotion, or verdict authority;
- every source receives the same semantic decision guidance, R2 ownership contract, and canonical representation; provider/model-specific semantic prompt branches are forbidden;
- all deterministic hard-verifier, resolution, validation, and authority regressions remain green.

`disagreement_only` is the frozen primary policy. `complete_unanimity` is sensitivity analysis only and cannot replace it after observation. External provider unavailability or quota exhaustion leaves R4 operationally incomplete; it must not be converted into a semantic pass or failure score. A failed gate rejects the candidate rather than tuning it against holdout-v4.

Passing this gate validates R3b only as an independently supported **optional configuration for the frozen two-source set**. It does not automatically replace the single-model `soft-semantic-v3` default and does not establish arbitrary N-source equivalence.

## Independent holdout-v4 freeze

`fixtures/semantic-judges-holdout-v4/` contains 28 new observation-free cases authored before the first R4 provider call: seven per diagnostic kind, with two positive, two negative, and three intentionally ambiguous cases per kind (8 positive, 8 negative, 12 ambiguous total).

Fixture IDs, request IDs, and exact request payloads must be unique relative to calibration and historical holdouts. `recorded_observations` must remain empty. Holdout-v1/v2/v3 remain historical diagnostic evidence and are not tuning data for this successor.

## Frozen R4 result: rejected

Run `33371523453` evaluated frozen main `55dbda5e71e83bdec95bf4495f65354ca301ef34`. The canonical gate was recorded on Issue #59 at `08:08:47Z`, before the run was created at `08:08:54Z`; PR #71 synchronized the same gate to the repository before artifact inspection.

Both sources completed 140/140 calls with zero operational failures. Under the primary `disagreement_only` policy:

- precision: `1.000` — pass;
- recall: `1.000` — pass;
- fixture-collapsed ambiguous abstention: `0.8333` — **fail** vs `>=0.85`;
- decision coverage: `0.6071` — pass vs `>=0.50`;
- clear-case coverage: `0.9375` — pass vs `>=0.90`.

Per-trial ambiguous abstention was `0.5833`, `0.7500`, `0.8333`, `0.7500`, and `0.6667`; four of five trials fail the frozen `>=0.80` threshold. Their mean is `0.7167`, but that mean is not the frozen fixture-collapsed aggregate metric. Per-trial precision/recall, overall coverage, and clear-case-coverage thresholds pass.

The independent labelled-polarity gate also fails. On `v4h-03-contradiction-negative`, Gemini returned `no_finding` for all five seeds while Ministral returned `finding` for all five. Cross-model disagreement makes the combined output safely `abstain`, but the source/seed assertive-polarity stability requirement is violated.

R3b also retains its structural limitation: if both sources make the same assertive decision, disagreement cannot expose the risk. Several ambiguity-labelled cases are assertive on individual trials; notably `v4h-13` and `v4h-20` are `finding` from both sources on every seed.

### Post-observation holdout-spec audit

A static audit against the already-frozen semantic decision guidance found two label/spec conflicts in holdout-v4:

- `v4h-13` explicitly states that backup frequency is not supplied. Under the frozen `unsupported_premise` rule, affirmative absence of support is a `finding`; `abstain` is reserved for partial, unbound, or uncertain support.
- `v4h-20` explicitly states that a simultaneous garbage-collector change was not isolated. Under the frozen `causal_gap` rule, explicit confounding/lack of directional isolation is a `finding` condition.

These issues were found only after provider observation. The corpus and labels therefore remain unchanged: holdout-v4 must not be post-hoc repaired, relabelled, or rerun as a corrected independent test. The spec conflicts make v4 imperfect diagnostic evidence, but they do not rescue the candidate because the predeclared per-trial uncertainty gate and labelled-polarity gate fail independently.

**Decision:** R3b is not adopted as an independently validated successor. Runtime `soft-semantic-v3` remains unchanged. Holdout-v4 and run `33371523453` are frozen diagnostic history and cannot be tuning data. A future successor requires fresh calibration-only research, a pre-observation fixture-label/spec review gate, and a newly frozen holdout-v5 for adoption testing.
