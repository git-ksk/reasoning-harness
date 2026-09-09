# Natural-language E2E v23 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v23 as the fresh held-out release surface for v0.4.2. Product candidates may change only through independently justified fixes between successor identities; measurement definitions remain locked from v11.

## Why v23 exists

Frozen v22 Mistral paired run `34318222654` is immutable VALID FAIL. Both control and candidate completed all 13 cases without correctness/safety boundary violations. Candidate preserved target recall `0.5 -> 0.5`, improved tool selection `0.9 -> 1.0`, false abstentions `6 -> 4`, avoidable follow-up stalls `1 -> 0`, trigger exposure `2 -> 3`, and preserved exposed mechanism conformance `1.0 -> 1.0`. The pair nevertheless failed because candidate recorded one `duplicate_action` rejection, and that field is a locked zero gate. v22 is not rerun or rescored.

A separate product-contract audit found #300: deterministic Harness selectors already excluded attempted target/capability pairs, but the model-facing action JSON Schema still advertised the unrestricted acquire shape and relied on prose to avoid reselection. PR #301 narrows only the structured model-facing schema to existing read-only, key-compatible, untried target/capability pairs plus `stop`. Runtime parsing remains broad and `validate_action()` remains authoritative and fail-closed. No target merging, fuzzy binding, selector authority, evidence/admission, verification, finalization, answer-safety, provider-specific correctness, evaluator, or scoring behavior changed.

v23 measures candidate `ced59271b437f23695b01e02bc49c051e99df970` against released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315`, using fresh corpus identity and seed 80000.

## Locked measurement semantics

v23 retains v11 meanings exactly: target recall is exact admitted `expected_fact_key` recall; tool selection is execution of a predeclared relevant capability; avoidable follow-up stall is a recalled follow-up target with zero actions; trigger exposure is the first configured cache action returning typed `no_result`; mechanism conformance is measured only on exposed cases and requires the immediate configured registry follow-up plus `harness_no_result_followup_selections`.

Action-rejection classes, precedence telemetry, and diagnostic traces remain diagnostic only except for the already-frozen release zero fields. They do not redefine utility metrics or correctness.

## Release rule

Required paired rows remain Mistral `ministral-8b-latest`, Google `gemini-3.5-flash-lite`, and Google `gemma-4-31b-it`. Each candidate row must be non-worse on target recall, tool selection, and false abstentions; never worsen follow-up stalls or trigger reachability; strictly improve at least one follow-up utility metric unless the paired control is already at the 0-stall / 3-trigger ceiling; preserve exposed mechanism conformance at 1.0; and preserve every correctness/safety zero gate. Cross-model averaging is forbidden.

Groq `openai/gpt-oss-120b` remains candidate-only generic-provider parity because released v0.4.1 does not expose the generic Groq provider path.

## Diagnostic sidecars

Candidate investigation calls write `reason-natural-diagnostic-trace-v1` sidecars to case-scoped files. Traces are preserved with canonical artifacts and validated for contract identity, but they are never evaluator inputs or scoring signals. The released control receives no diagnostic flag.

## Execution discipline

The corpus, evaluator, comparator, provider/model set, seed 80000, max tokens 1024, workflows, and checksums are frozen before any live credential is used. Mistral paired control/candidate runs first. Cross-model execution is allowed only after that same freeze commit passes the Mistral paired gate. A failed candidate requires a new independently justified product change and a fresh successor; v23 itself is never tuned, rerun, or rescored as a new observation.
