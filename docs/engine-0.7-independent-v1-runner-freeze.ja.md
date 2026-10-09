# Engine 0.7.0 — 新規・独立の3社比較評価 v1（#492）

[English](engine-0.7-independent-v1-runner-freeze.md)

**状態：評価runner・採点規則の事前固定候補。独立holdoutのデータは、runnerをマージ・凍結するまで作成しません。**

## 固定する評価条件

候補のEngineソース座標は`4cae9326bcc3e210c3250f05772d84f00e41b645`。0.6.1の出典処理のコードは不変。旧`finalize_source_attributed_answer`の結果と、0.7.0開発候補`reconcile_source_attributed_targets`を**同一の出典・target・時点・適用範囲**で比較します。

- Mistral：`ministral-8b-2512`
- Google：`gemini-3.5-flash-lite`
- Groq：`openai/gpt-oss-120b`

各社は、独立した12ケース・同一入力・固定閾値で採点します。最低4つの適切な互換性ケース、最低6つの明確な否定・由来不明・文脈相違ケースを含めることが必須です。モデルの同義判定は補助に過ぎません。**事前固定した評価用の独立正解ラベル**が互換性を許可している場合にのみ、既存の安全な出典レビューパスで統合します。モデルだけによる承認は不可です。評価用の正解ラベルは、外部事実の真偽証明ではありません。

合格条件は、Mistral・Google・Groqそれぞれで旧0.6.1比の根拠付き有用回答が**最低1件純増**し、追加model呼出が各target最大1回、provider試行最大2回、安全性違反0、引用の欠落・捏造0、元のConflictを不正に消すケース0、replayの外部呼出0、モデルAPIの運用失敗0件。運用失敗はモデルの意味的な不明と混同しません。

## 凍結順序

1. この文書、`evaluation/engine-0.7-independent-v1-protocol.json`、Rust runner、Python採点・鮮度検査、GitHub Actions、回帰テストをまずmainにマージし、`engine-0.7-independent-v1-runner-freeze`で固定。
2. **その後で初めて**、既存の開発用fixture・観測済みholdout・8語連続の引用窓と重複しない12件を新規作成。変更不可の`engine-0.7-independent-v1-corpus-freeze`としてタグ付け。
3. corpusタグによる一度だけの正式評価を3社で開始。認証情報は既存のGitHub Secretsを使用。各providerの初回出力は新規専用ファイルで保存し、途中結果も維持。初回FAILやAPIエラーがあっても結果を書き換えない。
4. 先に固定した採点プログラムで3社ともPASSなら、[#492](https://github.com/git-ksk/reasoning-harness/issues/492)のセッション互換性・source release条件を続けて確認。FAILなら失敗した初回観測を残したまま新しい候補IDで再開。

**禁止：** 合格するよう結果に合わせて入力、正解ラベル、閾値、runner、モデルの種類を変更しないこと。結果の一部だけを選んで成功としないこと。providerの自動切替や根拠の自動再取得も不可。

現在の公開版はEngine 0.6.1、Reason CLI 0.5.4。評価合格までEngine 0.7.0をタグ付け・配布しません。