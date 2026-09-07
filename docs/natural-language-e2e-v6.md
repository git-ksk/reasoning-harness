# Natural-language E2E v6 — v0.4.0 post-release independent measurement

Issue #239 defines `natural-language-e2e-v6` as a fresh, pre-observation measurement of the released product coordinate `v0.4.0` / `50c750d976be63b4e489ba5d7d7f3225bdd839b8`. It does not rerun, rescore, repair, or tune against natural-language E2E v1-v5, product-external-info v1-v4, Stage-C, RSD2, or the frozen #193/#195/#196 research surfaces.

## Frozen identities and product coordinate

- corpus: `natural-language-e2e-v6`
- evaluator/report: `reason-natural-language-e2e-v6`
- scoring: `natural-language-e2e-scoring-v6`
- product tag: `v0.4.0`
- product commit: `50c750d976be63b4e489ba5d7d7f3225bdd839b8`
- product version: `0.4.0`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- MCP adapter: `mcp_readonly_v3`
- provider: Mistral / `ministral-8b-latest`
- base seed: `43000`
- max tokens: `1024`
- inter-case pacing: `1500 ms`

The pre-measurement capability probe Actions `34073701804` ran on the release commit and completed 3/3 protocol probes. Mistral returned `ministral-8b-latest` as the observed model identifier. The provider does not expose a more specific backing revision in this response, so v6 freezes the requested/observed API identifier and records that limitation rather than inventing a backend version.

## Independent corpus

v6 has eight investigation cases and three session cases. The investigation cases use new task wording, case IDs, target fact keys, source identities, and fixture entities. Mechanical tests compare these markers against the committed historical corpus roots and the frozen historical refs listed in the manifest.

Investigation coverage includes:

1. one explicit fact-key / one explicit read-only capability pair, where #233 may select the action deterministically;
2. an ambiguous two-capability lookup that remains model-selected;
3. stale evidence rejection;
4. scope mismatch rejection;
5. no-result followed by a remaining unique safe action;
6. authority-claim mismatch rejection;
7. source-identity mismatch rejection;
8. a real `mcp_readonly_v3` generic-output non-promotion case.

Session coverage includes `add`, `correct`, `resume`, and `fork`. The evaluator checks typed invalidation, stale-finalization suppression through deterministic product tests, replay count zero, non-destructive fork behavior, and persisted finalization consistency.

## #233 measurement

The report keeps the runtime's two selection counters separate:

- `harness_unique_selections`: actions chosen by the Harness only when exactly one untried target/capability pair is mechanically compatible and explicitly fact-key-bound;
- `model_selected_action_calls`: model action-selector calls (`planner_calls` in investigation telemetry).

The evaluator does not infer which actor selected an action from outcome quality. It uses the product telemetry directly.

## MCP v3 lane

The independent v6 MCP case pins:

- image: `ghcr.io/github/github-mcp-server@sha256:46cdbbd810faf6f7aed1745ea04057443f5cb9fcadc15c7308add18cf9a83e33`
- historical image version reference: `v1.12.0`
- repository: `git-ksk/reasoning-harness`
- ref: `refs/tags/v0.4.0`
- path: `Cargo.toml`
- tool: `get_file_contents`
- read-only mode: required
- requested protocol: `2026-07-28`
- accepted downlevel protocol: `2025-11-25`

This is not the historical #204 acceptance case. The new case deliberately expects `unknown`: ordinary GitHub MCP file content is generic acquisition output and must not self-promote into Harness-authoritative facts. Before the model-backed observation, the workflow separately checks the pinned server/tool contract without using its result as a semantic score.

## Scoring

The hard correctness gate is `correctness_boundary_violations == 0`. The report separately exposes unsupported structured claims, unsupported exposed factual assertions, exposed-text contract violations, missed insufficiency, identity-unsafe admission, MCP output authority self-promotion, session invalidation/replay failures, and admission rejection telemetry.

Utility metrics include target recall/omission, grounded target coverage, relevant capability selection, useful follow-up, irrelevant acquisition attempts, false abstention, Harness deterministic selections, model-selected action calls, rounds, tool calls, and stop reasons.

Operational metrics remain separate from semantic results. The report records operational failure class, provider calls/attempts, model tokens where the public product envelope exposes them, provider latency, process wall-clock, observed model identifiers, tool calls, and stop reasons. Session operations currently do not expose per-turn token usage; `token_usage_case_coverage` makes that incompleteness explicit instead of estimating it.

## Freeze and observation rule

Before the first live v6 observation, the fixture tree, SHA-256 manifest, evaluator, tests, scoring, provider/model, seed, budgets, MCP coordinate, docs, and workflow must be committed and tagged `natural-language-e2e-v6-freeze`. The workflow revalidates that exact freeze before reading provider credentials.

After the first live observation, this surface is immutable. Operational retries may repeat the identical frozen coordinate only when the attempt is operationally incomplete and the failure remains recorded as operational evidence. Any semantic/evaluator/corpus/scoring change requires a new successor identity and separate Issue; historical v5 values remain historical and are never rewritten.
