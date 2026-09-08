# Natural-language E2E v21 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v21 as the fresh held-out release surface for v0.4.2. The product candidate may change between successor identities; the measurement definitions may not.

## Why v21 exists

Frozen v20 Mistral paired run `34253166790` is immutable valid failed product evidence under the unchanged v11-locked ruler. Both control and candidate completed all 13 cases with all correctness/safety zero gates preserved. Candidate `ce4f6f8fd34d28180fd94e855b0d2b82b5316070` improved avoidable stalls `1 -> 0`, tool selection `0.8 -> 1.0`, trigger exposure `2/3 -> 3/3`, and preserved exposed #249 mechanism conformance `1.0 -> 1.0`, but target recall regressed `0.6 -> 0.5`. v20 is never rerun or rescored.

The directly observed v20 regression is `stale-noketh-window`; its fresh v21 counterpart is `stale-mavora-window`. The v20 candidate safely executed the intended stale read-only capability and retained the typed `rejected_evidence(stale)` outcome, but its admitted target omitted `expected_fact_key` while control retained `noketh.rollout.gate_open`. That lane has only one read-only capability and is outside #288's strict shared-exact predicate. #291 therefore restores the exact pre-#288 planner system/task/schema request for every non-strict configuration. The narrow #288 strict shared same-single-key contract remains unchanged. Runtime parsing, selectors, evidence, authority, verification, finalization, answer safety, evaluator, trigger, and release-gate semantics remain unchanged.

v21 measures fresh candidate `48948454fe4fadce66fb08d65ed2197c3232bc98` with seed 78000 on a fresh pseudonym surface. Evaluator/comparator semantics remain v11-locked.

v21 retains the v11 meanings exactly:

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
- candidate: `48948454fe4fadce66fb08d65ed2197c3232bc98` until a failed candidate is replaced by a new narrow product successor.

Task text, target key/value, expected outcome, source behavior, provider/model, seed, and token budget are paired. v0.4.1 predates `selection_priority`, so only candidate follow-up configs contain that field. MCP configs differ only in the exact Git commit used as `fixed_arguments.ref`. `validate_natural_language_e2e_v21_pair.py` rejects every other semantic config drift.

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

The v21 corpus, evaluator, comparator, provider/model targets, seed 78000, max tokens 1024, workflows, and checksums are frozen before live credentials. Mistral paired control/candidate runs first. Cross-model execution is allowed only after that paired gate succeeds on the same freeze commit. Google models are serialized within the provider lane; Groq is a separate lane.

If a candidate fails, do not alter v21 metrics or rerun the failed candidate as if it were new evidence. Diagnose using the frozen telemetry, make a narrow Harness/product change, create a fresh successor identity, and pair that new candidate against v0.4.1 under the same metric meanings.
