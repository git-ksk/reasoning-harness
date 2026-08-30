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

Ambiguous cases are excluded from precision/recall confusion counts. They remain visible for decision coverage, disagreement, and abstention behavior. `ambiguous_abstention_rate` is reported separately so a judge cannot look strong on labelled precision/recall while aggressively converting intentionally uncertain cases into findings.

## Precision, recall, and abstention

Per judge, the report records:

- finding / no-finding / abstain counts;
- decision coverage;
- ambiguous-case abstention count and rate;
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
## Model-backed semantic discovery (#33)

The same typed request/output contract can now be driven by any existing provider-neutral `ModelAdapter`. The harness builds a structured-output request for `SoftJudgeOutput`, attaches judge/model/configuration identity itself, and validates the returned decision against the original requested kind and target. Model output cannot choose its own provenance.

The primary request uses JSON Schema structured output. If the adapter reports that schema mode is unsupported, or if the first response is not a valid typed decision, the harness may retry once using generic JSON-object mode plus the serialized schema. A malformed fallback fails closed as an operational/protocol failure; it is never converted into `no_finding`. Model-backed executions expose a harness-owned `fallback_reason` with `not_needed`, `primary_json_schema_unsupported`, or `invalid_primary_structured_output`. This telemetry describes only the harness primary→fallback protocol; provider-internal HTTP retries remain separate. Raw model output is not retained for fallback classification.

`reason eval-judges` supports optional live execution with `--provider`, `--model`, and `--trials`. Recorded mode remains unchanged. For live repeated trials:

- one failed fixture makes that entire trial operationally incomplete;
- incomplete trials are excluded from precision/recall/coverage/abstention stability distributions;
- provider/protocol failures are reported separately from semantic decisions;
- model findings remain soft regardless of observed precision or agreement;
- the ordinary `reason eval` correctness denominator is unchanged.

The manual live workflow can run the calibration corpus using secret-isolated Mistral, Google, or NVIDIA credentials. Repeated live results are research observations, not correctness authority. The first repeated Mistral study and the v1/v2 prompt-sensitivity result are documented in [live soft semantic-judge study](live-semantic-judge-study.md). Because v2 was calibrated against the original nine cases, those results are not evidence of generalization.

## soft-semantic-v3 generic decision contract (#38)

The v3 calibration revision expands the calibration corpus from 9 to 18 cases using generic semantic patterns rather than frozen holdout-v1 facts. It adds clear semantic equivalence, ambiguous proposition binding, paraphrased premise support, explicit reverse-causal alternatives, partial/scoped intervention evidence, and counterexample applicability.

The decision boundary is intentionally asymmetric:

- `finding` requires the supplied context to affirmatively establish the requested diagnostic concern;
- `no_finding` requires the supplied context to affirmatively resolve or negate the concern, including semantic equivalence/paraphrase and clearly out-of-scope contrary cases;
- `abstain` remains the terminal result when binding, scope, applicability, or mixed/partial evidence prevents either conclusion.

For causal gaps specifically, explicit correlation-only, confounding, or an explicit viable reverse-causal alternative with undistinguished direction can establish a directional-support gap. Partial intervention evidence or incomplete scope does not automatically establish a gap; when adequacy remains unresolved, the required result is `abstain`.

This revision changes only the advisory semantic contract and configuration identity (`soft-semantic-v3`). It does not add any path from model output to evidence, verification receipts, hard findings, epistemic promotion, or verdict authority. Holdout-v1 remains frozen and is not used to evaluate v3.

## Independent holdout v1

Issue #36 adds `fixtures/semantic-judges-holdout/` as a separate 28-case, observation-free holdout corpus. It contains 11 positive, 8 negative, and 9 ambiguous cases across contradiction, unsupported-premise, causal-gap, and counterexample families. Causal-gap coverage is intentionally heavier.

The source corpus contains no recorded model observations. Labels are evaluator-owned and are not included in model requests. After the merge that introduces holdout v1, its fixture/request IDs, labels, targets, and contexts are frozen for the first live study. Provider results must not be used to tune the prompt against this holdout version; a later independently measured prompt revision requires a new holdout version rather than rewriting observed v1 cases.

## Independent holdout v2 freeze

`fixtures/semantic-judges-holdout-v2/` is the independent evaluation corpus for `soft-semantic-v3`. It contains 28 observation-free cases created after the generic v3 contract was calibrated but before any v3 provider result was observed on this corpus:

- 10 positive, 9 negative, and 9 ambiguous labels;
- 7 contradiction, 6 unsupported-premise, 9 causal-gap, and 6 counterexample cases;
- independent facts and surfaces covering semantic equivalence, binding/scope ambiguity, paraphrased premise support, reverse causality, confounding, temporal-only support, partial and mixed interventions, incomplete applicability, and counterexample scope.

The v2 source fixtures intentionally contain no recorded model observations. Labels are evaluator-owned and are not sent to the model. Once this corpus is merged into `main`, its fixture IDs, request IDs, labels, targets, tasks, and contexts are frozen for the first `soft-semantic-v3` live study. Any later prompt or contract revision that is informed by v2 results requires a new holdout version rather than editing this corpus.
