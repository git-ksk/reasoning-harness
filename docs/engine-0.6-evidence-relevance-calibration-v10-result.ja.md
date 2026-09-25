# Engine 0.6 evidence-target relevance calibration v10 結果

Status: **immutable FAIL**。canonical run `36135286772`、attempt 1、freeze tag `engine-0.6-evidence-relevance-calibration-v10-freeze`、freeze commit `2acaf526823add813c07d96d0c258a39d864c20c`。rerun / rescore / tag overwriteは禁止する。

## Operational結果

v10ではv9のtransport/provider failureは解消した。3 provider armすべて56/56完走、provider failure 0。

- Mistral `ministral-8b-latest`: 56/56、provider failure 0
- Groq `openai/gpt-oss-120b`: 56/56、provider failure 0
- Google `gemini-3.5-flash-lite` replication: 56/56、provider failure 0

structured JsonSchema/JsonObject safety transportはv9のraw-Text protocol abortを再発せず、Google seed normalizationによりv9のsigned-32-bit HTTP 400も解消した。

## Semantic結果

Mistralはmaterialized exactがv9の38/47から49/56まで改善したがrequired gateはFAIL。unsafe negative rejection 1、unsafe positive acceptance 1、false relevance rejection 1、utility miss 7、materialized exact 49/56。primary bindingは22/56 exactに留まり、誤ったprimary routeにより本来必要なsafety stageへ到達しないfailureも残った。

Groqはoperational完走した一方、semantic safetyが不十分。unsafe negative rejection 14、unsafe positive acceptance 1、wrong-target relevance retention 1、relevant left ambiguous 1、utility miss 17、materialized exact 38/56。`safe_to_reject`というaction wordingがopen-world identity / ownership ambiguityでも選択されやすかった。

Google replicationも同じ構造的問題を示した。unsafe negative rejection 9、unsafe positive acceptance 2、wrong-target relevance retention 1、utility miss 9、materialized exact 46/56。

## Root cause

v10のtransport修正自体は成功したが、semantic architectureでは依然として単一のmodel classificationがhard final dispositionへ直結しすぎていた。

残ったfailure modeは2つ。

1. **Primary-route bypass**: primary `target/relation` bindingを誤るとsafety stage自体をbypassできる。代表例は`55_fresh_positive_injection_ignored`で、Mistralが明確なpositive relationを`different`と誤判定し、v6がpositive reviewなしに`irrelevant`へmaterializeした。
2. **Action-label overreach**: secondary modelの`safe_to_reject` / `safe_to_accept`がhard dispositionを直接許可した。Groqはrename / alias / successor / shared ownership / partial contextを多く`safe_to_reject`へ倒し、Groq / Googleはshared・uncertain ownershipをpositive acceptした例もあった。

これはsafety gateを緩めたりtransportだけ再調整して解く問題ではない。successorではhard dispositionに複数の独立semantic signalの一致を要求し、secondary stageにはfinal actionではなくevidence qualification / riskを報告させる必要がある。

## Successor方針

v11はtwo-key materialization boundaryへ移行する。

- primary target/relation bindingはadvisoryのままとし、exact-target caseをprimaryだけでhard rejectできなくする。
- 独立したstructured local-qualification guardは`accept/reject` actionではなくtarget-local supportと明示的ambiguity riskを返す。
- `Relevant` / `Irrelevant`はprimary proposal + Harness-owned anchor + independent guardの一致を要求する。
- disagreementまたはidentity / ownership / context riskが1つでもあれば`Ambiguous`。
- exact-target `relation=different`を無条件deterministic rejectionしない。
- hard safety gateはzero-toleranceを維持する。

v10 FAILからindependent holdoutは作成しない。fresh canonical successor PASSまでholdout authoringは引き続き禁止する。
