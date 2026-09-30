# Engine 0.6 evidence relevance independent holdout v7

Status: fresh pre-observation holdout. No provider observation has been consumed for this corpus.

Successor-v6 semantics remain frozen and unchanged:

- semantics freeze commit: `092bdeac5676856e7af311e99c66d9da164cd46f`
- semantics freeze tag: `engine-0.6-evidence-relevance-successor-v6-semantics-freeze`
- effective qualification: v8
- materialization: v21
- semantic checksum: `fixtures/evidence-relevance-holdout-successor-v6/semantics-v6.sha256` (30 files)

Holdout v6 canonical run `36750505629` attempt 1 is immutable operational FAIL. Every required arm stopped before provider execution because the v6 runner omitted the v6 suite from `checkpoint_profile()`. No v6 semantic observation was consumed and v8/v21 were not changed in response.

The operational successor runner was frozen before this v7 corpus was authored:

- runner freeze tag: `engine-0.6-evidence-relevance-holdout-v7-runner-freeze`
- runner freeze commit: `b6356b6db18a114c3fe3ef515a84cb99a7784257`
- checkpoint-profile completeness test covers all declared holdout/development profiles, including v6 and v7

## Freshness and distribution

- suite: `evidence-relevance-holdout-v7`
- cases: 26
- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8
- annotation protocol: `evidence-relevance-effective-qualification-v8`
- fixed core: `evidence-relevance-fixed-core-v7`
- status: `fresh_unobserved_holdout`

The surface was authored only after the runner freeze. Against holdout v1-v6 plus successor-v5 and successor-v6 development corpora it requires:

- case-ID overlap: 0
- canonical-entity overlap: 0
- task overlap: 0
- exact signal overlap: 0
- exact 8-token candidate-signal n-gram overlap: 0

No v6 case outcome exists, and no development or historical holdout observation may be used to shape v7 labels or production semantics.

## Coverage

The 26 cases preserve the established holdout structure while using fresh identities and text:

- exact canonical and Harness-authorized alias positives
- availability, pricing, limit, change/launch, definition, and benefit relations
- exact-target wrong-relation rejection
- repeated sibling ownership and navigation-only target plus sibling ownership
- strict named-target local absence with inert untrusted control text
- exact-target requested-relation absence
- explicit separate-service and structured distinct-product ownership
- comparison-only target mentions with same and different requested relations
- possible rename and alias-mapping uncertainty
- URL-only unnamed ownership
- shared ownership
- visibly truncated relation support
- navigation-only, URL-only, and no-target single-near-sibling ambiguity floors

## Pre-observation contract

Before any provider credential is used:

- frozen v8/v21 expected semantics: 26/26
- runner validate-only: 26 planned / 0 completed / non-scorable
- runner freeze commit must still match exactly
- semantic checksum must remain 30/30
- independence audit against holdout v1-v6 + successor-v5/v6 development must PASS
- exact v7 surface checksum must PASS
- canonical workflow rejects reruns with `GITHUB_RUN_ATTEMPT == 1`
- required providers are Mistral, Google, and Groq

## Groq admission

Holdout v6 made no provider call, so the last token-consuming canonical Groq anchor remains holdout v5 run `36694957246` attempt 1:

- observed tokens: 66,423
- completion: `2026-09-30T11:29:25Z`
- modeled start headroom: 55,000
- conservative post-v5 modeled headroom: approximately 6,539.96
- fail-closed not-before floor: `2026-09-30T17:18:20Z` / `2026-10-01 02:18:20 JST`
- canonical self-budget: 70,000
- pre-next-case reserve: 4,000
- pacing: 300 seconds

Known material intervening organization-level Groq usage invalidates this anchor and requires re-anchoring before the v7 canonical tag is pushed.

## Canonical rule

The first and only push of `engine-0.6-evidence-relevance-holdout-v7-freeze` is the canonical identity. Do not rerun, rescore, relabel, move, delete/recreate, or reinterpret it after observation.

All three required providers must satisfy 26/26 operational completion, provider failures 0, wrong-target Relevant 0, effective authority qualification 26/26 exact, materialization 26/26 exact, false relevance rejection 0, relevant-left-Ambiguous 0, and utility miss 0.
