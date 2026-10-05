# Engine 0.6 evidence relevance successor-v10 development

Status: Issue #468 の pre-observation fresh development candidate。successor-v10 の provider observation はまだ実施していない。

## 背景

Canonical holdout-v10 run 37085574010 は frozen effective qualification v11 / materialization v23 の下で immutable FAIL。required provider は全て operational 完走し safety boundary も維持したが、Groq が exact-target non-frame negative relation 1件を Irrelevant ではなく Ambiguous に残した。

successor では観測済み holdout-v10 surface の relabel / rerun / rescore / tuning を行わない。

v9後にローカルで試した v12/v24 は negative relation authority を広く対称化し、historical controls を regression させたため未commitで破棄済み。この番号を再利用せず、新しい限定candidateは effective qualification v13 / materialization v25 とする。

## Candidate boundary

successor は one-sided / restrictive。

専用 negative-relation verifier の出力は次の2値のみ:

- confirmed_different_relation
- not_confirmed

verifier は requested-relation authority、target identity、truth、freshness、verification、sufficiency、final relevance を生成できない。

v13 が v11 の relation_absent / unresolved を different_relation に変えられるのは、次を全て満たす場合だけ:

1. proposal / local qualification が両方存在;
2. deterministic scope risk = none;
3. effective identity = exact_target;
4. Harness-owned exact target anchor が存在;
5. current relation が non-positive;
6. bounded Harness-owned observable cue が別relationの明示的存在を示す;
7. Harness-owned requested-relation authority が無い;
8. strict target/relation absence ではない;
9. one-sided verifier = confirmed_different_relation。

observable cue は general semantic classification より意図的に狭い:

- 既存の conflicting coarse semantic frame;
- availability query に対する feature/API/protocol support（deployment/geography supportではない）;
- limit/quota query に対する numeric benchmark/telemetry observation（cap/maximum/quota frameではない）。

v25 が v23 Ambiguous を Irrelevant に変えられるのは、v13 が exact_target / different_relation / none かつ one-sided confirmation がある場合のみ。

Relevant へのpromotionは一切できない。

## Fresh development surface

fixtures/evidence-relevance-successor-v10-development/manifest.json は独立作成した16 case:

- confirmed_different_relation expected: 8
- not_confirmed expected: 8

positive側は limit vs observed telemetry、availability vs feature support、availability vs pricing、pricing vs limit、definition vs launch、benefit vs pricing、launch vs definition を含む。

negative control は true requested-relation、generic no-relation、explicit requested-relation absence、clipped context、prompt injection、omitted ownership/context、same-relation control を含む。

pre-observation automation で holdout-v10 の case/entity/task/signal/8-token surface reuse 0 を必須とする。

## Pre-observation deterministic checks

provider observation 前に:

- core v25 unit controls PASS;
- frozen successor-v9 relation controls / historical replay PASS;
- frozen holdout-v10 expectation tests PASS;
- fresh successor-v10 composition が monotone:
  - expected confirmation case は synthetic v23 Ambiguous baseline -> v25 Irrelevant;
  - expected not_confirmed case は v23 dispositionを完全維持;
- probe runner validate-only が provider call 0 で16 case manifestを検証。

## Live development gate

required provider:

- Mistral ministral-8b-latest
- Google gemini-3.5-flash-lite
- Groq openai/gpt-oss-120b

各 provider は16 case x matched 3 trials = 48 observations。

PASS条件:

- successful 48/48;
- confirmation match 48/48;
- false confirmation 0;
- missed confirmation 0;
- provider/protocol failure 0。

3 provider 全てPASSするまで candidate semantics はfreezeしない。freeze前の新しい independent holdout authoringは禁止。

最初の provider observation は pre-observation surface を annotated tag engine-0.6-evidence-relevance-successor-v10-development-v1-freeze に固定してから、その tag push で一度だけ起動する。workflow rerun は拒否する。

## Historical integrity

- holdout-v10 は immutable FAIL のまま;
- frozen v11/v23 behavior を変更せず replay;
- 破棄済み broad v12/v24 symmetry experiment は破棄済みのまま;
- この development surface は replacement holdout ではない。
