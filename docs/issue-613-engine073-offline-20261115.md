# Issue #613: Engine 0.7.3 real-document development regression — offline diagnosis

Date: 2026-11-15. **Verdict: partial fix; Issue OPEN/HOLD; Engine 0.7.3 release, CLI adoption and independent F1 still HOLD.** This document adds no new model evaluation and does not modify the fixed 44 outputs, cancelled Actions run, private gold, or encrypted originals.

## Provenance and limits

The original, one-shot, cancelled [workflow 43667947455](https://github.com/git-ksk/reasoning-harness/actions/runs/43667947455) yielded 44/48 complete observations across 22 paired questions. The previous [development-only review](../evaluation/engine073-531-real-doc24-20261115/review-44-observed-20261115.md) remains authoritative for the measured baseline F1 0.7407 and candidate F1 0.6400, and for 45/45 displayed citation integrity. No new F1 is asserted here. The original raw responses and developer labels remain on the owner's Mac, not in GitHub. Only non-generative offline replay was performed against the previously decrypted originals. Model variability means these observations do not establish a systematic causal effect of the 0.7.3 Core diff.

The observed paths were inspected as **source binding → plan/Fact Key → admitted evidence → verification receipts/contradictions → finalization → actually exposed factual claims**. The final displayed claim, not source quotation text or an internal candidate, is the scored unit.

## Case findings from the actual candidate outputs

| Case | Verified evidence and plan | Finalization / proximate failure | Offline disposition |
|---|---|---|---|
| `rw-02` curl | The retry-transient target had **no admitted binding**. The HTTP404 key had a supported bound canonical value **and** a contradicted model-generated different-value proposition under the same key. | Artifact verdict `reject`, investigation `no_progress`, final answer `abstain` (0 displayed supported claims). The unbound first target and same-key conflict prevent safe recovery. | Preserve abstention; no proof of a safe Core-only fix. |
| `rw-19` SQLite | Two plan targets share `sqlite.build.extension_before_tests`; the canonical admitted fact is *Helpful but not required before tests.* A model-generated `not required` variant is contradicted under the same key. | Artifact verdict `reject`, `round_budget`, final answer `abstain`. There is a supported claim but the same-key contradictory candidate blocks authorization; blindly equating paraphrases would bypass current safety policy. | Preserve abstention; requires a separately designed, verified contradiction/normalization policy. |
| `rw-21` Redis | Exact admitted source facts and current structured verification receipts existed for both `redis.allocator.linux_default=jemalloc` and `redis.allocator.force_libc=make MALLOC=libc`. The model-proposed second target **omitted `expected_fact_key`**, so the investigation ledger did not bind it, despite its verified receipt. | Artifact verdict `unknown`, investigation `round_budget`. Canonical coverage recovery took ledger-verified targets and selected renderer claims; it therefore retained only `jemalloc`. Original display had **one asserted fact**, not two. | Deterministic recovery gap reproduced and locally fixed; previously verified `MALLOC=libc` can now appear as an asserted fact with a pinned citation. |
| `rw-08` Git | One exact source-backed proposition includes exit codes **0, 1 and 128**. | Candidate answer asserts the entire proposition; citation checks passed. | Preserve the complete proposition; do not split it into independent model-derived claims or loosen checks. |

`no_progress` and `round_budget` are bounded stop conditions, **not evidence of truth or falsehood**. For `rw-02`/`rw-19`, the same-key conflict and missing/failing authority are blocking independent of increasing budgets. For `rw-21`, the failure to name the exact fact in a model plan created an artificial *answer-coverage* gap even though the *verified-evidence* path already contained it. Do not conflate investigation-ledger completeness with actual user-visible answer completeness.

## Minimal source-bound recovery change

- `crates/reasoning-harness-cli/src/answer_context.rs`: `verified_source_targets` now identifies frozen host-supplied source propositions **only after** source configuration validation, exact admitted evidence SHA, source ID, URL/version/quote-pointer checks, provenance/scope/time checks, current structured claim-and-receipt verification, and same-key conflict/qualification checks. Unmatched or forged source metadata yields no recovery authority. Different values for one key in the frozen source config also fail closed.
- `crates/reasoning-harness-cli/src/main.rs`: include only these separately verified source propositions in the existing `finalization_targets` before canonical recovery and before the unchanged answer-safety, source-citation and investigation-coverage gates. This does not modify model plan text, verifier semantics, global verdict, stop budgets, or Core `0.7.3` evidence-relevance policy.
- `.github/workflows/cli-platform-smoke.yml`: run #613 provider-free final-answer tests on the existing Linux/macOS ARM and Intel/Windows matrix, without a provider or the local-only encrypted artifacts.
- `crates/reasoning-harness-cli/src/answer_context.rs` tests: provider-free synthetic two-part answer with a missing plan selector, verification of actual fact text **before** citations, investigation/answer coverage separation, malformed evidence/receipt/version/conflict negative tests, and a Git `check-ignore` 0/1/128 control. Two explicitly ignored opt-in tests replay the original private candidate artifacts when supplied at runtime; CI contains no private data or API calls.

## Offline results

- `cargo fmt --all -- --check`: PASS.
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS.
- `cargo test --workspace --quiet`: **1,593 passed / 0 failed / 11 ignored**, with three new provider-free #613 tests passing as part of the suite.
- Explicit original-raw-artifact replay, only when paths are provided locally:
  - `rw-21`: PASS: old selected `jemalloc` alone, revised canonical recovery asserts **both** `jemalloc` and `make MALLOC=libc`; source-context composition yields **3** exact-bound citations. No new model text is invented.
  - `rw-02` / `rw-19`: PASS: current source-target authorization remains absent in the presence of actual contradictions; neither becomes an asserted answer.
  - `rw-08`: PASS: the original asserted proposition continues to contain all **0, 1, 128** statuses.
- Original 44/48 observation counts, original 45/45 citation audit, evaluation tag, and two unfinished questions are untouched. The above checks replay the **same development observations**, never a blind holdout or formal new F1.

### How to run private replay (local-only)

The `#[ignore]` tests require caller-created temporary files from the owner's existing encrypted-review directory. For `issue613_real_rw21_offline_artifact_replay`, set `RH613_PRIVATE_OBSERVATION` to the extracted `rw-21-candidate.stdout.json`, and `RH613_PUBLIC_CONFIG` to the public `configs/rw-21.json`. For `issue613_real_conflicts_and_check_ignore_offline_replay`, set `RH613_PRIVATE_OBSERVATION_DIR` to a private directory of extracted `rw-02`, `rw-19`, `rw-08` candidate JSON files, and `RH613_PUBLIC_CONFIG_DIR` to the checked-in public config directory. Use `cargo test -p reasoning-harness-cli --bin reason <test-name> -- --ignored`. Never check decrypted observations into source control, log raw private outputs publicly, or run the cancelled tag.

## Remaining acceptance blockers

1. `rw-02` retry-transient unbound fact and same-key conflict need safe deterministic prevention/correction, not simply larger rounds or automatic acceptance of a model candidate.
2. `rw-19` same-key *paraphrase versus exact canonical value* must be evaluated without relaxing contradictory/unknown negative-case withholding. Offline evidence is insufficient to claim that changing 0.7.3 itself caused this model output.
3. A **complete CLI pipeline** regression and independent, evaluator-frozen blind real-source cohort are still required to demonstrate quality improvement over 0.6.1. These offline replays exercise the finalized artifact/recovery/context surfaces but do not constitute new model runs or statistically valid quality results.
4. The existing Linux, macOS ARM/Intel and Windows platform matrix CI (including the added #613 provider-free test step) must pass for this particular PR before considering even the partial patch fully integrated. The primary repository `ci.yml` is Ubuntu; do not assume its green status proves macOS/Windows.
5. Job timeout isolation for a **future** distinct evaluation requires independent review before new provider spend. **Do not rerun** `eval-531-real-doc24-live-20261115-one-shot` or `rw-23`/`rw-24` by default.

**Decision:** Review and possibly merge this partial, defensive recovery change into the development evaluation stack only after CI, but keep #613 OPEN/HOLD. Do **not** release 0.7.3, merge the candidate into main, adopt it in CLI, or claim a new F1 until #531's independent acceptance gates are met.
