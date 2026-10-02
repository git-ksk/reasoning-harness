# Engine 0.6 evidence relevance independent holdout v6

状態: fresh pre-observation holdout。このcorpusではprovider observationをまだ一度も消費していない。

successor-v6 semanticsは、このsurfaceをauthorする前にfreeze済み:

- freeze commit: `092bdeac5676856e7af311e99c66d9da164cd46f`
- freeze tag: `engine-0.6-evidence-relevance-successor-v6-semantics-freeze`
- annotated tag object: `a3a12afc32340878a625a422d1c760e36a2280f9`
- effective qualification: v8
- materialization: v21
- semantic checksum: `fixtures/evidence-relevance-holdout-successor-v6/semantics-v6.sha256`（30 files）

holdout-v6準備ではfreeze済み30-file semantic surfaceを変更しない。historical holdout runner自体もsemantic checksum対象なので、v6 routingは別の新規binaryへ分離する。

## Freshness と distribution

- suite: `evidence-relevance-holdout-v6`
- cases: 26
- Relevant: 8
- Irrelevant: 10
- Ambiguous: 8
- annotation protocol: `evidence-relevance-effective-qualification-v8`
- fixed core: `evidence-relevance-fixed-core-v6`
- status: `fresh_unobserved_holdout`

surfaceはsuccessor-v6 semantics freeze後にのみauthorした。holdout v1-v5とsuccessor-v5/v6 development corpusに対して以下を必須にする:

- case-ID overlap: 0
- canonical-entity overlap: 0
- task overlap: 0
- exact signal overlap: 0
- exact 8-token candidate-signal n-gram overlap: 0

development fixtureやcaptured development observationはholdout acceptance evidenceとして扱わない。

## Coverage

26 casesでは以下を確認する:

- exact canonical / Harness-authorized alias のpositive ownership
- availability / pricing / limit / change-launch / definition / benefit relation
- exact-target + wrong-relation rejection
- repeated sibling ownershipとnavigation-only target + repeated sibling ownership
- strict named-target local absenceと、inertなuntrusted control text
- exact targetは存在するがrequested relationが明示的に不在な境界
- explicit separate-service / structured distinct-product ownership
- same/different requested relationを伴うcomparison-only target mention
- rename / alias mapping uncertainty
- URL-only unnamed ownership
- shared ownership
- visible truncationによるrelation uncertainty
- navigation-only / URL-only / no-target のsingle-near-sibling ambiguity floor

## Pre-observation validation contract

live provider call前に必ず:

- frozen v8/v21 expected semantics: 26/26
- runner validate-only: 26 planned / 0 completed / non-scorable
- distribution: 8 / 10 / 8
- holdout v1-v5 + successor-v5/v6 developmentに対するindependence audit: PASS
- freeze済みsuccessor-v6 30-file semantic checksum: exact PASS
- workflowのrequired provider: Mistral + Google + Groq
- `GITHUB_RUN_ATTEMPT == 1` でrerun拒否
- credentialを使う前にholdout surface checksumを再検証

既存roadmapとsuccessor-v6 design docsはfreeze済みsemantic surfaceに含まれるため、holdout-v6準備では意図的に編集しない。post-freezeのholdout状態はこの文書で記録する。

## Groq admission

直前のcanonical Groq armはholdout v5 run `36694957246` attempt 1:

- completion: `2026-09-30T11:29:25Z`
- observed tokens: 66,423
- modeled start headroom: 55,000
- frozen pacing: 300秒/case
- minimum request interval: 10秒
- conservative post-v5 modeled headroom: 約6,539.96
- required v6 start headroom: 55,000
- fail-closed v6 not-before floor: `2026-09-30T17:18:20Z` / `2026-10-01 02:18:20 JST`
- v6 Groq self-budget: 70,000
- pre-next-case reserve: 4,000
- v6 pacing: 300秒

このanchor以降にmaterialなorganization-level Groq usageが判明した場合、このモデルは無効としてcanonical tag push前にre-anchorする。

## Freeze / canonical rule

holdout surfaceはlocal annotated tag

`engine-0.6-evidence-relevance-holdout-v6-freeze`

が指すcommitでfreezeする。

Groq admission floorとintervening-usage checkを満たすまで、このtagをGitHubへpushしてはならない。tag pushがone-shot canonical triggerになる。

canonical tag push後はrerun / rescore / relabel / tag move / delete-recreate / PASS reinterpretationを禁止する。

Mistral / Google / Groqの全required armで必要なfinal gate:

- 26/26 operational completion
- provider failures = 0
- wrong-target Relevant = 0
- effective authority qualification exact = 26/26
- materialized exact = 26/26
- false relevance rejection = 0
- relevant left Ambiguous = 0
- utility miss = 0

required providerのいずれかが失敗した場合、holdout v6はimmutable FAILとして保持し、Issue #462はopenのままにする。
