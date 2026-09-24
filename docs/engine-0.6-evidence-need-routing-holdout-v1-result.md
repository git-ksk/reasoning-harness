# Engine 0.6 evidence-need routing independent holdout v1 result

Status: PASS. The first and only frozen independent holdout v1 observation completed successfully. Both provider arms passed operational, correctness, and utility gates. #461 is independently accepted.

## Frozen identity

- semantic freeze commit: `38d5e584e6f089da187b9ad0fb56f9160a8a2da6`
- holdout freeze commit: `1200a4faac7170b05cc5cfbf3f7c8e3cdba63ce5`
- freeze tag: `engine-0.6-evidence-need-holdout-v1-freeze`
- GitHub Actions run: `35965160995`
- suite: `evidence-need-routing-holdout-v1`
- cases: 26
- seed: `4611601`
- Mistral: `ministral-8b-latest`
- Google: `gemini-3.5-flash-lite`
- credentials: GitHub repository secrets only

The holdout was authored only after the semantic freeze commit. Before any provider credential was read, Actions revalidated the frozen checksum, exact suite/status/case count, deterministic expected-label materialization, and absence of exact task/target/context reuse from the 22-case calibration corpus.

Workflow reruns are prohibited for this identity.

## Independent holdout metrics

| metric | Mistral | Google |
| --- | ---: | ---: |
| successful provider cases | 26/26 | 26/26 |
| provider failures | 0 | 0 |
| proposal exact accuracy | 25/26 (96.15%) | 25/26 (96.15%) |
| materialized-mode exact accuracy | **26/26 (100%)** | **26/26 (100%)** |
| acquisition exact accuracy | **26/26 (100%)** | **26/26 (100%)** |
| correctness-boundary violations | **0** | **0** |
| utility misses | **0** | **0** |
| provider attempts | 26 | 26 |
| total tokens | 12,772 | 13,263 |
| model-call latency total | 20,145 ms | 22,996 ms |

The combined final gate reported `operationally_complete = true`, `correctness_gate_passed = true`, and `utility_gate_passed = true`.

## Diagnostic mismatch

Both providers proposed `context_only` for `h24_ambiguous_account_implication`, while the exact proposal expectation was `external_required`.

The target asks whether a general phased-migration article establishes the state of a specific account. The Harness-owned target kind is `ambiguous`, whose frozen safety floor is `external_required`. Both model proposals were therefore overridden deterministically and both arms materialized `external_required` mode and acquisition.

This is not a correctness or utility failure. It independently confirms the intended authority split: model proposals are advisory; Harness-owned target semantics control the safety floor.

## Independent acceptance

The holdout independently exercised distinct wording/domains across non-factual transformation, content-local tasks, partial/truncated context, external/current claims, explicit/trusted verification, evidence reuse and invalidation, mixed targets, follow-ups, prompt injection, ambiguity, resolver unavailability, and prevention of model-created trusted authority.

Across the frozen independent set:

- unsafe skipped acquisition = 0;
- context authority laundering = 0;
- explicit verification downgrade = 0;
- current-state downgrade = 0;
- trusted-verification downgrade = 0;
- model-created trusted authority = 0;
- invalid existing-evidence reuse = 0;
- mixed-target unsafe whole-turn routing = 0;
- avoidable stronger acquisition = 0;
- provider failures = 0.

Replay-side-effect safety remains covered by the deterministic/replay regression suite and was not changed by the holdout surface.

## Decision

Issue #461 passed fresh calibration, semantic freeze, separately authored post-freeze holdout, pre-observation holdout freeze, first/only model-backed holdout observation, and all operational/correctness/utility gates.

The #461 target-local evidence-need routing candidate is accepted for the Engine 0.6 line.

This does **not** by itself promote `engine-v0.6.0`. Issues #462 and #463 remain separate semantic/correctness tracks and must complete their own evidence-gated acceptance before an Engine 0.6 release coordinate is considered.
