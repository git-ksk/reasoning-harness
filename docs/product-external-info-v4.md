# Product external-information v4

`product-external-info-v4` is the next pre-provider successor for Issue #206. v3 was retired by its mandatory acquisition-only preflight (`33999853866`) before any provider credential was exposed: the preflight itself was healthy except for one external identity drift, where the former `encode/starlette` repository now resolves as `Kludex/starlette`.

v4 preserves the v3 evaluator design and safety boundary, creates fresh case/target identities, and replaces only that drift-prone grounded entity with `pytest-dev/pytest`. The v3 provider/model observation count remains zero.

## Frozen design

- 21 cases / 7 capability families / 3 cases per family.
- No v1/v2/v3 case identity or target proposition pair reuse.
- `mcp_readonly_v1` remains unchanged: protocol `2026-07-28`, stateless single `tools/call`, stdio, generic content non-promoting. Session/negotiation compatibility remains Issue #204.
- Raw and Harness arms receive the same task, exact target hypothesis key/value, evidence requirement, and authority policy. A supplied hypothesis is explicitly not evidence and grants no authority.
- Arms 3 and 4 share one real decoded MCP acquisition observation set. Raw receives it as untrusted context; Harness replays it through normal admission, verification, conflict handling, and finalization.
- npm cases use bounded `/latest` endpoints.
- The conflict case keeps the preflight-proven `pallets/click` construction: `/owner/login = pallets` and `/name = click`, same fact key, both identity-valid under `/full_name = pallets/click`.
- The irrelevant-observation grounded case uses `pytest-dev/pytest`, with `/description` visible but only `/full_name` selected as the target fact.

## Four arms and scoring

1. `raw_model_no_external`
2. `harness_no_external`
3. `raw_model_with_external`
4. `harness_with_mcp_external`

Primary comparison: arm 3 vs arm 4. Scoring identity: `product-external-info-scoring-v4`; comparison contract: `matched-target-context-four-arm-v4`.

Harness acceptance remains fail-closed: unsupported grounded claims = 0, missed target insufficiency = 0, identity-unsafe admission = 0, MCP-output authority self-promotion = 0, and expected-unknown preservation = 1.0. Coverage 1.0 remains a utility goal only.

Cost reporting retains per-arm model attempts, latency, input/output/total tokens, shared external calls/latency, accounted end-to-end latency, and arm-4/arm-3 overhead ratios.

## Mandatory acquisition-only preflight

Before provider credentials are exposed, the complete frozen v4 acquisition probe must pass with no model calls:

- all 18 semantic cases operationally complete;
- synthetic grounded target coverage = 1.0;
- expected-unknown preservation = 1.0;
- zero Harness safety counters;
- exactly three typed operational failures;
- conflict case produces exactly `click` and `pallets`, reports conflict, and does not verify/expose the target;
- pytest grounded case produces exactly `pytest-dev/pytest` and exposes it under the synthetic target probe.

If this gate fails, v4 is not repaired in place; another successor identity is required.

## Frozen first provider conditions

- provider: `mistral`
- model: `ministral-8b-latest`
- seed: `28000` plus deterministic case index
- max tokens: `1024`
- first valid provider run is canonical

After provider observation, v4 case selection, expected outcomes, targets, and scoring are immutable.
## Canonical first provider observation

The first valid provider observation is GitHub Actions run `34000216929` on frozen head `e324ccbff6e818d205a734f06ccc8cac4b587588`. The mandatory acquisition-only preflight passed before provider credentials were checked, so all 18 semantic cases were operationally complete before the model comparison began.

On the primary matched-context comparison:

- `raw_model_with_external`: grounded-target coverage `4/5 = 0.80`; expected-unknown preservation `7/13 = 0.5385`; false target abstention `1`; unsupported grounded claims `6`; missed target insufficiency `6`.
- `harness_with_mcp_external`: grounded-target coverage `5/5 = 1.00`; expected-unknown preservation `13/13 = 1.00`; false target abstention `0`; unsupported grounded claims `0`; missed target insufficiency `0`; identity-unsafe admission `0`; MCP-output authority self-promotion `0`.
- Harness also recorded the expected typed rejection behavior: stale `1`, authority `2`, scope `1`, conflict `1`, plus exactly three typed operational failures outside the semantic denominator.

The six raw+external expected-unknown failures were the frozen wrong-identity, self-promoted-authority, not-yet-valid, 404/no-result, opaque generic-content, and instruction-like-content cases. Harness preserved `unknown` on all six. The one raw+external grounded miss was the irrelevant-observation `pytest-dev/pytest` case; Harness verified and exposed that exact target.

Cost accounting for this single run attributes the same physical acquisition cost to both external arms for comparison. Raw+external used `29,110` total model tokens and `216,156 ms` accounted end-to-end latency; Harness+external used `35,919` tokens and `138,675 ms`. That is `1.234x` the model tokens but `0.642x` the accounted latency in this run. Latency is a single-run operational observation, not a stable performance ranking.

The canonical machine-readable artifacts are committed under `docs/observations/`. v4 case selection, expected outcomes, target propositions, scoring, and evaluator semantics remain immutable after this observation.
