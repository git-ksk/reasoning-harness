# Harness Engine 0.7.1 — Engine単独のソースパッチ

日本語 | [English](engine-0.7.1-release.md)

## リリース座標

- ソースpackage: `reasoning-harness-core 0.7.1`、`publish = false`。
- tag: `engine-v0.7.1`。固定済み`engine-v0.7.0`に続く、Engineソースのみの通常パッチ。
- [公開方針](release-policy.ja.md)：初回のGitHubプレリリース指定は、従来の0.x慣例を引き継いだ区分の誤りとして訂正します。固定したtag・commit・assetは変更しません。tag内の文書は当時のsnapshotを維持し、現在の区分はGitHub説明文と本方針に記録します。
- 公開済みCLIは **Reason CLI 0.5.4 / Engine 0.6.1** のままです。CLIへの採用とnative配布は別リリースです。#506のCLI 0.5.5 / Engine 0.6.1候補も分離して維持します。
- 今回のリリース準備は版番号と文書のみを変更し、Engine実装はマージ済みmain `3d712f6f7e76606229e014bd90cb97fbcfc9a6b3`と同一です。

## 含まれる修正

- **#514 / #515:** 対象自身を主語とする`was launched`・`has been launched`などの肯定的な発売記述を、対応する小数version・ドットを含むメーカー／製品名でも取りこぼさないように修正。主語・節の所有関係、否定・推測・架空、scope・identityの制約を保持します。[挙動と検証記録](launch-source-frames-514-515.md)。
- **#512:** AWS Lambdaの準備内容を問う限定的な日本語質問と、対象自身についての肯定的な英語抜粋を対応付けます。元の句読点と周辺文脈を検査し、否定・打消し線・引用符の除去による誤昇格、同名の別サービス、見出し等の否定・架空・撤回文脈を抑止します。[挙動と検証記録](bilingual-preparation-512.md)。
- 修正対象は後継v17/v30のみです。歴史的v16/v29の結果、凍結corpus、初回観測、既存wire/session契約ID、公開済みtagは維持します。
- ソースには既にマージ済みのWinGet doctor/lifecycle診断修正（#504）、provider fixture診断（#465）、Tokio/libc依存更新も含まれます。このEngine tagによる新しいCLIバイナリ配布はありません。旧MCP v1のtransport制約は残り、期限付き後継v2/v3を推奨します。

## 検証と限界

最終のリリース前実装はworkspace **1,468成功・失敗0・既存ignore 6**、strict all-target Clippy、formatをPASS。通常fixture 20件は正解率1.000・根拠なし受け入れ0、元のEngine 0.7観測archiveは13件のSHA-256を維持。#514/#515の独立v1-v9回帰288件と、#512の追加negative 21件をPASSしています。記録は`evaluation/launch-514-515-final-validation/`と`evaluation/bilingual-preparation-512-followup/`です。

launch corpus/reviewと以前の#512 reviewは合成データによる検証です。最終#512の句読点・文脈修正はローカルの決定論的検証で、以前の独立reviewは以前のソースに対するものです。元の期待値133件は固定したまま、132件は一致、`This claim is false.`という明示的な免責文の1件は意図的に保留へ変更しました。このパッチで新しい独立モデルreview、live-provider acceptance、実ユーザーの有用性、実機acceptanceを実施したとは主張しません。0.7.0の3provider観測は、そのリリースの歴史的証跡として保持します。

記述認識は限定的で、未対応表現では保守的に保留します。#488の出典系譜、#489の信頼済み改訂関係、#491の追加回答可否・再取得は、具体的な必要性を測定するまで保留です。

## 公開条件

リリースPRの最終headと検証済みマージ後mainで、locked format・strict Clippy・workspace tests・4platform CLI smoke・native候補／Rust不要consumer・lifecycle CIをPASSさせます。ソースcheckoutのCLI 0.5.4 / Engine 0.7.1、実装と凍結archiveの不変を確認して、そのmerge commitにtagを付けます。GitHub Releaseに正確なtag SHAとCIリンクを記録します。過去のEngine・Reason CLI artifactは不変です。
