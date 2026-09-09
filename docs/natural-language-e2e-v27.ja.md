# Natural-language E2E v27 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv27をv0.4.2 release gateのfresh held-out successorとして使う。v21〜v26はimmutableなhistorical surfaceであり、再実行・再採点・調整・書き換えをしない。

## v27を作る理由

freeze済みv26のMistral paired canonicalは、v11で固定したrulerのままPASSした。その後のfreeze済みcross-model run `34356517560` はimmutableなFAIL evidenceである。Gemini pairedはreleased-control structured planner JSONのEOFで失敗したが、historical diagnosticがprovider terminal statusを十分保持していなかったため、generic malformed JSONと`status=incomplete`/token exhaustionをv26からは区別できない。Groq `openai/gpt-oss-120b` candidate-only parityは`JsonSchema -> JsonObject -> strict Text`の後に3件のprotocol failureがあり、effective 256-token investigation planner/action budget下でstrict TextがHTTP success/choicesを返した一方contentは空だった。Gemma pairedではprovider-unavailable HTTP 500とstructured malformed JSONの両方を観測した。generic protocol failureは引き続きnon-retryableであり、v26は再実行・再採点しない。

独立に正当化したproduct fix #316をPR #317でcandidate commit `c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7`へmergeしてからv27を構築する。investigation plan/action requestはprovider-neutralな`ModelReasoningPreference::Minimize`を使用する。既知のGroq GPT-OSS model IDだけがlow reasoning + returned reasoning抑制へmappingされ、通常のGroq wire requestは変更しない。reasoning本文はHarness answerへ昇格しない。empty-content / structured-parse failureではbounded terminal finish/status/token diagnosticを保持し、opt-in diagnostic traceに`structured_mode = json_schema | json_object | text`を記録する。これらのdiagnosticはclassification、retry、evaluator input、scoringには使わない。#311 operational-only retry、strict structured parsing、planner token budget、authority/admission/verification/finalization、v11 metric lockは不変である。

v27はreleased v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` とcandidate `c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7` を、fresh seed `94000` とfresh 13-case corpusでpair測定する。

## Freshness / immutability

v27ではcase ID、task、fact key、source identity、fresh marker、seedを新規化する。deterministic testで、これらがv1〜v11のrepository surfaceとfreeze済みv12〜v26 refに存在しないことを機械確認する。workflowでもcredentialを出す前にv21〜v26 freeze SHAをexact pinする。

acceptance branchで変更できるのはv27 fixture、evaluator-driver、comparator/validator、docs、checksum、workflowだけ。`Cargo.toml`、`Cargo.lock`、`crates/` はcandidate commit `c0ee9aeb6efd58816eb0d51ff1c7b011d48f19a7` とbyte-identicalでなければならず、acceptance中のworkspace versionは`0.4.1`のままとする。

## 固定測定定義

release metricはすべてv11の意味を維持する。target recallはadmit済み`expected_fact_key`のexact recall。tool-selection successは事前定義relevant capabilityの実行。avoidable follow-up stallはfollow-up target recalledかつaction 0。trigger exposureはconfigured cacheの最初の実行がtyped `no_result`。mechanism conformanceはtrigger-exposed caseだけを分母にして、直後のconfigured registry continuationとexact targetに対する`harness_no_result_followup_selections`を要求する。

Action rejection record、precedence telemetry、diagnostic trace、operational retry auditはdiagnostic専用で、release metricの分子・分母には一切入れない。operational failureはsemantic correctnessとは別に扱う。

`validate_natural_language_e2e_v27_metric_lock.py` は `natural-language-e2e-v26-freeze` と比較し、locked runner scoring functionのAST diff 0、freeze済みv26が同じoperational-retry wrapperを既に含むためsession scoringのexact AST equality、acceptance comparator / pair validator / acceptance testのnormalized equalityを要求する。正規化してよいのはversion identity、fresh seed、candidate SHAだけ。

## Release rule

paired必須rowは次の3つ。

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Google `gemma-4-31b-it`

各candidate rowは自身のreleased-control rowに対してtarget recall、tool-selection success、false abstention、全correctness/safety zero fieldでnon-worse必須。follow-up stallを増やさず、v11 trigger reachabilityを下げない。controlが0 stall / 3 triggerの2-metric ceilingでない限り、locked follow-up utilityの少なくとも1項目をstrictに改善する。controlがceilingならexact non-regressionを要求する。trigger-exposed時のmechanism conformanceは1.0必須。cross-model averagingは禁止する。

Groq `openai/gpt-oss-120b` はreleased v0.4.1にgeneric Groq pathがないためcandidate-only parity。10 investigation + 3 session caseをoperational failure 0、同一correctness boundaryで完走する必要がある。

## 実行規律

corpus、evaluator、comparator、retry policy、provider/model、seed `94000`、max tokens `1024`、workflow、checksumをlive credential前にfreezeする。最初にMistral paired control/candidateを実行し、同じfreezeのMistral paired workflowがSUCCESSした場合だけcross-modelへ進む。candidate/live observationがFAILした場合、その結果はimmutable evidenceとして保存し、修正が必要ならindependent product fix + fresh successorへ進む。同一v27をtuning・rerunして新しいcanonical observationにはしない。
