# Engine 0.6 evidence relevance holdout v8

Status: dedicated runner freeze後にfresh corpusをauthor済み。未観測で、canonical実行前。

## Frozen semantic input

- successor semantics tag: `engine-0.6-evidence-relevance-successor-v7-semantics-freeze`
- successor semantics commit: `3a508bfba20386218436581dcbb69224b54948b3`
- effective qualification: v9
- materialization: v22
- historical v8/v21 predecessor semanticsはimmutableのまま

## Runner freeze

dedicated binaryは `reason-evidence-relevance-holdout-v8-study`。

- runner freeze tag: `engine-0.6-evidence-relevance-holdout-v8-runner-freeze`
- runner freeze commit: `924617edf2a6a50ef131ea5c04af729547c0c22e`
- tag object: `b12423ce7d3c23359b1c4609fdf465081f32f085`
- exact-head CI: 9/9 green
- runner freeze時点のcorpus: 未作成

つまりholdout-v8 caseを1件もauthorする前にrunnerをfreezeした。V8 profileは `evidence-relevance-holdout-v8` をeffective qualification v9 / materialization v22へwiringし、V8 checkpoint mappingとdeclared-profile completeness testを持つ。

## Fresh corpus

26 case:

- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8

exact-target positive evidence、authorized alias、relation mismatch、explicit local absence、distinct sibling ownership、comparison scope、mapping uncertainty、shared ownership、truncation/context gap、navigation / URL / no-target single-near-sibling ambiguityをgenericにカバーする。

V8 runner tag push後にのみauthorした。holdout v1-v7 + successor-v5/v6 development + successor-v7 development v1/v2に対して、以下をmachine-enforcedする:

- case-ID overlap: 0
- canonical-entity overlap: 0
- task overlap: 0
- exact signal overlap: 0
- exact 8-token candidate-signal n-gram overlap: 0

26 caseすべて、frozen v9/v22 semanticsでoffline expected labelがexactに一致する。

## Groq admission

古いv5 anchorは再利用せず、immutable canonical holdout-v7 run `36800959088` attempt 1 の実Groq armから再計算した:

- model: `openai/gpt-oss-120b`
- completed cases: 26/26
- observed tokens: 66,922
- canonical Groq completion: `2026-10-01T03:37:52Z`
- modeled start headroom: 55,000 tokens
- minimum holdout pacing: 7,760秒
- v7中のmodeled refill: 約17,962.96 tokens
- conservative modeled post-v7 headroom: 約6,040.96 tokens
- v8 required start headroom: 55,000 tokens
- fail-closed not-before floor: `2026-10-01T09:30:23Z` / `2026-10-01 18:30:23 JST`
- self-budget: 70,000 tokens

floorは既に経過済み。ただしmaterialなintervening organization-level Groq usageが判明した場合はanchorを無効化し、v8 canonical tag push前にre-anchorする。

## Canonical policy

`engine-0.6-evidence-relevance-holdout-v8-freeze` のfirst/only pushをcanonical identityとする。observation後のrerun / rescore / relabel / tag move / delete-recreate / reinterpretationは禁止。

required provider:

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Groq `openai/gpt-oss-120b`

全required armで26/26 operational completion、provider failure 0が必要。final gateはeffective authority qualification 26/26 exact、materialization 26/26 exact、wrong-target Relevant 0、false relevance rejection 0、relevant-left-Ambiguous 0、utility miss 0を要求する。

corpus freeze前にsurface checksum、semantics checksum、runner freeze coordinate、independence、validate-only、v9/v22 expected labels、affected tests、Clippy、rustfmt、workflow YAML、exact-head CIを再検証する。
