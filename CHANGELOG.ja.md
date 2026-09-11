# 変更履歴

`reason` CLI に関する、製品向けの注目すべき変更を記録する。research-only binary と fixture study の変更は research note に記載する。実行可能ファイルは semantic versioning に従うが、v0.x では product interface を引き続き hardening 中である。machine-readable contract identity は executable version より厳密な compatibility boundary である。

## [Unreleased]

## [0.4.2] - 2026-09-11

v0.4.2はinvestigation utilityとprovider parityのexternal-preview patch release。v0.4.xのcorrectness/authority boundaryを維持したままstochastic investigation control pathをhardeningし、freeze済みmetric-v13 acceptanceでMistral / Groq / Gemini / Gemmaの最終candidateを検証した。

### 追加

- Issue #261: 1つのexact investigation targetについて、明示read-only capability priorityから実行可能なunique highest-priority choiceが機械的に決まる場合だけ、Harness-owned deterministic acquisition precedenceを適用。同じkeyのsibling、tie、priority欠落、wildcard/keyless target、attempt済pair、non-read-only action、terminal budgetは暗黙repairせずfail-closed/model pathに残す。
- Issue #262: 既存Groq adapterをgeneric natural-language `reason --provider groq`のgeneration/planning/action/regeneration/render/sessionへ接続。provider固有のcorrectness/authority semanticsは追加しない。
- diagnostic-only structured generation trace、typed action rejection/precedence telemetry、operational observability bounds、incomplete control向けconservative paired acceptance。

### 変更

- investigation plan/action schemaを構造的に制約し、exact fact-key/target/capability bindingを維持、attempt済pairを除外、selection priorityをmodel-invisibleに保ち、実行ID欠落をrepairせずfail closed。
- structured provider outputはboundedなJSON-schema -> JSON-object -> strict-text JSON compatibility pathへdegradeしつつterminal metadataとfail-closed parsingを維持。factual render schemaでは明示`factual_claims`を必須化。
- typed `no_result`後に合法なexact-target continuationがある場合、round boundaryでもbounded continuationを維持。historical frozen observationは再採点しない。
- Google provider operationはbounded transient retry、structured quota-window classification、shared request pacer、canonical 6000ms request-start floor、inter-case delayとの独立policyを使用。

### Release acceptance

- 最終immutable v36: `natural-language-e2e-v36-freeze` / `57bea659d472a103cc48d86ddee7dfe4a41de790`、candidate `9497b563ad914fada13d33e0c1a7fee549a1f1de`、released v0.4.1 control `29a9e4be6273dbffeda324e15517dc64930ad315`、seed `738214`、metric `v13`。canonical rerun / post-freeze mutationはいずれも`0`。
- Mistral paired Actions `34564120392`: PASS。
- Cross-model Actions `34564672351`: Groq PASS、Gemini 3.5 Flash-Lite paired PASS、Gemma 4 31B paired PASS。全required candidate rowは13/13完走し、operational/generation/correctness-boundary failureは`0`。cross-model averagingは使用していない。
- Geminiのfrozen paired follow-up utilityはcontrolのtool selection `0.6`、trigger exposure `0`、avoidable stall `3`からcandidate `1.0`、`3`、`0`へ改善し、target recall `1.0`とzero correctness gateを維持。
- Google 6000ms pacingにより最終acceptanceでv34のfree-tier RPM failureは再発せず、v35 Gemmaの`pacing == inter_case_delay` eval-runner invariant failureも再発しなかった。
- 詳細: [Natural-language E2E v36 canonical release acceptance](docs/natural-language-e2e-v36-result.ja.md)。

## [0.4.1] - 2026-09-07

v0.4.1はexternal-previewのpatch release。freeze済みv9で観測したbounded investigationのutility residualだけを狭くhardeningし、target identity、authority、admission、verification、finalization、answer safety、machine contract semantics、freeze済みnatural-language E2E v1〜v9 evidenceは変更しない。

### 変更

- Issue #249: typed `no_result` の直後、同じexact targetに対して`expected_fact_key`を明示対応する未試行read-only capabilityがちょうど1つだけ残る場合に限り、bounded investigationがdeterministicに継続できる。同じfact keyの別targetは分離したままで、keyless、wildcard-only、複数候補、`no_result`以外、terminal stateではこの継続を使わない。audit用に`harness_no_result_followup_selections`を追加し、admission、authority、verification、finalization、answer safety、freeze済みnatural-language E2E v1〜v9は変更しない。

## [0.4.0] - 2026-09-06

第4のexternal-preview capability release。v0.4.0では公開する事実テキストをHarness authorityへbindし、bounded investigation、resumable session、subprocess/MCP hardening、自然文E2E validationを追加した。frozen research generationは変更せず、model/tool outputがcorrectness authorityを自己付与できない境界も維持する。

### 変更

- Issue #233: bounded investigationで、expected fact keyとread-only capabilityの明示supported keyが一致する未試行pairが1組だけの場合、Harnessがdeterministicに選択する。複数候補/key不明/wildcardはmodel selectorのまま。authority/admission/verificationは不変で、`harness_unique_selections` telemetryを追加。historical `natural-language-e2e-v5` utility値はtuning surfaceとして再利用せず、そのまま保持する。
- Issue #232: release artifact actionsと`sha2`を0.11へ更新。frozen v12 adoption checksum fileはbyte-for-byte不変のまま、CIでfreeze時Cargo.lock provenanceとcurrent build dependencyを分離し、dependency maintenanceがhistorical semantic source identityを書き換えないようにした。
- Issue #204を`mcp_readonly_v3`で完了。#211共通deadline内でpersistent stdio `initialize`/negotiation/`initialized`/`tools/list`/`tools/call`をboundedに実行し、protocol allowlist fail-closed、選択toolのserver `readOnlyHint`強制、typed negotiation/session failure、negotiation policyと実revisionをbindするconfig/replay provenanceを追加した。generic output非昇格は不変。frozen `mcp_readonly_v1`は変更せず、v2はdeadline-only historical successorとして保持する。pinned公式GitHub MCP server probeもv3で成功しgeneric resultはopaqueのまま。
- Issue #214をfreeze済み`natural-language-e2e-v5`で完了。canonical Actions `34032191037`は10/10ケース完走、operational failure 0、correctness-boundary violation 0。unsupported exposed assertion / unsupported structured claim / missed insufficiency / session external replayはいずれも0、identity/freshness/scope/authority rejectionは4/4。utility残差（target recall 2/7、tool selection 5/7、false abstention 3）は観測後にtuningせず保持し、v1-v4はimmutable diagnosticとして残す。
- 自然文JSON outputを`reason-natural-output-v3`から`reason-natural-output-v4`へ更新し、post-investigationの最終Harness artifact/verdictを`final_outcome`として明示。公開finalization、session checkpoint、評価が同じ最終stateを参照するようにし、`resolution_rounds[-1]`がcandidate再生成前のacquisition stateだった曖昧さを解消した。
- Issue #214の観測前`natural-language-e2e-v1`評価面を追加。明示hypothesisなしbounded investigation、identity/freshness/scope/authority rejection、multi-turn session add/correction/resume/fork、exposed-textとstructured claimを分離した安全採点、network非依存deterministic acquisition fixture、correctness violation 0のfreeze済みadoption gateを含む。
- #214のv1初回live attemptがevaluator `NameError`でnon-scorableとなったため、v1をimmutableに保持したまま`natural-language-e2e-v2` successorをfreeze。10ケースとprovider policyは維持し、successor evaluator pathのみ修正して、live前にscoring実行を直接検証するregression coverageを追加した。
- Issue #210のexposed-text correctness gapを解消。`GroundedAnswer` / `QualifiedPartialAnswer`ではmodel rendererの`text`を公開せず、`harness-canonical-exposed-text-v1`のもとでaccepted factual claimまたはtyped recovery stateからHarnessが表示文章を構築する。
- 自然文JSON outputを`reason-natural-output-v2`から`reason-natural-output-v3`へ更新し、`exposed_text` policy telemetryを追加。renderer proseに依存していたconsumerは`finalization.text`を使用する。旧v2挙動はcommit `3a601c8`でhistorical reproduction可能だが、P0 gapを再導入するためsafety rollbackとしては提供しない。
- Issue #211のsubprocess timeout gapを解消。`external_command`、`trusted_command`、v0.4 product向け`mcp_readonly_v2`で共通のabsolute wall-clock deadlineを使用し、spawn、stdin全量write、stdout read、process termination、non-blocking cleanup handoffまでを覆う。historical `mcp_readonly_v1`はfreeze contractどおり不変。大きいblocked writeやdescendantの継承pipeでcallerが設定timeoutを超えて待たされず、oversized stdoutもtimeoutへ誤分類せずtyped protocol failureとして維持する。
- Issue #212 の bounded investigation planning を自然文 path に追加。`bounded-investigation-v1` の下で closed plan/action schema、Harness が受け入れる untrusted target、設定済み read-only capability 選択、typed follow-up / no-progress stop、admitted evidence 後の candidate regeneration、通常の qualification / verification への強制復帰を実装した。static resolver lane は変更せず、investigation external command は `external_command_v1` を拡張せず `investigation_external_command_v1` / `reason-investigation-external-resolver-request-v1` を使う。
- Issue #213 の resumable session を `reason-session-v1` として追加。`reason session start|inspect|resume|add|correct|fork|close`、typed `ReasoningThread` checkpoint上のatomic local persistence、typed input-change invalidation、provider/model/safety/config identity保存、revalidation中のstale finalization抑止、non-destructive fork lineage、inspect/resume/fork時のexternal acquisition replay 0件を実装した。continuation turnは`session-replay-only-acquisition-v1`を使い、start時resolver/MCP/investigation設定を暗黙replayしない。

## [0.3.0] - 2026-09-04

第3の external-preview capability release。v0.3.0 は research generation、semantic runtime、answer-safety identity を変更せず、bounded external evidence と resolution を追加した。

### 追加

- Harness-owned の source、freshness、scope、authority policy を fail-closed で扱う `external_command_v1` と `external_evidence_admission_v1`。
- external-resolution budget、typed operational failure、telemetry、replay-safe record。
- read-only `mcp_readonly_v1`、別系統の `trusted_command_verifier_v1`、任意の Rust-only `reason-mcp` native-runtime delegation。
- `external-resolution-acceptance-v1`。unsupported grounded claims と missed target insufficiency は `0`、safe recovery は2件、live AWS RSS の別系統 smoke は `Unknown -> Accept` だった。

### 維持したもの

- Frozen Stage-C/RSD2/historical holdout は変更しない。
- Semantic runtime は `semantic-decidability-d3-v1`、answer safety は `verified-target-answer-gate-v1` のまま。MCP は correctness boundary の外側である。

## [0.2.0] - 2026-09-04

native Reasoning Harness CLI の第2の external-preview release。既存の research/authority foundation 上の **product capability release** であり、frozen Stage-C/RSD2 evidence の rewrite ではない。

### 自然言語AI CLI

- 現行 `reason-natural-output-v2` JSON identity を使う直接の `reason "TASK"` AI execution を追加し、v0.1 structured product command を維持。
- provenance-aware `--file` と piped-stdin の untrusted context、bounded input size を追加。
- `--fact`、`--hypothesis`、bounded `--resolver-fact` を追加。ただし arbitrary prose/model output は trusted evidence に self-promote できない。
- final-claim coverage の後段に model-backed final rendering を追加し、renderer が exact authorized target を省略/弱化した場合の deterministic recovery を追加。
- structurally isolated な verified target の target-local qualified recovery を追加し、global `Reject`/`Unknown` と authority check を維持。

### プロダクト評価と信頼性

- incident-analysis と architecture-review workload で raw / Harness baseline / current-safety を比較する `reason-product-dogfood`。
- Google/Gemini の一時的 429、HTTP 500/502/503/504、isolated empty-model-text anomaly の bounded retry。credential、quota、deterministic 4xx/protocol、transport、timeout は fail-fast のまま。
- adapter と structured-output fallback call の provider HTTP-attempt telemetry。
- `reason-product-dogfood-v10` の exact-identity checkpoint/resume。完全完了 case だけ再利用し、active case は先頭から再開、provider/protocol failure は semantic abstention ではなく operational evidence として保持。
- Ministral 8B の6-case product revalidation で Harness target coverage は historical 0.25 slice から 1.00 へ改善。unsupported grounded claims と missed target insufficiency はともに zero。

### CLI互換性と配布

- 実際の `reason` binary を実行し、`reason-cli-output-v1`、schema ID、stdin behavior、epistemic `unknown` の exit 0、typed operational failure の exit 1、CLI usage failure の exit 2 を固定する process-level compatibility test。
- Linux x86_64、macOS arm64、macOS x86_64、Windows x86_64 で compatibility contract を実行。
- documented v1.0 readiness gate が current main で満たされても、v0.x は external-preview として明示。release automation は 0.x GitHub Release を自動で prerelease 化。

### 研究と権威情報の来歴

- frozen Stage-C candidate/holdout と historical RSD2 outcome を変更せず、prior provider failure を semantic success と解釈しない。
- successor semantic candidate は `993874fa0051d06a02c8db8f7a220a2ac7773c17`、semantic runtime は `semantic-decidability-d3-v1`、answer-safety は `verified-target-answer-gate-v1`。
- model output、retrieval prose、retry success、checkpoint reuse は verification authority の外側である。

## [0.1.0] - 2026-09-01

native Reasoning Harness CLI の最初の external preview。

### プロダクトCLI

- `reason run`、`reason verify`、`reason semantic-check`、`reason schema` を supported product surface として追加。`eval*` は research/evaluation のまま。
- non-interactive JSON input 用の stdin (`-`) と one-consumer protection。
- `reason-cli-output-v1`、`reasoning-artifact-v1`、`reasoning-candidate-v1`、`reason-config-v1`、`semantic-check-input-v1` の machine-readable contract identity/schema discovery。
- CLI flags > explicit config > project config > user config > defaults の layered non-secret config と `--no-config` hermetic execution。
- machine-readable product failure envelope と normalized provider/input/config/harness failure class。process failure は exit 1、`accept | reject | unknown` 成功結果は exit 0。

### セマンティックランタイム

- `reason semantic-check` から `semantic-decidability-d3-v1` runtime を公開。ただし soft diagnostic に final-verdict authority は与えない。
- `soft-semantic-v3` rollback selection を維持。
- semantic decision と分離した typed operational failure output。
- Ministral 8B と Google-hosted Gemma 4 31B の D3/v3 rollback live product smoke に成功。

### 配布

- Linux x86_64、macOS arm64、macOS x86_64、Windows x86_64 の credential-free product smoke。
- single supported `reason` binary の `cargo install --git` installation。
- tag-driven standalone GitHub Release archive と SHA-256 checksum。research binary は release artifact ではない。
