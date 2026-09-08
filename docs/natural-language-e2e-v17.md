# Natural-language E2E v17 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v17 as the fresh held-out release surface for v0.4.2. The product candidate may change between successor identities; the measurement definitions may not.

## Why v17 exists

v16 was a valid metric-locked paired Mistral observation for candidate `3e6d0f5e...`. Both control and candidate completed 13/13 with zero operational and correctness/safety failures. The locked utility metrics were flat: target recall `0.6 -> 0.6`, tool selection `0.9 -> 0.9`, false abstentions `6 -> 6`, avoidable follow-up stalls `1 -> 1`, trigger exposure `2/3 -> 2/3`, and #249 conformance `2/2 -> 2/2`. The acceptance comparator therefore failed with `no strict improvement in locked follow-up utility metrics`. v16 is immutable failed evidence and is not rerun.

The directly observed stalled follow-up case recalled its target but produced four `invalid_shape` action rejections, executed no acquisition, and stopped at `round_budget`. The frozen v16 report retained only rejection counts, so the exact invalid proposal shape and the reason #261 precedence did not select were not directly observable. Issue #275 makes one narrow product/diagnostic change: rejected action proposals and typed validation reasons are retained, #261 skip reasons are recorded as diagnostic-only telemetry, typed validation rejections are included in the next action-planner request, and acquire/stop shape rules are explicit. It does not infer IDs, accept invalid actions, merge identities, expose `selection_priority`, relax #269, or change admission, verification, finalization, budgets, evaluator semantics, or release gates. v17 measures product candidate `3b3c2d35f437838603cdd88c489fe93843f6c011`.

v17 retains the v11 meanings exactly:

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
- candidate: `3b3c2d35f437838603cdd88c489fe93843f6c011` until a failed candidate is replaced by a new narrow product successor.

Task text, target key/value, expected outcome, source behavior, provider/model, seed, and token budget are paired. v0.4.1 predates `selection_priority`, so only candidate follow-up configs contain that field. MCP configs differ only in the exact Git commit used as `fixed_arguments.ref`. `validate_natural_language_e2e_v17_pair.py` rejects every other semantic config drift.

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

The v17 corpus, evaluator, comparator, provider/model targets, seed 74000, max tokens 1024, workflows, and checksums are frozen before live credentials. Mistral paired control/candidate runs first. Cross-model execution is allowed only after that paired gate succeeds on the same freeze commit. Google models are serialized within the provider lane; Groq is a separate lane.

If a candidate fails, do not alter v17 metrics or rerun the failed candidate as if it were new evidence. Diagnose using the frozen telemetry, make a narrow Harness/product change, create a fresh successor identity, and pair that new candidate against v0.4.1 under the same metric meanings.
