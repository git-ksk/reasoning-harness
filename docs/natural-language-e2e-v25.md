# Natural-language E2E v25 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v25 as the fresh held-out release surface for v0.4.2. Product candidates may change only through independently justified fixes between successor identities; measurement definitions remain locked from v11.

## Why v25 exists

Frozen v24 Mistral paired run `34328390548` is an immutable PASS for the primary paired gate. The subsequent frozen cross-model run `34329379978` was operationally incomplete rather than a valid semantic comparison: the Gemini row terminated with a protocol generation failure, the Gemma row encountered repeated Google HTTP 500 provider-unavailable failures, and the Groq candidate-only parity row encountered structured-output HTTP 400 generation/validation failures. v24 is not rerun or rescored, and no v24 held-out case identity or answer is carried into the v25 rationale.

Independent provider-contract audits then found #306 and #307. PR #308 makes retryable Google 5xx and empty-text transients consume the adapter's already-declared four-attempt ceiling instead of terminating early. PR #309 classifies the Groq structured JSON generation/validation failure family consistently and, after bounded native retries, routes the generic investigation planner/action structured call through the already-existing schema-in-prompt JSON-object fallback. Both changes remain provider/transport robustness changes: the Harness-owned parser, evidence admission, verification, finalization, answer-safety authority, candidate/final-render semantics, evaluator, and scoring rules are unchanged.

v25 measures candidate `c804551b381f48f14809f2ab37d0609c33006ddf` against released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315`, using a fresh corpus identity and seed 82000.

## Locked measurement semantics

v25 retains v11 meanings exactly: target recall is exact admitted `expected_fact_key` recall; tool selection is execution of a predeclared relevant capability; avoidable follow-up stall is a recalled follow-up target with zero actions; trigger exposure is the first configured cache action returning typed `no_result`; mechanism conformance is measured only on exposed cases and requires the immediate configured registry follow-up plus `harness_no_result_followup_selections`.

Action-rejection classes, precedence telemetry, and diagnostic traces remain diagnostic only except for the already-frozen release zero fields. They do not redefine utility metrics or correctness.

## Release rule

Required paired rows remain Mistral `ministral-8b-latest`, Google `gemini-3.5-flash-lite`, and Google `gemma-4-31b-it`. Each candidate row must be non-worse on target recall, tool selection, and false abstentions; never worsen follow-up stalls or trigger reachability; strictly improve at least one follow-up utility metric unless the paired control is already at the 0-stall / 3-trigger ceiling; preserve exposed mechanism conformance at 1.0; and preserve every correctness/safety zero gate. Cross-model averaging is forbidden.

Groq `openai/gpt-oss-120b` remains candidate-only generic-provider parity because released v0.4.1 does not expose the generic Groq provider path.

## Diagnostic sidecars

Candidate investigation calls write `reason-natural-diagnostic-trace-v1` sidecars to case-scoped files. Traces are preserved with canonical artifacts and validated for contract identity, but they are never evaluator inputs or scoring signals. The released control receives no diagnostic flag.

## Execution discipline

The corpus, evaluator, comparator, provider/model set, seed 82000, max tokens 1024, workflows, and checksums are frozen before any live credential is used. Mistral paired control/candidate runs first. Cross-model execution is allowed only after that same freeze commit passes the Mistral paired gate. A failed candidate requires a new independently justified product change and a fresh successor; v25 itself is never tuned, rerun, or rescored as a new observation.
