# Engine 0.6 evidence relevance independent holdout v6

Status: fresh pre-observation holdout. No provider observation has been consumed for this corpus.

Successor-v6 semantics were frozen before this surface was authored:

- freeze commit: `092bdeac5676856e7af311e99c66d9da164cd46f`
- freeze tag: `engine-0.6-evidence-relevance-successor-v6-semantics-freeze`
- annotated tag object: `a3a12afc32340878a625a422d1c760e36a2280f9`
- effective qualification: v8
- materialization: v21
- semantic checksum: `fixtures/evidence-relevance-holdout-successor-v6/semantics-v6.sha256` (30 files)

The frozen 30-file semantic surface is not modified by holdout-v6 preparation. The v6 runner is a separate new binary because the historical holdout runner itself is part of the frozen semantic checksum.

## Freshness and distribution

- suite: `evidence-relevance-holdout-v6`
- cases: 26
- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8
- annotation protocol: `evidence-relevance-effective-qualification-v8`
- fixed core: `evidence-relevance-fixed-core-v6`
- status: `fresh_unobserved_holdout`

The surface was authored only after the successor-v6 semantics freeze. Against holdout v1-v5 plus successor-v5 and successor-v6 development corpora it requires:

- case-ID overlap: 0
- canonical-entity overlap: 0
- task overlap: 0
- exact signal overlap: 0
- exact 8-token candidate-signal n-gram overlap: 0

No development fixture or captured development observation counts as holdout acceptance evidence.

## Coverage

The 26 cases cover:

- exact canonical and Harness-authorized alias positives
- availability, pricing, limit, change/launch, definition, and benefit relations
- exact-target wrong-relation rejection
- repeated sibling ownership and navigation-only target plus repeated sibling ownership
- strict named-target local absence with inert untrusted control text
- exact-target requested-relation absence
- explicit separate-service and structured distinct-product ownership
- comparison-only target mentions with same and different requested relations
- possible rename and alias-mapping uncertainty
- URL-only unnamed ownership
- shared ownership
- visibly truncated relation support
- navigation-only, URL-only, and no-target single-near-sibling ambiguity floors

## Pre-observation validation contract

Before any live provider call:

- frozen v8/v21 expected semantics: 26/26
- runner validate-only: 26 planned / 0 completed / non-scorable
- distribution: 8 / 10 / 8
- independence audit against holdout v1-v5 + successor-v5/v6 development: PASS
- the frozen successor-v6 30-file semantic checksum must still verify exactly
- workflow requires Mistral + Google + Groq
- canonical workflow rejects reruns by requiring `GITHUB_RUN_ATTEMPT == 1`
- exact holdout surface checksum is revalidated before provider credentials are used

Existing roadmap and successor-v6 design documents are part of the frozen semantic surface and are intentionally not edited by holdout-v6 preparation. This document records the post-freeze holdout state instead.

## Groq admission

The previous canonical Groq arm was holdout v5, run `36694957246` attempt 1:

- completion: `2026-09-30T11:29:25Z`
- observed tokens: 66,423
- modeled start headroom: 55,000
- frozen pacing: 300 seconds between cases
- minimum request interval: 10 seconds
- conservative post-v5 modeled headroom: approximately 6,539.96
- required v6 start headroom: 55,000
- fail-closed v6 not-before floor: `2026-09-30T17:18:20Z` / `2026-10-01 02:18:20 JST`
- v6 Groq self-budget: 70,000
- pre-next-case reserve: 4,000
- v6 pacing: 300 seconds

Known material intervening organization-level Groq usage invalidates this model and requires re-anchoring before the canonical tag is pushed.

## Freeze and canonical rule

The holdout surface is frozen at the commit identified by the local annotated tag:

`engine-0.6-evidence-relevance-holdout-v6-freeze`

The tag must not be pushed to GitHub until the Groq admission floor and intervening-usage check are satisfied. Tag push is the one-shot canonical trigger.

After the canonical tag is pushed, do not rerun, rescore, relabel, move, delete/recreate, or reinterpret its identity.

Required final gates for Mistral, Google, and Groq:

- 26/26 operational completion
- provider failures = 0
- wrong-target Relevant = 0
- effective authority qualification exact = 26/26
- materialized exact = 26/26
- false relevance rejection = 0
- relevant left Ambiguous = 0
- utility miss = 0

Any required-provider failure leaves holdout v6 as immutable FAIL and Issue #462 remains open.
