# 研究計画

この文書は、Reasoning Harness の研究を「何を確かめたいのか」「どこまで確認できたのか」「次に何を研究するのか」の順で追えるようにまとめたものです。

細かい Issue 番号や実験結果も残しますが、まずはこの3点だけ押さえれば全体像をつかめます。

1. **候補回答を作る AI は信用しすぎない。** 正しさの判断は Harness 側の evidence、qualification、verification が持つ。
2. **分からないときは `unknown` / `abstain` を維持する。** 回答率を上げるために根拠不足を隠さない。
3. **研究成果は、独立 holdout と運用安定化を通るまで製品に入れない。** 研究結果と product release を分離する。

## 現在地

現在の semantic runtime は **`semantic-decidability-d3-v1`** です。これは D3 の研究結果を operational stabilization まで通したうえで採用したものです。以前の **`soft-semantic-v3`** は rollback profile として残しています。

その後の residual-sufficiency 研究 (#91) では、RSD0 → RSD1 → RSD2 → 独立 holdout → operational stabilization → product wiring → NL-5 まで完了しました。ここでも model output 自体に correctness authority は与えていません。

現時点では、**「次の semantic successor を作ること」自体が目的ではありません。** 新しい研究を始めるのは、D3 と現在の answer-safety gate で説明できない具体的な failure/gap が新たに観測されたときです。

次の候補として残しているのは、主に以下です。

- selective / conformal abstention
- causal relation 単位の evidence sufficiency
- 新しい形式の semantic instability への対策

ただし、いずれも **新しい gap が実測され、fresh evaluation を用意できる場合にのみ開始**します。

## 研究と製品は分けて進める

現在の product follow-up line は完了しています。v0.3.0 は新しい reasoning study ではなく、既存の研究成果を外部 evidence / resolution へ接続した integration / productization release です。

- **#90 完了:** CLI の外部contract、install / release compatibility、process-level observability、readiness gate を整備
- **#126 完了:** bounded transient retry、provider-attempt telemetry、exact-identity resume を追加。過去の研究結果は変更していない
- **#139 完了:** six-case Ministral 8B product workload で coverage / withholding gap を再検証し、authority boundary を保ったまま改善を確認
- **#173 完了:** v0.3.0 milestone。#174 / #175 / #178 で external-resolution lane、#176 で read-only MCP acquisition、#177 で trusted external verification、#179 で non-frozen open-world acceptance、#180 で optional `reason-mcp` surface を追加

つまり、**研究の成功 = 即リリース**ではありません。研究は研究 identity のまま凍結し、製品化は別の変更として行います。

## 何を研究しているのか

中心の仮説はシンプルです。

> 小型・低コストの model が候補回答を作っても、Harness が evidence と検証を管理すれば、unsupported claim や hidden assumption を減らせるのではないか。

さらに一歩進めた product-level hypothesis は次です。

> 「なぜ今は答えられないのか」を Harness が正確に特定できれば、必要な evidence だけを追加取得して再検証し、unsafe な回答を増やさずに答えられるケースを増やせるのではないか。

そのため、Reasoning Harness は単なる「回答採点器」ではなく、最終的には **evidence-grounded reasoning runtime** として評価します。

## 研究成果を製品へ入れる条件

研究成果は、そのまま product に入れません。新しい semantic candidate は、原則として次の順序を通ります。

1. calibration で仮説を確認する
2. calibration に使っていない fresh holdout を先に凍結する
3. holdout で事前定義した gate を満たすか確認する
4. operational failure、checkpoint、telemetry、rollback を整える
5. runtime/profile identity を固定する
6. CLI / product compatibility を確認する
7. 別の可逆な product change として採用する

この手順を通らない研究結果は、たとえ数値が良くても「採用候補」ではなく研究記録として扱います。

現在の product 側の詳細は [プロダクトロードマップ](product-roadmap.ja.md) を参照してください。

## これまでの大きな流れ

研究の流れを短くまとめると、次のようになります。

- **E1〜E5:** provenance、uncertainty、causal structure、model差、semantic preservation の基礎を確認
- **E6〜E8:** `unknown` からの bounded recovery、grounded finalization、evidence qualification を実装
- **soft semantic judge:** hard verifier にできない semantic signal を、authority を持たない補助観測として研究
- **R1〜R4:** 出力形式や seed/model 差による semantic instability を調査。R4 successor は holdout gate に失敗して不採用
- **D1〜D3:** 「model の semantic decision が正しいか」ではなく、「assertive な decision を許可してよい条件」を Harness 側で判定
- **RSD0〜RSD2 / #91:** D3 後に残った evidence-sufficiency gap を独立研究し、別の answer-safety bridge として昇格

## 初期実験

### E1 — provenance の規律

**目的:** evidence が足りないとき、model がもっともらしい情報を補ってしまわないか確認する。

意図的に evidence が欠けた fixture で、直接の model answer と Harness を通した answer を比較します。

**主指標:** unsupported accepted claims

### E2 — 不確実性を明示できるか

**目的:** 答えに必要な fact がない場合、無理に埋めず `unknown` を選べるか確認する。

answerable な fixture から required fact を1つだけ取り除きます。

**主指標:** fabricated completion ではなく、正しい `unknown` classification

### E3 — causal structure を型として扱う

**目的:** prose だけの Five Whys より、evidence reference を持つ typed causal link の方が誤った因果を抑えられるか確認する。

**主指標:** unsupported causal edge、restated symptom、root-cause mismatch

### E4 — model を入れ替えても Harness が効くか

**目的:** model 性能が変わっても、Harness が output quality の差をどこまで吸収できるか確認する。

low-cost / free candidate generator を含む複数の model adapter に同じ fixture を通します。

### E5 — verified artifact を説明文へ変換しても意味を壊さないか

**目的:** 検証済み artifact を人間向けの簡潔な説明へ変換するとき、重要な invariant が落ちたり、未検証の fact が増えたりしないか確認する。

## Grounded runtime の実験

### E6 — `unknown` から安全に回復できるか — 決定論的ベースライン実装済み

**目的:** 足りない evidence を bounded に取りに行き、答えられる場合だけ `unknown` から回復できるか確認する。

controlled nine-scenario suite は、required evidence が欠けるか qualification 不足で `unknown` になるケースから始まります。resolver は missing evidence、refuting fact、no result、malformed / untrusted data などを返せます。

比較するのは次の3方式です。

- one-shot generation
- diagnose-only Harness
- bounded resolution + re-verification

**主指標:** initially-unknown case recovery rate、unsafe final answer rate、supported / refuted / exhausted の終端分布、resolution attempt 数、追加 token / latency / tool cost

回答数が増えても unsupported final claim が増えるなら成功とはみなしません。

### E7 — 最終回答に未検証の fact が混ざらないか — コアゲート実装済み

**目的:** verified artifact から final prose を生成するとき、renderer が勝手に事実を増やしたり、重要な留保を消したりしないか確認する。

adversarial renderer では、次のような崩し方を試します。

- 正しい内容の paraphrase
- 重要な qualification の omission
- もっともらしい新 fact の追加
- uncertainty を certainty に変換

**主指標:** factual final-claim coverage、unsupported addition、uncertainty / qualification preservation、新しい proposition の verification への再 routing

### E8 — 取得できた evidence が「使ってよい evidence」か判定できるか — 決定論的ベースライン実装済み

**目的:** resolver が何かを返しただけで回答を確定せず、freshness / scope / authority まで満たしたときだけ使えるか確認する。

controlled suite には stale、wrong-scope、insufficient-authority、conflicting、not-yet-valid な evidence を含めます。

**主質問:** required qualification を満たしていない evidence では `unknown` を維持できるか。

## ランタイム側で実装済みの基盤

- temporal / scope / provenance evidence qualification (#16)
- stable base-case identity を持つ versioned / stratified benchmark corpus v1 (#14)
- provider-neutral typed resolution request
- bounded per-run / per-request resolution attempt・token・time policy
- resolver adapter を correctness authority boundary の外に置く contract
- candidate repair / regeneration と mandatory re-verification
- grounded finalization と factual-claim coverage check
- composable `ReasoningPolicy` と invalidation (#27)
- durable typed `ReasoningThread`、checkpoint / resume / fork、deterministic replay (#28)
- grounded / qualified / refuted / exhausted / unavailable / human-review / unresolved / abstain を区別する terminal state
- calibrated soft semantic-judge contract と offline reliability metric (#13)

なお、concrete な web / database / MCP / human-review resolver adapter は core runtime の correctness authority にはしません。

## Semantic research の詳細履歴

ここから下は、semantic 系研究の「何を試し、何を採用しなかったか」を追うための記録です。現在地だけ知りたい場合は、上の「現在地」までで十分です。

### soft semantic judge

- **#33 完了:** contradiction / unsupported-premise / causal-gap を soft-only boundary で観測。ambiguity abstention も別指標として測定。
- **#36 完了:** expanded ambiguity / counterexample を含む independent holdout v1 を freeze。Mistral の最初の five-trial study を完了。
- **#38 完了:** `soft-semantic-v3` calibration と independent holdout-v2。five-trial holdout で precision / recall `1.000`、mean coverage `0.700`、ambiguous abstention `0.933`。
- **#46/#53/#55 完了:** cross-model conformance を評価。v4 は frozen holdout-v3 の portability gate に失敗し、conformant / usable model は0。runtime baseline は `soft-semantic-v3` に戻した。

ここで分かった重要なことは、**model-facing output format 自体も semantic behavior を変えうる**という点です。

### R1〜R4 — semantic instability の調査

- **#57:** strict discriminated output は Ministral 14B の protocol failure を改善した一方、Mistral の uncertainty behavior を変えた。PR #58 は未採用で終了。
- **R1a / #59:** Gemini 3.5 Flash-Lite で format invariance を評価。five-trial v3-vs-nested は representation ごとに 90/90 pair、format flip は 2/90。Mistral full-corpus は operationally blocked。
- **R2 / #59:** model には `decision` と optional `advisory_note` だけを返させ、`finding` の `kind` / `target` は Harness が request から deterministic に再構成。Gemini 3.5 Flash-Lite と Ministral 8B で 90/90 protocol-complete。
- **R3b / #59:** Gemini 3.5 Flash-Lite + Ministral 8B の cross-model disagreement を risk signal として評価。calibration では 180/180 call、precision / recall 1.0、ambiguous abstention 1.0、clear-case coverage 1.0。
- **R4 / #59:** independent holdout-v4 run `33371523453` は 280/280 call を完了したが、frozen uncertainty gate と source/seed labelled-polarity stability に失敗。**R3b successor は不採用。**
- 観測後 audit で `v4h-13` と `v4h-20` に label/spec conflict を確認。holdout-v4 は修正せず、imperfect diagnostic evidence として凍結したまま残す。

### D1〜D3 — 「正しいか」ではなく「assertive に答えてよいか」を判定

R4 の失敗を受けて、研究の問いを変えました。

それまで:

> model の semantic decision をより正しくできるか？

D 系列:

> Harness が持つ typed evidence / binding 情報から、assertive な semantic decision を許可してよい条件を deterministic に判定できるか？

この変更により、`permit` は correctness evidence ではなく、**「assertive decision を禁止する理由が Harness 側で見つからなかった」ことだけを意味する**ようにしました。

- **#73 D1:** target binding、evidence presence、temporal / scope / authority qualification、required metadata、qualified-evidence conflict を使う deterministic gate を設計。causal-gap は permit-only。
- **#73 D2:** semantic polarity と assertive eligibility を分離。eligible clear case だけで precision / recall を測り、typed-insufficiency abstention は別 denominator にした。
- D2 v1 は15 semantic caseを使用し、clear case 7件に paired typed-insufficiency mutation、ambiguous case 4件を別 denominator で評価。
- frozen first-observation plan は Gemini 3.5 Flash-Lite / Ministral 8B、seeds 6000–6004、5 trials、512 output tokens。
- run `33377619803` は両 provider で全 gate 通過。75/75 call、5/5 complete trial、eligible clear coverage / precision / recall `1.000`、typed insufficiency abstention 35/35、unsafe assertion 35 → 0、clear seed disagreement 0。
- **D3 candidate `semantic-decidability-d3-v1`:** `soft-semantic-v3` と R2 model semantics を変えず、deterministic typed-preconditions gate だけを compose。
- fresh holdout-v5 は observation 前に freeze。24 semantic case、10 typed-insufficiency variant、1 inference-binding case。
- D3 stabilization では runtime/config identity、R2 capability preflight、typed telemetry、atomic partial-result checkpoint、rollback profile を実装。
- stabilization 後の別PRで D3 を compiled default に採用し、`soft-semantic-v3` を rollback として保持。
- **Gemma 4 cross-family replication:** run `33384957101` で `gemma-4-31b-it` が R2、D2、holdout-v5 を完走。D2 / v5 とも clear coverage / precision / recall `1.000`、typed-insufficiency abstention `1.000`、composed unsafe assertion 0。v5 の120 matched case/seedでは Gemma 4 と Ministral 8B の base decision が全件一致。これは cross-family generalization の裏付けであり、過去の D3 provider set を後から変更するものではない。
- **#84 完了:** quota reset 後の Gemini 3.5 Flash-Lite frozen holdout-v5 rerun、Actions run `33380880478` attempt 2 は 120/120 call、5/5 trial、clear coverage / precision / recall `1.000`、typed-insufficiency abstention 50/50、unsafe assertion 50 → 0、permit-control escalation 0、clear-case seed disagreement 0、provider / protocol failure 0。ambiguous abstention は 32/40 で、disagreement は adoption threshold 外の3 ambiguous fixture に限定。
- **#39 実装済み:** structured-output fallback の原因を provider-neutral な typed telemetry で分類し、model comparison と semantic result から operational fallback を分離。

## D3 後の研究

D3 採用後も、typed metadata だけでは説明できない residual evidence-sufficiency gap が残りました。この gap は #91 の RSD0 / RSD1 / RSD2 で別研究として扱い、独立 holdout と product validation を通したうえで `d3-sufficiency-answer-gate-v2` に昇格しました。

重要なのは、D3 自体を後から書き換えていない点です。D3 は semantic diagnostic baseline のまま維持し、その上に別 identity の answer-safety bridge を追加しています。

今後の研究も同じ原則で進めます。既存の frozen result を tuning に使わず、新しい gap には新しい研究 identity と fresh evaluation を用意します。

## 評価原則

単一の judge-model score に最適化しません。

優先順位は次の通りです。

1. measurable protocol property
2. golden / adversarial fixture
3. external oracle
4. 明示的な authority boundary
5. model-judge metric は soft evidence として利用

また、grounded runtime では「回答率を最大化すること」も「何でも abstain すること」も目標にしません。

狙うのは **grounded answerability を増やしつつ、unsafe final output を増やさないこと**です。

そのため recovery metric には、必ず unsafe-final-answer または final-claim-coverage の指標を組み合わせます。

## ベンチマーク手法

naive baseline と Harness arm を比較するときは、generated candidate を固定し、差が別の model sample ではなく deterministic process から生じるようにします。

recorded candidate は CI regression fixture にだけ使い、empirical claim には live repeated provider run を要求します。詳細は [ベンチマーク設計](benchmark.ja.md) を参照してください。

benchmark は次の両方を罰します。

- 間違った回答を `accept` すること
- 何でも `unknown` にして安全に見せること

そのため、verdict accuracy と accept / reject / unknown recall を、unsupported accepted claim と併記します。

future resolution-loop benchmark でも、one-shot / diagnose-only / bounded-resolution 間で corpus-v1 の stable base-case identity を維持します。recovered case が既存の correctness denominator を黙って置き換えることはありません。

operational failure、resolution exhaustion、missing resolver coverage は correctness と分けて報告します。

## Oracle 管理の回帰テストと open-world research

fixture oracle receipt を使うのは、expected hard result が意図的に既知のケースだけです。ここで確認したいのは、Harness が authority を正しく使い、model に authority を渡していないことです。

一方で、次の問題は別の open-world research として扱います。

- contradiction discovery
- counterexample generation
- semantic causal evaluation
- retrieval quality

resolver が document を見つけたことと、その document が要求された proposition を証明したことは別です。resolution experiment では acquisition success と verification success を必ず分けて測定します。

[ADR-0002](adr/0002-grounded-resolution-and-finalization.ja.md) は、これらの実験が検証する runtime authority と finalization boundary を定義しています。
