# Harness Engine 0.7.2 — 発売根拠の安全性パッチ

日本語 | [English](engine-0.7.2-release.md)

## リリース座標

- package: `reasoning-harness-core 0.7.2`、`publish = false`。
- 公開tag: `engine-v0.7.2`。[公開方針](release-policy.ja.md)に沿った**Engine単独の通常ソースパッチ**（`prerelease=false`・`latest=false`）。[PR #525](https://github.com/git-ksk/reasoning-harness/pull/525)の`296690a86e049b75ee693ae5032a71504366137c`で公開。[#524](https://github.com/git-ksk/reasoning-harness/issues/524)に完了を記録します。tag内文書は準備時点のsnapshotを保持し、現在のノートとGitHubリリースに公開結果を記録します。
- 実装は受け入れ済み[PR #523](https://github.com/git-ksk/reasoning-harness/pull/523)のマージ`0e5b3389af61d55422205b83830c634f49d024bd`と同一。リリース準備ではpackage版・lock記録・CI／版識別の固定期待値・文書だけを更新します。
- `evidence_relevance.rs` SHA-256: `5343e722618721e58ad6a24b12c55c1272eb3c581bb3ada26b24d8596dfebbf5`。
- 公開CLIは**0.5.4 / Engine 0.6.1**を継続。ソースcheckoutはCLI 0.5.4 / Engine 0.7.2ですが、新しいCLI binaryやcrates.io packageは配布しません。CLIへの採用や#506のCLI 0.5.5 / Engine 0.6.1候補は別工程です。

## 修正内容

- **#520 上位文脈:** 見出し・title・別文の架空・撤回・明示的な偽の文脈を、肯定的に見える発売断片の判定まで保持します。否定的文脈の適用範囲を確定できない場合は保留し、明確に別対象や局所的機能だけを指す場合は正当な発売記述を維持します。
- **#521 元の記号:** 正規化前の取り消し線・否定記号と、文をまたぐ取り消し線の範囲を検査。正当な句読点、ドット付き名称、対象の所有関係を維持します。
- 対象は後継の関連性判定**v17**・materialization **v30**。誤`Relevant`／要求関係の受け入れを修正するもので、後段の誤回答や外部事実の`Known`／`Supported`への誤昇格を再現したとは主張しません。
- 歴史的v16/v29、machine/session契約ID、凍結corpus、初回観測、公開済みEngine/CLI tagは維持。#488 出典系譜・#489 信頼済み置き換え・#491 回答可否／追加取得は今回の範囲外で保留します。

## 検証と限界

[修正報告](launch-context-markers-520-521.md)／[検証記録](../evaluation/launch-520-521-final-validation/README.md)に、ソース座標、元の56件監査、初回観測を保存しています。別作成者の新規v10初回評価はcorpus固定後、候補`2f12d4c61546e60d31026d9b691c9eb369c37957`で**50/50**（肯定25・否定25、誤Relevant・否定例の要求関係受け入れ・肯定取りこぼし・API error各0）。以前の初回失敗を新規独立評価として採点し直しません。

修正ソースは**1,490 PASS / 0 FAIL / 8 ignored**、strict all-target Clippy、format、標準fixture **20/20**・根拠なし受け入れ0、元archive **13/13**ファイルのhash一致、出典照合の権限／replay **22/22**が成功。旧v16/v29は公開済み0.7.1と**597個の完全入力**で出力全体がbyte一致。ignoreの6件は既存、2件は後から導入したfactuality条件と衝突する旧v1/v3の元期待値を保持したrunnerです。修正した期待値は既知データの開発用controlで別途検査します。

限定的な合成入力・ローカル・CIの証拠です。新しいlive-provider、外部事実のtruth、実ユーザーの有用性、実機、独立した人間の承認検証は主張しません。未対応の表現では保守的に保留する場合があります。0.7.0の過去provider観測を今回の新規受け入れとは扱いません。別CLI配布のWindows実機条件は、このEngine単独ソースtagの公開条件には含めません。

## 最終公開確認

- [リリースPR #525](https://github.com/git-ksk/reasoning-harness/pull/525)の最終head `f6c80c1d6c65bd0031ed72733de9a75fe7e9987d`: **23/23チェック成功**。
- 公開merge `296690a86e049b75ee693ae5032a71504366137c`: **22/22チェック成功**。treeは検証済み最終headと同一。
- 最終候補のローカル検証: locked offline workspace **1,490成功／失敗0／ignore 8**、strict all-target Clippy、format、fixture **20/20**・根拠なし受け入れ0、元archive **13/13**、ソースcheckout doctor **CLI 0.5.4 / Engine 0.7.2**。ローカル集計とソース／hash不変の検査はPR #525に記録。
- 実装は受け入れ済み#523とbyte同一。元の凍結証拠と既存公開tagは不変。初回の版固定CI失敗は保持し、修正後の最終headで全チェックが成功しました。

- [credential-store-smoke](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132718): SUCCESS。
- [lifecycle-smoke](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132689): SUCCESS。
- [ci](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132621): SUCCESS。
- [fresh-install-acceptance](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132769): SUCCESS。
- [cli-platform-smoke](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132738): SUCCESS。

通常のEngine単独公開として`latest=false`を要求します。共通`/releases/latest`はCLI選択APIではなく、CLI consumerは`reason-v*`と検証済みCLI/Engineの組み合わせを選びます。新しいCLI binaryやcrates.io packageは配布しません。

マージ後macOS Intel CLIの初回はrunnerのgithub.com接続・DNS障害でビルド前に失敗しました。同一commitでそのジョブだけ1回再試行し、[初回記録](https://github.com/git-ksk/reasoning-harness/actions/runs/38494132738/attempts/1)は保持しています。独立した意味評価の採点し直しとは区別します。
