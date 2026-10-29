# Ordinary CLI evaluation plan — 2026-10-26

[日本語](engine-073-cli-evaluation-plan.ja.md) | English

The user approved separating ordinary CLI evaluation from model-free auxiliary API contract checks. This successor plan does not alter any frozen v1/v2 inputs, labels, reports, receipts, tags or their historical verdicts. Adoption remains pending. This document alone admits no live callback or release.

## First live run and next repair, 2026-10-29

[PR568](https://github.com/git-ksk/reasoning-harness/pull/568) recorded actual-runtime, independent operational and genuine-receipt admission. The48-job ordinary Mistral [run40873570944](https://github.com/git-ksk/reasoning-harness/actions/runs/40873570944) stopped with2 completed,1 failed and45 suppressed jobs. Full F1 remains unavailable. The rw-02-candidate final_render request was13,365bytes against the unchanged12,000-byte pre-send bound. All8 preceding HTTP responses were200 with known usage; the ninth was never sent. Preserve the historical usage_budget_unmeasurable stop record and track its misleading diagnostic separately in #540.

Prioritize the common CLI fix in [#569](https://github.com/git-ksk/reasoning-harness/issues/569). Compact only structural JSON whitespace in the verified artifact/schema for both initial and fallback requests, retaining every JSON value, user text, verdict, system, format, seed and bound. An offline replay of the saved request yields11,186bytes with identical information. After reproduction, independent review, both Engine builds and CI, freeze a distinct successor source/runtime/profile. Do not rerun the old v1. Fresh independent accuracy acceptance and CLI adoption remain incomplete.

## Scope

- **Ordinary CLI:** real-model questions, investigation, acquired evidence and final display. This is the primary CLI adoption evaluation.
- **Auxiliary APIs:** no model calls or credentials. Check specification-defined results, invalid input refusal, authority, tampering/stale reviews, deterministic replay, original artifact preservation and public API compatibility.
- **Earlier model-assisted helper experiment:** retain as a combined adviser/controlled-host/API reference result. Its0.400000 case F1 is not an auxiliary API unit score or an independent ordinary CLI adoption gate.

The API records and applies authorized caller reviews; it does not discover general semantic equivalence. Untrusted model advice alone has no review authority.

## Current evidence

[The v2 first score](cli-paired-real24-v2-20261026.ja.md) remains immutable. Gemini ordinary F1 is0.933333 on23/24 tasks in each arm (precision1.000, recall0.875); full coverage and final-candidate effects still require confirmation. Mistral has15/24 baseline and13/24 candidate complete observations, so subset F1 values0.428571/0.461538 are not a comparable improvement. PR560 merged the capture repair; its85 tests exercise measurement code, not the auxiliary API itself. Groq stays held and unavailable.

## Current work

- Policy, Japanese/English roadmaps and Issues531/538/540 are synchronized in PR561.
- [Independent model-free API contract review](../evaluation/cli-normal-v1/model-free-api-review/review.json): PASS for pinned candidate;960 existing core tests passed,2 historical first-observation controls remain ignored,44 direct API groups and9 legacy compatibility cases passed. The host still owns authentication, semantic determination and freshness.
- [Independent Mistral diagnosis](../evaluation/cli-normal-v1/mistral-independent-diagnosis/diagnosis-summary-v1.md):48 saved ordinary executions;14 over-limit and2 duplicate-ID plans;11 missing-claims and3 nested-typo model responses.
- 2026-10-28: [PR561](https://github.com/git-ksk/reasoning-harness/pull/561) merged Issues562/563/564 at `82b48a4`: effective request limits, model candidate shapes and fallback token budgets. Bounded independent reviews and all22 final-head CI checks passed. The common evaluation host is fixed at `c38d87d`; public candidate contracts and explicit `claims:[]` remain compatible.
- Duplicate acquisitions, early Reject, selector/typed-value alignment and trusted explanations remain distinct diagnosed issues.
- [PR565](https://github.com/git-ksk/reasoning-harness/pull/565) merged at `a7de5a4` on 2026-10-29. Its Mistral-only 48-job ordinary profile now binds actual run/artifact/receipt/raw observations and scoring, and verifies downloaded images in checkout code before any container runs. Final `4bab9cd` has independent source-only PASS (3 host-boundary,20 normal,85 inherited controls). Preserve the initial `7f0508e` FAIL and repair history. Actual AMD64 image/proof validation is running in [offline run40787270979](https://github.com/git-ksk/reasoning-harness/actions/runs/40787270979); formal operational acceptance, new model generations and F1 remain incomplete.

## Pre-execution repair, 2026-10-28

The successor common CLI is pinned to `dbad2dc` from [PR567](https://github.com/git-ksk/reasoning-harness/pull/567), with bounded independent source review, both Linux Engine builds and actual repeated-acquisition loopback controls passed. All22 exact-head CI checks passed before merge `41fbd17`, closing #566. macOS cancellations were unassigned hosted-runner capacity failures; only affected jobs were rerun. Final runtime/image acceptance and real-model evaluation remain separate incomplete gates.


Prioritize [Issue566](https://github.com/git-ksk/reasoning-harness/issues/566). Independent operational review reproduced identical snapshot reacquisition being marked malformed by the existing core in14/48 saved Mistral and2/48 Google ordinary payloads. The prospective runner correctly stops. Preserve scoring and failure classification; investigate a successor common CLI that recognizes only an exact currently admitted snapshot under the current admission policy as no progress. An earlier raw resolver emission alone does not prove admission. Preserve real calls/costs and reject changed content, unknown metadata or denied authority.

Order: successor CLI repair and boundary controls, independent review/CI, rebuild the same host with both Engine arms, independently accept a new runtime/image/proof, then execute48 Mistral jobs and measure F1. The source-only PASS for preparation3086778 and local Docker controls do not approve a new live run. Historical raw/F1, published cores and freezes remain immutable.

## Order

1. Synchronize this plan, both roadmaps and Issues531/538/540 first.
2. Independently check the model-free APIs of pinned candidate `ab3c6322921c8c738ec35bd018345917b5afa538`. Reuse existing meaningful contract tests; add controls only for uncovered boundaries or reproduced defects.
3. Diagnose Mistral's48 saved ordinary raw outputs without rescoring or changing them. Separate capture gaps from actual missing answers. Repair only reproduced defects. Any common CLI repair applies identically to both Engine arms.
4. Freeze a separate ordinary-only profile:24 tasks×2 arms=48 jobs per provider, zero helper model calls. Do not reuse the historical72-job profile or receipt as live authorization.
5. Freeze the actual source, binaries, image, inputs, scorer, independent review and run conditions before new Mistral measurement. Confirm Gemini on all24 tasks for the final candidate. These consumed tasks remain development diagnostics.
6. After evaluator/candidate freeze, another author creates24 new independent real-document tasks. Review complete sources, citation bindings and gold before the first run. Do not reuse historical annotations/scores.
7. Select the Engine version, then complete a dedicated CLI integration PR, all supported OS checks, installation/distribution/upgrade/rollback acceptance.

## Future ordinary CLI gates

For each required provider, complete the whole cohort; both case and claim F1≥0.90, precision≥0.95, recall≥0.85. Keep hard safety violations at0, citation precision/coverage/literal fidelity at1.0, no known safety defect and no regression in previously useful controls. Report paired useful answers, regressions, denominators and missingness. Equal quality is not an improvement claim. Separate Engine effects from common CLI repairs and single model-run variation.

The future usability decision uses absolute ordinary quality and non-regression; useful-answer gain remains separately reported. The old per-track gain gates retain their old scope, rather than requiring unused API display gains for CLI adoption. Historical verdicts are unchanged. Efficiency bounds remain logical-call mean delta≤1, total-token ratio≤1.25 and p95-wall ratio≤1.5 on complete pairs. No undefined or incomplete metric passes.

Mistral/Google are prioritized; Groq remains held and no all-provider acceptance is claimed without its required evidence. Keep fixed models/tiers, single attempts, pacing, durable reservations, unknown-usage stops/cancellation and USD2.5/7.5 self-caps. No implicit retry, budget increase or model swap. Model-free API PASS, CLI quality, source release, CLI adoption and distribution are distinct gates. Issues488/489/491 require reproduced needs, not blanket implementation.
