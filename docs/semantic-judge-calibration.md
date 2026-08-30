# Soft semantic-judge calibration

Issue #13 defines how model-backed semantic diagnostics may be measured before they are allowed into broader research workflows.

The central rule is simple: **a semantic judge is a soft observer, never a correctness authority**.

## Contract

`SoftDiagnosticJudge` receives a typed `SoftJudgeRequest` and returns only a `SoftJudgeOutput` payload. The harness attaches the adapter-owned `SoftJudgeIdentity` and request ID to form `SoftJudgeObservation`. Model output therefore cannot choose its own provenance. The output has one of three decisions:

- `finding`
- `no_finding`
- `abstain`

A `finding` contains only `SoftSemanticFinding { kind, target, note }`. The type intentionally has no hard/soft strength switch, verification receipt, epistemic-state mutation, or final verdict field. There is no core conversion from a soft judge observation to an authority-bearing verifier result.

`SoftJudgeIdentity` records:

- stable judge ID;
- model ID;
- configuration ID.

Identity must remain consistent across a calibration run so disagreement is attributable instead of silently pooling different judge configurations.

## Diagnostic families

The initial provider-neutral request kinds are:

- contradiction;
- counterexample;
- unsupported premise;
- causal gap.

Targets are typed propositions, causal relations, claims, or inference edges. This is a discovery/calibration surface, not a semantic truth taxonomy.

## Calibration labels

The committed calibration corpus uses three labels:

- `positive`: a labelled finding is expected;
- `negative`: a labelled finding is not expected;
- `ambiguous`: the case is intentionally not treated as positive or negative ground truth.

Ambiguous cases are excluded from precision/recall confusion counts. They remain visible for decision coverage, disagreement, and abstention behavior.

## Precision, recall, and abstention

Per judge, the report records:

- finding / no-finding / abstain counts;
- decision coverage;
- true/false positive and true/false negative counts;
- precision where at least one positive prediction exists;
- recall where at least one positive-labelled case exists.

An abstention on a positive-labelled case is a missed detection and therefore contributes to false negatives/recall. An abstention on a negative-labelled case is not credited as a true negative. This prevents a judge from appearing accurate merely by abstaining broadly.

## Agreement

Two agreement views are reported.

### Pairwise categorical agreement

For each case, every pair of non-abstaining judge decisions is compared. The report preserves:

- comparable pairs;
- agreeing pairs;
- disagreeing pairs;
- total abstain votes;
- observed pairwise agreement.

Abstention is not majority-voted into another category.

### Nominal Krippendorff alpha

The report also computes nominal Krippendorff alpha over `finding | no_finding`, treating abstention as missing data. Unit coincidences are normalized by the number of non-missing ratings for that case so cases with more available judges do not receive quadratic weight.

Alpha is omitted when expected disagreement is zero, because reliability is then undefined rather than perfect by construction.

The alpha is a reliability statistic, not a verifier score and not a final harness correctness metric.

## Deterministic calibration corpus

`fixtures/semantic-judges/` contains nine offline cases:

- three positive;
- three negative;
- three ambiguous;
- contradiction, unsupported-premise, and causal-gap families;
- three recorded synthetic judge identities;
- deliberate disagreement and abstention.

The recorded judge identities are **calibration fixtures, not claims about real model performance**. Their purpose is to regression-test aggregation semantics without credentials or stochastic provider calls.

Run:

```bash
cargo run -p reasoning-harness-cli -- eval-judges fixtures/semantic-judges --format json
```

The committed fixture observations currently produce different precision/recall/coverage values per synthetic judge, non-zero disagreement, preserved abstentions, pairwise agreement below 1.0, and a chance-corrected alpha below pairwise agreement. Those values are test data only.

## Live studies

A live semantic judge can implement the same provider-neutral `SoftDiagnosticJudge` contract. Live studies are optional/manual and must preserve model/config identity, raw decisions, abstentions, and operational failures.

No live judge result may:

- create a `VerificationReceipt`;
- create a hard finding;
- mutate a claim to `known`, `supported`, or `contradicted`;
- decide `accept | reject | unknown`;
- become a trusted resolver merely because it is another model.

A future `ReasoningPolicy` may use calibrated soft findings as advisory triggers for evidence acquisition or deterministic verification, but the resulting authority must still come through the existing harness-owned boundaries.
