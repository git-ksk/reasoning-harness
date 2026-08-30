# Live soft semantic-judge study

Issue #33 validates the model-backed soft semantic-judge path with a repeated live provider study while preserving the rule that model output has no correctness authority.

## Study design

The study used the nine committed calibration cases under `fixtures/semantic-judges/`:

- three positive-labelled cases;
- three negative-labelled cases;
- three intentionally ambiguous cases;
- contradiction, unsupported-premise, and causal-gap diagnostic families.

The live model was `ministral-8b-latest` through the existing Mistral `ModelAdapter`. Both repeated studies used five trials, seed base `1000`, and 256 maximum output tokens per request. Provider credentials remained isolated in the manual GitHub Actions workflow.

These cases are a **calibration set, not a holdout set**. The v2 prompt was changed after observing v1 behavior on this same corpus. Therefore the v2 result measures protocol behavior after calibration; it is not unbiased evidence of semantic-judge generalization.

## v1: generic abstention guidance

GitHub Actions run `33307653357` executed 45 fixture calls across five complete trials.

Operational result:

- 45/45 successful fixture runs;
- 0 operational failures;
- 35,826 total tokens;
- 64 successful provider-generation attempts.

Complete-trial semantic distributions:

| metric | mean | min | max | stddev |
|---|---:|---:|---:|---:|
| precision | 1.000 | 1.000 | 1.000 | 0.000 |
| recall | 0.400 | 0.333 | 0.667 | 0.133 |
| decision coverage | 0.156 | 0.111 | 0.222 | 0.054 |
| abstentions per 9 cases | 7.6 | 7 | 8 | 0.490 |
| ambiguous abstention rate | 1.000 | 1.000 | 1.000 | 0.000 |

The model was operationally reliable but too conservative. It detected the positive contradiction 5/5, while the positive causal-gap case was 0/5 and the positive unsupported-premise case was only 1/5. All ambiguous cases abstained.

## v2: diagnostic-kind decision semantics

The v2 configuration (`soft-semantic-v2`) added provider-neutral decision semantics derived from the typed diagnostic contract. It defines what `finding`, `no_finding`, and `abstain` mean for contradiction, counterexample, unsupported-premise, and causal-gap requests without adding fixture-specific facts or any authority path.

GitHub Actions run `33307898636` repeated the same model/corpus/trial configuration.

Operational result:

- 45/45 successful fixture runs;
- 0 operational failures;
- 43,016 total tokens;
- 64 successful provider-generation attempts.

Complete-trial semantic distributions were identical across all five trials:

| metric | value |
|---|---:|
| precision | 1.000 |
| recall | 1.000 |
| decision coverage | 0.667 |
| abstentions per 9 cases | 3 |
| ambiguous abstention rate | 0.667 |

The labelled positive/negative behavior became much less conservative: all three positive cases were found in every trial, and two of the three negative cases received `no_finding` in every trial. The remaining negative contradiction case consistently abstained.

However, the causal ambiguous case was classified as `finding` in all five trials. Because ambiguous cases are intentionally excluded from precision/recall confusion counts, precision and recall alone would hide this behavior. #33 therefore adds explicit `ambiguous_abstentions` and `ambiguous_abstention_rate` metrics.

## Interpretation

The live study supports four narrow conclusions:

1. The provider-neutral model-backed soft-judge path is operationally viable: both five-trial runs completed 45/45 fixture calls with no provider/protocol failures.
2. Semantic behavior is materially prompt-sensitive even when the typed output/authority contract is unchanged.
3. Precision/recall and broad decision coverage are insufficient calibration metrics by themselves; ambiguous-case behavior must remain visible.
4. None of these observations creates hard authority. Every live decision remains a `SoftJudgeObservation` and can only trigger further evidence acquisition, deterministic verification, or review through existing policy boundaries.

The v2 numbers must **not** be presented as general model quality. The prompt was calibrated against the same nine cases used for measurement. The next semantic-quality study should use a separate holdout/expanded ambiguity corpus with paraphrases, mixed evidence, and unseen cases before comparing models or making reliability claims.


## Independent holdout v1

Issue #36 froze a separate 28-case, observation-free holdout corpus before any provider result was observed. The corpus contains 11 positive, 8 negative, and 9 ambiguous cases across contradiction, unsupported-premise, causal-gap, and counterexample families. The first live study ran from merged `main` commit `c50aa5b822307096b08dcdf63826cd3d40ad0f7d`; no holdout fixture or prompt change was made after observing the result.

GitHub Actions run `33314808691` evaluated `ministral-8b-latest` for five trials, producing 140 fixture calls.

Operational result:

- 140/140 successful fixture runs and 5/5 complete trials;
- 0 operational failures;
- 151,699 total tokens;
- 276,440 ms aggregate fixture latency;
- 210 successful provider-generation attempts;
- 70/140 successful runs used the harness JSON-object fallback path (`fallback_rate = 0.500`).

Complete-trial semantic distributions:

| metric | mean | min | max | stddev |
|---|---:|---:|---:|---:|
| precision | 0.909 | 0.909 | 0.909 | 0.000 |
| recall | 0.909 | 0.909 | 0.909 | 0.000 |
| decision coverage | 0.664 | 0.643 | 0.679 | 0.017 |
| ambiguous abstention rate | 0.778 | 0.778 | 0.778 | 0.000 |
| abstentions per 28 cases | 9.4 | 9 | 10 | 0.490 |

The independent corpus exposed several stable generic error classes that the calibration-set score did not show: semantically equivalent wording was overcalled as contradiction, reverse-causality uncertainty was treated as abstention rather than a directional causal-gap finding under the current label contract, and partial or incompletely scoped causal evidence was overcalled on two intentionally ambiguous cases. Some negative cases also remained conservatively undecided.

This result is evidence about `soft-semantic-v2` on this frozen holdout version only. It does not promote the model to correctness authority and does not justify a broad model ranking. Issue #38 tracks semantic calibration from the generic contract using the calibration corpus, while keeping holdout v1 frozen. Issue #39 separately tracks why half of successful calls required the JSON-object fallback path. The broader model matrix remains gated on those follow-ups.

## soft-semantic-v3 calibration result

After the generic decision contract and 18-case calibration corpus were merged, GitHub Actions run `33316513051` evaluated `ministral-8b-latest` for five calibration trials (90 calls). This is a calibration result, not independent evidence of generalization.

Operationally, all 90 calls succeeded across 5/5 complete trials. The run used 80,646 total tokens and 143,197 ms aggregate fixture latency. There were 121 successful provider-generation attempts. The JSON-object fallback was used for 31/90 calls (`0.3444`); all 31 were classified as `invalid_primary_structured_output`, with zero `primary_json_schema_unsupported` cases.

Semantic stability on the calibration corpus was precision `1.000`, recall `1.000`, mean decision coverage `0.622` (range `0.611`–`0.667`), and mean ambiguous abstention `0.971` (range `0.857`–`1.000`). Clear semantic equivalence and paraphrased premise support resolved to `no_finding` in all five trials; the explicit undistinguished reverse-causal alternative resolved to `finding` in all five; partial intervention and incomplete causal scope cases abstained in all five; and the clearly out-of-scope counterexample resolved to `no_finding` in all five. One older mixed causal calibration case produced one `finding` across five trials.

Because the v3 contract was calibrated on this corpus, these numbers are not used as a reliability claim. A separate holdout-v2 is frozen before the first v3 provider evaluation.
