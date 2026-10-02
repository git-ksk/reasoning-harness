# Engine 0.6 evidence relevance independent holdout v2

Status: freeze candidate / unobserved。Issue #462 successor semantics（effective qualification v4 + materialization v17）は、`engine-0.6-evidence-relevance-successor-v2-semantics-freeze` / `55c4163acb815bc31db22da531766a26020091bf` ですでに先にfreeze済み。このsurfaceはその固定済みsemanticsを独立評価する。

## Independence boundary

holdout v2はsuccessor semantic freeze後にのみauthorした。case、entity名、candidate text、labelは新規であり、canonical run `36495389012` がimmutable FAILであるholdout v1をrewrite / relabel / rerun / PASS化しない。

v1でFAILを決めた `h14_negative_comparison_only_mention` をコピーまたは表層言い換えしていない。v2には `Umber DB` / `Violet DB` identityもholdout-v1 case IDも含まれない。freeze前監査ではholdout v1とのcanonical entity overlapは0、candidate textのexact 8-token n-gram overlapも0。

holdout v1のobserved outputはfrozen successor semanticsのpostmortem/replayにだけ使える。v2の新規labelやproduction branchの根拠には使わない。

## Frozen candidate contract

- suite: `evidence-relevance-holdout-v2`
- cases: 26
- label distribution: Relevant 8 / Irrelevant 10 / Ambiguous 8
- required providers:
  - Mistral `ministral-8b-latest`
  - Groq `openai/gpt-oss-120b`
  - Google `gemini-3.5-flash-lite`
- annotation protocol: `evidence-relevance-effective-qualification-v4`
- materialization: v17
- fixed core: `evidence-relevance-fixed-core-v2`
- seed: `4629202`
- intended freeze tag: `engine-0.6-evidence-relevance-holdout-v2-freeze`
- canonical policy: tag triggerのfirst run / attempt 1 only。canonical rerun / replacement observationは禁止

runnerはfrozen holdout v1との後方互換を維持する。v1はeffective qualification v3 + materialization v16のまま、v2 profileだけが明示的にv4/v17へ切り替わる。

## Acceptance gates

3 required provider armすべて26/26 operational完走が必要。各providerが独立に以下を満たすこと:

- wrong-target Relevant retention = 0;
- false relevance rejection = 0;
- Relevant left Ambiguous = 0;
- utility miss = 0;
- materialized exact = 26/26;
- effective authority qualification exact = 26/26;
- effective identity / relation authority miss = 0;
- scope-risk miss / spurious-risk = 0;
- provider-arm latch / operational abort / incomplete provider attemptなし。

operational provider failureはoperationalのまま扱い、semantic PASS / missへ変換しない。holdout v1がFAILしたzero-wrong-target hard gateは一切緩和しない。

## Groq TPD admission

tiny readinessはtransport / credential / TPM / RPD evidenceに限定し、TPD-headroom proofには使わない。

v2 workflowはimmutable holdout-v1 Groq observationからTPD modelを再anchorする:

- actual observed tokens: 65,244;
- observation completion anchor: `2026-09-29T01:10:50Z`（canonical job logでfinal total-token resultが記録された時刻）;
- v1 configured start headroom: 55,000;
- v1 frozen pacing: case間300,000 ms + minimum request interval 10,000 ms;
- conservative modeled v1 end headroom: 約7,719 tokens;
- v2 required modeled start headroom: 55,000;
- v2 26-case self-budget: 70,000 tokens;
- 次case前reserve: 4,000 tokens;
- v2 pacing: case間300,000 ms + minimum request interval 10,000 ms;
- precommitする最速modeled floor: `2026-09-29T06:51:16Z`（15:51:16 JST）。

workflow自身がfloorを再計算し、configured timestampがderived floorより早い場合、またはpaced supplyが70K self-budgetを満たさない場合はfail closedする。materialなorganization-level Groq利用が既知または疑われる場合、このmodelはinvalidateし、freeze tag push前に再anchorする。

## Freeze discipline

holdout-v2 freeze tag push前に:

1. v1 / v2 study profileをlocal validate;
2. immutable calibration v23 / holdout-v1 observationをfrozen v4/v17でreplay;
3. core/providers/CLI full test、workspace all-target Clippy、fmt、diff、workflow YAMLを実施;
4. exact surface checksumを再検証;
5. public diffについてsecret / local path / provider・entity固有production branch / holdout-v1 surface accidental reuseをscan;
6. PR #466をDraftのままexact-head PR CI greenまで確認;
7. long live observationを開始するfreeze前に現在位置を報告。

fresh independent holdoutがoriginal acceptance gateをPASSするまでIssue #462はopenのまま維持する。
