# Semantic judge selective-abstention stability research

Issue #59 R3 studies whether bounded disagreement can be used as a **risk signal** that conservatively escalates a soft semantic decision to `abstain`. It is calibration-only research. It cannot create trusted evidence, hard findings, verification receipts, epistemic promotion, or verdict authority.

## Why seed stability alone is insufficient

The repeated R2 materialization study separated two failure modes:

- Gemini 3.5 Flash-Lite was protocol-complete but one ambiguous fixture changed across seeds.
- Ministral 8B was 90/90 protocol-complete under the R2 decision-owned/model, binding-owned/harness contract and completely seed-stable, while ambiguous abstention remained 0.5714 on every trial.

A model can therefore be **stably assertive**. R3 must measure more than seed disagreement.

## Probe axes

The first R3 surface combines two provider-neutral axes:

1. seed perturbation;
2. information-equivalent R2 output representations.

The R2 ownership contract and semantic decision guidance stay fixed. Only the model-facing representation changes:

- `decision_note_object`;
- `compact_decision_note_object`;
- `nested_decision_note_object`.

Decision labels remain canonical. Model-owned `kind`, `target`, provenance, evidence, or authority fields remain forbidden in every representation.

## Risk assessment

For one fixture the harness records every configured probe as either a valid soft decision or an operationally incomplete observation. Operational failure is never converted to `no_finding` or any other semantic claim.

The assessment records independently:

- decision disagreement;
- operational incompleteness;
- no successful observation.

No vote count is interpreted as truth.

## Selective candidates

R3 reports two predeclared candidates:

- `disagreement_only`: if successful probes disagree, return `abstain`; operational incompleteness remains visible but does not itself override an otherwise unanimous successful decision.
- `complete_unanimity`: require every configured probe to succeed and agree; disagreement or missing probes conservatively returns `abstain`.

Both candidates report precision, recall, decision coverage, ambiguous abstention, risk-fixture count, and abstention escalation count. A candidate that gains abstention merely by destroying useful coverage does not pass.

These are research policies, not runtime defaults.

## Execution design

`reason-stability-study` interleaves the three R2 representations within each fixture/trial and rotates their order with `(fixture_index + trial) mod 3`. This reduces provider-time/order drift. The workflow starts with a causal positive/negative/ambiguous triad before any full calibration matrix.

Only this checkout's canonical `fixtures/semantic-judges` directory is accepted. Historical holdouts remain blocked from R3 tuning.
