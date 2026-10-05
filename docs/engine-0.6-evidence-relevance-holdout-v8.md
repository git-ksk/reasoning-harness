# Engine 0.6 evidence relevance holdout v8

Status: fresh corpus authored after the dedicated runner freeze; unobserved and not yet canonical.

## Frozen semantic input

- successor semantics tag: `engine-0.6-evidence-relevance-successor-v7-semantics-freeze`
- successor semantics commit: `3a508bfba20386218436581dcbb69224b54948b3`
- effective qualification: v9
- materialization: v22
- historical v8/v21 predecessor semantics remain immutable

## Runner freeze

The dedicated binary is `reason-evidence-relevance-holdout-v8-study`.

- runner freeze tag: `engine-0.6-evidence-relevance-holdout-v8-runner-freeze`
- runner freeze commit: `924617edf2a6a50ef131ea5c04af729547c0c22e`
- tag object: `b12423ce7d3c23359b1c4609fdf465081f32f085`
- exact-head CI: 9/9 green
- corpus state at runner freeze: absent

The V8 runner was frozen before any holdout-v8 case was authored. Its profile maps `evidence-relevance-holdout-v8` to effective qualification v9 and materialization v22, includes the V8 checkpoint mapping, and is covered by the declared-profile completeness test.

## Fresh corpus

The corpus contains 26 cases:

- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8

It preserves generic coverage of exact-target positive evidence, authorized aliases, relation mismatch, explicit local absence, distinct sibling ownership, comparison scope, mapping uncertainty, shared ownership, truncation/context gaps, and navigation/URL/no-target single-near-sibling ambiguity.

The corpus was authored only after the V8 runner tag was pushed. Against holdout v1-v7 plus successor-v5/v6 development and both successor-v7 development surfaces, the following are required and machine-enforced:

- case-ID overlap: 0
- canonical-entity overlap: 0
- task overlap: 0
- exact signal overlap: 0
- exact 8-token candidate-signal n-gram overlap: 0

Offline expected labels are exact under frozen v9/v22 semantics for all 26 cases.

## Groq admission

The admission anchor is recalculated from the actual Groq arm of immutable canonical holdout-v7 run `36800959088`, attempt 1, rather than reusing the older v5 anchor:

- model: `openai/gpt-oss-120b`
- completed cases: 26/26
- observed tokens: 66,922
- canonical Groq completion: `2026-10-01T03:37:52Z`
- modeled start headroom: 55,000 tokens
- minimum holdout pacing: 7,760 seconds
- modeled refill during v7: approximately 17,962.96 tokens
- conservative modeled post-v7 headroom: approximately 6,040.96 tokens
- required v8 start headroom: 55,000 tokens
- fail-closed not-before floor: `2026-10-01T09:30:23Z` / `2026-10-01 18:30:23 JST`
- self-budget: 70,000 tokens

The floor has already elapsed. Any known material intervening organization-level Groq usage still invalidates this modeled anchor and requires re-anchoring before the v8 canonical tag is pushed.

## Canonical policy

The first and only push of `engine-0.6-evidence-relevance-holdout-v8-freeze` is the canonical identity. Do not rerun, rescore, relabel, move, delete/recreate, or reinterpret the tag after observation.

Required providers:

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`

Each required arm must complete 26/26 with provider failures 0. The final gate requires effective authority qualification 26/26 exact, materialization 26/26 exact, wrong-target Relevant 0, false relevance rejections 0, relevant-left-Ambiguous 0, and utility misses 0.

Before corpus freeze, revalidate the surface checksum, semantics checksum, runner freeze coordinate, independence checks, validate-only output, v9/v22 expected labels, affected tests, Clippy, rustfmt, workflow YAML, and exact-head CI.
