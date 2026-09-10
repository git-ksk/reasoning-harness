# Natural-language E2E v31 — Investigation Utility & Provider Parity acceptance

Issue #263 では、immutable v30 と、そこから独立に正当化して実装した product fix #331 / PR #332 の後継として fresh held-out v31 を使う。freeze 済み v1〜v30 evidence は immutable で、再実行・再採点・調整・書き換えは禁止する。release closeout までは Cargo workspace version を `0.4.1` のまま維持する。

## v31 を作る理由

v30 (`natural-language-e2e-v30-freeze`, commit `cf0cada8f4cf666f75b8dfb6c012a6ca63fb43a3`, seed `97362`) は immutable な VALID RELEASE FAIL である。Mistral paired と Groq candidate-only は PASS。Gemini paired は、released control の false abstentions が conservative bound `[6,8]`、candidate が `7` で非悪化を証明できなかったため、事前固定した v13 規則どおり `INCONCLUSIVE`。Gemma paired は candidate の `scope-ratube-index` に operational protocol failure が1件発生したため hard `FAIL` だった。correctness / safety violation はゼロのまま維持された。

保存済み Gemma trace では、primary structured response の先頭に完全な typed `{"action":"stop"}` object があり、その後ろに non-JSON output が続いていた。planner decoder はresponse全体をinvalidとして既存の JSON-object fallbackへ進み、fallbackが `{}` を返したため protocol terminal になった。candidate生成とfinal-renderは既に「完全なtyped JSON 1個 + non-JSON suffix」を受理していたため、investigation plan/action decoderだけ挙動が不整合だった。

#331 / PR #332 は、この不整合だけを provider-neutral に修正してからmainへmergeした。共通typed decoderは、**型検証に通る完全なJSON valueがちょうど1個あり、後続が明確なnon-JSON textだけ**の場合に限ってsuffixを無視する。一方、2個目のJSON value、JSON-likeな壊れた断片、truncation、missing discriminant、missing ID、unknown field、その他typed-data errorは従来どおりfail-closed。semantic fieldの推測・補完、semantic/generic protocol retry追加、provider/model固有correctness branch、#283 deterministic action materializationは導入していない。merged candidate coordinate は `6bde9227d56ba237cb305777adf452461a0df864`。

したがってv31は、別途正当化されたproduct changeを**新しいheld-out corpus**で評価する。v30のrerunではなく、v13のrulerも変更しない。corpus seedは `98473` に固定し、case identity / task / fact key / answer / source identity / fresh marker はlive provider使用前に固定し、frozen predecessorとのcollisionを機械的に確認する。

### Surface revision r2

最初のv31 freeze (`natural-language-e2e-v31-freeze` @ `4542eecc8a656eb2cd2f625f6cfbf4e72045ae78`) は、canonical armを1件もlaunchする前にevaluation-infrastructure packaging defectを露呈した。run `34442213199` はfreeze/precredential gateをすべて通過した後、workflowがpaired orchestrator v2のoptionを指定している一方、branchにはmain由来の旧v1 helperしか無かったため `argparse` で終了した。control/candidate attempt marker、report、acceptance report、orchestration recordは1つも生成されず、model observationも開始されていない。r1 tag/runはinfra defectのimmutable evidenceとして保存し、rerunしない。

live armが未観測なので、r2では事前固定済みseed `98473` とv31 corpusをそのまま維持する。r2の修正はevaluation infrastructureだけで、immutable v30に存在したprovider-neutral `paired-canonical-observation-v2` helper/testsを持ち込み、helper CLI surfaceの明示回帰を追加し、r2専用workflow label/artifact名と新tag `natural-language-e2e-v31-freeze-r2` を使う。product runtime、scoring、metric、prompt/case semantics、provider coordinate、retry policy、candidate coordinateは変更しない。

## Measurement lock

v31 は v11 の target/tool/finalization semantics、v12 の continuation-opportunity semantics、v13 の operational observability / conservative bounds semanticsをすべて変更せず維持する。`config/natural-language-e2e-metric-v13.json` が引き続きsource of truthで、bound identityは `natural-language-e2e-operational-bounds-v13`。policyの `first_allowed_successor` がv30のままなのは、v30がv13を最初にprospective適用したhistorical factだからである。

各scoring metricはcase単位で `observed` / `censored` / `not_applicable` に分類する。operational terminalより前に確定したpositive monotone witnessはobservedのまま残す。一方、terminal後に起こり得たイベントを「起きなかった」とは扱わない。censored caseはcomplete-case deletionせず、都合の良い値も代入せず、controlの論理的に許される区間を計算する。

higher-is-betterの非悪化はexact candidate valueがcontrol upper bound以上の場合だけ成立する。lower-is-betterはcandidateがcontrol lower bound以下の場合だけ成立する。strict improvementもcandidateに不利なcontrol endpointに対して証明する。証明できない比較は `INCONCLUSIVE` であり、`INCONCLUSIVE` ではreleaseしない。

candidateのoperational incompletenessは従来どおりhard `FAIL`。既存correctness / safety zero fieldsもすべてhard gateのまま。v12 continuation eligibilityがtrueのcandidateはmechanism conformance 1.0必須。#324 candidate-only diagnostic sidecarは引き続き `scoring_input=false` で、observability/boundsには使わない。

v31 metric-lock validatorはimmutable v30をpredecessorにする。runner scoring functionsとpair scrubが、v30/v31 identity・fresh seed・candidate coordinateの正規化以外でAST同一であることを証明し、その上で変更していないv13 policyとacceptance boundaryを検証する。live observation後のmetric/semantic rewriteは禁止する。

## Freshness / pairing

case familyはlogical coverageを維持する。13件 = investigation 10 + session 3、そのうち observational exact-target typed-`no_result` follow-up 3件、read-only GitHub MCP nonpromotion 1件。freshness testはv1〜v11 repository rootとimmutable v12〜v30 freeze refを対象にする。GitHub MCP caseはexact paired coordinateの `Cargo.toml` を読み、control/candidate config差分はcoordinate refと3組のfollow-upに対するcandidate-only `selection_priority` だけに限定する。

provider coordinateはlive前に固定する。Mistral `ministral-8b-latest`、Google `gemini-3.5-flash-lite`、Google `gemma-4-31b-it`、Groq `openai/gpt-oss-120b` candidate-only。`planner_max_tokens = 256` とprovider `max_tokens = 1024` は変更しない。Google Gemini / Gemmaはmatrix `max-parallel: 2` の別jobで実行するが、各paired row内部ではcontrol → candidateのcanonical順序を厳守する。

## Pre-live gate

provider credentialを使う前に、exact control/candidate coordinate、immutable v30 predecessor identity、candidateからの `Cargo.toml` / `Cargo.lock` / `crates` runtime diffなし、workspace version `0.4.1`、corpus/surface checksum、v13 metric lock、pair validator、validate-only、no-model/no-network preflight、exact CLI capability probe、full deterministic Python tests、full Rust workspace tests、fmt、clippy `-D warnings`、pinned GitHub MCP contract、workflow-policy tests、通常PR CIをすべて通す。

これらがgreenになるまで `natural-language-e2e-v31-freeze-r2` は作成しない。provider credentialを露出する前にfreeze tagとchecksumを確定させる。

## Canonical live 順序

1. immutable v31 r2 freezeでMistral paired canonicalを1回だけ実行する。
2. Mistral paired gateが `PASS` の場合だけ、同一freezeでcross-model workflowを開始する。
3. Gemini paired canonicalとGemma paired canonicalを各1回実行する。各rowはcontrolを先に、candidateを後に各1回だけ実行する。controlがnonzeroでもcanonical evidenceが保存されていればv13 acceptance comparatorに判定を委譲し、自動的な成功扱い・失敗扱いはしない。
4. Groq candidate-only canonicalを1回実行する。
5. raw canonical stdout、report、orchestration record、candidate diagnostic sidecarを保存する。paired orchestratorの45秒heartbeatはstderrのnon-scoring progressで、captured canonical stdoutは変更しない。
6. 必須model rowはそれぞれ独立判定し、cross-model averagingは禁止する。

v31が `FAIL` または `INCONCLUSIVE` ならimmutable evidenceとして確定し、rerun / rescore / tuneはしない。次のsuccessorには、別途正当化されたprospective product/evaluation changeが必要である。
