# Harness Engine 0.6.0 release

Harness Engine 0.6.0 is the second independently versioned Engine source release under the split `engine-v*` namespace. It promotes the already-accepted target-local evidence semantics on integrated main without changing the frozen runtime behavior during release closeout.

## Coordinate

- package owner: `reasoning-harness-core`
- Engine version: `0.6.0`
- release tag: `engine-v0.6.0`
- release namespace: `engine-v*`
- release closeout: #472 / milestone #8
- pre-release integrated candidate: `cfaa5592273716b0eb470f46744653db617e784a`
- Reason CLI package version at this source release: `0.5.3`
- provider implementation crate version remains an internal coordinate

The published `reason-v0.5.3` binaries remain immutable on Harness Engine 0.5.0. Engine 0.6.0 is not retrofitted into those artifacts; a later Reason CLI release may adopt it explicitly under a new CLI coordinate.

## Accepted semantic delta

Engine 0.6.0 adds three Harness-owned stages around external/context evidence while preserving existing hard verification authority:

1. **Target-local evidence need (#461).** Before acquisition, each target has a typed evidence requirement and acquisition disposition. Model proposals are advisory; Harness-owned floors prevent a context/local proposal from weakening explicit external/current/trusted-verification requirements.
2. **Evidence-target semantic relevance and relation qualification (#462/#468).** Retrieved material is assessed against exact target identity, coarse relation, and risk under Harness-owned composition. Passing relevance creates no verification authority; ambiguous/wrong-target material fails closed.
3. **Source-attributed qualified prose (#463).** Relevant admitted prose may support a claim only as an attributed statement about what the bound source says. Exact target/evidence/source/span identity, authority ceilings, conflict state, citation exposure, and replay persistence are Harness-owned. Attributed prose alone cannot create external-world `Known` or `Supported` truth.

Supporting PR #460 adds a bounded provider-neutral JSON-Schema -> JSON-object transport fallback. It preserves the original task/system/budgets/seed/reasoning preference and still requires typed parsing/validation; it does not repair semantics or create evidence/authority.

## Release evidence

The release uses already-observed immutable evidence rather than a new release-tuning surface.

### #461 evidence need

- calibration v3 freeze: `engine-0.6-evidence-need-calibration-v3-freeze`
- calibration run: `35957170730`
- independent holdout: `engine-0.6-evidence-need-holdout-v1-freeze`
- canonical holdout run: `35965160995`
- result: Mistral + Google 26/26 materialized mode and acquisition; correctness violations 0; utility misses 0; provider failures 0

### #462/#468 relevance and relation

- final accepted freeze: `engine-0.6-evidence-relevance-holdout-v12-freeze`
- freeze commit: `78c6894e871d9aa4dd79aef0a30c95647c075c4d`
- canonical run: `37218652869`, attempt 1
- required providers: Mistral, Google, Groq
- result: 26/26 each; authority failures 0; identity/risk failures 0; materialization 26/26; wrong-target Relevant 0; false relevance rejection 0; Relevant-left-Ambiguous 0; utility misses 0

Historical failed/operational relevance observations remain immutable and are not reinterpreted as passes.

### #463 source attribution

- development v6 freeze: `engine-0.6-source-attribution-development-v6-freeze`
- development run: `37294100665` — PASS
- independent holdout: `engine-0.6-source-attribution-holdout-v2-freeze`
- freeze commit: `bb8d43616ee60370c099db6dda35f9a7f6d209f4`
- canonical run: `37326666360`, attempt 1 — PASS
- required providers: Mistral, Google, Groq
- result: 18/18 each; useful attribution 6/6; citation coverage 100%; provider failures 0; truth-promotion/source-binding/renderer-only exposure/semantic-strengthening/wrong-target/missing-citation/replay-refetch hard gates all 0

The earlier source-attribution holdout-v1 operational FAIL remains immutable and was not rerun, rescored, relabeled, or retagged.

## Release boundary

This release advances only the Engine source coordinate. It does not publish new Reason CLI binaries, does not change historical machine-contract identities, and does not rewrite any prior release or frozen evaluation artifact. CLI adoption of Engine 0.6.0 is a separate product/lifecycle decision.
