# Natural-language E2E v24 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv24をv0.4.2向けのfresh held-out release surfaceとして使う。successor間のproduct変更は、held-outとは独立して正当化できる修正に限定し、測定定義はv11から固定する。

## v24を作る理由

freeze済みv23 Mistral paired run `34325217339` はimmutableなVALID FAILである。control/candidateはいずれも13 caseを完走し、correctness/safety boundary violationはなかった。candidateはduplicate-action rejectionを0にし、target recall `0.6 -> 0.6`、tool selection `1.0 -> 1.0`、avoidable follow-up stall `0 -> 0`、trigger exposure `3 -> 3`、mechanism conformance `1.0 -> 1.0`を維持したが、false abstentionは`4 -> 6`へ悪化した。v23は再実行・再採点しない。v23 held-outのcase identityやanswerはv24 rationaleへ持ち込まない。

別のgeneric final-render contract監査で #303 を発見した。renderer instructionは意図する全factual propositionを`factual_claims`へ列挙するよう要求していた一方、model-facing JSON Schemaはruntime `#[serde(default)]`を継承し、このfield省略を許していた。PR #304ではmodel-facing schemaのみを狭めて`factual_claims`をrequiredにする。runtime parsingは後方互換のまま、`finalize_answer()`はauthoritative/fail-closedのまま、canonical recovery scopeも変更しない。task proseからのtarget inference、fuzzy matching、admitted-target rewrite、evidence/verification authority変更、provider固有correctness branch、evaluator/scoring変更は行わない。

v24はreleased v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` と candidate `a0c145dc713b848719c539ea2fbc6bf436db85a2` を、fresh corpus identityとseed 81000でpair測定する。

## 固定測定定義

v24でもv11の意味を変更しない。target recallはadmit済み`expected_fact_key`のexact recall、tool selectionは事前定義relevant capabilityの実行、avoidable follow-up stallはtarget recalledかつaction 0、trigger exposureはconfigured cacheの最初の実行がtyped `no_result`、mechanism conformanceはtrigger-exposed caseのみを分母として直後のconfigured registry follow-upと`harness_no_result_followup_selections`を要求する。

Action rejection class、precedence telemetry、diagnostic traceは、既に固定されたrelease zero fieldを除きdiagnostic専用であり、utility/correctness定義を変更しない。

## Release rule

paired必須rowは Mistral `ministral-8b-latest`、Google `gemini-3.5-flash-lite`、Google `gemma-4-31b-it`。各candidate rowで target recall / tool selection / false abstentionを悪化させず、follow-up stallを増やさず、trigger reachabilityを下げず、controlが0 stall / 3 triggerの上限でない限りfollow-up utilityの少なくとも1項目をstrictに改善し、trigger-exposed時のmechanism conformance 1.0と全correctness/safety zero gateを維持する。cross-model averageは禁止する。

Groq `openai/gpt-oss-120b` はreleased v0.4.1にgeneric Groq provider pathがないためcandidate-only parityとする。

## Diagnostic sidecar

candidate investigation callではcase単位の `reason-natural-diagnostic-trace-v1` sidecarを保存する。trace contractはartifact preservation時に検証するが、evaluator inputやscoring signalには使わない。released controlへdiagnostic flagは渡さない。

## 実行規律

corpus、evaluator、comparator、provider/model、seed 81000、max tokens 1024、workflow、checksumをlive credential前にfreezeする。まずMistral paired control/candidateを実行し、同じfreeze commitでpaired gateをPASSした場合だけcross-modelへ進む。FAILしたcandidateを同じidentityで再実行・再採点・tuningして新しい観測として扱わない。
