# Natural-language E2E v11 — trigger-conditioned v0.4.1 successor

Issue #254 defines `natural-language-e2e-v11` as the fresh full-product successor after the frozen v10 Mistral observation exposed a planner trigger miss before the v0.4.1 / Issue #249 continuation could run.

## Historical boundary

Frozen v9 and v10 are immutable historical observations and are not rerun, rescored, repaired, or used as tuning data.

- v9: the dedicated Umber lane reached `cache -> typed no_result` but did not continue to the registry.
- v10 canonical Mistral run `34125135760`: 11/11 cases completed with zero correctness-boundary violations and zero operational failures, but the dedicated Cobalt lane recalled the target while executing no action (`planner_calls=4`, `actions=0`, `round_budget`). The #249 trigger was therefore not reached and its live mechanism effect was inconclusive.

The released product under evaluation remains exactly:

- tag: `v0.4.1`
- commit: `29a9e4be6273dbffeda324e15517dc64930ad315`
- natural output contract: `reason-natural-output-v4`
- MCP adapter: `mcp_readonly_v3`

No production semantics are changed for v11.

## Why there is no controlled live intervention lane

The released `reason` CLI has no supported input that injects an investigation action or pre-populated `InvestigationState`. Adding such an injector only for measurement would no longer be an exact released-v0.4.1 product observation and would risk restating the deterministic #249 unit regression instead of measuring the natural-language product path.

v11 therefore remains observational and uses multiple independently frozen fresh trigger cases instead.

## Corpus

v11 keeps the ordinary whole-product E2E families and expands the dedicated follow-up surface:

- 1 unique mechanically selectable grounded acquisition
- 1 ambiguous tool-selection case
- 1 stale-evidence rejection case
- 1 scope rejection case
- 3 fresh observational `cache -> no_result -> registry` follow-up cases
- 1 authority rejection case
- 1 source-identity rejection case
- 1 pinned read-only GitHub MCP generic-content non-promotion case
- 3 session add/correct/resume-fork cases

Total: 13 cases, 10 investigation + 3 session.

All case IDs, task strings, target keys, fresh markers, config source references, and base seed are mechanically disjoint from the observed predecessor surfaces v1-v10.

## Three separate follow-up measurements

The three follow-up cases are not scored as one binary gate.

### 1. Trigger reachability / planner utility

Denominator: all 3 frozen follow-up cases.

A case is `trigger_exposed` only when its first executed relevant capability is the configured cache and that action returns typed `no_result`.

`trigger_reachability_rate = trigger_exposed_cases / 3`.

A trigger miss is planner-utility data. It is not a #249 mechanism failure and does not make the observation non-scorable.

### 2. Conditional #249 mechanism conformance

Denominator: trigger-exposed cases only.

A trigger-exposed case is mechanism-conformant when the immediately following action is the configured exact-target registry and `harness_no_result_followup_selections == 1` records the Harness-owned continuation.

The evaluator deliberately does **not** require total `planner_calls == 1`: invalid stochastic attempts before the cache are trigger-reachability data, and later downstream work must not be confused with whether the post-`no_result` selector itself called the planner.

If zero cases expose the trigger, the mechanism classification is `inconclusive`, never success or failure. If one or more expose it, the exact numerator/denominator is reported descriptively.

### 3. Downstream utility

Registry selection is not sufficient utility evidence. The report separately records whether the registry produced admitted evidence / verification progress and whether the final target became grounded.

## Measurement-validity semantics

v11 applies the separation requested by Issue #247:

- **measurement observability**: the frozen study executed and emitted all case reports;
- **path exposure**: whether MCP or the no-result trigger was actually exercised;
- **hard correctness**: unsupported exposed/structured claims, missed insufficiency, identity unsafe admission, and related authority-boundary failures;
- **operational completeness**: process, typed-action, and generation failures;
- **planner utility**: target recall, tool selection, trigger reachability, false abstention;
- **conditional mechanism conformance**: only over trigger-exposed cases;
- **downstream utility**: useful evidence/verification and grounded target recovery.

Utility success, MCP exposure, trigger reachability, and #249 mechanism success are not prerequisites for `measurement_validity_passed`. They remain measured product outcomes.

The live workflow still fails its report gate for hard correctness or operational-completeness failures; those failures are preserved in the uploaded artifact rather than rerun away.

## Frozen provider policy

- provider: `mistral`
- model: `ministral-8b-latest`
- base seed: `57000`
- max tokens: `1024`
- inter-case delay: `1500 ms`

The first live-case launch is canonical. Failures before live-case launch that are purely pre-live infrastructure may be retried; after the live boundary is entered, no rerun or tuning is allowed under the v11 identity.

## Pre-observation status

No v11 live observation has been performed yet. Corpus, evaluator, workflow, tests, checksums, and documentation must be frozen before credentials are used.
