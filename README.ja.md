# Reasoning Harness

日本語 | [English](README.md)

**根拠が足りないのにAIが自信満々で答えるのを防ぐCLIです。**

Reasoning Harnessは、model出力の外側にevidence / verificationの境界を置くAI CLI/runtimeです。自然文のtaskと「実際に信用できる根拠」を渡すと、AIは候補を作りますが、最終的にどこまでgroundedに答えられるかはHarness側が決めます。

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

AIは**候補を出す役**であり、正しさを決めるauthorityではありません。evidence admission、verification、不確実性、最終的なfactual claim coverageはHarness側が管理します。

## どんなときに使う？

LLMやAgentは使いたいけれど、**「AIがそう言った」だけでは結果を信用したくない**ときに使います。

たとえば:

- **RAG / 調査AI** — 取得した根拠以上のことを断言するのを防ぐ。
- **障害 / architecture分析** — 観測済みfactは返しつつ、未証明のroot causeや総合判断は未確定のままにする。
- **Agent / CI** — 次の自動処理へ渡す前にmodel出力を検証する。
- **安価なLLMの活用** — candidate生成は安いmodelに任せ、信用境界はprovider-neutralなHarnessに残す。

ざっくりいうと:

```text
普通:
  evidence -> LLM -> answer

Reasoning Harnessあり:
  evidence -> LLM -> candidate -> verify / resolve -> grounded | qualified | unknown
```

## 30秒Quickstart

### 1. 現在のv0.2.0 previewをインストール

`v0.2.0`が現在の自然文first external previewです。Rust 1.88+がある場合:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.2.0 --locked reasoning-harness-cli --bin reason

reason --version
```

standalone archiveと`SHA256SUMS`は[v0.2.0 Release](https://github.com/git-ksk/reasoning-harness/releases/tag/v0.2.0)から取得できます。`main`は未releaseの開発変更を意図的に使う場合だけ選んでください。

### 2. 自然文task + 明示factを渡す

```bash
export MISTRAL_API_KEY='...'

reason "確認できるdeployment regionを答えて" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact service.region=us-east-1 \
  --hypothesis service.region=us-east-1
```

AIはcandidateと最終文章を生成しますが、`service.region=us-east-1`を検証できる根拠は`--fact`です。provider/modelはconfigへ入れておけば毎回指定する必要はありません。

### 3. わざと根拠不足のtaskを試す

```bash
reason "DBがHTTP 503のroot causeだと断定できる？" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact http.status_code=503 \
  --fact db.connection_errors=7 \
  --hypothesis incident.root_cause=database
```

503とDB connection errorが同時に観測されても、それだけで因果関係までは証明できません。candidateとverified stateに応じて、`reason`は条件付き回答を返すか`unknown`のまま止まります。これは失敗ではなく、安全側へ判断できた正常結果です。

> **APIキーなしで試したい場合:** 外部AIが作ったcandidateをofflineで検証するstructured pathもあります。[高度なstructured実行モード](#高度なstructured実行モード)を参照してください。

## どんな回答が返る？

人向けの自然文pathは、主に3種類の結果を返します。

| 状況 | 表示 | 意味 |
| --- | --- | --- |
| requested targetを根拠で確認できる | **grounded answer** | 最終factがHarness側のverified stateでcoverされている。 |
| 観測済みfactはあるが結論までは証明できない | **条件付き回答** | 確認できるfactだけ返し、未証明の結論は未確定と明示する。 |
| 安全に回答を外へ出せない | **unknown / abstain** | 追加evidenceまたはconfigured resolverが必要。 |

たとえばHTTP 503とDB connection error 7件は確認できても、root causeを証明するcausal evidenceがなければ、概念的にはこう返します。

> DBがroot causeとは確認できません。HTTP 503とconnection error 7件は同じ時間帯に観測されていますが、それだけでは因果関係は確定できません。

重要なのは文章そのものではなく、**確認できる観測factは役立てつつ、そこから強い結論へ勝手に昇格しない**ことです。

## 何を入力するの？

普通に使う場合は、自然文taskに「実際に持っているcontext / authority」だけを足します。

| Input | Harness上の意味 |
| --- | --- |
| positional `TASK` | 聞きたいこと。**evidenceではない**。 |
| `--file PATH` / piped stdin | AIが読めるcontext。別途verifyされるまでは**untrusted**。 |
| `--fact KEY=VALUE` | Harness-ownedの明示structured evidence。deterministic verification対象にできる。 |
| `--hypothesis KEY=VALUE` | 評価・resolveしたいproposition。 |
| `--resolver-fact KEY=VALUE` | bounded resolution → admission → 再verification経由だけで使うlocal fact。 |
| `--resolver-command PROGRAM` | `main`のexternal stdio JSON resolver。取得結果はHarness-owned admissionを通るまでuntrusted。 |

`main`の#175では、external evidenceはsource allowlistとHarness-owned freshness/scope/authority policyを明示した場合だけadmitされます。resolverのauthority自己申告だけでは昇格せず、admit後も通常のqualification / verificationを再通過します。

trusted supportが足りなければ、条件付き回答や`unknown`になるのが正しい動作です。文書に文章が書かれているだけではverified evidenceにはなりません。

`HarnessInput` / `ReasoningCandidate` JSONは、アプリ統合、CI、再現性、offline candidate検証用の高度なsurfaceとして残しています。

## アプリ / 自動化への組み込み例

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

別アプリ側ですでにretrievalやcandidate生成を持っている場合は、これが基本的な**integration pattern**です。

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

## 高度なstructured実行モード

高度なintegration向けには`reason run`のstructuredな使い方も残っています。どちらも**候補ができた後の検証パイプラインは同じ**で、違うのは「untrusted candidateを誰が作るか」です。

| モード | コマンド | Harness内でAIを呼ぶ？ | 向いているケース |
| --- | --- | --- | --- |
| **外部AIの候補を持ち込む** | `reason run --input ... --candidate ...` | **呼ばない** | 自作Agent、RAG、ChatGPT/Claude/Codex的な別システムなどが、すでに構造化された候補を作っている。 |
| **`reason`に候補生成も任せる** | `reason run --input ... --provider ... --model ...` | **呼ぶ** | Mistral / Google / NVIDIAへ`reason`自身が問い合わせて候補を作り、そのまま検証したい。 |

Product commandごとに見るとこうです。

| コマンド | `reason`内でAI必要？ | 理由 |
| --- | --- | --- |
| `reason run --candidate ...` | **不要** | 既存candidateを、決定論的なmaterialization・evidence verification・diagnostics・acceptance policyへ通せる。 |
| `reason verify artifact.json` | **不要** | すでに作られたartifactの構造・invariantを検証する。 |
| `reason run --provider ...` | **必要** | untrusted candidateそのものをproviderに生成させる。 |
| `reason semantic-check ...` | **必要** | semantic runtimeはmodel-backedなsoft diagnostic surfaceだから。 |

つまりReasoning Harnessは、**必ずAI endpointへ接続しないと動かないツールではありません**。既存AIの出力をチェックするcore pathは、APIキーなしでも動きます。

## AIなしで、どうやって回答を判定できるの？

Harnessは「この文章、正しそう？」と別のAIへ聞いているわけではありません。もっと狭く、**構造化された主張が、Harness管理の根拠とルールで裏付けられるか**を確認しています。

```text
外部AI / Agent / RAG
        |
        | claimやinferenceを提案
        v
 ReasoningCandidate          HarnessInput
   (信用しない)          (task + 管理された根拠)
        |                        |
        +-----------+------------+
                    v
          1. 安全にmaterialize
                    |
                    v
          2. 構造・参照をvalidation
                    |
                    v
          3. evidenceとverification
                    |
                    v
          4. diagnosticsを実行
                    |
                    v
          5. acceptance policyで判定
                    |
        +-----------+-----------+
        |           |           |
      accept      reject      unknown
```

### 1. まずAIの自己申告を信用しない

外部AIがcandidate内で「これは`known`」「`supported`」「`inferred`」「`contradicted`」と書いても、その強いstateをそのまま採用しません。現在のmaterializationでは、それらは原則いったん`assumed`へ落とされます。`unknown`や明示的な`assumed`は、安全側の状態としてそのまま扱えます。

つまり、AIが自分で「俺の回答は検証済み」と宣言しても、権限はもらえません。

### 2. Harness側のevidenceと照合する

たとえばcandidateが次を主張したとします。

```json
{
  "proposition": {
    "key": "service.region",
    "value": "us-east-1"
  }
}
```

HarnessInputに、Harness側が管理するstructured factがあるとします。

```json
{
  "facts": {
    "service.region": "us-east-1"
  }
}
```

決定論的なstructured verifierは`key=value`を照合できます。

- 一致する根拠がある -> Harness側が`VerificationReceipt: supported`を作れる
- 別の値が確認される -> `VerificationReceipt: contradicted`になり得る
- 根拠がない -> receiptを作らず、不確実性を残す
- time/scope/authorityなどのqualification条件を満たさない -> hard receiptを出さず、不確実性を残す

**verification receiptを作れるのはtrusted boundary側で、candidateを作ったAIではありません。**

### 3. 最後に保守的なpolicyでまとめる

現在のStrict policyは分かりやすく保守的です。

- `contradicted`が1つでもある -> **`reject`**
- `assumed` / `unknown`が1つでも残る -> **`unknown`**
- claimがあり、必要なclaimが十分に確立している -> **`accept`**
- claim自体がない -> **`unknown`**

contradiction / counterexampleの探索、assumption inspection、evidence qualification、Five Whysなどのdiagnosticも走りますが、diagnosticが勝手にtrusted evidenceを作ったり、model outputだけで最終verdictを支配したりはできません。

だから、

```bash
reason run --input evidence.json --candidate ai-output.json --no-config --format json
```

は**APIキーなし**で意味のある判定を返せます。AIによる候補生成はすでに外で終わっていて、Harnessは「その候補を信用してよい部分はどこか」を決定論的な境界で判定しているからです。

さらに詳しいstate遷移、verification receipt、evidence qualification、semantic safety runtimeとの役割分担は[仕組みの詳細](docs/how-it-works.ja.md)にまとめています。[用語ガイド](docs/terminology.ja.md)では製品概念・互換性ID・過去の研究フェーズ名を分けて説明しています。

## Semantic safety check

soft semantic diagnosticが勝手に最終判断権限を持たないよう、semantic runtimeは`reason run`とは別のproduct surfaceになっています。

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

CLIでは`--profile current`（default）と`--profile rollback`を使います。再現性のためmachine configuration IDは`semantic-decidability-d3-v1` / `soft-semantic-v3`のまま維持し、従来の`d3` / `v3` selectorもaliasとして利用できます。

contradiction / counterexample / unsupported premise / causal gapなどをsemanticに診断したい場合に使う高度なsurfaceです。人が普通にtaskを依頼するなら**`reason "TASK"`**、structuredなアプリ/CI統合なら**`reason run`**から始めます。

## Product command一覧

| コマンド | 使いどころ |
| --- | --- |
| `reason "TASK"` | 人が自然文taskを依頼するprimary path。verified runtime全体を通す。 |
| `reason run` | candidate/evidenceを渡すstructuredなアプリ・CI統合path。 |
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

現在の`v0.2.0` external previewでは次を実装しています。`main`はtagより先へ進むことがあるため、再現可能なproduct snapshotが必要ならtagを基準にしてください。

- `HarnessInput` / `ReasoningCandidate` / `ReasoningArtifact`のtyped contract
- evidence binding、provenance/referenceの決定論的検証
- structured fact verificationとtrusted verification receipt
- contradiction、counterexample、assumption、causal、temporal/scope、evidence qualification診断
- `accept | reject | unknown`とfail-closed runtime
- bounded resolution/finalization primitivesと`ReasoningPolicy`
- `ReasoningThread` event/checkpoint replay primitives
- current semantic runtimeと明示的rollback profile（exact compatibility IDは再現性のため維持）
- Mistral / Google / NVIDIA provider adapter
- versioned JSON envelope、schema-backed config、stdin、typed failure class
- Linux x64 / macOS Apple Silicon・Intel / Windows x64のproduct smoke
- Ministral 3B/8B/14B / Mistral Small / Gemma 4 31B / Gemini 3.1/3.5 Flash-Liteでproduct dogfood実測済み。Gemma 4 26B A4B / Nemotron 3.5 Lightningはこのproduct workloadではprotocol-incomplete

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

次のplanned capability milestoneは **v0.3.0 — External Evidence & Resolution** (#173) です。不足根拠を特定し、実際の外部adapterから追加evidence/verificationを取得して、同じauthority boundaryを再度通し、それでも根拠不足なら無理に完成させません。read-only MCP acquisition (#176) はそのadapter経路の1つであり、新しいcorrectness boundaryではありません。

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
