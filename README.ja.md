# Reasoning Harness

日本語 | [English](README.md)

**根拠が足りないのに、AIが自信満々で答えるのを防ぐ。**

Reasoning Harnessは、AIの出力をエビデンスと検証の境界で包むruntimeです。人が直接使うCLIが **`reason`**。AIは回答候補を作りますが、どの事実をgrounded（根拠確認済み）として出せるか、どこを条件付きにするか、何を`unknown`のまま残すかはHarness側が決めます。

> **AIはcandidate（回答候補）を作る役であって、正しさを決めるauthority（権限）ではありません。**

```text
 task + evidence
       |
       v
      AI  -> untrusted candidate
       |
       v
 Reasoning Harness
       |
       +--> grounded answer
       +--> 条件付き回答
       +--> unknown / abstain
```

## どんなときに使う？

LLMやAgentは使いたい。でも、**「AIがそう言った」だけでは結果を信用したくない**ときに使います。

たとえば:

- **RAG / 調査AI** — 実際に検証できた根拠を超えて断言しないようにする。
- **障害 / architecture分析** — 観測済みfactは返しつつ、未証明のroot causeへ勝手に昇格させない。
- **Agent / CI** — model出力を次の自動処理へ渡す前に検証する。
- **安価なLLMの活用** — candidate生成は安いmodelに任せ、信用判断はprovider-neutralなruntimeに残す。

Harnessなし:

```text
evidence -> LLM -> answer
```

Harnessあり:

```text
evidence -> LLM -> candidate -> verify / resolve -> grounded | qualified | unknown
```

## 何が変わる？

たとえば障害中に、次の2つだけが確認できているとします。

```text
HTTP status = 503
DB connection errors = 7
```

流暢なAIは、ここから簡単にこう飛躍できます。

```text
「DBが障害の原因です」
```

Reasoning Harnessは、観測factと強い因果結論を分けます。root causeを裏付けるtrustedな因果エビデンスがなければ、結果は条件付きまたは`unknown`のままです。

```text
DBがroot causeとは確認できません。
HTTP 503とconnection error 7件は観測されていますが、それだけでは因果関係は確定できません。
```

これがこのプロダクトの中心です。**AIを便利に使いつつ、AIの自信をauthorityへ変換しない。**

## Quickstart

現在のexternal previewは`v0.4.2`です。Rust 1.88+がある場合:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.4.2 --locked reasoning-harness-cli --bin reason

reason --version
```

standalone archiveと`SHA256SUMS`も[v0.4.2 Release](https://github.com/git-ksk/reasoning-harness/releases/tag/v0.4.2)から取得できます。

自然文taskと、Harnessに明示的なstructured factとして扱わせたい根拠を渡します。

```bash
export MISTRAL_API_KEY='...'

reason "確認できるdeployment regionを答えて" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact service.region=us-east-1 \
  --hypothesis service.region=us-east-1
```

次に、わざと根拠不足のケースを試せます。

```bash
reason "DBがHTTP 503のroot causeだと断定できる？" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact http.status_code=503 \
  --fact db.connection_errors=7 \
  --hypothesis incident.root_cause=database
```

強い結論を裏付ける根拠がなければ、条件付き回答や`unknown`になるのが正常な安全結果です。

**APIキーなしで試したい場合:** 外部AIが作ったstructured candidateをofflineで検証できます。[Getting Started](docs/getting-started.ja.md)を参照してください。

## 結果は何を意味する？

| 結果 | 意味 |
| --- | --- |
| **Grounded answer** | 表示するfactがHarness管理のverified stateで裏付けられている。 |
| **条件付き回答** | 確認できた観測factは出すが、未証明の強い結論は不確実なままにする。 |
| **Unknown / abstain** | 現在のtrusted evidenceでは要求された結論を安全に出せない。 |

`unknown`はepistemic（知識状態）の結果であり、自動的に処理失敗を意味するわけではありません。

## 何がevidenceになる？

大事なのは、**context（読ませる情報）とauthority（正しさを支える権限）を分けること**です。

| Input | Harness上の意味 |
| --- | --- |
| positional `TASK` | 聞きたいこと。evidenceではない。 |
| `--file PATH` / piped stdin | AIが読めるcontext。別途verifyされるまではuntrusted。 |
| `--fact KEY=VALUE` | deterministic verification対象にできる明示structured evidence。 |
| `--hypothesis KEY=VALUE` | 評価・resolveしたいproposition（命題）。 |
| external resolver / read-only MCP output | 取得データ。Harness管理のadmission（受け入れ判定）とverificationを通るまでauthorityにはならない。 |

RAGやツールが文章を返しただけでは、その文章が自動的にverified evidenceになるわけではありません。

詳しい入力・設定契約は[CLIガイド](docs/cli.ja.md)を参照してください。

## アプリ / 自動化への組み込み

### 既存LLM / RAGの回答をチェックする

アプリ側ですでにretrievalとcandidate生成を持っている場合:

```bash
reason run \
  --input retrieved-evidence.json \
  --candidate model-candidate.json \
  --format json > checked-result.json
```

次の処理はLLMの文章ではなく、Harnessのstructured resultを見て判断します。

### `reason`にcandidate生成も任せる

```bash
reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

providerが作るのはあくまで**untrusted candidate**です。その後に同じHarness管理のverification pathが走ります。

### CI / Agentのgateとして使う

```bash
reason verify artifact.json --format json
```

または:

```bash
cat artifact.json | reason verify - --format json
```

exit codeはprocess stateです。`accept | reject | unknown`を判断したいautomationはJSON resultを確認してください。

## 別のLLMに採点させるのと何が違う？

Reasoning Harnessは「この回答、正しそう？」と別のAIへ聞いて、その返事をそのまま信用する仕組みではありません。

```text
 外部AI / Agent / RAG          Harness管理input
          |                         |
          v                         v
   untrusted candidate        evidence / policy
          |                         |
          +-----------+-------------+
                      v
              safeにmaterialize
                      |
                structure validation
                      |
                evidence verification
                      |
                  diagnostics
                      |
                acceptance policy
                      |
             accept | reject | unknown
```

modelが自分で`known`や`supported`と書いても、それだけではtrusted stateになりません。strong stateはHarness境界の中で、deterministic checkや明示的にtrustedなverifierから再構築されます。

詳しくは[Reasoning Harnessの仕組み](docs/how-it-works.ja.md)を参照してください。

## 現在のプロダクトsurface

| Command | 用途 |
| --- | --- |
| `reason "TASK"` | 人が直接使う自然文path。 |
| `reason session ...` | sessionの保存・確認・追加・訂正・resume・fork・close。 |
| `reason run` | Application / CI統合、live candidate生成。 |
| `reason verify` | materialize済み`ReasoningArtifact`のdeterministic validation。 |
| `reason semantic-check` | soft semantic diagnostic。最終authorityは持たない。 |
| `reason schema` | versioned machine contractの確認。 |

Mistral、Google Gemini/AI Studio、NVIDIA Hosted NIM、Groqのprovider adapterを実装済みです。read-only MCP取得、external resolver、trusted deterministic verifier、bounded investigation、resumable sessionも現在のruntimeに含まれます。

## プロジェクトの実測をどう信用する？

このプロジェクトでは、都合の悪い観測を消して再測定するのではなく、**freezeしたevaluationを研究証跡として残し、現行releaseのclaimを再現可能な実測へ結びつける**方針を取っています。

`v0.4.2`最終release gateは、freshにfreezeした13ケースのnatural-language E2Eを使用しました。provider間の平均ではなく、required rowをそれぞれ独立にPASSさせています。

| Model / provider | v0.4.2最終実測 |
| --- | --- |
| **Mistral / Ministral 8B** | PASS — candidate 13/13、operational failure 0、correctness-boundary violation 0 |
| **Groq / GPT-OSS 120B** | PASS — candidate 13/13、operational failure 0、correctness-boundary violation 0 |
| **Gemini 3.5 Flash-Lite** | PASS — 13/13、tool selection `0.6 -> 1.0`、trigger `0/3 -> 3/3`、avoidable stall `3 -> 0` |
| **Gemma 4 31B** | PASS — candidate 13/13、operational failure 0、correctness-boundary violation 0 |

freeze座標、metric、run ID、pacing policy、provenanceは[v0.4.2 v36 release acceptance](docs/natural-language-e2e-v36-result.ja.md)に固定しています。過去のstudyは研究証跡として残しますが、通常ユーザー向けの主要導線からは分離します。

## Product / Engine / Researchを分ける

`v0.4.2`は、product CLIとreasoning engineが同じversion座標を共有する最後のreleaseです。

次のlineからは:

- **Reason CLI** — terminal UX、setup、distributionなど人向けproductをversioning。
- **Harness Engine** — reasoning / correctness behaviorをversioning。
- **Machine contract ID** — wire/schema compatibilityを独立してversioning。

次の一般向けlineは **Reason CLI 0.5.0 + Harness Engine 0.4.2** を予定しています。setup、OS credential storage、interactive UX、installer、doctor、update/uninstallなどを改善しても、Engineのcorrectness semanticsが変わったように見せないためです。

詳しくは[versioning](docs/versioning.md)と[Reason CLI 0.5.0 roadmap](docs/reason-cli-0.5-roadmap.md)を参照してください。

## Documentation

`docs/`をファイル名順・時系列順に読む必要はありません。ここには通常のproduct documentationと、freeze済みの研究/evaluation証跡が両方あります。

まず **[Documentation index](docs/README.ja.md)** から入ってください。

- 初回セットアップと日常CLI利用
- Application / CI / MCP integration
- Architectureとtrust boundary
- 現在のroadmap / project status
- 過去のresearch / evaluation evidence

を分けて案内しています。

## これは何ではない？

- 汎用chat client / coding agent。
- prompt集。
- 特定model専用agent framework。
- 別LLMの自己申告をauthorityにするpost-hoc judge。
- open-world reasoningを数学的に解決したという主張。
- compiler、test、schema、policy engine、proof checkerなどdeterministic oracleの代替。
- correctness coreへ埋め込んだ汎用crawler / RAG framework。

## 開発者向け

Rust 1.88+がsupported toolchainです。first-party runtime componentはRust-onlyです。

```bash
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

[CONTRIBUTING.ja.md](CONTRIBUTING.ja.md)、[SECURITY.ja.md](SECURITY.ja.md)、[architecture](docs/architecture.ja.md)、[Documentation index](docs/README.ja.md)を参照してください。
