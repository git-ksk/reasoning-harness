# Engine 0.6 evidence relevance independent holdout v5

Status: fresh pre-observation holdout。まだこのcorpusに対するprovider observationは一切消費していない。

Successor-v5 semanticsを先にfreeze済み:

- freeze commit: 3d5ed5e4ad756c683dd53d06c97a12b6eea00307
- freeze tag: engine-0.6-evidence-relevance-successor-v5-semantics-freeze
- effective qualification: v7
- materialization: v20
- semantic checksum: fixtures/evidence-relevance-holdout-successor-v5/semantics-v5.sha256 (25/25)

## Freshness / distribution

- suite: evidence-relevance-holdout-v5
- 26 case
- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8
- annotation protocol: evidence-relevance-effective-qualification-v7
- fixed core: evidence-relevance-fixed-core-v5
- status: fresh_unobserved_holdout

surfaceはsuccessor-v5 semantics freeze後にのみauthorした。holdout v1-v4とreusable successor-v5 development corpusに対し、以下を必須とする:

- case ID overlap: 0
- canonical entity overlap: 0
- task overlap: 0
- exact signal overlap: 0
- exact 8-token candidate-signal n-gram overlap: 0

successor v5を調整するために観測済みのdevelopment failure surfaceはfresh holdoutから明示的に除外する。development fixture / captured development observationはholdout acceptance evidenceには数えない。

## Coverage

26 caseで以下を含む:

- exact canonical / authorized alias positive ownership
- availability / pricing / limit / change-launch / definition / benefit relation
- exact-target wrong-relation rejection
- repeated sibling ownership
- navigation-only target + repeated sibling ownership
- prompt-injection textを含むexplicit local target absence
- explicit separate-service ownership
- structured distinct-product ownership
- same/different requested relationのcomparison-only target mention
- possible rename / alias mapping uncertainty
- URL-only unnamed ownership
- shared ownership
- truncated relation support
- single near-sibling ambiguity floorを3種類に分離:
  - target identityなし
  - navigation-only target identity
  - URL-only target identity

## Pre-observation validation

live provider call前に:

- frozen v7/v20 expected semantics: 26/26
- runner validate-only: 26 planned / 0 completed / non-scorable
- distribution: 8 / 10 / 8
- v1-v4 + successor-v5 developmentに対するindependence audit: PASS
- required provider workflow: Mistral + Groq + Google
- Groqはこのfinal holdout workflowにのみ存在し、successor-v5 development workflowには存在しない
- canonical workflowは GITHUB_RUN_ATTEMPT == 1 を必須としrerunを拒否
- provider credential使用前にexact frozen surfaceとsuccessor-v5 semantic checksumを再検証

## Groq admission

直前のcanonical Groq armはholdout v4:

- run: 36650257492
- completion: 2026-09-30T02:38:34Z
- observed tokens: 67,212
- modeled start headroom: 55,000
- frozen pacing: case間300秒
- minimum request interval: 10秒
- conservative post-v4 modeled headroom: 約5.75K
- v5 required start headroom: 55K
- fail-closed v5 not-before floor: 2026-09-30T08:33:10Z / 2026-09-30 17:33:10 JST
- v5 Groq self-budget: 70K
- pre-next-case reserve: 4K
- v5 pacing: 300秒

既知のmaterialなintervening Groq organization usageがあればこのモデルは無効なので、canonical tag push前に再anchorする。

## Canonical rule

canonical identityは以下tagのfirst / only push:

engine-0.6-evidence-relevance-holdout-v5-freeze

Groq admission floor前にはpushしない。観測後のrerun / rescore / relabel / tag移動 / delete-recreate / PASSへの再解釈は禁止。

3 providerすべてのrequired final gate:

- operational completion 26/26
- provider failure = 0
- wrong-target Relevant = 0
- effective authority qualification exact = 26/26
- materialized exact = 26/26
- false relevance rejection = 0
- relevant left Ambiguous = 0
- utility miss = 0

required providerが1つでもFAILならholdout v5はimmutable FAILで、Issue #462はopenのまま。
