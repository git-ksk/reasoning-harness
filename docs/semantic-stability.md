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

## Measured R3 calibration result

The first 18-fixture single-trial R3 representation study produced two distinct regimes. Gemini 3.5 Flash-Lite had two ambiguous fixtures with cross-representation disagreement; unanimity-based selective abstention escalated both to `abstain`, yielding precision/recall 1.0, ambiguous abstention 1.0, and decision coverage 0.6111. Ministral 8B was 18/18 protocol-complete under all three R2 representations and produced identical decisions across them, leaving ambiguous abstention at 0.5714.

The Mistral result is a stable-miscalibration/self-consistent-error case: seed and representation agreement do not imply correctness or adequate uncertainty handling. R3 therefore characterizes a useful but bounded detector rather than a complete reliability mechanism.

## R3b cross-model risk

R3b adds a separate optional risk axis for deployments that have more than one model/provider available. Every source receives the same R2 semantic/materialization contract and canonical `decision_note_object` representation. Model identity may affect adapter mechanics only; it cannot select a different semantic prompt or decision rule.

Cross-model outputs are probes, never votes. If successful sources disagree, the existing unanimity evaluator conservatively returns `abstain`. If all sources agree, the soft decision may be preserved but gains no additional authority. Operationally missing sources remain a separate risk signal and can be handled by the stricter complete-unanimity candidate.

The CLI accepts N distinct `provider:model` sources. The initial GitHub Actions surface is intentionally bounded to two sources for the first calibration study. This direction is motivated by Tan et al., *Too Consistent to Detect: A Study of Self-Consistent Errors in LLMs* (EMNLP 2025, DOI 10.18653/v1/2025.emnlp-main.238), which shows that self-consistent errors are difficult for same-model consistency detectors and that cross-model evidence can provide an orthogonal signal.

## R3b repeated calibration and R4 handoff

The five-seed all-calibration R3b run (`33368618724`) completed 180/180 provider calls. Cross-model disagreement was limited to four ambiguous fixtures: three causal ambiguity cases disagreed on every seed, while one contradiction-binding ambiguity case disagreed on three of five seeds. No positive or negative fixture disagreed on any seed. The combined `disagreement_only` result retained precision/recall 1.0, ambiguous abstention 1.0, decision coverage 0.6111, and clear-case coverage 1.0.

This is sufficient to advance to an independent test, not to claim general correctness. The R4 thresholds and candidate identity were frozen before the first holdout-v4 provider observation; see `semantic-successor-r4.md`.
