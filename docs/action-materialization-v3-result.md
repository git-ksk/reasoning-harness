# Action materialization v3 — adoption result

Issue #283's third fresh adoption holdout was frozen and executed as `action-materialization-v3`. This successor intentionally evaluates the Issue #283 action-selection/materialization boundary separately from downstream #248 finalization.

Both provider coordinates passed the frozen adoption gate. The candidate preserved the measured correctness boundary, retained same-key sibling identities, eliminated legacy executable-ID planner calls on the eligible path, and exercised Harness-owned intent materialization in every complete case.

## Frozen coordinate

- freeze tag: `action-materialization-v3-freeze`
- freeze commit: `34eada58093d8e4086849a6d8c3ac08882be8448`
- control product commit: `94ff1b79ac0e9c183d2fc30ceaf41f23f3927d3e`
- candidate product commit: `7a91d272af1bab0a97bf80ed7bba027ff253d50a`
- corpus: `action-materialization-v3`
- evaluator: `reason-action-materialization-v3`
- scoring: `action-materialization-scoring-v3`
- canonical Actions run: `35431076755`
- Mistral job: `105865637487`
- Google job: `105865637557`
- primary seeds: `104211`–`104215`
- observed all-k group: `k=5`

The surface, scoring policy, provider/model coordinates, case identities, seeds, coordinate order, and adoption predicate were frozen before the first live provider call. No semantic rerun was performed.

## Adoption result

| provider/model | control complete | candidate complete | control action-path success | candidate action-path success | correctness violations, control | correctness violations, candidate |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` | 5/5 | 5/5 | 5/5 | 5/5 | 0 | 0 |
| Google `gemini-3.5-flash-lite` | 5/5 | 5/5 | 4/5 | 5/5 | 0 | 0 |

Both provider jobs passed the complete frozen acceptance check.

### Architecture-path evidence

Across both providers:

- all ten complete candidate cases exposed distinct same-key sibling target identities;
- all ten complete control cases exercised the legacy executable-action planner path;
- all ten complete candidate cases conformed to `reason-investigation-intent-v1` plus `target-intent-materialization-v1`;
- candidate legacy executable-action planner calls were zero;
- candidate intent rejection and action rejection were zero;
- relevant read-only capability selection remained present;
- no correctness-boundary violation was observed;
- no IID-derived `p^k` value was reported.

Call-path totals were:

| provider/model | control legacy executable-action planner calls | candidate legacy executable-action planner calls | candidate intent calls | candidate Harness materializations |
| --- | ---: | ---: | ---: | ---: |
| Mistral `ministral-8b-latest` | 20 | 0 | 30 | 30 |
| Google `gemini-3.5-flash-lite` | 20 | 0 | 20 | 15 |

The Google control coordinate had one action-path failure in seed `104214`, stale Solvane case. The stochastic control planner proposed `solvane-window-secondary` again for `target-primary` after that target/capability pair had already been attempted. Harness correctly rejected it as `duplicate_action`. The case retained zero correctness-boundary violations. The candidate coordinate had no action or intent rejection and completed all five action-path trials.

This is fresh observed evidence that, on the predeclared mechanically safe path, the candidate removes stochastic executable-ID selection from the model-facing decision while preserving the tested fail-closed and correctness boundaries.

## Downstream finalization remains separate

Downstream finalization was recorded but was not part of the v3 adoption gate.

Mistral continued to expose the known finalization interaction:

- control: `requires_verification` 9 cases, `unresolved` 1 case;
- candidate: `requires_verification` 10 cases;
- grounded-case false abstentions: 5 on control and 5 on candidate.

Google produced the same downstream outcome distribution on control and candidate:

- `grounded_answer`: 5 cases;
- `requires_verification`: 3 cases;
- `unresolved`: 2 cases;
- grounded-case false abstentions: 0.

The v3 result does not change #248 finalization semantics. It only prevents those downstream outcomes from being conflated with the Issue #283 action-materialization adoption question.

## Cost and latency observations

These are descriptive measurements over five paired trials per provider and are not generalized performance claims.

Mistral candidate versus control:

- provider calls: 60 → 70 (+16.67%)
- provider attempts: 74 → 88 (+18.92%)
- tokens: 58,139 → 64,735 (+11.35%)
- provider latency: 110,423 ms → 137,787 ms (+24.78%)
- wall time: 111,852 ms → 139,234 ms (+24.48%)

Google candidate versus control:

- provider calls: 55 → 55 (0.00%)
- provider attempts: 59 → 63 (+6.78%)
- tokens: 39,070 → 34,629 (-11.37%)
- provider latency: 164,801 ms → 260,004 ms (+57.77%)
- wall time: 165,533 ms → 260,604 ms (+57.43%)

Issue #283 therefore should not be interpreted as a total-call, token, or latency optimization. The adopted architectural benefit is narrower: executable capability identity is Harness-owned on the mechanically safe path rather than selected stochastically by the planner.

## Deterministic and freeze validation

Before the v3 freeze:

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `cargo test --workspace`: PASS
- v3 evaluator unit tests: 5/5 PASS
- fixture preflight: PASS
- surface checksum verification: PASS
- YAML parse: PASS
- `git diff --check`: PASS
- product inputs relative to candidate commit `7a91d272...`: no diff

Both canonical jobs revalidated the frozen surface and exact product coordinates before credentials and live observation.

## Disposition

The fresh v3 evidence satisfies the Issue #283 adoption requirement for the evaluated architecture:

- mechanically safe action materialization is Harness-owned;
- the candidate removes legacy stochastic executable-ID selection from the eligible path;
- same-key sibling identity is preserved;
- retry/attempted-pair rejection remains fail closed;
- correctness-boundary violations remain zero in the canonical observations;
- downstream finalization authority remains separate and unchanged;
- the control/candidate product commits remained fixed across v1, v2, and v3.

The product candidate can proceed to PR review/merge with the v1 failed measurement, v2 mixed result, and v3 adoption result all preserved rather than rewritten.

## Preserved machine reports

- [Mistral raw machine report](observations/action-materialization-v3-mistral-run-35431076755-2026-09-19.json), SHA-256 `01eddaf5d14a1e456b803a4e2a5e5a06fba5f7df9cc4fc6ced444b246d735996`
- [Google raw machine report](observations/action-materialization-v3-google-run-35431076755-2026-09-19.json), SHA-256 `0d0062eea2980a8a4bf231158c336eb4105c16748f736cd51f1eb83c4b6809e8`
