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

## First frozen v2 observation: diagnostic only

The first provider observation ran from the frozen commit `103b898cc6fe6f41b4fd30c8debdef08f9d5ec7c` in GitHub Actions run `33978554958` with Mistral `ministral-8b-latest`, seed `26000`, and `1024` max tokens. The 21 cases completed, but the frozen safety assertion failed, so this run is retained as a diagnostic rather than accepted as the canonical product comparison.

The Harness+MCP arm exposed all 5/5 scored expected-grounded targets (`1.00` coverage) with `0` unsupported grounded claims, `0` identity-unsafe admissions, and `0` MCP-output authority self-promotions. However, expected-unknown preservation was 11/12 (`0.9167`) because `conflict-qualified-facts-flask` exposed its target. Inspection showed that the second frozen acquisition profile had an identity assertion that could never match the fetched object, so it emitted no conflicting fact. The Harness therefore saw one valid verified target rather than two conflicting facts; target-scoped partial recovery behaved consistently with the actual admitted evidence. This is a fixture-construction defect, not evidence that the conflict policy selected one of two admitted conflicting facts.

The same run also exposed a primary comparison fairness defect: Harness candidate generation received the exact target proposition through Harness-owned hypotheses/evidence requirements, while the raw arms received only the natural-language task. Raw scoring nevertheless required exact internal proposition-key equality. Therefore the observed raw+external coverage `0.00` and its unsupported-grounded count cannot be used as a fair arm-3-vs-arm-4 product comparison.

One semantic case, `identity-npm-react-dom-vs-react`, was operationally incomplete because the full npm registry document exceeded the fixture server's 8 MiB bounded-response limit and was typed `policy_denied`; it was correctly excluded from the semantic denominator.

The machine-readable diagnostic is [`observations/product-external-info-v2-mistral-ministral-8b-seed-26000-2026-09-06.json`](observations/product-external-info-v2-mistral-ministral-8b-seed-26000-2026-09-06.json). `product-external-info-v2` remains immutable after this observation. Correcting the fixture and comparison contract requires a new corpus/evaluator identity.
