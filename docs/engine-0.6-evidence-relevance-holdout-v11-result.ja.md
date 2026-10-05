# Engine 0.6 evidence relevance independent holdout v11 result

Status: immutable canonical FAIL。

Freeze tag: engine-0.6-evidence-relevance-holdout-v11-freeze
Freeze commit: 4343e7939e964e91d466a0788358733d629e92f2
Canonical run: 37182677114、attempt 1 only
Cases: 26（Relevant 8 / Irrelevant 10 / Ambiguous 8）
Required providers: Mistral + Google + Groq
Effective qualification: v15
Materialization: v28
Final gate: FAIL

この結果は immutable。rerun / rescore / relabel / freeze tag の移動・再作成 / PASS への再解釈は禁止。

## Provider results

Mistral:
- operational 26/26、provider failure 0
- authority / identity-risk failure 0
- effective qualification 26/26 exact
- materialized 26/26 exact
- wrong-target Relevant 0、utility miss 0
- PASS

Google:
- operational 26/26、provider failure 0
- authority / identity-risk failure 0
- effective qualification 25/26 exact
- materialized 26/26 exact
- wrong-target Relevant 0、utility miss 0
- v11h22 の unresolved -> relation_absent は diagnostic-only の安全側差分
- PASS

Groq:
- operational 26/26、provider failure 0
- identity/risk failure 0
- effective qualification 24/26 exact
- materialized 25/26 exact
- wrong-target Relevant 0
- false relevance rejection 0
- relevant-left-Ambiguous 0
- utility miss 1
- FAIL

## Terminal miss

Case: v11h21_ambiguous_generic_no_relation

Task: Moss Gallery の月額料金は何か。
Candidate: Moss Gallery documentation covers account setup and navigation examples.

Frozen expectation:
- exact_target
- relation_absent
- risk none
- Ambiguous
- forbid_different

Groq の advisory negative relation が v15 で DifferentRelation のまま残り、v28 がその effective state を同期して Irrelevant に materialize した。

この candidate は target-owned の generic documentation だが、pricing とは別の coarse relation を affirmative に主張しているわけではない。requested relation が無いことと different relation の根拠があることは別であり、frozen expectation は維持する。

wrong-target Relevant や positive-authority の safety failure ではなく、negative relation authority が強すぎることによる over-rejection。

v11h20 の diagnostic difference は final Ambiguous を維持しており gate failure ではない。

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- qualification gate: FAIL
- materialization gate: FAIL
- utility gate: FAIL
- overall: FAIL

Mistral / Google は frozen acceptance contract を PASS。FAIL provider は Groq のみで、terminal miss も v11h21 の1件だけ。

## Successor direction

v11 の tuning / rerun / rescore / relabel はしない。

successor は一般則として、
1. exact target に属する affirmative な別 coarse relation proposition
2. requested relation が単に無い、または generic documentation しかない complete local unit
を分離する。

(2) で model-only DifferentRelation を terminal negative authority にしない。修正は one-sided とし、unsupported negative authority を除去できるだけで RequestedRelation authority は作らない。

fresh successor holdout 前に v15/v28、true different-relation family、instruction control、strict absence、context risk、identity scope、immutable historical run を replay する。

Issue #468 は OPEN、PR #469 は Draft のまま維持する。
