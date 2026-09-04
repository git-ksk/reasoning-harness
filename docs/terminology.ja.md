# 用語と命名ルール

Reasoning Harnessでは、**プロダクト概念**、**機械向け識別子**、**過去の研究フェーズ名**を分けて扱います。これらは関連していますが、1本の共通バージョン列ではありません。

## Product release version

`v0.2.0`のようなCLI SemVerは、**product/distributionの互換性座標**です。product minor releaseでCLI/runtime機能が増えても、それだけで新しいresearch generationになるわけではありません。特に`v0.2.0`は、freeze済みStage-C/RSD2 evidence、semantic runtime ID、answer-safety ID、successor candidate identityを書き換えません。

## プロダクトで使う言葉

README、CLIガイド、現行ロードマップ、現在の製品作業では次の名前を使います。

| 用語 | 意味 |
| --- | --- |
| **semantic runtime** | model-backedなsoft semantic診断。soft decisionを維持または保守化できますが、verification authorityは作れません。 |
| **answer-safety gate** | grounded claimを外へ出す前に、追加verification / bounded resolution / abstainを要求できる現在の制限的安全チェック。 |
| **bounded resolution** | 不足根拠を取得し、admissionと再verificationを必須にするHarness所有の解決ループ。 |
| **verified utility recovery** | evidence/authorityルールを緩めず、すでに検証済みの有用な回答をdeterministicに回収する仕組み。 |
| **smoke set** | 既存6ケースのproduct dogfood (`product-dogfood-v1`)。 |
| **capability matrix** | freeze済み24ケース・8 familyの開発評価 (`product-dogfood-v2`)。 |
| **replication** | freeze済みcapability matrixを、事前宣言したfresh seedで複数回評価する工程。 |
| **fresh holdout** | 開発・選抜後に別途作成/freezeし、それまで未観測のまま保つ最終評価ケース。 |

## 機械向け・互換性識別子

report、rollback、schema、automationが参照するため、以下のexact IDは変更しません。利用者が暗記するための製品名ではありません。

| Identity | 役割 |
| --- | --- |
| `semantic-decidability-d3-v1` | 現在のsemantic runtime configuration ID。 |
| `soft-semantic-v3` | rollback用に保持する以前のsemantic runtime configuration ID。 |
| `verified-target-answer-gate-v1` | 現在のanswer-safety configuration ID。exact trusted verification済みtargetでは冗長なmodel sufficiency判定を短絡できます。 |
| `d3-sufficiency-answer-gate-v2` | 再現性のため保持するanswer-safety rollback configuration。 |
| `d3-sufficiency-answer-gate-v1` | rollback/testing用の以前のanswer-safety configuration ID。 |
| `shared-candidate-initial-render-v1` | product evaluationの比較contract。 |
| `reason-product-dogfood-v10` | product dogfood report schema version。v10はsemantic gateを変えずretry/checkpoint execution telemetryを追加。 |

CLIでは説明的なselectorを使います。

- `--profile current` / `--profile rollback`
- `--safety-profile current` / `--safety-profile legacy-v1` / `--safety-profile baseline`

互換性のため従来selectorも受け付けます。`d3`はcurrent semantic runtime、`v3`はそのrollback、`d3-sufficiency` / `d3-sufficiency-v2`は直前のanswer-safety rollback、`d3-sufficiency-v1`はさらに古いv1 gateを選択します。

## 過去の研究フェーズ名

`R1`〜`R4`、`D1`〜`D3`、`RSD0`〜`RSD4`、`NL-1`〜`NL-5`は、**特定Issue内で使った研究・実装フェーズ名**です。プロジェクト全体で比較できる1本のバージョン列ではありません。

研究根拠、freeze済みartifact名、過去run、chronologyではprovenanceを失わないため残します。新しいactive product workでは、新たな短縮コード列を増やさず説明的な名前を使います。

## 読み分け

**製品を使う人**は、verified evidence / semantic runtime / answer safety / bounded resolution / grounded・qualified・unknownだけ理解すれば十分です。

**運用・統合する人**は、再現性とrollbackのためmachine IDも参照します。

**研究履歴を読む人**だけが、その研究Issue内のphase labelを参照します。
