# 共通CLI比較環境のGitHub確認 — 2026-10-25

[GitHub run40151642317](https://github.com/git-ksk/reasoning-harness/actions/runs/40151642317)は成功。両Engineを同じCLI・provider計測・取得制御patchで構築し、外部ネットワーク禁止の固定応答確認を完了した。これは実資料評価・F1測定・実行許可receiptではない。

| 項目 | 確認結果 |
| --- | --- |
| 構築source | `1d4bf8813d98cd333f983c3efd068d5ccc2e1221`（PR558、mainへマージ済み） |
| 実行環境 | GitHub Linux x86_64、Rust1.88.0 |
| baseline / candidate | `4abee90f` / `ab3c6322` |
| 共通CLI host | `73bbd181`（PR555の固定source） |
| core変更 | 両側に同じresolution opt-in patchのみ |
| バイナリ | 両側各5本、全10本のhash照合成功 |
| 評価器 | 51件成功 |
| 実CLI | 17経路成功（keyless取得・検証・引用を含む） |
| provider計測 | 18経路成功（通常・過大入力・usage欠落） |
| 停止・予約・再開拒否 | 7経路成功 |
| registry | 両coreの実serializer出力が一致 |
| 新しいモデルAPI / accuracy問題 / F1再計算 | いずれも0 |

## 成果物

- GitHub artifact ID: `12581944878`
- GitHubでの期限: `2026-11-08T01:48:19Z`。取得済みのローカルarchiveとhashも保持している。
- image ID: `sha256:90485cf58e79182701f4a58dc0cfe174e2ab83e1a6bec37adc8657b657726720`
- image archive SHA256: `13b12ba49435c58b04738570b0bf5d621a85b25ec420e0181563e24305cc395e`
- paired identity SHA256: `dbd1f69f5088e377716ccd578d30c2226f2b1d2d2c3feeb8e0809a4b48f4b348`
- runtime proof SHA256: `9cf7306a96b33d3e53605f83e9d487da70ae9bfa8f91f1b98d0005ef4eb5ea16`

取得後にarchive・identity・preparationのhashとsource commitを照合した。proofの状態は`BUILT_OFFLINE_NOT_EXECUTION_ADMISSION`、`API_execution_admitted=false`。

## 初回失敗と修正

最初のrun40151442710は、空のcheckoutに出力先の親ディレクトリがなく、準備開始時に停止した。PR558で親ディレクトリの作成を追加し、既存のleafは引き続き上書き拒否することを確認。ソース修正後のrun40151642317が成功した。元失敗runを消したり成功へ読み替えたりしていない。

ローカル旧scheduler検証の19/21と子準備前の短いdeadlineによる2件FAILも保持している。未変更schedulerを使い、signal handler準備を確認する後継検証がローカル・GitHubで成功した。

## 本評価前に残ること

1. Mistralの旧256-token planner切断を踏まえて、両arm同条件の後継profileを固定する。512-token案は候補で、まだ実測した改善とは扱わない。aggregate output4096等の上限と予算は維持する。
2. 新しいprofile・runtime・停止/会計・実行receiptを独立に確認する。`552a6d7`の限定source/captureレビューはPASSだが、追加workflow/proofおよび最終運用レビューは担当の利用上限で未完了。
3. Mistral/Googleの最新readinessと新しいreceiptを固定してから、既知24問を開発測定として一度だけ実行する。ownerはMistral再開可能と連絡済み。Groqの全量capacity保留は維持する。
4. 新出力の全文と引用を独立に注釈し、F1/P/R・引用・有用性・効率を採点する。fresh独立受入とCLI採用はその後。

元Google通常F1 `0.583333`、helper F1 `0.400000`、元raw/gold/注釈/初回scoreは不変。report SHA256 `bfe04912745e2236a8d1995e0166f57aa6cd396e988229fc168aafc2dea37051`。Engine0.7.3公開・CLI採用はNO_GOを維持する。
