# Natural-language E2E v27 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v27 as the fresh held-out successor for the v0.4.2 release gate. v21-v26 are immutable historical surfaces and are not rerun, rescored, tuned, or rewritten.

## Why v27 exists

Frozen v26 Mistral paired canonical evidence passed under the unchanged v11-locked ruler. The later frozen cross-model run `34356517560` is immutable failed evidence: Gemini paired failed on a released-control structured planner EOF whose historical diagnostics did not retain enough terminal provider status to distinguish ordinary malformed JSON from `status=incomplete`/token exhaustion; Groq `openai/gpt-oss-120b` candidate-only parity had three protocol failures after `JsonSchema -> JsonObject -> strict Text`, with strict Text receiving HTTP success/choices but empty content under the effective 256-token investigation planner/action budget; Gemma paired exposed both provider-unavailable HTTP 500 and structured malformed-JSON failures. Generic protocol failures remain non-retryable and v26 is not rerun or rescored.

The independently justified product fix #316 was merged by PR #317 at candidate commit `c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7` before constructing v27. Investigation plan/action requests now use provider-neutral `ModelReasoningPreference::Minimize`. Only known Groq GPT-OSS model IDs map that preference to low reasoning with returned reasoning suppressed; ordinary Groq wire requests are unchanged and reasoning text is never promoted as the Harness answer. Empty-content and structured-parse failures retain bounded terminal finish/status/token diagnostics, and the opt-in diagnostic trace records `structured_mode = json_schema | json_object | text`. These diagnostics do not enter classification, retry, evaluator input, or scoring. #311 operational-only retry semantics, strict structured parsing, planner token budget, authority/admission/verification/finalization, and the v11 metric lock remain unchanged.

v27 pairs released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` with candidate `c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7`, using fresh seed `94000` and a fresh 13-case corpus.

## Freshness and immutability

v27 uses new case IDs, tasks, fact keys, source identities, fresh markers, and seed. Deterministic tests require those identities and source refs to be absent from v1-v11 repository surfaces and from every frozen v12-v26 ref. The workflows also pin the exact v21-v26 freeze SHAs before credentials are exposed.

The acceptance branch may contain only v27 fixtures, evaluator-driver files, comparator/validator files, docs, checksums, and workflows. `Cargo.toml`, `Cargo.lock`, and `crates/` must remain byte-identical to candidate commit `c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7`; the workspace version remains `0.4.1` during acceptance.

## Locked measurement semantics

All release metrics retain the v11 meanings. Target recall is exact admitted `expected_fact_key` recall. Tool-selection success requires execution of a predeclared relevant capability. An avoidable follow-up stall is a recalled follow-up target with zero actions. Trigger exposure means the first executed configured cache action returned typed `no_result`. Mechanism conformance is measured only over trigger-exposed cases and requires the immediate configured registry continuation plus `harness_no_result_followup_selections` for the exact target.

Action-rejection records, precedence telemetry, diagnostic traces, and operational-retry audit records are diagnostic only and never enter the release metric numerator or denominator. Operational failures remain separate from semantic correctness.

`validate_natural_language_e2e_v27_metric_lock.py` compares v27 against `natural-language-e2e-v26-freeze`. It requires zero AST diff for the locked runner scoring functions, exact AST equality for session scoring because frozen v26 already contains the same operational-retry wrapper, and normalized equality for the acceptance comparator, pair validator, and acceptance tests. Version identity, fresh seed, and candidate SHA are the only normalized identity substitutions.

## Release rule

Required paired rows are:

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Google `gemma-4-31b-it`

Each candidate row must be non-worse than its own released-control row on target recall, tool-selection success, false abstentions, and all correctness/safety zero fields. Follow-up stalls may not increase and v11 trigger reachability may not decrease. Unless the control is already at the two-metric ceiling of 0 stalls and 3 triggers, the candidate must strictly improve at least one locked follow-up utility metric. If the control is already at that ceiling, exact non-regression is required. Exposed mechanism conformance must remain 1.0. Cross-model averaging is forbidden.

Groq `openai/gpt-oss-120b` remains candidate-only generic-provider parity because released v0.4.1 does not expose generic Groq. It must complete all 10 investigation and 3 session cases with zero operational failures and the same correctness boundary.

## Execution discipline

The corpus, evaluator, comparator, retry policy, provider/model set, seed `94000`, max tokens `1024`, workflows, and checksums are frozen before any live credential is used. Mistral paired control/candidate runs first. Cross-model execution on the same freeze is permitted only after the Mistral paired workflow succeeds. A failed candidate/live observation is immutable evidence; any product correction requires an independently justified product fix followed by a fresh successor identity rather than tuning or rerunning v27 as a new canonical observation.
