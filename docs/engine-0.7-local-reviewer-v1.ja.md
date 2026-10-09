# Engine 0.7.0 — ローカル承認レビューの開発用ワークフロー（#496）

日本語 | [English](engine-0.7-local-reviewer-v1.md)

**状態：開発候補。独立評価・正式リリースは未完了。** 公開済みReason CLI 0.5.4とHarness Engine 0.6.1の配布物は不変です。

## 今回の機能

オプトインの専用開発バイナリ `reason-source-review-local` で、ローカルのレビュー主体を登録し、同じ対象に対する**2つの正確な引用**を人間が確認・承認できます。署名付き承認の保存、根拠の再検証、元の出典を保持した表示、レビュー鍵の失効まで扱います。通常の`reason`コマンドが自動承認することはありません。

意味が互換と人間が承認した場合、新しい追加viewだけが`reviewed_compatible`になります。**従来の`SourceAttributionFinalization.status`は`Conflict`のまま**、原文と各sourceのcitation、hard verificationも維持します。外部事実を`Known`や`Supported`へ昇格させません。

## 認証・信頼境界

信頼するのは**ローカルOSにログイン済みのユーザー、対話式ターミナル、OSのキーチェーン／資格情報ストア**です。ホスト側でランダムな256bitのHMAC-SHA256鍵を作成してOSストアへ登録し、原文や承認JSON、モデル出力、コンソールへ鍵を出しません。

- **登録・承認・失効**は対話式ターミナルで正確な確認フレーズを手入力する必要があり、標準入力のパイプ、CI、バックグラウンド操作は拒否します。
- 登録済みレビュー主体の上書きは拒否。失効で鍵を削除し、昔の承認ファイルが再検証できなくなります。再登録時は新規鍵です。
- 承認時はtarget、claim、2つの原文、source binding、source version、scope、適用時点、出典情報を**スナップショットとして全部表示**し、SHA-256も提示します。モデルがJSONへ承認フラグを書いても意味はありません。
- 承認ファイルは既存ファイルを上書きせず、Unixでは0600の新規ファイルに保存。原文と紐付け情報を含むため**機密ファイルとして扱います**。
- 承認ファイルにはレビュー主体、発行日時、30日以内の有効期限、ランダムnonce、出典スナップショット、HMACを格納。表示時には**現在登録されているOSストアの鍵を再取得**し、定時間のMAC照合と現行artifactの出典・対象・時点・scope・版の一致を確認します。
- **明確な限界：** 同じOSユーザー権限でコードを実行でき、キーチェーンにアクセスできる別プログラムによるなりすましは防ぎません。生体認証や外部の本人確認・専門家認定ではなく、同義性の正しさは承認する人が判断する必要があります。キーチェーンが使用できない場合、環境変数や平文鍵ファイルにはフォールバックせず拒否します。

保存した承認JSONのpolicy IDだけでは自分をレビュー主体と認証できません。モデルの推測、MCP応答、取得文書の指示から承認権限を生成することもありません。

## 使い方（開発用・人間による操作が必要）

通常の配布済み`reason`にはまだ組み込んでいません。以下は**ログイン済み端末のユーザーが直接操作する**開発用の手順です。受理済みの`ReasoningArtifact`をJSONとして用意し、targetとsource-attributed claimのIDを指定します。

```bash
cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  enroll --reviewer local-owner

cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  approve --artifact /private/path/artifact.json \
  --target TARGET_ID --first-claim CLAIM_ID_A --second-claim CLAIM_ID_B \
  --reviewer local-owner --output /private/path/approval.json

cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  show --artifact /private/path/artifact.json --target TARGET_ID \
  --reviewer local-owner --approval /private/path/approval.json

cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  revoke --reviewer local-owner
```

3件以上の出典で互換性を承認するには、各ペアを個別にレビューして複数の`--approval`を指定します。必要な全ペアの審査が完了しなければ`Conflict`のまま。`show --json`で追加viewの構造化結果を確認できます。

## 人間の承認前に行う実機QAの準備

OSキーチェーンの登録や外部モデル呼出を**まったく行わない**、専用の2コマンドを追加しました。

```bash
cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  demo --output "$HOME/reason-review-synthetic-demo.json"

cargo run --locked -p reasoning-harness-cli --bin reason-source-review-local -- \
  inspect --artifact "$HOME/reason-review-synthetic-demo.json" \
  --target demo-target --first-claim demo-claim-0 --second-claim demo-claim-1
```

`demo`は架空の2引用を含む合成artifactを**秘密権限で新規保存**し、既存ファイルの上書きを拒否します。`inspect`は原文・target・claim・binding・evidence・SHA-256を**読み取り専用で**表示し、「承認ではない」と明示します。取得元の制御文字はJSONエスケープし、ターミナル操作の乗っ取りを防ぎます。どちらもキーチェーンを読み書きせず、承認・署名は発生しません。

開発用Macでは`/tmp/reason-source-review-demo-496.json`（権限0600）の生成と読み取り専用プレビューがPASS。続く`enroll`→`approve`→`show`→`revoke`は**実際の利用者自身が対話的に**行う必要があります。新viewが`reviewed_compatible`でも旧v1の`Conflict`が維持されること、鍵の失効後は昔の承認ファイルが拒否されることを確認します。AIやCIが本人に代わり承認することは禁止です。

凍結時のRustテスト4本と18ケースの採点は不変。凍結**後**にデモ生成・秘密権限と制御文字の安全性の2本を追加し、現在はRustテスト6本です。旧4本のログと新6本のログをそれぞれ再現可能にし、凍結18件の結果を書き換えません。

## 事前固定した開発評価

- 実装前の凍結タグ：`engine-0.7-reviewer-host-dev-v1-spec-freeze`（`fa87aac3c6fc602c2874b604f6e027a4d093784a`）
- fixture SHA-256：`cc0b0acc4bb53ac94eaeb0d60489b225691f2ec5427a5bbf5398da3e7e35bee4`
- 固定18ケース：`fixtures/engine-0.7-reviewer-host-development-v1/manifest.json`
- Rust実装・テスト：`crates/reasoning-harness-cli/src/bin/source_review_local.rs`
- 機械可読レポート：`evaluation/engine-0.7-reviewer-host-dev-v1-result.json`
- 再現・改ざん検出：`scripts/engine-0.7-reviewer-host-dev-v1-report.py`、`scripts/test_engine_0_7_reviewer_host_dev_v1_report.py`

| 事前固定した判定 | 件数 |
| --- | ---: |
| 明示レビューを承認 | 2（独立した肯定例1、replay例1） |
| レビューなし・保守的なConflict | 2 |
| 従来どおりQualified | 1 |
| なりすまし・改ざん・失効・期限・矛盾等を拒否 | 13 |
| **総計** | **18/18 PASS** |

RFC 4231 HMAC標準テストの初回実行では、**期待ハッシュの転記誤りによるFAIL**が1回ありました。Python標準の`hmac`/`hashlib`と照合して期待値を修正し、修正後の全RustテストがPASS。初回FAILを過去のPASSとして書き換えていません。凍結18ケースも変更していません。

今回の18件は**開発用の人工データ**。実際のOSキーチェーン登録と本人の承認操作は実施せず、決定論的テストではメモリ上の試験鍵を使用しました。外部モデル呼出・provider試行・外部取得・hard fact昇格はすべて0件でした。

## #496・#490・#492の残作業

1. 実際のユーザー自身の承認を伴い、キーチェーン登録・解除、承認ファイルの保存、再表示、失効と再登録を物理端末で確認する。ユーザーになりすまして承認することはしません。
2. 0.6.1でConflictになっていた実際の質問について、**ユーザーにとって意味のある出典付き表示の改善**を実測する。人工ケース1件の改善だけでは商用の有用性を証明できません。
3. 候補コード・評価runnerを固定し、それ以降に**開発ケースとは別の独立holdout**を作成。Mistral／Google／Groqをそれぞれ同じ入力の0.6.1と比較し、各モデルで有用な回答の純増1件以上、安全性重大違反0件を確認。初回独立FAILは書き換えず、provider障害を意味的な失敗と混同しません。
4. 条件がそろうまで#496と#490は閉じず、Engine 0.7.0のバージョンアップ・タグ付け・CLI正式採用はしません。

この仕組みは**ローカルの人間のレビューを明示的につなぐ開発用の橋渡し**であり、モデルによる自動同義判定、出典信頼性ランキング、汎用的な本人認証基盤ではありません。
