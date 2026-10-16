# #520 — Engine 0.7.3 撤回・非実在の判定修正

日本語 | [English](launch-retraction-520.md)

[計画 #538](https://github.com/git-ksk/reasoning-harness/issues/538)／[Draft PR #539](https://github.com/git-ksk/reasoning-harness/pull/539)。修正コードは`fd091b182999d874b802dcf92483f71ce359627c`、evidence_relevance.rs SHA256は`77727cdc14de86ad4821b1698a43799afb5b42e86d53ef566971623f1d9663f7`です。元の独立v19初回FAIL49/60を保持し、新規独立v20は初回FAIL51/60（安全性の漏れ3・肯定取りこぼし6）。ソース受入は未完了です。0.7.3は未公開・CLI未採用で、専用リリースで昇格するまでpackage表記は0.7.2です。

## 修正した動作と境界

公開0.7.2は`Silver Lens was launched. We retract this claim.`をRelevantにする場合があります。後継候補は、現在・過去形、一人称、取り下げ・否認、短縮形、非実在、対象そのものの不在、資料全体の撤回を保留します。一方、別に明確な対象の発売肯定があり、訂正対象が完全に特定された別の性質または別製品の発売だけなら、その肯定を維持します。

理由・対比・時間・報告の各節を別々に確認します。「報告する」という肯定の外枠を外しても、内側の否定の範囲は維持します。「また・明示的に」といった報告の修飾や入れ子でも、内側の対象・全体否定を維持します。括弧内・並列の訂正は全ての対象を確認。印刷された引用文字の性質は外側の完全な属性・対象指定がある場合だけ扱い、発売命題を後付けの説明で性質に付け替えません。Object/Subject/Itemの型と役割別の完全な所有者補足は混ぜません。別の役割の補足、対象への既存の結び付け、所有者不明・混在、性質から発売への型の貸し借りは保留します。完全な同格説明、物理・印刷の属性、文書の性質は局所的に扱います。未解決の参照や資料全体の否定は通しません。

製品名や完全な対象外スコープの注記による粗い欠落誤判定は、独立した正確な結び付け・rawリスクなし・別の事実肯定・元の各否定節の確認・再計算した完全性がそろう場合だけ補正します。実際の欠落・省略・切り抜き、所有者不明、対象や資料全体の否定は消しません。資料自身の「対象外」という説明だけを安全性の根拠にしません。

変更は現行v17/v30と後継専用helperだけです。公開tag・歴史v16/v29は不変です。任意の自然言語や照応、別の属性の真偽、Known/Supportedへの昇格、最終回答の有用性・実世界の精度まで実証したものではありません。

## 初回結果を保持

| 評価識別 | 固定コード | 元の初回結果 |
| --- | --- | --- |
| v1 | b6c1598 | Operational FAIL; zero-budget input, no semantic output |
| v2 | b6c1598 | FAIL50/60; unsafe1; positive misses9 |
| v3 | d8c7776 | FAIL39/60; unsafe2; positive misses19 |
| v4 | 4e36fa5 | FAIL46/60; v17 residual4/v30 unsafe0; positive misses10 |
| v5 | 7e107f1 | FAIL43/60; unsafe2; positive misses15 |
| v6 | 06d6c0b | FAIL55/60; unsafe1; positive misses4 |
| v7 | a80391e | FAIL41/60; unsafe0; positive misses19 |
| v8 | dd5b488 | FAIL50/60; unsafe0; positive misses10 |
| v9 | 9229307 | FAIL39/60; unsafe5; positive misses16 |
| v10 | e06c4ec | FAIL43/60; unsafe0; positive misses17 |
| v11 | 26c3baa | NOT_EVALUATED; parent found2 unsafe compound exclusions before freeze/semantics |
| v12 | 20e372d | FAIL40/60; unsafe3; positive misses17 |
| v13 | 2b75f04 | FAIL40/60; unsafe0; positive misses20 |
| v14 | 5edd26f | FAIL49/60; unsafe0; positive misses11 |
| v15 | 9a40b92 | FAIL45/60; unsafe1; positive misses14 |
| v16 | 854ef6d | FAIL50/60; unsafe0; positive misses10 |
| v17 | f3f1362 | NOT_EVALUATED; parent found3 unknown-owner loans before generation/freeze/semantics |
| v18 | 258667b | FAIL45/60; unsafe1; positive misses14 |
| v19 | f3d9ad1 | FAIL49/60; unsafe0; positive misses11 |
| v20 | fd091b1 | FAIL51/60; unsafe3; positive misses6 |

元の入力・ラベル・採点器・raw・hashは変更していません。観測済みセットは開発用の回帰例です。異なるセットの得点を改善率として扱いません。ローカル・CI成功で独立評価の失敗を取り消しません。v11/v17はsemantic未実行で、合否の母数に加えません。新規独立v19はこの修正コード固定後に作成し、初回FAIL49/60（否定30/30保留・肯定19/30維持）でした。初稿の「自分は発売していない」という行為主体の否定1対は、complete freezeとEngine観測前に明示的な発売否認へ修正し、初稿・ラベル・静的結果と理由を保持しました。期待極性と他58件は不変です。受入は未完了です。

## 修正コードの開発検証

[保存済み検証](../evaluation/engine073-launch-development-v19-repair/README.md)：元監査14/14（肯定1/1維持・否定13/13保留）、既知v2〜v10/v12〜v16/v18/v19の960件すべて成功、関連83テスト、workspace1547成功・失敗0・既存ignore8。strict Clippy・fmt・fixture20件・元archive13件も成功。前候補のlint不合格と所有者不明の初回不合格を保持し、親側の修正後4変種はすべて保留します。歴史v16/v29の597出力は全体がbyte一致し、親側の複合否定4件も保留します。修正途中のREDと限界も保存しています。「別の」発売参照へ前の別製品の所有者を貸しません。所有者不明と、追加所有者がいないという一意性は区別します。文字通りの名前`No Owner`は前候補でも保留する診断で、任意の否定語を含む名前の新規受入はしていません。

[前候補のCI](../evaluation/engine073-launch-source18-ci-status.json)はf3d9ad1で19/19成功しましたが、独立v19の不合格を取り消しません。[現候補fd091b1のCI](../evaluation/engine073-launch-source19-ci-status.json)は19/19成功しましたが、独立v20は不合格です。[観測済み11件の分類](../evaluation/engine073-launch-development-v19/README.md)は、印刷された引用文字2・完全な属性への参照4・別製品の所有者/引用/同格説明4・複数roleの訂正1。入力構造の分類であり、実装原因の確定ではありません。[実際の判定経路の開発診断](../evaluation/engine073-launch-trace-development-v1/README.md)では、型付き参照5・対象への否定3・参照範囲未解決3で保留を確認。現修正は属性参照を直前の完全な主張/同じroleへ結び付け、並列のroleを次の述語まで保ち、明示された別製品の所有者を扱います。一時的な計測コードは修正前に除去。観測済み11件は全て通り、追加境界と否定・歴史動作も維持。[研究と実装状況](engine-073-research-informed-repair.ja.md)では、開発検証と新規v20・実資料24問の受入を分けています。

[#540 安全な実行基盤](https://github.com/git-ksk/reasoning-harness/issues/540)は[PR #541](https://github.com/git-ksk/reasoning-harness/pull/541)で最終head CI5/5・オフライン47件と独立再レビューの新規6境界が限定PASSとなりmerge済みです。最初のP1不合格3件は不変で保持。新しい固定応答を使った同一CLIホストのinstrumented offline実行では、実0.6.1/development5eddが同じ回答となり、この通常経路ではpublic関連性API呼出し0でした。現候補の精度や、保存されなかった元Mistral応答の復元とは扱いません。live callback/dependencyと数値予算、実資料受入は未完了。合成予約300は実課金予算ではありません。

[#531 新規実資料24問・provider受入](engine-07-realworld-evaluation.ja.md)は0/24、CLI採用は開発比較・限界・NO-GO記録の1/6を維持します。Groqは記録済みの日次枠枯渇により停止中です。ソース公開・専用CLI統合・配布は[0.7.3ロードマップ](engine-0.7.3-roadmap.ja.md)に沿って別に判断します。

## v20を受けた次の修正

[初回v20](../evaluation/engine073-launch-independent-v20/README.md)は1回のみ実行・採点、ラベル変更/再実行0。全478ファイルの最終anchor `9b0edb72270648f270c497b525658d9d17aa8b71c1375b018242786ab62b1396` を親でも照合済み。持ち主が不明な別イベント参照とrole付き対象不存在の3件を先に修正する。属性の印刷引用、複数role、同格説明、明示owner補足の6件の保留は、完全な命題/参照先に限って解消し、対象/全体否定と未知ownerが除外に紛れ込まない対照を追加する。元の初回結果は不変保存し、修正・回帰/歴史チェック・新候補固定の後に別の新規独立評価を行う。実資料24問/CLI受入には数えない。
