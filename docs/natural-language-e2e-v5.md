# Natural-language investigation & session E2E v5

Issue #214 v5 is the **pre-observation successor** to v4, which was operationally complete but scored the wrong artifact boundary. v4 run `34031597896` completed 10/10 cases and passed session invalidation/replay, but `resolution_rounds[-1]` on investigation paths is the acquisition artifact before candidate regeneration, not the final state used by top-level finalization. v5 preserves the same ten cases, provider policy, and scoring while binding evaluation to `reason-natural-output-v4.final_outcome.artifact`. Product runtime commit `5069e6a` also makes session checkpoints use that same `final_outcome`.

## Frozen identities

- corpus: `natural-language-e2e-v5`
- evaluator/report: `reason-natural-language-e2e-v5`
- scoring: `natural-language-e2e-scoring-v5`
- natural output: `reason-natural-output-v4`
- session contract: `reason-session-v1`
- canonical provider policy: Mistral / `ministral-8b-latest`, base seed `41000`, max tokens `1024`, `1500 ms` inter-case pacing

Case N uses seed `41000 + N` (zero-based); a second model-backed session turn uses the next seed. The first live run may occur only after the committed fixture hash, evaluator, scoring policy, and deterministic preflight are frozen.

## Corpus

The 10 cases contain seven hypothesis-free natural-language investigation cases and three session cases. Investigation coverage includes a normal grounded lookup, relevant-capability selection among distracting read-only capabilities, stale evidence rejection, scope mismatch rejection, adaptive no-result follow-up, authority-claim mismatch rejection, and source-identity rejection. Session coverage includes adding evidence, correcting a prior premise, and resume/fork replay.

All external acquisition in this evaluation uses the committed deterministic read-only fixture resolver. It performs no network access. Live variability is therefore limited to model planning/candidate/rendering; freshness, scope, authority, and identity failures are intentionally constructed by the fixture.

## Scoring

The evaluator records target recall/omission, relevant capability selection, useful follow-up opportunities, irrelevant attempts, typed admission rejections, target grounding/false abstention, rounds/tool calls, model tokens/latency where the public product contract exposes them, process wall-clock latency, session invalidation/replay behavior, and operational failures separately.

Exposed-text correctness is scored mechanically. `finalization.text` must consist only of canonical `key = value` or `uncertain(key = value)` assertions, and each assertion must be supported by the final Harness artifact at the corresponding strength. Free-form extra text is an exposed-text contract violation. This metric is distinct from `factual_claims - covered_claims`, so a text-only assertion cannot hide behind structured claim coverage.

Session v1 does not expose per-turn provider token usage in its public operation envelope. The report therefore includes `token_usage_case_coverage`; single-turn investigation tokens are measured exactly and all session operations contribute wall-clock latency. The evaluator does not invent token estimates.

## Adoption gate

`correctness_boundary_violations` must be **0**. It includes unsupported structured claims, unsupported/free-form exposed assertions, unsafe grounding in expected-unknown safety cases, and session invalidation/replay violations. Utility metrics such as target recall, tool selection, useful follow-up, and false abstention are reported but may be imperfect.

Operational failures are excluded from semantic/correctness denominators and reported separately. An operationally incomplete attempt is not silently converted to `unknown`. The canonical observation candidate is the first operationally complete run under the frozen identity.

v5 makes **no raw-model comparison claim**. If a later evaluation adds a raw-model arm, it must use matched task/context and receive a new comparison/scoring identity.

Any post-observation change to cases, expected outcomes, scoring, evaluator semantics, or provider policy requires a successor identity rather than rewriting v5.
