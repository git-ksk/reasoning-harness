# Natural-language E2E v26 — metric固定のv0.4.1 / v0.4.2 paired acceptance

Issue #263ではv26をv0.4.2 release gateのfresh held-out successorとして使う。v21〜v25はimmutableなhistorical surfaceであり、再実行・再採点・調整・書き換えをしない。

## v26を作る理由

freeze済みv25 Mistral paired run `34342578577` はPASS。attempt 1はlive API実行前のdeterministic Transport flake `external_command::tests::investigation_adapter_uses_separate_request_identity` で停止したため、failed jobのみをrerunし、attempt 2を同一freeze上のcanonical live observationとした。

freeze済みv25 cross-model run `34344002296` はcandidate性能FAILではなくoperationally incompleteだった。released v0.4.1 Google control側でpaired comparison成立前に失敗し、Geminiはprotocol generation failure 1件、GemmaはGoogle HTTP 500のprovider-unavailableが反復した。Groq candidate-only parityはv24のoperational failure 9件からv25で3件まで改善したが、残り3件は`JsonSchema`からdegradeした後の`JsonObject`もGroq server-side JSON generationを要求し、HTTP 400 generation/validation failureになっていた。

v25はそのままfreezeする。その後、held-out identityに依存しないgeneric fixを2件mergeしてからv26を作成した。

- #312 / PR #313: Groqのrecognized structured generation/validation failureを`JsonSchema`と`JsonObject`の両方でtyped capability failureとして扱い、generic structured-call ladderを`JsonSchema -> JsonObject -> Text`までdegrade可能にした。Textでは同じschemaをpromptへ埋め込み、既存のstrict Harness parser / serde contractだけで受理する。repair、fuzzy extraction、additional-field許容、authority変更、evaluator変更、scoring変更はしない。
- #311 / PR #314: paired evaluationへbounded case-level operational-only retry driverを追加した。retry対象はtyped transient provider generation failure（`transport` / `provider_unavailable` / `timeout`）と、released control互換のGoogle empty-model-text protocol subtypeだけ。同じcommand/model/seed/token/config/coordinateを再利用する。generic protocol、invalid JSON、semantic/scoring failure、quota、credentials、通常のprovider 4xx、unsupported capability、resolver/tool/action failure、rendering fallbackはretryしない。最初のoperationally completeまたはnon-retryable attemptをcanonicalとし、それ以前のoperational attemptはaudit専用。whole-run retryは禁止する。statefulなsession add/correctはmodel実行前にinvalidation stateをpersistするためretry対象外とし、natural execution成功後にだけpersistするsession startのみretry-safeとする。

v26はreleased v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315` とcandidate `2107af7942dd3cb7dac5fc30603359095f4a2709` を、fresh seed `93000` とfresh 13-case corpusでpair測定する。

## Freshness / immutability

v26ではcase ID、task、fact key、source identity、fresh marker、seedを新規化する。deterministic testで、これらがv1〜v11のrepository surfaceとfreeze済みv12〜v25 refに存在しないことを機械確認する。workflowでもcredentialを出す前にv21〜v25 freeze SHAをexact pinする。

acceptance branchで変更できるのはv26 fixture、evaluator-driver、comparator/validator、docs、checksum、workflowだけ。`Cargo.toml`、`Cargo.lock`、`crates/` はcandidate commit `2107af7942dd3cb7dac5fc30603359095f4a2709` とbyte-identicalでなければならず、acceptance中のworkspace versionは`0.4.1`のままとする。

## 固定測定定義

release metricはすべてv11の意味を維持する。target recallはadmit済み`expected_fact_key`のexact recall。tool-selection successは事前定義relevant capabilityの実行。avoidable follow-up stallはfollow-up target recalledかつaction 0。trigger exposureはconfigured cacheの最初の実行がtyped `no_result`。mechanism conformanceはtrigger-exposed caseだけを分母にして、直後のconfigured registry continuationとexact targetに対する`harness_no_result_followup_selections`を要求する。

Action rejection record、precedence telemetry、diagnostic trace、operational retry auditはdiagnostic専用で、release metricの分子・分母には一切入れない。operational failureはsemantic correctnessとは別に扱う。

`validate_natural_language_e2e_v26_metric_lock.py` は `natural-language-e2e-v25-freeze` と比較し、locked runner scoring functionのAST diff 0、operational-retry wrapperだけを除去したsession scoringのnormalized equality、acceptance comparator / pair validator / acceptance testのnormalized equalityを要求する。正規化してよいのはversion identity、fresh seed、candidate SHAだけ。

## Release rule

paired必須rowは次の3つ。

- Mistral `ministral-8b-latest`
- Google `gemini-3.5-flash-lite`
- Google `gemma-4-31b-it`

各candidate rowは自身のreleased-control rowに対してtarget recall、tool-selection success、false abstention、全correctness/safety zero fieldでnon-worse必須。follow-up stallを増やさず、v11 trigger reachabilityを下げない。controlが0 stall / 3 triggerの2-metric ceilingでない限り、locked follow-up utilityの少なくとも1項目をstrictに改善する。controlがceilingならexact non-regressionを要求する。trigger-exposed時のmechanism conformanceは1.0必須。cross-model averagingは禁止する。

Groq `openai/gpt-oss-120b` はreleased v0.4.1にgeneric Groq pathがないためcandidate-only parity。10 investigation + 3 session caseをoperational failure 0、同一correctness boundaryで完走する必要がある。

## 実行規律

corpus、evaluator、comparator、retry policy、provider/model、seed `93000`、max tokens `1024`、workflow、checksumをlive credential前にfreezeする。最初にMistral paired control/candidateを実行し、同じfreezeのMistral paired workflowがSUCCESSした場合だけcross-modelへ進む。candidate/live observationがFAILした場合、その結果はimmutable evidenceとして保存し、修正が必要ならindependent product fix + fresh successorへ進む。同一v26をtuning・rerunして新しいcanonical observationにはしない。
