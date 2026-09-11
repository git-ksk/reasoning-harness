# Natural-language E2E v11 — cross-model replication result

Issue #256 replicated the immutable released v0.4.1 natural-language E2E v11 surface across Google and Groq targets without changing the frozen evaluator, scoring contract, corpus, seed, or released product source.

## Frozen coordinate

- replication Actions run: `34135141249`
- replication freeze tag: `natural-language-e2e-v11-cross-model-v1-freeze`
- replication freeze commit: `6a6db2fd436816d113303e52e0a28f7e0c6baf97`
- reference v11 tag / commit: `natural-language-e2e-v11-freeze` / `a758af17a998493c1005702365b100e05b05f95d`
- released product: `v0.4.1` / `29a9e4be6273dbffeda324e15517dc64930ad315`
- seed / max tokens: `57000` / `1024`
- fixed cases: `13`
- canonical Mistral reference: Actions `34129798774`, `ministral-8b-latest`

The replication freeze and historical v11 reference remain immutable. No failed or incomplete target is rerun, repaired, rescored, or tuned in place.

## Result summary

| target | completed | hard correctness | operational completeness | target recall | tool selection | trigger | #249 conformance | useful follow-up | grounded target coverage |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` reference | 13/13 | PASS | PASS | 0.60 | 0.80 | 1/3 | 1/1 | 1/1 | 0.00 |
| Google `gemma-4-31b-it` | 12/13 | PASS | FAIL | 0.80 | 0.90 | 3/3 | 3/3 | 3/3 | 0.20 |
| Google `gemini-3.5-flash-lite` | 13/13 | PASS | PASS | 1.00 | 0.60 | 0/3 | inconclusive | n/a | 0.20 |
| Groq `openai/gpt-oss-120b` | 0/13 | no semantic violation observed | FAIL | n/a | n/a | n/a | n/a | n/a | n/a |
| Groq `qwen/qwen3.8-27b` | 0/13 | no semantic violation observed | FAIL | n/a | n/a | n/a | n/a | n/a | n/a |
| Groq `openai/gpt-oss-20b` | 0/13 | no semantic violation observed | FAIL | n/a | n/a | n/a | n/a | n/a | n/a |

The table deliberately keeps semantic/correctness evidence separate from operational completeness. In particular, the Gemma row is semantically useful but operationally incomplete; the Groq rows never entered model generation through the generic CLI and therefore are not model-quality evidence.

## Google observations

### Gemma 4 31B

Gemma exposed all three predeclared no-result predecessor triggers. Each case selected the configured cache first, observed typed `no_result`, and then used the existing #249 exact-target continuation to the corresponding registry. Conditional mechanism conformance was therefore `3/3`, and all three continuations produced useful downstream evidence.

The run nevertheless failed operational completeness: `12/13` cases completed, with four operational failures plus one process-level operational failure. Observed generation failure classes included `provider_unavailable` and `protocol`; the process failure class was `provider_unavailable`. Session persistence remained valid and external side-effect replay stayed zero, but aggregate session-fork validity was false because the relevant session path did not complete. These failures are retained as operational evidence, not converted into semantic failures.

### Gemini 3.5 Flash-Lite

Gemini completed all `13/13` cases with hard correctness, measurement, operations, and report gates passing. Target recall was `1.00`, but tool-selection success was `0.60`.

All three predeclared follow-up cases recalled the exact target yet executed zero actions, made four planner calls, and stopped at `round_budget`. Trigger reachability was therefore `0/3`, so the #249 mechanism denominator was zero and its conformance result is correctly `inconclusive`, not a mechanism failure.

This is direct evidence that the remaining pre-trigger planner/action-selection gap is model-sensitive and can occur even when target recall itself succeeds.

## Groq observation

All three Groq targets failed `0/13` before provider generation because the generic `reason` CLI rejected `--provider groq`:

`invalid value 'groq' for '--provider <PROVIDER>'; possible values: mistral, google, nvidia`

The existing `GroqAdapter` and dedicated external-information evaluation path are therefore not disproven. The observed defect is generic natural-language CLI/provider wiring, tracked for v0.4.2 by #262. Groq quota, rate-limit, or model semantics were not reached by this frozen replication.

## Cross-model interpretation

Across every case where the released v0.4.1 #249 trigger was actually exposed, the exact-target typed-`no_result` continuation was conformant: Mistral `1/1` plus Gemma `3/3`, for `4/4` observed conformant continuations. No correctness-boundary violation, unsupported exposed assertion, unsupported structured claim, identity-unsafe admission, MCP authority self-promotion, or session external side-effect replay was observed in the completed semantic rows.

The product utility gap is therefore not evidence that #249 is malfunctioning. The primary pre-trigger residual is planner/action selection: Mistral exposed one of three follow-up triggers, Gemini exposed none, and Gemma exposed all three. The downstream grounding residual also remains: useful follow-up evidence did not reliably become a grounded final answer. That finalization/grounding boundary remains assigned to #248 / Harness Engine 0.5.0 and is not moved into v0.4.2.

## Closeout decision

Issue #256 is complete as immutable cross-model replication evidence. It establishes the v0.4.1 baseline that v0.4.2 must beat rather than a result to tune against in place.

The next patch therefore has two product targets only:

1. #261: add a narrow Harness-owned deterministic read-only acquisition precedence rule before stochastic planner stalls, preserving all authority and identity boundaries;
2. #262: expose the already-existing Groq adapter through the generic natural-language `reason` provider path.

Fresh successor measurement is owned by #263. v0.4.2 must not release unless that new frozen measurement shows strict utility improvement over this baseline with no correctness, authority, identity, admission, session, or final-answer safety regression.
