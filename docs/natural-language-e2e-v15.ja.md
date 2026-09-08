# Natural-language E2E v15 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv15をv0.4.2向けのfresh held-out release surfaceとして使う。successor identity間でproduct candidateは変更してよいが、測定指標の定義は変更しない。

## v15を作る理由

v14は、復元したv11固定metric意味の下で初めて成立したvalid paired control/candidate observationである。Mistral rowではreleased v0.4.1 controlは正常完走した一方、candidate `2d53a27d...` の `adaptive-rethane-owner` でproduct-side acquisition loopが露出した。`cache(no_result) -> registry(verification_progress)` の後、#261 precedenceが同じ `expected_fact_key` を持つ別のadmitted target identityをdeterministicに選べる状態となり、同じcapability chainをもう一度実行した。2巡目registryはtyped `malformed_output`となり、mechanism conformanceは3/3から2/3、downstream useful follow-upも3/3から2/3へ悪化した。したがってv14はimmutableなfailed product evidenceとして保持し、再実行しない。

Issue #269ではこのproduct defectだけを狭く修正した。`unique_precedence_action_proposal()` は、同じexpected fact keyを複数のadmitted target identityが共有する場合、片方のsiblingがcapability pairを使い切った後でも常にfail-closedとなる。target identityはmergeせず、#249 continuation、admission、verification、finalization、budget、evaluator定義は変更しない。v15はproduct candidate `c069954cca77361b0e8d3e91334a448efdd2acd9` を測るfresh held-out successorである。

v15ではv11の意味をそのまま固定する。

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
- candidate: 現在は `c069954cca77361b0e8d3e91334a448efdd2acd9`。失敗後に狭いproduct修正を行う場合は新しいsuccessor identityへ進める。

Task、target key/value、expected outcome、source behavior、provider/model、seed、token budgetはpairで固定する。v0.4.1には`selection_priority`が存在しないため、そのfieldだけcandidate follow-up configへ追加する。MCP configはexact coordinate commitを指す`fixed_arguments.ref`だけが異なる。その他のsemantic driftは`validate_natural_language_e2e_v15_pair.py`でrejectする。

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

v15 corpus、evaluator、comparator、provider/model、seed 71000、max tokens 1024、workflow、checksumはlive credential前にfreezeする。まずMistralのpaired control/candidateを実行し、その同一freeze commitでpaired gateを通過した場合だけcross-modelへ進む。Google lane内は直列、Groqは別provider laneとする。

candidateが失敗した場合、v15のmetricを変更したり同じcandidateを再実行して新しい証拠として扱ったりしない。freeze済みtelemetryで原因を直接診断し、Harness/productへ狭い修正を行い、新しいsuccessor identityで同じmetric意味のpaired comparisonを行う。
