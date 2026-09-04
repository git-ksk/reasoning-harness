# 研究計画

## 仮説

決定論的な reasoning harness は、candidate generation に小型または低コストの model を使う場合でも、unsupported claim と hidden assumption を減らせる。

次の product-level hypothesis はさらに強い。harness が candidate が unresolved である正確な理由を特定できれば、bounded resolution と re-verification loop によって、unsupported final answer を増やさずに grounded で answerable な case をより多く回復できる。

したがって harness は diagnostic system と、将来の evidence-grounded reasoning runtime の両方として評価する。diagnosis quality は必要条件だが、最終的な product test は、verified intermediate state が追加の evidence acquisition、repair、finalization を安全に制御できるかである。

## プロダクトと研究の分離

採用済み D3 runtime は stable semantic baseline のままである。最初の residual-sufficiency successor program (#91) は、RSD0/RSD1/RSD2、independent frozen holdout、operational stabilization、versioned product wiring、NL-5 を、model output に authority を与えず完了した。

以前の分離を動機づけた product follow-up line は完了し、v0.3.0 は新しい reasoning study ではなく integration/productization phase である。

- **Completed product CLI (#90):** external contract hardening、install/release compatibility、process-level observability、documented readiness gate が完了。
- **Completed provider reliability (#126):** bounded transient retry、provider-attempt telemetry、exact-identity product evaluation resume が、historical research outcome を書き換えず完了。
- **Completed product utility (#139):** current successor product rerun が、authority boundary を維持したまま six-case product workload の Ministral 8B coverage/withholding gap を閉じた。
- **Current release:** v0.3.0 は既存の research foundation 上に external-evidence/resolution capability を package し、新しい research generation は作らない。
- **Completed product milestone (#173):** v0.3.0 は real external acquisition と trusted verifier/oracle adapter を既存の bounded-resolution boundary に接続した。#174/#175/#178 が lane を確立し、#176 が read-only MCP acquisition、#177 が hard external verification、#179 が non-frozen open-world acceptance evidence、#180 が research identity を変更しない optional downstream `reason-mcp` product surface を追加した。
- **Follow-on research:** selective/conformal abstention、relation-level causal sufficiency、その他の機構は、新たに測定した gap が新しい research identity と fresh evaluation sequence を正当化する場合にのみ開始する。

Research は決して直接 ship しない。future candidate は fresh calibration、adoption 用の independently frozen holdout、operational stabilization、explicit profile/rollback、CLI compatibility coverage を再び通過してから、別の可逆的 product-adoption change とする。

[native CLI product roadmap](product-roadmap.ja.md) を参照。

## 初期実験

### E1 — 来歴管理の規律

意図的に evidence が欠けた fixture で、直接の model answer と harness を通した answer を比較する。

Primary metric: unsupported accepted claims。

### E2 — 明示的な不確実性

それ以外は answerable な fixture から required fact を除く。

Primary metric: fabricated completion ではなく、正しい `unknown` classification。

### E3 — フレームワーク構造

prose-only 5 Whys と、evidence reference を要求する typed causal link を比較する。

Primary metrics: unsupported causal edge、restated symptom、root-cause mismatch。

### E4 — モデル置換

low-cost/free candidate generator を含む複数の model adapter に同一 fixture を通す。

Primary question: harness は output quality variance をどの程度吸収するか。

### E5 — セマンティック保持

verified artifact を段階的に簡略化した explanation に変換し、invariant の脱落や unsupported addition を検出する。

## 次の実験

### E6 — 有界解決の復旧 — 決定論的ベースライン実装済み

初期の controlled nine-scenario suite は、required evidence が欠けるか qualification 不足で意図的に `unknown` になる fixture から始める。missing evidence、refuting fact、no result、malformed/untrusted data を返せる controlled resolver を runtime に与える。

比較対象:

- direct one-shot generation;
- diagnose-only harness execution;
- bounded resolution plus re-verification。

Primary metrics は initially-unknown case recovery rate、unsafe final answer rate、supported/refuted/exhausted terminal distribution、resolution attempt と追加 token/latency/tool cost、trusted resolution がない場合の `unknown` 保持である。answer rate の増加は unsupported final claim の増加を伴えば成功ではない。

### E7 — グラウンデッド最終化のカバレッジ — コアゲート実装済み

finalization contract は renderer に verified artifact を与え、final prose が artifact の supported proposition set 内に収まるかを検査する。正確に paraphrase、重要な qualification の omission、もっともらしい新 fact の導入、uncertainty の certainty への変換を行う adversarial renderer を含める。

Primary metrics: supported artifact proposition に対する factual final-claim coverage、final output に到達した unsupported addition、uncertainty/qualification preservation、新しく導入された factual proposition の verification への正しい routing。

### E8 — 解決中のエビデンス適格性評価 — 決定論的ベースライン実装済み

controlled resolution suite は、real だが stale、wrong-scope、insufficient-authority、conflicting、not-yet-valid な evidence を返す resolver を含む。

Primary question: resolution loop は false closure を拒否し、新しく得た evidence が required qualification を実際に満たす場合を除いて `unknown` を保つか。

## 計画中のランタイム機能

- [implemented] temporal、scope、provenance evidence qualification (#16)
- [implemented] stable base-case identity を持つ versioned/stratified benchmark corpus v1 (#14)
- [implemented] provider-neutral typed resolution request
- [implemented] bounded per-run/per-request resolution attempt/token/time policy
- [implemented contract] resolver adapter は correctness authority boundary の外に置き、concrete domain adapter は deferred
- [implemented] candidate repair/regeneration と mandatory re-verification
- [implemented] grounded finalization と factual claim coverage check
- [implemented] composable reasoning policy/invalidation (#27)
- [implemented] durable typed ReasoningThread checkpoint/resume/fork replay (#28)
- [implemented] grounded、qualified、refuted、exhausted、unavailable、human-review、unresolved、abstain を含む explicit resolution/finalization terminal state
- [implemented] policy-change dependency/finalization invalidation を備えた composable reasoning policy (#27)
- durable reasoning thread、checkpoint/resume/fork、deterministic replay (#28)
- [implemented] calibrated soft semantic-judge contract と offline reliability metric (#13); live semantic discovery は optional/manual のまま

## 計画中の診断・推論作業

- [implemented #33] calibrated soft-only boundary 下での repeated live semantic contradiction/unsupported-premise/causal-gap discovery と explicit ambiguity-abstention measurement
- [implemented #36] expanded ambiguity/counterexample coverage を持つ frozen independent semantic-judge holdout v1。frozen corpus を変更せず Mistral の first five-trial study を完了
- [implemented #38] `soft-semantic-v3` calibration と independent holdout-v2 study。five-trial holdout precision/recall 1.000、mean coverage 0.700、ambiguous abstention 0.933
- [implemented #46/#53/#55] cross-model semantic conformance。v4 は frozen holdout-v3 の portability gate に失敗し、conformant/usable model は zero。runtime baseline は `soft-semantic-v3` に復元
- [calibration result #57] strict discriminated output は v3 semantic wording から分離され、Ministral 14B の protocol failure を修正したが Mistral の uncertainty behavior を変えた。model-facing structured output は semantically neutral と仮定できないため PR #58 は merge せず終了
- [calibration result #59] R1a format invariance は Gemini 3.5 Flash-Lite で characterization 済み。counterbalanced five-trial v3-vs-nested run は representation ごとに 90/90 pair、format flip 2/90。Mistral full-corpus R1a は operationally blocked
- [calibration result #59] R2 materialization は model-owned `decision` と optional `advisory_note` のみを公開し、`finding` の場合だけ request-known `kind`/`target` を deterministic にコピーする。Gemini 3.5 Flash-Lite と Ministral 8B で五 trial 90/90 protocol-complete
- [calibration result #59] R3b Gemini 3.5 Flash-Lite + Ministral 8B は five seeds で 180/180 calibration call を完了。disagreement は ambiguous fixture 4件に限定され、disagreement-only policy は precision/recall 1.0、ambiguous abstention 1.0、clear-case coverage 1.0
- [rejected #59] independent holdout-v4 run `33371523453` は 280/280 call を完了したが frozen uncertainty gate と source/seed labelled-polarity stability に失敗。R3b は採用しない
- [frozen diagnostic #59] post-observation audit は `v4h-13` と `v4h-20` の label/spec conflict を発見。holdout-v4 は変更せず tuning data にしない
- [designed #73] deterministic semantic decidability/evidence-sufficiency と final soft decision を分離。explicit missing binding または unsatisfied typed evidence requirement は `abstain` のみを強制し、`permit` は correctness evidence ではない
- [implemented #73] deterministic calibration は target binding、evidence presence、temporal/scope/authority qualification、required metadata、qualified-evidence conflict をまたぐ7 matched control/mutation pair、14 fixture。causal-gap は permit-only
- [designed #73] D2 は semantic polarity と assertive eligibility を分離し、eligible clear case のみで precision/recall を計算。typed-insufficiency abstention と unsafe-assertion reduction は別 denominator
- [implemented #73] D2 v1 は既存15 calibration semantic case を typed-eligibility manifest/runner に解決。clear case 7件には paired typed-insufficiency mutation、ambiguous case 4件は別 denominator の eligible control
- [implemented #73] D2 runner は provider initialization 前に source label と deterministic gate expectation を検証し、matched variant 間で unchanged R2 observation を再利用。provider/protocol failure は semantic abstention ではなく operationally incomplete
- [frozen #73] D2 v1 first-observation plan は15-case corpus、Gemini 3.5 Flash-Lite/Ministral 8B、seeds 6000-6004、five trials、512 output tokens、predeclared gate を固定
- [calibration result #73] frozen D2 run `33377619803` は両 provider で全 gate を通過。75/75 call、5/5 complete trial、eligible clear coverage/precision/recall 1.000、35/35 typed insufficiency abstention、unsafe assertion 35 -> 0、clear seed disagreement zero
- [frozen #73] D3 candidate `semantic-decidability-d3-v1` は `soft-semantic-v3` と R2 model semantics を変えず、deterministic explicit-typed-preconditions gate のみを compose
- [frozen #73] fresh holdout-v5 は observation-free で静的 review 済み。24 semantic case、10 typed-insufficiency variant、one inference-binding case、SHA-256-frozen source/manifest payload
- [frozen #73] independent execution plan は Gemini 3.5 Flash-Lite と Ministral 8B、seeds 7000-7004、five trials、512 output tokens、predeclared D3 adoption gate を固定
- [implemented stabilization #73] D3 は frozen runtime/config identity、corpus-independent R2 capability preflight、typed telemetry、atomic partial-result checkpoint、明示的 `soft-semantic-v3` rollback profile を持つ
- [adopted #73] stabilization CI 後の別 reversible runtime PR が compiled default を D3 に切り替え、historical D2/v5 workflow は不変、`soft-semantic-v3` は rollback として保持
- [next semantic research #73] D3 adoption 後は deterministic typed metadata が残した measured gap に対する residual soft decidability を検証。selective/conformal abstention は後続 option、causal relation-level sufficiency は typed directional relation evidence が整うまで deferred

## 評価原則

単一の judge-model score に最適化しない。measurable protocol property、golden/adversarial fixture、external oracle、明示的 authority boundary を優先し、model-judge metric は soft evidence と明示する。grounded runtime では raw answer rate でも maximum abstention でもなく、grounded answerability と unsafe final output の trade-off を最適化する。すべての recovery metric に unsafe-final-answer または final-claim-coverage metric を組み合わせる。

## ベンチマーク手法

最初の benchmark は naive baseline と harness arm の間で generated candidate を固定し、差が別の model sample ではなく deterministic process から生じるようにする。recorded candidate は CI regression fixture に限り、empirical claim には live repeated provider run が必要である。[benchmark design](benchmark.ja.md) を参照。

benchmark は false acceptance と trivial over-conservatism の双方を罰する。したがって verdict accuracy と per-class accept/reject/unknown recall を unsupported accepted claim と併記する。

future resolution-loop benchmark は one-shot、diagnose-only、bounded-resolution variant 間で corpus-v1 の stable base-case identity を保つ。recovered case が correctness denominator を黙って置換してはならない。operational failure、resolution exhaustion、missing resolver coverage は correctness と別に報告する。

## Oracle管理回帰テストとオープンワールド研究

fixture oracle receipt は expected hard result が意図的に既知の場合だけ使う。harness が authority を正しく消費し、model に authority を与えないことを検証する。open-world contradiction discovery、counterexample generation、semantic causal evaluation、retrieval quality は別の research problem として別計測する。

resolver が document を見つけたことは、要求された proposition を oracle が証明したことと同値ではない。resolution experiment は acquisition success と verification success を明示的に区別する。

[ADR-0002](adr/0002-grounded-resolution-and-finalization.ja.md) は、これらの experiment が検証対象とする runtime authority と finalization boundary を定義する。
