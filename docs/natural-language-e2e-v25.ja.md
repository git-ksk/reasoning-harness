# Natural-language E2E v25 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv25をv0.4.2向けのfresh held-out release surfaceとして使う。successor間のproduct変更はheld-outとは独立して正当化できる修正に限定し、測定定義はv11から固定する。

## v25を作る理由

freeze済みv24 Mistral paired run `34328390548` はprimary paired gateのimmutableなPASSである。その後のfreeze済みcross-model run `34329379978` はsemantic comparisonとしてのFAILではなくoperationally incompleteだった。Gemini rowはprotocol generation failure、Gemma rowはGoogle HTTP 500によるprovider-unavailableの反復、Groq candidate-only parity rowはstructured-output HTTP 400のgeneration/validation failureで完走できなかった。v24は再実行・再採点しない。v24 held-outのcase identityやanswerはv25 rationaleへ持ち込まない。

その後、held-outとは独立したprovider contract監査で #306 と #307 を発見した。PR #308はGoogle adapterが既に宣言している4-attempt ceilingを、retryable 5xxとempty-text transientでも最後まで使うようにする。PR #309はGroqのstructured JSON generation/validation failure familyを一貫して分類し、bounded native retryを使い切った後、generic investigation planner/action structured callを既存のschema-in-prompt JSON-object fallbackへ流す。いずれもprovider/transport robustnessの修正であり、Harness-owned parser、evidence admission、verification、finalization、answer-safety authority、candidate/final-render semantics、evaluator、scoring ruleは変更しない。

v25はreleased v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` とcandidate `c804551b381f48f14809f2ab37d0609c33006ddf` を、fresh corpus identityとseed 82000でpair測定する。

## 固定測定定義

v25でもv11の意味を変更しない。target recallはadmit済み`expected_fact_key`のexact recall、tool selectionは事前定義relevant capabilityの実行、avoidable follow-up stallはtarget recalledかつaction 0、trigger exposureはconfigured cacheの最初の実行がtyped `no_result`、mechanism conformanceはtrigger-exposed caseのみを分母として直後のconfigured registry follow-upと`harness_no_result_followup_selections`を要求する。

Action rejection class、precedence telemetry、diagnostic traceは、既に固定されたrelease zero fieldを除きdiagnostic専用であり、utility/correctness定義を変更しない。

## Release rule

paired必須rowは Mistral `ministral-8b-latest`、Google `gemini-3.5-flash-lite`、Google `gemma-4-31b-it`。各candidate rowでtarget recall / tool selection / false abstentionを悪化させず、follow-up stallを増やさず、trigger reachabilityを下げず、controlが0 stall / 3 triggerの上限でない限りfollow-up utilityの少なくとも1項目をstrictに改善し、trigger-exposed時のmechanism conformance 1.0と全correctness/safety zero gateを維持する。cross-model averageは禁止する。

Groq `openai/gpt-oss-120b` はreleased v0.4.1にgeneric Groq provider pathがないためcandidate-only parityとする。

## Diagnostic sidecar

candidate investigation callではcase単位の`reason-natural-diagnostic-trace-v1` sidecarを保存する。trace contractはartifact preservation時に検証するが、evaluator inputやscoring signalには使わない。released controlへdiagnostic flagは渡さない。

## 実行規律

corpus、evaluator、comparator、provider/model、seed 82000、max tokens 1024、workflow、checksumをlive credential前にfreezeする。まずMistral paired control/candidateを実行し、同じfreeze commitでpaired gateをPASSした場合だけcross-modelへ進む。FAILしたcandidateを同じidentityで再実行・再採点・tuningして新しい観測として扱わない。
