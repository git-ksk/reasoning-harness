# Ordinary CLI evaluation plan — 2026-10-26

[日本語](engine-073-cli-evaluation-plan.ja.md) | English

The user approved separating ordinary CLI evaluation from model-free auxiliary API contract checks. This successor plan does not alter any frozen v1/v2 inputs, labels, reports, receipts, tags or their historical verdicts. Adoption remains pending. This document alone admits no live callback or release.

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
- [PR565](https://github.com/git-ksk/reasoning-harness/pull/565) prepares a Mistral-only48-job ordinary profile. Twelve new and85 inherited offline controls and inert actual-CLI loopback controls passed. Independent source review of preparation `7f0508e` found missing provenance that could mix old observations into a new run; bind actual run/artifact/receipt/raw before re-review. No new model generations or F1 score have been produced.

## Pre-execution repair, 2026-10-28

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
