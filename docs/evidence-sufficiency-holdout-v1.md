# Fresh independent evidence-sufficiency holdout v1

Tracking: #91, #125. Execution workflow: `evidence-sufficiency-holdout-v1`.

This holdout was created after RSD0-RSD2 calibration/characterization and before any product-bridge adoption claim. It does not reuse the RSD calibration corpus. The committed corpus contains 24 cases across eight new families with exactly one `sufficient`, `insufficient`, and `mixed` case per family. The checksum manifest is verified before provider calls. Frozen semantic holdout-v4/v5 remain untouched.

## Frozen execution

- models: `ministral-8b-latest` and Google-hosted `gemma-4-31b-it`;
- 24 fixtures x 5 trials = 120 observations/model;
- seeds: 7000-7004;
- maximum output tokens: 128;
- surface: `holdout`;
- no workflow inputs can reshape corpus, model set, trial count, seeds, or thresholds.

Promotion gates were fixed before observation:

1. operational completion = 1.00;
2. conservative binary accuracy >= 0.95;
3. false-safe count = 0;
4. false-abstain rate <= 0.05;
5. sufficient recall >= 0.95;
6. binary fixture unanimity = 1.00.

## Observed result

GitHub Actions run `33568061693` on main commit `c59790891911f9e75b85ac9cd30eb07994bec707` passed both model arms.

| metric | Ministral 8B | Gemma 4 31B |
| --- | ---: | ---: |
| operational completion | 1.000 | 1.000 |
| conservative binary accuracy | 1.000 | 1.000 |
| false-safe count | 0 | 0 |
| false-abstain rate | 0.000 | 0.000 |
| sufficient recall | 1.000 | 1.000 |
| binary fixture unanimity | 1.000 | 1.000 |
| exact 3-class accuracy | 0.8833 | 1.000 |
| exact fixture unanimity | 0.9583 | 1.000 |

Ministral's 14 exact-label errors were all `mixed -> insufficient`. They therefore remained on the predeclared `non_sufficient` side of the safety boundary; no case crossed from non-sufficient to sufficient or vice versa. This is reported as diagnostic drift, not hidden by majority voting or threshold changes.

The frozen holdout supplies predeclared task-specific `required_information`; it does not validate any later product mechanism for generating or selecting those requirements. The first product bridge used `generic-answer-sufficiency-requirements-v1`; NL-5 later showed that policy could over-suppress safe partial facts, so the product successor now versions a separate claim-local policy (`claim-local-answer-sufficiency-requirements-v1`) under `d3-sufficiency-answer-gate-v2`. Neither product policy is retroactively treated as holdout-validated.

## Authority interpretation

Passing this holdout is evidence only for promoting a conservative product gate. `sufficient` remains non-authoritative and cannot create trusted evidence, verification receipts, hard findings, epistemic-state promotion, or a final verdict. Only `insufficient` / `mixed` may force verification, bounded resolution, or abstention. Product usefulness is evaluated separately in NL-5 and the frozen holdout is never tuning data.
