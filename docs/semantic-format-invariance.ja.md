# セマンティック判定器のフォーマット不変性研究

Issue #59では、モデルに提示するoutput representationをboundedに変更しても、`soft-semantic-v3`のsemantic decisionが維持されるかを調べる。これはcalibration専用のresearch surfaceである。runtimeのsemantic-judge contractを置き換えるものではなく、過去のholdout v1/v2/v3をtuningに使ってはならない。

## R1a 介入契約

主要なR1a比較では、`ModelOutputFormat::JsonSchema`だけを変更し、v3 primary requestのtask text、system text、request JSON、kind-specific decision guidance、authority boundary、reasoning preference、token budget、fixture、trial index、seedは維持する。

design re-reviewの結果、decision-only schemaは、v3 promptを変更しない条件では純粋なR1 representation changeではないと判明した。v3 promptは、`finding`が要求された `kind` と `target` を保持することを明示的に求める。decision-onlyまたはscalar-label schemaでは、その指示を満たすために必要なfieldが取り除かれる。したがってR1aでこれらのschemaを使うと、representation biasをinstruction/schema conflictおよびR2 materialization hypothesisと混同する。

そのためR1aでは、information-equivalent representationのみを使う。

- `v3_full_json`: 現在のv3 primary requestとschemaをそのまま使う。
- `nested_result_object`: 完全なv3 outputを `result` fieldの下にネストする。
- `decision_finding_tuple`: 完全なdecisionとoptional findingを、2要素JSON tupleとしてencodeする。
- `compact_key_object`: canonical decision label、完全なfinding payload、binding semanticsを保持しながら、top-level keyを `d`、`f` にcompact化する。

decision-only JSON、scalar label、echoされたfinding bindingを取り除くprotocolはR2に延期する。R2ではHarness-owned materializationを明示的に研究でき、R1に紛れ込ませずに済む。

baseline requestは `build_soft_judge_model_request` とbyte-for-byte equalityになることをregression-testする。R1a variantは、`output_format`以外のすべてのmodel request fieldを保持することをregression-testし、parseされたvariantはすべて同じv3 finding/binding validationに通す。

R1aにはfallbackを意図的に設けない。全variantがJSON Schema outputを要求する。provider capability failureまたはmalformed representationはoperational resultであり、semantic decisionではない。fallbackその他のenforcement-mechanism changeは、別途報告するR1b diagnosticに属し、pure representation flip estimateにpoolしてはならない。

## プロバイダーの強制適用忠実度

要求したoutput formatは、実効的なprovider enforcementと自動的に同じになるわけではない。studyでは両方のcoordinateを記録する。

- Mistralは `JsonSchema` をprovider側のstrict JSON Schema enforcementにmapする。
- Googleは `JsonSchema` をresponse JSON schema mechanismにmapする。
- 現在のNVIDIA Hosted NIM adapterは `JsonObject` と `JsonSchema` の両方を `json_object` にmapし、schemaを送信しない。したがってNVIDIAでR1aを実行しても、representation interventionはnullになる。

このため最初のR1a runnerはMistralとGoogleだけを受け付ける。NVIDIAはsemantic tuningの対象外とする。これはinterventionを現時点でinstantiateできないためである。NVIDIAはR1b、またはmaterialに送信されるschemaをprovider capabilityがサポートした後に再検討できる。

providerごとのeffective enforcement mechanismが異なるため、cross-provider resultをmatched observationとしてpoolしてはならない。R1 matchingは単一provider/model内で行う。

## 判定抽出境界

R1 parserは、完全なrepresentationを既存v3 finding contractに照らしてvalidationした後、untrusted `SoftJudgeDecision`だけを抽出する。新しい `SoftSemanticFinding` の構築、malformed semantic outputの修復、曖昧さの解消は行わない。runtime validation contractは変更しない。Harness-owned materializationはR2 hypothesisであり、R1 implementation detailではない。

完全なJSON valueが1つあり、その後ろにnon-JSON trailing textがある場合は、既存のbounded normalization policyに従う。複数のJSON value、無効なdecision label、必須finding payloadの欠落、kind/target bindingの不一致、semantic interpretationを要する出力はfail closedする。

## 対応付け比較

単一provider/model studyでは、caseを `(fixture_id, trial, seed)` でmatchし、providerとmodelをstudy-level coordinateとして固定する。

複数representationのstudyでは、format blockを1つずつ完全実行せず、fixture/trial単位でinterleaveする。representation orderは `(fixture_index + trial) mod representation_count` によりdeterministically rotateし、各caseは `execution_position` を記録する。これによりpositionをcounterbalanceし、provider-time/load driftがconfoundになるのを減らす。このexecution-design changeにはresearch configuration identity `format-invariance-r1a-v2` を使う。過去のbounded probeはv1 historical diagnosticとして残す。

`format_flip_rate` は次のとおり。

```text
変更されたsemantic decision数 / matched successful baseline-variant pair数
```

operationally incompleteなpairは別に数え、semantic denominatorから除外する。reportには完全なdecision-transition tableを残し、`abstain -> finding` のようなchangeが単一scalarに潰れないようにする。

representationごとに、protocol completion、precision、recall、decision coverage、ambiguous abstention、token usage、latencyを報告する。semantic metricはoperationally completeなtrialについてのみ出力する。R1aではfallbackをdisabled/not applicableとして報告し、fallbackがないことをruntime observationのzero-rateとして暗黙に扱わない。representation parsingに失敗したprovider responseも、返されたtoken usageはoperational reportに残す。

R1a runnerのoutput budgetはデフォルト512 tokenである。最初のboundedな256-token Mistral probeでは、baselineとnested formの両方でpositive causal fixtureがちょうど256 output tokenに達し、EOF parse failureとなった。この不完全なprobeはsemantic format evidenceではなくoperational truncation evidenceとして扱う。将来のtruncationをmalformed complete outputと区別できるよう、`finish_reason`を保持する。

matched 512-token rerunでは、token-limit finishは再現しなかった。失敗したMistral generationは、baselineとnested form双方のpositive fixture、およびambiguous baseline fixtureを含め、約310 output token後に `finish_reason=error` を返した。これらは `provider_generation_error` と分類し、semantic denominatorの外に置く。provider側structured-generation behaviorがcharacterizeされるまで、full Mistral R1a matrixへの拡大は保留する。parseに失敗する `stop` responseは `representation_protocol` のまま、token-limit finish reasonは `truncation_protocol` とする。

最初のGemini 3.1 Flash-Lite causal-triad probeは `nested_result_object` と `decision_finding_tuple` で完了し、各比較で3つすべてのmatched decisionがv3から変わらなかった（`format_flip_rate=0`）。ambiguous causal fixtureはbaselineとvariantの双方で `finding` だったため、これはformat stabilityであって、良好なuncertainty calibrationのevidenceではない。その後の `compact_key_object` probeはsemantic evidenceではない。3つのvariant callはすべてretry後にmodelのfree-tier request quotaを使い切った。このsampleはoperational historyとして保持し、Google adapterは現在、quota/rate-limit failureとgeneric provider failureを区別する。

## 測定した R1a キャリブレーション結果

Gemini 3.5 Flash-Liteは、全18 fixtureを対象とするcounterbalanced all-calibration single-trial studyを完了した。v3 baselineと `nested_result_object` はともに18/18がprotocol-completeで、matched decision changeは0だった。`compact_key_object` は17/18がcompleteで、successful pair間のchangeは0だった。`decision_finding_tuple` は7/18しかcompleteでなかった。completeなprovider responseが、`no_finding` または `abstain` とnon-null finding payloadを繰り返し組み合わせたためである。したがって、successful semantic pairが安定していても、representation choiceはprotocol robustnessに影響する。

続くR1a gateでは、v3 baselineと `nested_result_object` だけを5 matched trialで比較した。90 matched pair、seedは1000-1004である。両representationとも90/90がprotocol-completeで、operational failureはなかった。decisionは2つ変わり、いずれも `15_causal_incomplete_scope_ambiguous` で、`format_flip_rate = 2/90 = 0.0222` となった。seed 1001と1002ではV3が `finding`、nestedが `abstain` を返した。2つのflipは逆のexecution orderで発生したため、単純なfirst/second-call position effectでは説明できない。5 seed全体ではnested representationはfixture-stableであり、seed間で唯一decisionが不安定だったのは、その1つのambiguous fixtureにおけるv3 baselineだった。これはfixture/seed/representation interactionを示すbounded calibration evidenceであって、nestingが普遍的に優れている証明ではない。

## キャリブレーション専用の実行

research binaryはrequested pathをcanonicalizeし、このcheckoutの正確な `fixtures/semantic-judges` directoryだけを受け付ける。rename/copy/symlinkしたholdoutをtuning dataとして差し替えることはできない。

```text
cargo run -p reasoning-harness-cli --bin reason-format-study -- \
  fixtures/semantic-judges \
  --provider mistral \
  --model ministral-8b-latest \
  --representation nested-result-object \
  --fixture 07_causal_positive \
  --fixture 08_causal_negative \
  --fixture 09_causal_ambiguous \
  --seed 1000 \
  --trials 1
```

専用の `semantic-format-study` GitHub Actions workflowは、3つのcausal positive/negative/ambiguous triadをdefaultにする。v3 baselineはimplicitなので、default validationはfull cross-provider matrixから始めず、provider callを6回実行する。`all-calibration`とrepeated trialは、後段で明示的に選ぶ。

## 汚染と権限不変条件

- holdout-v1/v2/v3はimmutableな過去のdiagnostic evidenceであり、tuning dataではない。
- holdout-v4は、provider-neutralなR1-R3 candidateが事前宣言したcalibration gateを通過するまでblockedのままにする。
- model-specific semantic prompt/schema branchは禁止する。
- representation disagreementはrisk signalであって、truth voteではない。
- model outputはuntrusted/advisoryに限る。
- model outputはtrusted evidence、hard finding、verification receipt、epistemic promotion、verdict authorityを生成できない。
- operational failureを `no_finding` に変換してはならない。
- incomplete trialをsemantic denominatorに入れない。
- hidden chain of thoughtはpersistもevaluateもしない。

## 研究上の根拠

R1 designは、format restrictionによってmodel performanceが変わり得ること、また表面的には同等なstructured representation間でも変化し得ることを示すevidenceに基づく。

- Tam et al., *Let Me Speak Freely?* (EMNLP Industry 2024);
- Long et al., *LLMs Are Biased Towards Output Formats!* (NAACL 2025);
- Schall and de Melo, *The Hidden Cost of Structure* (RANLP 2025);
- Yuan et al., *Quantifying the Impact of Structured Output Format on Large Language Models through Causal Inference* (Findings of EACL 2026);
- Hamilton and Mimno, *Lost in Space: Finding the Right Tokens for Structured Output* (GEM 2026)。

これらの研究はmeasurementの動機にはなるが、Harnessのauthority boundaryを上書きしたり、provider-specific tuningを正当化したりするものではない。
