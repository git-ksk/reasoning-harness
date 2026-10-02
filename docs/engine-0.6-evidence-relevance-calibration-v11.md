# Engine 0.6 candidate: evidence-target relevance calibration v11

Status: pre-freeze successor to immutable v10 FAIL. Historical v1-v10 fixtures, tags, observations, and scores remain immutable. Independent holdout authoring is prohibited until the first/only frozen v11 canonical calibration passes.

## Why v11 exists

v10 solved the operational defects from v9: Mistral, Groq, and Google all completed 56/56 with zero provider failures. The remaining failure was semantic architecture. A wrong primary binding could bypass the secondary stage, while a single secondary action (`safe_to_reject` / `safe_to_accept`) could authorize a hard disposition too directly. This produced unsafe rejection/acceptance across all three providers even though the transport was healthy.

v11 does not relax the safety gates and does not add fuzzy repair. It changes the ownership boundary.

## Two-key local qualification

The primary binding remains the same provider-neutral advisory pair:

- `target_binding = exact | different | unresolved`
- `relation_binding = exact | different | unresolved`

The primary prompt is tightened so embedded instructions/control text are ignored as untrusted data and cannot be mistaken for a factual relation mismatch.

Every case then invokes an **independent local qualification guard regardless of the primary route**. The guard never returns a final accept/reject action. It returns only:

- `target_support = supported | not_supported | unresolved`
- `relation_support = supported | not_supported | unresolved`
- `identity_mapping_risk = absent | present | unresolved`
- `ownership_scope_risk = absent | present | unresolved`
- `context_completeness_risk = absent | present | unresolved`
- `explicit_local_absence = present | absent | unresolved`

Risk fields are fail-closed. A risk that cannot be ruled out from the supplied local material is `unresolved`, not `absent`. Rename/alias/successor/cross-language/version-lineage uncertainty is identity risk. Shared or clipped ownership is ownership risk. URL-only, navigation-only when required context is missing, partial/truncated material, or omitted binding context is context risk. Factual disagreement, staleness, authority, verification, and answer sufficiency remain downstream concerns and do not create relevance risk by themselves.

## Materialization policy v7

The Harness owns final disposition.

A hard `Relevant` requires all of:

- primary `target=exact` and `relation=exact`;
- guard `target_support=supported` and `relation_support=supported`;
- all three guard risks are `absent`;
- the strict Harness-owned identity anchor floor is satisfied when required.

A hard `Irrelevant` requires a risk-free independent negative agreement:

- primary `target=different` plus guard `target_support=not_supported`; or
- primary non-exact target plus guard `target_support=not_supported` plus `explicit_local_absence=present`; or
- primary `target=exact, relation=different` plus guard `target_support=supported, relation_support=not_supported`.

Everything else is `Ambiguous`. In particular, an exact-target `relation=different` primary is no longer an unconditional rejection path.

A CanonicalUrl-only target-name match without a non-URL Harness-owned anchor is also a deterministic hard floor to `Ambiguous`, regardless of model output. Navigation/footer-only target mentions are not treated as this URL hard floor; a clearly scoped substantive other-product passage can still be rejected when both semantic keys agree.

## Transport and budgets

Primary binding uses `JsonSchema` with at most one provider-neutral `JsonObject` transport fallback. Local qualification independently uses `JsonSchema` with at most one equivalent `JsonObject` transport fallback. The fallback preserves task/system semantics and is not a semantic retry or fuzzy repair.

Every case invokes local qualification. Primary and qualification share the same 60,000 ms case deadline. Each stage permits at most two model calls, so the absolute per-case ceiling is four model calls only when both stages require transport fallback. Two consecutive operational failures still open the run circuit.

Google seed normalization introduced in v10 remains in the provider adapter.

## Calibration corpus

v11 contains 65 synthetic cases:

- all 56 v10 cases retained as regression/comparability cases with frozen local-qualification expectations;
- 9 fresh v11 cases covering injected relation-control text, indirect alias uncertainty, positive-looking shared ownership, comparison sibling scope, explicit generic absence, exact-target wrong relation, stale/contradictory-but-relevant evidence, URL-only hard-floor behavior, and a Harness-owned alias positive case.

Expected final dispositions: 19 Relevant, 24 Irrelevant, 22 Ambiguous. Twenty-two cases carry at least one expected fail-closed qualification risk; six contain explicit local absence. Production motivating product content remains excluded.

## Canonical provider roles and gate

Required:
- Mistral `ministral-8b-latest`
- Groq `openai/gpt-oss-120b`

Full non-gating replication:
- Google `gemini-3.5-flash-lite`

The first/only frozen canonical run uses tag `engine-0.6-evidence-relevance-calibration-v11-freeze`, attempt 1, and the exact checksummed surface. Each required arm must independently satisfy:

- 65/65 operational completion, provider failures 0, complete provider-attempt telemetry;
- local qualification expected/invoked 65/65 (no primary-route bypass);
- qualification risk misses 0;
- wrong-target/unexpected relevance retention 0;
- false relevance rejection 0;
- expected Relevant left ambiguous 0;
- utility misses 0;
- materialized disposition exact 65/65.

Primary proposal exactness, complete local-qualification exactness, and spurious risk blocks remain diagnostic. Final disposition plus missed safety risk are the acceptance boundary.

A v11 FAIL becomes immutable and is not rerun or rescored. Only a canonical PASS permits a fresh independently authored/frozen holdout. Runtime integration acceptance follows only after holdout PASS. PR #466 remains Draft through all stages.
