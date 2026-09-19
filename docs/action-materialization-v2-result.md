# Action materialization v2 — mixed adoption result

Issue #283's second fresh adoption holdout was frozen and executed as `action-materialization-v2`. The evaluation surface itself was operationally valid: both provider coordinates completed all five paired trials with zero correctness-boundary violations and exercised the intended control/candidate architecture split. The predeclared adoption gate nevertheless produced a mixed result because its planner-success predicate still included downstream finalization.

## Frozen coordinate

- freeze tag: `action-materialization-v2-freeze`
- freeze commit: `5a99912ffd6cd90afc71664cabadbe5f8727e2e2`
- control product commit: `94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e`
- candidate product commit: `7a91d272af1bab0a97bf80ed7bba027ff253d50a`
- corpus: `action-materialization-v2`
- evaluator: `reason-action-materialization-v2`
- scoring: `action-materialization-scoring-v2`
- canonical Actions run: `35429022032`
- Mistral job: `105860070988`
- Google job: `105860071151`
- primary seeds: `98221`–`98225`
- observed all-k group: `k=5`

The surface, evaluator, scoring identity, provider/model coordinates, seeds, success predicate, coordinate ordering, and k group were frozen before the first provider call. No semantic rerun was performed.

## Architecture-path observation

The architecture-specific #283 signal was consistent across both providers:

| provider/model | control complete | candidate complete | correctness violations, control | correctness violations, candidate | control legacy executable-action planner calls | candidate legacy executable-action planner calls | candidate intent calls | candidate Harness materializations |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` | 5/5 | 5/5 | 0 | 0 | 20 | 0 | 30 | 30 |
| Google `gemini-3.5-flash-lite` | 5/5 | 5/5 | 0 | 0 | 16 | 0 | 20 | 15 |

For both providers:

- all ten same-key-sibling cases exposed two distinct target identities on control and candidate;
- control exercised the legacy executable-action planner path in all ten cases;
- candidate exercised `reason-investigation-intent-v1` plus `target-intent-materialization-v1` in all ten cases;
- candidate legacy executable-action planner calls were zero;
- candidate intent rejection and action rejection were zero;
- relevant read-only capabilities were selected;
- no correctness-boundary violation was observed.

This is direct evidence that the candidate moves mechanically safe executable capability materialization into Harness control flow on the evaluated path without weakening the measured safety boundary.

## Frozen gate result

The frozen v2 gate produced different provider outcomes:

| provider/model | control planner success | candidate planner success | frozen acceptance |
| --- | ---: | ---: | --- |
| Mistral `ministral-8b-latest` | 0/5 | 0/5 | fail |
| Google `gemini-3.5-flash-lite` | 5/5 | 5/5 | pass |

This difference was not caused by an action-materialization-path regression.

### Mistral

Every grounded Velmora case on both control and candidate ended in `requires_verification` with two blocked unverified propositions. The same-key sibling targets resolved to the same proposition, and the existing finalization bridge intentionally fails closed when more than one target candidate maps to that proposition. The grounded case therefore counted as a false abstention under the v2 planner-success predicate.

All ten Mistral stale cases also ended in `requires_verification`, but they still satisfied the unknown-case planner predicate because the target was not incorrectly grounded.

### Google

Every grounded Velmora case on both control and candidate ended in `grounded_answer` and satisfied the frozen planner predicate. The stale Tarsenne cases were either `unresolved` or `requires_verification` while remaining correctly ungrounded.

The control and candidate coordinates matched each other within each provider. The provider difference therefore reflects downstream target/finalization behavior exercised by model-generated target structure, not a difference introduced by the #283 candidate product commit.

## Why v2 is not the final adoption decision

The v2 planner-success predicate mixes two separable questions:

1. Did the planner/action path recall the target, preserve same-key siblings, select a relevant capability, and exercise the intended legacy-vs-intent/materialization architecture?
2. Did the downstream #248 finalization bridge produce a grounded final answer for the generated target structure?

Issue #283 is specifically about ownership of mechanically safe action materialization. Issue #248 intentionally remains the separate target-to-finalization authority bridge. Making downstream final-answer grounding a required component of the #283 adoption gate caused provider-dependent finalization behavior to mask an otherwise consistent architecture-path observation.

The frozen v2 result is preserved exactly as measured. Its scoring is not changed and the failed Mistral coordinate is not reclassified.

## Cost/latency observations

These are descriptive observations over five paired trials per provider, not general performance claims.

Mistral candidate versus control:

- provider calls: 60 → 70 (+16.67%)
- provider attempts: 77 → 84 (+9.09%)
- tokens: 61,563 → 58,033 (-5.73%)
- provider latency: 163,270 ms → 130,417 ms (-20.12%)
- wall time: 164,697 ms → 131,843 ms (-19.95%)

Google candidate versus control:

- provider calls: 51 → 55 (+7.84%)
- provider attempts: 57 → 61 (+7.02%)
- tokens: 34,959 → 33,839 (-3.20%)
- provider latency: 200,703 ms → 198,311 ms (-1.19%)
- wall time: 201,619 ms → 198,914 ms (-1.34%)

The #283 architecture does not imply a reduction in total model calls. Its directly measured benefit is removal of stochastic executable-ID selection from the eligible path; total-call, token, and latency effects remain workload/provider dependent.

## Disposition

`action-materialization-v2` is preserved as a valid mixed observation and will not be rescored or rerun.

A fresh `action-materialization-v3` successor should:

- keep the same control and candidate product commits;
- use fresh case identities, fact keys, fixture content, and seeds;
- keep correctness and operational completeness as independent hard axes;
- define the #283 adoption gate only over target recall, same-key sibling exposure, relevant capability selection, action/intent rejection, control legacy-path exposure, candidate legacy-path elimination, and candidate intent/materialization conformance;
- record downstream finalization state, grounded answer state, false abstention, and blocked propositions as separate observational metrics rather than adoption-gate inputs;
- preserve #248 finalization semantics unchanged;
- freeze before live observation and perform one first canonical run per provider only.

## Preserved machine reports

- [Mistral raw machine report](observations/action-materialization-v2-mistral-run-35429022032-2026-09-19.json), SHA-256 `40f83ee1b75801eddc410c8ad8fd7c64c2f612bf7939deaa5bb4a6a7b687b982`
- [Google raw machine report](observations/action-materialization-v2-google-run-35429022032-2026-09-19.json), SHA-256 `f147f5c5ab53e8eb5cc035abf5ff3d17ae96eeb9d36dd0d990968f66b48c6b6d`
