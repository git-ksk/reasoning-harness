# Engine 0.7.3 — 研究知見を反映した修正方針

日本語 | [English](engine-073-research-informed-repair.md)

確認日2026-10-17。[#520](https://github.com/git-ksk/reasoning-harness/issues/520)・[#531](https://github.com/git-ksk/reasoning-harness/issues/531)・[0.7.3ロードマップ](engine-0.7.3-roadmap.ja.md)に適用します。公開資料の知見、このプロジェクトでの応用、実装状況を記録します。

## 参照資料と取り入れる内容

| 資料 | 参考になる知見・技術 | 今回への応用 |
| --- | --- | --- |
| [Anthropic：Demystifying evals for AI agents](https://www.anthropic.com/engineering/demystifying-evals-for-ai-agents)（2026） | 能力評価と回帰評価を分け、実行記録と結果を読む。 | 既知の回帰・新しい文章での能力・実CLIの結果を別に記録。元のv19 FAILは受入を止める結果として維持。 |
| [OpenAI：Traces, Evals, and Codexによる改善](https://developers.openai.com/cookbook/examples/agents_sdk/agent_improvement_loop)（2026） | 実行記録とフィードバックを評価・修正へつなげる。 | 次の修正前に、実際の判定ルールと範囲を診断。コードの判断と根拠への参照を残す。 |
| [OpenAI：Codexの反復修正ループ](https://developers.openai.com/cookbook/examples/codex/build_iterative_repair_loops_with_codex)（2026） | レビュー・限定した修正・検証を明確なフィードバックで回す。 | 修正ごとに範囲・参照の仮説を置く。観測済みケースは開発用に使い、新規受入の前に候補を固定。 |
| [CheckList](https://aclanthology.org/2020.acl-main.442/)（ACL2020） | 最小機能・意味を保つ変更・方向を指定した変更で言語動作を調べる。 | 否定・引用・所有者・参照・並列・信号境界をテストの軸にする。意味を保つ変更と明示的な否定の追加を区別。 |
| [HANS](https://aclanthology.org/P19-1334/)（ACL2019） | 同じ単語や部分文への依存で、意味の含意を誤る場合がある。 | 製品名と発売文の出現だけで事実とせず、外側の否定・引用・所有者の違いを確認。 |
| [FActScore](https://aclanthology.org/2023.emnlp-main.741/)（EMNLP2023） | 回答全体の一括判定より、個々の事実と根拠を評価する。 | 将来の実資料24問では回答の各主張と根拠を記録。正しい主張があっても、根拠のない主張は相殺しない。 |
| [ALCE](https://aclanthology.org/2023.emnlp-main.398/)（EMNLP2023） | 取得から回答生成までを対象に、正しさと引用品質を評価する。 | 最終回答の根拠対応・引用の充足を、補助APIの互換表示と分けて測定。 |
| [Claude Citations](https://platform.claude.com/docs/en/build-with-claude/citations) | 引用が元資料内の位置と、明示的な索引の規則を持つ。 | 内部の根拠参照に資料ID・原文範囲・位置の単位を保持。位置の有効性と意味的な支持は別々に検証。 |
| [Anthropic：Building effective agents](https://www.anthropic.com/engineering/building-effective-agents)（2024） | 単純な処理の流れ・環境からの結果・反復の上限を重視する。 | provider共通の流れを維持し、複雑化は効果を測定できる場合に採用。 |

以下の内部表現は**このプロジェクトで検証する実装仮説**です。論文・公式資料がv19の問題解決を実証したものではありません。FActScoreは評価方法であり、自然言語の解析が正しいことを保証しません。

## 次の修正：範囲と参照を追う

1. 実際のqualification経路に開発用の判定記録を追加する。信号ID、原文範囲、事実・報告・印刷された引用文字の区別、所有者、関係、肯定・否定、参照先、保留したルールを残す。コードの判断記録であり、モデルの非公開の思考を取得するものではない。
2. 局所的な主張と参照のつながりを明示する。原文・引用境界・外側の否定を保持する。並列の各対象に所有者・型の根拠が必要で、別の性質から発売主張へ所有者を貸さない。
3. Rustの試作では原文に対するUTF-8 byte範囲を使い、範囲・文字境界・原文一致を検証する。Claudeの文字索引とは明示的な変換が必要。位置が有効でも、意味の含意や事実の正しさは別に確認する。
4. 所有者不明・欠落・対象や資料全体の否定・rawリスク・歴史v16/v29を維持する。モデル提案は未信頼の入力として扱う。小さな内部試作を検証してから公開契約の拡張を判断する。
5. 変更した動作ごとに最小機能・不変性・方向性の開発テストを置き、初回結果は保持する。意味変更後は新候補を固定してから新規独立受入。既存のソース・実資料24問の受入条件は変更しない。

## 今回実装したもの

[アーカイブ診断v1](../evaluation/engine073-launch-research-diagnostics-v1/README.md)を追加しました。固定444ファイルを検証し、保存済み採点から30対を集計します。Engine・採点器・providerの実行は0。元のFAIL49/60は不変で、両側成功19対・肯定側だけ失敗11対・否定側の違反0です。失敗11件は全て`local_qualification_disagreement`を持ちますが、保存出力には内部のルール経路・原文範囲と参照のつながりがありません。診断に必要な情報の不足を示し、11件の実装原因を確定したものではありません。

保存結果の集計、初回出力の改変、未記録ファイルの追加、過去証拠の改変、最終manifestの差し替えを5件で検証しました。このアーカイブ診断自体は意味判定を修正しません。常設の判定記録API・汎用の主張graphは未実装です。

その後、[一時的な実経路計測](../evaluation/engine073-launch-trace-development-v1/README.md)で11件の保留箇所を特定し、計測module/hookを除去して元sourceを完全復元。[候補fd091b1の修正](../evaluation/engine073-launch-development-v19-repair/README.md)で、属性参照を直前の完全な主張/同じroleへ結び付け、並列roleを次の述語まで保ち、別製品の明示ownerを扱いました。観測済み960件・関連83/全体1547成功。途中で見つかった報告節の退行とREDも保存。新規独立v20は初回FAIL51/60（安全性の漏れ3・肯定取りこぼし6）です。限定した内部の範囲・参照処理であり、汎用graphや新しい公開契約ではありません。

実資料24問は0/24、CLI採用は1/6です。実0.6.1と後継候補の比較、重大な根拠なし主張、有用な回答率、引用、矛盾・時間の扱い、運用コストは、完全なhost・評価器・モデル・数値予算の固定後に測定します。異なる合成セットから一般精度・改善率・完成率は計算しません。Groq停止、live callback/dependency・数値予算の未完了は維持します。[v19の初回記録](../evaluation/engine073-launch-independent-v19/README.md)も参照してください。
