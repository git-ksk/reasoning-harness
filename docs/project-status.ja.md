# プロジェクト状況

## 現在のフェーズ

このリポジトリは、エビデンスに基づくランタイムコアと、自然言語を第一の入口とするネイティブ `reason` CLI の成熟を進めている。プロダクト境界について重要な点はすでに安定している。モデルは引き続き信頼しない候補生成器／レンダラーであり、エビデンスの受け入れ、適格性判定、検証、有界な解決処理、最終的な事実主張の公開可否は Harness 側が所有する。

**v0.3.0 — External Evidence & Resolution** マイルストーン（#173）はリリースまで完了した。v0.3.0 は、既存の bounded-resolution runtime を実際の外部取得と trusted verification に接続する一方で、correctness boundary、semantic runtime `semantic-decidability-d3-v1`、answer-safety `verified-target-answer-gate-v1` は変更していない。

v0.3.0 で完了した範囲は次のとおり。

- **external resolver integration — #174 実装済み** — `external_command_v1` を、サポート対象の自然言語 CLI／config 経路と既存 resolver boundary に接続した。closed な stdio response schema から trusted metadata、receipt、verdict、final prose を持ち込むことはできない。
- **external evidence qualification — #175 実装済み** — `external_evidence_admission_v1` は、完全一致の source allowlist と Harness-owned の authority rank、freshness bound、scope policy を使用する。resolver が返す acquisition metadata は untrusted のままで、自ら authority を昇格させることはできない。typed admission rejection は resolution attempt 上で観測でき、admit された evidence は通常の qualification／verification に戻される。
- **external-resolution operations — #178 実装済み** — external command resolution に wall-clock timeout と response-size 上限を追加した。authentication／permission／policy／timeout／transport／protocol outcome は typed operational terminal として扱い、実 call 数、latency、optional cost telemetry を hashed adapter/admission config identity とともに記録する。replay は side-effect-free のままである。
- **MCP acquisition — #176 実装済み** — `mcp_readonly_v1` は MCP 2026-07-28 の stdio `tools/call` を使い、server/tool の明示 allowlist、Harness-owned provenance、acquisition-only resolver class、typed operational failure を適用する。MCP output から authority を昇格させることはない。
- **trusted external verification — #177 実装済み** — `trusted_command_verifier_v1` は、明示的に信頼された deterministic oracle 用の独立 lane である。external output は receipt identity／binding を指定できず、qualified evidence requirement も引き続き強制される。
- **open-world acceptance — #179 合格** — non-frozen の `external-resolution-acceptance-v1` CI set で、unsupported grounded claims `0`、missed target insufficiency `0`、false abstention `0`、safe recovery 2件を記録した。さらに 2026-09-04 の AWS What's New RSS を使った live smoke では、初期 `unknown` から `aws.whats_new_feed_available=true` を回復した。
- **optional MCP product surface — #180 実装済み** — Rust-only の `reason-mcp` は、closed な `reason_ask`、`reason_run`、`reason_verify`、`reason_schema` tool を、native product runtime への薄い委譲として公開する。native JSON contract、finalization/runtime identity、operational failure を保持し、1回の MCP invocation が呼び出し側 agent loop 全体の正しさを保証することはない。

MCP external-information の entity identity 研究（#193、#195、#196）も独立した research line として完了した。trusted-context candidate v12 `d1db067e6efe6033656b8e7c3315a9fe322c015d` は、新たに凍結した #196 one-shot holdout の最初の1回だけの観測で 16/16 に合格し、semantic false decision、context-unverified fact admission、planner/tool/operational/budget failure はすべて 0 だった。#197 はこの凍結 semantics を main へ自動 merge せず stable source へ materialize する adoption phase である。#193/#195/#196 holdout は historical evidence のままで、再実行・再利用・tuning を禁止する。詳細は[エンティティ同一性ゲート](entity-identity-gate.ja.md)を参照。

#147 の product-capability evaluation と #150 の verified-utility-recovery マイルストーンも完了している。`1f27bef9e5e7d1b8d2e95c4e4245c8fe8e77b352` で freeze した semantic candidate は Stage B と、別途 freeze した16-case Stage-C holdout を完了した。最終 Stage-C target coverage は Ministral 8B、Mistral Small、Gemma 4 31B、Gemini 3.1 Flash-Lite で `1.00`、Ministral 14B は再現可能に `0.875` だった。完了した全 Stage-C run で unsupported grounded claims は `0`、missed target insufficiency は `0`。したがって14Bの残差は conservative utility evidence として引き継ぎ、観測済み holdout や gate を変更して修復しない。

successor semantic change は #159、#160、#164 に分離し、それぞれ新しい candidate/evaluation identity を必要とする。#159 は Harness-owned の未解決 target を完全一致で優先する変更として `79ec3b44971c32f9a8847d8173672675947c7288` から successor line を開始した。#160 で renderer uncertainty downgrade の完全一致 recovery を追加し、identity は `a020b5925497ff3fdf200a9622270fa1889a6aa1` へ進んだ。#164 では、狭く型付けした `Reject` target-local lane を追加して `993874fa0051d06a02c8db8f7a220a2ac7773c17` へ進めた。direct evidence-bound trusted `Supported` の target で、reject／unresolved な non-target state が構造的に分離できる場合に限り `QualifiedPartialAnswer` を公開できる。global `Reject` 自体は保持する。freeze 済み Stage-C candidate／holdout は historical evidence のままで、これらの変更の tuning には使っていない。semantic runtime、fixture、gate behavior を変えない provider-only reliability work は独立して進められる。

#126 はその独立した operational layer である。provider attempt count は bounded adapter-internal retry を含むようになり、Google には capped transient-5xx recovery と isolated-empty-output recovery を追加した一方、deterministic failure は fail-fast のままにした。product dogfood v10 では exact-identity checkpoint/resume と operational-failure history の保持も追加した。`993874fa0051d06a02c8db8f7a220a2ac7773c17` semantic candidate、frozen fixture、answer-safety gate、historical RSD2／Stage-C score は変更していない。

#90 の CLI contract gap は process level で解消済みである。`product_cli_contract` は実際の `reason` binary を起動し、versioned JSON envelope、supported stdin behavior、全 schema-discovery ID、epistemic `unknown` の exit 0、typed JSON operational failure の exit 1、`clap` usage failure の exit 2 を固定する。同じ test は main の four-platform product CLI matrix run `33822514005` で green、deterministic CI も `33822514022` で green。current live semantic runtime と current+rollback product smoke は run `33822794171` で Ministral 8B、Gemma 4 31B ともに green だった。

#139 も gate を弱めるのではなく、新しい product evidence によって完了した。main `5c5701f77df9dd507c3949294708f8c07a054064` の Actions run `33822567155` で、同じ6-case Ministral 8B product workload は historical Harness target coverage `0.25`／false target abstention `0.75` から、両 Harness arm とも target coverage `1.00`／false target abstention `0.00` へ改善した。unsupported grounded claims と missed target insufficiency は zero のままだった。raw arm は coverage `0.25` に留まったため、この回復は好条件の raw-model rerun ではなく Harness successor path によるものと判断できる。expected-unknown target は未解決のままで、人間向け CLI は最終回答を捏造せず deterministic な evidence-insufficiency guidance を表示する。

これらの結果を合わせると、文書化済みの #90 v1.0 **readiness gate** は満たしている。v0.3.0 は現在の tagged external preview であり、新しい semantic research generation を作らずに external-evidence/resolver capability を追加している。一方で compatibility semantics は意図的に v0.x／prerelease のまま維持する。v1.0 は別途、安定版化とリリースを明示的に判断する必要がある。

現在の semantic runtime と answer-safety gate は、再現性と rollback のため exact machine configuration ID を維持する。これらの ID と、`D3`、`R4`、`RSD2`、`NL-5` のような過去の label は、共通の product version sequence ではない。詳細は[用語と命名ルール](terminology.ja.md)を参照。

これは open-world reasoning が解決済みだという主張ではない。現在の correctness gain は、hard answer が存在する箇所で deterministic structure と trusted oracle を使えることに依存している。今後 utility を伸ばす場合も、retrieval、model self-correction、流暢な prose を暗黙の真実扱いせず、この authority boundary を維持しなければならない。

## 実装済み

- Rust-only の core、CLI、eval、provider adapter crate。Mistral、Google Gemini Interactions、NVIDIA Hosted NIM の candidate-generation adapter を実装済み。
- Harness-owned evidence と untrusted `ReasoningCandidate` の境界。
- deterministic な structural／provenance validation。
- `accept | reject | unknown` policy。
- model-owned／model-visible にはならない trusted verification receipt。
- receipt-backed の support promotion と contradiction rejection。
- 問題のある inference edge だけに限定して適用する、狭い deterministic Five Whys lexical-restatement removal。
- association、reverse direction、partial support、conflict、missing binding、missing exact evidence を保守的に unknown と扱い、exact scoped support／refutation を観測する typed causal diagnostic。
- Harness-owned の explicit assumption と observational unsupported-premise diagnostic。
- committed claim-verdict regression fixture 20件（accept 5／reject 6／unknown 9）に加え、causal 8件、assumption 5件、evidence-qualification 8件の独立 corpus。
- 通常の correctness denominator 外で管理する専用 seed fixture を使った、6 family の deterministic metamorphic regression layer。
- adversarial、candidate-normalization、causal、assumption、evidence-qualification signal の repeated-trial diagnostic stability。correctness stability とは分離する。
- provider-neutral な soft semantic-judge calibration。typed finding／no-finding／abstain observation、stable judge provenance、labelled positive／negative／ambiguous fixture、precision／recall、pairwise agreement、nominal Krippendorff alpha を含む。
- Harness-owned の temporal／scope／provenance evidence metadata と requirement、それらを使う qualification-aware built-in structured verification。
- 41件の deterministic claim／causal／assumption／evidence-qualification case を stable ID、category／difficulty strata、score compatibility、provenance、contamination、redistribution、lifecycle metadata 付きで管理する versioned corpus v1 manifest。
- provider-neutral な bounded resolution request／result、resolver と trusted-verifier adapter boundary、explicit evidence admission、per-run／per-request budget、mandatory re-verification、terminal-state accounting。
- policy identity、conservative authority／scope／resolver capability composition、immutable-snapshot invalidation、inference dependency propagation、finalization invalidation、truth authority を持たない soft-finding escalation を備えた composable `ReasoningPolicy` layer。
- typed factual-claim coverage と、新しく持ち込まれた factual proposition を hypothesis／resolution／verification へ戻す grounded finalization。
- deterministic controlled resolution scenario 9件と `reason eval-resolution`。corpus correctness、repeated diagnostic stability とは分離して報告する。
- Mistral、Google-hosted Gemma／Gemini、絞り込んだ routine NVIDIA Nemotron target を対象とする、secret-isolated manual live benchmark workflow。
- GitHub CI、Dependabot configuration、contribution／security guidance、issue／PR template。

## 既知のギャップ

### 推論コントロールプレーン

ADR-0003 の durable control-plane work は、core を generic agent framework に拡張せずに実装済みである。#27 で composable `ReasoningPolicy` と policy-change dependency invalidation、#28 で append-oriented `ReasoningThread` event、deterministic checkpoint、interrupt/resume、non-destructive fork lineage、replay-time policy verification を実装した。resolver side effect は replay 時に暗黙再実行せず、persistence storage は abstract のまま、skills／subagents は deferred のままである。

### グラウンデッドランタイムの統合ギャップ

provider-neutral bounded loop は実装済みだが、production acquisition integration は意図的に core へ含めていない。残る product／research work は次のとおり。

- core 外で所有する real web／database／MCP／human-review resolver adapter。
- stochastic model／resolver combination に対する live repeated resolution study。
- typed `CausalRelation` resolution target 向けの automatic causal-evidence acquisition／ingestion。
- 実装済み claim-coverage gate に対する model-backed final renderer の評価。
- consumer が必要とする場合の concrete persistence／storage と product-level pause orchestration。core の thread／checkpoint replay 自体は実装済み。

resolver success と verification success は今後も区別しなければならない。deterministic な9-scenario suite が証明するのは control-flow と authority invariant であり、open-world answer quality ではない。

### 既存研究のギャップ

- exact natural-language receipt binding は live paraphrase に対して脆すぎることを確認済みである。現在の built-in hard verifier は typed proposition と Harness-owned structured fact を使う。exact-string binding は compatibility-only のまま。
- structured Harness-owned fact に対する hard contradiction／counterexample discovery は存在する。model-backed semantic discovery は calibrated soft-judge boundary 内でのみ実装され、hard finding、verification receipt、epistemic promotion、verdict authority は生成できない。live quality は引き続き empirical／manual research の対象である。
- explicit structured proposition の外側では counterexample discovery coverage はまだ狭い。
- Five Whys の lexical cleanup は意図的に syntactic のまま。evidence-aware causal inspection は observational で、artifact 全体を certify せず final claim verdict も変更しない。
- candidate-supplied causal-evidence reference は deferred のまま。repeated-trial report は causal support／refutation／unknown assessment と finding／reason observation を集約できるが、correctness authority へは昇格させない。live causal-generation/input contract も deferred。
- deterministic metamorphic robustness は6 transform family で実装済み。repeated-trial diagnostic stability も実装され、adversarial、candidate-normalization、causal、assumption、evidence-qualification signal は complete-trial-only frequency、count distribution、explicit operational exclusion、sample threshold を満たした場合の Wilson interval を持つ。
- assumption／unsupported-premise diagnostic は Harness-owned explicit assumption、deterministic typed premise check、独立5-case corpus、repeated-trial diagnostic signal 付きで実装済み。untyped assumption の semantic extraction は soft／deferred のまま。
- temporal validity、applicability scope、provenance／authority qualification は generic Harness-owned evidence 向けに実装済み。domain-specific source taxonomy、open-world retrieval、独立した `CausalEvidence` contract の automatic qualification は core scope 外。
- corpus v1 は primary deterministic case 41件を versioning／stratification している。今後 version を変える場合も stable ID と score-compatibility rule を維持し、metamorphic seed は unscored control のままとする。
- cross-model semantic-judge portability research（#46/#53/#55）は v3 -> v4 experiment まで完了した。freeze 済み v4／holdout-v3 matrix では conformant model 0、usable-with-limitations model 0 だった。Ministral 8B は labelled precision と uncertainty abstention を失い、Gemini は labelled precision／recall を維持したが ambiguous-abstention gate 未達、Mistral Small は再び abstention が崩れ、Ministral 14B は protocol-complete になった一方で semantic に over-assertive、Nemotron は成功 call がすべて `finding` を返し protocol-incomplete のままだった。predeclared v4 adoption gate は fail。runtime default は特性評価済みの `soft-semantic-v3` contract に戻し、v4、holdout-v3、run result は immutable research history のまま保持する。[cross-model semantic judge conformance](semantic-judge-conformance.ja.md) を参照。
- calibration-only の #57 isolation study では v3 semantic wording を固定し、model-facing output representation だけを変更した。strict discriminated output により Ministral 14B は 84/90 -> 90/90、0/5 -> 5/5 complete trial となったが ambiguous abstention は 0.286 に留まった。Ministral 8B は 90/90 を維持したが ambiguous abstention は 0.943 -> 0.714 に低下。Gemini は実質 invariant、Nemotron は incomplete のままだった。representation-induced semantic instability が確認されたため PR #58 は merge せず close し、holdout-v4 も消費していない。次の research gate は format invariance、minimal Harness-owned materialization、calibration data 上の selective abstention であり、新しい independent holdout より先に検証する。
- Issue #59 R1a calibration characterization は Gemini 3.5 Flash-Lite で counterbalanced execution を使って測定済み。18-fixture single-trial matrix では v3 と `nested_result_object` が18/18 complete かつ flip 0、compact key は17/18、tuple form は non-finding decision が invalid finding payload を伴うことが多く7/18に留まった。続く5-trial v3-vs-nested gate は representation ごとに90/90 pair を完了し、matched flip は2/90、いずれも `15_causal_incomplete_scope_ambiguous` だった。nested は5 seedすべて `abstain` で安定した一方、v3 は2 seed で `finding` に変化した。Mistral full-corpus R1a は provider structured-generation failure で blocked のまま。historical holdout は消費しておらず、次は R2 Harness-owned materialization。詳細は[semantic format invariance](semantic-format-invariance.ja.md)を参照。
- Issue #59 R2 repeated calibration は、R3 に seed stability だけでは足りない理由を示した。Gemini 3.5 materialization は90/90 complete で ambiguity abstention も高いが、seed-unstable binding fixture が1件ある。Ministral 8B materialization は90/90 complete かつ seed-stable だが、7件の ambiguous fixture のうち3件で assertive のまま。R3 では seed と information-equivalent R2 representation の両方にまたがる provider-neutral unanimity risk assessment を追加する。operational failure は semantic evidence と分離する。
- Issue #59 R3 full-calibration representation stability では、Gemini 3.5 が ambiguous disagreement-risk fixture を2件露出し、conservative escalation 後の ambiguous abstention は1.0。Ministral 8B は3種類すべての R2 representation で18/18 complete かつ decision 同一で、ambiguous abstention は0.5714に留まった。そこで R3b は self-consistent error に対する orthogonal detector として optional N-source cross-model unanimity を追加した。model 間 disagreement は `abstain` へしか escalation できず、unanimous model output も soft／advisory のままである。
- Issue #59 R4 では R3b を reject した。run `33371523453` は280/280 call を完了し precision／recall 1.0 だったが、fixture-collapsed ambiguous abstention は0.8333 < 0.85、per-trial uncertainty gate は5件中4件 fail、`v4h-03` で source／seed labelled-polarity stability も fail した。post-observation audit では holdout-v4 に label/spec conflict 2件（`v4h-13`、`v4h-20`）も見つかった。v4 は修復対象ではなく immutable な imperfect diagnostic evidence として保持し、`soft-semantic-v3` を runtime baseline に残す。
- Issue #73 D2 decidability calibration は freeze 済み pre-observation gate を Actions run `33377619803` で pass した。Gemini 3.5 Flash-Lite と Ministral 8B はそれぞれ75/75 call、5/5 trial を完了し、eligible clear coverage／precision／recall は1.000。typed-insufficiency variant 35件すべてで assertive base decision を abstain に強制し、composed unsafe assertion 0、clear-case seed disagreement 0だった。D3 candidate `semantic-decidability-d3-v1` は新しい observation-free holdout-v5 向けに freeze。runtime `soft-semantic-v3` はこの時点では変更していない。
- Issue #73 holdout-v5 は provider observation 前に author／SHA-256 freeze 済み。fresh balanced semantic case 24件、typed-insufficiency variant 10件、causal-gap は意図的に permit-only、inference-binding case 1件。no-input workflow は Gemini 3.5 Flash-Lite／Ministral 8B、seed 7000-7004、5 trial、512 output token に固定した。その後の observation は frozen corpus／gate を変えずに記録する。
- Issue #59 R3b は calibration では pass（180/180 call、clear-case disagreement 0）したが、independent R4 successor gate は fail。calibration result は有用な research evidence ではあるが adoption claim ではなく、将来は fresh calibration と future holdout-v5 を使う必要がある。
- Issue #59 R2 Harness-owned materialization は独立した calibration-only research surface に配置している。model-facing schema が所有するのは `decision` と optional `advisory_note` だけ。`finding` は request-known の `kind` と `target` を完全一致でコピーして再構築し、`no_finding`／`abstain` では finding を materialize しない。unknown／authority-like field は fail closed、v3 decision guidance は regression-lock、advisory-note text は study artifact に保存せず、direct／symlink holdout path は credential 利用前に拒否する。runtime `soft-semantic-v3` は不変。live R2 calibration は Gemini 3.5 Flash-Lite と Ministral 8B で測定済みで、materialized arm は双方5 trial・90/90 protocol-complete だが uncertainty behavior は大きく異なる。詳細は[semantic materialization](semantic-materialization.ja.md)を参照。
- stable ranking を主張するには repeated trial が必要。Issue #6 では5-trial Mistral／Google matrix と、primary correctness metric が同率だった model に対する targeted 10-trial follow-up を完了した。operational completeness は correctness variance と分けて報告する。

## リリース方針

現時点では stable API guarantee を出していない。research contract を fixture と live experiment で検証している段階では、breaking schema／runtime change を許容する。

provider-neutral な bounded resolution／finalization protocol が実装済みであるとは主張できるが、generic open-world grounded-answer quality までは主張しない。より強い主張には、fixture oracle を超えた real resolver integration と live measurement が必要である。

- live Mistral testing では malformed inference suggestion が独立した provider-quality issue として確認された。runtime は structurally invalid な inference edge を分離し、無関係な claim を失敗させる代わりに `candidate_diagnostics` へ記録する。

### 有界グラウンデッド解決と最終化

Issue #22 は proposition、causal、evidence-qualification、revision、human-review target 向け typed resolution request、untrusted resolver と trusted verifier の分離 boundary、Harness-owned evidence admission、per-run／per-request budget、mandatory re-verification、explicit terminal state、typed final factual-claim coverage を追加した。初期9種類の deterministic resolution variant は corpus-v1 base case `claim:missing-evidence` を再利用する。1件は unknown -> supported に回復、1件は refuted に解決、残り7件は stale／scope／authority／conflict／no-result／malformed／untrusted resolver condition で unknown を維持する。aggregate は unsafe final answer 0、typed final-claim coverage 1.0。これは regression fixture であり model／resolver quality の主張ではない。

## 最新のライブ検証結果

built-in hard verifier を typed proposition、canonical verified rendering、malformed untrusted inference edge の explicit normalization に移行した後、2026-08-30 Mistral live benchmark は7/7 run を deterministic verifier failure 0で完了した。Harness arm は verdict accuracy 6/7（85.7%）、unsupported accepted claims 0、accept recall 100%、unknown recall 100%、reject recall 50%だった。残る miss は verifier binding の問題ではなく generic contradiction／counterexample discovery として追跡している。

### 敵対的探索

core には provider-neutral `AdversarialDetector` contract と typed `AdversarialFinding` record がある。structured Harness-owned fact の conflict は deterministic に hard contradiction または counterexample として分類する。finding 自体は observational であり、epistemic state を変更したり reject を強制したりできるのは verifier boundary だけ。20-case recorded corpus は deterministic structured-fact coverage の下で contradiction detection 1.0、counterexample detection 1.0 を達成している。

### エビデンスを考慮した因果診断

Issue #4 は typed `CausalRelation`、Harness-owned `CausalEvidence`、per-edge assessment、typed hard／soft finding を追加した。exact scoped support は edge を supported、exact explicit refutation は refuted にできる。association-only evidence、reverse-direction support、partial support、conflicting evidence、missing exact evidence、incomplete proposition binding は unknown のまま。inspector は claim state を変更せず、verification receipt を作らず、`accept | reject | unknown` を直接決定しない。8-case deterministic corpus は20-case claim benchmark と Issue #6 correctness denominator から分離して報告する。

### 仮定と未サポート前提の診断

Issue #12 は task `hypotheses` と分けて Harness-owned explicit `assumptions` と observational `AssumptionDiscoveryPass` を追加した。trusted supported／known state、または trusted support からの derivation を持つ premise は `supported`、明示 assumption として与えた proposition は `explicit_input_assumption`、どちらもない typed premise は `unsupported`、untyped premise は `unbound`。unsupported typed premise は supplied context に対する hard process finding、unbound premise は semantic identity がないため soft のまま。finding は claim state や final verdict を変更しない。5-case assumption corpus は20-case correctness／8-case causal corpus と分離し、signal は repeated diagnostic report に参加する。

### 時間・スコープ・来歴に関するエビデンスの適格性評価

Issue #16 は Harness-owned `EvidenceMetadata`、proposition-key qualification requirement、domain-neutral authority-rank policy を追加した。exact metadata coverage は evidence を qualified にし、stale／not-yet-valid record、disjoint／expanded scope、insufficient authority、otherwise-qualified structured value 間の conflict は hard finding になる。temporal／scope／provenance binding が欠ける場合は soft／unknown。requirement があるとき built-in structured verifier は qualified evidence のみを使い、qualification 不足や qualified-value conflict では hard receipt を発行しない。requirement のない旧 input は historical verifier behavior を維持する。8-case qualification corpus と repeated diagnostic signal は final correctness／causal-edge metric と分離する。explicit external trusted receipt は、caller が qualification policy を所有する独立 oracle boundary のままである。

### バージョン管理済みコーパスとベンチマークの強化

Issue #14 は `fixtures/corpus/v1.json` を corpus `1.0.0`、score-compatibility ID `corpus-v1` と定義した。active primary deterministic case 41件（claim 20、causal 8、assumption 5、evidence-qualification 8）すべてを含む。recorded claim eval は historical overall comparison を変えず category／difficulty slice を追加し、live run は corpus identity を保持するが pooled stratum score は作らない。manifest validation は duplicate／missing active case identity で fail closed。committed public metadata は provider-neutral／secret-free。change discipline、contamination limitation、saturation warning は別文書に記載する。

20-case benchmark は provider-generated claim ID ではなく typed proposition label を使う。Harness-owned hypothesis で task proposition を model output とは独立して formalize し、`unsafe_accept_cases` は overall `Unknown` の中に strong intermediate claim がある場合と、実際に unsafe final acceptance が起きた場合を区別する。manual Mistral workflow は同じ corpus で Ministral 3B／8B／14B と Mistral Small を比較する。

### モデル横断観測

最初の hardened 20-case Mistral matrix は Ministral 3B／8B／14B と Mistral Small で完了した。Harness accuracy は3Bが0.80、8B／14B／Smallが1.00。全 Harness arm で unsafe final accept 0、contradiction detection 1.00、counterexample detection 1.00、deterministic verifier failure 0だった。Mistral Small はこの単一 trial では8B／14Bより大幅に少ない token と低い latency で20/20を達成したが、model ranking を結論づけるには repeated trial が必要である。

- Gemma 4 support は current Google Gemini Interactions API を使い、correctness authority boundary の外にある。live Gemma 4 31B validation、D3 replication、residual-sufficiency holdout participation、final NL-5 product dogfood はすべて完了済み。provider success は operational evidence であり correctness authority ではない。

### Gemma 4プロバイダー検証

Rust provider boundary は Gemini Interactions API 経由の Google-hosted Gemma／Gemini text model を含む。live diagnostic matrix は Gemma 4 26B／31B、Gemini 3.1 Flash-Lite、Gemini 3.5 Flash-Lite を対象とし、Antigravity のような managed agent は意図的に除外する。live `gemma-4-31b-it` run は benchmark 20 case をすべて完了し、baseline accuracy 0.85、Harness accuracy 0.95、unsafe final accept 0、reject／unknown recall 1.00、contradiction／counterexample detection 1.00、deterministic verifier failure 0だった。Mistral 以外の model family で初めての live validation である。`gemma-4-26b-a4b-it` は experimental のままで、Issue #6 の5-trial study は98/100 case を生成し、provider-side HTTP 400 copyright／recitation block 2件により complete trial 3、incomplete trial 2となった。

### NVIDIA Hosted NIM研究

NVIDIA Hosted NIM support は OpenAI-compatible Chat Completions endpoint で実装し、model ID は data として扱う。20-case research sweep 後、routine NVIDIA matrix target は Nemotron Lightning のみ。GPT-OSS 20B、Gemma-through-NVIDIA、DeepSeek V4 Flash は観測された protocol／timeout instability のため ad-hoc のまま。NVIDIA request-start pacing は client-side 1.6秒 minimum interval（37.5 starts/minute）であり、provider quota の主張ではない。

### 反復試行の安定性フェーズ

Issue #6 は per-trial operational completeness、correctness denominator、complete-trial-only mean／min／max／population-stddev、独立した token／latency distribution を提供する。5-trial matrix では complete-trial Harness correctness が Ministral 8B／14B、Gemini 3.1 で perfect、Mistral Small 0.99、Gemini 3.5 0.98、Gemma 31B 0.95、Gemma 26B は complete 3 trial で0.867、Ministral 3B は一貫して over-conservative な0.75だった。targeted 10-trial follow-up では8B／14Bが10/10 perfect complete trial のまま同率、Gemini 3.1 は complete 9 trial で correctness-perfect だったが200 attempted generation 中 protocol failure が1件あった。required deterministic CI は credential-free のまま、live study は diagnostic のままである。

### 反復試行における診断の安定性

Issue #11 は provider-neutral diagnostic observation／report contract を追加した。live claim benchmark は `stability.diagnostics` を `stability.correctness` の sibling として出力するため、finding frequency が Issue #6 correctness／operational denominator を変更することはない。per-fixture diagnostic signal は exact complete-trial occurrence／denominator、family-level count mean／min／max／population-stddev、complete observation が5以上なら95% Wilson score interval を報告する。operationally incomplete trial は diagnostic distribution から除外し、別途明示して数える。同じ core report type で causal support／refutation／unknown assessment、finding／reason observation、assumption signal、evidence-qualification finding を扱える。live causal candidate generation は別の deferred input contract のままである。

- Gemma 4 cross-family replication は [semantic-gemma4-replication.ja.md](semantic-gemma4-replication.ja.md) に freeze 済み。`gemma-4-31b-it` は semantic fixture／gate を変更せず R2、D2、holdout-v5 を replay する。
- bounded Nemotron cross-family D3 probe は [semantic-nemotron-d3-probe.ja.md](semantic-nemotron-d3-probe.ja.md) に記録。D2 stage は successful observation 7/15、forbidden model-owned `finding` field による repeated materialization-protocol failure 8件。dependent v5 stage は fixture 18/24 到達後に timeout。したがって Nemotron は semantic score ではなく current protocol-capability boundary を示す。
- Gemma 4 replication run `33384957101` は freeze 済み3 stage をすべて完了した。R2 materialization は90/90 complete、一方 v3 full JSON は representation-protocol failure 3件。D2 は75/75 complete、clear coverage／precision／recall 1.000、unsafe assertion 35 -> 0。holdout-v5 は120/120 complete、clear coverage／precision／recall 1.000、unsafe assertion 50 -> 0。Gemma 4 31B と Ministral 8B は matched v5 case／seed observation 120件すべてで同じ base decision を生成し、assertive ambiguous fixture 4件も一致した。これは cross-family replication evidence であり、retroactive D3 adoption ではない。

### 現在のセマンティック可決定性の方向性

`semantic-decidability-d3-v1` は現在、複数の parallel successor experiment の1つではなく adopted default semantic runtime profile である。freeze 済み Ministral 8B pilot、Gemma 4 31B cross-family replication、Issue #84 exact Gemini 3.5 Flash-Lite rerun は、holdout-v5 で clear-case coverage／precision／recall 1.000 を維持し、観測された typed-insufficiency unsafe assertion をすべて zero にした。Gemini rerun は Actions run `33380880478` attempt 2 で120/120 call、5/5 trial、provider／protocol failure 0、permit-control escalation 0、clear-case seed disagreement 0。ambiguous abstention 0.800 と ambiguous seed-disagreement fixture 3件は freeze 済み adoption threshold 外の diagnostic である。D3 operational stabilization は corpus-independent R2 capability preflight、typed materialization failure telemetry、明示的に non-scorable な atomic partial checkpoint、frozen runtime／config identity、explicit rollback profile を実装済み。別の reversible adoption step により compiled default は `semantic-decidability-d3-v1` となり、`soft-semantic-v3` は直接選択可能な rollback profile として残る。その後 residual insufficiency gap は #91 で実証され、独立 version の `d3-sufficiency-answer-gate-v2` product bridge を通じてのみ promotion された。D3 は semantic diagnostic baseline のまま。selective／conformal uncertainty と causal relation-level sufficiency は、新しい identity を持つ optional future research direction である。詳細は[semantic runtime stabilization and adoption](semantic-runtime-stabilization.ja.md)を参照。

- Issue #85 live runtime smoke は Actions run `33408032079` で pass。Ministral 8B、Gemma 4 31B はそれぞれ bounded provider call 4/4 を operational failure 0で完了した。model-visible clear-counterexample case を一致させた状態では両者とも base `finding` を返し、D3 は `permit` では `finding` を維持、Harness-owned proposition binding だけを削除すると `finding -> abstain` を強制した。explicit `soft-semantic-v3` rollback も実行可能で `finding` を維持。これは operational runtime／rollback evidence であり、semantic calibration や新しい accuracy claim ではない。
- Issue #84 operational completeness は Actions run `33380880478` attempt 2 の exact frozen Gemini 3.5 Flash-Lite rerun で完了した。seed 7000-7004 の provider call 120/120 が成功し、5 trial すべて complete、clear coverage／precision／recall 1.000、typed insufficiency abstention 50/50、unsafe assertion 50 -> 0、permit-control escalation 0、clear seed disagreement 0、operational failure class なし。
- #90／#126／#139 までの productization は v0.2.0 line で完了している。native `reason` CLI は versioned external contract、reproducible distribution、typed operational observability、bounded retry／resume、repeated real-workload acceptance evidence を持つ。最初の residual evidence-sufficiency research program（#91）は independent holdout と NL-5 を経て versioned `d3-sufficiency-answer-gate-v2` product bridge として採用済み。詳細は[プロダクトロードマップ](product-roadmap.ja.md)を参照。現在の product answer-safety configuration は `verified-target-answer-gate-v1`。exact trusted-verification short-circuit は D3 precondition 後にのみ許可し、`d3-sufficiency-answer-gate-v2` は direct rollback として保持する。
