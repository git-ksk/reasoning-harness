# Engine 0.6 evidence relevance independent holdout v7

状態: fresh pre-observation holdout。このcorpusではprovider observationをまだ消費していない。

successor-v6 semanticsはfreeze済みのまま変更しない:

- semantics freeze commit: `092bdeac5676856e7af311e99c66d9da164cd46f`
- semantics freeze tag: `engine-0.6-evidence-relevance-successor-v6-semantics-freeze`
- effective qualification: v8
- materialization: v21
- semantic checksum: `fixtures/evidence-relevance-holdout-successor-v6/semantics-v6.sha256` (30 files)

holdout v6 canonical run `36750505629` attempt 1 は immutable operational FAIL。3 required armすべて、v6 runnerの `checkpoint_profile()` にv6 suiteが無かったためprovider実行前に停止した。v6 semantic observationは0件で、この結果を理由にv8/v21は変更していない。

v7 corpus authoring前にoperational successor runnerをfreezeした:

- runner freeze tag: `engine-0.6-evidence-relevance-holdout-v7-runner-freeze`
- runner freeze commit: `b6356b6db18a114c3fe3ef515a84cb99a7784257`
- checkpoint-profile completeness testはv6/v7を含む全declared holdout/development profileを網羅

## Freshness / distribution

- suite: `evidence-relevance-holdout-v7`
- cases: 26
- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8
- annotation protocol: `evidence-relevance-effective-qualification-v8`
- fixed core: `evidence-relevance-fixed-core-v7`
- status: `fresh_unobserved_holdout`

runner freeze後にのみauthorした。holdout v1-v6 + successor-v5/v6 development corpusに対して次を必須にする:

- case-ID overlap: 0
- canonical-entity overlap: 0
- task overlap: 0
- exact signal overlap: 0
- exact 8-token candidate-signal n-gram overlap: 0

v6 case outcomeは存在しない。development/historical holdout observationをv7 labelやproduction semanticsのshapingに使わない。

## Pre-observation contract

provider credential使用前に必ず:

- frozen v8/v21 expected semantics: 26/26
- runner validate-only: 26 planned / 0 completed / non-scorable
- runner freeze commit exact match
- semantic checksum 30/30
- holdout v1-v6 + successor-v5/v6 developmentとのindependence audit PASS
- v7 surface checksum PASS
- `GITHUB_RUN_ATTEMPT == 1` でrerun拒否
- required provider: Mistral + Google + Groq

## Groq admission

holdout v6ではprovider callが0件だったため、最後のtoken-consuming canonical Groq anchorはholdout v5 run `36694957246` attempt 1のまま:

- observed tokens: 66,423
- completion: `2026-09-30T11:29:25Z`
- modeled start headroom: 55,000
- conservative post-v5 modeled headroom: 約6,539.96
- fail-closed not-before floor: `2026-09-30T17:18:20Z` / `2026-10-01 02:18:20 JST`
- canonical self-budget: 70,000
- pre-next-case reserve: 4,000
- pacing: 300秒

materialなintervening organization-level Groq usageが判明した場合はこのanchorを無効化し、v7 canonical tag push前にre-anchorする。

## Canonical rule

`engine-0.6-evidence-relevance-holdout-v7-freeze` のfirst/only pushをcanonical identityとする。observation後のrerun / rescore / relabel / tag move / delete-recreate / reinterpretationは禁止。

Mistral / Google / Groqの全required providerで26/26 operational completion、provider failure 0、wrong-target Relevant 0、effective authority qualification 26/26 exact、materialization 26/26 exact、false relevance rejection 0、relevant-left-Ambiguous 0、utility miss 0を必須とする。
