# Engine 0.6 evidence relevance holdout successor v10

Status: immutable holdout-v10 FAIL と successor-v10 development v1/v2/v3 FAIL、その後の development v4 PASS を受けた pre-freeze successor。candidate semantics は effective qualification v15 + materialization v28。successor-v10 semantics-freeze tag が存在するまで fresh acceptance holdout の authoring は禁止する。

## Historical boundary

Canonical holdout-v10 run 37085574010 は frozen v11/v23 の immutable FAIL のまま。Successor-v10 development v1 run 37097092197、v2 run 37100490212、v3 run 37105677785 も immutable FAIL development evidence のまま保持する。rerun / rescore / relabel は行わず acceptance evidence にも使わない。

v4 が変更するのは事前 adjudicate 済みの successor gap のみ:
- canonical Groq v10h18: Ambiguous -> Irrelevant
- v2 Mistral sv10v2_18: Irrelevant -> Relevant
- v3 Mistral sv10v3_08: Ambiguous -> Irrelevant
- v3 Mistral sv10v3_20: Irrelevant -> Ambiguous
- available v3 Google partial の direct-definition miss を unrelated terminal change なしで修正

Frozen v11 / v14 / v23 / v27 は変更しない。

## Effective qualification v15

v15 は v14 から派生し、successor-only の bounded change を2点だけ追加する。

1つ目は instruction/control text に対する model-only DifferentRelation の fail-closed 化。exact-target / risk-none で、独立した clean Harness-owned negative cue、Harness-owned requested-relation authority、strict absence rule がない場合、untrusted instruction/control text 由来の model-only DifferentRelation を Unresolved に落とす。advisory label を直接指定する control-schema instruction もこの one-sided safety floor に含める。この経路は model-only negative authority を除去できるだけで、RequestedRelation / DifferentRelation authority を新規生成しない。

2つ目は direct Definition wording。requested relation が Definition 以外で、target-owned substantive segment に "is defined as" / "defined as" があり、既存の exact-target / no-risk / local-authority 制約を満たす場合のみ bounded な other-relation evidence として扱う。comparison-only、other-entity、context-gap、strict absence、instruction-shaped content は fail-closed を維持する。

## Materialization v28

v28 は v15 を導出し、advisory proposal bindings と local qualification の両方を Harness-owned effective state に同期してから frozen v23 behavior に委譲する。これにより v15 が修正した relation / identity authority を stale raw proposal が再導入できない。

v28 自体は requested-relation / different-relation authority を新規生成しない。

## Fresh development v4 evidence

24-case development-v4 surface は v4 provider observation 前に author 済みで、observed holdout-v10 / development-v1 / v2 / v3 合計80件に対し case ID / entity / task / exact signal / 8-token window の再利用は0。

annotated tag engine-0.6-evidence-relevance-successor-v10-development-v4-freeze、commit d9a5bd49d3c6f3ee5e79f9ba6704f0dc502a00c0 からの one-shot run 37111376390 は immutable PASS development evidence。

- Mistral ministral-8b-latest: 24/24 complete、authority / identity-risk / materialization failure 0、effective v15 exact 24/24、final v28 exact 24/24
- Google gemini-3.5-flash-lite: 24/24 complete、authority / identity-risk / materialization failure 0、effective v15 exact 23/24、final v28 exact 24/24
- Groq は未観測のまま fresh independent acceptance holdout 用に保持
- holdout_acceptance_evidence=false

run と summary は fixtures/evidence-relevance-successor-v10-development-v4-result/ に固定済み。

## Pre-freeze audit

development-evidence commit a661ba648aba3573bbcf01d2efb05ff4675d0179 時点:
- exact-head GitHub CI 8/8 PASS
- working tree clean、branch HEAD == origin
- semantic candidate files は observed v4 freeze commit d9a5bd49d3c6f3ee5e79f9ba6704f0dc502a00c0 から byte-for-byte unchanged
- v4 validate-only は 24 planned / 0 completed / non-scorable
- checksum、fmt、core/CLI Clippy -D warnings、core library tests、v4/v28 tests、historical replay、successor-v9、holdout-v10、predecessor regression suites は green
- production semantic source/export に v4 case ID / synthetic entity literal の special-case は0

semantics-freeze commit で追加してよいのは freeze evidence / documentation のみ。behavior code と observed development surface は変更しない。

Intended freeze coordinate: engine-0.6-evidence-relevance-successor-v10-semantics-freeze。

この annotated tag を push した後にだけ fresh independent acceptance holdout runner/corpus を author できる。holdout は observed development surface を再利用せず、Mistral + Google に加えて Groq を復帰させる。
