# Owner-managed paired v4 — 429再試行と観測の耐障害性

Issue #620 の開発用比較。Engine改善PR #622と両Coreの固定コミットを変更しない。旧run 45101280387、45198846318、45319236239と既存38正常観測、凍結Gold・採点契約・claimは不変で保持し、新結果とは混合しない。新規独立holdoutでも正式品質認証でもない。

共有CLI 7cf783d2169cd7882832fd780070ab7280ad63c0。従来のb85a4a0との差分は共有CLI/providerの6ファイルだけ。既存の両arm共通Core互換パッチ2個はそのまま使用。24問・48観測、資料・モデルministral-8b-2512・seed・交互arm順序・max-model-calls=12、max-output-tokens=8192、max-total-tokens=32192、1生成max-tokens=1024を固定する。

## 予算付き再試行

明示的なowner開発用 --mistral-accounted-retry のみを許可する。正式native HTTP journal・通常の累積予算付き単一試行は不変。一時的容量不足、既知のレート制限429だけ、最大3再試行、30/60/120秒を基本、累積再試行待機210秒。Retry-After/分単位resetを短縮せず尊重し、個別120秒超や無効ヘッダーは停止する。課金・月間quota・認証・未知エラーは再送しない。

各HTTP試行前に既存native admissionで用いた運用上の予約式を再利用する。入力は論理request JSONとwire JSONのUTF-8 byte数の大きい方+512、出力は要求max_tokens。残予算に収まる場合だけ送信。成功または完全かつ整合した429 usageは報告値を確定し、使用量のない既知の一時的429は予約額全額を消費する。不整合・予約超過・成功usage欠落・モデルpin不一致・不明障害はfail closed。予約はプロバイダーの請求実績を意味せず、hosted tokenizerに関する既存の保守的運用仮定である（プロバイダーによる厳密な上限保証ではない）。予約超過が観測された場合は実験全体を停止する。

CLI UsageReportのtoken_accountingで「reported_plus_conservative_reservations_v1」を明示し、実測報告値と不明試行の予約値は各試行のstderrから分離して集約する。予約値は実消費として報告しない。HTTP試行回数と論理モデルコール数、再試行実待機、成功後pacing実待機を別々に記録する。

## 実行・補完

初回v4 run 45433270563は最初の00-baseline完了後、sealの完全ref名259 byteがGitHubの255 byte制限を超え停止した。APIの422を実際に再現し、短縮した247 byte refを使用する修正版を作成。元claim・halt・launch tag・artifactを変更しない。

修正版は eval-owner-paired-v4-repair-<cohort>-<payload-sha> の新規軽量tagで起動する。同一cohort、CLI/Core、入力、測定条件を保持する。Mac上で元artifact 14943095748のzip SHA、CMS復号、原本ハッシュ、入力/観測pin、課金ゲート、終端分類・トークン台帳を再検証した00-baselineだけを引き継ぐ。旧runの固定head・claim・halt・artifact metadataをadmissionで照合し、短縮namespace rh4r-*に旧headを指すclaim/sealをcreate-onlyで記録する。元runのraw原本は変更せず、このslotは再実行禁止。残り47観測だけを対象とする。入力は既存のGold-free secretを再利用するがpayload hashを厳密に固定。offline-controls、cohort admission、各shardで両armの実buildと48 request plan照合、各slot前の同一API keyによる課金ゲート（PAYG無効・含有枠1通貨単位以上・モデル提供）を必須とする。GitHub Actionsは1 shardずつ、各shard6観測、1観測2700秒、shard345分。新規観測間60秒、終端一時的429後120秒、shard開始時も120秒待機。

各slotはcreate-only claimを取得した後で一度だけ起動。完了回答、厳密な予算枯渇・予約不足、完全に予算計上された限定429終端、明示的ローカルdecode失敗のみを個別終端として後続観測を継続する。失敗を成功として採点したり、同じ観測を再実行しない。その他の認証・課金・予算超過・使用量不明・証跡不整合などはcohortを停止する。

slot単位の原本、予約、開始証跡、課金ゲート、使用量・試行ログを既存受取証明書でCMS暗号化し直ちにuploadする。秘密とGoldをrunnerログに出さず、作業homeもarchiveに入れない。upload済みartifact ID・zip digest・暗号化ファイルSHAを不変sealに固定する。hard stop時も可能な限り原本を暗号化保存する。runner強制消失などでclaimのみ残ったときは開始有無を推測せず、補完を拒否する。

未割当runner/未claimだけの補完は、同一修正版launch commitのeval-owner-paired-v4-repair-<cohort>-<payload-sha>-resume-<prior-run-id> tagを新規pushする。直前runの終端、同一workflow/commit/tag系譜、既存全claimの一意sealとartifact存在・digest、修正版halt無しを確認する。引き継いだ1slotだけは元headとartifactを厳密に固定する。seal済み観測はskipし、claimのないslotだけを実行する。既存claim、seal、halt、launch tagの削除・変更、GitHub rerunは禁止。

## 採点

48 slotの証跡を収集・検証してから、Mac上の既存凍結Goldと採点契約でarmを伏せて採点し、判定固定後に比較する。予算・429・decode終端は正常回答に含めない。欠損・未確定があれば正式F1/品質合格を主張しない。報告済み使用量と予約額、HTTP試行・論理コール・待機を分け、予約を含む総額は保守的上限として明記する。
