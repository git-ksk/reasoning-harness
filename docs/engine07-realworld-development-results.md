# Engine 0.7 practical development comparison — first results

[日本語](engine07-realworld-development-results.ja.md) | English

**FAIL / incomplete measurement; CLI adoption NO-GO.** Updated 2026-10-15 JST. [Issue #531](https://github.com/git-ksk/reasoning-harness/issues/531), [evaluation PR #534](https://github.com/git-ksk/reasoning-harness/pull/534). These are two preserved development observations, not fresh independent acceptance or a representative production error rate.

Both actual immutable source tags were built inside the same published CLI0.5.5 host, with unchanged provider/dependency code: Engine0.6.1 `4abee90f0501f0d8fc09b453a0b45c942fbed69e` and Engine0.7.2 `296690a86e049b75ee693ae5032a71504366137c`. Linux preflight proved identities and deterministic controls before credentials. Source tags and original failed observations are unchanged.

## Scope and denominators

Eight captured official documents from four publisher groups support 24 reused scenarios (17 distinct normal questions). Each provider has 24 normal invocations per Engine, including 18 answerable scenarios, plus a separate 24-case helper comparison. Three instruction-injection derivatives are explicitly synthetic. Context, scope and extracted structured facts were **agent/host-curated** and checked against privately captured full documents. The frozen v2 README's “human-curated” wording does not establish human review. Neither run tests raw automatic extraction, open-Web discovery, actual same-time conflicting publications, real correction lineage or production use. Six helper missing/wrong-target prebindings are diagnostic-only, not validated target detection or safety successes.

Models were precommitted: Mistral `ministral-8b-2512`, Google `gemini-3.5-flash-lite`, Groq `openai/gpt-oss-120b`. Alternating arm order, tasks, sources, settings and budgets are held equal within each version. v2 and v3 are separate conditions and must not be pooled.

## Original v2: frozen FAIL

Freeze `107c05dc3f2ffe20ea2116db84a69e57e85a1389`, [run38656022911, attempt1](https://github.com/git-ksk/reasoning-harness/actions/runs/38656022911). [Archive, exact summary and verification](../evaluation/engine07-realworld-v2-results/README.md).

| Provider | Useful answers, 0.6.1 → 0.7.2 (/18) | Normal operational failures, baseline / candidate (/24) | Helper display gains | Helper operational failures (/24) |
| --- | --- | --- | --- | --- |
| Mistral | 0 → 0 | 22 / 22 | 0 | 15 |
| Google | 14 → 14 | 3 / 3 | 0 | 12 |
| Groq | 8 → 7 | 15 / 16 | 2 | 2 |

These are **frozen gate-qualified useful-answer counts, not bare Engine accuracy**. v2 incorrectly compared trace adapter-generation counts against a six-logical-operation ceiling; 26 Mistral rows were disqualified by that unit error. It also requested helper advice as Text while exact JSON parsing rejected fenced JSON (15 Mistral and12 Google helper failures). Do not repair or rescore the consumed v2 result. Its terminal failures also included genuine CLI budget exhaustion and missing provider usage: Mistral12 `usage_budget_exceeded` +2 `usage_budget_unmeasurable`; Google6 `usage_budget_exceeded`; Groq10 budget-exceeded +8 unmeasurable +13 daily quota failures. Missing traces on early CLI failure are tracked in [#535](https://github.com/git-ksk/reasoning-harness/issues/535).

Groq's flagged rw05/rw12 positive regressions coincide with candidate operational failure/missing traces. They do not isolate a causal Engine regression. Observed hard-violation counts are zero, but operational incompleteness and known #520 prevent safety acceptance.

## Seen development successor v3: frozen FAIL

Freeze `7432710f61d747595cb8498aa2dea9e44d2ff7fb`, [run38659305389, attempt1](https://github.com/git-ksk/reasoning-harness/actions/runs/38659305389). [Archive, exact summary and verification](../evaluation/engine07-realworld-v3-results/README.md).

v3 fixes the helper request to `JsonSchema`, and separates logical operations (12), adapter generations (24) and observed HTTP attempts (72). Its higher logical-operation ceiling is an explicit **seen-data cost calibration**, not satisfaction of the old six-operation draft criterion. Per-generation output1024, cumulative output4096/total28096, normal wall180s and provider models remain unchanged. v2 results stand.

| Provider | Useful answers, 0.6.1 → 0.7.2 (/18) | Normal operational failures, baseline / candidate (/24) | Helper display gains | Helper operational failures (/24) |
| --- | --- | --- | --- | --- |
| Mistral | 5 → 5 | 6 / 6 | 2 | 0 |
| Google | 14 → 14 | 8 / 8 | 2 | 5 |
| Groq | 0 → 0 | 24 / 24 | 0 | 13 |

No normal-track useful-answer improvement was observed. Mistral rw03 has a baseline useful answer and candidate withholding; this is a paired regression flag requiring fixed-response replay/causal analysis, not a proven isolated Engine defect. Mistral had three missing-usage terminal failures in total and source-acquisition/adapter-output failures in other rows. Google had16 normal wall timeouts (8 per Engine) and5 helper RateLimit failures. Its missing usage cannot be counted as zero. Groq had47 normal typed quota failures, one normal usage-unmeasurable failure and13 helper Quota failures. Helper gain2 for Mistral/Google is bounded opt-in display evidence; it is not a normal-answer gain, external truth authentication or all-provider acceptance.

Every provider's `complete_measurement` and `measurement_gate_pass` is false. Frozen hard-violation counts remain zero on observed/scored rows; unknown/unexecuted paths remain incomplete. Normal structured citation/source-attribution surfaces were absent, so citation-binding quality is unproven. Aggregate full-lane calls/tokens remain null where traces/usage are missing; `observed_*` values are only partial sums. No currency cost is invented.

## Groq operational correction

TPD200000 was reached in v2. v3 was started without importing the historical run-level quota latch and known-exhaustion start constraint; minute pacing did not repair daily headroom. This was an evaluator operations defect. Preserve this failed observation and make no further Groq calls in the known exhausted window. Do not silently switch models or tiers or add outer retries.

[Next-run controls and12 offline tests](../evaluation/engine07-realworld-operations-v1/README.md) implement a shared normal/candidate/helper arm guard: first typed quota, unknown token/attempt accounting, an interrupted host callback, insufficient conservative reserve or two consecutive capacity failures suppress following requests. Suppressed rows remain operationally incomplete. The module is **not retrofitted into frozen v2/v3 and is not yet wired into a new live runner**. The persisted Groq state blocks new starts; recovery and shared-organization headroom are not demonstrated. Historical holdout-v5 headroom/self-budget values must be recalculated for this larger practical workload.

[#535 source repair PR #536](https://github.com/git-ksk/reasoning-harness/pull/536) preserves failed-run usage and early-return/cooperative-cancellation traces; it cannot recover a SIGKILLed process. The next runner must wire the guard, pin the diagnostic repair, validate graceful timeout handling and its arm budget, and pass offline/independent checks before a new freeze. No new live identity has been started.

## Decision and next work

Published CLI0.5.5 / Engine0.6.1 stays the product pair. Known [#520](https://github.com/git-ksk/reasoning-harness/issues/520) source-relevance residual independently blocks0.7.2 adoption. Correct it in a newly pinned successor, preserve0.7.2 and both first results, then prepare new full-source conflict/correction/lineage tasks and a separately authored disjoint acceptance set after complete runner freeze. Independent **design review** was completed before v2/v3 observations; independent fresh acceptance is still0/24. The initial operational follow-up was blocked by a service usage limit; a later read-only [independent operational review](../evaluation/engine07-realworld-operations-v1/independent-review.md) completed with zero provider calls. [PR #537](https://github.com/git-ksk/reasoning-harness/pull/537) repairs quota representation, cancellation/contradictory frames and accounting-completeness boundaries; next-run controls now have26 offline tests. Real-runner integration, durable state/recovery admission and fresh independent acceptance remain incomplete.

Adoption checklist is1/6: only paired development results, limits and NO-GO decision are recorded. Full provider/fresh evaluation, repairs/reacceptance, Engine selection, CLI integration and distribution remain open. #488/#489/#491 remain conditional; no new semantic feature is justified by these incomplete observations alone. #465 and #506 remain separate.
