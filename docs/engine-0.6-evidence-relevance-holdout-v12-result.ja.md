# Engine 0.6 evidence relevance independent holdout v12 result（結果）

Status: immutable canonical PASS。

Freeze tag: engine-0.6-evidence-relevance-holdout-v12-freeze
Freeze commit: 78c6894e871d9aa4dd79aef0a30c95647c075c4d
Canonical run: 37218652869, attempt 1 only
Cases: 26 (Relevant 8 / Irrelevant 9 / Ambiguous 9)
Required providers: Mistral + Google + Groq
Effective qualification: v17
Materialization: v30
Final gate: PASS
Holdout acceptance evidence: true

この結果は immutable。rerun / rescore / relabel / freeze tag の移動・再作成は禁止。

## Provider results

Mistral / ministral-8b-latest:
- operational 26/26, provider failures 0
- authority failures 0; identity/risk failures 0
- effective qualification exact 22/26; materialized exact 26/26
- wrong-target Relevant 0; false relevance rejection 0; Relevant-left-Ambiguous 0; utility misses 0
- total tokens 44,843
- PASS

Google / gemini-3.5-flash-lite:
- operational 26/26, provider failures 0
- authority failures 0; identity/risk failures 0
- effective qualification exact 25/26; materialized exact 26/26
- wrong-target Relevant 0; false relevance rejection 0; Relevant-left-Ambiguous 0; utility misses 0
- total tokens 46,022
- PASS

Groq / openai/gpt-oss-120b:
- operational 26/26, provider failures 0
- authority failures 0; identity/risk failures 0
- effective qualification exact 21/26; materialized exact 26/26
- wrong-target Relevant 0; false relevance rejection 0; Relevant-left-Ambiguous 0; utility misses 0
- total tokens 63,336
- PASS

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- qualification gate: PASS
- materialization gate: PASS
- utility gate: PASS
- overall acceptance: PASS

前回v11のGroq terminal failure classは、v12のfresh generic target-owned / no-classifiable-relation controlでcoverされ、terminal missは0。fresh ownership-gap / relation-only controlも identity/risk authority failureなしで frozen v17/v30 contractを通過した。

Issue #468 はopen、PR #469 はDraftのまま維持する。この文書はresearch acceptance結果のみを固定する。
