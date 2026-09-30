# Engine 0.6 evidence relevance independent holdout v5

Status: fresh pre-observation holdout. No provider observation has been consumed for this corpus.

Successor-v5 semantics were frozen first:

- freeze commit: 3d5ed5e4ad756c683dd53d06c97a12b6eea00307
- freeze tag: engine-0.6-evidence-relevance-successor-v5-semantics-freeze
- effective qualification: v7
- materialization: v20
- semantic checksum: fixtures/evidence-relevance-holdout-successor-v5/semantics-v5.sha256 (25/25)

## Freshness and distribution

- suite: evidence-relevance-holdout-v5
- cases: 26
- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8
- annotation protocol: evidence-relevance-effective-qualification-v7
- fixed core: evidence-relevance-fixed-core-v5
- status: fresh_unobserved_holdout

The surface was authored only after the successor-v5 semantics freeze. Against holdout v1-v4 and the reusable successor-v5 development corpus it requires:

- case-ID overlap: 0
- canonical-entity overlap: 0
- task overlap: 0
- exact signal overlap: 0
- exact 8-token candidate-signal n-gram overlap: 0

The development failures that shaped successor v5 are explicitly excluded from the fresh surface. No development fixture or captured development observation may count as holdout acceptance evidence.

## Coverage

The 26 cases cover:

- exact canonical and authorized-alias positive ownership
- availability, pricing, limit, change/launch, definition and benefit relations
- exact-target wrong-relation rejection
- repeated sibling ownership
- navigation-only target plus repeated sibling ownership
- explicit local target absence, including untrusted prompt-injection text
- explicit separate-service ownership
- structured distinct-product ownership
- comparison-only target mentions with same and different requested relations
- possible rename / alias mapping uncertainty
- URL-only unnamed ownership
- shared ownership
- truncated relation support
- three separate single-near-sibling ambiguity floors:
  - no target identity
  - navigation-only target identity
  - URL-only target identity

## Pre-observation validation

Before any live provider call:

- frozen v7/v20 expected semantics: 26/26
- runner validate-only: 26 planned / 0 completed / non-scorable
- distribution: 8 / 10 / 8
- independence audit against v1-v4 + successor-v5 development: PASS
- workflow is required-provider Mistral + Groq + Google
- Groq is present only in this final holdout workflow, not in the successor-v5 development workflow
- canonical workflow rejects reruns by requiring GITHUB_RUN_ATTEMPT == 1
- exact frozen surface and successor-v5 semantic checksums are revalidated before provider credentials are used

## Groq admission

The previous canonical Groq arm was holdout v4:

- run: 36650257492
- completion: 2026-09-30T02:38:34Z
- observed tokens: 67,212
- modeled start headroom: 55,000
- frozen pacing: 300 seconds between cases
- minimum request interval: 10 seconds
- conservative post-v4 modeled headroom: approximately 5.75K
- required v5 start headroom: 55K
- fail-closed v5 not-before floor: 2026-09-30T08:33:10Z / 2026-09-30 17:33:10 JST
- v5 Groq self-budget: 70K
- pre-next-case reserve: 4K
- v5 pacing: 300 seconds

Known material intervening Groq organization usage invalidates this model and requires re-anchoring before the canonical tag is pushed.

## Canonical rule

The canonical identity is the first and only push of:

engine-0.6-evidence-relevance-holdout-v5-freeze

Do not push that tag before the Groq admission floor. Do not rerun, rescore, relabel, move, delete/recreate, or reinterpret the canonical identity after observation.

Required final gates for all three providers:

- 26/26 operational completion
- provider failures = 0
- wrong-target Relevant = 0
- effective authority qualification exact = 26/26
- materialized exact = 26/26
- false relevance rejection = 0
- relevant left Ambiguous = 0
- utility miss = 0

Any required-provider failure leaves holdout v5 as immutable FAIL and Issue #462 remains open.
