# Natural-language E2E v24 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v24 as the fresh held-out release surface for v0.4.2. Product candidates may change only through independently justified fixes between successor identities; measurement definitions remain locked from v11.

## Why v24 exists

Frozen v23 Mistral paired run `34325217339` is immutable VALID FAIL. Both control and candidate completed all 13 cases without correctness/safety boundary violations. Candidate eliminated duplicate-action rejections, preserved target recall `0.6 -> 0.6`, tool selection `1.0 -> 1.0`, avoidable follow-up stalls `0 -> 0`, trigger exposure `3 -> 3`, and mechanism conformance `1.0 -> 1.0`, but false abstentions worsened `4 -> 6`. v23 is not rerun or rescored, and no v23 held-out case identity or answer is carried into the v24 rationale.

A separate generic final-render contract audit found #303. The renderer instruction requires every intended factual proposition to be declared in `factual_claims`, while the model-facing JSON Schema inherited the runtime `#[serde(default)]` and allowed that field to be omitted. PR #304 narrows only the model-facing schema so `factual_claims` is required. Runtime parsing remains backward-compatible, `finalize_answer()` remains authoritative and fail-closed, and canonical recovery scope is unchanged. No target inference from prose, fuzzy matching, admitted-target rewrite, evidence/verification authority change, provider-specific correctness branch, evaluator change, or scoring change was introduced.

v24 measures candidate `a0c145dc713b848719c539ea2fbc6bf436db85a2` against released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315`, using fresh corpus identity and seed 81000.

## Locked measurement semantics

v24 retains v11 meanings exactly: target recall is exact admitted `expected_fact_key` recall; tool selection is execution of a predeclared relevant capability; avoidable follow-up stall is a recalled follow-up target with zero actions; trigger exposure is the first configured cache action returning typed `no_result`; mechanism conformance is measured only on exposed cases and requires the immediate configured registry follow-up plus `harness_no_result_followup_selections`.

Action-rejection classes, precedence telemetry, and diagnostic traces remain diagnostic only except for the already-frozen release zero fields. They do not redefine utility metrics or correctness.

## Release rule

Required paired rows remain Mistral `ministral-8b-latest`, Google `gemini-3.5-flash-lite`, and Google `gemma-4-31b-it`. Each candidate row must be non-worse on target recall, tool selection, and false abstentions; never worsen follow-up stalls or trigger reachability; strictly improve at least one follow-up utility metric unless the paired control is already at the 0-stall / 3-trigger ceiling; preserve exposed mechanism conformance at 1.0; and preserve every correctness/safety zero gate. Cross-model averaging is forbidden.

Groq `openai/gpt-oss-120b` remains candidate-only generic-provider parity because released v0.4.1 does not expose the generic Groq provider path.

## Diagnostic sidecars

Candidate investigation calls write `reason-natural-diagnostic-trace-v1` sidecars to case-scoped files. Traces are preserved with canonical artifacts and validated for contract identity, but they are never evaluator inputs or scoring signals. The released control receives no diagnostic flag.

## Execution discipline

The corpus, evaluator, comparator, provider/model set, seed 81000, max tokens 1024, workflows, and checksums are frozen before any live credential is used. Mistral paired control/candidate runs first. Cross-model execution is allowed only after that same freeze commit passes the Mistral paired gate. A failed candidate requires a new independently justified product change and a fresh successor; v24 itself is never tuned, rerun, or rescored as a new observation.
