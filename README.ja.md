# Reasoning Harness

日本語 | [English](README.md)

**LLM・RAG・AIエージェントの出力に「根拠チェック」を追加するCLIです。**

Reasoning Harnessは、根拠が足りないときにAIが推測で断言するのではなく、**`unknown`と言えるようにする**ためのネイティブCLIです。

```text
LLM / Agent / RAG
       |
       v
 構造化された候補回答
       |
       v
 Reasoning Harness
       |
       +--> accept  採用可能
       +--> reject  棄却
       +--> unknown 根拠不足
```

LLMは**候補を出す役**であり、正しさを決める権限は持ちません。根拠の紐付け、決定論的な検証、verification、不確実性、最終的な判断境界はHarness側が管理します。

> Stochastic intelligence, deterministic process.

## どんなときに使う？

すでにLLMやAIエージェントを使っているけれど、**「モデルがそう言った」だけで結果を信用したくない**ときに使います。

たとえば:

- **RAG / 調査AI** — 取得した資料だけでは回答を裏付けられないときに、断言を防ぐ。
- **AIリサーチパイプライン** — 根拠不足や矛盾した情報を、モデルが自信満々の結論へ変換するのを防ぐ。
- **Agent / CI** — 次の自動処理へ渡す前に、構造化された推論成果物を検証する。
- **安価なLLMの活用** — 候補生成は安いモデルに任せつつ、「信用してよいか」の判断はprovider-neutralなHarness側に残す。

ざっくりいうと、違いはこれです。

```text
普通:
  根拠 -> LLM -> 回答

Reasoning Harnessあり:
  根拠 -> LLM/Agent -> 候補 -> 検証/診断 -> accept | reject | unknown
```

## 何を入力するの？

現在の`reason`は**非対話型・構造化データ優先**のCLIです。自由文をチャットのように投げて、それを自動的に「信用できる根拠」とみなすツールではありません。

基本の`reason run`では、アプリ側から次を渡します。

1. タスクとHarness管理の根拠を含む`HarnessInput` JSON
2. LLM/Agentが作った`ReasoningCandidate` JSON、または候補を生成するlive provider

その後、Harnessが`ReasoningArtifact`を作成・検証します。モデル自身がtrusted evidenceやverification receiptを作ることはできません。

正確なJSON SchemaはCLIから確認できます。

```bash
reason schema artifact
reason schema candidate
reason schema config
reason schema semantic-check
```

## 30秒Quickstart

以下はPOSIX shellの例です。Windowsでは同じJSONをファイルへ保存し、同等のpathを指定して`reason.exe`を実行できます。

### 1. インストール

v0.1.0は外部利用向けpreviewです。まだv1.0の安定性を宣言する段階ではありません。

Rust 1.88+がある場合:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.1.0 --locked reasoning-harness-cli --bin reason

reason --version
```

Rustを入れたくない場合は、[v0.1.0 Release](https://github.com/git-ksk/reasoning-harness/releases/tag/v0.1.0)からLinux x64 / macOS Apple Silicon / macOS Intel / Windows x64向けの単体バイナリを利用できます。`SHA256SUMS`も同梱しています。

### 2. APIキー不要のサンプルを実行

repoをcloneしていなくても、そのままコピペで試せます。小さなevidenceとuntrusted candidateを作ります。

```bash
cat > /tmp/reason-input.json <<'JSON'
{
  "task": "Determine what can be concluded from the supplied evidence.",
  "evidence": [{
    "id": "e1",
    "source": "demo",
    "observation": "The source states that service.region is us-east-1.",
    "facts": {"service.region": "us-east-1"}
  }]
}
JSON

cat > /tmp/reason-candidate.json <<'JSON'
{
  "claims": [
    {
      "id": "c1",
      "statement": "The service is in us-east-1.",
      "proposed_state": "known",
      "proposition": {"key": "service.region", "value": "us-east-1"},
      "evidence_ids": ["e1"]
    },
    {
      "id": "c2",
      "statement": "The service is highly available.",
      "proposed_state": "unknown",
      "evidence_ids": []
    }
  ],
  "inferences": []
}
JSON

reason run \
  --input /tmp/reason-input.json \
  --candidate /tmp/reason-candidate.json \
  --no-config \
  --format json
```

このサンプルには、

- `service.region=us-east-1`という根拠で確認できるclaimが1つ
- 高可用性については根拠がなく確認できないclaimが1つ

入っています。そのためHarness全体の結果は:

```json
{
  "result": {
    "outcome": {
      "verdict": "unknown"
    }
  }
}
```

となります。

`unknown`はエラーではありません。**「今ある根拠だけでは断言できない」ことを正常に判断できた結果**なので、process exit codeは`0`です。

### 3. Harnessが何をしたかを見る

同じJSONの中で、根拠が確認できたclaimはHarness側のverification receiptを通って`supported`へ進み、根拠不足のclaimは`unknown`のまま残ります。

最初はこの辺を見ると分かりやすいです。

```text
result.outcome.verdict
result.outcome.artifact.claims
result.outcome.artifact.verification_receipts
result.outcome.artifact.*_findings
```

## 実用パターン3つ

### A. LLM / RAGの回答を公開前にチェックする

アプリ側で根拠を取得し、LLMに構造化された候補回答を作らせたあと、両方を`reason`へ渡します。

```bash
reason run \
  --input retrieved-evidence.json \
  --candidate model-candidate.json \
  --format json > checked-result.json
```

その後の処理はLLMの文章そのものではなく、`result.outcome.verdict`を見て判断します。

現時点では、RAGで取得した文書が自動的にtrusted evidenceになるわけではありません。アプリ側でprovenanceを含めて`HarnessInput`のevidenceへ表現し、必要に応じてtrusted receipt/oracleへ接続します。

これがReasoning Harnessの基本的な利用方法です。

### B. 候補生成も`reason`からlive providerへ任せる

たとえばMistralの場合:

```bash
export MISTRAL_API_KEY='...'

reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

providerが生成するのはあくまで**untrusted candidate**です。その後に同じHarness管理の検証プロセスが走ります。

Google Gemini/AI Studio、NVIDIA Hosted NIMのadapterも実装済みです。APIキーは環境変数で扱い、trusted evidenceにはなりません。

### C. Agent / CIの安全ゲートとして使う

すでに作られた`ReasoningArtifact`を決定論的に検証できます。

```bash
reason verify artifact.json --format json
```

パイプでも使えます。

```bash
cat artifact.json | reason verify - --format json
```

自動化向けのexit codeは:

- `0` — コマンド処理成功。`run`の推論結果は`accept` / `reject` / `unknown`のいずれでもあり得る。
- `1` — input / provider / runtime / validationなどの処理失敗。
- `2` — CLI引数・usageエラー。

JSON modeでは失敗時もmachine-readableなfailure envelopeを返します。

## Semantic safety check

soft semantic diagnosticが勝手に最終判断権限を持たないよう、semantic runtimeは`reason run`とは別のproduct surfaceになっています。

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

defaultは`semantic-decidability-d3-v1`です。characterized済みの`soft-semantic-v3`へ戻す場合は`--profile v3`を使えます。

contradiction / counterexample / unsupported premise / causal gapなどをsemanticに診断したい場合に使います。**通常のアプリ統合はまず`reason run`から**始めるのがおすすめです。

## Product command一覧

| コマンド | 使いどころ |
| --- | --- |
| `reason run` | LLM/Agentの候補をHarness管理の検証プロセスへ通す。 |
| `reason verify` | 完成済み`ReasoningArtifact`を決定論的に検証する。 |
| `reason semantic-check` | soft semantic runtimeを、最終判断権限を与えずに実行する。 |
| `reason schema` | versionedなproduct JSON contractを確認する。 |

`reason eval`、`reason eval-resolution`、`reason eval-judges`、専用study binaryは研究・評価用です。v0.1のproduct compatibility対象ではありません。

## 「別のLLMに採点させる」のと何が違う？

別のLLMも結局は確率的なモデル出力です。Reasoning Harnessでは、モデルの文章そのものへ正しさの権限を渡しません。

- modelはHarness管理のevidenceを作れない
- modelはtrusted verification receiptを作れない
- soft semantic findingだけでtrusted final answerを作れない
- provider障害をsemantic evidenceや`abstain`へ変換しない
- 根拠不足なら`unknown`を維持できる

テスト、schema、compiler、database、policy engine、信頼されたhuman reviewなどの決定論的oracleは、evidence/verifier sourceとして統合できます。

## 現在できること

v0.1.0 previewでは次を実装しています。

- `HarnessInput` / `ReasoningCandidate` / `ReasoningArtifact`のtyped contract
- evidence binding、provenance/referenceの決定論的検証
- structured fact verificationとtrusted verification receipt
- contradiction、counterexample、assumption、causal、temporal/scope、evidence qualification診断
- `accept | reject | unknown`とfail-closed runtime
- bounded resolution/finalization primitivesと`ReasoningPolicy`
- `ReasoningThread` event/checkpoint replay primitives
- `semantic-decidability-d3-v1`と明示的v3 rollback
- Mistral / Google / NVIDIA provider adapter
- versioned JSON envelope、schema-backed config、stdin、typed failure class
- Linux x64 / macOS Apple Silicon・Intel / Windows x64のproduct smoke

詳細な使い方は[日本語CLI guide](docs/cli.ja.md)、完全な仕様は[英語CLI guide](docs/cli.md)、v0.xの互換性は[support policy](docs/support.md)を参照してください。

## これは何ではない？

- ChatGPT/Codexのような対話型チャット・汎用coding agentではありません。
- prompt集ではありません。
- 特定モデル専用のagent frameworkではありません。
- LLMが別のLLMを自己認証するpost-hoc judgeではありません。
- open-worldなLLM推論を数学的に正しくできる、という主張ではありません。
- compiler / test / schema / policy engine / proof checkerなどの決定論的oracleの代替ではありません。
- correctness coreに組み込まれた汎用web crawler / RAG frameworkではありません。

## 研究について

このプロジェクトの研究テーマは次です。

> 小型・低コストなモデルでも、typed intermediate state、evidence binding、明示的不確実性、adversarial pass、deterministic acceptance gate、bounded resolution/re-verificationを通すことで、推論の信頼性を実質的に高められるか？

目標は悪い推論を見つけるだけではありません。不足している根拠を特定し、外部adapterから追加のevidence/verificationを得て、同じauthority boundaryを再度通し、それでも根拠が足りなければ無理に完成させないruntimeを目指しています。

研究機能は、calibration → 独立したfrozen evaluation → operational stabilization → runtime identity/rollback → CLI compatibilityという昇格手順を通るまでproduct CLIへ入りません。

[Research plan](docs/research-plan.md) / [Product roadmap](docs/product-roadmap.md) / [Project status](docs/project-status.md)

## 開発者向け

Rust 1.88+を利用します。Node.js/TypeScript runtime dependencyはありません。

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p reasoning-harness-cli -- run \
  --input examples/input.json \
  --candidate examples/candidate.json \
  --no-config \
  --format json
```

設計資料: [architecture](docs/architecture.md)、[reasoning policy](docs/reasoning-policy.md)、[evidence qualification](docs/evidence-qualification.md)、[grounded resolution](docs/grounded-resolution.md)、[ADR-0001](docs/adr/0001-interface-and-packaging-boundaries.md)、[ADR-0002](docs/adr/0002-grounded-resolution-and-finalization.md)
