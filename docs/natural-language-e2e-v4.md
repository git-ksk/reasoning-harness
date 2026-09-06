# Natural-language investigation & session E2E v4

Issue #214 v4 is the **pre-observation successor** to v3 run `34031245504`. v3 was operationally complete (10/10), preserved all admission boundaries, exposed no unsupported structured or text-only assertions, and replayed zero external calls, but the evaluator mis-scored two successful session cases because it treated the nested serialized `ReasoningThreadEvent.kind` enum object as a flat string. v4 changes only that event-shape interpretation and adds a direct nested-serde regression test. The ten cases, provider/model/seed/token policy, product runtime at `defd404`, and correctness boundary remain unchanged.

## Frozen identities

- corpus: `natural-language-e2e-v4`
- evaluator/report: `reason-natural-language-e2e-v4`
- scoring: `natural-language-e2e-scoring-v4`
- natural output: `reason-natural-output-v3`
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

v4 makes **no raw-model comparison claim**. If a later evaluation adds a raw-model arm, it must use matched task/context and receive a new comparison/scoring identity.

Any post-observation change to cases, expected outcomes, scoring, evaluator semantics, or provider policy requires a successor identity rather than rewriting v4.
