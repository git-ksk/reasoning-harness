# Semantic judge holdout v3

This directory is the observation-free independent holdout for `soft-semantic-v4` cross-model conformance research.

Freeze rules:

- Frozen on 2026-08-31 before any live provider run using `soft-semantic-v4`.
- 28 fixtures: seven per diagnostic kind.
- Label balance: 8 positive, 8 negative, 12 intentionally ambiguous.
- `recorded_observations` must remain empty in source control.
- Labels and fixture content are evaluator-owned and are never sent as model answers.
- Do not change prompt/contract/schema wording in response to results from this corpus and continue calling the changed contract v4. A material post-observation change requires a new configuration identity and a new observation-free holdout.
- Holdout-v1 and holdout-v2 remain historical/diagnostic corpora and are not independent evidence for v4.
- Model agreement or majority vote is never treated as truth.
