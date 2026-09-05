# Product MCP external-information successor evaluation

Issue #206 defines the successor to the frozen `product-external-info-v1` observation from Issue #203. The historical v1 result is evidence, not a tuning set: run `33974104359` remains fixed at `0.75` expected-grounded target coverage for the scored Harness+MCP arm, with all frozen safety gates passing. The post-observation `ec79570` change was reverted and is not replayed against v1.

## Frozen identities

Before any successor provider observation, the repository freezes:

- corpus: `product-external-info-v2`
- case schema: `product-external-info-case-v2`
- semantic/finalization contract: `verified-target-finalization-successor-v2`
- four-arm contract: `product-external-info-four-arm-v2`
- scoring contract: `product-external-info-scoring-v2`
- evaluator report: `reason-product-external-info-v2`
- comparison contract: `single-acquisition-four-arm-target-finalization-v2`
- SHA-256 manifest: `fixtures/product-external-info-v2.sha256`
- baseline main: `a365a46d5fa948063e9ac745ad14646c23456ede`

The successor contains 21 cases, again arranged as 7 capability families × 3 cases, but its case IDs and target key/value pairs are disjoint from v1. The six `product-dogfood-v1` cases and all historical identity holdouts remain immutable and are not reused for tuning.

## Four-arm comparison

The evaluation compares:

1. `raw_model_no_external` — model only, no Harness and no external information.
2. `harness_no_external` — Harness, no external information.
3. `raw_model_with_external` — no Harness; the model receives the acquired external observation snapshot as untrusted context.
4. `harness_with_mcp_external` — Harness using the same acquired external snapshot through normal Harness-owned admission, verification, and finalization.

The primary product comparison is arm 3 versus arm 4. Arms 1 and 2 are ablations.

For arms 3 and 4, each case performs one real `mcp_readonly_v1` acquisition. The decoded acquisition result is retained as the frozen per-case observation set. Arm 3 receives that observation set as model context without Harness admission or authority. Arm 4 replays the same acquisition result through the ordinary Harness evidence-admission and verification path. No second retrieval is permitted, so provider, model, seed, token limit, retrieval opportunity, and external snapshot are matched as closely as the current evaluator can enforce.

`mcp_readonly_v1` protocol/session semantics are unchanged by Issue #206. Negotiated/session stdio compatibility remains Issue #204.

## Finalization semantics under evaluation

Issue #206 does not add entity-specific correctness rules. It evaluates the already-general target-scoped finalization machinery used by the product path:

- `canonical_verified_target_answer`
- `canonical_verified_target_partial_answer`
- `recover_verified_target_renderer_downgrade`
- `canonical_verified_target_reject_partial_answer`

These helpers do not promote an artifact-global verdict. They expose only exact Harness-owned requested targets that satisfy their existing typed authority and isolation checks. Model/planner output remains untrusted; the Harness continues to own identity sufficiency, evidence admission, verification, freshness, scope, authority, conflict handling, stopping/budget, terminal safety, and final factual exposure.

## Scoring

The frozen scoring contract records at least:

- expected-grounded target coverage
- false target abstention
- expected-unknown preservation
- unsupported grounded claims
- missed target insufficiency
- external acquisition attempts and successes
- verification successes
- identity-unsafe admission
- stale, authority, scope, and conflict rejection
- typed operational failures
- model latency and token usage from per-case call observations
- external call and elapsed-time telemetry from the Harness external arm

Typed operational failures remain outside the semantic denominator.

The Harness safety gate remains fail-closed:

- unsupported grounded claims = `0`
- missed target insufficiency = `0`
- identity-unsafe admission = `0`
- MCP-output authority self-promotion = `0`
- expected-unknown preservation = `1.0`

Coverage `1.0` is a utility goal, not permission to weaken any safety gate.

## Frozen first observation conditions

The first valid successor provider observation is declared before running it:

- provider: `mistral`
- model: `ministral-8b-latest`
- seed: `26000`
- max tokens: `1024`

The first valid run under this freeze becomes the canonical v2 observation. After that run, changing case selection, expected outcomes, semantics, or scoring requires another successor identity rather than rewriting v2.

## CI discipline

`product-external-info-successor-freeze.yml` is credential-free. It validates the v2 manifest and evaluator wiring, verifies `product-external-info-v1` and `product-dogfood-v1` against their existing manifests, and asserts that `mcp_readonly_v1` is unchanged from the v2 baseline main.

`product-external-info-successor-live.yml` is label-gated and revalidates the freeze before provider credentials are exposed. Its live safety checks gate only the Harness arm; raw+external is intentionally allowed to reveal unsafe behavior so arm 3 versus arm 4 remains an informative product comparison.
