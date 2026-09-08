# Natural-language E2E v19 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v19 as the fresh held-out release surface for v0.4.2. The product candidate may change between successor identities; the measurement definitions may not.

## Why v19 exists

Frozen v18 Mistral run `34233275948` is valid failed product evidence under the unchanged v11-locked ruler. Both rows completed all 13 cases with operational failures `0` and all correctness/safety zero gates preserved, but candidate `03d68b4ab3a51a7457eafe3a4cc820a687b940e9` regressed: target recall `0.6 -> 0.6`, tool selection `0.8 -> 0.4`, false abstentions `5 -> 6`, avoidable follow-up stalls `1 -> 2`, trigger exposure `2/3 -> 1/3`, and exposed #249 conformance `1.0 -> 0.0`. v18 remains immutable.

The directly observed cause was repeated `acquire` proposals with `target_id` present but `capability_id` missing, rejected as `invalid_shape` until `round_budget`. #281 narrowly changes only the provider-neutral model-facing action schema to a closed acquire/stop union: acquire requires non-empty `target_id` and `capability_id`; stop forbids both. Runtime parsing/validation remains fail-closed and does not infer or repair IDs. No sibling merge, fuzzy matching, provider-specific correctness behavior, authority, admission, verification, finalization, or answer-safety semantics change.

v19 measures fresh candidate `1b889d843616ca257ec5e0e98411906e2c7152f6` with seed 76000. Evaluator/comparator semantics remain v11-locked.

v19 retains the v11 meanings exactly:

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
- candidate: `1b889d843616ca257ec5e0e98411906e2c7152f6` until a failed candidate is replaced by a new narrow product successor.

Task text, target key/value, expected outcome, source behavior, provider/model, seed, and token budget are paired. v0.4.1 predates `selection_priority`, so only candidate follow-up configs contain that field. MCP configs differ only in the exact Git commit used as `fixed_arguments.ref`. `validate_natural_language_e2e_v19_pair.py` rejects every other semantic config drift.

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

The v19 corpus, evaluator, comparator, provider/model targets, seed 76000, max tokens 1024, workflows, and checksums are frozen before live credentials. Mistral paired control/candidate runs first. Cross-model execution is allowed only after that paired gate succeeds on the same freeze commit. Google models are serialized within the provider lane; Groq is a separate lane.

If a candidate fails, do not alter v19 metrics or rerun the failed candidate as if it were new evidence. Diagnose using the frozen telemetry, make a narrow Harness/product change, create a fresh successor identity, and pair that new candidate against v0.4.1 under the same metric meanings.
