# Engine 0.6 evidence relevance holdout successor v9

Status: immutable holdout v8 FAIL と未採用 successor-v8/v10 development 観測の後続 pre-freeze successor。candidate は effective qualification v11 + materialization v23。successor-v9 semantics-freeze tag が存在するまで fresh holdout v9 の authoring を禁止する。

## Historical boundary

Canonical holdout v8 run 36946457925 attempt 1 は freeze commit 867b7a90b4911c09c1397e2d15642b3dc9895022 / tag engine-0.6-evidence-relevance-holdout-v8-freeze の immutable FAIL。

Mistral / Google / Groq は全て26/26完走、provider failure 0。3 provider 全て materialization 26/26 exact、wrong-target Relevant 0、false relevance rejection 0、relevant-left-Ambiguous 0、utility miss 0。required qualification gate が落ちた理由は v8h15_negative_explicit_separate_service の effective relation-scope miss が Mistral / Google に各1件あったことだけで、Groq は exact。

requested relation は availability。fresh distinct-target proposition は historical bounded availability lexicon ではなく semantic paraphrase で deployment coverage を表現していた。identity は既に distinct_target で正しく、この successor が所有するのは relation axis のみ。historical v8 は immutable FAIL のまま、rerun / rescore / relabel / retag しない。

## v10 development result

Effective qualification v10 は Availability / Pricing / Limit / Change/Launch / Definition / Benefit/Use-case の bounded positive semantic frame を導入し、identity と scope-risk semantics を維持した。

candidate 706902c9e7ccb6213b8f021c945159ffe86bff41 の fresh successor-v8 development run 36978705359 は FAIL のまま保持する。Mistral は14/14で successor-owned relation authority exact。Google も14/14完走したが non-frame control で relation-authority miss が3件残った。positive frame recall だけでは不十分で、Harness-owned relation evidence が無くても historical model-only RequestedRelation fallback が relation authority を生成できる境界が確認された。

run は fixtures/evidence-relevance-successor-v8-development/ に capture し、development evidence としてのみ扱う。

## v11 selective relation authority

v11 は v10 に重ね、frozen v9/v22 は変更しない。

Harness-owned positive requested-relation authority は、substantive local material に requested coarse-relation semantic frame がある場合、または historical lexical requested-relation cue があり conflicting coarse semantic frame が無い場合だけ成立する。advisory model output だけで requested_relation になる場合、v11 は relation axis だけを unresolved に戻す。deterministic different_relation / relation_absent authority、identity scope、scope-risk は維持する。

frame detector は compositional / relation-specific。deployment coverage と feature support、commercial pricing と incidental currency/number、hard limit と observed count、rollout event と unrelated phrasal use、definition と documentation reference、benefit/use-case と topical mention を control で区別する。prompt-injection/control text は factual relation authority に使わず、URL/navigation/source-title も semantic-frame path から除外する。

## Materialization v23

v23 は frozen v22 へ delegate する前に v11 effective qualification を導出する。historical v22 が Relevant を materialize する一方で v11 に Harness-owned requested-relation authority が無い場合、v23 は conservative Ambiguous にする。positive safety floor のみで、negative authority、truth、provenance、source authority、target identity は拡張しない。

## Replay and controls

successor-v9 surface は6 coarse relation 全て、lookalike/non-frame、availability と general-availability/change conflict、model-only requested-relation agreement、model disagreement、exact-target lexical history、mapping/context risk、prompt injection、identity/relation orthogonality を含む。

historical replay は v9 identity scope 不変、v9 scope-risk 不変、expected requested-relation regression 0、false requested-relation authority 0、frozen materialization safety 維持を必須とする。

canonical holdout-v8 の v11 replay は relation-scope miss 0。v9 から v11 の変更は original v8h15 に対する Mistral / Google の2件だけで、expected requested_relation へ移る。Groq は不変。frozen v22 materialization は exact、wrong-target Relevant 0。failed v10 development も replay し、v11 は provider-specific branch 無しで Google の false-positive relation-authority 3件を除去する。

## Fresh successor-v9 development

fresh successor-v9 development surface は v11 provider observation より前に author した19 case。require_requested 8、forbid_requested 9、scope-risk preservation 2。CI は prior holdout / successor development manifest に対する case ID / task / canonical entity / exact 8-token surface window の reuse 0 を強制する。

candidate 78e635e1aea485e3f13fe78e01fa3a7f5ddd5e90 の live run 37015407859 は iterative provider 2本を完走。

- Mistral ministral-8b-latest: 19/19、provider failure 0、authority failure 0、identity/risk failure 0、materialization failure 0。
- Google gemini-3.5-flash-lite: 19/19、provider failure 0、authority failure 0、identity/risk failure 0、materialization failure 0。
- 両provider: wrong-target Relevant 0、false relevance rejection 0、relevant-left-Ambiguous 0、utility miss 0。
- final two-provider selective-authority development gate: PASS。

contract が conservative abstention を許す case では full effective-qualification exactness は診断値。Mistral 19/19 exact、Google は conservative relation-scope difference 5件で14/19 exact だが required/forbidden authority mode 違反は0、19件全て materialized disposition exact。

live observation と final summary は fixtures/evidence-relevance-successor-v9-development/ に capture し、unchanged v11/v23 で offline replay する。

## Freeze blockers

semantics freeze 前に relation controls、historical replay、captured live replay、surface independence、immutable holdout-v8 replay、wrong-target Relevant 0、production special-case scan、frozen v9/v22 invariance、orthogonality / false-positive controls、full workspace tests、all-target Clippy -D warnings、rustfmt、workflow YAML parse、diff check、exact-head GitHub CI を全て green にする。

## Pre-freeze audit

capture commit 842b6f4b07fc99c3ee0add823f279df0b295eda8 時点:

- full workspace tests PASS;
- all-target Clippy -D warnings PASS;
- rustfmt / workflow YAML parse / diff check PASS;
- semantic source/export の production special-case scan は successor-v9 case/entity literal 0;
- frozen derive_effective_evidence_local_qualification_v9 は successor-v7 semantics freeze から byte-for-byte 不変;
- frozen materialize_evidence_relevance_v22 も successor-v7 semantics freeze から byte-for-byte 不変;
- capture commit の exact-head GitHub CI は 8/8 green。

intended freeze coordinate: engine-0.6-evidence-relevance-successor-v9-semantics-freeze。

この tag を push した後にのみ fresh independent holdout v9 runner/corpus を author できる。holdout では Mistral + Google + Groq を one-shot cross-provider acceptance surface に戻し、successor-v9 development corpus を再利用しない。
