# Engine 0.6 evidence relevance successor-v10 development v3

Status: Issue #468 の pre-observation fresh development candidate。v3 provider observation はまだ実施していない。

## 背景

Development v2 run 37100490212 は immutable FAIL。Google は18/18 PASS。Mistral は authority / identity-risk failure 0 だったが、v26 が corrected v14 effective qualification ではなく stale raw qualification を frozen v23 に渡したため materialized 17/18 となった。

v3 は effective qualification v14 を一切変更しない。変更は final composition のみ。

## Candidate materialization v27

v27:
1. original proposal / raw qualification から v14 effective local qualification をderive;
2. final relevance policy自体は frozen v23 に委譲するが、local qualification inputとして v14 effective qualification を渡す;
3. materialization policy IDだけ v27 にする。

新semantic cue、新model call、requested-relation authority、直接的なRelevant/Irrelevant特例は追加しない。目的は final materialization が stale provider-local state ではなく既にauthoritativeな v14 resultを消費することだけ。

## Historical replay

v3 provider observation 前に:

- frozen successor-v9 / holdout-v10 / development-v2 expected contractをexact維持;
- canonical holdout-v10 全78 observations replayで terminal change は1件だけ:
  - Groq v10h18: Ambiguous -> Irrelevant
- immutable v2 全36 observations replayで terminal change は1件だけ:
  - Mistral sv10v2_18: Irrelevant -> Relevant
- その他112 historical terminal dispositionsは不変。

これはdiagnostic replayでありimmutable runのrescoreではない。

## Fresh v3 development surface

fixtures/evidence-relevance-successor-v10-development-v3/manifest.json は新規20 case:

- require_different: 8
- require_requested: 8
- forbid_different: 3
- preserve_risk: 1

holdout-v10 / development-v1 / development-v2 の観測済み60ケースに対して case ID / entity / task / exact signal / 8-token signal-window reuse 0。

requested-relation controlには availability + feature support、limit + pricing、pricing + limit、definition + launch のmixed-relation contentも含め、v2 wordingを再利用せずv27 compositionを直接検証する。

## Live development gate

required provider:
- Mistral ministral-8b-latest
- Google gemini-3.5-flash-lite

Groq は candidate shaping から除外し、後続fresh independent holdoutまで温存。

各providerは通常の proposal + local qualification pathで20 caseを1回ずつ観測。

PASS条件:
- operational 20/20
- provider/protocol failure 0
- effective identity / scope risk exact
- require_different -> different_relation
- require_requested -> requested_relation
- forbid_different -> different_relationではない
- preserve_risk -> expected risk維持
- materialized disposition 20/20 exact
- wrong-target Relevant / false relevance rejection / relevant-left-Ambiguous / utility miss 全て0

raw proposal/local labelsはdiagnostic扱い。

## One-shot discipline

最初のv3 observationは annotated tag engine-0.6-evidence-relevance-successor-v10-development-v3-freeze に固定。tagはimmutable、workflow rerunは禁止。

v3 PASS + successor semantics別freeze完了まで fresh independent holdout authoringは禁止。
