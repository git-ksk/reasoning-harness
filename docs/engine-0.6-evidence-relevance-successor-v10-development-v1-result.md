# Engine 0.6 evidence relevance successor-v10 development v1 result

Status: immutable development FAIL.

- Freeze tag: engine-0.6-evidence-relevance-successor-v10-development-v1-freeze
- Freeze commit: 7c21f3e3d207da72027924114009795b7508e167
- Run: 37097092197
- Candidate: effective qualification v13 / materialization v25
- Surface: 16 cases, 3 matched trials per provider
- Result: FAIL

The v1 tag and observation are retained as historical development evidence. Do not move the tag or reinterpret this run as PASS.

## Mistral result

Mistral / ministral-8b-latest completed all 48 observations operationally:

- successful observations: 48/48
- provider failures: 0
- confirmation matches: 21/48
- false confirmations: 3
- missed confirmations: 24

All eight expected-confirm families were returned as not_confirmed in all three trials, accounting for 24 missed confirmations.

The prompt-injection negative control was returned as confirmed_different_relation in all three trials, accounting for 3 false confirmations.

The dedicated model verifier therefore fails both the utility requirement and the standalone one-sided classification requirement.

## Cancellation

Once Mistral had completed and made the all-provider gate impossible to pass, run 37097092197 was intentionally cancelled to avoid unnecessary Groq quota consumption.

Google and Groq were still running and were cancelled. Their incomplete arms are operationally incomplete and provide no semantic acceptance evidence. Google attempt telemetry and partial Groq rate telemetry remain in the GitHub run artifacts.

The final gate correctly failed.

## Adjudication

The result does not show a v11/v23 safety regression. v11/v23 remained unchanged, and the candidate's deterministic Harness cue still gates the model verifier before v13 can create DifferentRelation.

In particular, the observed Mistral prompt-injection false confirmation cannot by itself produce a v25 Irrelevant result because the Harness-owned non-requested-relation cue is absent on that control.

However, all positive development families were missed by the verifier, so the model-verifier key prevents the candidate from recovering the residual over-abstention that motivated the successor.

## Next direction

Do not tune or rewrite this observed v1 result.

The next candidate should test whether the narrow Harness-owned semantic cue can itself serve as the negative-relation authority under exact-target / risk-none / non-positive relation constraints, without a model confirmation vote.

That successor must prove deterministically that:
- true requested-relation evidence is never downgraded;
- generic or explicit absence is not converted into DifferentRelation;
- context-gap / truncated evidence remains Ambiguous;
- prompt injection remains inert;
- the fresh negative controls remain unchanged even under an adversarial confirmation signal;
- historical relation controls still replay unchanged.

A fresh development observation should then exercise the full proposal + qualification + materialization path across providers before any new independent holdout.
