# Engine 0.6 evidence relevance independent holdout v3

Status: pre-freeze / unobserved。

successor v3 semanticsは `engine-0.6-evidence-relevance-successor-v3-semantics-freeze` / `8c1f02c181e2a7c4e267d10ce610e0517162004d` で先にfreeze済み。holdout v3はその後にのみauthorし、frozen successorを変更せずeffective qualification v5 + materialization v18を評価する。

## Independence boundary

holdout v3はcase ID、entity名、candidate text、labelを新規にした26-case surface。immutable predecessorを書き換えない。

- holdout v1: canonical run `36495389012`、immutable FAIL
- holdout v2: canonical run `36533340582`、immutable FAIL

freeze前にv1/v2両方とのcase-ID overlap 0、canonical-entity overlap 0、candidate-signal exact 8-token overlap 0を要求する。v1/v2 failed caseのentity名・case IDも含めない。

v3 surfaceにはfresh repeated-sibling negativeに加え、single-signal near-sibling、URL-only identity、mapping uncertainty、shared ownership、truncated contextなどのabstention controlも含む。labelはprovider observationではなくannotation contractから独立に固定する。

## Frozen candidate contract

- suite: `evidence-relevance-holdout-v3`
- cases: 26
- distribution: Relevant 8 / Irrelevant 10 / Ambiguous 8
- required provider:
  - Mistral `ministral-8b-latest`
  - Groq `openai/gpt-oss-120b`
  - Google `gemini-3.5-flash-lite`
- annotation protocol: `evidence-relevance-effective-qualification-v5`
- materialization: v18
- fixed core: `evidence-relevance-fixed-core-v3`
- seed: `4629303`
- intended freeze tag: `engine-0.6-evidence-relevance-holdout-v3a-freeze`
- canonical policy: tag triggerのfirst run / attempt 1 only。rerun / replacement canonicalは禁止

runnerは後方互換を維持し、v1=v3/v16、v2=v4/v17、v3=v5/v18を明示的に選択する。

## Acceptance gates

3 required provider armすべて26/26 operational完走が必要。各providerが独立に以下を満たすこと。

- wrong-target Relevant retention = 0
- false relevance rejection = 0
- Relevant left Ambiguous = 0
- utility miss = 0
- materialized exact = 26/26
- effective authority qualification exact = 26/26
- effective identity/relation authority miss = 0
- scope-risk miss / spurious-risk = 0
- provider-arm latch / operational abort / incomplete provider attemptなし

operational failureはoperationalのまま扱い、semantic PASS/FAILへ読み替えない。

## Groq TPD admission

v3 workflowはimmutable holdout-v2 actualからconservative TPD modelを再anchorする。

- v2 observed tokens: 65,555
- v2 completion anchor: `2026-09-29T09:05:28Z`
- v2 configured start headroom: 55,000
- v2 frozen pacing: case間300,000 ms + minimum request interval 10,000 ms
- modeled post-v2 headroom: 7,407.96 tokens
- v3 required start headroom: 55,000
- v3 self-budget: 70,000 tokens
- pre-case reserve: 4,000 tokens
- v3 pacing: case間300,000 ms + minimum request interval 10,000 ms
- earliest modeled floor: `2026-09-29T14:48:08Z` / `2026-09-29 23:48:08 JST`

tiny readinessはtransport / credential / TPM / RPD evidenceのみでTPD headroom proofには使わない。v2 anchor以降にmaterialなorganization-level Groq利用が既知または疑われる場合、freeze tag push前にmodelを再anchorする。

## Operational preflight incident

最初のtag engine-0.6-evidence-relevance-holdout-v3-freeze はrun 36585464494 attempt 1でpreflight operational FAIL。原因はworkflow内に残った suite_id == evidence-relevance-holdout-v2 assertionで、Groq floor / independence / checksumはPASS後、このassertで停止した。live provider jobはskipされ、Mistral / Groq / Google API observationは0件。semantic surfaceは未観測のまま。元tag/runはimmutableに保持し、rerun / tag移動はしない。修正後の新しいoperational identityは engine-0.6-evidence-relevance-holdout-v3a-freeze。semantic manifest / label / v5-v18 semanticsは変更しない。

## Freeze discipline

holdout-v3 freeze tag push前に:

1. v1/v2/v3 study profileをvalidate
2. immutable recorded observationに対するdeterministic replay/regressionのみ実行し、canonical live v1/v2はrerunしない
3. CLI/core focused test、workspace Clippy、fmt、diff、workflow YAML、checksum、independence、public-safety scanを実施
4. PR #466をDraft、Issue #462をopenのままexact-head PR CI greenを確認
5. Groq floor到達とTPD anchor有効性を再確認
6. one-shot live observation開始前にpre-freeze位置を報告

v3がFAILした場合はimmutable記録し、別version successorへ進む。同じfrozen v3 surfaceを見てv5/v18をtuningしない。
