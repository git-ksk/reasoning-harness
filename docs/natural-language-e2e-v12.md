# Natural-language E2E v12 — v0.4.2 release gate

v12 is the fresh successor measurement surface for the unreleased v0.4.2 candidate at product commit `2d53a27d5ea0e2eb28bba355496db1f2b513f6a7`. The Cargo version intentionally remains `0.4.1` until this gate passes. v12 does not reopen or repair v9-v11; every previously observed surface remains historical evidence.

## Hypothesis

v0.4.1 showed a pre-trigger planner/action-selection residual: a model could recall the exact fact target yet spend all action-selection rounds without executing a capability. v0.4.2 adds a narrow, authority-neutral acquisition precedence rule. When there is exactly one target identity, exact fact-key-bound read-only capabilities, and one unique highest explicit `selection_priority`, the Harness selects that capability without asking the model action selector. A typed `no_result` then remains governed by the existing Issue #249 exact-target continuation.

The three fresh adaptive cases freeze the same residual *class* without reusing any v11 target names, fact keys, answers, source identities, task text, or seed. Each uses priority `20` for the deterministic cache/no-result action and `10` for the exact-target registry follow-up.

## Frozen coordinate

- corpus: `natural-language-e2e-v12`
- evaluator: `reason-natural-language-e2e-v12`
- scoring: `natural-language-e2e-scoring-v12`
- product commit: `2d53a27d5ea0e2eb28bba355496db1f2b513f6a7`
- release target: `v0.4.2` (unreleased)
- canonical provider/model: `mistral / ministral-8b-latest`
- cross-model rows: `google / gemma-4-31b-it`, `google / gemini-3.5-flash-lite`, `groq / openai/gpt-oss-120b`
- base seed: `61000`
- max tokens: `1024`
- inter-case delay: `1500 ms`
- freeze tag: `natural-language-e2e-v12-freeze`

The first live case launch for each frozen target is canonical for that target. After any live observation, the corpus, evaluator, thresholds, target set, seed, token budget, workflow, or checksums must not be changed. A semantic change requires a new successor identity.

## Added observability

v12 preserves the v11 correctness and operational metrics and adds:

- `harness_precedence_selections`
- per-case `action_rejections` from `InvestigationTelemetry.rejected_actions`
- aggregate `action_rejection_count` and `duplicate_action_rejections`
- `precedence_selection_conformant`
- `deterministic_acquisition_ambiguity`
- `avoidable_followup_stall`
- aggregate `deterministic_acquisition_ambiguities` and `avoidable_followup_stalls`

A v12 trigger is exposed only when the Harness precedence selector chooses the configured higher-priority cache first and that action returns typed `no_result`. Issue #249 conformance is then measured only over trigger-exposed cases and must remain `1.0` whenever the denominator is non-zero.

## Pre-live gates

Before credentials are checked, the workflows prove the exact freeze tag/PR head, product commit, checksums, historical-surface immutability, full workspace tests, clippy/format, provider-aware concurrency policy, v12 evaluator tests, no-model/no-network resolver/admission/MCP-shape preflight, and an exact CLI provider/model probe. The CLI probe removes all provider credentials and requires each frozen coordinate to reach the typed `credentials` failure rather than a provider/model parse rejection; therefore it performs no model request or network call.

## Per-model release gate

No cross-model average is permitted. Every required row must independently be operationally complete, preserve the correctness/session/authority/identity safety boundary, have zero deterministic-acquisition ambiguity, zero duplicate-action rejection, and preserve Issue #249 conformance when exposed. Target recall and tool-selection success may not worsen, and false abstentions may not increase, where a historical semantic baseline exists.

| Row | v0.4.1 reference | Required v12 result |
| --- | --- | --- |
| Mistral `ministral-8b-latest` | recall `0.6`, tool selection `0.8`, false abstentions `6`, stalls `2/3`, trigger `1/3` | stalls strictly lower; trigger strictly higher; other listed utility metrics non-regressing |
| Google `gemini-3.5-flash-lite` | recall `1.0`, tool selection `0.6`, false abstentions `4`, stalls `3/3`, trigger `0/3` | stalls strictly lower; trigger strictly higher; other listed utility metrics non-regressing |
| Google `gemma-4-31b-it` | recall `0.8`, tool selection `0.9`, false abstentions `4`, stalls `0/3`, trigger `3/3`; historical row operationally incomplete | fresh row must be operationally complete and preserve the semantic ceiling: stalls `0/3`, trigger `3/3`; other listed utility metrics non-regressing |
| Groq `openai/gpt-oss-120b` | v11 generic CLI rejected the provider before generation, so no semantic baseline exists | full generic natural-language generation → investigation → action/tool → final report path must be operationally complete with the same zero-regression safety boundary |

A flat, mixed, worse, or incomplete required row blocks v0.4.2. The canonical result is preserved; the same frozen successor is not tuned and rerun.

## Concurrency

The canonical Mistral measurement runs once. Cross-model replication is allowed only after a successful canonical workflow for the same freeze commit. Google models share a provider lane and run serially (`max-parallel: 1`). Groq is a separate lane and may run independently of Google. There is no repository-wide `max-parallel: 1` across providers.
