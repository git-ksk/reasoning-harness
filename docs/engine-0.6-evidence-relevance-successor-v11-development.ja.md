# Engine 0.6 evidence relevance successor v11 development

Status: immutable holdout-v11 FAIL 後の pre-observation fresh development candidate。

Historical boundary:
- holdout-v11 freeze: engine-0.6-evidence-relevance-holdout-v11-freeze / 4343e7939e964e91d466a0788358733d629e92f2
- canonical run: 37182677114、immutable FAIL
- result record: 08885938bd989bd26ca0b3a3f193b372b149bbba
- Mistral / Google は PASS、Groq の v11h21_ambiguous_generic_no_relation だけが terminal miss
- rerun / rescore / relabel / freeze-tag move は禁止

## Candidate semantics

Effective qualification v16 は frozen v15 から派生する。

中心ルールは negative authority の対称化。exact target に対し、Harness が target-owned の affirmative な別 coarse relation proposition を確認できない場合、advisory model output だけで DifferentRelation authority を作らせない。

v15 後に model-only DifferentRelation が残っても Harness-owned alternate-relation cue が無ければ v16 は Unresolved に戻す。RequestedRelation / identity / scope-risk authority は新規生成しない。

successor-only cue v4 は frozen global semantic frame を変更せず、v3 cue に以下を追加する:
- requested relation が Limit 以外で、target-owned limited to + numeric は Limit evidence
- requested relation が ChangeOrLaunch 以外で、target-owned launched / launches / released / rollout は ChangeOrLaunch evidence

materialization v29 は v28 full composition を維持し、v16 effective state を frozen v23 に渡す。stale advisory proposal が削除済み negative authority を再導入できない。

## Historical replay

successor-v9 development、holdout-v10、successor-v10 development v2/v3/v4、holdout-v11 の実provider observation を replay。

holdout-v11 の terminal change は Groq v11h21 の Irrelevant -> Ambiguous だけ。新規 terminal miss は0。

## Fresh development surface

Suite: evidence-relevance-successor-v11-development
Cases: 22
Protocol: v16 + v29
Fixed core: evidence-relevance-fixed-core-successor-v11-development-v1

内訳:
- require_different 8
- forbid_different 6
- require_requested 4
- preserve_risk 2
- preserve_absence 2
- final Irrelevant 10 / Ambiguous 8 / Relevant 4

observed holdout-v1-v11 / successor-v5-v10 development に対し case ID / canonical entity / task / exact signal / 8-token signal window の再利用を禁止する。

property control:
- true alternate relation は conservative model output 下でも DifferentRelation を回復
- generic/no-relation 6件は adversarial DifferentRelation vote 下でも DifferentRelation にせず Ambiguous
- strict target-specific relation absence は RelationAbsent / Irrelevant を維持
- requested positive / scope risk は維持

## Provider policy

development observation は Mistral + Google のみ。

Groq は candidate shaping から除外し、successor-v11 semantics freeze 後の fresh independent holdout でのみ復帰可能。

Intended development freeze coordinate:
engine-0.6-evidence-relevance-successor-v11-development-v1-freeze
