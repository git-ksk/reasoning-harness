# Natural-language E2E v9 — exercised-lane successor

Issue #245 defines `natural-language-e2e-v9` as the fresh successor to the frozen v8 observation. v8 remains immutable historical evidence at `a262eba0b12bef569a631910c2fa7e80dae94010` / `natural-language-e2e-v8-freeze` / Actions `34078381222`.

v8 passed its frozen hard-correctness and rejection-validity gates, but post-observation audit showed two live capability-coverage gaps: the MCP case did not invoke the configured GitHub MCP lane, and the dedicated adaptive case did not execute the intended no-result -> registry follow-up. v9 does not repair, rescore, or rerun v8.

## Frozen product coordinate

The product under measurement remains exact shipped v0.4.0:

- tag: `v0.4.0`
- commit: `50c750d976be63b4e489ba5d7d7f3225bdd839b8`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `49000`
- max tokens: `1024`
- inter-case pacing: `1500ms`

Measurement-only files may differ from the release commit; `Cargo.toml`, `Cargo.lock`, and `crates/` must remain byte-diff clean against the released product coordinate.

## Fresh corpus

v9 contains eight fresh investigation cases and three fresh session cases, with fresh case/task/target/source/session markers mechanically disjoint from observed v1-v8 surfaces.

Investigation dimensions remain: unique-safe selection, ambiguous tool selection, freshness rejection, scope rejection, adaptive follow-up, authority rejection, identity rejection, and MCP generic-output non-promotion.

The live MCP lane uses pinned `CHANGELOG.md` at `refs/tags/v0.4.0` through the same digest-pinned official GitHub MCP image. The dedicated follow-up lane explicitly asks for the cache first and registry second after `no_result`.

## Pre-observation integrity

Before credentials/live execution, v9 requires:

- v1-v8 observed surfaces unchanged;
- fresh-marker disjointness in both directions;
- 10 resolver capabilities protocol-valid without model/network;
- 7 evidence-source checks, 6 allowlisted sources, 1 intentional identity-negative source;
- 3 mechanically admissible positive fixture cases;
- 4 mechanically encoded intended rejection contracts;
- exactly one `mcp_exercised` and one `no_result_followup` coverage contract;
- status-aware `RequiresVerification` scoring and `reason-session-v1` fork semantics regression tests;
- positive cases with no expected rejection report `expected_rejection_observed = null`, not `true`.

## Scoring

### Hard correctness

`hard_correctness_gate_passed` requires zero correctness-boundary violations. This includes unsupported final structured claims, unsupported/contract-invalid exposed factual text, unsafe grounding in expected-unknown cases, identity/authority/scope/freshness bypass, MCP self-promotion, session invalidation, and replay violations.

### Measurement validity

`measurement_validity_passed` is separate from product utility. It requires:

- 11/11 completed and operational failures `0`, including process failures, typed investigation action failures, and investigation generation failures hidden inside otherwise successful result envelopes;
- freshness/scope/authority/identity rejection coverage `4/4`;
- MCP live coverage `1/1`: the target is recalled and the configured MCP capability returns a non-operational action observation at least once;
- adaptive follow-up coverage `1/1`: the configured cache produces `no_result`, the configured registry is subsequently invoked, and that follow-up produces `applied_evidence` or `verification_progress`;
- session persistence/fork reconstruction valid and external replay `0`;
- token-usage case coverage at least `0.70`.

`adoption_gate_passed` is true only when both hard correctness and measurement validity pass.

### Utility

Utility remains observational, not tuned into the gate: target recall, grounded-target coverage, false abstentions, tool-selection success, useful follow-up, harness unique selections, irrelevant acquisition attempts, and stop reasons.

## Freeze discipline

Corpus, checksum manifest, evaluator, tests, scoring identity, provider/model, seed, budgets, MCP coordinate, docs, and workflow are frozen before the first live observation. After that observation, v9 becomes immutable historical evidence; any semantic or measurement change requires a successor identity.
