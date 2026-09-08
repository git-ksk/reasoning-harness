# Natural-language E2E v17 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv17をv0.4.2向けのfresh held-out release surfaceとして使う。successor identity間でproduct candidateは変更してよいが、測定指標の定義は変更しない。

## v17を作る理由

v16はcandidate `3e6d0f5e...` に対するvalidなmetric固定paired Mistral観測だった。control/candidateとも13/13完走し、operational failureとcorrectness/safety violationは0。固定utility metricは target recall `0.6 -> 0.6`、tool selection `0.9 -> 0.9`、false abstention `6 -> 6`、avoidable follow-up stall `1 -> 1`、trigger exposure `2/3 -> 2/3`、#249 conformance `2/2 -> 2/2` で同値だったため、acceptance comparatorは `no strict improvement in locked follow-up utility metrics` でFAILした。v16はimmutableなfailed evidenceとして保持し、再実行しない。

直接観測できたstall caseではtargetをrecallした後、action proposalが `invalid_shape` で4回rejectされ、acquisition 0のまま `round_budget` で終了した。ただしfreeze済みv16 reportはrejection countしか保持しておらず、invalid proposalの具体形状や#261 precedenceが選択されなかった理由までは直接観測できなかった。Issue #275では、rejected action proposalとtyped validation reasonを保持し、#261 skip reasonをdiagnostic-only telemetryとして記録し、typed rejectionを次のaction-planner requestへ渡し、acquire/stopのshape ruleを明示する狭いproduct/diagnostic修正を行った。ID推測補完、invalid actionの受理、identity merge、`selection_priority`公開、#269緩和、admission/verification/finalization/budget/evaluator/release gate変更は行わない。v17はproduct candidate `3b3c2d35f437838603cdd88c489fe93843f6c011` を測定する。

v17ではv11の意味をそのまま固定する。

- `target_recalled`: expected fact keyがadmit済みinvestigation planに存在すること。
- `tool_selection_success`: 実行actionの少なくとも1つが事前定義したrelevant capabilityを使うこと。
- avoidable follow-up stall: freeze済みfollow-up caseで `target_recalled=true && action_count=0`。
- `trigger_exposed`: 最初に実行されたconfigured cache actionがtyped `no_result`を返すこと。
- #249 denominator: trigger-exposed caseのみ。
- #249 conformance: 直後にconfigured registryへ進み、`harness_no_result_followup_selections`が記録されること。

`harness_precedence_selections`とaction rejection classはdiagnostic telemetryに限定し、trigger reachabilityの定義やrelease合否を置き換えない。

## Paired coordinate

同じfresh logical corpusを次の2座標へ実行する。

- control: released v0.4.1 `29a9e4be6273dbffeda324e15517dc64930ad315`。
- candidate: 現在は `3b3c2d35f437838603cdd88c489fe93843f6c011`。失敗後に狭いproduct修正を行う場合は新しいsuccessor identityへ進める。

Task、target key/value、expected outcome、source behavior、provider/model、seed、token budgetはpairで固定する。v0.4.1には`selection_priority`が存在しないため、そのfieldだけcandidate follow-up configへ追加する。MCP configはexact coordinate commitを指す`fixed_arguments.ref`だけが異なる。その他のsemantic driftは`validate_natural_language_e2e_v17_pair.py`でrejectする。

## Release rule

paired必須rowはMistral `ministral-8b-latest`、Google `gemini-3.5-flash-lite`、Google `gemma-4-31b-it`。各rowでcandidateは次を満たす。

- target recall、tool-selection success、false abstentionをcontrolより悪化させない。
- avoidable follow-up stallを増やさない。
- v11定義のtrigger reachabilityを下げない。
- controlが0 stall / 3 triggerの構造上限でない限り、上記2 utility metricの少なくとも一方をstrictに改善する。
- trigger-exposed時の#249 conformanceを1.0で維持する。
- v0.4.x correctness/safety zero gateをすべて維持する。

cross-model averageは禁止し、強いmodelで別modelのregressionを隠さない。

Groq `openai/gpt-oss-120b`はv0.4.1 generic `reason`に存在しなかったためcandidate-onlyとする。同じ10 investigation + 3 session corpusをgeneric provider pathで完走し、同じcorrectness boundaryを維持することをgateにする。

## 実行規律

v17 corpus、evaluator、comparator、provider/model、seed 74000、max tokens 1024、workflow、checksumはlive credential前にfreezeする。まずMistralのpaired control/candidateを実行し、その同一freeze commitでpaired gateを通過した場合だけcross-modelへ進む。Google lane内は直列、Groqは別provider laneとする。

candidateが失敗した場合、v17のmetricを変更したり同じcandidateを再実行して新しい証拠として扱ったりしない。freeze済みtelemetryで原因を直接診断し、Harness/productへ狭い修正を行い、新しいsuccessor identityで同じmetric意味のpaired comparisonを行う。
