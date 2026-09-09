# Natural-language E2E v22 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v22 as the fresh held-out release surface for v0.4.2. The product candidate may change between successor identities; the measurement definitions may not.

## Why v22 exists

Frozen v21 Mistral paired run `34290310154` is immutable valid failed product evidence under the unchanged v11-locked ruler. Both coordinates completed the canonical paired observation without correctness/safety boundary violations, but candidate false abstentions worsened from `3` to `5`; v21 therefore remains a VALID FAIL and is never rerun, rescored, or tuned against.

After that freeze, a separate current-main code/contract audit found #297 independently of v21 held-out cases. Investigation plan admission still trimmed and normalized model-proposed `expected_fact_key` values even though #288 explicitly requires exact provider key identity with no trimming, normalization, guessing, or post-provider rewrite. The regression-first generic fixture demonstrated that a proposal such as `" routing.owner "` was silently admitted as `"routing.owner"`, which could manufacture eligibility for Harness-owned exact-key selection. PR #298 fixes only that boundary by preserving the proposed fact key byte-for-byte. Target ID/question admission, selector ordering, evidence admission, authority, verification, grounding, finalization, answer safety, provider behavior, evaluator, trigger definitions, and release metrics are unchanged.

The separately merged #295 observability work adds candidate-only diagnostic sidecars. Those traces are artifact evidence only: they do not enter the natural output report, evaluator input, or release scoring. The released v0.4.1 control is not passed a diagnostic flag.

v22 measures fresh candidate `9051049733a82571acb1ad572769b2c3f7495c50` with seed 79000 on a fresh pseudonym/task/fact/source surface. Evaluator/comparator semantics remain v11-locked.

v22 retains the v11 meanings exactly:

- `target_recalled`: expected fact key appears in the admitted investigation plan;
- `tool_selection_success`: an executed action uses a predeclared relevant capability;
- avoidable follow-up stall: `target_recalled=true && action_count=0` on a frozen follow-up case;
- `trigger_exposed`: the first executed configured cache action returns typed `no_result`;
- #249 denominator: trigger-exposed cases only;
- #249 conformance: immediate configured registry follow-up plus `harness_no_result_followup_selections`.

`harness_precedence_selections` and action-rejection classes are diagnostic telemetry only. They cannot redefine trigger reachability or independently pass/fail the release.

## Paired coordinates

The same fresh logical corpus is evaluated against:

- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`;
- candidate: `9051049733a82571acb1ad572769b2c3f7495c50` until a failed candidate is replaced by a new narrow product successor.

Task text, target key/value, expected outcome, source behavior, provider/model, seed, and token budget are paired. v0.4.1 predates `selection_priority`, so only candidate follow-up configs contain that field. MCP configs differ only in the exact Git commit used as `fixed_arguments.ref`. `validate_natural_language_e2e_v22_pair.py` rejects every other semantic config drift.

## Release rule

Required paired rows are Mistral `ministral-8b-latest`, Google `gemini-3.5-flash-lite`, and Google `gemma-4-31b-it`. For each row the candidate must:

- remain non-worse on target recall, tool-selection success, and false abstentions;
- never worsen avoidable follow-up stalls;
- never reduce v11-defined trigger reachability;
- strictly improve at least one of those two follow-up utility metrics unless the paired control is already at the structural ceiling of 0 stalls / 3 triggers;
- preserve #249 conformance at 1.0 whenever the trigger is exposed;
- preserve every v0.4.x correctness/safety zero gate.

Cross-model averaging is forbidden. A strong model cannot hide another model's regression.

Groq `openai/gpt-oss-120b` is candidate-only because generic `reason --provider groq` did not exist in v0.4.1. Its gate is operational generic-provider parity over the same 10 investigation + 3 session corpus with the same correctness boundary.

## Execution discipline

The v22 corpus, evaluator, comparator, provider/model targets, seed 79000, max tokens 1024, workflows, and checksums are frozen before live credentials. Mistral paired control/candidate runs first. Cross-model execution is allowed only after that paired gate succeeds on the same freeze commit. Google models are serialized within the provider lane; Groq is a separate lane.

For every candidate investigation case, the runner writes a `reason-natural-diagnostic-trace-v1` sidecar to a case-scoped path. The workflow preserves those ten traces with the canonical observation artifact and validates their contract IDs whenever the candidate live boundary was entered. Session cases and the released control remain unchanged. Sidecar presence is an evidence-completeness check, never a semantic metric.

If a candidate fails, do not alter v22 metrics or rerun the failed candidate as if it were new evidence. Diagnose using the frozen telemetry, make a narrow Harness/product change, create a fresh successor identity, and pair that new candidate against v0.4.1 under the same metric meanings.
