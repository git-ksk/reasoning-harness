# Engine 0.6 evidence-target relevance calibration v4 result

Status: frozen v4 observation completed, but acceptance still failed on the Google arm. Mistral passed all semantic and operational gates. Google improved materially from v3 but remained operationally incomplete and retained two safe-ambiguity utility misses. v4 remains immutable historical calibration evidence.

## Frozen identity

- freeze tag: `engine-0.6-evidence-relevance-calibration-v4-freeze`
- candidate commit: `8537220b8d9226890f35dab0f7bf63d1a280ac22`
- GitHub Actions run: `35998574508`
- suite: `evidence-relevance-calibration-v4`
- cases: 26
- seed: `4624604`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`

## Raw v4 metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 23/26 |
| failed provider cases | 0 | 3 |
| binding proposal exact accuracy | 17/26 (65.38%) | 14/23 (60.87%) |
| materialized exact accuracy | **26/26 (100%)** | 21/23 (91.30%) |
| wrong-target / unresolved-binding retained as relevant | **0** | **0 on completed cases** |
| false relevance rejection on expected-relevant | 0 | 0 on completed cases |
| expected-relevant left ambiguous | 0 | 0 on completed cases |
| utility misses | **0** | **2** |
| operational assessment timeouts | 0 | **3** |
| lexical baseline exact accuracy | 12/26 (46.15%) | 12/26 (46.15%) |
| lexical baseline wrong-target relevance retention | 8 | 8 |
| total tokens | 13,571 | 12,375 |
| model-call latency total | 14,203 ms | 250,800 ms |

## Mistral semantic confirmation

The v4 target-first materializer closed the v3 utility failure. `21_unknown_rename` again produced `target=unresolved / relation=different`, but the Harness correctly materialized `ambiguous`. Mistral therefore reached:

- 26/26 operational completion;
- 26/26 exact materialized disposition;
- correctness violations 0;
- utility misses 0.

This confirms that unresolved target identity must dominate relation-level rejection.

## Google operational finding

Increasing the assessment budget from 15s to 30s improved Google operational completion from 9/26 to 23/26, but three cases still hit the exact 30-second Harness deadline before a completed provider attempt:

- `04_semantic_paraphrase`;
- `19_relation_mismatch_same_target`;
- `21_unknown_rename`.

The budget remains correctly fail-closed, but `gemini-3.5-flash-lite` does not provide stable operational completion for this workload under the current bounded envelope.

## Google utility findings

Two completed negative cases remained safe but over-conservative:

- `16_broad_landing_no_support`: model proposed `target=exact / relation=unresolved`; strict identity prevented unsafe relevance, final result `ambiguous` instead of expected `irrelevant`;
- `20_prompt_injection_self_declare`: model proposed `target=unresolved / relation=unresolved`; final result `ambiguous` instead of expected `irrelevant`.

Neither case created a correctness violation. Making the deterministic materializer more aggressive solely to force these cases to `irrelevant` risks false rejection of partial identity, uncertain rename, or truncated-passage cases. The next step should therefore test whether this is model-specific before changing the Harness contract again.

## Next calibration step

Before freezing another full canonical successor, run a bounded calibration diagnostic on the five unresolved Google cases using the same frozen v4 semantic contract and compare:

- `gemini-3.5-flash-lite`;
- `gemini-3.1-flash-lite`, which already has substantial completed semantic-study history in this repository.

This diagnostic is tuning evidence only, not an independent holdout and not a release gate. It may select the Google calibration arm for a future frozen successor, but it must not mutate v4 or reuse a future holdout.
