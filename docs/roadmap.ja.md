# Roadmap（ロードマップ）

## プロジェクトの方向性

Reasoning Harness は汎用モデル runner や第二の Inspect/lm-eval になろうとしているわけではない。その中核となる差別化要因は、provider-neutral かつ authority-aware な中間 reasoning の制御である。deterministic な構造と harness-owned evidence は hard finding を生成し得る一方、model-backed semantic discovery は独立して検証されるまで soft かつ観測的なものに留まる。

この diagnostic layer は基盤であり、最終的な product boundary ではない。

長期的な product の方向性は、確率的な candidate generation を取り巻く loop を担う **evidence-grounded reasoning runtime** である：

```text
候補を生成
  -> 根拠付け / 検証 / 診断
  -> 不足している根拠を解決、または反証された推論を修正
  -> 同じ authority boundary の下で再検証
  -> 十分に根拠付けされた proposition だけから最終化
```

runtime は `unknown`、qualified partial answer、または abstention で停止できなければならない。answerability の向上は、retrieved data、model repairs、流暢な final prose を、黙って correctness authority に昇格させることを決して必要としてはならない。

[ADR-0002](adr/0002-grounded-resolution-and-finalization.ja.md) を参照。

## 現在のプロダクトマイルストーン

現在のsplit external previewは **Reason CLI 0.5.3 / Harness Engine 0.5.0** である。`v0.4.2`はimmutableな最後のunified releaseかつhistorical Engine 0.4.2 baselineとして保持する。**v0.4.2 — Investigation Utility & Provider Parity（milestone #5 / #260）** は完了・release済み。#261 deterministic safe acquisition precedence、#262 generic Groq provider parity、#281以降のstructured planner/action hardening、v0.4.x authority/finalization boundaryを変えずmetric-v13 acceptanceを完了するprovider/eval resilienceを含む。

releaseは最終immutable v36までevidence-gatedを維持し、Mistral paired PASS、Groq generic candidate PASS、Gemini 3.5 Flash-Lite paired PASS、Gemma 4 31B paired PASS。Geminiでfrozen rowのstrict utility improvementを観測し、全required candidate rowでoperational / generation / correctness-boundary failure `0`を維持した。canonical rerun / post-freeze mutationは0。詳細は[v36 release acceptance](natural-language-e2e-v36-result.ja.md)。

最後のunified releaseである`v0.4.2`以降は、3本のlineを独立して進める。

1. **Reason CLI 0.5.x — General-use Productization**（milestone #6 / parent #359）は0.5.0〜0.5.2を **Harness Engine 0.4.2** でreleaseし、completed #455でrelease済みEngine 0.5.0を`reason-v0.5.3`へadoptした。残るのはP1 #375だけで、Homebrew 0.5.3 acceptanceは完了、WinGetはvalidation / CLA完了後のcommunity moderator approval待ち。phase / P0 / P1の詳細は[CLI 0.5.0製品化ロードマップ](reason-cli-0.5-roadmap.ja.md)を参照。
2. **Harness Engine 0.5.0 — Verified Investigation Utility**（milestone #4）はreleaseまで完了。#248/#247/#282/#283に加え、final hardening #445/#446/#450でfinalization scoring、explicit-fact correction continuity、admitted exact-fact investigation materializationに残っていたavoidable stochastic dependencyを除去した。`engine-0.5-final-v3-freeze` run `35457038163` は独立required 6 row・fresh 18/18 caseをPASSし、correctness-boundary violation 0、session external replay 0を維持した。詳細は[Engine 0.5.0 final-v3 acceptance](engine-0.5-final-v3-result.ja.md)。Engine source releaseは`engine-v0.5.0`で、`reason-v0.5.3`がこれをdistributionし、既存Reason CLI 0.5.2はEngine 0.4.2のままimmutable。

3. **Harness Engine 0.6.0 candidate — Target-local Evidence Semantics** は次に進めるsemantic/correctness research lineであり、まだrelease coordinateではない。release済みEngine 0.5.0を書き換えず、production gap #461/#462/#463から開始する。#461はacquisition前のtarget-local evidence-need routing、#462はacquisition後のevidence-target semantic relevance、#463はrelevant material admission後のsource-attributed qualified proseを担当する。原則の実装・評価順は #461 -> #462 -> #463 とし、依存関係の実測で必要な場合だけ変更する。#461はfrozen calibration v3（run `35957170730`）でcandidate semanticsをfreezeした後、新規authorしたindependent holdout v1（run `35965160995`）でもMistral/Google両armとも26/26 operational complete、materialized mode/acquisition 26/26、correctness violation 0、utility miss 0を達成し、Engine 0.6 line向けにindependent accepted。 #462は`feat/462-evidence-target-relevance`で進行中。provider-neutral relevance contract、binding proposal v2、target-first materialization policy v3、strict Harness-owned identity floorを実装済み。frozen v4 run `35998574508` はMistral 26/26 PASS、Google 23/26で3件timeout＋safe ambiguity 2件となりFAIL。後続Google diagnostic run `36000374933` では `gemini-3.5-flash-lite` が未解決5件を5/5 operational / materialized exactで完走し、3.1へのmodel変更は採用しない。frozen v5 run 36008648993 はMistral 26/26 PASSだった一方、Googleは11/26 operational、14件 assessment_timeout＋1件4-attempt後の明示的HTTP 503 high-demandでFAILした。完了11件はmaterialized 11/11 exact、correctness / utility miss 0のためsemantic regressionではなくprovider-capacity instabilityが主因と判断する。v5はrerunせずimmutable FAILとして保持する。6-case Google recovery smoke run 36018360038 は4/6 operationalで、21_unknown_rename と26_url_only_identityが再び60,000 ms assessment_timeoutとなった。完了4件はmaterialized 4/4 exact、semantic miss 0。したがってprovider recoveryは確認できず、19-case recovery diagnostic v2は実施せず、v6 canonicalより先にjitter・run/client-level retry/overload budget・cooldown/circuit・tail latency telemetry・diagnostic gate表示のoperational hardeningへ進む。60秒deadline、semantic contract、expected labelsは現時点では変更しない。frozen v6 run `36022978827` はhardening動作を確認できた一方canonical acceptanceはFAIL。Mistralは26/26 PASS、semantic gate 0、latency p50/p95/max 554/863/944 ms。Googleはcase 01を47,242 msで完了後、case 02/03が各60,001 ms assessment timeoutとなり、2件連続failure circuitが開いて残り23 requestを抑止した。完了Google semanticsはexact、correctness / utility miss 0を維持。v6はimmutable FAILとして保持し、bounded jitter、fail-fast load shedding、attempt-telemetry completeness、tail-latency reporting、operational incompleteを明示的にredにするgateは維持する。v7のrequired-provider gateはlive observation前にMistral `ministral-8b-latest`＋Groq `openai/gpt-oss-120b`へ事前固定する。選定根拠は#462 semantic outcomeではなく独立したprovider evidenceで、GPT-OSS 120Bは既存Engine 0.5 final-v2/final-v3 required/reference rowをPASSし、現在もGroq Production ModelかつJSON Schema対応。Google 3.5はrequired operational gateから外すがsemantic failureへ再分類しない。v4-v6はimmutable Google operational/replication evidenceとして保持し、将来のGoogle requalificationは別freeze studyで行う。fresh v7は26 case、expected labels、relevance semantics、60秒case deadline、operational failure 2件連続circuitを変更しない。 frozen Google attempt-telemetry diagnostic v1 run `36026307543` では同じearly 3 Google request shapeが全てfirst attempt・HTTP 200で3/3完了し、retry / cancellation / 429 `RESOURCE_EXHAUSTED` / 503 `UNAVAILABLE` は0だった。一方headers latencyは約28.4-35.9秒と高止まりしている。v6中のhidden transient quotaをretroactiveに完全否定するものではないが、v5の明示的503 high-demandと合わせるとquota exhaustionよりintermittent serving-tail / capacity variabilityの説明が強く、この3-case probeだけでGoogleをrequired 26-case canonical providerへ再認定はしない。frozen v7 run `36026148264` もimmutable FAIL/incompleteとしてrerunしない。Mistralは26/26 operational完了したが `21_unknown_rename` がexpected `ambiguous` に対し `irrelevant` へmaterializeされutility miss 1件、unsafe relevance admissionは0。Groqはworkflow cancel前に先頭3 caseのみ完了し、全てHTTP 200・1 attempt・materialized exactだったがnon-scorable。fresh successor前にcase21を一般的なidentity uncertainty / advisory-model authority設計問題として扱い、このcalibration caseへseed/prompt tuningせずindependently authored identity-ambiguity probeで検証する。 fresh identity ambiguity diagnostic v1 run `36029430165` はMistral/Groqとも36/36 operational完了。Mistral primaryはexpected-unresolvedを11/18 observationでfalse `different`へ寄せbaseline 25/36だったが、one-sided distinctness candidateはfalse confirmation 0・explicit-different miss 0・gated disposition 36/36。Groqもprimary false `different` 1件、baseline 35/36に対してcandidateは36/36。v7 case21は単発seed事故ではなくopen-world identity uncertaintyの構造的riskとして再現したため、fresh successorではnegative identity確定にaffirmative distinctness confirmationを要求する方向を設計する。 次のrequired-provider構成を決める前に、Google `gemini-3.5-flash-lite` は別freezeの26-case full requalification v1でnon-canonical / non-gatingに再測定する。全26 fixtureを明示指定し、60秒deadline・6秒pacing・2連続operational failure circuit・attempt telemetryを維持し、26/26 operational completionのみをprovider requalification gateとする。 frozen full requalification v1 run `36078211994` は26/26 operational、全26 callがfirst attempt・HTTP 200、429/503/retry/cancellation 0、latency p50/p95/max 695/843/950 msでPASSした。Googleはoperationally requalifiedとするが、v4-v6 historical instabilityを消さず、次semantic successorではMistral + Groqをrequired arm、Googleをfull 26-case non-gating replication armとする。Googleのrequired復帰はnew successor semantics上の独立実証とoperational stability継続後に別decisionとする。 次のfresh semantic successorはcalibration v8としてmaterialization v4を導入する。primary `target=different`だけでは`irrelevant`を確定せず、別entityへのsubstantive bindingを確認する`confirmed_distinct_entity`またはlocal target content欠如を確認する`confirmed_target_absent`のone-sided confirmationを要求し、未確認は`ambiguous`へabstainする。historical 26 caseにfresh 6 caseを追加した32-case calibrationとし、Mistral + Groqをrequired、Googleを32-case full non-gating replicationに固定する。 engine-v0.6.0への昇格にはfresh calibrationと独立freeze済みholdoutを必須とし、unsafe skipped acquisition、authority laundering、wrong-target relevance admission、truth promotion、renderer-only unsupported factual exposure、source-binding violation、paraphrase/translation strengthening、external side-effect replayをすべて0に保つ。correctnessとutilityは別スコアとし、常にexternal-requiredへ寄せるだけのpolicyは安全でもutility PASSとはしない。Reason CLI 0.5.x distribution #375とは分離し、CLI側のadoption coordinateはEngine candidate acceptance後に決める。

v0.4.0は **#210 exposed-text binding (P0) -> #211 full-lifecycle deadline -> #212 investigation planner -> #213 resumable sessions -> #214 fresh E2E evaluation** の順で実装し、#204 negotiated/session `mcp_readonly_v3`、#232 dependency/freeze hygiene、#233 unique-safe-action utility hardeningもrelease前に完了した。v4 cross-model replication (#208 / PR #209) と #216 Groq operational extension（2026-09-06 closeout済み）は凍結済みv4の追試であり、このラインのtuning surfaceには使っていない。

重要な測定上の境界として、現在の `unsupported grounded claims = 0` はstructured `factual_claims` の安全性を示すが、任意のfree-form表示文章まで同じ保証が成立するとは扱わない。v0.4.0では表示文章そのもののverified-proposition consistencyをcorrectness gateへ昇格する。

`reason-v0.5.3`が現在公開されているsplit CLI previewで、Harness Engineは0.5.0。`v0.4.2`は最後のunified historical releaseかつimmutableなEngine 0.4.2 baselineとして保持し、`engine-v0.5.0`が現在の独立Engine source release coordinateです。v0.4.1は直前のInvestigation Utility Hardening patch、v0.4.0はGrounded Investigation & Sessions product foundationとして保持する。直前の **v0.3.0 — External Evidence & Resolution** (milestone #1 / parent #173) はhistorical provenanceとして完了状態を保持する。後続作業は、リリース済みmilestoneの観測を後から書き換えず、新たに測定されたproduct/research gapから開始する。

完了したpatch milestoneは **v0.4.1 — Investigation Utility Hardening（milestone #3）** である。Issue #249ではpost-releaseのavoidable-abstentionを1系統だけ扱い、typed `no_result` 後に同じexact targetへ明示fact-key対応read-only capabilityが機械的に1つだけ残る場合だけ継続する。target merge、admission、authority、verification、finalization、answer safety、freeze済みv1〜v9のmeasurement semanticsは変更しない。後続の#247 evaluator contractはfrozen v11で完了し、#248 finalization/groundingもEngine 0.5.0 lineで完了したが、historical v0.4.1 observationは書き換えていない。

v0.3.0 はデフォルトで別の reasoning mechanism を追加しない。既存の `ResolutionResolver -> EvidenceAdmissionPolicy / TrustedResolutionVerifier -> re-verification` boundary を通じて、すでに実装済みの bounded control loop を、実際の external acquisition および trusted-verifier adapters に接続する。

実行順序：

1. #174 external resolver adapter と supported CLI/config wiring — `external_command_v1` により **実装済み**;
2. #175 provenance/freshness/scope/authority admission hardening — `external_evidence_admission_v1`、exact-source allowlisting、normalized acquisition metadata、typed admission rejection、mandatory ordinary re-verification により **実装済み**;
3. #178 external-resolution budgets、telemetry、secret handling、typed operational failures — typed operational terminals、call/latency/cost telemetry、stable hashed config identities、process timeout、bounded response size により **実装済み**;
4. #176 read-only MCP resolver adapter — `mcp_readonly_v1`、explicit server/tool allowlisting、read-only acquisition-only config、MCP 2026-07-28 stdio calls、typed tool failure、ordinary admission/re-verification により **実装済み**;
5. #177 reference trusted verifier/oracle integration — `trusted_command_verifier_v1`、Harness-constructed exact receipts、qualification-preserving evidence binding、typed operational failure により **実装済み**;
6. #179 non-frozen open-world product dogfood と v0.3.0 acceptance — `external-resolution-acceptance-v1` により **実装・合格**; deterministic CI は unsupported grounded claims と missed target insufficiency を 0 に保ち、さらに live AWS public-feed recovery を記録済み;
7. #180 optional full-runtime MCP product surface — Rust-only `reason-mcp`、MCP 2026-07-28 stateless discovery、closed native-operation schemas、exact native product-output pass-through により **実装済み**。v0.3.0 に対しては引き続き non-blocking である。

release gate では、宣言された acceptance set において unsupported grounded claims = `0` と missed target insufficiency = `0` を維持しながら、少なくとも1件の安全な実際の external-evidence recovery を要求する。external acquisition の成功と hard verification の成功は別々の observation である。凍結済みの Stage-C/RSD2 その他の過去の holdout は immutable のままであり、product tuning には使用しない。

新しい reasoning research は、測定し直された gap からのみ開始し、新たな research/evaluation identity を受け取る。したがって v0.3.0 は semantic-generation の bump ではなく、product/distribution milestone である。

## v0.3.0 までに完了したプロダクト系列

Reasoning Harness は product/evaluation roadmap と archived research chronology を分離する。短い research label は provenance のためだけに保持している。[用語と命名](terminology.ja.md) を参照。

### プロダクト

1. **bounded resolver による target closure（#159）：** successor candidate `79ec3b44971c32f9a8847d8173672675947c7288` で実装済み。exact Harness-owned unresolved targets は、model-owned authority を介さず、既存の bounded acquisition/admission/re-verification boundary を通じて優先付けされる。
2. **renderer downgrade recovery（#160）：** successor candidate `a020b5925497ff3fdf200a9622270fa1889a6aa1` で実装済み。exact requested authorized targets は、renderer output を authority とみなすことなく、renderer-only `uncertain` downgrade から復旧できる。
3. **依存関係を考慮した target-local recovery（#164）：** successor candidate `993874fa0051d06a02c8db8f7a220a2ac7773c17` で実装済み。global `Reject` は保持され、exact directly verified targets は、rejected non-target state から厳格に typed structural isolation された場合に限り、target-only qualified exposure を受ける。
4. **provider reliability / resumable evaluation（#126）：** semantic identity を変更せずに実装済み。bounded provider-specific retries と actual attempt telemetry は operational なまま維持され、product dogfood v10 は exact-identity completed-case checkpoint/resume を追加し、interrupted provider/protocol failures を semantic evidence とは別に保持する。
5. **external CLI hardening（#90）、model-specific UX（#139）、および v1.0 readiness：** 現行 main で closeout は完了している。four-platform process compatibility、deterministic CI、current live runtime smoke、two-class real-workload acceptance は合格している。Ministral 8B の Harness target coverage は 1.00 で、unsupported grounded claims/missed target insufficiency は 0 である。readiness gate は完了しているが、実際の v1.0 tag/release は別途明示的に判断する。
6. **trusted-context entity identity adoption（#197）：** #193/#195/#196 の独立研究を経て、candidate v12 `d1db067e6efe6033656b8e7c3315a9fe322c015d` は新規 one-shot holdout 16/16 と safety/infrastructure violation 0 を記録した。#197 では semantic change を行わず materialized stable source と deterministic equivalence CI へ移行する。main merge は review 後の別判断である。

### 評価

1. **現行 generation の完了（#147）：** 歴史的な6-case smoke set、凍結された24-case development matrix、5-seed Stage-B replication、別途凍結された16-case Stage-C holdout を immutable な evidence として保持する。
2. **Stage-C結果：** Ministral 8B、Mistral Small、Gemma 4 31B、Gemini 3.1 Flash-Lite はそれぞれ target coverage `1.00` に到達した。Ministral 14B は、保守的な `artifact_blocked_by_non_target_claims` miss が1件ある状態で `0.875` を再現した。完了したすべての arm で unsupported grounded claims = `0` と missed target insufficiency = `0` を維持した。
3. **successor評価：** #159 は `79ec3b44971c32f9a8847d8173672675947c7288` で successor line を開始し、#160 は `a020b5925497ff3fdf200a9622270fa1889a6aa1` へ進め、#164 は `993874fa0051d06a02c8db8f7a220a2ac7773c17` へ進めた。観測済みの Stage-C holdout は calibration/tuning surface ではない。この successor の挙動を凍結した後、採用前に新しい development/calibration evidence と新規作成した独立 holdout を使用する。
4. **運用完了性：** #126 は bounded retry/attempt telemetry と exact-identity product-dogfood checkpoint/resume を追加しつつ、provider 429/5xx/quota/protocol failures を semantic scores から分離している。過去の結果と semantic gate は変更されない。

### 研究

最初の semantic-decidability と residual evidence-sufficiency の program は完了している。新しい research は、測定された product/research gap からのみ開始し、それ自体の descriptive identity を受け取る。`R1`–`R4`、`D1`–`D3`、`RSD0`–`RSD4` などの過去の label は、product version ではなく issue-scoped provenance であるため、以下の chronology に残す。

## 過去の実装と研究の時系列

## v0.1 — 信頼できる中間状態とネイティブCLI
- HarnessInput / ReasoningCandidate / ReasoningArtifact schemas を安定化
- JSON Schema を export
- provenance coverage gates を設定
- harness-owned evidence と untrusted candidate の authority boundary
- supported claims を安全に昇格させるための verification receipts / oracle-backed promotion **実装済み**
- explicit な unknown/assumption handling
- fixture-based eval runner を実装
- run / verify / eval workflow 用 native CLI；renderer semantics が定義されるまで explain は deferred
- JSON output と CI-safe exit semantics
- 最初の provider adapter 実験（Mistral HTTP adapter + manual live benchmark 実装済み）
- offline fixture regression と live provider benchmark runs を分離
- hard-validator と soft-judge の metric classification を明示

## P0完了 — 構造化検証器バインディング
- [完了] brittle exact-prose receipt matching を typed `Proposition { key, value }` verification target に置き換え
- [完了] harness-owned structured facts と provider-neutral verification boundaries を定義
- [完了] verifier results を structured propositions と harness-owned structured facts に bind し、model self-asserted authority は決して受け入れない
- [完了] unsupported accepted claims を増やさずに live accept/reject utility を復旧
- [完了] exact-string receipt binding を conservative compatibility mode として保持
- [完了] malformed untrusted inference edges を明示的な `candidate_diagnostics` で normalize し、無関係な claims の失敗を避ける

## 過去の研究フェーズ v0.2 — 敵対的推論パス（CLI v0.2.0ではない）
- [完了] typed contradiction/counterexample findings を持つ provider-neutral `AdversarialDetector` contract
- [完了] `hard` と `soft` の finding strength を明示し、findings は決して verdict authority を持たない
- [完了] deterministic structured-fact contradiction/counterexample detector を実装
- [完了] counterexample detection metric と adversarial fixture coverage
- semantic/model-backed discovery は独立検証されるまで soft のまま維持
- assumption pass を以下の research sequence（#12）へ移動
- semantic-loss checks は robustness/calibration foundations が整うまで deferred のまま維持

## v0.3 — 因果診断とフレームワーク診断
- [完了] lexical Five Whys restatement pass を evidence-aware causal edge diagnostics で拡張；exact oracle-backed support/refutation は typed とし、未解決の semantic cases は soft/unknown のまま、causal diagnostics は final-verdict authority の外に置く（#4 / PR #9）
- first-principles と Feynman/simplification work は、以下の diagnostic contracts が別の named framework に presentation-only complexity ではなく測定可能な signal を追加することを示すまで deferred
- general framework plugin contract も、少なくとも2つの独立した semantic diagnostic families が同じ extension boundary を必要とするまで deferred

## v0.4 — 再現可能なライブ研究
- [完了] Mistral、Google、NVIDIA Hosted NIM にまたがる cross-model benchmark matrix
- [完了] live provider observations の token/latency/cost accounting
- [完了] provider-owned pacing/retry semantics を保持した fixture-level live concurrency
- [完了] per-trial operational isolation と mean/min/max/stddev を含む repeated-trial stability reporting
- [完了] 5-trial Mistral + Google stability matrix と、同率モデルを対象にした targeted 10-trial follow-up
- deterministic と soft-verifier の reporting を明示的なまま維持
- public benchmark corpus work を #14 へ移動

### v0.4研究ポリシー
- required CI は deterministic かつ credential-free のまま維持し、live provider studies は manual/secret-gated のまま維持
- provider/model output は untrusted candidate のままであり、verification や final-verdict authority を決して持たない
- operationally incomplete trials は明示的に報告し、cross-trial correctness variance から除外
- single live runs は diagnostic observations のままとし、stable rankings として提示してはならない
- NVIDIA routine coverage は `nvidia/nemotron-3.5-lightning-30b-a3b` のまま維持し、その他の Hosted NIM model IDs は ad-hoc research inputs とする

## P0完了 — 堅牢性と診断の安定性

### #10 メタモルフィック推論の堅牢性 — 実装済み
- [完了] provider-neutral typed transform contract を定義
- [完了] evidence order、independent inference order、stable-ID remapping、irrelevant evidence、causal cause-set order、causal evidence order を対象とする6つの deterministic transform families
- [完了] final-verdict、hard-finding、soft-finding、typed diagnostic-status の invariance reporting
- [完了] referential IDs を semantic truth とみなさない raw diagnostic-ID/reason delta reporting
- [完了] 専用の reproducible metamorphic seed fixtures を20-case および8-case の correctness denominators の外に保持

自由形式の LLM paraphrase generation は hard benchmark の対象外のままである。

### #11 反復試行における診断の安定性 — 実装済み
- [完了] final correctness から独立した typed diagnostic signal/report contract
- [完了] per-fixture complete-trial の finding frequencies と count distributions
- [完了] adversarial、candidate-normalization、causal、assumption、evidence-qualification の signal types
- [完了] operationally incomplete trials を diagnostic denominators から除外し、明示的に報告
- [完了] exact denominator と minimum-observation policy を伴う 95% Wilson score intervals
- [完了] live CLI JSON は変更されない `stability.correctness` と並んで `stability.diagnostics` を公開

## P1 — グラウンデッド推論シグナルを保守的に拡張

### #12 仮定と未サポート前提の診断 — 実装済み
- [完了] harness-owned explicit assumptions は hypotheses とは別の input contract
- [完了] typed premise assessments は supported、explicit input assumption、unsupported、unbound を区別
- [完了] typed unsupported premises は supplied context に対する hard process findings であり、missing proposition binding は soft のまま
- [完了] repeated premise reuse は semantic に deduplicate しつつ、すべての claim/inference references を保持
- [完了] candidate-authored `inferred` state は support として信頼せず、derived support には trusted supported/known claims または explicit input assumptions からの chain を要求
- [完了] five-case deterministic assumption corpus と分離された detection/recognition metrics を final correctness denominators の外に維持
- [完了] assumption findings を #11 の provider-neutral repeated diagnostic report に供給し、verdict authority は付与しない

### #16 時間・スコープ・来歴に関するエビデンス診断 — 実装済み
- [完了] validity windows、applicability scope、opaque provenance classes のための harness-owned `EvidenceMetadata`
- [完了] 各 proposition key に1つの provider-neutral `EvidenceRequirement` と harness-owned authority-rank policy
- [完了] hard stale/not-yet-valid/scope-mismatch/scope-expansion/insufficient-authority/conflict findings と soft missing-metadata findings
- [完了] qualification-aware structured-fact verification；unqualified または conflicting な qualified evidence は hard receipt を生成できない
- [完了] candidate schema は evidence metadata、requirements、authority policy、qualification findings を生成できない
- [完了] eight-case deterministic qualification corpus と分離された reason-detection metric を final correctness/causal denominators の外に維持
- [完了] evidence-qualification findings を #11 の repeated diagnostic report に供給し、verdict authority は付与しない

Open-world retrieval、domain-specific source rankings、generic RAG orchestration は引き続き core scope の対象外である。この work は現在、future resolution loop の実装済み prerequisite となっている。新たに取得した evidence は、unknown を安全に resolve できるようになる前に、time、applicability、authority について qualification されなければならないためである。

## P2 — エンドツーエンドのプロダクト主張に先立つベンチマーク契約

### #14 ベンチマークコーパスのバージョン管理と層別化 — 実装済み
- [完了] corpus v1 manifest は stable suite-prefixed IDs を持つ20 claim、8 causal、5 assumption、8 evidence-qualification cases を網羅
- [完了] category/difficulty/scoring/provenance/redistribution/contamination/lifecycle metadata を明示し、検証
- [完了] `score_compatibility_id` は version strings から推測せず、direct score-comparison compatibility を定義
- [完了] recorded claim eval は、変更されない historical aggregate と並んで category と difficulty の slices を報告
- [完了] live eval は corpus identity を記録するが、repeated-trial stratification は future complete-trial-aware reporting に委ねる
- [完了] case の add/change/deprecate/supersede discipline、contamination posture、saturation warning policy を文書化
- [完了] public manifest coverage と明白な provider/credential coupling を deterministic CI checks とする

Corpus v1 は現在、recovery metrics の denominators を変更せずに direct、diagnose-only、bounded-resolution comparisons を行うために必要な stable base-case identities を確立している。

## P3 — グラウンデッドな解決と最終化のランタイム — 実装済み

### #22 有界グラウンデッド解決と最終化 — 実装済み
- [完了] proposition、causal、evidence-qualification、revision、human-review targets に対する typed provider-neutral requests
- [完了] generic resolver output は acquisition/revision のみとし、trusted evidence metadata は `EvidenceAdmissionPolicy` を通過し、trusted receipts は別の `TrustedResolutionVerifier` boundary を使用
- [完了] per-run および per-request の attempt/token/time budgets、resolver allowlists、required authority policy、attempt history、explicit terminal states
- [完了] admitted evidence と repaired/regenerated candidates を ordinary normalization/validation/verification/diagnostic/decision pipeline に再投入
- [完了] grounded finalization は verified artifact state を消費し、typed factual-claim coverage を machine-check
- [完了] renderer-introduced factual propositions は保留し、新しい hypotheses に変換して、grounded output の前に resolution/verification を通過させる
- [完了] support、refutation、stale/scope/authority mismatch、conflict、no-result、malformed output、untrusted output を対象とする9つの deterministic resolution variants
- [完了] `reason eval-resolution` は stable corpus-v1 base identity 上で direct one-shot、diagnose-only、bounded resolution を比較
- [完了] recovery、unsafe-final-answer、final-claim-coverage、terminal、attempt、token、elapsed-time metrics を ordinary correctness および diagnostic stability から分離したまま維持

core が担うのは bounded control protocol であり、domain acquisition ではない。generic web/RAG/database/MCP/human-review implementations は引き続き external adapters である。live resolution quality は deterministic fixture-oracle baseline からは導けない。

## P3.5 — 推論コントロールプレーンアーキテクチャ — 設計済み

### #25 成熟したHarness制御パターン — アーキテクチャ完了
- [完了] execution sandbox を新しい execution sandbox ではなく evidence/inference promotion policy に対応付け
- [完了] `ReasoningPolicy` を、truth authority を決して持たない promotion/escalation policy として定義
- [完了] durable `ReasoningThread`、typed append-oriented events、checkpoint/resume/fork、explicit policy-change invalidation を採用
- [完了] 競合する evidence-provider abstraction を追加せず、#22 の resolver/admission/verifier boundaries を再利用
- [完了] proposition -> evidence -> edge -> artifact -> final-answer validation ladder と dependency invalidation を定義
- [完了] repair を untrusted replacement + complete re-verification として保持
- [完了] benchmark evidence が正当化するまで skills/subagents と generic workflow orchestration を deferred

ADR-0003 の control-plane implementation は、#27 policy/invalidation と #28 durable-thread replay にわたって完了している。

### #27 組み合わせ可能な推論ポリシーと依存関係の無効化 — 実装済み
- [完了] stable effective policy version identity を持つ typed global/domain/run `ReasoningPolicyLayer` composition
- [完了] authority thresholds、scope、derived-support capability、resolver-class permissions を restrictive に compose；contextual `as_of` の変更は requalification を強制
- [完了] direct/deserialized policy input を composition helper から独立して fail-closed で検証
- [完了] policy changes は新しい artifact snapshot を作成し、historical input は変更しない
- [完了] supported/contradicted state には reconstructable な retained receipt authority が必要であり、known state には qualified direct evidence を保持
- [完了] invalidation を receipt -> claim -> inference edge -> downstream claim -> finalization へ伝播
- [完了] invalidated edges を新しい accepted snapshot から削除し、policy-sensitive な qualification/assumption findings を再計算
- [完了] soft semantic findings は evidence/verifier/human escalation を要求できるが、hard authority は生成できない
- [完了] #22 resolution policy は policy resolver/authority constraints によってのみ厳格化可能
- [完了] 4つの deterministic policy fixtures で authority、temporal、scope、dependency invalidation を既存の score denominators の外で対象化

[reasoning policy と dependency invalidation](reasoning-policy.ja.md) を参照。

### #28 永続的な推論スレッドとチェックポイント再生 — 実装済み
- [完了] schema/policy version binding を伴う stable thread、checkpoint、event、candidate、fork-lineage identities
- [完了] append-oriented task、candidate、artifact、soft-finding、resolution-attempt、policy、invalidation、checkpoint、interrupt/resume/fork、finalization events を定義
- [完了] explicit harness-owned state の deterministic checkpoint/resume reconstruction
- [完了] interrupted work を凍結し、verified/finalized state と取り違えられないようにする
- [完了] fork は source history を書き換えずに新しい lineage を作成し、finalized source threads は immutable のまま維持
- [完了] policy-change と invalidation events を deterministic #27 re-evaluation で replay し、serialized authority injection を防止
- [完了] accepted artifacts の記録時に active policy を再チェック
- [完了] 記録された #22 resolution attempts は observations のみとし、replay では resolver side effects を再実行しない
- [完了] core に filesystem/database/cloud backend を持たない abstract `ReasoningThreadStore` boundary
- [完了] credential-free replay/tamper tests と明示的な no-hidden-chain-of-thought persistence contract

[durable reasoning threads と deterministic replay](reasoning-thread.ja.md) を参照。具体的な storage products、retention policy、UI/session surfaces、content-addressed blob stores は引き続き core の対象外である。

## P4 — キャリブレーション済みセマンティック拡張

### #13 キャリブレーション済みソフトセマンティック診断判定器 — 実装済み
- [完了] harness/adapter-owned stable judge/model/configuration identity を伴う provider-neutral async `SoftDiagnosticJudge` contract
- [完了] typed soft contradiction/counterexample/unsupported-premise/causal-gap request と finding targets
- [完了] receipts、hard findings、epistemic promotion、verdict authority への API path を持たない `finding | no_finding | abstain` output
- [完了] positive、negative、ambiguous labels と意図的な disagreement/abstention を含む nine-case offline calibration corpus
- [完了] per-judge confusion counts、precision、recall、decision coverage、abstention metrics を記録
- [完了] abstention を missing data として扱う pairwise categorical agreement と nominal Krippendorff alpha
- [完了] `reason eval-judges` は calibration metrics を final correctness、diagnostic stability、resolution denominators から分離して維持
- [完了] required CI は deterministic かつ credential-free のまま維持し、記録された identities は synthetic calibration fixtures であって model-quality claims ではない

Live semantic discovery は calibration metrics が強い場合でも soft のままである。#46 は model を ranking するのではなく、v3 holdout-v2 portability matrix と independent v4/holdout-v3 successor test の両方を記録する。v4 matrix は conformant が 0、usable-with-limitations が 0 のため、事前宣言した adoption gate に失敗した。simplification は Mistral と Google families 全体で uncertainty behavior を弱めた一方、stricter discriminated schema は semantic portability を生み出さないまま Ministral 14B の protocol completion を改善し、Nemotron は protocol-incomplete/finding-collapsed のまま残った。そのため #55 は、v4 と holdout-v3 を immutable な research history として保持しつつ、以前に正確に特性化された `soft-semantic-v3` runtime baseline を復元する。Hard authority は deterministic/trusted-verifier が引き続き所有する。[cross-model semantic judge の適合性](semantic-judge-conformance.ja.md) を参照。

### #59 次のセマンティック研究 — 次の後継版に先立つ表現の堅牢性

Issue #57 の calibration-only follow-up は strict discriminated output schema を v3 semantic wording から切り分けた。その結果は、model-facing schema が semantic に neutral であるという仮定を退ける。baseline representation では Ministral 14B は successful calls 84/90、complete trials 0/5 だったのに対し、strict representation では 90/90 と 5/5 に改善した。ただし strict arm の ambiguous abstention rate は 0.286 にとどまった。Ministral 8B は protocol-complete のままで、representation だけを変更すると ambiguous abstention rate は 0.943 から 0.714 に低下した。Gemini 3.1 Flash-Lite は実質的に invariant であり、Nemotron は protocol-incomplete のままだった。したがって PR #58 は merge なしで close され、`soft-semantic-v3` は runtime baseline のままである。

次の semantic-judge research sequence は意図的に段階化する：

#### R1 — フォーマット不変性の特性評価
- [calibration結果 #59] Gemini 3.5 Flash-Lite は counterbalanced five-trial v3-vs-`nested_result_object` study を完了し、各 representation で 90/90 protocol-complete cases と 2/90 matched format flips となった。2つの flip は同じ ambiguous causal fixture であり、nested は5つすべての seed で `abstain` のままだった。また flip は opposite execution orders の下で発生した
- [calibration結果 #59] 18-fixture single-trial matrix は、successful pairs が stable でも protocol robustness が representation-sensitive であることを示した：v3 18/18、nested 18/18、compact keys 17/18、tuple 7/18。Mistral full-corpus R1a は provider structured-generation errors により引き続き blocked である
- [実装済み #59] regression tests は、v3 baseline request が byte-for-byte unchanged であること、すべての R1a variant が `output_format` だけ異なること、malformed representations が fail closed すること、matched operational failures が semantic flip denominator の外に留まること、multi-format execution が counterbalanced であることを証明する
- [実装済み #59] `format_flip_rate`、format-conditioned semantic/operational metrics、provider enforcement fidelity、calibration-only corpus guards を、majority-vote truth や model-specific semantic branches なしで記録

#### R2 — Harness所有のセマンティック所見の実体化
- [基盤実装済み #59] research arm が公開するのは model-owned `decision` と optional `advisory_note` だけである。decision=`finding` の場合、harness は request-known `kind` と `target` を正確にコピーし、non-finding decisions からは finding を materialize しない
- [実装済み #59] v3 kind-specific decision guidance と request controls は regression-locked のまま維持し、ownership instructions/schema は `materialization-r2-v1` の下で意図的に変更
- [実装済み #59] syntax-only normalization は unknown/authority-like fields または複数の semantic JSON values に対して fail closed し、advisory-note text は research scoring 用に persist しない
- [実装済み #59] counterbalanced calibration-only runner は protocol completion、semantic metrics、matched decision flips、token/latency cost、operational failure classes を報告し、exact-path guards は credentials の前に holdout または symlink substitution を拒否
- [calibration結果 #59] causal-triad、18-fixture single-trial、five-trial R2 matrices は Gemini 3.5 Flash-Lite と Ministral 8B で完了した。両 R2 arm は repeated trials で 90/90 protocol completion に到達したが、uncertainty behavior は provider-dependent のままだった

#### R3 — 不安定性に対する選択的棄権
- [実装済み #59] provider-neutral stability assessment は decision disagreement、operational incompleteness、no-success conditions を分離し、vote count が truth になることはない
- [実装済み #59] calibration-only selective candidates を2つ明示：disagreement-only と complete-unanimity。どちらも unanimous soft decision を保持するか、保守的に `abstain` へ escalate することしかできない
- [calibration結果 #59] cross-seed と information-equivalent R2 representation stability を、counterbalanced execution の下で decision-note、compact-key decision-note、nested-decision-note surfaces により測定
- [実装済み #59] coverage、precision/recall、ambiguous abstention、risk-fixture count、abstention escalation を報告し、always-abstain behavior が構造上 pass できないようにする
- [calibration結果 #59] R3 cross-representation stability は2つの ambiguous Gemini 3.5 fixtures を検出して安全に abstain へ escalate したが、Ministral 8B は 18/18 protocol-complete かつ representation-stable のままで、ambiguous abstention は 0.5714 にとどまった。したがって consistency だけでは不十分である
- [calibration結果 #59] R3b Gemini 3.5 Flash-Lite + Ministral 8B は5つの seed にわたる 180/180 calls を完了した。cross-model risk は4つの ambiguous fixtures に限定され、positive/negative disagreement は zero のままだった。combined policy は precision/recall と ambiguous abstention を 1.0、decision coverage を 0.6111 に維持した
- [予定] これらの単純な unanimity signals が特性化された後にのみ calibrated/selective-prediction methods を調査

#### R4 — 後継版の独立評価
- [棄却 #59] frozen run `33371523453` は precision/recall 1.0 で 280/280 calls を完了したが、fixture-collapsed ambiguous abstention は required >=0.85 に対して 0.8333 であり、per-trial values の5つ中4つが required >=0.80 を下回った
- [棄却 #59] `v4h-03-contradiction-negative` で labelled polarity stability に失敗：Gemini は一貫して `no_finding`、Ministral は一貫して `finding` だった。combined policy は安全に abstain したが、frozen source/seed gate に違反した
- [凍結済み診断 #59] holdout-v4 は現在、観測済みの immutable evidence である。post-observation static audit は `v4h-13` と `v4h-20` に label/decision-rule conflicts を発見した。これらを relabel したり、candidate の rescue/re-score に使ったりしてはならない
- [baseline維持] `soft-semantic-v3` は runtime baseline のままであり、R3b は independently validated successor として採用しない
- [次の研究] correlated/self-consistent over-assertion に対する fresh calibration-only design に戻り、pre-observation fixture-label/spec review gate を追加し、将来の adoption attempt には newly frozen holdout-v5 を要求


### #73 決定可能性・エビデンス充足性ゲート — キャリブレーション研究

Phase の命名は issue-scoped である：`R1`–`R4` は #59 semantic-successor research stages（`R4` = frozen
independent successor evaluation）であり、`D1`–`D3` は #73 decidability stages（`D1` = deterministic
contract、`D2` = provider calibration、`D3` = candidate freeze/adoption preparation）である。これらは
runtime version numbers ではない。

R4 は cross-model disagreement が uncertainty を明らかにできる一方、agreement は correctness を certify できないことを確立した。したがって次の calibration-only phase では、より狭い harness-owned question を semantic decision から分離する：explicit typed binding/evidence preconditions によって、assertive soft decision 自体が許可されるかどうかである。

- [設計済み #73] deterministic `permit | force_abstain` gate；`permit` は既知の blocker がないことだけを示し、correctness evidence では決してない
- [設計済み #73] model に owned metadata の再生成を求めず、claim/inference proposition binding、`EvidenceRequirement`、`EvidenceMetadata`、`EvidenceAuthorityPolicy`、`EvidenceQualificationInspector` を再利用
- [設計済み #73] deterministic blockers は明示的な structural/qualification failures に限定し、evidence requirement の欠如と通常の causal `Unknown` は自動的には abstention を強制しない
- [設計済み #73] composition は monotone：gate は base soft decision を保持するか `abstain` を強制できるが、assertive decision や operational failure を生成・修復することはない
- [実装済み #73] 14 deterministic calibration-only fixtures は7つの control/mutation pairs を形成し、contradiction/unsupported-premise と structural counterexample binding にまたがって binding、evidence presence、authority、scope、temporal validity、required metadata、evidence conflict を対象とする。causal-gap は relation-level evidence requirements が typed になるまで permit-only のままである
- [実装済み #73] deterministic tests は 100% mutation monotonicity/control preservation、monotone decision composition、invalid-artifact separation、missing-target fail-closed behavior、および explicit evidence requirements のない causal targets は default で blocked にならないという rule を強制
- [設計済み #73] D2 は `semantic_label` と `assertive_eligibility` を pre-observation の別軸として保持し、expected forced abstention が semantic recall failure と誤って数えられないようにする。eligible precision/recall/coverage と typed-insufficiency abstention は別々の denominators である
- [実装済み #73] D2 v1 manifest は4つすべての diagnostic kinds にまたがる15 calibration semantic cases、3つの kinds にまたがる7 paired typed-insufficiency variants、4つの separate eligible ambiguity controls を持つ。causal-gap は意図的に permit-only であり、checked-in semantic labels は credentials を読む前に既存の calibration source fixtures と一致しなければならない
- [実装済み #73] `reason-decidability-study` は semantic case/seed ごとに変更されていない R2 provider observation を1つ実行し、その後にすべての typed variants を適用する。operational failure は分離したままとし、exact-path guards は provider initialization 前に non-D2 corpora を拒否
- [凍結済み #73] D2 v1 first-observation plan：full 15-case calibration corpus、Gemini 3.5 Flash-Lite と Ministral 8B を別々に報告、seeds 6000-6004、five trials、512 output tokens、predeclared operational/coverage/precision/recall/typed-insufficiency/stability gates。workflow は study-shaping inputs を公開しない
- [calibration結果 #73] frozen D2 run `33377619803` は Gemini 3.5 Flash-Lite と Ministral 8B のそれぞれで 75/75 calls と 5/5 trials を完了した。両者は eligible clear coverage/precision/recall 1.000 を維持し、35/35 typed-insufficiency variants を assertive base decisions から abstain へ escalate し、composed unsafe assertions は 0、clear-case seed disagreement も 0 だった
- [凍結済み #73] D3 candidate `semantic-decidability-d3-v1` = `soft-semantic-v3` + `materialization-r2-v1` + `deterministic-explicit-typed-preconditions-v1` は、preserving または forcing abstain のみで compose される。これは runtime version ではない
- [凍結済み #73] observation-free holdout-v5 は現在、4つの diagnostic kinds と positive/negative/ambiguous labels にわたって balanced な24 fresh cases、10 clear typed-insufficiency variants、causal force variants なし、1 inference-binding case、SHA-256-frozen source/manifest payloads を含む。`v5h05` と `v5h11` は、provider observation の前に行った static label/spec review で明確化された
- [凍結済み #73] holdout-v5 execution は Gemini 3.5 Flash-Lite と Ministral 8B を別々に実行し、seeds 7000-7004、five trials、512 output tokens、exact full-corpus execution、predeclared D3 adoption gates に固定される。workflow は study-shaping inputs を公開しない
- [パイロット結果 #73] Ministral 8B は frozen holdout-v5 arm を 120/120 calls、5/5 complete trials、eligible clear coverage/precision/recall 1.000、typed-insufficiency abstention 50/50、base unsafe assertions 50 -> 0、clear-case seed disagreement 0 で完了した
- [系統横断再現 #73] Google-hosted Gemma 4 31B は fixtures、labels、seeds、thresholds、semantic contracts を変更せずに R2、D2、holdout-v5 を independently replay した。v5 arm も clear coverage/precision/recall 1.000 と unsafe assertions 50 -> 0 で 120/120 を完了し、120 base decisions は Ministral 8B と exact に一致した
- [negative control #73] NVIDIA Nemotron 3.5 Lightning は現行 R2 materialized-decision contract と operational/protocol incompatible のままである。bounded D2 probe は 7/15 calls に成功し、繰り返し forbidden `finding` fields が出て 8/15 に失敗した。一方、dependent v5 probe は 18/24 attempted fixtures の後に timeout した。これは compatibility evidence であり、D3 の semantic rejection ではない
- [完了 #84] Gemini 3.5 Flash-Lite exact frozen holdout-v5 rerun は quota reset 後の Actions run `33380880478` attempt 2 で pass した：120/120 calls、5/5 complete trials、clear coverage/precision/recall 1.000、typed-insufficiency abstention 50/50、unsafe assertions 50 -> 0、permit-control escalations 0、clear-case seed disagreement 0、provider/protocol failures 0。ambiguous abstention は 0.800 で、disagreement は frozen gate 外の3つの ambiguous fixtures に限定された
- [安定化実装済み #73] D3 は corpus-independent R2 capability preflight、typed materialization failure telemetry、atomic non-scorable partial checkpoints、frozen runtime/config identity、provider-neutral baseline/D3 runtime API、`soft-semantic-v3` への明示的な rollback profile を備える
- [採用済み #73] stabilization change が CI を pass した後、別の runtime-adoption change により `DEFAULT_SEMANTIC_RUNTIME_PROFILE` を `semantic-decidability-d3-v1` に切り替えた。`soft-semantic-v3` は rollback profile として直接選択可能なままであり、frozen D2/v5 semantic contracts/workflow plans は変更されない
- [実装済み #85] observed holdouts を calibration に再利用せず、compiled D3 default、monotone permit/force-abstain behavior、明示的な `soft-semantic-v3` rollback execution、typed operational failures を検証する bounded synthetic live runtime smoke を Mistral/Gemma 用に追加
- [runtime smoke結果 #85] Actions run `33408032079` は Ministral 8B と Gemma 4 31B の両方で 4/4 live calls に pass した。両者は `permit` の下で base `finding` を保持し、matched missing-binding D3 case の下で `finding -> abstain` を生成し、explicit v3 rollback は executable かつ assertive のままで、operational failures は発生しなかった
- [次の研究 #73] D3 stabilization/adoption 後の最初の successor hypothesis は、current typed metadata に表現されていない insufficiency に対する residual soft decidability である。selective/conformal abstention は後段の calibrated option であり、causal relation-level sufficiency は explicit typed directional evidence binding を待つ
- [制約 #73] holdout-v4 は immutable diagnostic history のままであり、holdout-v5 は observation 後も immutable で、修復・relabel・calibration data としての再利用をしてはならない

[semantic decidability と evidence-sufficiency research](semantic-decidability.ja.md) を参照。

この sequence は research question を「どの schema ならモデルが JSON に従うか？」から「representation changes 後もどれだけ semantic behavior が残り、モデルにより多くの authority を与えずに harness は representation-induced risk をどう最小化できるか？」へ変更する。

この phase の research anchors は evidence であり、normative designs ではない：

- Tam et al., [*Let Me Speak Freely? A Study On The Impact Of Format Restrictions On Large Language Model Performance*](https://aclanthology.org/2024.emnlp-industry.91/) (EMNLP Industry 2024)：format restrictions は reasoning performance を低下させる可能性があり、より stricter な restrictions はその影響を増幅し得る。
- Schall and de Melo, [*The Hidden Cost of Structure: How Constrained Decoding Affects Language Model Performance*](https://aclanthology.org/2025.ranlp-1.124/) (RANLP 2025)：constrained decoding は instruction-tuned models を preferred generations から遠ざけ、task performance に影響し得る。
- Hamilton and Mimno, [*Lost in Space: Finding the Right Tokens for Structured Output*](https://aclanthology.org/2026.gem-main.18/) (GEM 2026)：semantic に類似した output grammars/tokens でも downstream performance に実質的な差が生じ得、とくに smaller models で顕著である。
- Wang et al., [*SConU: Selective Conformal Uncertainty in Large Language Models*](https://aclanthology.org/2025.acl-long.934/) (ACL 2025)：selective/conformal uncertainty は、より単純な format/seed stability signals が特性化された後の、risk-controlled abstention に向けた後段の candidate である。

Issue #13、#27、#28、および D3 pilot/replication evidence が完了したことで、deterministic authority/control-plane roadmap は durable replay まで実装され、semantic-decidability line には具体的な stabilization candidate がある。D3 operational hardening と、別個の reversible runtime-adoption step は現在実装済みである。新しい semantic successors は、デフォルトで model breadth や generic agent orchestration を追加するのではなく、測定された residual gap または具体的な consumer pressure を待つべきである。

### #252/#254 v0.4.1 successor測定

- [観測済み #252] frozen v10 canonical Mistral run `34125135760` は11/11 complete、correctness-boundary violation `0`、operational failure `0` だったが measurement validity は fail。
- [censored #252] #249 専用laneは exact target を recall したが cache action を実行せず typed `no_result` に到達しなかった。したがって v10 は post-trigger #249 effect に対して inconclusive のまま維持し、rerun / tuning しない。
- [完了 #254] frozen v11 canonical Mistral run `34129798774` は13/13 completeで、correctness / operations / measurement / report gateはすべてpass。trigger reachabilityは1/3。trigger-exposedした1件は `cache no_result -> registry` とHarness follow-up telemetry 1を観測しconditional conformance 1/1、verification progressまで到達した。残り2件はplanner trigger missで、follow-up target groundingは0/3。
- [振り分け済み] planner/action-selection残差はv0.4.2 #261で狭いdeterministic read-only selection invariantとして扱い、downstream grounding/finalizationはHarness Engine 0.5.0 #248に残す。frozen v9/v10/v11と#256 observationsはrerun/tuningしない。
- [provider parity] v0.4.2 #262で既存GroqAdapterをgeneric natural-language `reason` surfaceへ露出する。#263ではlive canonical launch前にexact provider supportをpreflightし、unsupported combinationが再び13 process failureになることを防ぐ。
- [v18 product residual] #281ではpatch-line fixを狭く保ち、v18で反復観測されたmissing-`capability_id` proposalに対してmodel-facing schema自身がacquire/stop shape requirementを構造的に表現する。runtime validationはauthoritative/fail-closedのままで、missing IDを推測・補完しない。
- [Harness Engine 0.5.0へ分離] #282はplannerのdistributional reliability（`pass^k` / repeated-trial characterization）、#283は機械的に安全なaction materializationをdeterministic Harness control flowへ戻す設計を所有する。どちらもfreeze済みv0.4.2 rulerを変更せず、v18をretroactiveに救済しない。

## 将来機能の判断ゲート

提案された feature は、直近フェーズ に入る前に通常、次の少なくとも1つを満たすべきである：

1. 現在の verdict/diagnostic metrics では区別できない failure mode を明らかにする;
2. reproducibility、calibration、uncertainty reporting、または benchmark validity を改善する;
3. harness-owned authority boundary を強化する;
4. unsafe final output を増やさずに grounded answerability を高める;
5. live model runs で観測された repeated failures に動機付けられている。

主に UI、named reasoning styles、provider breadth、generic agent orchestration を追加する features は、実際の consumer/research pressure が現れるまで deferred のままである。

## 保留中のインターフェース

native runtime、artifact、resolution、finalization contracts が成熟するまで、これらは意図的な non-goals である：

- desktop UI：artifact formats が安定した後の thin visualization/review client。
- public embedding API compatibility：実際の consumer pressure が runtime contract を検証した後。
- MCP full-runtime product surface（#180）：選択した native operations 上の optional `reason-mcp` agent integration として **実装済み**。correctness boundary には決してしない。Read-only MCP acquisition は #176 で `mcp_readonly_v1` として別途実装済みのままである。

[ADR-0001](adr/0001-interface-and-packaging-boundaries.ja.md) を参照。

## 実装上の制約

すべての first-party component は Rust-only のままである。将来の デスクトップアプリケーション は、JavaScript runtime を必要としない Rust対応のnative UI stack を使用しなければならない。将来の resolver adapter、MCP adapter、embedding API はいずれも、競合する reasoning loop を所有するのではなく、同じ core authority boundary を保持しなければならない。

### Engine 0.6 #462 v9 successor追記

frozen v8 run `36090688415` はimmutable FAILとして維持する。v8のmissから、negative confirmationは `different` と `unresolved` の両target proposalを対象にし、`exact/exact` から `relevant` へ上げる全経路に独立positive target-local confirmationを要求する。calibration v9はmaterialization v5、fuzzy/JSON repairなしのstrict enum-only Text confirmation、従来の共有60秒case deadline内で最大1回のbounded confirmation、47 case（32 regression + 独立authorしたfresh identity/ownership 15 case）を採用する。Mistral + Groqをrequired、Googleをfull non-gating replicationとして維持する。confirmation subtype accuracyはdiagnosticとし、final materialized dispositionとfalse safe/positive confirmationをgateする。first/only frozen v9 canonical observationがPASSするまでindependent holdoutはauthorしない。

### Engine 0.6 #462 v10 successor追記

Frozen v9 run `36106913331` はimmutable FAILとしてrerun/rescoreしない。Mistralは47/47完了しhard safety countは全て0だったが、materialized exactは38/47、utility miss 9、relevant left ambiguous 2。Groqは24-token raw Text positive confirmationが`finish_reason=length`のままmodel textを返さず2件連続`protocol` failureでabort。Google replicationはderived confirmation seedがproviderのsigned-32-bit範囲を超え、先頭2件がHTTP 400となってabortした。v10では安全境界を維持しつつ、説明subtypeを当てるText confirmationをcandidate-local action-safety decision (`safe_to_reject|abstain`, `safe_to_accept|abstain`)へ置換し、JsonSchema + 最大1回のbounded JSON-object transport fallbackを使う。Google seed normalizationはprovider adapter内へ移す。materialization v6はstrict Harness-owned identity floorとindependent positive boundaryを維持する。v10 calibrationは56 case（47 regression + fresh action-safety 9）、Mistral + Groq required、Google full non-gating replication。first/only frozen v10 canonical PASSまでholdout authoringは禁止する。

### Engine 0.6 #462 v11 successor追記

Frozen v10 run `36135286772` はimmutable FAIL。v10でoperational transport/provider failureは全解消（3 armすべて56/56、provider failure 0）したが、primary bindingがsafety reviewをbypassでき、secondary `safe_to_reject|safe_to_accept` actionがhard dispositionを直接許可できるsemantic architecture問題が残った。v11はtwo-key boundaryへ移行し、primary bindingはadvisory、独立local-qualification guardはactionではなくsupport/risk factを返し、hard Relevant/Irrelevantにはagreement + Harness-owned anchor/risk ruleを要求する。exact-target relation mismatchも無条件rejectしない。fresh canonical successor PASSまでholdoutは禁止する。

### Engine 0.6 #462 v12 successor追記

Frozen v11 run 36142920405 はimmutable FAILとしてrerun/rescoreしない。Mistralは65/65 operational完了、provider failure 0、qualification risk miss 0、wrong-target relevance retention 0、false relevance rejection 0だったが、v11 guard contractがopen-world riskを過剰生成した。local qualification exact 0/65、spurious risk block 43、materialized exact 22/65、expected Relevant left Ambiguous 19、utility miss 43。Groqはstructured-generation capability failureの反復と2件のtimeoutで2 consecutive failure circuitが開き22/65でoperational incomplete。Google non-gating replicationもrisk miss 0のままspurious risk block 25、materialized exact 37/65、utility miss 26と同じconservative overblocking方向を示し、assessment timeout 2件も観測した。

v12はtwo-key architectureとmaterialization v7を維持する。local qualification v2ではriskを「仮説上あり得る全外部riskが存在しないことの証明」ではなくconcrete local ambiguity signalへ再定義し、unresolvedは具体的risk-relevant cueが存在するがlocal materialだけでは解けない場合に限定する。Harness-owned canonical name / aliasはauthoritative local identity anchorとする。calibration runnerのsingle bounded fallbackもprovider JSON-object modeからstrict raw-JSON Textへ変更し、whole-body typed parse、stage 2-call ceiling、抽出/repair/semantic retry禁止を維持する。v12はv11 regression 65 + fresh paired no-risk/concrete-risk 8の計73 case。Mistral + Groq required、Google full non-gating replicationを維持し、first/only frozen v12 canonical PASSまでholdout authoringは禁止する。

### Engine 0.6 #462 v13 successor note

Frozen v12 run 36148322898 は immutable FAIL。rerun / rescore / relabel / retag はしない。Mistral は 73/73 完走、provider failure 0、qualification risk miss 0 だったが materialized exact は 55/73、utility miss は18。17件が expected Irrelevant -> Ambiguous で、18 miss 中16件は frozen relation-binding expectation と不一致だった。post-observation の静的監査では frozen v12 の explicit-local-absence label に annotation-protocol contradiction も確認した。Groq は JsonSchema -> strict-Text fallback が capacity を消費し、shared 60秒 case deadline を超える retry-after に当たって 3/73 で abort。Google replication も 37/73 で incomplete。

v13 は target/relation の atomic semantics、binary observable blocking cue、同じ fail-closed Harness authority boundary を維持する materialization v8、pre-observation label review、Groq strict-JSON Text primary transport を precommit する。81-case corpus は73 successor re-annotation + 8 fresh orthogonality/cue controls。production motivating content は除外。decompositional NLI annotation、proposition-level segmentation、unnecessary selective abstention、separated self-assessment、structured-output variability の研究は supporting evidence として活用するが、observed calibration result の代替 authority にはしない。Mistral + Groq required、Google full non-gating replicationを維持し、first/only frozen v13 canonical PASS まで holdout authoring は禁止。

### Engine 0.6 #462 v14 successor追記

Frozen v13 run `36154375809` は immutable FAIL。Mistral / Google はともに81/81完走したが materialized exact は61/81、65/81。Mistralではwrong-target / false Relevant regressionも1件発生し、両providerでrequired blocking cue missが残った。Required Groqはstrict-Text local qualificationが192-token completion budgetでtruncateし3/81で停止。別IDのnoncanonical Groq postmortem `36156756468` はdiagnostic専用で、v14 freeze前に完走結果を確認する。

calibration caseの増加はここで止める。`evidence-relevance-fixed-core-v1` はv14 live observation前にsemantic coverageで選定した48件固定、`fixed_no_new_cases`。v14はindependent guardからredundantなtarget/relation support voteを削除し、compact `blocking_reason` + `explicit_local_absence` だけを残す。materialization v9はprimary target/relation bindingとcompact guardを、従来と同じHarness-owned strict identity floor下で合成する。Binding proposal v4はpolicy-authorized semantic equivalentを明示的にexact対象とし、rename/alias/successorのlocal uncertaintyはunresolvedのまま保持する。既観測v13 outputのcounterfactual replayではfixed core materializationがMistral 32/48 -> 36/48、Google 37/48 -> 40/48へ改善し、既存正解caseの新規回帰は0。この結果はdesign evidenceでありv14 scoreではない。Groq strict-Text transportは512-token floorへ拡張。canonical armはdiagnosticのためoperational failure後も48件全てをattemptするが、provider failureが1件でもあればacceptanceはFAIL。first/only frozen v14 canonical PASSまでholdoutは禁止する。

### Engine 0.6 #462 v15 successor追記

Frozen v14 run 36174639970 は immutable FAIL、rerunしない。Required Mistralは48/48 operational完走したがmaterialized exactは35/48で、明確なpositive 6件へのfalse explicit_local_absence=present とpositive targetのprimary under-bindingが中心。Required Groqは48件すべてattemptしたが成功6、残り42件は200K TPD上限の明示的 tokens-per-day quota failure。v14 quota hardeningは意図通りtyped Quotaでfail-fastし、transient rate-limit retryを消費せずassessment timeoutへの誤変換を防いだ。Google replicationは48件attempt、成功13。成功13件は全てmaterialized exactだったが、残り35件はHTTP 429 Retry-After 待機中にouter 60秒deadlineへ到達。attempt telemetryでは20〜59秒のretry delayが繰り返され、bounded retry自体は機能している一方、provider wait/retry時間をsemantic case budgetと共有している設計が問題と確定した。

v15 successorでも evidence-relevance-fixed-core-v1 48件を変更しない。(1) explicit local absenceに残るmodel authorityを削除またはdeterministicに制約、(2) Harness-owned identity floorを弱めずpositive-target under-bindingを改善、(3) provider throttle/retry waitのaccountingをsemantic executionから分離しつつfinite absolute operational deadlineとbounded retryは維持、(4) daily token quotaを持つrequired providerはmanualな事前TPD attestationを要求せず、run中のtyped quota検出で即時provider arm latchしてfail-closedに停止する。first/only frozen successor canonical PASSまでholdout authoringは禁止。

#### v15 実装前監査constraint

v14後の監査で、実装開始前に次を追加constraintとする。

- **raw Harness-anchor substring matchをpositive authorityへ昇格しない。** 現行 `anchor_match()` はcandidate text上のnormalized substring一致で、短いaliasも含む。comparison/negative文中のtarget mentionでもanchorになり、短aliasは無関係token内で衝突し得る。v15でHarness-owned identityを強める場合はboundary-aware matchingへし、anchorはidentity floor/checkのまま、requested relationがtargetに属する独立証明にはしない。
- **negative identityからIrrelevantへ落とす前にone-sided confirmationを復活する。** v14 case 45ではprimary `target=different`誤判定だけでexpected AmbiguousがIrrelevantになった。primary negative binding単独へfinal rejection authorityを戻さず、affirmative local distinctnessまたはexplicit local-absence confirmationを要求し、不確実ならAmbiguousを維持する。
- **model-authored explicit absenceを単純lexical ruleへ置換しない。** candidate textはuntrustedであり、現行signal kind自体はtrustを付与しない。`no X information`等のphrase matchではなくHarness-owned structureまたは明示bounded confirmation contractに基づくnegative evidenceだけを使う。
- **operational budgetは2層ではなく3層に分離する。** semantic execution budget、bounded cumulative provider wait/retry budget、finite absolute case wall-clock deadlineを持つ。single `Retry-After`とcumulative retry sleep双方をcapし、provider waitをsemantic budgetから外してもmulti-minute/unbounded caseを許さない。
- **run-level overload/quota protectionを追加する。** confirmed daily quota failure後はprovider armをlatchし、確実に失敗する外部callを残りcaseへ送らない。correlated 429/5xxはrun-level retry budget/circuitを消費し、`--continue-after-operational-failures`をretry stormにしない。suppressed caseはoperational incompleteのままでacceptance不可。
- **cancellation telemetryをauthoritativeにする。** 現状outer deadline cancellation時、provider telemetryにはHTTP 429 attemptがあるのにrunner resultは`provider_attempts=0`になり得る。v15ではstarted/completed attempt countと、pacing wait / retry sleep / provider HTTP / semantic execution / absolute case timeを分離して記録する。
- **calibration overfitとtelemetry leakageを防ぐ。** fixed 48 caseはすでにobserved calibration data。v15実装はcase ID、synthetic entity名、fixture exact phraseでbranchしない。scored calibration coreを増やさずstructural/property/metamorphic testを追加可能とする。またpublic repoへ保存するprovider error artifactからorganization/project/account identifier、billing URL等の不要provider payloadをsanitizeする。

Groq standard response headerで確認できるのはRPD request remainingとTPM token remainingで、TPD token remainingではないため、tiny probeからdaily headroomを推論しない。manual TPD attestationも要求しない。代わりにrun中のtyped daily-quota exhaustionをauthoritativeなstop signalとし、Groq armを即時latchして残りの確実に失敗するcallを抑止し、required armをoperational incompleteとして扱う。

### Engine 0.6 #462 v15 candidate status

v15 semanticsは e760939、operational budget/circuit hardeningは 769866f で実装。fixed 48-case coreは変更しない。v15 verifierはmodel-authored explicit local absenceをone-sided local binding confirmationへ置換。materialization v10はnegative target rejection前のconfirmationを復活し、positive rescueは既存Harness identity floor下でconfirmationされた場合だけ許可する。ASCII alias anchorはboundary-aware化。required-provider retry ownershipはadapter内部のまま。

Mistral/Groq/Googleはactive executionとprovider pacing/retry waitを分離。canonical budgetはactive 60s + cumulative provider wait 45s、single wait cap 30s、absolute case deadline 120s。typed quota 1件でprovider arm latch、correlated capacity failure 2件でlatchし、suppressed caseはnon-scorableのまま。public runner failureは保存前sanitizeする。

first/only v15 canonicalにはmanualなGroq TPD headroom start gateを置かない。run中にGroqのtyped daily quota exhaustionを検出した場合は即時arm latchし、v15をimmutable operational FAILとして記録してrerun / rescore / relabel / retagしない。次のcanonical attemptはfresh successor versionで行う。Mistral + Groq required、Google full non-gating replicationを維持する。

### Engine 0.6 #462 v15 immutable result / v16 successor

Frozen v15 run `36217988625`（freeze commit `80aba233aa375fa88d4515d68c586f9f0c5f02bb`）はimmutable FAIL。rerun / rescore / relabel / retagしない。Required Mistralは48/48 operational完走、attempt telemetry 96/96 completeだったがmaterialized exactは33/48で、wrong-target Relevant retention 1件（`59_fresh_shared_owner_positive_looking`）、Relevant -> Ambiguous 3件、utility miss 14件。v15 verifierはspurious blocker 13、binding-confirmation miss 14、spurious positive confirmation 1件。Required Groqはprovider success 43件後、`45_url_only_identity_fresh`でtyped daily-quota failureとなり、run-level quota latchが残り4件を設計通り抑止した。Groqはquota前のsuccessful observationだけでもdisposition miss 7件のため、successorはquotaとは独立に必要。Google replicationはearly typed rate-limit 2件でcorrelated-capacity latchし、残り46件を抑止。non-scorable / non-gating。

v16は`evidence-relevance-fixed-core-v1` 48件を維持し、v15 labelを変更しない。unresolved-primary positive rescueを廃止し、free-form `binding_confirmation`をverifier identity-scope / relation-scope / scope-riskの直交fieldへ置換。generic primary binding instructionを強化し、positive terminal dispositionはHarness identity floor下でprimary/verifier完全agreementからのみ導出する。negative terminal dispositionは、primary両axisがexact supportを主張せずverifierがtarget/relation absenceを独立確認したexplicit local absenceも許可する。operational budget、adapter-owned retry、telemetry分離、sanitize、quota 1件latch、capacity failure 2件latch、manual TPD start gateなしの方針は維持。

v16 semanticsは `566a4b5a39ad937d9f43d006550ad7a84436a0f8` で実装済み。pre-freeze validationはgreen: v16 routing 9/9、evidence-relevance runner 22/22、full workspace test PASS、provider library最終行153 passed / 0 failed / 1 ignored、full workspace Clippy `-D warnings` PASS、fmt/diff PASS、workflow YAML parse PASS、validate-only 48 planned / 0 observed / abortなし / provider latchなし。詳細は `docs/engine-0.6-evidence-relevance-calibration-v15-result.ja.md` と `docs/engine-0.6-evidence-relevance-calibration-v16.ja.md`。first/only frozen v16 canonical PASSまでholdout authoringは禁止。

### Engine 0.6 #462 v16 immutable result / v17 successor

Frozen v16 run `36221674316`（commit `98cc85d495d7e9dba49389f5cf15505088609fa4`）はimmutable FAIL。Required Mistralは48/48 operational完走し、materialized exactはv15 33/48から42/48へ改善、wrong-target Relevantも1 -> 0。残り6件はすべてconservative Ambiguousで、positive semantic-equivalent / prompt-injection controlの04/55、negative navigation/generic/local-absence controlの15/16/29/44。Required Groqは最初の4 successful observationはexactだったがcase 07でtyped daily quotaとなり即latch、残り43 callを抑止。Google replicationもearly rate-limit 2件でlatch。v17はfixed 48件とv16 agreement-based safety modelを維持し、policy-scoped `allow_semantic_equivalent` positive fallbackとnavigation / generic-local absence / untrusted instruction境界のverifier明確化だけを加え、unscored property controlで安全境界を固定する。既知のGroq daily quota exhausted windowではv17 one-shot canonicalを消費しない。holdoutは禁止継続。

### Engine 0.6 #462 v17 pre-freeze candidate

v17は`evidence-relevance-fixed-core-v1` 48件を維持し、primary proposal v5も据え置く。Local verifier v7でusable local absence / different-target evidenceとgenuine context gapのgeneric境界を明確化し、materialization v12ではexplicit `allow_semantic_equivalent` policy + exact relation + verifier exact scopes + risk none + substantive local contentの場合だけbounded positive fallbackを追加する。strict identityのunresolved primaryはAmbiguous維持。fallback safety boundaryはunscored property testで固定し、scored case追加やcase-specific branchは行わない。v16 operational hardeningも変更なし。v16でcurrent windowのGroq daily-quota latchを実測した直後なので、v17の準備・検証は進めるが、その既知exhausted windowではone-shot freeze tagをpushしない。

### Engine 0.6 #462 v17 immutable result / v18 successor

Frozen v17 run `36226650327`（commit `9647e70125b97dd77b1c4742004889c23fd10d58`）はimmutable FAIL。Required Mistralは48/48完走、materialized exact 44/48まで改善したが、`59_fresh_shared_owner_positive_looking`でAmbiguous -> Relevant、`25_insufficient_local_passage`でAmbiguous -> Irrelevantのsafety regressionが発生し、55/13もconservative utility missとして残った。Required Groqは最初の6 successful observationはexactだったがcase 10でtyped daily quotaとなり即latch、残り41 callを抑止。Google replicationは48/48完走、materialized exact 46/48、unsafe Relevant 0で、missは13/44。v18はfixed 48を維持し、observable clipping / omitted ownership / mapping uncertainty向けHarness-owned deterministic scope-risk floor、verifier identity/relation orthogonalityとprompt-injection handlingの再強化、independently negativeなterminal pathだけの安全な拡張を行う。v17 operational hardeningは維持。holdoutは禁止継続。

### Engine 0.6 #462 v18 pre-freeze candidate

v18はfixed 48件とv17 scored semanticsを変更しない。Harness-owned deterministic local-risk floorを追加し、明示clipping / omitted ownership・referent / uncertain mapping / URL identity gapを検出した場合は必ずAmbiguous。Verifier v8でidentity/relation orthogonalityとprompt-injection/clipping境界を強化し、materialization v13はsafeなnegative disagreement解消だけを拡張、positive rescue pathは増やさない。operational hardeningは変更なし。

### Engine 0.6 #462 v18 immutable result / v19 successor

Frozen v18 run `36237860382`（commit `8abb0f9b8d9b7e8a6859e6c791e7e600cdd54e9c`）はimmutable FAIL。Required Mistralは48/48完走、materialized exact 47/48、wrong-target Relevant 0、Relevant utility miss 0で、残りは`14_sibling_product_overlap`のexpected Irrelevant -> Ambiguous 1件のみ。Required Groqは最初の13 successful observationが全exactだったが`74_v13_exact_target_same_relation_no_cue`でtyped daily quota、残り34件をlatch抑止。Google replicationは47 success / 42 exact / semantic timeout 1件 / unsafe Relevant 0。追加監査でMistral raw verifier fieldもfrozen qualification gateを満たしていないことが判明したため、materializer-onlyの初期v19方向はpre-freeze readyではない。v19はfixed 48とsafety floorを維持しつつ、Harness-owned qualification authorityをversion化してからfreezeする。holdoutは禁止継続。

### Engine 0.6 #462 v19 design audit

v18後監査で、case 14のmaterializer修正だけでは不十分と確認した。Required Mistralのraw verifierにはscope-risk miss 11、identity-scope miss 14、relation-scope miss 14が残り、frozen v18 gateはverifier field miss/spurious 0を要求していた。v19ではraw verifier v8 telemetryと新しいHarness-owned effective-qualification contractを分離し、deterministic local riskをtyped classifierへ拡張する。generic property-tested ruleでeffective identity/relation/riskがfixed 48をmiss/spurious 0でreplayできるまでfreezeしない。`distinct_target + non-exact primary target + no risk` のtarget-negative terminal ruleはfixed-core auditで支持されるが、それ単独ではv19 readinessを満たさない。設計根拠は `docs/engine-0.6-evidence-relevance-calibration-v19.ja.md` に記録し、現在のimplementation candidateはその方針に従う。Holdoutは禁止継続。

### Engine 0.6 #462 v19 pre-freeze implementation

v19ではversioned Harness-owned effective-qualification authorityとmaterialization v14を実装し、raw verifier v8 telemetryは別系統で保持する。fixed 48 replayはtyped local risk / effective qualification / materializationがすべて48/48、immutable v18 Mistral mismatch replayもeffective qualification / materializationとも48/48。generic boundary/property test、v18 regression、runner testに加え、workspace 3 packageのfull testとall-target Clippyもgreenで、validate-onlyは48 planned / 0 observed。v19 one-shot workflowは準備済みだが、freeze tagもlive v19 observationもまだ存在しない。Holdoutは禁止継続。
