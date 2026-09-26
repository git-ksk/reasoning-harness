# Engine 0.6 evidence-target relevance calibration v18 — successor design

状態: implemented pre-freeze candidate。v18 live observationは0。Independent holdout authoringは禁止継続。

## Fixed surface

- fixed core: `evidence-relevance-fixed-core-v1`
- case: 48件固定
- expected policy / candidate / proposal / verifier annotation / dispositionはv17と完全同一
- case ID / synthetic name / exact fixture phraseによるproduction分岐なし

contract:
- primary proposal: `reason-evidence-relevance-binding-proposal-v5`（変更なし）
- local verifier: `reason-evidence-local-qualification-v8`
- Harness materializer: `target-evidence-relevance-binding-materialization-v13`
- annotation protocol: `evidence-relevance-scope-verifier-v18`

## Why v18 exists

v17はRequired Mistral 44/48、Google replication 46/48まで改善したが、Mistralでshared/clipped ownershipをwrong-target Relevantとして1件再発した。provider-authored scopeが明示clipping / omitted ownershipを落とす場合や、exact-target different-relationをdistinct-target identityへ混同する残差も確認した。v18はv17 scoring semanticsを維持したままHarness-owned deterministic local-risk floorとverifier boundary強化を導入する。

## Deterministic local-risk floor

terminal materialization前に、bounded candidate自身に明示されたambiguity cueをHarnessがdeterministicに検出する。generic detector対象:
- clipped / truncated / omitted excerpt・bullet・column・row・referent・captured passage;
- uncertain alias / rename / successor / product mappingの明示;
- unresolved row/product ownershipの明示;
- URL identity + local materialがproduct/valueをidentify/bindしない旨の明示。

具体的riskが1つでもあればv13は必ず`Ambiguous`。このfloorはfail-closedで、Relevant / Irrelevant authorityを新規生成しない。

## Verifier v8

3-field schemaは変更なし。promptを次の境界で強化:
- exact target + different relationは`exact_target/different_relation`;
- prompt-injection textはinert quoted dataでありtarget/relation absenceを作らない;
- omitted/clipped local contextはcontext gapであってlocal absenceではない;
- shared/multi-product row + omitted ownershipはownership risk、clipping併存なら`multiple`;
- complete generic/broad unitはtarget propositionがなければlocal absenceにできる。

## Materialization v13

v12 ruleを維持し、追加するのは:
- deterministic local-risk floorがあれば常にAmbiguous;
- verifierが`target_absent + relation_absent`、scope risk none、primary relation != exactなら、弱いtarget mentionでprimary targetだけexactに寄っていてもlocal absenceをIrrelevantへmaterialize可能;
- primary exact-target/different-relation + verifier different_relation + risk noneなら、Harness identity floorを破らない範囲でverifier identityがexactでなくてもIrrelevantへmaterialize可能。

変更はnegative disagreement解消とdeterministic riskだけ。positive rescue pathは増やさない。

## Operational policy

v17 hardeningをそのまま継承: active 60s、provider wait/retry 45s、single wait 30s、wall 120s、adapter-owned retry、typed quota 1件latch、capacity failure 2件latch、telemetry分離、public failure sanitize、manual TPD attestationなし。

Required providerはMistral `ministral-8b-latest` + Groq `openai/gpt-oss-120b`。Google `gemini-3.5-flash-lite`はnon-gating replication。

## Pre-live acceptance

freeze前に:
- v17 -> v18 scored semantics 48/48完全同一;
- routing/property testでdeterministic-risk false-positive boundaryと48件materializationをcover;
- runner / workspace / clippy / fmt / diff PASS;
- `surface-v18.sha256`一致;
- validate-only 48 planned / 0 observed / latchなし / non-scorable;
- PR #466 Draft維持;
- independent holdout未作成。

first/only frozen v18 canonicalは開始後one-shot immutable。
