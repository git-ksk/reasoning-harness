# Engine 0.6 evidence relevance successor-v10 development v4

Status: Issue #468 の pre-observation fresh development candidate。v4 provider observation は未実施。

## 背景

Canonical holdout-v10 は frozen v11/v23 の immutable FAIL。successor-v10 development v1 / v2 / v3 も immutable FAIL development evidence。

v3 run 37105677785 で残存gapを2つ特定した:
- launch query に対する direct affirmative definition wording "is defined as" を別relationとして認識できない;
- instruction/control text由来の advisory model-only DifferentRelation が inherited v11 negative pathを通り、v27がeffective semanticsを正しくcomposeした結果 terminal Irrelevant になり得る。

v4 は v14 / v27 を変更せず、新effective qualification v15 / materialization v28 を導入する。

## v15 negative-authority boundary

v15 は v14 からderiveする。

まず次の場合、model-only DifferentRelation を Unresolved にdemoteする:
- identity = exact_target;
- deterministic scope risk = none;
- candidate に untrusted instruction/control text がある;
- clean factual segment に独立した Harness-owned negative-relation cue が無い;
- requested-relation Harness authority が無い;
- strict target/relation absence ではない。

instruction/control text はinert。別のclean factual segmentがあれば、そのsegmentのbounded negative-relation evidenceは維持できる。

v15では frozen v14 のmarker vocabulary自体は変更せず、successor-onlyで `relation_binding` / `relation_scope` / `final disposition` などのmodel-facing fieldや、`different_relation` / `irrelevant` 等の結果ラベルを命令するcontrol-schema textも検出する。旧来の自然言語prompt-injection phraseを使わないcontrol textがmodel-only negative authorityを残す境界を閉じる。この検出はone-sidedで、model-only negative authorityを除去するだけであり、別clean segmentのHarness-owned factual cueは維持する。

さらに successor-only direct Definition cueを追加する。target-owned substantive segmentの "is defined as" / "defined as" は、requested relationがDefinitionでなく通常のv14 safety restrictionを満たす場合に別relationのDefinition evidenceとして扱える。

frozen v10/v11/v14のglobal semantic frameは変更しない。

## v28 materialization

v28 は v15 を frozen v23 へcomposeする。

v27と異なり、v28はlocal qualificationだけでなく advisory proposal bindingもHarness-owned effective v15 semanticsへ同期してからv23へ渡す。これによりstale model proposalが、v15で補正済みのrelation/identity decisionを再生成できない。

v28自体は RequestedRelation / DifferentRelation authorityを新規生成しない。

## Historical replay

v4 provider observation前に、precommitted expected contractと観測済み履歴をreplayする。

対象:
- canonical holdout-v10 observations
- immutable v2 observations
- immutable v3 Mistral observations
- cancel前に取得済みv3 Google partial observations
- frozen successor-v9 / holdout-v10 / v2 / v3 expected contract

意図するterminal changeは既知missだけ:
- canonical v10 Groq v10h18: Ambiguous -> Irrelevant
- immutable v2 Mistral sv10v2_18: Irrelevant -> Relevant
- immutable v3 Mistral sv10v3_08: Ambiguous -> Irrelevant
- immutable v3 Mistral sv10v3_20: Irrelevant -> Ambiguous
- v3 Google partial の direct-definition missを修正し、無関係なterminal changeは0

## Fresh v4 development surface

fixtures/evidence-relevance-successor-v10-development-v4/manifest.json は新規24 case:
- require_different: 11
- require_requested: 9
- forbid_different: 3
- preserve_risk: 1

過去観測済み holdout-v10 / v1 / v2 / v3 の80 caseに対し、case ID / entity / task / exact signal / 8-token signal-window reuse 0。

fresh recovery family:
- direct Definition wording vs launch/pricing/availability/benefit
- limit vs throughput/latency observation
- availability vs feature support
- pricing vs limit
- definition vs launch
- benefit vs pricing

control:
- true requested relation
- clean negative factual segment + separate instruction segment
- requested relation + separate instruction segment
- requested relation + another relation同居
- numeric / definition-shaped prompt injection
- generic no-relation
- clipped direct-definition context

## Strengthened offline gate

provider observation前に:
- v28 unit control PASS
- v28 historical replay PASS
- v4 expected-path PASS
- require_different 全caseをconservative model outputで強制してもHarness cueで DifferentRelation / Irrelevantを回収
- prompt-injection controlへ adversarial model DifferentRelation を入れても Ambiguous維持
- context-gapを DifferentRelationへ回収しない
- control-schema injection / comparison-only target mention / distinct-target Definition / strict requested-relation absenceから exact-target negative relation authorityを生成しない
- validate-onlyは24 planned / 0 completedかつmodel/provider call 0
- v4 workflowはpredecessor v3ではなくv4 configuration/suite IDを検証する
- frozen successor-v9 / holdout-v10 / v1 / v2 / v3 test green
- fmt / Clippy / checksum / validate-only green

## Live development gate

required development provider:
- Mistral ministral-8b-latest
- Google gemini-3.5-flash-lite

Groq は candidate shaping から除外し、後続fresh independent holdoutまで温存する。

各providerは通常の proposal + local qualification pathで24 caseを1回観測。

PASS条件:
- operational 24/24
- provider/protocol failure 0
- identity / scope-risk exact
- require_different -> different_relation
- require_requested -> requested_relation
- forbid_different -> different_relationではない
- preserve_risk -> expected risk維持
- final v28 disposition 24/24 exact
- wrong-target Relevant / false relevance rejection / relevant-left-Ambiguous / utility miss 全て0

raw proposal/local labelはdiagnostic扱い。

## One-shot discipline

最初のv4 provider observationは annotated tag:

engine-0.6-evidence-relevance-successor-v10-development-v4-freeze

に固定。tagはimmutable、workflow rerunは禁止。FAILした場合もrescore/relabelしない。

v4 development PASS + 別途 successor-semantics freeze 完了後にのみ fresh independent acceptance holdout をauthorする。
