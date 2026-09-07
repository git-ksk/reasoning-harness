# Natural-language E2E v10 — canonical v0.4.1 Mistral result

Issue #252 froze `natural-language-e2e-v10` as a fresh successor to immutable v9 in order to observe released v0.4.1 / Issue #249 exact-target continuation after typed `no_result` without modifying production semantics.

## Canonical coordinate

- Actions run: `34125135760`, attempt 1
- freeze tag: `natural-language-e2e-v10-freeze`
- freeze commit: `6b3c4e1b3aed09ff9af1b5ad12e48c1b88e396de`
- released product: `v0.4.1` / `29a9e4be6273dbffeda324e15517dc64930ad315`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `53000`
- artifact: `10019908126`
- artifact digest: `sha256:b269af8e1284c60a760bfe5cf2e5f4016f0a9d75915236f730cbd81bc0238657`
- result JSON digest: `sha256:7a04a8fafc64b257bffcfa3304eccc5db08439e9e7f4b96a021e7ae427850b0e`

The preserved attempt marker records `live_case_launch_boundary_entered=true`; this is therefore the canonical v10 observation despite the failed adoption gate. v10 must not be rerun, rescored, repaired, or tuned in place.

## Aggregate result

- hard correctness gate: **PASS**
- correctness-boundary violations: `0`
- operational failures: `0`
- completed cases: `11/11`
- measurement-validity gate: **FAIL**
- adoption gate: **FAIL**
- target recall: `0.50`
- tool-selection success: `0.75`
- grounded-target coverage: `0.00`
- false abstentions: `4`
- admission rejection coverage: `3/4`
- MCP live coverage under the frozen validity definition: `0/1`
- adaptive follow-up coverage: `0/1`
- `harness_no_result_followup_selections`: `0`

The failure is therefore an exercised-path/utility result, not a correctness or operational failure.

## Issue #249 lane

The dedicated `adaptive-cobalt-owner` case recalled `cobalt.routing.owner`, but Mistral never selected the configured cache action:

- target recalled: `true`
- planner calls: `4`
- actions executed: `0`
- cache invocation: `0`
- typed `no_result`: not observed
- Harness exact-target follow-up selection: `0`
- registry invocation: `0`
- stop reason: `round_budget`
- false abstention: `1`
- correctness violation: `0`
- operational failure: `0`

Accordingly, v10 is **trigger-missed / censored for the live effect of Issue #249**. The run does not show that v0.4.1 continuation failed, because its predecessor condition was never reached.

This differs from frozen v9: `adaptive-umber-owner` executed the cache and observed typed `no_result`, then failed to invoke the registry before `round_budget`. v9 supplied the motivating post-trigger residual; v10 did not reproduce that trigger exposure and therefore cannot estimate the post-trigger effect.

## Other validity misses

The MCP case invoked `github-v041-changelog` twice and correctly avoided authority self-promotion, but target recall was false. Because the frozen validity contract required both target recall and a non-operational MCP invocation, `mcp_lane_exercised` remained false. The identity-rejection lane likewise failed to exercise its intended target/action. Neither miss created a correctness-boundary or operational failure.

## Interpretation

The canonical result supports the following statements only:

1. released v0.4.1 preserved the frozen v10 correctness boundary in this run;
2. the run had zero operational failures;
3. natural-language planner/target utility remained weak enough to prevent several intended measurement lanes from being exercised;
4. the live effect of Issue #249 remains unresolved because its typed-`no_result` trigger was not reached.

It does **not** support either “Issue #249 fixed the v9 residual” or “Issue #249 failed to fix the v9 residual.”

Issue #254 owns the fresh successor design. It must separate observational trigger reachability from conditional post-trigger mechanism evidence and keep any controlled intervention explicitly distinct from ordinary natural-language planner behavior.
