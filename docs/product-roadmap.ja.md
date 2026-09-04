# プロダクトロードマップ：エビデンスに基づくAI CLI

Reasoning Harness は、まずネイティブRust製の `reason` CLI としてプロダクト化される。v0.1.0では構造化された正確性と自動化の契約を確立し、v0.2.0では**AIを利用した自然言語CLI**をエンドユーザー向けの主要経路にした。現在の外部プレビュー製品リリースであるv0.3.0では、同じHarness所有ランタイム上に、制約付きの外部エビデンス取得と解決を追加している。ユーザーは、Harnessに推論させるためだけに内部JSONを組み立てる必要はない。

このプロダクトの目標は、汎用エージェントフレームワークより意図的に狭い。

> ユーザー、開発者、自動化に対して、検査可能でエビデンスに基づく推論プロセスを通じて生成され、型付きの不確実性、abstention、失敗の意味論をモデルではなくHarnessが担保する、シンプルなAIインターフェースを提供する。

研究は並行して継続する。新しい仕組みは、独立した検証と運用上の安定化を経て初めてCLIに昇格する。プロダクトの表面は、すべての実験を逐一追随するものではない。

## 現在のプロダクト経路

v0.3.0のデフォルト体験は、引き続き自然言語優先かつAIを利用する。

```text
自然言語のタスク
        |
        v
Reasoning Harness
        |
        v
モデルが信頼できない候補を生成
        |
        v
エビデンス / 検証 / semantic + answer-safetyゲート
        |
        +--> 支持が不足 -> 制約付き解決 / 再生成 -> 再検証
        |
        v
根拠付き回答 | 条件付き回答 | unknown
```

既存の構造化インターフェースを**削除するわけではない**。`HarnessInput`、`ReasoningCandidate`、`ReasoningArtifact`、schema discovery、`reason run --candidate`、`reason verify` は、引き続き高度な利用、統合、デバッグの表面および内部表現としてサポートされる。ただし、主要なAI経路を使う前にユーザーがそれらの表現を理解しなければならない状態を解消することが、今後のプロダクトの取り組みである。

自然言語の利便性によって正確性の境界を弱めてはならない。ユーザーの文章、ファイル内容、モデルによる抽出、ツール出力、過去のモデル出力は、CLIが受け付けたというだけで信頼済みエビデンスにはならない。エビデンスの取り込み、受け入れ、検証、semantic/answer-safety診断、制約付き解決、再検証、最終主張のカバレッジは、引き続きHarnessが所有する。

## 完了済み v0.2.0 プロダクトライン

1. **制約付きresolverのtarget closure（#159）：** 後継候補ラインで実装済み。Harnessが所有する未解決仮説/エビデンス要件を、候補が所有する未解決主張より先に、正確に優先する。一方、resolverクラス、予算、受け入れ、qualification、必須再検証は変更しない。
2. **renderer downgradeからの復旧（#160）：** 後継候補 `a020b5925497ff3fdf200a9622270fa1889a6aa1` で実装済み。rendererが、要求された許可済みtargetと完全一致するものを `uncertain` として出力した場合、artifact authorityから決定的に復旧する。`Unknown`/`Reject`、qualification、adversarial、answer-safetyの境界は維持する。
3. **依存関係を考慮したtarget-local recovery（#164）：** 後継候補 `993874fa0051d06a02c8db8f7a220a2ac7773c17` で実装済み。artifact全体の `Reject` は変えず、型付きのblocker/dependency/evidence isolationを実証できる場合に限り、直接検証済みの正確なtargetをtarget-onlyの `QualifiedPartialAnswer` として出力できる。結合が曖昧ならfail closedする。
4. **providerの信頼性と再開可能な評価（#126）：** 運用専用の後継レイヤーとして実装済み。Googleの一時的な5xx/分離された空出力のretryを狭く制限し、実際のprovider試行をtelemetryに伝播する。また `reason-product-dogfood` は、正確なidentityを持つケース単位のcheckpoint/resumeをサポートする。中断された運用失敗はsemantic scoringの外に維持する。semanticの後継候補は `993874fa0051d06a02c8db8f7a220a2ac7773c17` のままである。
5. **外部CLIの堅牢化（#90）と実ワークロードUX（#139）：** closeout完了。サポート対象4リリースプラットフォームすべてでプロセスレベルの互換性を固定し、現在のlive semantic/runtime smokeはgreen。後継のMinistral 8Bプロダクト再実行では、Harness target coverageが0.25から1.00に回復し、unsupported grounded claimsは0、missed target insufficiencyも0になった。

## v0.3.0 — 外部エビデンスと解決

Tracking: milestone **v0.3.0 — External Evidence & Resolution**, parent Issue #173。

v0.3.0はv0.2.0後に完了した、プロダクト機能のマイルストーンである。制御ループはすでにcoreに実装されており、このマイルストーンでは、ドメインのretrievalやtrustをcoreへ移さずに、実際の外部取得とhard-verification adapterへ接続する。

必須の実行経路は次のとおり。

```text
unknown / 支持不足
  -> 型付き ResolutionRequest
  -> 外部取得adapter
  -> AcquiredEvidence
  -> EvidenceAdmissionPolicy
  -> 任意の TrustedResolutionVerifier / trusted verifier
  -> 通常の再検証 + 診断 + 判定
  -> grounded | qualified | unknown
```

完了順序：

1. **#174 external resolver adapter + CLI/config wiring — 実装済み。** `external_command_v1` は既存の `ResolutionResolver` 境界、閉じたstdio JSON protocol、literal argv、fail-closedの外部エビデンス受け入れを使う。trusted metadata、receipt、verdict、最終 prose は返せない。
2. **#175 external evidence qualification — 実装済み。** source identity、観測/取得時刻、scope、claimed authorityを正規化する。正確なsource allowlistとHarness所有のrank/max-age/scope policyが受け入れを決める。resolverのauthority claimによる自己昇格はできず、拒否理由は型付きtelemetryとなる。受け入れたエビデンスは通常のpipelineで再qualification/再検証される。
3. **#178 operational hardening — 実装済み。** 外部呼び出しを試行回数、時間、応答サイズで制限し、型付き失敗をoperational terminalとして維持する。resolution telemetryには実際の呼び出し、latency、任意のtoken/cost data、adapter/admission config identityのhashを記録し、ReasoningThread replayはツールを再実行せず記録を保持する。
4. **#176 read-only MCP resolver adapter — 実装済み。** `mcp_readonly_v1` は `ResolutionResolver` を通じ、明示的にallowlistされたread-only MCP toolだけを呼ぶ。一般的なtool outputはopaqueなまま、任意のacquisition envelopeもuntrusted dataのままで、受け入れられた事実は通常の検証に戻される。
5. **#177 reference trusted verifier/oracle — 実装済み。** `trusted_command_verifier_v1` は取得を分離し、外部oracleから conclusion + evidence IDs のみを受け付け、正確なauthority-bearing receiptをHarness内部で構築する。
6. **#179 open-world dogfood and release acceptance — 実装済み/合格。** `external-resolution-acceptance-v1` は safe recovery、stale/scope/irrelevant/conflict/operational/budgetのケースをCIで対象とし、AWS public-informationのlive smokeを別途記録する。
7. **#180 optional full-runtime MCP product surface — 実装済み。** `reason-mcp` は `reason_ask`、`reason_run`、`reason_verify`、`reason_schema` を、サポート済みネイティブruntimeの閉じた薄いwrapperとして公開する。native product JSONは変更せず返し、MCP invocationのscopeはその呼び出しに明示的に限定される。これはv0.3.0にとってnon-blockingのままである。

### v0.3.0受け入れゲート

- 最初はunsupportedだった実ワークロードtargetを少なくとも1つ、通常の制約付きresolution経路を通じて、実際の外部sourceから回復できること。
- resolver/tool outputが `Supported`、trusted metadata、verification receipt、verdict、grounded final proseを直接生成できないこと。
- provenance/freshness/scope/authority要件が機械的に観測でき、fail closedすること。
- acquisition successとtrusted verification successを別々に測定すること。
- operational tool/provider failure、policy denial、timeout、budget exhaustionは、semantic evidenceではなくoperational stateであり続けること。
- 受け入れたevidenceまたはcandidate revisionの各stepが、通常のvalidation、verification、diagnostics、decision、finalizationへ戻ること。
- 宣言されたv0.3.0 acceptance setが unsupported grounded claims = `0` と missed target insufficiency = `0` を維持すること。
- 過去のStage-C/RSD2など、観測済みのresearch holdoutは変更せず、product-tuning surfaceにも使わないこと。

MCPには意図的に分離された2つの役割がある。#176はReasoning Harnessがallowlist済みMCP toolをresolverとして呼ぶもの、#180は外部MCP clientがReasoning Harnessの全runtimeを呼ぶものだ。どちらも正確性の境界ではなく、MCP呼び出しが成功しても呼び出し側のagent loop全体をcertifyすることはない。

v0.3.0はproduct/distribution coordinateであり、新しいsemantic research generationではない。別の測定済みgapが、下記のresearch-to-product promotion gateを通過しない限り、現在のsemantic/runtimeとanswer-safety identityは変わらない。

現在のanswer-safety behaviorとsemantic runtimeにはrollbackと再現性のための正確なmachine configuration IDがあるが、それらはproduct phase名ではない。[Terminology and naming](terminology.ja.md)を参照。

## 過去の研究の来歴

初期の作業では `NL-1`–`NL-5`、`D1`–`D3`、`RSD0`–`RSD4` のようなissue単位のlabelを使った。research recordを追跡する際には今も有用だが、**プロジェクト全体のversion sequenceではなく**、新しいactive product phaseの命名にも使わない。

完了したsequenceで確立したもの：

- 同じverification/finalization boundary上の自然言語product path（#107/#109–#113）；
- 独立してcalibrateされたsemantic runtimeと保守的なrollback（#73/#84/#85）；
- authorityを生成できない残余evidence-sufficiency classifier（#91/#116/#118/#121/#125）；
- 明示的rollbackを備えた現在のclaim-local answer-safety configuration（#129/#134）；
- target-aware/shared-render product dogfoodとexposed-text review（#113/#131/#133/#137）。

正確な過去のphase label、凍結されたrun identity、machine configuration IDは、provenanceを書き換えないようresearch/evidence documentに残す。

## 現在のベースライン

すでに利用可能：

- natural-language-first pathに加え、サポート対象の `run`、`verify`、`semantic-check`、`schema` product command、制約付き外部resolution、任意の `reason-mcp` adapterを備えたexternal-preview `reason` v0.3.0 executable。research/evaluation commandは分離されている。
- provider-neutral core runtimeと型付き `ReasoningArtifact`；
- correctness authority boundaryの外部にある、Mistral、Google、NVIDIA用provider adapter；
- 制約付きresolution/finalization、evidence qualification、policy、checkpoint/replay、型付きdiagnostics；
- 現在のsemantic runtimeと、明示的にcharacterizeされたrollback profile（正確なmachine IDは安定して文書化済み）；
- credential-free deterministic CIと、分離されたlive provider smoke/research workflow。

v0.1.0は、外部から利用できる最初のstructured previewだった。v0.2.0ではnatural-language-first path、successor verified-target recovery、provider retry/resume reliability、process-level compatibility testを追加した。v0.3.0は現在のexternal-preview product releaseであり、同じresearch/authority provenanceを維持しながら、external acquisition/admission、operational hardening、read-only MCP acquisition、trusted deterministic verification、release acceptance、任意の `reason-mcp` product surfaceを追加する。versioned machine contractとサポート対象product commandはv0.x support policyのもとでcompatibility-trackedされるが、これはv1.0のstability promiseではない。

## 過去のマイルストーン：サポート対象コマンドとデータ契約

Tracking: Issue #90。

最初のproduct milestoneは、既存CLIを人間、shell pipeline、CIにとって予測可能にするものだった。

- [implemented #90] `run`、`verify`、`schema`を、research-only/evaluation commandとは分けてsupported product commandとして定義；
- [implemented #90] サポート対象JSON inputについて `-` stdinとfile/stdout behaviorを安定化し、1 invocationあたりのstdin consumerを最大1つに限定；
- [implemented #90] `reason-cli-output-v1` と `reasoning-artifact-v1` / `reasoning-candidate-v1` のmachine-readable contract identityおよびschema discoveryを定義；
- [implemented #90] exit-code semanticsを文書化。成功した `accept | reject | unknown` 実行はexit 0、command/runtime/validation failureはexit 1、CLI parse failureはexit 2；
- [implemented #93] semantic runtimeを、canonical machine identity、明示的rollback、semantic/final-verdict authorityの外部に置かれた型付きoperational failureを備える、分離された `reason semantic-check` product commandとして公開；
- [implemented #100] `run`/`verify`および既存 `semantic-check` failure surfaceのmachine-readable product failureを正規化。JSON automationではinput/config/harness/provider failure classとepistemic outcomeを分離；
- [implemented #94] schema-backed `reason-config-v1` は、明示的CLI flag > 明示的config > current-project config > user config > defaultの順にlayer化。`--no-config`でhermetic runを可能にし、unknown fieldはfail closed、provider secretはデフォルトでenvironmentが所有；
- `--format json`をautomationに適したものにし、人間向けoutputは明示的にnon-authoritativeとする；
- 短いinstall/quickstart pathと、copy-paste可能なshell/CI exampleを追加。

CLIは、core validation、verification、acceptance、finalization invariantをskipするflagを決して公開してはならない。

## 過去のマイルストーン：インストール、リリース、互換性

`reason`を簡単に入手でき、安全にupgradeできるようにする。

- [implemented #97] 再現可能な `cargo install --git` pathと、サポート対象の `reason` binaryだけを含むtag-driven standalone GitHub Release artifact；
- [implemented #97] release tagがCLI semverと一致することを必須化し、releaseにSHA-256 checksumを含める；
- [implemented #97] Linux x64、macOS arm64、macOS Intel、Windows x64を対象とするcredential-free product smoke；
- [implemented #90] `reason-cli-output-v1`、supported stdin behavior、schema contract ID、exit 0の `unknown`、exit 1のtyped operational failure、exit 2のCLI usage failureを固定するcross-platform process-level compatibility test；
- [implemented #102] v0.xで意図的なbreaking changeを行う際のchangelog/migration discipline；
- [implemented #102] provider operationとprovider-neutral correctness boundaryを分離する、明示的なproduct/platform/provider support policy。

package splitは必須ではない。実際の外部consumerが独立したversioningまたはdependency boundaryを作るまでは、現在のCargo workspaceをdefaultとする。

## インテグレーションと可観測性

CLIは引き続き最初のcompatibility surfaceである。自然言語AI pathは完全なnative runtimeを呼び出し、structured JSON commandはautomation、debugging、third-party integration向けのadvanced compatibility surfaceとして残る。どちらのpathもlower-level bypass APIを発明してはならない。

Product telemetryは、model confidenceをcorrectness authorityに変えることなく、operatorにとってHarnessを有用にするべきである。#126のv0.2.0 provider-reliability workは完了した。v0.3.0では#178により、同じ運用上の規律をexternal resolver/toolにも拡張する。呼び出しとretryはbounded、typed、observableであり、semantic `unknown`やabstentionへ変換されない。

- runtime/profile/config identity；
- `accept | reject | unknown` とabstention/unknown reason；
- grounded final-claim coverageとunsafe-final-answer counter；
- 測定可能なdeterministic gate interventionと、阻止したunsafe assertion；
- provider/protocol/quota/rate-limit/timeout failure class；
- attempt、retry、token、latency；
- semantic outcomeとoperational completenessの明示的分離。

v0.3.0では、evidence admission、trusted verification、必須のre-verificationを維持する場合に限り、reference external resolver/oracle integrationを実装する。MCPはcorrectness boundaryではなくadapterであり、read-only resolver roleは#176、任意のfull-runtime product surfaceは#180で追跡する。real consumer pressureによってその境界の必要性が検証されるまでは、public embedding compatibilityは保留する。

## 実ワークロードの採用エビデンス

Product readinessには、凍結されたresearch holdoutではないworkloadが必要である。自然言語のacceptance disciplineでは、**raw model vs current Harness baseline vs current answer-safety gateを備えた同じHarness**の3群比較を行う。2026-09-04の6ケースincident-analysis + architecture-review product workloadに対するsuccessor revalidation（Actions run `33822567155`、main `5c5701f77df9dd507c3949294708f8c07a054064`）で#139をcloseした。Ministral 8B raw target coverageは0.25のままだったが、両Harness armはtarget coverage 1.00、false target abstention 0、unsupported grounded claims 0、missed target insufficiency 0に到達した。Expected-unknown caseは安全に未解決のままだった。人間が使う `reason` pathは、model proseをauthorityに昇格させず、unresolved/verification-required stateに対して決定的なevidence-insufficiency guidanceを提供する。

#147のproduct-evaluation generationはcloseして凍結された。Stage Bは変更なしの24-case matrixで完了し、Stage Cはselection後にのみ作成した、SHA-256で別途凍結した16-case holdoutを使った。最終Stage-C semantic panelのtarget coverageはMinistral 8B、Mistral Small、Gemma 4 31B、Gemini 3.1 Flash-Liteで `1.00`、Ministral 14Bでは再現可能に `0.875` だった。完了したすべてのStage-C runでunsupported grounded claims = `0`、missed target insufficiency = `0`を維持した。14Bのmissは保守的なutility failureであり、unsafe exposureではない。Gemini 3.5 Flash-Liteは、semantic failureではなく、事前宣言したStage-B replicationが運用上quota-incompleteだったためStage Cの対象外となった。

現在のsemantic generationはcandidate `1f27bef9e5e7d1b8d2e95c4e4245c8fe8e77b352`で凍結されている。current `main`にはsemantic runtime/gate/holdout behaviorを変えないprovider-transport reliability changeが含まれる可能性がある。#150はverified-utility-recovery milestoneとしてcloseした。successor semantic workは#159、#160、#164に意図的に分割され、観測済みStage-C holdoutをtuning surfaceとして再利用せず、新しいruntime/evaluation identityを持たなければならない。

#159のsemantic behavior changeはcommit `79ec3b44971c32f9a8847d8173672675947c7288` で別個のsuccessor candidateを開始した。このidentityが記録するのは、Harnessが所有するbounded-target priorityだけである。凍結された `1f27bef9e5e7d1b8d2e95c4e4245c8fe8e77b352` のStage-C candidateを置き換えたり再解釈したりせず、観測済みStage-C holdoutもtuning surfaceとして再実行しない。#160または#164の後続semantic changeは、fresh evaluationの前にそれぞれ固有のsuccessor identityを受け取る。

#160のrenderer-downgrade changeにより、successor candidateは `a020b5925497ff3fdf200a9622270fa1889a6aa1` に進んだ。これは正確なHarness-owned target identityと既存artifact authorityだけを再利用する。rendererの `uncertain` modeはtriggerであってevidenceではない。独自のrecovery helperは `Reject` を上書きせず、`Unknown`はtarget-only qualified resultのままである。

#164のdependency-aware target-local changeにより、successor candidateは `993874fa0051d06a02c8db8f7a220a2ac7773c17` に進んだ。global decisionを緩めず、`Reject` scopeのqualified laneを別に追加する。exact targetにはdirect evidence-bound trusted `Supported` receipt、contradicted blockerには自身のevidence-bound trusted contradiction receiptが必要である。同じkey、型なし、shared evidence、target-local qualification/adversarial/contradiction、inference/dependency couplingはすべてfail closedする。凍結済みStage-C corpus/resultは変更せず、tuningのため再実行していない。

Issue #126は別のsemantic candidateを作らない。これは `993874fa0051d06a02c8db8f7a220a2ac7773c17` の周囲のprovider/evaluation control planeをhardeningする。Googleの一時429はboundedかつquota-awareのまま、500/502/503/504とisolated empty model textには狭く上限付きのretryを行う。実際のadapter attemptはobservableであり、16-case product dogfood/Stage-C runnerは、exact fixture/provider/seed/config/runtime/executable identityのもとで、完了済みcaseのexact prefixだけをresumeできる。実行中に中断されたcaseは最初から再開し、以前のoperational failureも記録に残す。過去のRSD2/Stage-C outcomeは書き換えない。

別個のdogfood/reference workloadを使い、次に答える：

- Harnessは現実的な利用でunsupported final assertionを減らすか。
- 正しくabstainする頻度と、不要にabstainする頻度はどの程度か。
- bounded resolutionで、当初unsupportedだった回答をverifiedへ変換できる頻度はどの程度か。
- 実運用で繰り返し現れるmissing-support patternは何か。
- safety processのlatency/token/retry costはどれだけか。
- `unknown`、abstention、failure telemetryをユーザーは理解し行動に移せるか。

実ワークロードの失敗から**新しいcalibration corpus**を作ることはできるが、観測済みの凍結holdoutを修復またはretuneするために使ってはならない。

実ワークロードのevidenceは、interactive session surfaceをproductizeする価値があるかどうかも決める。汎用agent CLIとのparityだけを理由にchat風REPLを追加してはならない。まず、実ユーザーが繰り返しevidenceを追加する必要があるか、`unknown` resultを見直すか、Harnessがabstainした理由を調べるか、複数commandにわたって同じreasoning stateを継続するかを観測する。その需要を測定できたら、既存runtimeと `ReasoningThread` checkpoint/replay modelの上に薄い `reason shell` / `reason repl` layerを設計する。interactive turnも同じauthority boundaryを維持しなければならない。conversation historyはtrusted evidenceではなく、過去のmodel outputはself-promoteできず、policy/evidence changeはre-validationを発生させ、assertive resultはすべて通常のHarness-owned verification/finalization pathを通る。

## v1.0対応準備ゲート

次のすべてを満たすまで、CLIをstable/v1.0として提示しない。

1. supported command、JSON、exit-code、configuration contractがcompatibility-testされていること。
2. install/release/upgrade flowが再現可能で、文書化されていること。
3. deterministic CIとbounded live runtime smoke gateがgreenであること。
4. 少なくとも2種類の異なるreal workload classにproduct acceptance evidenceがあること。
5. runtime identity、rollback、typed failure、operational-completeness semanticsが文書化され、テストされていること。
6. research/eval commandとsupported product surfaceが明確に区別されていること。
7. breaking-change policyとsecurity/secret-handling guidanceが明示されていること。
8. natural-language AI pathが同じverification/finalization authority boundaryを維持し、raw-model baselineとのproduct acceptance evidenceを持つこと。

readiness evidenceの基準commitは `5c5701f77df9dd507c3949294708f8c07a054064` である。8条件すべてに記録済みのevidenceがあり、process-level compatibility contractはPR #170としてActions run `33822514022` とfour-platform run `33822514005`でgreen、再現可能なrelease/install pathはv0.1.0 release workflowに記録され、current bounded live runtime/product smokeはrun `33822794171` と `33822567155`でgreenである。incident-analysisとarchitecture-reviewの両方にproduct acceptance evidenceがあり、runtime/rollback/failure/secret-handlingおよびresearch surfaceの境界も文書化・テスト済みである。これはcurrent main lineの **readiness gate** 完了を示すが、stable v1.0を公開・tag付け・保証するものではなく、通常のprovenance workflowによる明示的なversion/release decisionは別途必要である。

## 研究からプロダクトへの昇格ゲート

research trackはproduct trackより速く進めてよいが、新しいreasoning mechanismはcalibration metricが改善しただけではstable CLIに入らない。

```text
fresh calibration-only hypothesis
  -> pre-observation spec/label review
  -> calibrated candidate
  -> fresh independently frozen holdout
  -> operational stabilization + typed failures
  -> explicit runtime profile + rollback
  -> CLI compatibility/observability coverage
  -> reversible product adoption
```

現在のsemantic runtimeと完了した #91 residual evidence-sufficiency programは、別々のmachine identityとrollback boundaryを持つ。answer-safety configurationはsemantic runtimeとは独立にversion管理され、どちらもverification authorityを生成しない。frozen holdout-v4/v5とsufficiency holdoutは不変のresearch historyであり、product tuning corpusには決して使わない。

## 保留中のプロダクトサーフェス

- **Public Rust embedding API:** 実際のCLI consumerが適切なcompatibility boundaryを検証した後に検討する。
- **MCP full-runtime product surface (#180):** optionalな `reason-mcp` downstream integrationとして実装済み。tool resultの成功はそのnative Harness invocationにだけ適用され、callerのagent loop全体が検証済みであることのevidenceにはならない。read-only MCP resolver roleは #176 の `mcp_readonly_v1` として別途実装済みである。
- **Interactive CLI (`reason shell` / `reason repl`):** repeated real-workload dogfoodで需要が確認された後に判断する。採用時も `ReasoningThread`/checkpoint/replayと同じproduct runtime上の薄いstateful sessionとし、別のchat authorityやevidence shortcutにはしない。
- **Desktop UI:** artifactとCLI contractが安定した後に限り、薄いinspection/review clientとして検討する。

参照：[ADR-0001](adr/0001-interface-and-packaging-boundaries.ja.md)、[roadmap](roadmap.ja.md)、[research plan](research-plan.ja.md)。
