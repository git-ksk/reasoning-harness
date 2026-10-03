# Engine 0.6 evidence relevance successor-v10 development v2

Status: Issue #468 の pre-observation fresh development candidate。v2 provider observation はまだ実施していない。

## 背景

Canonical holdout-v10 run 37085574010 は frozen effective qualification v11 / materialization v23 で immutable FAIL。terminal semantic miss は Groq の v10h18 1件だけで、exact-target telemetry observation に対し provider が different_relation ではなく relation_absent を返したため Ambiguous に残った。

Successor-v10 development v1 は専用 one-sided model verifier を追加する two-key design を試した。run 37097092197 は immutable FAIL。Mistral は expected positive verifier family を全てmissし、prompt-injection controlを false confirm。all-provider PASS が不可能になった時点で Google / Groq は quota 節約のためcancelした。

v2 はこの失敗した model verifier を完全に外す。unchanged v11/v23 baseline の上に、Harness-owned の限定local structureだけを追加する。

## Candidate semantics

candidate effective qualification は v14、materialization は v26。

v14 は v11 から開始し、relation_absent / unresolved だけを different_relation に変更できる。次を全て必須とする:

1. proposal / local qualification が両方存在;
2. deterministic scope risk = none;
3. effective identity = exact_target;
4. Harness-owned exact target anchor が存在;
5. current relation が non-positive;
6. この deterministic path に関係する untrusted-instruction marker が無い;
7. other-relation cue が、Harness target名/aliasがlocal ownerとして現れる substantive segment 内にあり、comparison/context-only mentionではない;
8. bounded cue が別relationを明示する;
9. requested relation の local lexical cue が無い;
10. Harness-owned requested-relation authority が無い;
11. strict target/relation absence ruleではない。

bounded cue は、既存 conflicting coarse semantic frame、availability query に対する feature/API/protocol support、limit/quota query に対する numeric benchmark/telemetry observation に限定する。

lexical exclusion は意図的。 "became generally available" のような wording は launch/change と availability の両方に見えるため、v14 は negative authority を作らず unresolved を維持する。

v26 はまず frozen v23 に委譲し、v23 が Ambiguous かつ v14 が exact_target / different_relation / none の場合だけ Irrelevant に変更する。Relevant へのpromotionはできない。

v13/v25 は development-v1 の historical semantics として変更しない。

## Fresh v2 development surface

fixtures/evidence-relevance-successor-v10-development-v2/manifest.json は新規18 case:

- require_different: 8
- require_requested: 6
- forbid_different: 3
- preserve_risk: 1

canonical holdout-v10 と観測済み successor-v10 development v1 の両方から独立。prior 42 observed cases に対する case ID / entity / task / exact signal / 8-token signal window reuse 0 をpre-observationで必須とする。

recovery family は exact-target limit vs throughput/latency observation、availability vs feature/API support、pricing vs limit、definition vs launch、benefit/use-case vs pricing、launch vs definition。

control は true requested relation、generic/explicit absence、context-gap telemetry、numeric benchmark風 prompt injection、requested relationと別relationの同居を含む。

## Historical replay requirements

provider observation 前に:

- frozen successor-v9 development expected contract が v14/v26 で不変;
- frozen holdout-v10 expected contract が v14/v26 で不変;
- canonical holdout-v10 全78 observations replayで変更するterminal dispositionは Groq v10h18 Ambiguous -> Irrelevant の1件だけ;
- Mistral / Google canonical v10 disposition は全件不変;
- v10h25 cross-frame "became generally available" は Ambiguous 維持;
- comparison-only target mention、target-title + other-entity-body は target-local negative relation authorityを作らない。

historical replay は development diagnostic であり、holdout-v10 のrescoreではない。

## Live development gate

required development provider は Mistral ministral-8b-latest と Google gemini-3.5-flash-lite。

Groq は candidate shaping から除外し、successor semantics freeze 後の fresh independent holdout まで温存する。

各providerは通常の proposal + local qualification 2-stage pathで18 caseを1回ずつ観測。専用negative-relation model callは追加しない。

provider PASS条件:

- operational 18/18;
- provider/protocol failure 0;
- effective identity / scope risk がprecommitted contractと一致;
- require_different -> different_relation;
- require_requested -> requested_relation;
- forbid_different -> different_relationではない;
- preserve_risk -> expected deterministic risk維持;
- final v26 disposition 18/18 exact;
- wrong-target Relevant / false relevance rejection / relevant-left-Ambiguous / utility miss 全て0。

raw proposal / raw local qualification のexact labelはdiagnostic。acceptanceはHarness-owned effective semantics + final materializationで定義する。

## One-shot discipline

最初のv2 provider observationは annotated tag engine-0.6-evidence-relevance-successor-v10-development-v2-freeze に固定する。

tagはimmutable、workflow rerunは禁止。FAILした場合もrescore/relabelしない。

v2 development PASS + 別途 successor-semantics freeze が完了するまで新しい independent holdout はauthorしない。
