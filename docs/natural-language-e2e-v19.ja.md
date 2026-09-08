# Natural-language E2E v19 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv19をv0.4.2向けのfresh held-out release surfaceとして使う。successor identity間でproduct candidateは変更してよいが、測定指標の定義は変更しない。

## v19を作る理由

freeze済みv18 Mistral run `34233275948` はv11-locked ruler不変のvalid failed product evidenceである。control / candidateはいずれも13 case完走、operational failure 0、correctness/safety zero gate維持だったが、candidate `03d68b4ab3a51a7457eafe3a4cc820a687b940e9` はtarget recall `0.6 -> 0.6`、tool selection `0.8 -> 0.4`、false abstention `5 -> 6`、avoidable follow-up stall `1 -> 2`、trigger exposure `2/3 -> 1/3`、exposed #249 conformance `1.0 -> 0.0`へ悪化した。v18はimmutableのまま保持する。

直接観測された原因は、`target_id`を持つ一方で`capability_id`を欠く`acquire` proposalが`invalid_shape`としてrejectされ、`round_budget`まで反復したことだった。#281はprovider-neutralなmodel-facing action schemaだけをclosed acquire/stop unionへ狭く変更し、acquireではnon-emptyな`target_id`と`capability_id`をrequired、stopでは両IDを禁止する。runtime parser/validationはfail-closedのままID推測・補完を行わない。sibling merge、fuzzy matching、provider-specific correctness、authority、admission、verification、finalization、answer safetyは変更しない。

v19はfresh candidate `1b889d843616ca257ec5e0e98411906e2c7152f6`、seed 76000で測定する。evaluator/comparator semanticsはv11-lockedのまま変更しない。

v19ではv11の意味をそのまま固定する。

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
- candidate: 現在は `1b889d843616ca257ec5e0e98411906e2c7152f6`。失敗後に狭いproduct修正を行う場合は新しいsuccessor identityへ進める。

Task、target key/value、expected outcome、source behavior、provider/model、seed、token budgetはpairで固定する。v0.4.1には`selection_priority`が存在しないため、そのfieldだけcandidate follow-up configへ追加する。MCP configはexact coordinate commitを指す`fixed_arguments.ref`だけが異なる。その他のsemantic driftは`validate_natural_language_e2e_v19_pair.py`でrejectする。

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

v19 corpus、evaluator、comparator、provider/model、seed 76000、max tokens 1024、workflow、checksumはlive credential前にfreezeする。まずMistralのpaired control/candidateを実行し、その同一freeze commitでpaired gateを通過した場合だけcross-modelへ進む。Google lane内は直列、Groqは別provider laneとする。

candidateが失敗した場合、v19のmetricを変更したり同じcandidateを再実行して新しい証拠として扱ったりしない。freeze済みtelemetryで原因を直接診断し、Harness/productへ狭い修正を行い、新しいsuccessor identityで同じmetric意味のpaired comparisonを行う。
