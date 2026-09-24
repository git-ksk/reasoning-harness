# Engine 0.6 evidence-target relevance calibration v3 result

Status: frozen v3 observation completed, but acceptance failed. Mistral was operationally complete with zero correctness violations and one utility miss. Google was semantically exact on all completed cases but operationally incomplete because the 15-second Harness-owned assessment deadline expired before 17 provider calls completed. This result is immutable historical calibration evidence.

## Frozen identity

- freeze tag: `engine-0.6-evidence-relevance-calibration-v3-freeze`
- candidate commit: `53402a19eb979b53c7ddaa80afceffcdfbee8fbe`
- GitHub Actions run: `35996093336`
- suite: `evidence-relevance-calibration-v3`
- cases: 26
- seed: `4623603`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

Preflight passed the exact frozen checksum, validate-only contract, formatting, clippy, deterministic v3 materialization, and runner tests. Both live jobs completed and preserved their first canonical observations. The combined gate failed because Mistral retained one utility mismatch and Google was operationally incomplete.

## Raw v3 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 9/26 |
| failed provider cases | 0 | 17 |
| binding proposal exact accuracy | 17/26 (65.38%) | 7/9 (77.78%) |
| materialized exact accuracy | 25/26 (96.15%) | 9/9 (100%) |
| wrong-target / unresolved-binding retained as relevant | **0** | **0 on completed cases** |
| false relevance rejection on expected-relevant | 0 | 0 on completed cases |
| expected-relevant left ambiguous | 0 | 0 on completed cases |
| utility misses | **1** | 0 on completed cases |
| operational assessment timeouts | 0 | **17** |
| lexical baseline exact accuracy | 12/26 (46.15%) | 12/26 (46.15%) |
| lexical baseline wrong-target relevance retention | 8 | 8 |
| model calls | 26 | 26 |
| provider attempts | 26 | 9 |
| total tokens | 13,571 | 4,896 |
| model-call latency total | 18,439 ms | 307,129 ms |

Google is not semantically scored as a complete canonical arm because 17 cases failed before a provider response was returned.

## Semantic finding: binding precedence

Mistral case `21_unknown_rename` returned:

- target binding: `unresolved`;
- relation binding: `different`.

The frozen v3 materializer treated any `different` binding as sufficient for final `irrelevant`, so this became a utility miss. That ordering is too destructive.

If target identity itself is unresolved, relation binding cannot safely establish that the candidate is irrelevant to the requested target. The final policy should be hierarchical:

1. target `different` => irrelevant;
2. target `unresolved` => ambiguous;
3. target `exact` + relation `different` => irrelevant;
4. target `exact` + relation `unresolved` => ambiguous;
5. target `exact` + relation `exact` => relevant, subject to deterministic identity floors.

This preserves affirmative wrong-target rejection while preventing a secondary relation judgment from destroying material whose target identity remains unresolved.

## Operational finding: 15-second deadline

Google completed only 9/26 cases. All 17 failures were typed `assessment_timeout` at approximately 15,000 ms, with `provider_attempts = 0`. The Harness deadline expired while awaiting the provider future before a completed provider attempt could be recorded.

The nine completed Google cases materialized 9/9 exactly. This therefore does not establish a Google semantic regression.

The 15-second assessment budget was bounded but too narrow for this provider/model observation. A successor may widen the explicit elapsed budget while keeping:

- a finite Harness-owned deadline;
- the existing two-model-call maximum;
- provider failures separated from semantic failures;
- no silent retry of the frozen v3 observation.

## v4 requirements

A successor must use a new frozen identity and must not rewrite or rerun v3. It should change only:

1. final binding materialization precedence so unresolved target identity dominates relation-level rejection;
2. the explicit assessment elapsed budget to a still-bounded value supported by the observed provider latency envelope.

The proposal contract may remain the v2 binding contract. Because materialization semantics change, the Harness materialization policy identity must advance.

No independent holdout may be authored until the successor calibration passes correctness, utility, and operational completeness on both canonical provider arms.
