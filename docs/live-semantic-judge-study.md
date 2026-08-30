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
