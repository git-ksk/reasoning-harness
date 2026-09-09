# Natural-language E2E v26 — metric-locked paired v0.4.1 / v0.4.2 acceptance

Issue #263 uses v26 as the fresh held-out successor for the v0.4.2 release gate. v21-v25 are immutable historical surfaces and are not rerun, rescored, tuned, or rewritten.

## Why v26 exists

Frozen v25 Mistral paired run `34342578577` passed. Its first job attempt stopped before live API execution on the pre-live deterministic Transport flake `external_command::tests::investigation_adapter_uses_separate_request_identity`; rerunning only that failed job produced the canonical live observation and preserved the frozen surface.

Frozen v25 cross-model run `34344002296` was operationally incomplete rather than candidate-performance evidence. The released v0.4.1 Google control failed before a valid paired comparison could be completed: Gemini had one protocol generation failure and Gemma had repeated provider-unavailable Google HTTP 500 failures. Groq candidate-only parity improved from 9 operational failures in v24 to 3 in v25, but the remaining three failures occurred after `JsonSchema` degradation because the `JsonObject` fallback still required Groq server-side JSON generation and returned HTTP 400 generation/validation failures.

v25 stays frozen. Two independently justified generic fixes were then merged before creating this successor:

- #312 / PR #313: recognized structured generation/validation failures are capability failures for both Groq `JsonSchema` and `JsonObject`; the generic structured-call ladder may degrade `JsonSchema -> JsonObject -> Text`, with the same schema embedded in the prompt and the existing strict Harness parser/serde contract. No repair, fuzzy extraction, additional-field tolerance, authority change, evaluator change, or scoring change was introduced.
- #311 / PR #314: paired evaluation receives a bounded, case-level operational-only retry driver. Retry is limited to typed transient provider generation failures (`transport`, `provider_unavailable`, `timeout`) plus the exact released-control Google empty-model-text protocol subtype. The same command/model/seed/token/config/coordinate is reused. Generic protocol failures, invalid JSON, semantic/scoring failures, quota, credentials, ordinary provider 4xx, unsupported capability, resolver/tool/action failures, and rendering fallbacks are not retried. The first operationally complete or non-retryable attempt is canonical; prior operational attempts are audit-only. Whole-run retry is forbidden. Stateful session mutation commands are intentionally not retried because they persist invalidation state before model execution; session start remains retry-safe because persistence occurs only after successful natural execution.

v26 pairs released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` with candidate `2107af7942dd3cb7dac5fc30603359095f4a2709`, using fresh seed `93000` and a fresh 13-case corpus.

## Freshness and immutability

v26 uses new case IDs, tasks, fact keys, source identities, fresh markers, and seed. Deterministic tests require those identities and source refs to be absent from v1-v11 repository surfaces and from every frozen v12-v25 ref. The workflows also pin the exact v21-v25 freeze SHAs before credentials are exposed.

The acceptance branch may contain only v26 fixtures, evaluator-driver files, comparator/validator files, docs, checksums, and workflows. `Cargo.toml`, `Cargo.lock`, and `crates/` must remain byte-identical to candidate commit `2107af7942dd3cb7dac5fc30603359095f4a2709`; the workspace version remains `0.4.1` during acceptance.

## Locked measurement semantics

All release metrics retain the v11 meanings. Target recall is exact admitted `expected_fact_key` recall. Tool-selection success requires execution of a predeclared relevant capability. An avoidable follow-up stall is a recalled follow-up target with zero actions. Trigger exposure means the first executed configured cache action returned typed `no_result`. Mechanism conformance is measured only over trigger-exposed cases and requires the immediate configured registry continuation plus `harness_no_result_followup_selections` for the exact target.

Action-rejection records, precedence telemetry, diagnostic traces, and operational-retry audit records are diagnostic only and never enter the release metric numerator or denominator. Operational failures remain separate from semantic correctness.

`validate_natural_language_e2e_v26_metric_lock.py` compares v26 against `natural-language-e2e-v25-freeze`. It requires zero AST diff for the locked runner scoring functions, normalized equality for session scoring after removing only the operational-retry wrapper, and normalized equality for the acceptance comparator, pair validator, and acceptance tests. Version identity, fresh seed, and candidate SHA are the only normalized identity substitutions.

## Release rule

Required paired rows are:

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Google `gemma-4-31b-it`

Each candidate row must be non-worse than its own released-control row on target recall, tool-selection success, false abstentions, and all correctness/safety zero fields. Follow-up stalls may not increase and v11 trigger reachability may not decrease. Unless the control is already at the two-metric ceiling of 0 stalls and 3 triggers, the candidate must strictly improve at least one locked follow-up utility metric. If the control is already at that ceiling, exact non-regression is required. Exposed mechanism conformance must remain 1.0. Cross-model averaging is forbidden.

Groq `openai/gpt-oss-120b` remains candidate-only generic-provider parity because released v0.4.1 does not expose generic Groq. It must complete all 10 investigation and 3 session cases with zero operational failures and the same correctness boundary.

## Execution discipline

The corpus, evaluator, comparator, retry policy, provider/model set, seed `93000`, max tokens `1024`, workflows, and checksums are frozen before any live credential is used. Mistral paired control/candidate runs first. Cross-model execution on the same freeze is permitted only after the Mistral paired workflow succeeds. A failed candidate/live observation is immutable evidence; any product correction requires an independently justified product fix followed by a fresh successor identity rather than tuning or rerunning v26 as a new canonical observation.
