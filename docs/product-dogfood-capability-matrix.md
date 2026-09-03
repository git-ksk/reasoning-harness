# Product dogfood capability matrix

Issue #147 expands product dogfood from the original six-case smoke slice into a broader, pre-observation model-fitness evaluation. This is product evaluation, not a new semantic-authority experiment: the adopted D3 runtime, sufficiency policy, verifier boundary, and frozen research holdouts remain unchanged.

## Three-stage design

### Stage A — 24-case development/product matrix

`fixtures/product-dogfood-v1` remains the original six-case smoke/seed set. `fixtures/product-dogfood-v2` is the development/product capability matrix used for broader model comparison.

The v2 corpus contains exactly 24 cases: eight capability families with three independently worded/domain-varied cases each.

| Capability family | Cases | What it probes |
| --- | ---: | --- |
| `direct_grounding` | 3 | direct reporting of harness-owned structured facts |
| `insufficient_evidence` | 3 | refusal to promote weak/irrelevant observations into the requested conclusion |
| `bounded_resolution` | 3 | acquisition/admission/re-verification of an explicitly configured resolver fact |
| `safe_partial` | 3 | preservation of useful supported non-target facts while the requested target remains unconfirmed |
| `contradiction` | 3 | consistent-control vs conflicting structured records |
| `causal_boundary` | 3 | explicit verified cause vs association/sequence that does not establish causation |
| `temporal_validity` | 3 | valid-window control, stale evidence, and not-yet-valid evidence |
| `scope_entity_boundary` | 3 | exact scope vs region/tenant mismatch |

The outcome mix is deliberately not forced to 50/50: 7 cases are directly grounded, 3 become grounded only after bounded resolution, and 14 are expected to remain unknown. Capability coverage, not a synthetic class balance, determines the corpus size.

Before provider observation, the corpus payload is frozen by `fixtures/product-dogfood-v2.sha256`. Any changed fixture requires a new corpus identity rather than rewriting the observed v2 payload.

All Stage-A models use the same corpus, base seed, max-token limit, and `shared-candidate-initial-render-v1` comparison contract. Results remain workload-specific compatibility/utility evidence, not a universal model leaderboard.

### Stage B — five-run replication

Only operationally complete/useful Stage-A candidates advance. The frozen v2 corpus is rerun across five predeclared base seeds. Comparison is paired by case and seed; aggregate mean target coverage is not sufficient by itself.

At minimum, replication records:

- provider/protocol completion;
- unsupported exposed grounded claims;
- missed target insufficiency;
- target coverage and false target abstention;
- bounded-resolution success;
- safe-partial retention;
- token and latency overhead;
- per-case disagreement across runs.

A provider/protocol failure stays operational evidence and does not enter a semantic denominator as a fabricated abstention.

### Stage C — fresh holdout

Stage-B model/runtime selection is complete for the frozen evaluation candidate `1f27bef9e5e7d1b8d2e95c4e4245c8fe8e77b352`. The fresh holdout is now authored as `fixtures/product-dogfood-holdout-v1`: 16 cases, eight capability families × two cases, with zero exact target-key overlap with `product-dogfood-v2`. Its payload is frozen by `fixtures/product-dogfood-holdout-v1.sha256` before any live provider observation.

The pre-observation Stage-C evaluation plan is frozen as follows:

- base seed: `15000`;
- max tokens: `1024`;
- comparison contract: `shared-candidate-initial-render-v1`;
- current answer-safety configuration: `verified-target-answer-gate-v1`;
- selected model panel: `ministral-8b-latest`, `ministral-14b-latest`, `mistral-small-latest`, `gemma-4-31b-it`, and `gemini-3.1-flash-lite`;
- Gemini 3.5 Flash-Lite is not part of the Stage-C panel because its fifth predeclared Stage-B seed was operationally blocked by the provider free-tier quota after four semantically complete 1.00 runs. This is an operational exclusion, not a semantic failure or a rewritten gate.

Acceptance is fixed before observation: every semantically scored model must preserve unsupported exposed grounded claims = `0`, missed target insufficiency = `0`, contradiction/temporal/scope protections, and mean grounded target coverage >= `0.90`. Provider/protocol failures are operational evidence only and may be retried on the same frozen model/seed without changing fixtures, gates, or model-facing contracts. Any semantic miss is recorded against this version; it does not trigger holdout rewriting or an in-place runtime change.

## Evaluator hardening required by v2

Temporal and scope cases exposed an evaluation-side weakness in the earlier dogfood helper: raw support accounting checked only structured key/value equality. v2 changes that helper to reuse the same trusted structured-fact verifier selection used by the runtime, including evidence qualification requirements. Stale, out-of-scope, and conflicting evidence therefore cannot be counted as supported merely because a matching key/value appears somewhere in the input.

This change affects evaluation accounting only. It does not grant the evaluator or model new authority and does not modify the adopted runtime policy.

## Workflow

The manual `product-dogfood` workflow explicitly selects `product-dogfood-v1`, `product-dogfood-v2`, or `product-dogfood-holdout-v1`. v2 remains the development matrix; holdout-v1 is a separate Stage-C surface. Before loading provider credentials, the workflow parses the selected corpus and enforces its frozen structure/hash contract: v1 = 6 cases, v2 = 24 cases / 8 families ×3 plus its SHA-256 manifest, and holdout-v1 = 16 cases / 8 families ×2 plus its SHA-256 manifest. After a live holdout run, the same workflow enforces the predeclared Stage-C semantic gates: current-safety unsupported grounded claims = 0, missed target insufficiency = 0, and mean grounded target coverage >= 0.90.

The six-case v1 corpus remains available for fast smoke/regression use and for interpreting historical NL-5 runs.

## Stage-A result and utility-recovery interlude

Stage A completed from frozen `product-dogfood-v2` on base seed `12000` / 1024 max tokens. Six models completed all 24 cases with zero unsupported exposed grounded claims and zero missed task-target insufficiency in the Harness arms. Successor target coverage was: Gemini 3.5 Flash-Lite `1.00`, Mistral Small `0.70`, Gemma 4 31B `0.60`, Ministral 8B `0.20`, Ministral 14B `0.20`, and Ministral 3B `0.10`. Gemma 4 26B and Nemotron 3.5 Lightning were protocol-incomplete. Gemini 3.1 Flash-Lite reached case 18 before an operational Google HTTP 500 high-demand failure and therefore has no Stage-A semantic score.

The expanded matrix exposed a product-portability problem that the six-case slice did not localize. In particular, multiple Ministral expected-grounded misses ended with `final_verdict=accept` but no exposable structured final claim, while Gemma 4 31B produced semantically readable renderer claims whose proposition keys drifted from the exact harness-owned keys and were therefore correctly blocked by finalization. These are not reasons to relax exact proposition identity. They motivate harness-owned recovery from already verified artifact state.

Issue #150 is therefore an explicit interlude before Stage B. It separates:

1. behavior-neutral failure provenance;
2. deterministic canonical recovery of exact `Known` / `Supported` task targets;
3. bounded resolver closure driven only by unresolved harness-owned targets, with mandatory admission and re-verification;
4. conservative safe-partial recovery;
5. a separate provider-neutral structured-output resilience lane;
6. before/after development comparison followed by the predeclared fresh Stage-B seeds `13000`, `13100`, `13200`, `13300`, `13400`.

The v2 fixtures and hash manifest remain frozen throughout this work. Stage-B replication is intentionally deferred until the #150 candidate behavior is frozen. The fresh 12–16 case holdout remains deferred until after Stage-B selection.
