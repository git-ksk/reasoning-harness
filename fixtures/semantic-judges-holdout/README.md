# Semantic-judge holdout v1

This directory is an independent observation-free holdout corpus for live soft semantic-judge research.

- `fixtures/semantic-judges/` remains the calibration corpus.
- Holdout v1 fixture/request IDs are frozen after the first merge that introduces them.
- Provider results must not be used to edit these cases or tune the judge prompt against this version.
- If later prompt changes require another independent measurement, add a new holdout version instead of rewriting observed v1 cases.
- Labels are evaluator-owned and are never included in the model request.
- Recorded observations remain empty in source; live observations are generated only in run output.

Model decisions remain advisory and cannot create verification authority, hard findings, epistemic promotion, or verdicts.
