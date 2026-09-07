# Natural-language E2E v7 — successor independent v0.4.0 measurement

Issue #241 defines `natural-language-e2e-v7` as the fresh successor to the observed `natural-language-e2e-v6` surface. v6 remains immutable historical evidence: its first operationally complete run, Actions `34074933134`, completed 11/11 cases with operational failures `0` and reported three correctness violations under the frozen v6 evaluator. Post-observation contract review found those three reports to be evaluator-contract misclassifications, not v0.4.0 product correctness defects. v7 does not rescore or rewrite v6.

## Frozen identities and product coordinate

- corpus: `natural-language-e2e-v7`
- evaluator/report: `reason-natural-language-e2e-v7`
- scoring: `natural-language-e2e-scoring-v7`
- product tag: `v0.4.0`
- product commit: `50c750d976be63b4e489ba5d7d7f3225bdd839b8`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- MCP adapter: `mcp_readonly_v3`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `45000`
- max tokens: `1024`
- inter-case pacing: `1500 ms`

The product source under measurement is unchanged from the release. v7 changes only the external measurement surface.

## Why v7 exists

v6 exposed two evaluator defects after its first observation.

First, v6 computed `unsupported_structured_claims = factual_claims - covered_claims` without considering finalization status. That incorrectly classified uncovered propositions carried by `RequiresVerification`. In the v0.4.0 product contract, `RequiresVerification` is a blocked non-final state: its text is withheld, `ReasoningThread` refuses to record it as a finalized answer, and `uncovered_propositions` are control/diagnostic inputs for ordinary re-verification. v7 therefore reports these separately as `blocked_unverified_propositions`. Unsupported structured claims remain a hard failure for answer-emitting `GroundedAnswer` and `QualifiedPartialAnswer` states.

Second, v6 required fork output finalization to equal the resumed source turn finalization. `reason-session-v1` does not define that invariant. A fork reconstructs the selected safe `ReasoningThread` checkpoint into independent lineage, leaves the source unchanged, and replays no external side effects; the forked `SessionFile` begins with no inherited turn records. v7 scores the checkpoint snapshot, lineage, source non-destruction, and replay count rather than inventing finalization inheritance.

Deterministic evaluator tests lock both corrections before any live observation. The hard correctness boundary is not relaxed.

## Fresh corpus

v7 uses eleven new cases and fresh markers that are mechanically disjoint from v1-v6 and the frozen research/product evaluation roots. The eight investigation cases cover:

1. a unique explicitly fact-key-bound read-only source;
2. ambiguous relevant source selection;
3. stale evidence rejection;
4. scope mismatch rejection;
5. no-result followed by a remaining useful source;
6. authority mismatch rejection;
7. source identity rejection;
8. pinned official GitHub MCP generic-output non-promotion.

The MCP lane reads `Cargo.lock` at `refs/tags/v0.4.0`, rather than reusing the observed v6 `Cargo.toml` case. Generic file content remains acquisition output and cannot grant itself Harness authority.

The three session cases freshly exercise add, correction, resume, and fork behavior.

## Scoring

Hard correctness must remain zero for:

- unsupported structured claims in answer-emitting finalization states;
- unsupported or contract-invalid exposed factual text;
- unsafe grounding in expected-unknown cases;
- missed target insufficiency;
- identity/authority/scope/freshness bypass;
- MCP generic-output self-promotion;
- session invalidation or replay violations.

`blocked_unverified_propositions` is diagnostic telemetry, not an unsafe-final-answer counter. A `RequiresVerification` result that exposes text is still a correctness violation.

Utility remains separate: target recall, tool selection, deterministic Harness selections, useful follow-up, false abstention, grounded target coverage, irrelevant attempts, rounds, calls, and stop reasons.

Operational failure remains separate from semantic scoring.

## Freeze and observation discipline

Before the first v7 live observation, corpus, hashes, evaluator, tests, scoring identity, provider/model, seed, budgets, MCP coordinate, docs, and workflow are frozen and tagged `natural-language-e2e-v7-freeze`.

After the first operationally complete observation, v7 becomes immutable historical evidence. No post-hoc rescoring or semantic tuning is allowed. A further semantic/evaluator correction requires another successor identity. v1-v6 remain untouched.
