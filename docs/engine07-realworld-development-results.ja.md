# Engine 0.7 実資料による開発用比較 — 初回結果

日本語 | [English](engine07-realworld-development-results.md)

**評価は不合格・測定不十分。CLI採用はNO-GOです。** 2026-10-15 JST。[Issue #531](https://github.com/git-ksk/reasoning-harness/issues/531)／[PR #534](https://github.com/git-ksk/reasoning-harness/pull/534)。開発用の初回結果2件を保存しています。新しい独立受入や実運用の精度を示す結果ではありません。

実際の公開tagであるEngine0.6.1 (`4abee90f0501f0d8fc09b453a0b45c942fbed69e`) と0.7.2 (`296690a86e049b75ee693ae5032a71504366137c`) を、同一CLI0.5.5・provider・依存コードで別々にビルドしました。Linuxの事前検証で実体と固定対照を確認してからAPI測定しました。公開tagや元の失敗結果は変更していません。

## 評価の範囲

公式8文書・4発信元、使い回しを含む24シナリオ・通常質問17種類です。各providerで通常CLIを両Engine24件ずつ（回答可能18件）実行し、補助API24件の比較を別に行いました。指示混入の3派生資料は合成と明記しています。引用を原文全体と照合し、文脈・範囲・構造化事実は**エージェント／評価ホストが作成**しました。固定v2 READMEの「human-curated」は人間による評価完了の証拠になりません。原文からの自動抽出、自由なWeb検索、同時点の実際の矛盾、実際の訂正・系譜、実運用は未検証です。対象未確認・誤対象のhelper6件は診断専用で、安全性の合格件数にしません。

固定モデルはMistral `ministral-8b-2512`、Google `gemini-3.5-flash-lite`、Groq `openai/gpt-oss-120b`。各版の中では質問・資料・設定・予算を揃え、両Engineの実行順を交互にしました。v2とv3は条件が違うため合算しません。

## v2の初回結果 — 不合格を保持

固定commit `107c05dc3f2ffe20ea2116db84a69e57e85a1389`、[run38656022911、attempt1](https://github.com/git-ksk/reasoning-harness/actions/runs/38656022911)。[元データ・集計・検証](../evaluation/engine07-realworld-v2-results/README.md)。

| Provider | 有用な回答 0.6.1 → 0.7.2（/18） | 通常経路の実行失敗 旧／新（各/24） | 補助APIの互換表示増加 | 補助APIの実行失敗（/24） |
| --- | --- | --- | --- | --- |
| Mistral | 0 → 0 | 22 / 22 | 0 | 15 |
| Google | 14 → 14 | 3 / 3 | 0 | 12 |
| Groq | 8 → 7 | 15 / 16 | 2 | 2 |

数値は**固定した全条件を満たした有用回答数**です。Engine単体の正答率ではありません。v2はadapterの生成回数をCLIの論理操作6回上限と誤って比較し、Mistral26行を除外しました。またhelperがText指定で、厳密JSON採点ではコードフェンス付き回答がMistral15件・Google12件失敗しました。消費済みv2の条件や点数は直しません。

本当のCLI予算停止・使用量欠落もありました。Mistralは予算超過12件＋使用量不明2件、Googleは予算超過6件。Groqは予算超過10件＋使用量不明8件＋日次quota13件です。CLIの早期失敗でtraceが失われる不具合は[#535](https://github.com/git-ksk/reasoning-harness/issues/535)。Groqのrw05/rw12の退行フラグはcandidateの実行失敗・trace欠落と重なり、Engineの原因と確定できません。観測上の重大違反0件でも、未完了と既知#520により安全性合格にはできません。

## v3の開発用後継評価 — 不合格

固定commit `7432710f61d747595cb8498aa2dea9e44d2ff7fb`、[run38659305389、attempt1](https://github.com/git-ksk/reasoning-harness/actions/runs/38659305389)。[元データ・集計・検証](../evaluation/engine07-realworld-v3-results/README.md)。

helperをJsonSchema指定にし、論理操作12回・adapter生成24回・観測HTTP試行72回を分離しました。論理操作上限の変更は既に見たデータによる費用調整であり、旧draftの6回条件に合格した意味ではありません。1生成1024／累積出力4096／累積合計28096token、通常経路180秒、モデルは維持しています。元のv2失敗は保持します。

| Provider | 有用な回答 0.6.1 → 0.7.2（/18） | 通常経路の実行失敗 旧／新（各/24） | 補助APIの互換表示増加 | 補助APIの実行失敗（/24） |
| --- | --- | --- | --- | --- |
| Mistral | 5 → 5 | 6 / 6 | 2 | 0 |
| Google | 14 → 14 | 8 / 8 | 2 | 5 |
| Groq | 0 → 0 | 24 / 24 | 0 | 13 |

通常回答の有用性増加は観測できませんでした。Mistralのrw03は旧版で回答、新版で保留となる退行フラグがあります。固定モデル応答で原因を切り分ける必要があり、Engine固有の不具合とはまだ確定していません。Mistralは使用量不明による終了3件のほか、資料取得・アダプター出力の失敗があります。Googleは通常経路の180秒timeout16件（両版8件ずつ）、helperのRateLimit5件。Groqは通常経路quota47件、使用量不明1件、helperのQuota13件でした。Mistral・Googleの補助APIで各2件増えても、通常回答の改善・外部事実の認証・全provider合格にはなりません。

全providerの `complete_measurement` と `measurement_gate_pass` はfalseです。採点可能な観測上の重大違反0件と、未実行・不明の経路を分けます。通常回答では引用／出典照合の構造化表示がなく、引用対応の品質は未証明です。欠けたtraceや使用量を0で補わず、全体の呼び出し・token合計はnull、`observed_*` は観測できた部分の合計に限ります。料金も推測していません。

## Groqの実行方針を修正

v2でTPD200000の日次枠に達しました。過去のquota打ち切り・枯渇中の開始抑止を引き継がずv3を開始した点は、評価運用の不備です。分あたりの間隔では日次枠の余裕を確保できません。初回失敗を残し、この既知の枯渇中はGroqの追加呼び出しを行いません。モデル・料金tier・再試行回数を黙って変更しません。

[次回用ガードとオフライン12テスト](../evaluation/engine07-realworld-operations-v1/README.md)を実装しました。同一provider/modelの旧版・新版・helperで停止状態を共有し、最初のquota、使用量・試行数不明、途中中断、次の要求の保守的な予算不足、連続2回の容量失敗で後続APIを止めます。抑止したケースは未完了のまま残します。**凍結済みv2/v3には追加せず、新しいlive runnerへの組み込みはこれからです。** Groqの枯渇状態を記録しており、回復や共有組織の余裕は未確認です。過去holdout-v5の55K/70K/300秒を今回の大きな評価へそのまま流用しません。

[#535のソース修正PR #536](https://github.com/git-ksk/reasoning-harness/pull/536)は早期失敗・協調キャンセルで途中traceと使用量を残します。SIGKILLは保存を保証できません。次のrunnerへガードを組み込み、修正ホストを固定し、穏当なtimeout停止とprovider全体の自己予算を検証してから新しい識別子を固定します。追加のlive実行は開始していません。

## 判定と残作業

公開製品はCLI0.5.5 / Engine0.6.1を維持します。既知の[#520](https://github.com/git-ksk/reasoning-harness/issues/520)だけでも0.7.2採用は不可です。新しい後継候補で直し、0.7.2と両初回結果を保持します。その後、実際の矛盾・訂正・系譜の資料を揃え、完全なrunner固定後に別作成者の新規受入を行います。v2/v3の実行前の独立**設計レビュー**は完了していますが、新規独立受入は0/24です。その後の運用再レビューは別エージェントの利用枠到達で完了できず、独立確認済みとは記録しません。

採用チェックは1/6。「開発用の両版比較結果・限界・採用不可の判定を記録」のみ完了です。全provider／新規独立評価、修正・再受入、Engine選定、CLI統合、配布は未完了。#488/#489/#491は条件付きのままで、この不十分な観測だけを理由に新機能を増やしません。#465と#506は別枠です。
