# Engine 0.6 evidence relevance: v7 required-provider gate decision

Status: precommitted provider-role decision before any v7 live observation.

## Decision

For calibration v7, the two required canonical arms are:

- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Google `gemini-3.5-flash-lite` is removed from the **required operational gate** for v7. This is not a semantic downgrade or a claim that Google produced incorrect relevance judgments. Google v4/v5/v6 completed observations were semantically safe/exact enough to rule out that interpretation; the repeated problem was canonical-run availability.

Google historical evidence remains immutable and explicit:

- v4: 23/26 operational under the 30-second envelope;
- v5: 11/26 operational under the 60-second envelope, including explicit HTTP 503 high-demand;
- recovery smoke: 4/6 operational;
- v6: one success followed by two 60-second timeouts, then the run-level circuit suppressed the remaining 23 requests.

No Google live requests are added to v7. Google requalification, if needed later, must use a separately frozen operational study rather than silently re-entering a passing calibration.

## Why Groq GPT-OSS 120B is precommitted

The selection is intentionally based on evidence independent of the #462 semantic calibration outcomes:

1. Repository history: Engine 0.5 final-v3 canonical run `35457038163` accepted Groq `openai/gpt-oss-120b` as a validated-reference row, 3/3 with zero correctness violations and zero session replay.
2. Repository history: Engine 0.5 final-v2 also recorded the same model as a validated required PASS.
3. Current provider status: Groq documents `openai/gpt-oss-120b` as a Production Model rather than a Preview Model.
4. Current capability: Groq documents JSON Object and JSON Schema / Structured Outputs support for `openai/gpt-oss-120b`.
5. Existing Harness adapter: Groq candidate output remains untrusted, uses Harness-owned parsing/materialization, supports bounded 429 retry, structured-output fallback retry, request/TPM pacing, and minimized reasoning controls for GPT-OSS.

Official current references checked before freeze:

- https://console.groq.com/docs/models
- https://console.groq.com/docs/model/openai/gpt-oss-120b
- https://console.groq.com/docs/structured-outputs

NVIDIA is not selected because existing repository evidence for its routine Nemotron candidate includes protocol/timeout incompleteness. Qwen 3.8 is not selected by running the #462 corpus and choosing a winner; doing so would tune provider choice to the calibration data. GPT-OSS 120B has the stronger pre-existing validated-reference history and current Production designation.

## Evaluation invariants

Changing the required provider does not change:

- binding proposal v2;
- target-first materialization v3;
- strict Harness-owned identity floor;
- any of the 26 calibration cases;
- any expected proposal/disposition label;
- 60-second per-case elapsed budget;
- max two model calls / 192 output tokens;
- zero-tolerance semantic correctness gates;
- run-level fail-fast after two consecutive operational provider failures.

The v7 result must stand on its own. v4/v5/v6 are not rescored under the new provider set.
