# Engine 0.6 evidence-target relevance calibration v9 result

Status: **immutable FAIL**. Canonical run `36106913331`, attempt 1, frozen tag `engine-0.6-evidence-relevance-calibration-v9-freeze`, freeze commit `79089d0f1981207e47055fede9b342a60cb6f9a7`. Do not rerun, rescore, or overwrite the tag.

## Required-arm result

Mistral `ministral-8b-latest` completed 47/47 with zero provider failures and preserved all hard safety boundaries: wrong-target relevance retention 0, false relevance rejection 0, false-safe negative confirmation 0, and false-positive target-local confirmation 0. It nevertheless failed utility/materialization: materialized exact 38/47 (80.85%), utility misses 9, and expected Relevant left ambiguous 2. Primary proposal exact accuracy was 20/47 (42.55%); negative confirmation exact was 19/30 (63.33%) and positive confirmation exact was 11/13 (84.62%). The v9 policy was safe but over-abstained.

Groq `openai/gpt-oss-120b` was operationally incomplete. Cases `01_exact_name_availability` and `02_acronym_alias` both reached the positive Text confirmation stage, consumed the 24-token output budget, and returned no model text with `finish_reason=length`. Both became typed `protocol` failures; the two-consecutive-operational-failure circuit opened after case 02 and suppressed the remaining 45 cases. This is a confirmation transport/budget failure, not scored semantic evidence.

## Replication-arm result

Google `gemini-3.5-flash-lite` was non-gating and also operationally incomplete at 2/47. Both early positive-confirmation calls were rejected by Gemini with HTTP 400 because the v9 derived confirmation seed (`case_seed XOR 0xa93c_2b41`) exceeded the provider's accepted signed-32-bit integer range. The run correctly classified these as provider failures and opened the circuit. This was a provider-adapter seed-domain bug, not a semantic outcome.

## Successor implications

v10 must not weaken the hard safety gates. Instead it addresses two independent defects:

- replace enum-only raw Text confirmation with a small typed action-safety JSON contract and one bounded transport-only JSON-object fallback;
- classify the local action the Harness needs (`safe_to_reject` / `safe_to_accept` / abstain) rather than requiring the model to perfectly subtype global identity explanations;
- preserve explicit rename/alias/successor/shared-ownership uncertainty as abstention;
- normalize abstract Harness seeds into Google's supported seed domain inside the Google adapter;
- retain the shared 60-second case deadline and the two-consecutive-operational-failure circuit.

No independent holdout is authored from this FAIL. Holdout authoring remains blocked until a fresh canonical successor passes.
