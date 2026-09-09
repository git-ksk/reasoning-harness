# Natural-language E2E v28 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv28をv0.4.2 release gateのfresh held-out successorとして使う。v21〜v27はimmutableなhistorical surfaceであり、再実行・再採点・調整・書き換えをしない。

## v28を作る理由

freeze済みv26のMistral paired canonicalは、v11で固定したrulerのままPASSした。その後のfreeze済みcross-model run `34356517560` はimmutableなFAIL evidenceである。Gemini pairedはreleased-control structured planner JSONのEOFで失敗したが、historical diagnosticがprovider terminal statusを十分保持していなかったため、generic malformed JSONと`status=incomplete`/token exhaustionをv26からは区別できない。Groq `openai/gpt-oss-120b` candidate-only parityは`JsonSchema -> JsonObject -> strict Text`の後に3件のprotocol failureがあり、effective 256-token investigation planner/action budget下でstrict TextがHTTP success/choicesを返した一方contentは空だった。Gemma pairedではprovider-unavailable HTTP 500とstructured malformed JSONの両方を観測した。generic protocol failureは引き続きnon-retryableであり、v26は再実行・再採点しない。

独立に正当化したprovider fix #316はPR #317で`c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7`へmerge済みである。v27のVALID FAIL後、別の一般的product defectとして#320を特定し、PR #321でexact-target `no_result` continuationをround-limit直前でも失わない修正をmain `756b63b5a5cbe024f89797b6c7da48be6700bc36`へmergeした。このpost-#320 mainをv28 candidateとする。investigation plan/action requestはprovider-neutralな`ModelReasoningPreference::Minimize`を使用する。既知のGroq GPT-OSS model IDだけがlow reasoning + returned reasoning抑制へmappingされ、通常のGroq wire requestは変更しない。reasoning本文はHarness answerへ昇格しない。empty-content / structured-parse failureではbounded terminal finish/status/token diagnosticを保持し、opt-in diagnostic traceに`structured_mode = json_schema | json_object | text`を記録する。これらのdiagnosticはclassification、retry、evaluator input、scoringには使わない。#311 operational-only retry、strict structured parsing、planner token budget、authority/admission/verification/finalizationは不変である。v11のtarget recall / tool selection / false abstention / follow-up stall / trigger reachability / correctness / safety算式も維持し、#319で事前定義したv12差分だけを追加する。

v28はreleased v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` とcandidate `756b63b5a5cbe024f89797b6c7da48be6700bc36` を、fresh seed `95100` とfresh 13-case corpusでpair測定する。

## Freshness / immutability

v28ではcase ID、task、fact key、source identity、fresh marker、seedを新規化する。deterministic testで、これらがv1〜v11のrepository surfaceとfreeze済みv12〜v27 refに存在しないことを機械確認する。workflowでもcredentialを出す前にv21〜v27 freeze SHAをexact pinする。

acceptance branchで変更できるのはv28 fixture、evaluator-driver、comparator/validator、docs、checksum、workflowだけ。`Cargo.toml`、`Cargo.lock`、`crates/` はcandidate commit `756b63b5a5cbe024f89797b6c7da48be6700bc36` とbyte-identicalでなければならず、acceptance中のworkspace versionは`0.4.1`のままとする。

## 固定測定定義

v11のrelease metric意味は維持する。target recallはadmit済み`expected_fact_key`のexact recall。tool-selection successは事前定義relevant capabilityの実行。avoidable follow-up stallはfollow-up target recalledかつaction 0。trigger exposureはconfigured cacheの最初の実行がtyped `no_result`であり、このtrigger定義・reachability算式は変更しない。

v12で追加する差分は#249 mechanismの**合法なcontinuation opportunity**の明示化だけである。`continuation_eligible`はtriggerが露出し、同じexact target IDがcase targetのexact fact keyへboundされ、configured cache以外に唯一のexplicit read-only exact-key follow-upが残り、通常のaction budget / no-progress terminal budgetがcontinuationを許す場合だけtrueとする。#320によりこのcontinuationはtriggerを生んだselection round内で実行されるため、round budgetだけではineligibleにしない。mechanism denominatorは`continuation_eligible` caseだけで、成功には直後のconfigured registryが**同じexact target ID**へ実行され、`harness_no_result_followup_selections`が増えることを要求する。eligible caseが0ならmechanismは`inconclusive`であり成功扱いにはしない。

released controlのmechanism conformanceはbaseline utility observationであり、control row validityそのもののhard gateにはしない。一方candidateはeligible opportunityが1件以上ある場合、mechanism conformance 1.0を必須とする。control/candidate両方についてoperational completenessとcorrectness/safety zero fieldsは従来どおりhard gateである。

Action rejection record、precedence telemetry、diagnostic trace、operational retry auditはdiagnostic専用で、v11 utility/correctness metricの分子・分母には入れない。operational failureはsemantic correctnessとは別に扱う。

`validate_natural_language_e2e_v28_metric_lock.py` はimmutableな `natural-language-e2e-v27-freeze` をpredecessorとして比較する。runnerで変更可能なのは`validate_corpus`、`score_investigation`、`aggregate`、`main`と新規`continuation_budget_allows`だけで、helper ASTをexact固定する。v11由来のscore return 46項目、aggregate return 63項目、主要score assignment、paired utility gate、hard correctness/safety ZERO fields、pair scrub semanticsはv27とAST一致を要求する。許可差分はv12 continuation eligibility/conformanceとcandidate-only hard mechanism gateだけである。

## Release rule

paired必須rowは次の3つ。

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Google `gemma-4-31b-it`

各candidate rowは自身のreleased-control rowに対してtarget recall、tool-selection success、false abstention、全correctness/safety zero fieldでnon-worse必須。follow-up stallを増やさず、v11 trigger reachabilityを下げない。controlが0 stall / 3 triggerの2-metric ceilingでない限り、locked follow-up utilityの少なくとも1項目をstrictに改善する。controlがceilingならexact non-regressionを要求する。released controlのmechanism conformanceはbaseline observationとして保持し、candidateは`continuation_eligible_cases > 0`ならmechanism conformance 1.0必須。cross-model averagingは禁止する。

Groq `openai/gpt-oss-120b` はreleased v0.4.1にgeneric Groq pathがないためcandidate-only parity。10 investigation + 3 session caseをoperational failure 0、同一correctness boundaryで完走する必要がある。

## 実行規律

corpus、evaluator、comparator、retry policy、provider/model、seed `95100`、max tokens `1024`、workflow、checksumをlive credential前にfreezeする。最初にMistral paired control/candidateを実行し、同じfreezeのMistral paired workflowがSUCCESSした場合だけcross-modelへ進む。candidate/live observationがFAILした場合、その結果はimmutable evidenceとして保存し、修正が必要ならindependent product fix + fresh successorへ進む。同一v28をtuning・rerunして新しいcanonical observationにはしない。
