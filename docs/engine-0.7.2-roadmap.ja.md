# Harness Engine 0.7.2 — 発売根拠の安全性パッチ

日本語 | [English](engine-0.7.2-roadmap.md)

**計画段階・未実装・未リリース。** [Milestone #11](https://github.com/git-ksk/reasoning-harness/milestone/11) · [0.7系の親#486](https://github.com/git-ksk/reasoning-harness/issues/486)。更新日: 2026-10-13。

次のEngineソースパッチでは、**0.7.1で再現した関連性判定の回帰2件**を直します。公開済み`engine-v0.7.0`と`engine-v0.7.1`は変更しません。配布中のReason CLI 0.5.4はEngine 0.6.1です。#506の別CLI候補やEngine 0.7のCLI採用には、それぞれ別の検証が必要です。

## 必須の修正範囲

| Issue | 再現した問題 | 必要な動作 |
| --- | --- | --- |
| [#520 上位の文脈](https://github.com/git-ksk/reasoning-harness/issues/520) | 見出し・タイトル・別文が「架空・撤回済み・偽」と示す発売記述を、v17/v30が肯定の事実として採用する | 肯定判定の前に、別signalや別文から対象記述を修飾する文脈を保持する |
| [#521 元の否定記号](https://github.com/git-ksk/reasoning-harness/issues/521) | 正規化で取り消し線・否定記号が消え、残った肯定文をv17/v30が採用する | 正規化前の原文で記号の適用範囲を検査し、通常の句読点・ドット付き名称の認識は維持する |

別々の修正として追跡し、安全性・肯定例の回帰確認を共有します。通常の0.7.2公開には両方の解決が必要です。問題は**関連性／根拠取得判定**であり、後段の誤回答や外部事実の`Known`/`Supported`への誤昇格を再現したとは主張しません。

[保存した再点検記録](../evaluation/engine-0.7.1-rereview-20261013/README.md)に、版・入力・public API再現コード・出力・hash・検証ログがあります。root作成の合成56件で、肯定7件は成功しました。negative49件は上位文脈28件・記号12件が誤Relevant、残り9件は保留です。同じ40入力は**公開済み0.7.0のv30**ではAmbiguousでした。これは開発・監査用の証拠で、独立評価や実利用の不具合率ではありません。

## 実装・受け入れの順序

1. 両Issueのv17/v30直接回帰を追加し、今回の失敗記録を保持する。変更は後継経路に限定し、歴史的v16/v29と凍結済みartifactを維持する。
2. 肯定の発売文、提供元の付記、小数・ドット付きの製品／ベンダー名、日英の準備記述、中立の複数文、別対象だけへの否定・記号を保護する。別対象・条件・疑問・噂・架空・出典bindingの既存ガードも維持する。複数文や句読点を一律に拒否しない。
3. 候補の版と採点条件を固定後、**別の作成者による新しい**negative／positive評価を初回実行する。作成者・provenance・正確なsource SHA・初回観測・採点を記録し、誤Relevant 0と肯定例の認識退行なしを必須にする。今回の56件を新しい独立評価へ読み替えない。特定の外部レビューサービスは必須にしない。
4. 正確な候補でlocked workspace tests、strict all-target Clippy、format、標準fixture、元archive、該当する契約／replay検査をPASSさせる。provider／運用失敗は別集計し、出典照合の権限境界と元のConflict・引用の保持も確認する。
5. [公開方針](release-policy.ja.md)に沿った専用ソースリリースPRで、限定範囲・証拠・限界を明記する。必要条件を満たしてから新しいimmutableな`engine-v0.7.2`を通常公開する。必須acceptanceが残る候補を公開する場合は、明示的なプレリリース接尾辞と昇格条件を付ける。この計画は公開の実施や完了を意味しない。

## 0.7系でほかに残るもの

上記2件が**現時点で再現できた修正バックログ**です。すべての不具合を発見した証明ではありません。再点検では、出典照合の権限／replayの追加22件、既存workspace 1,468件（既存ignore 6件）、strict Clippy、format、fixture20件、凍結archive13件の照合も成功しました。成功はその検査範囲に限り、新しいprovider・実ユーザーの有用性・独立した人間の承認検証は未実施です。

既存の機能仮説3件は、このパッチの外で保留します。

| 保留Issue | 採用前に必要な実証 |
| --- | --- |
| [#488 出典系譜／独立した裏付け](https://github.com/git-ksk/reasoning-harness/issues/488) | 同じ原典の重複が問題になる利用場面の実測と、信頼できる系譜情報の供給元。引用・URLの数だけでは独立性を証明できない |
| [#489 信頼済みの改訂／置き換え](https://github.com/git-ksk/reasoning-harness/issues/489) | 改訂・時点指定での残課題の実測と、明示的で信頼できる置き換え関係。取得日時の新しさだけでtruthを決めない |
| [#491 回答可否／予算内の追加取得](https://github.com/git-ksk/reasoning-harness/issues/491) | 既存のneed／qualification制御では解決しないタスク全体の失敗と、安全性・有用性／効率の改善の実測 |

これらは[Milestone #10](https://github.com/git-ksk/reasoning-harness/milestone/10)に残します。元の0.7.0ソース範囲は公開済みです。3件は将来の実装の必要性を判断する実験であり、既知の不具合3件や今回の修正パッチの必須条件ではありません。#465は旧MCPのCI fixture安定性調査、#506はCLI配布の検証として別に追跡します。
