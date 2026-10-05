# Engine 0.6 evidence relevance independent holdout v3a result

Status: immutable canonical FAIL。

- Freeze tag: engine-0.6-evidence-relevance-holdout-v3a-freeze
- Freeze commit: 7d8d0cb5bb665f7fd26e147f3140076e63c38409
- Canonical run: 36586770785、attempt 1 only
- Cases: 26（Relevant 8 / Irrelevant 10 / Ambiguous 8）
- Required provider: Mistral + Groq + Google
- Effective qualification: v5
- Materialization: v18
- Operational: 3 armすべて26/26、provider failure 0
- Final gate: FAIL

先行v3 freeze run 36585464494はstale suite-id assertionによりlive observation前のpreflightでFAILした。v3aで修正したのはoperational assertion/tag identityだけで、manifest、label、v5 qualification、v18 materialization semanticsは不変。

この結果はimmutable。rerun / rescore / relabel / freeze tagの移動・再作成 / PASSへの読み替えは禁止。

## Provider result

Mistral ministral-8b-latest:
- authority-qualified 23/26
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- utility miss 1
- tokens 45,907
- latency p50/p95/max 1,253 / 5,001 / 7,187 ms
- terminal miss: v3h02_positive_authorized_alias_availability、Relevant -> Ambiguous

Groq openai/gpt-oss-120b:
- authority-qualified 23/26
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- utility miss 1
- tokens 65,015
- latency p50/p95/max 10,697 / 11,272 / 14,151 ms
- retry wait 0
- terminal miss: v3h25_ambiguous_single_signal_near_sibling、Ambiguous -> Irrelevant

Google gemini-3.5-flash-lite:
- authority-qualified 23/26
- materialized exact 25/26
- wrong-target Relevant 0
- false relevance rejection 0
- utility miss 1
- tokens 47,277
- latency p50/p95/max 7,288 / 37,747 / 59,582 ms
- retry wait 0
- terminal miss: v3h18_negative_comparison_different_relation、Irrelevant -> Ambiguous

## Root cause

残るgapはprovider operationではなくHarness/model authority composition。

1. Harness-authorized canonical/alias identityが複数local signalでsubstantive contentを所有し、requested relationが独立確認できる場合、model identity disagreementより強いpositive identity authorityになり得る。
2. single-signal near siblingは、両model stageがdifferent/distinctでもnegative identity authorityとして不足し、Ambiguousへabstainすべき。
3. comparison-only target mention + stable repeated sibling subjectはnegative ownershipを確立でき、requested relationの明示的除外は別axisでdifferent relationを確立できる。
4. URL-only target identityでlocal ownerがunnamedならcontext gapを維持する。

comparison-only textからpositive relevanceを作らず、mere anchor absenceだけでrejectせず、mapping / ownership / truncation uncertaintyを潰してはならない。

## Final gate

- operational completeness: PASS
- correctness hard gate: PASS
- utility: FAIL
- materialization: FAIL
- qualification: FAIL

Issue #462はopenのまま。別version successorはimmutable observationをregression用途にのみreplayできる。successor semanticsをfreezeした後でのみfresh independent holdoutをauthorする。
