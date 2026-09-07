# Natural-language E2E v11 — canonical v0.4.1 Mistral result

Issue #254 froze `natural-language-e2e-v11` as a fresh full-product successor after immutable v10 missed the predecessor trigger for the released v0.4.1 / Issue #249 typed-`no_result` continuation. v11 keeps trigger reachability, conditional mechanism conformance, and downstream utility as separate observations.

## Canonical coordinate

- Actions run: `34129798774`, attempt 1
- freeze tag: `natural-language-e2e-v11-freeze`
- freeze commit: `a758af17a998493c1005702365b100e05b05f95d`
- released product: `v0.4.1` / `29a9e4be6273dbffeda324e15517dc64930ad315`
- provider/model: Mistral / `ministral-8b-latest`
- base seed: `57000`
- artifact: `10021729999`
- artifact digest: `sha256:5b84e62006e73d4bb9ee93a4793bad9b65d67f12b7a404ec325c55d14d2d13e3`
- result JSON digest: `sha256:f108582cddffb0eafd3d7ff122c1f27b22d30225f226b6aba69de8acc6e28383`
- attempt-marker digest: `sha256:f7b018d5621df82dd5a0e4d71fa25bbb19b24940957884b926db1897df747c34`

The preserved attempt marker records `live_case_launch_boundary_entered=true` for `unique-arclume-endpoint`, with GitHub run `34129798774`, attempt `1`. This is therefore the canonical v11 observation and must not be rerun, rescored, repaired, or tuned in place. The GitHub Actions run preserves the execution log, while the uploaded artifact preserves the freeze/preflight evidence, canonical result, stdout copy, and attempt marker.

## Aggregate result

- completed cases: `13/13`
- hard correctness gate: **PASS**
- measurement observability / validity: **PASS**
- operational completeness: **PASS**
- report gate: **PASS**
- correctness-boundary violations: `0`
- exposed-text contract violations: `0`
- unsupported structured claims: `0`
- unsupported exposed assertions: `0`
- operational failures: `0`
- process operational failures: `0`
- typed operational action failures: `0`
- generation failures: `0`
- target recall: `0.60`
- tool-selection success: `0.80`
- grounded-target coverage across investigation cases: `0.00` (`0/10`)
- false abstentions: `6`
- required admission rejections observed: `4/4`
- admission behavior coverage: `1.00`
- MCP path exposure: `1/1`
- session persistence valid: `true`
- session fork state valid: `true`
- session external calls replayed: `0`

The run is therefore fully scorable under the frozen v11 semantics. Weak planner/grounding utility remains visible as measurement data rather than invalidating correctness, operations, or path observability.

## Trigger reachability / planner utility

The three predeclared fresh no-result follow-up cases produced **1/3 trigger exposure** (`0.3333`).

### `adaptive-emberquill-owner`

- target recalled: `true`
- planner calls: `1`
- first relevant capability: `emberquill-owner-cache`
- first relevant typed status: `no_result`
- trigger exposed: `true`
- selected sequence: `emberquill-owner-cache -> emberquill-owner-registry`
- actions executed: `2`
- stop reason: `resolved`

### `adaptive-fluxmere-owner`

- target recalled: `true`
- planner calls: `4`
- actions executed: `0`
- trigger exposed: `false`
- trigger miss reason: `no_relevant_action`
- stop reason: `round_budget`

### `adaptive-glyntide-owner`

- target recalled: `true`
- planner calls: `4`
- actions executed: `0`
- trigger exposed: `false`
- trigger miss reason: `no_relevant_action`
- stop reason: `round_budget`

The two misses are planner-selection utility observations. They are not counted as Issue #249 mechanism failures.

## Conditional Issue #249 mechanism conformance

The mechanism denominator is trigger-exposed cases only: **1**.

For `adaptive-emberquill-owner`:

- cache returned typed `no_result`;
- the immediately following relevant capability was the single configured registry;
- `harness_no_result_followup_selections == 1`;
- the action sequence contained no duplicate or reordered relevant capability;
- mechanism classification: `conformant`.

Aggregate conditional mechanism result: **1/1 conformant**, classification `all_exposed_conformant`.

This is direct observational evidence that released v0.4.1 executed the #249 continuation correctly in the one natural-language case that reached its predecessor trigger. The denominator is one, so this result is descriptive for this frozen workload/model slice and is not a broad model-level effect-size claim.

## Downstream utility

The trigger-exposed Emberquill registry action produced one admitted evidence item and `verification_progress`; `downstream_followup_useful=true`. Thus downstream utility was **1/1 among trigger-exposed follow-up opportunities**, or **1/3 across the three predeclared follow-up cases**.

However, the target did not become grounded in Emberquill, and none of the three follow-up targets finished grounded (`0/3`). The Emberquill finalization remained `unresolved`, so mechanism conformance must not be conflated with grounded-answer success.

## Full E2E observations

### Correctness and admission

All 13 cases completed with zero correctness-boundary violations. The four required rejection contracts were exercised: stale evidence, scope expansion, authority-claim mismatch, and untrusted source. The MCP lane additionally rejected evidence for `missing_observation_time`. No rejected or opaque evidence self-promoted into authority.

### MCP

`mcp-v11-v041-cargo-nonpromotion` invoked the pinned GitHub MCP lane once. `mcp_path_exposed=true`, target recall was false, and `mcp_output_authority_self_promotion=0`. Under the v11/#247 semantics, path exposure is reported independently from target utility; the lane is observable without claiming that the target was successfully recalled or grounded.

### Sessions

All three session contracts preserved state semantics and replayed no external calls. `session-add-jadewisp` and `session-resume-fork-lumera` reached grounded answers. `session-correct-krysal` preserved correction/persistence invariants but finished unresolved and contributes one false abstention. Session persistence and fork-state aggregate checks both passed.

### Utility residuals

Natural-language utility remains the principal residual:

- target recall: `0.60`;
- tool-selection success: `0.80`;
- investigation grounded-target successes: `0/10`;
- false abstentions: `6`;
- two of three fresh no-result follow-up cases recalled their targets but never selected any action before `round_budget`.

These residuals do not weaken the observed #249 conditional mechanism conformance; they identify the next optimization surface as planner/action selection and downstream grounding rather than the post-`no_result` continuation itself.

## Interpretation

The canonical v11 result supports the following statements:

1. released v0.4.1 preserved the frozen v11 correctness boundary and operational completeness in this run;
2. v11 successfully exposed the #249 predecessor trigger in `1/3` fresh follow-up cases;
3. conditional on that exposure, released v0.4.1 performed the exact-target Harness continuation correctly in `1/1` observed case;
4. the selected registry created admitted evidence and verification progress, but did not produce a grounded target;
5. the two trigger misses remain planner/action-selection utility data, not #249 mechanism failures;
6. MCP non-promotion and session persistence/fork boundaries remained intact under the fresh full-product successor.

Issue #254 is therefore complete as a trigger-conditioned successor measurement. Frozen v9, v10, and v11 remain immutable observations; future work on planner selection or grounded-answer utility requires a new issue/successor rather than rerunning or tuning v11.
