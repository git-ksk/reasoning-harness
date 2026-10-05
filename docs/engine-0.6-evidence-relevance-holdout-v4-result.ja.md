# Engine 0.6 evidence relevance independent holdout v4 結果

Status: immutable canonical FAIL。

- freeze tag: engine-0.6-evidence-relevance-holdout-v4-freeze
- freeze commit: 4e60db418175be36f1e910f52be88c4cb3b34830
- canonical run: 36650257492、attempt 1のみ
- 26 case（Relevant 8 / Irrelevant 10 / Ambiguous 8）
- required provider: Mistral + Groq + Google
- effective qualification: v6
- materialization: v19
- operational: 3 armすべて26/26、provider failure 0
- correctness hard gate: 全provider PASS、wrong-target Relevant = 0
- final gate: FAIL

この結果はimmutable。rerun / rescore / relabel / freeze tagの移動・再作成 / PASSへの再解釈は禁止する。

## Provider結果

Mistral ministral-8b-latest:
- effective authority qualification exact: 25/26
- materialized exact: 24/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 2
- tokens: 47,537
- latency p50/p95/max: 1,228 / 2,305 / 2,819 ms
- terminal miss:
  - v4h11_negative_navigation_target_other_subject: Irrelevant期待 -> Ambiguous
  - v4h12_negative_generic_catalog_absence: Irrelevant期待 -> Ambiguous

Groq openai/gpt-oss-120b:
- effective authority qualification exact: 26/26
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 1
- tokens: 67,212
- latency p50/p95/max: 10,636 / 10,999 / 11,099 ms
- terminal miss:
  - v4h15_negative_prompt_injection_absence: Irrelevant期待 -> Ambiguous

Google gemini-3.5-flash-lite:
- effective authority qualification exact: 25/26
- materialized exact: 25/26
- wrong-target Relevant: 0
- false relevance rejection: 0
- relevant left Ambiguous: 0
- utility miss: 1
- tokens: 49,165
- latency p50/p95/max: 6,974 / 10,311 / 19,898 ms
- terminal miss:
  - v4h14_negative_explicit_separate_service: Irrelevant期待 -> Ambiguous

v4h22_ambiguous_truncated_relation は3 providerとも最終Ambiguousで正しかったが、Groq / Googleはdeterministic context_gap下で relation_absent を返しており、terminal safetyは保ったままauthority qualification missとなった。

## Root cause

残存gapはprovider operationではなく、negative authorityの体系的なcomposition不足。

1. navigation/footerだけのtarget名はproposition ownershipを作らない。本文のheading/bodyで別subjectが安定して反復される場合、advisory modelがexactに誤投票してもlocal ownerを別targetとして確定可能。
2. bounded local unitがtarget/relation不在を明示している場合、そのdeterministic local absenceはnegative evidenceとして使える。advisory exact/exactだけでAmbiguousへ戻してはならない。
3. 反復されるsibling subjectがHarness targetとは別service/productだと明示される場合、Harness-owned negative identity evidenceとして扱える。
4. visible truncation / context gapを relation_absent に昇格させない。relation bindingがunresolvedならeffective relationもunresolvedを維持する。
5. wrong-target Relevant hard gateは弱めず、global absenceを推論せず、single near-name signalだけではdistinct-target authorityを作らない。

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- utility: FAIL
- materialization: FAIL
- qualification: FAIL

Issue #462はopenのまま。successor semanticsはimmutable v4 observationをregression replayにのみ利用できる。successor-v5 semanticsをfreezeする前のfresh independent holdout v5 authoringは禁止する。
