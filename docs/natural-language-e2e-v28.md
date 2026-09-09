# Natural-language E2E v28 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v28 as the fresh held-out successor for the v0.4.2 release gate. v21-v27 are immutable historical surfaces and are not rerun, rescored, tuned, or rewritten.

## Why v28 exists

Frozen v26 Mistral paired canonical evidence passed under the unchanged v11-locked ruler. The later frozen cross-model run `34356517560` is immutable failed evidence: Gemini paired failed on a released-control structured planner EOF whose historical diagnostics did not retain enough terminal provider status to distinguish ordinary malformed JSON from `status=incomplete`/token exhaustion; Groq `openai/gpt-oss-120b` candidate-only parity had three protocol failures after `JsonSchema -> JsonObject -> strict Text`, with strict Text receiving HTTP success/choices but empty content under the effective 256-token investigation planner/action budget; Gemma paired exposed both provider-unavailable HTTP 500 and structured malformed-JSON failures. Generic protocol failures remain non-retryable and v26 is not rerun or rescored.

The independently justified provider fix #316 was merged by PR #317 at `c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7`. After the v27 VALID FAIL, a separate general product defect was identified as #320: a legal exact-target `no_result` continuation could be lost when the next loop terminalized on the round limit before evaluating #249. PR #321 fixed that boundary and merged to main `756b63b5a5cbe024f89797b6c7da48be6700bc36`, which is the v28 candidate coordinate. Investigation plan/action requests now use provider-neutral `ModelReasoningPreference::Minimize`. Only known Groq GPT-OSS model IDs map that preference to low reasoning with returned reasoning suppressed; ordinary Groq wire requests are unchanged and reasoning text is never promoted as the Harness answer. Empty-content and structured-parse failures retain bounded terminal finish/status/token diagnostics, and the opt-in diagnostic trace records `structured_mode = json_schema | json_object | text`. These diagnostics do not enter classification, retry, evaluator input, or scoring. #311 operational-only retry semantics, strict structured parsing, planner token budget, and authority/admission/verification/finalization remain unchanged. The v11 target-recall, tool-selection, false-abstention, follow-up-stall, trigger-reachability, correctness, and safety formulas also remain locked; v28 adds only the predeclared #319 v12 measurement revision described below.

v28 pairs released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` with candidate `756b63b5a5cbe024f89797b6c7da48be6700bc36`, using fresh seed `95100` and a fresh 13-case corpus.

## Freshness and immutability

v28 uses new case IDs, tasks, fact keys, source identities, fresh markers, and seed. Deterministic tests require those identities and source refs to be absent from v1-v11 repository surfaces and from every frozen v12-v27 ref. The workflows also pin the exact v21-v27 freeze SHAs before credentials are exposed.

The acceptance branch may contain only v28 fixtures, evaluator-driver files, comparator/validator files, docs, checksums, and workflows. `Cargo.toml`, `Cargo.lock`, and `crates/` must remain byte-identical to candidate commit `756b63b5a5cbe024f89797b6c7da48be6700bc36`; the workspace version remains `0.4.1` during acceptance.

## Locked measurement semantics

The v11 release-metric meanings are preserved. Target recall is exact admitted `expected_fact_key` recall. Tool-selection success requires execution of a predeclared relevant capability. An avoidable follow-up stall is a recalled follow-up target with zero actions. Trigger exposure still means that the first executed configured cache action returned typed `no_result`; the trigger definition and trigger-reachability formula are unchanged.

The only v12 scoring revision is an explicit definition of a **legally executable #249 continuation opportunity**. `continuation_eligible` is true only when the trigger is exposed, the triggering target ID is bound to the case's exact fact key, the configured cache has exactly one remaining explicit read-only exact-key follow-up for that target, and the ordinary action/no-progress terminal budgets still permit continuation. Because #320 makes this continuation part of the selection round that produced the trigger, round budget alone does not make the opportunity ineligible. Mechanism conformance is measured over `continuation_eligible` cases only and requires the immediate configured registry action on the **same exact target ID** plus an increment of `harness_no_result_followup_selections`. Zero eligible cases are classified as `inconclusive`, not success.

Released-control mechanism conformance is retained as baseline utility observation rather than a hard validity condition on the control row. The candidate, however, must have mechanism conformance 1.0 whenever at least one eligible opportunity exists. Operational completeness and all correctness/safety zero fields remain hard gates for both control and candidate.

Action-rejection records, precedence telemetry, diagnostic traces, and operational-retry audit records remain diagnostic-only and do not alter the v11 utility/correctness numerators or denominators. Operational failures remain separate from semantic correctness.

`validate_natural_language_e2e_v28_metric_lock.py` uses immutable `natural-language-e2e-v27-freeze` as the predecessor. Only `validate_corpus`, `score_investigation`, `aggregate`, and `main` may differ in the runner, with one exact new helper, `continuation_budget_allows`. The validator requires AST equality with v27 for 46 preserved score-return entries, 63 preserved aggregate-return entries, key scoring assignments, paired utility gate conditions, hard correctness/safety ZERO fields, and pair-scrub semantics. The only allowed release-scoring delta is v12 continuation eligibility/conformance plus the candidate-only hard mechanism gate.

## Release rule

Required paired rows are:

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Google `gemma-4-31b-it`

Each candidate row must be non-worse than its own released-control row on target recall, tool-selection success, false abstentions, and all correctness/safety zero fields. Follow-up stalls may not increase and v11 trigger reachability may not decrease. Unless the control is already at the two-metric ceiling of 0 stalls and 3 triggers, the candidate must strictly improve at least one locked follow-up utility metric. If the control is already at that ceiling, exact non-regression is required. Released-control mechanism conformance remains baseline observation; the candidate must have mechanism conformance 1.0 whenever `continuation_eligible_cases > 0`. Cross-model averaging is forbidden.

Groq `openai/gpt-oss-120b` remains candidate-only generic-provider parity because released v0.4.1 does not expose generic Groq. It must complete all 10 investigation and 3 session cases with zero operational failures and the same correctness boundary.

## Execution discipline

The corpus, evaluator, comparator, retry policy, provider/model set, seed `95100`, max tokens `1024`, workflows, and checksums are frozen before any live credential is used. Mistral paired control/candidate runs first. Cross-model execution on the same freeze is permitted only after the Mistral paired workflow succeeds. A failed candidate/live observation is immutable evidence; any product correction requires an independently justified product fix followed by a fresh successor identity rather than tuning or rerunning v28 as a new canonical observation.
