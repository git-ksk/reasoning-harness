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

`v0.3.0` は現在公開されている external preview である。**v0.3.0 — External Evidence & Resolution** (milestone #1 / parent #173) は完了している。後続作業は、リリース済み milestone を暗黙に延長するのではなく、新たに測定された product または research gap から開始しなければならない。

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
