# Engine 0.7.3 — 個人開発向けペア品質測定の運用手順

## 目的・位置付け

#531/#613/#620 の実測を、2026-11-14 の非公開 Dev24 測定と同じ GitHub Actions hosted runner・Secrets・暗号化 Artifact 回収方式で繰り返し利用できるようにする。別チャットによる準備、意味評価、集計の手続き的分離。第三者認証済みの正式独立評価ではない。#620 の正式認証実装・EG1〜EG8・凍結済み成果物・HOLD は変更しない。採点条件は事前固定、品質基準は緩和しない。

前例:
- [43581076449](https://github.com/git-ksk/reasoning-harness/actions/runs/43581076449): 非公開24問、48観測すべて成功。Secrets経由入力、暗号化Artifact回収。
- [43667947455](https://github.com/git-ksk/reasoning-harness/actions/runs/43667947455): 実資料24問、44/48完了。350分上限で終了。残りを再実行して補完しない。次回から独立した短い8ジョブ。従来の完了スコアは今回のスコアの代用にしない。

## 固定条件

- Core baseline `4abee90f0501f0d8fc09b453a0b45c942fbed69e` / candidate `7466c6f90d420a98c99713b96d6a0623e2aeef52` に既存 #552/#613 修正パッチを同一適用。共有CLI `b85a4a0b2ab03397b361a8620d7a97dd26b2eb36`。
- Mistral `ministral-8b-2512`、24問（16陽性/8陰性は非公開Goldで事前確認）、48初回観測、8シャード×3ケース×2アーム。シャードは各々独立した hosted Mac job、同時実行1（プロバイダ共通クォータ）。各CLIに12論理model calls/28,096 tokens/4,096 output tokens/1,024 max generation tokens、全体576/1,348,608。既存CLIの内部transport attemptは別計数。外側再実行なし。
- F1 0.90以上、precision 0.95以上、recall 0.85以上、安全違反・無効引用ゼロ、ペア/集計劣化なし。欠落・不明・不完全・timeout・cancelはHOLD。事後ルール変更、失敗枠差し替え、再試行、正式F1扱い禁止。

## 1. 事前準備（Mac・モデル呼び出しなし）

Workflow: `.github/workflows/engine073-owner-paired-v1.yml`、実装 `evaluation/owner-paired-v1/measure.py`。凍結されたCore/CLIと採点契約は別途保存。次バージョンのCore/CLI/model/データ件数変更はこのv1を書き換えて暗黙に流用せず、固定条件を更新した別プロトコルを事前作成する。

事前準備ディレクトリ（owner-only、リポジトリ外）には以下が必要。今回の実体は `~/.rh-issue620-owner-evaluation-20261123-r1/`。
- `runner-input/questions.json`（case_id/question/source_id のみ）、`runner-input/manifest.json`（12資料の不変SHA/URL）、`runner-input/sources/<id>`（凍結された原文）
- `source-configs/<id>.json`（同じ資料に属する2問は同一の取得/引用設定。各quoteを原文の行範囲と照合、Gold・期待ラベル・ケース別正解情報を入れない）
- 凍結済み問題・Gold・採点契約・独立性の制約は別の非公開保管先。GoldのSHA等はowner側で保全し、runner payloadにGoldを含めない。
- 設定は準備担当とは別のチャットで、ソース内事実の正確性、引用範囲、両アームへの公平性を確認してから実行。自己申告の署名や独立認証とは区別する。

Macでソース・設定・既存receiptを検証。以下のpackは実データをstdoutに出さず、公開可能なhashタグだけ出力する。payloadをGitに追加しない。

```bash
umask 077
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
prepared="$HOME/.rh-issue620-owner-evaluation-20261123-r1"
ref=$(python3 evaluation/owner-paired-v1/measure.py pack "$prepared" "$tmp/input.b64")
# ref形式: eval-owner-paired-v1-<stable-cohort-sha>-<exact-payload-sha>
# 48KiB制限内をpackで検証。超過時は別途安全な分割transportを事前実装。
```

コホートIDは問題文・源文書ハッシュ・固定Core/CLI/modelから作る。順序/ID/設定変更だけで再測定権を作らない。意味的に同じ問題文を言い換えた場合も別データとしてリトライしない（事前レビューで検知）。前回の一回限りのタグ・claim・Artifactは変更/削除しない。

## 2. 実行時のみ（オーナー許可後）

既存 `MISTRAL_API_KEY`（GitHub repository secret）を再利用。MacでのAPIキー再発行・登録、Self-hosted runner、新規外部認証サービスはいずれも不要。専用入力secretのみ一回限り更新する。他のシークレットを触らない。GitHub hosted runnerへの非公開問題/資料設定の移送をownerが許可した場合に限る。公開repoのGit/ログ/Artifactに平文を置かない。ベンダー側の保持や同一GitHubアカウントの設定変更権限を排除できるわけではない。

```bash
# 先にWorkflow実装があるレビュー済みコミットをcheckout、タグ/claim未使用を確認。
repo=git-ksk/reasoning-harness
commit=$(git rev-parse HEAD)
cohort=${ref#eval-owner-paired-v1-}
cohort=${cohort%%-*}
git ls-remote --tags origin "refs/tags/$ref" "refs/tags/eval-owner-claimed-v1-$cohort"
# 出力があれば停止（既存run確認）。active workflowがないことも確認。
gh secret set ENGINE073_OWNER_PAIRED_V1_INPUT --repo "$repo" < "$tmp/input.b64"
git tag "$ref" "$commit"      # lightweight tag; never force
git push origin "refs/tags/$ref"
```

普通のPRは合成fixtureのみのoffline controls。タグのpushとrun_attempt=1のみlive候補。全48計画の一致確認後、claim jobが別名タグ `eval-owner-claimed-v1-<cohort>` をGitHub APIでcreate-onlyに作成する。既存claim/失敗は停止。claim後のキャンセルやtimeoutでも同一コホートは再投入しない。claimの書き換え/削除禁止。owner/adminが意図的に回避できない客観的な独立台帳ではない。

各8ジョブでpayloadタグと全48初回リクエストを再検証。既存 `issue531_real24_api_billing_gate.py` と同一キーでAPI PAYG両方OFF・含有予算残>=1 USD/EUR・固定model利用可能を確認後のみ実行。モデル呼出は1枠1回。予約を排他的に作成し、失敗/未知usage/上限超過で停止、後続シャードもfail-fastで停止。各CLIに別HOME/作業ディレクトリ。GitHubのキャンセル等では未回収データもあり得るので未知はHOLD。CIのログは集計件数のみ。

## 3. 結果回収・別チャット採点

結果は暗号化された `evidence.cms` のみ（7日保持）、既存reviewer-cert.pem（SHA256 d917aeb4c55699b1ea19117c776e5f515aa40193488197a12c00082517695b43、期限2027-11-15 UTC、満了前に対応鍵を確認して別途更新）。秘密復号鍵の持ち出し/表示は禁止。実行前にMac側の対応復号鍵が利用可能と確認する。Artifactを公開用途に転送しない。

```bash
gh run list --repo "$repo" --workflow engine073-owner-paired-v1.yml --limit 10
gh run view <RUN_ID> --repo "$repo"
# Macの非公開新規ディレクトリ（0700、バックアップ除外）へ回収
gh run download <RUN_ID> --repo "$repo" -p 'owner-paired-v1-<RUN_ID>-shard-*' -D /path/to/private/new-download
# 各evidence.cmsは既存の対応reviewer秘密鍵で復号し、安全なディレクトリ内に展開
openssl cms -decrypt -binary -inform DER -in evidence.cms -recip /path/to/reviewer-cert.pem -inkey /path/to/private-reviewer-key.pem -out raw.tgz
```

8/8シャード・同一run ID/tag・48/48一意のattempt=1・6/6 closure・raw stdout/stderr/hash・usage・finalizationを照合。部分出力も原本保存。全件完了時のみ、アーム名を伏せた最終表示と原資料+Goldを別チャット担当に渡す。必要に応じ別チャットで不一致裁定、ロック後に対応表開示、case-level TP/FP/FN/TN、引用妥当性、安全性、ペア回帰と全閾値を集計。#620 の既存scoring contractとevidence eligibility candidateを事前確認。合成スコアラーに実データを偽装投入しない。品質スコアをowner-managed procedural measurementと明記し、正式独立scoreに昇格しない。集計値のみIssueに記録し、生回答・Gold・問題は公開しない。公開/CLI採用/mergeは別の判断。

input secretの削除は実行終了・回収確認後にownerの許可で行う（旧Mistralキーは削除しない）。途中停止・実行履歴・claimは消さない。
