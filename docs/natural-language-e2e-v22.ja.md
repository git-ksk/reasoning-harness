# Natural-language E2E v22 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv22をv0.4.2向けのfresh held-out release surfaceとして使う。successor identity間でproduct candidateは変更してよいが、測定指標の定義は変更しない。

## v22を作る理由

freeze済みv21 Mistral paired run `34290310154` は、変更していないv11-locked ruler上のimmutableなVALID FAILである。canonical paired observationは両coordinateともcorrectness/safety boundary violationなしで完走したが、candidateのfalse abstentionが `3 -> 5` に悪化した。そのためv21はVALID FAILとして固定し、再実行・再採点・held-out caseへのtuningは行わない。

そのfreeze後、v21 held-out caseとは独立したcurrent-mainのcode/contract監査で #297 を発見した。investigation plan admissionは、#288がproviderから返されたfact keyをtrim/normalize/guess/post-provider rewriteしないexact identityとして要求しているにもかかわらず、model-proposed `expected_fact_key` をtrim/normalizeしていた。genericなregression-first fixtureで `" routing.owner "` が `"routing.owner"` としてsilently admitされ、Harness-owned exact-key selectionのeligibilityを人工的に作り得ることを再現した。PR #298はその境界だけを修正し、proposed fact keyをbyte-for-byteで保持する。target ID/question admission、selector ordering、evidence admission、authority、verification、grounding、finalization、answer safety、provider behavior、evaluator、trigger定義、release metricは変更しない。

別途merge済みの #295 observabilityはcandidate-only diagnostic sidecarを追加する。このtraceはartifact evidence専用で、natural output report、evaluator input、release scoringには入らない。released v0.4.1 controlへdiagnostic flagは渡さない。

v22はfresh candidate `9051049733a82571acb1ad572769b2c3f7495c50`、seed 79000、fresh pseudonym/task/fact/source surfaceで測定する。evaluator/comparator semanticsはv11-lockedのまま変更しない。

v22ではv11の意味をそのまま固定する。

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
- candidate: 現在は `9051049733a82571acb1ad572769b2c3f7495c50`。失敗後に狭いproduct修正を行う場合は新しいsuccessor identityへ進める。

Task、target key/value、expected outcome、source behavior、provider/model、seed、token budgetはpairで固定する。v0.4.1には`selection_priority`が存在しないため、そのfieldだけcandidate follow-up configへ追加する。MCP configはexact coordinate commitを指す`fixed_arguments.ref`だけが異なる。その他のsemantic driftは`validate_natural_language_e2e_v22_pair.py`でrejectする。

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

v22 corpus、evaluator、comparator、provider/model、seed 79000、max tokens 1024、workflow、checksumはlive credential前にfreezeする。まずMistralのpaired control/candidateを実行し、その同一freeze commitでpaired gateを通過した場合だけcross-modelへ進む。Google lane内は直列、Groqは別provider laneとする。

candidateの各investigation caseでは、runnerがcase単位のpathへ `reason-natural-diagnostic-trace-v1` sidecarを書き出す。candidate live boundaryへ入った場合、workflowは10本のtraceとcontract IDを検証し、canonical observation artifactと一緒に保存する。session caseとreleased controlは従来どおりである。sidecarの存在確認はevidence completenessであり、semantic metricではない。

candidateが失敗した場合、v22のmetricを変更したり同じcandidateを再実行して新しい証拠として扱ったりしない。freeze済みtelemetryで原因を直接診断し、Harness/productへ狭い修正を行い、新しいsuccessor identityで同じmetric意味のpaired comparisonを行う。
