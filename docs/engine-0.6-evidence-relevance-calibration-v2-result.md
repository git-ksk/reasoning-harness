# Engine 0.6 evidence-target relevance calibration v2 result

Status: frozen v2 observation completed operationally and passed the correctness gate on both provider arms. Utility still failed, so #462 semantics remain unfrozen and no independent holdout may be authored yet.

## Frozen identity

- freeze tag: `engine-0.6-evidence-relevance-calibration-v2-freeze`
- candidate commit: `e7ffdbe27572d7113410b5f5187a84570d3dd4a4`
- GitHub Actions run: `35992854291`
- suite: `evidence-relevance-calibration-v2`
- cases: 26
- seed: `4622602`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

Preflight and both live provider arms succeeded. The combined final gate failed only on utility.

## Raw v2 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 26/26 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 22/26 (84.62%) | 24/26 (92.31%) |
| materialized exact accuracy | 23/26 (88.46%) | 25/26 (96.15%) |
| wrong-target / unresolved-binding retained as relevant | **0** | **0** |
| false rejection on expected-relevant | 0 | 0 |
| expected-relevant left ambiguous | 0 | 0 |
| utility misses | **3** | **1** |
| ambiguous dispositions | 3 | 7 |
| deterministic safety overrides | 0 | 2 |
| lexical baseline exact accuracy | 12/26 (46.15%) | 12/26 (46.15%) |
| lexical baseline wrong-target relevance retention | 8 | 8 |
| lexical baseline expected-relevant misses | 2 | 2 |
| model calls | 26 | 26 |
| provider attempts | 26 | 26 |
| total tokens | 11,707 | 12,221 |
| model-call latency total | 15,573 ms | 21,895 ms |

v2 therefore fixes the v1 correctness failure and materially improves ambiguity handling, but it does not yet satisfy the frozen zero-utility-miss acceptance criterion.

## Remaining Mistral utility misses

Mistral still returns `irrelevant` where the expected disposition is `ambiguous` for:

- `21_unknown_rename`: the source describes another name but explicitly leaves whether it replaces the target unresolved;
- `22_partial_identity`: the material uses a partial product identity that cannot be bound to the exact target;
- `25_insufficient_local_passage`: the exact target is present, but the locally relevant release-note bullet is omitted from the supplied passage.

The v2 prose instruction explicitly says these shapes are ambiguous, but a small model can still collapse semantic uncertainty into a destructive rejection.

## Remaining Google utility miss

Google returns `ambiguous` for `16_broad_landing_no_support`, where the frozen label is `irrelevant`: the page is a generic cloud-services landing page and contains no target-specific local support.

This is safe but unnecessarily retains a candidate that the calibration considers affirmatively non-target-local.

## v3 design conclusion

Further prose-only tuning would make the model responsible for a subtle three-way final policy decision. Instead, v3 should reduce model-owned authority in the relevance decision itself.

The model-facing contract should decompose semantic assessment into typed advisory bindings, for example:

- target binding: exact / different / unresolved;
- requested-relation binding: exact / different / unresolved.

The Harness should then materialize the final disposition deterministically:

- exact target + exact relation => eligible `relevant` subject to existing deterministic identity floors;
- affirmative different target or different relation => `irrelevant`;
- any unresolved binding => `ambiguous`.

This keeps the model useful for semantic matching while moving the final destructive retention/rejection policy back into Harness-owned materialization. Factual contradiction remains separate and does not alter target/relation binding.

The v2 observation is immutable and must not be rerun or rescored after this change. A v3 successor requires a new proposal/materialization contract identity, new calibration identity/freeze, and a first/only canonical observation.
