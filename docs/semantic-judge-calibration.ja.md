# ソフト意味判定器のキャリブレーション

Issue #13 は、モデルを利用した意味診断を、より広い研究ワークフローに投入できるようになる前に、どのように測定するかを定める。

中心となるルールは単純である。**意味判定器はソフトな観測者であり、正しさの権威では決してない。**

## 契約

`SoftDiagnosticJudge` は型付き `SoftJudgeRequest` を受け取り、`SoftJudgeOutput` ペイロードだけを返す。ハーネスはアダプターが保持する `SoftJudgeIdentity` とリクエスト ID を付加して `SoftJudgeObservation` を形成する。したがって、モデル出力が自分自身の来歴を選ぶことはできない。出力の判定は次の3つのいずれかである。

- `finding`
- `no_finding`
- `abstain`

`finding` に含まれるのは `SoftSemanticFinding { kind, target, note }` だけである。この型には意図的に、hard/soft の強度切替、検証レシート、認識論的状態の変更、最終判定フィールドを持たせていない。ソフト判定器の観測を、権威を持つ検証結果へ変換するコアの経路も存在しない。

`SoftJudgeIdentity` は次を記録する。

- 安定した判定器 ID
- モデル ID
- 構成 ID

判定器の構成が異なる結果を暗黙にまとめず、意見の不一致を帰属可能にするため、アイデンティティはキャリブレーション実行中一貫していなければならない。

## 診断ファミリー

初期のプロバイダー非依存リクエスト種別は次のとおりである。

- contradiction（矛盾）
- counterexample（反例）
- unsupported premise（未支持の前提）
- causal gap（因果の空白）

対象は、型付き命題、因果関係、主張、または推論エッジである。これは発見・キャリブレーション用のサーフェスであり、意味上の真理の分類体系ではない。

## キャリブレーションラベル

コミット済みのキャリブレーションコーパスは3つのラベルを使う。

- `positive`: ラベル付きの finding が期待される
- `negative`: ラベル付きの finding は期待されない
- `ambiguous`: そのケースを意図的に positive または negative の ground truth として扱わない

曖昧なケースは precision/recall の混同行列カウントから除外する。ただし、判定カバレッジ、不一致、棄権の挙動では引き続き可視化する。`ambiguous_abstention_rate` は別途報告する。これにより、意図的に不確実なケースを積極的に finding へ変換していても、ラベル付き precision/recall だけで判定器が強く見えることを防ぐ。

## 適合率、再現率、棄権

判定器ごとに、レポートには次を記録する。

- finding / no-finding / abstain の件数
- 判定カバレッジ
- 曖昧ケースでの棄権件数と率
- true/false positive と true/false negative の件数
- positive 予測が少なくとも1件ある場合の precision
- positive ラベルのケースが少なくとも1件ある場合の recall

positive ラベルのケースでの棄権は検出漏れであり、false negative/recall に寄与する。negative ラベルのケースでの棄権は true negative として加点しない。これにより、広範に棄権するだけで正確に見えることを防ぐ。

## 一致度

2つの一致指標を報告する。

### ペア単位のカテゴリ一致

各ケースについて、棄権していない判定器の全ペアを比較する。レポートには次を残す。

- 比較可能なペア
- 一致したペア
- 不一致のペア
- 棄権票の総数
- 観測されたペア単位の一致率

棄権を多数決で別カテゴリへ変換することはない。

### 名義尺度の Krippendorff の alpha

さらに、棄権を欠測データとして扱い、`finding | no_finding` 上で名義尺度の Krippendorff の alpha を計算する。ケースごとの欠測でない評定数で unit coincidence を正規化し、利用可能な判定器が多いケースに二次的な重みが付かないようにする。

期待不一致がゼロの場合、alpha は省略する。これは信頼性が構成上完全なのではなく、未定義だからである。

alpha は信頼性統計であり、検証スコアでも、最終的なハーネス正しさ指標でもない。

## 決定論的キャリブレーションコーパス

`fixtures/semantic-judges/` には、オフラインの9ケースが含まれる。

- positive が3件
- negative が3件
- ambiguous が3件
- contradiction、unsupported-premise、causal-gap の各ファミリー
- 記録済みの合成判定器アイデンティティが3つ
- 意図的な不一致と棄権

記録済みの判定器アイデンティティは、**実モデルの性能に関する主張ではなく、キャリブレーション用フィクスチャである**。資格情報や確率的なプロバイダー呼び出しなしに、集計セマンティクスを回帰テストすることが目的である。

実行:

```bash
cargo run -p reasoning-harness-cli -- eval-judges fixtures/semantic-judges --format json
```

コミット済みフィクスチャの観測では、合成判定器ごとに precision/recall/coverage の値が異なり、不一致はゼロではなく、棄権は保持され、ペア単位の一致率は1.0未満、chance 補正 alpha はペア単位の一致率未満になる。これらの値はテストデータにすぎない。

## ライブ研究

ライブ意味判定器は、同じプロバイダー非依存の `SoftDiagnosticJudge` 契約を実装できる。ライブ研究は任意かつ手動であり、モデル/構成のアイデンティティ、raw 判定、棄権、運用上の失敗を保持しなければならない。

ライブ判定器の結果は次のいずれも行ってはならない。

- `VerificationReceipt` を作成する
- hard finding を作成する
- claim を `known`、`supported`、`contradicted` に変更する
- `accept | reject | unknown` を決める
- 別のモデルであるというだけの理由で、信頼された resolver になる

将来の `ReasoningPolicy` は、キャリブレーション済みのソフト finding を、証拠取得や決定論的検証の助言トリガーとして使ってもよい。ただし、そこから生じる権威は既存のハーネス所有境界を通らなければならない。

## モデルに基づくセマンティック発見 (#33)

同じ型付きリクエスト/出力契約を、既存の任意のプロバイダー非依存 `ModelAdapter` で駆動できるようになった。ハーネスは `SoftJudgeOutput` 用の構造化出力リクエストを作成し、判定器/モデル/構成のアイデンティティを自身で付加し、返された判定を元の要求された kind と target に対して検証する。モデル出力が自分の来歴を選ぶことはできない。

主リクエストでは JSON Schema structured output を使う。アダプターが schema mode 非対応を報告した場合、または最初の応答が有効な型付き判定でない場合、ハーネスは generic JSON-object mode とシリアライズ済み schema を使って1回だけ再試行できる。不正な fallback は運用/プロトコル失敗として fail closed し、`no_finding` には変換しない。モデルを利用した実行では、ハーネス所有の `fallback_reason` として `not_needed`、`primary_json_schema_unsupported`、`invalid_primary_structured_output` のいずれかを公開する。このテレメトリが表すのはハーネスの primary→fallback プロトコルだけであり、プロバイダー内部の HTTP retry は別である。fallback 分類のために raw model output は保持しない。

`reason eval-judges` は `--provider`、`--model`、`--trials` による任意のライブ実行をサポートする。記録モードは変わらない。ライブの反復試行では次のとおりである。

- 1つのフィクスチャが失敗すると、その trial 全体が運用上未完了になる
- 未完了 trial は precision/recall/coverage/abstention の安定性分布から除外する
- provider/protocol failure は意味判定と別に報告する
- 観測された precision や agreement にかかわらず、モデルの finding はソフトのままである
- 通常の `reason eval` の correctness denominator は変わらない

手動のライブワークフローでは、secret-isolated な Mistral、Google、NVIDIA の資格情報を使ってキャリブレーションコーパスを実行できる。反復ライブ結果は研究上の観測であり、正しさの権威ではない。最初の反復 Mistral study と v1/v2 prompt-sensitivity の結果は [live soft semantic-judge study](live-semantic-judge-study.ja.md) に記録されている。v2 は元の9ケースに対してキャリブレーションされたため、これらの結果は一般化の証拠ではない。

## soft-semantic-v3 汎用判定契約 (#38)

v3 のキャリブレーション改訂では、凍結された holdout-v1 の事実ではなく、一般的な意味パターンを使ってキャリブレーションコーパスを9ケースから18ケースへ拡張する。明確な意味的同値、曖昧な命題結合、言い換えられた前提の支持、明示的な逆因果の代替、部分的/スコープ付き介入証拠、反例の適用可能性を追加する。

判定境界は意図的に非対称である。

- `finding` は、与えられたコンテキストが要求された診断上の懸念を肯定的に確立する場合に限る
- `no_finding` は、意味的同値/言い換えや明確にスコープ外の反対ケースを含め、与えられたコンテキストが懸念を肯定的に解消または否定する場合に限る
- 結合、スコープ、適用可能性、または混在/部分的な証拠によってどちらの結論も出せない場合、`abstain` が終端結果となる

特に causal gap では、相関のみ、交絡、または方向を区別していない明示的で実行可能な逆因果の代替が、方向性の支持の空白を確立できる。部分的な介入証拠や不完全なスコープだけで gap を自動的に確立することはない。十分性が未解決なら、必要な結果は `abstain` である。

この改訂が変更するのは助言的な意味契約と構成アイデンティティ (`soft-semantic-v3`) だけである。モデル出力から証拠、検証レシート、hard finding、認識論的昇格、判定権威へ至る経路は追加しない。holdout-v1 は凍結されたままであり、v3 の評価には使わない。

## 独立ホールドアウト v1

Issue #36 は、観測を含まない独立した28ケースの holdout コーパスとして `fixtures/semantic-judges-holdout/` を追加する。contradiction、unsupported-premise、causal-gap、counterexample の各ファミリーにまたがり、positive が11件、negative が8件、ambiguous が9件である。causal-gap の比重は意図的に高い。

ソースコーパスには記録済みモデル観測がない。ラベルは evaluator が所有し、モデルリクエストには含めない。holdout v1 を導入する merge 後、その fixture/request ID、ラベル、target、context は最初のライブ study のために凍結する。プロバイダー結果を使ってこの holdout version に対する prompt を調整してはならない。後から独立して測定する prompt 改訂には、観測済み v1 ケースを書き換えるのではなく、新しい holdout version が必要である。

## 独立ホールドアウト v2 の凍結

`fixtures/semantic-judges-holdout-v2/` は `soft-semantic-v3` 用の独立評価コーパスである。一般化された v3 契約のキャリブレーション後、このコーパスで v3 のプロバイダー結果が観測される前に作成された28の観測なしケースを含む。

- positive が10件、negative が9件、ambiguous が9件
- contradiction が7件、unsupported-premise が6件、causal-gap が9件、counterexample が6件
- 意味的同値、結合/スコープの曖昧さ、言い換えられた前提の支持、逆因果、交絡、時間のみの支持、部分的/混在した介入、不完全な適用可能性、反例のスコープを扱う独立した事実とサーフェス

v2 のソースフィクスチャには意図的に記録済みモデル観測を含めない。ラベルは evaluator が所有し、モデルには送らない。このコーパスが `main` に merge された時点で、fixture ID、request ID、ラベル、target、task、context は最初の `soft-semantic-v3` ライブ study のために凍結する。v2 の結果を受けた後の prompt または契約の改訂には、このコーパスを編集するのではなく、新しい holdout version が必要である。

凍結後最初のプロバイダー study は GitHub Actions run `33318380199` として記録されている。5/5 trial 完了、140/140 呼び出し成功、precision/recall `1.000`、平均判定カバレッジ `0.700`、平均 ambiguous abstention `0.933` であった。測定後もコーパスは凍結され、反復した `v2h20_causal_partial_payload_scope` の境界は、調整で消すのではなく記録されている。

## soft-semantic-v4 移植性の後継版 (#46/#53)

モデル横断の v3 結果から、分離可能な2つの portability cost が明らかになった。モデル依存の不確実性挙動と、モデル依存の型付き出力一貫性である。v4 候補は v3 の意味境界を保ちつつ、グローバルルールとして一度だけ表現する。`finding` は要求された懸念への肯定的支持、`no_finding` は肯定的な解消/否定を必要とし、結合、スコープ、適用可能性、権威、十分性が未解決、混在、または部分的で、どちらも十分に支持されない場合、`abstain` が終端となる。kind 固有の文言は要求された懸念だけを定義する。

モデル向け JSON Schema は、公開/内部の optional-finding struct ではなく discriminated union である。`finding` は型付き finding object を必須とし、`no_finding` と `abstain` はそれを許可しない。パース済み出力は同じ内部 `SoftJudgeOutput` に戻され、kind/target の完全一致検証も必須のままである。これはプロトコル表現の変更であり、意味または権威の不変条件を緩和するものではない。

`fixtures/semantic-judges-holdout-v3/` は、ライブの `soft-semantic-v4` プロバイダー測定前に凍結される。28の観測なしケースを含み、診断ファミリーごとに7件、ラベルは positive が8件、negative が8件、ambiguous が12件である。互換性の閾値は [cross-model semantic judge conformance](semantic-judge-conformance.ja.md) に固定されている。holdout-v2 は v3 挙動の診断専用であり、独立した v4 評価コーパスではない。

### 独立ホールドアウト v3 の結果と v4 の棄却

v4 契約、互換性基準、`fixtures/semantic-judges-holdout-v3/` が merge され凍結された後、5モデルのプロバイダー測定では、事前宣言した `usable_with_limitations` tier にすら達するモデルはなかった。Runs `33342332130`、`33342547879`、`33342334655`、`33342335857` は Ministral 8B、Mistral Small、Gemini 3.1 Flash-Lite、Ministral 14B についてそれぞれ 140/140 呼び出しを完了した。Nemotron run `33342337031` は 69 protocol failure を伴い、71/140 呼び出しで完了 trial はなかった。

簡略化されたグローバル文言は、複数のモデルファミリーで不確実性の挙動を弱めた。より厳格な discriminated model schema は、v3 で Ministral 14B に繰り返し見られた non-finding-plus-finding protocol failure を解消したが、その個別のプロトコル改善だけでは意味 portability 基準を満たさなかった。したがって v4 adoption gate は失敗し、runtime baseline は、以前に特性評価済みの `soft-semantic-v3` request/schema contract と完全に同じものへ戻された。

Holdout-v3 は現在観測済みで、引き続き凍結されている。v4 または後継を調整するために使ってはならない。将来の実質的な契約/schema 後継には、キャリブレーション専用の設計と、ライブプロバイダー測定前に凍結した観測なしの holdout-v4 が必要である。
