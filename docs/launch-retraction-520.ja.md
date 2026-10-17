# #520 — Engine 0.7.3 撤回・非実在の判定修正

日本語 | [English](launch-retraction-520.md)

## 最新の修正・独立評価と残件

候補`01dc78b`で残る21件を修正した。同じ固定60件の開発比較はv17/v30ともTP30/FP0/TN30/FN0、Precision/Recall/F1各1.000（c20のv30 F1 0.461538から改善）。全体1572成功/失敗0/既存ignore8、strict Clippy/fmt、既知1140・元監査14、歴史597全出力byte一致とv29の60件不変を確認。これは観測済み合成APIの開発測定であり、実回答F1や一般精度ではない。新規独立v23は 初回FAIL40/60（肯定10/30・否定30/30、両APIのPrecision1.000・Recall0.333333・F1 0.500、誤通過0・肯定保留20・運用/不完全0）。元v22初回FAIL30/60・全過去初回は不変。マージ・公開・CLI採用はNO-GO。新しい20保留を現コードで診断し、同じ開発入力の指標を追って修正する。

[実回答の採点準備](engine073-realworld-metrics-preparation.ja.md)で主張grounding Precision、必要主張Recall/F1、引用、正しい/過剰保留、有用性、運用欠測、provider別会計を定義した。PREPARATION_ONLY、M0/M1未固定・実回答未採点。新scorer、実host callback/dependency closure・数値予算/headroom・金額自己上限、featureが働く通常CLI経路が残る。[現候補の同一host offline CLI](../evaluation/engine07-cli-path-preparation-v3/README.md)は固定Node1経路で両Engineの同じgrounded回答・取得1・関連性API呼出し0、直接v17/v30対照ではcounter動作を確認した。全経路の不活性や今回修正の実回答効果を確定した結果ではない。#541共有runnerはmerge済みだが#540のlive準備はOPEN。#531新規実24問0/24・採用1/6、Groq追加起動禁止、provider/model/Oracle追加0を維持する。

過去候補c20時点の記録：[計画 #538](https://github.com/git-ksk/reasoning-harness/issues/538)／[Draft PR #539](https://github.com/git-ksk/reasoning-harness/pull/539)。現修正候補は`c20c5136d59cb3a0bfdac4fb8ca9dade2d93e076`、evidence_relevance.rs SHA256 `6b5aa28035bf23e6ad5eded264b323109786963354f3769a74fffd41ebe54a31`。同じ観測済み60件の開発比較ではv30 F1 0→0.4615、肯定9/30・否定30/30、過剰保留21が残る。独立v22初回FAIL30/60（7431333）は不変で、新候補の新規独立評価は未実施。0.7.3未公開・CLI未採用、package表記は0.7.2。

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
| v21 | d647ca4 | FAIL48/60; unsafe0; positive misses12 |
| v22 | 7431333 | FAIL30/60; unsafe0; positive misses30 |

元の入力・ラベル・採点器・raw・hashは変更していません。観測済みセットは開発用の回帰例です。異なるセットの得点を改善率として扱いません。ローカル・CI成功で独立評価の失敗を取り消しません。v11/v17はsemantic未実行で、合否の母数に加えません。独立v19はf3d9ad1固定後に作成し、初回FAIL49/60（否定30/30保留・肯定19/30維持）でした。初稿の「自分は発売していない」という行為主体の否定1対は、complete freezeとEngine観測前に明示的な発売否認へ修正し、初稿・ラベル・静的結果と理由を保持しました。期待極性と他58件は不変です。受入は未完了です。

## 修正コードの開発検証

[現候補の検証](../evaluation/engine073-launch-development-v22-foundation/README.md)：候補c20c513、全体1564成功・失敗0・既存ignore8、strict Clippy/fmt・fixture20・元archive13成功。過去1080件成功、元監査14/14、歴史597出力byte一致、同じv22入力60件のv29全出力も不変。v22を開発再現に使った39/60は不合格で、F1/Recallの改善は新規独立受入に数えない。最初の修正はF1 0.45・誤通過1で不採用にし、[出力・途中検証・限界](../evaluation/engine073-launch-development-v22-foundation-rejected/README.md)を保存。新しい意味コードのCIは別確認が必要で、旧7431333 CI19成功とは区別する。

[前候補のCI](../evaluation/engine073-launch-source18-ci-status.json)はf3d9ad1で19/19成功しましたが、独立v19の不合格を取り消しません。[前候補fd091b1のCI](../evaluation/engine073-launch-source19-ci-status.json)は19/19成功しましたが、独立v20は不合格です。[観測済み11件の分類](../evaluation/engine073-launch-development-v19/README.md)は、印刷された引用文字2・完全な属性への参照4・別製品の所有者/引用/同格説明4・複数roleの訂正1。入力構造の分類であり、実装原因の確定ではありません。[実際の判定経路の開発診断](../evaluation/engine073-launch-trace-development-v1/README.md)では、型付き参照5・対象への否定3・参照範囲未解決3で保留を確認。fd091b1の修正は属性参照を直前の完全な主張/同じroleへ結び付け、並列のroleを次の述語まで保ち、明示された別製品の所有者を扱います。一時的な計測コードは修正前に除去。観測済み11件は全て通り、追加境界と否定・歴史動作も維持。[研究と実装状況](engine-073-research-informed-repair.ja.md)では、開発検証と新規v20・実資料24問の受入を分けています。

[#540 安全な実行基盤](https://github.com/git-ksk/reasoning-harness/issues/540)は[PR #541](https://github.com/git-ksk/reasoning-harness/pull/541)で最終head CI5/5・オフライン47件と独立再レビューの新規6境界が限定PASSとなりmerge済みです。最初のP1不合格3件は不変で保持。新しい固定応答を使った同一CLIホストのinstrumented offline実行では、実0.6.1/development5eddが同じ回答となり、この通常経路ではpublic関連性API呼出し0でした。現候補の精度や、保存されなかった元Mistral応答の復元とは扱いません。live callback/dependencyと数値予算、実資料受入は未完了。合成予約300は実課金予算ではありません。

[#531 新規実資料24問・provider受入](engine-07-realworld-evaluation.ja.md)は0/24、CLI採用は開発比較・限界・NO-GO記録の1/6を維持します。Groqは記録済みの日次枠枯渇により停止中です。ソース公開・専用CLI統合・配布は[0.7.3ロードマップ](engine-0.7.3-roadmap.ja.md)に沿って別に判断します。

## v20を受けた修正と新規受入

[初回v20](../evaluation/engine073-launch-independent-v20/README.md)は1回のみ実行・採点、ラベル変更/再実行0。全478ファイルanchor `9b0edb72270648f270c497b525658d9d17aa8b71c1375b018242786ab62b1396` を親でも照合。[一時的な実経路診断v2](../evaluation/engine073-launch-trace-development-v2/README.md)の後、候補d647ca4で、別の未解決イベントへ前のforeign ownerを貸さず、同じ文のowner不明と役割付きの対象不存在を確認するよう修正した。完全な同格属性、役割別の精度否定とowner補足、カンマで並ぶ述語を正しい参照先へ結び付け、印刷引用の閉じ記号を保持。既存の後継専用viewでは、完全な印刷属性を証明した場合だけ引用の中身を置き換え、引用外の文字列・実際の欠落とrawリスクによる適格性制限を残す。

元の9件は観測済み再現ですべて成功。追加した7境界群・元の回帰/歴史確認も成功。途中のカンマによる旧v5の退行、引用2変種のREDも保持。親の検証用ライブラリ参照ミスは新規独立実行前に検出し、[不採用の証拠](../evaluation/engine073-launch-development-v20-buildbinding-rejected/README.md)として保存した。明示したビルド設定とcargo compiler-artifactから正しいrlibを確定し、別freeze/output/binaryで上記の最終検証を完了した。元のv20初回結果は変更しない。

新規独立v21は正しいsource/core/serdeとopaque freeze固定後に著作・初回FAIL48/60（安全性の漏れ0・肯定保留12）。最終保存までHEAD/source/通常targetを固定し、全体検証は別targetで実施。[候補d647ca4のCI](../evaluation/engine073-launch-source20-ci-status.json)は19/19成功。新規独立受入の成功とは別です。新規受入・実24/CLI/provider/人の受入は未完了で、異なる合成セットの得点を改善率としない。

## v21を受けた修正と新規評価

[初回v21](../evaluation/engine073-launch-independent-v21/README.md)は不合格を保存。著作段階の裸role参照をexplicit theへ揃えた元author/静的AST/差分・意味/極性不変を保持、complete freeze後は変更0。旧478を含む全516ファイルanchor `9db10bc88ef24b76cd6dd360bd0b23b90b32a19dac74cbbcb7c6f6dd851ee913` を親でも照合。[実経路v3](../evaluation/engine073-launch-trace-development-v3/README.md)は型付き参照9・対象否定経路2・未解決範囲1を記録した。

候補7431333では、完全な別製品の材料・文書属性をその属性のownerへ結び付け、publisherや引用内のイベント名を発売ownerに転用しない。複合属性、印刷引用の外側のowner/属性、役割別の補足・有限参照、完全属性への照応と独立した否定を扱う。観測済み12件は解消し、肯定/否定の新境界6群と上記の全体・歴史検証を通過。新規60件の独立v22初回はFAIL30/60（陽性0/30・陰性30/30）。初回実行・採点は各1回、ラベル変更/再実行0。受入条件は変更しない。


リスク仕様の確認：モデルのrawリスクは助言であり、過去v1〜v16は資料中の局所的な欠落・範囲から有効リスクを導く。rawがNone以外なら今回の新しい投影は使わず、v16由来の有効リスクを維持する。raw enumを常にそのまま有効リスクへ写す仕様ではない。実際の欠落・owner不明・対象/全体の否定は保留する。relationが見えていてもriskで保留できる。追加した親テストの2期待値はこの既存契約へ訂正し、独立評価の入力・ラベル・初回結果は変更0。

## v22の記録と次の診断

[保存済み初回結果](../evaluation/engine073-launch-independent-v22/first-summary.json)は不合格を維持。独立担当が初回保存後に利用上限で停止したため、[親による閉鎖記録](../evaluation/engine073-launch-v22-parent-closure/README.md)は旧516と現存v22の29、計545ファイルを改変せず照合した。これは独立担当の最終レビューの代用ではない。その後の再開で独立担当の最終保存が完了した。停止時の親記録も履歴として保持する。候補7431333の[CI19/19成功](../evaluation/engine073-launch-source21-ci-status.json)も不合格を取り消さない。

陽性30件全てが保留したため、ソースとは別のコピー/targetで観測済み30件の実経路を診断する。完全属性の型・owner、引用外の証明、同じroleへの参照を枝ごとに確認し、実原因と修正範囲を記録してから意味変更する。属性語の追加だけで完了とせず、対象/全体否定・所有者不明・複数役割を維持する。過去初回は変更せず、同じ固定入力の修正前後F1は開発比較、新規独立F1は汎化確認として別集計する。

## 指標を固定して測る

[F1等の集計](../evaluation/engine073-launch-first-metrics-v1/README.md)では、陽性を「v17 requested_relation / v30 Relevantが必要な固定ラベル」と定義し、API別にTP/FP/TN/FN、Precision、Recall、陽性F1、specificity、balanced accuracy、通過率を記録する。分母0は未定義null。v22は両APIともTP0/FP0/TN30/FN30、Precision未定義・Recall0・F1 0・specificity1・balanced accuracy0.5・通過率0。これはCLIの実回答率/最終回答F1ではない。運用失敗・未観測を陰性成功に変換せず、別コーパスのF1を改善率とせず、既知再現と新規独立を分ける。安全性FP0はF1にかかわらず必須。実資料24問では主張ごとの根拠対応と回答有用性も別に採点し、現時点では未測定。


再開した独立担当が最終保存を完了し、全551ファイルを照合。anchor359a6712b07bc1a58fce4332f77f190578b6fec0bad4f9aab0e022c6b3fef160、初回FAILは不変。[親の最終照合](../evaluation/engine073-launch-v22-parent-final-verification.json)・[初回指標の集計](../evaluation/engine073-launch-first-metrics-v1/README.md)を保存。[別コピーの実経路診断](../evaluation/engine073-launch-trace-development-v4/README.md)では型付き参照27・対象否定2・未解決範囲1が最初の保留箇所。次は完全な主張の包みを扱うが、内側の命題が独立に既存の属性確認を通る場合だけ認める。同義の冠詞/未所有表現は、一意な同じroleのforeign ownerがある場合だけ結ぶ。対象/全体否定、競合/未知owner、歴史/rawリスク制限を維持し、同じ入力の開発F1と誤通過数を測る。新規受入とは別で、マージ・公開・採用はしない。

## 過去c20時点の限定修正と残件

[候補c20c513の記録](../evaluation/engine073-launch-development-v22-foundation/README.md)では、同じ60件のv30 F1 0→0.4615、Precision未定義→1.0（通した9件）、Recall0→0.30、API通過率0→0.15、FP0を確認。v17はF1 0.5・Recall1/3、relationが見えてもriskで保留する1件がありAPI間で混ぜない。過去初回を再採点したものではない。最初の修正のFP1は拒否し、競合する所有者の宣言を扱ってFP0へ戻した。全体1564成功、旧1080と歴史597を維持。

残る21件は現在のコードで診断してから、物理/文書属性のhas・prints・uses/測定値、引用文字の外側の属性/owner証明、別製品イベントの引用・同格、canonical Clip名による偽欠落を限定修正する。毎回同じ開発セットのF1/Recall/Precision/FPを記録する。全肯定・否定の回帰を通して候補を再固定した後に、新規独立評価を用意する。現時点で追加の新規v23を消費しない。#531実24問の最終回答/主張の根拠・引用・有用性の指標は未測定で、合成API指標では代替しない。マージ・公開・CLI採用は保留。
