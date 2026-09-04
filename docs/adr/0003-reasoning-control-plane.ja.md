# ADR-0003: 推論のコントロールプレーン、ポリシー、永続スレッド

- Status: Accepted
- Date: 2026-08-30

## 背景

ADR-0002 では、境界を設けた grounded-resolution loop を定めた。次のアーキテクチャ上の問いは、そのループを、ポリシーの変更、中断／再開、修復サイクル、証拠の無効化、クライアントの再接続を含む、より長期間にわたる作業でどのように動作させるかである。

成熟したエージェントハーネスには有用なコントロールプレーンのパターンがあるが、それは Reasoning Harness が汎用エージェントフレームワークになることを意味しない。現在の Codex アーキテクチャは、技術的なサンドボックス境界と承認ポリシーを分離し、スレッドのライフサイクルと永続化をランタイムに保持し、クライアントが再接続できるイベントストリームを公開している。LangGraph も同様に、スレッド単位のチェックポイントを永続的な実行状態として扱い、resume／fork をチェックポイント操作としてモデル化している。Claude Code は再開／分岐可能なセッション、ライフサイクルフック、権限、任意のサブエージェントを公開している。これらはアーキテクチャ上の影響要因にすぎず、正しさの権威でもランタイム依存でもない。

## 決定

Reasoning Harness は、既存の artifact／resolution runtime の周囲に、推論に特化した小規模なコントロールプレーンを追加する。中核概念は次のとおりである。

1. `ReasoningPolicy` — 明示的な権限と、昇格／エスカレーションのルール。
2. `ReasoningThread` — 実行や修復をまたぐ推論セッションの永続的な識別子。
3. 型付きの append-oriented events — 受け入れ済みランタイム状態を再構成するために必要な来歴。
4. 明示的な checkpoint／resume／fork の意味論。
5. ポリシー変更による無効化と、影響を受ける状態の決定論的な再評価。

プロジェクトでは、2つ目の resolver／evidence-provider 抽象化は追加しない。取得は #22 の `ResolutionResolver -> EvidenceAdmissionPolicy / TrustedResolutionVerifier` 境界を通じて継続する。

## 成熟したHarness概念との対応付け

| Mature harness concept | Reasoning Harness analogue | Decision |
|---|---|---|
| execution sandbox | evidence/inference promotion policy | `ReasoningPolicy` として採用 |
| approval policy | promotion/escalation policy | `ReasoningPolicy` 内に採用 |
| thread/session | durable `ReasoningThread` | 採用 |
| event stream | typed reasoning/provenance events | 採用 |
| checkpoint | reconstructable verified runtime snapshot | 採用 |
| resume/fork | continue or branch from a stable checkpoint | 採用 |
| tool boundary | #22 resolver/admission/verifier boundary | 再利用し、重複させない |
| scoped instructions | global/domain/run policy layering | 汎用的なポリシー合成として採用 |
| post-action validation | proposition -> evidence -> edge -> artifact -> final-answer ladder | 採用 |
| retry/repair | #22 repair + full re-verification | すでに採用済み |
| skills/subagents | specialist semantic workers | ベンチマークの証拠が得られるまで保留 |

## エビデンスと推論のサンドボックス

推論サンドボックスは、モデルが提案した状態が grounded output に影響できる範囲を制限する。これはポリシーであり、別個の実行環境ではない。

初期モードは、1つのポリシーオブジェクトに対する概念的なプリセットである。

- `strict`: 直接検証された命題だけを昇格できる。
- `bounded`: 明示的に許可された決定論的推論クラスだけが派生サポートを生成できる。
- `exploratory`: 未検証の仮説を作業状態に残せるが、grounded factual output には入れられない。

実際の要件が明らかになったら、実装ではモード enum よりも明示的な capability／field を優先する。プリセットが既存の verifier の権威を迂回することは決してあってはならない。

## 昇格とエスカレーションのポリシー

`ReasoningPolicy` は真理ではなく、許可された状態遷移を所有する。ポリシーは次の選択肢を持てる。

- 検証済み状態を保持する。
- 条件付き／不確実な状態を保持する。
- 証拠を要求する。
- 決定論的な検証を要求する。
- 修復／再生成を要求する。
- 人間によるレビューを要求する。
- 拒否する。
- unknown／棄権で終了する。

ソフトな意味論上の所見は、ポリシーが追加作業のトリガーに使う場合でも、助言にとどまる。所見が直接 verification receipt や hard finding を作成することはできない。

## ReasoningThreadとイベントモデル

`ReasoningThread` は、1つの推論調査のための永続コンテナである。隠れた chain-of-thought の保存や公開を必要としてはならない。永続化する状態は、明示的な型付きランタイム artifact と制御イベントだけで構成する。

候補となるイベント群:

- タスク／質問を受信した。
- 証拠を取得／受け入れ／条件付けした。
- 候補を提案／置換した。
- claim または edge を検証／反証した。
- 診断所見を提起した。
- 解決／修復／人間レビューを要求した。
- ポリシーを変更した。
- 状態を無効化した。
- checkpoint を作成した。
- 回答を確定した。

イベントは、受け入れ済みの作業中／検証済み状態を再構成するのに十分な、安定したエンティティ ID と因果参照を持つべきである。大きな生 payload は、すべてのイベントへ重複して格納する代わりに、content-addressed にしてもよい。

## 中断、再開、フォーク

- **interrupt** は安全な checkpoint 境界を記録する。不完全な作業を証拠へ変換してはならない。
- **resume** は、明示的に把握されたポリシーバージョンの下で checkpoint から継続する。
- **fork** は、元の履歴を保持したまま、過去の checkpoint から新しいスレッド系譜を作成する。
- 同じ schema／policy バージョンの下では、ハーネスが所有する状態の replay／再構成は決定論的でなければならない。

外部 resolver における副作用は引き続きアダプターが所有し、冪等にするか外部で重複排除しなければならない。ハーネス状態の replay は、外部副作用の replay を意味してはならない。

## ポリシーの合成と無効化

ポリシーは汎用的に層を重ねる。

```text
global policy
  -> domain policy
  -> task/run policy
```

コアが知るのは、権威しきい値、freshness／scope 要件、許可された推論 capability、resolver クラス、エスカレーションルールなどの汎用 field だけである。ドメイン固有のソース名や分類体系はコアの外部に置く。

ポリシー変更は追記専用の履歴であり、過去イベントの編集ではない。ランタイムは、受け入れ済みの証拠、receipt、claim、edge、確定結果のうち、もはや受け入れ可能でないものを計算し、明示的な無効化イベントを発行する。推論を再開する前に、下流の依存状態を再評価する。

より厳格なポリシーによって、支持がもはや適格でない結論を暗黙に保持してはならない。

## 検証ラダーと無効化の伝播

検証は、影響を受けた最小単位から段階的に広げる。

1. 命題／schema の妥当性。
2. 証拠の適格性と、証拠から claim への支持。
3. 推論／因果 edge の妥当性。
4. ローカルな依存チェーンの整合性。
5. artifact レベルの整合性と判断。
6. 最終的な事実 claim のカバレッジ。

上流の支持が変わった場合、より広い検査を実行する前に、依存する検証／派生状態を無効化する。これはモデルによる自己修正ではなく、依存関係の伝播である。

## 修復権限の不変条件

修復は #22 のランタイムプリミティブであり続ける。置換候補は次の条件を満たす。

- 未信頼の状態から始まる。
- 過去の所見をコンテキストとしてのみ受け取れる。
- 検証の権威を何も継承しない。
- 正規化、検証、適格性評価、検証、診断、ポリシー、確定の各段階へ再投入される。
- 上流の命題が異なる場合、下流の状態を無効化する。

## 保留・却下した概念

測定された利益が得られるまで保留するもの:

- skills。
- subagents／マルチエージェント・オーケストレーション。
- 第一級のコア概念としての、専門的な evidence-seeker／critic agent。
- 汎用 workflow graph DSL。

却下するもの:

- 隠れた chain-of-thought をランタイム要件として永続化すること。
- #22 resolver と競合する並列の evidence-provider インターフェース。
- ポリシー、confidence、retrieval、意味論判定が検証の権威を作り出すこと。
- 汎用コアにドメイン固有のポリシールールを置くこと。

## 実装の順序

この ADR は、将来の実装トラックを2つ定める。

1. #27 — `ReasoningPolicy` の合成、capability／promotion ルール、依存関係の無効化。**Implemented.**
2. #28 — `ReasoningThread` のイベント、checkpoint／resume／fork、決定論的な再構成。**Implemented.**

両方のコントロールプレーン・トラックは現在実装済みである。#13 の soft-judge calibration も実装済みである。型付きのソフト所見は、権威を持たないスレッド観測として記録できるが、replay の権威を得ることは決してない。

## 研究リファレンス

- OpenAI, “Unlocking the Codex harness: how we built the App Server”: https://openai.com/index/unlocking-the-codex-harness/
- OpenAI, “Running Codex safely at OpenAI”: https://openai.com/index/running-codex-safely/
- LangGraph persistence/interrupt/time-travel documentation: https://docs.langchain.com/oss/python/langgraph/persistence and https://docs.langchain.com/oss/python/langgraph/use-time-travel
- Claude Code Agent SDK/session documentation: https://docs.anthropic.com/en/docs/claude-code/sdk

これらの参照は、コントロールプレーンの分離だけを動機付けるものである。Reasoning Harness は、プロバイダーに依存しない独自の正しさと権威のモデルを維持する。
