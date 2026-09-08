# Natural-language E2E v15 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v15 as the fresh held-out release surface for v0.4.2. The product candidate may change between successor identities; the measurement definitions may not.

## Why v15 exists

v14 was the first valid paired control/candidate observation under the restored v11-locked metric meanings. Its Mistral row completed the released v0.4.1 control cleanly, but candidate `2d53a27d...` exposed a product-side acquisition loop in `adaptive-rethane-owner`: after `cache(no_result) -> registry(verification_progress)`, #261 precedence could deterministically select a second admitted target identity carrying the same `expected_fact_key`, causing the same capability chain to run again. The second registry invocation ended as a typed `malformed_output`, mechanism conformance fell from 3/3 to 2/3, and downstream useful follow-up fell from 3/3 to 2/3. v14 is therefore immutable failed product evidence and is not rerun.

Issue #269 fixes only that product defect. `unique_precedence_action_proposal()` now stays fail-closed whenever an expected fact key is represented by more than one admitted target identity, including the dynamic state where one sibling has already exhausted its capability pairs. Target identities remain distinct; #249 continuation, admission, verification, finalization, budgets, and evaluator definitions are unchanged. v15 is the fresh held-out successor for product candidate `c069954cca77361b0e8d3e91334a448efdd2acd9`.

v15 retains the v11 meanings exactly:

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
- candidate: `c069954cca77361b0e8d3e91334a448efdd2acd9` until a failed candidate is replaced by a new narrow product successor.

Task text, target key/value, expected outcome, source behavior, provider/model, seed, and token budget are paired. v0.4.1 predates `selection_priority`, so only candidate follow-up configs contain that field. MCP configs differ only in the exact Git commit used as `fixed_arguments.ref`. `validate_natural_language_e2e_v15_pair.py` rejects every other semantic config drift.

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

The v15 corpus, evaluator, comparator, provider/model targets, seed 71000, max tokens 1024, workflows, and checksums are frozen before live credentials. Mistral paired control/candidate runs first. Cross-model execution is allowed only after that paired gate succeeds on the same freeze commit. Google models are serialized within the provider lane; Groq is a separate lane.

If a candidate fails, do not alter v15 metrics or rerun the failed candidate as if it were new evidence. Diagnose using the frozen telemetry, make a narrow Harness/product change, create a fresh successor identity, and pair that new candidate against v0.4.1 under the same metric meanings.
