# `reason` はじめかた

日本語 | [English](getting-started.md)

このガイドは、current `v0.4.2` external previewを使ってReasoning Harnessの考え方を最短で体験するための入口です。

## 1. インストール

Rust 1.88+がある場合:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.4.2 --locked reasoning-harness-cli --bin reason

reason --version
```

Linux x86_64、macOS arm64、macOS x86_64、Windows x86_64向けのstandalone release archiveも使えます。

`main`には未release変更が入る場合があります。再現可能なproduct snapshotが必要ならtagged releaseを使ってください。

## 2. Live providerを使うか、オフラインで試すか決める

自然文pathではprovider credentialを設定します。Mistralの例:

```bash
export MISTRAL_API_KEY='...'
```

provider credentialはoperational secretであり、trusted evidenceではありません。

Reason CLI 0.5.0開発ラインでは、初回は`reason setup`だけでprovider選択、native OS credential storeへの保存、recommended model default、non-billable local readinessまで設定できます。

```bash
reason setup
```

live provider readiness checkはquota/costを消費し得るため、interactive promptで明示的に同意した場合、または`--live-check`を指定した場合だけ実行します。個別のcredential管理には`reason auth ...`を使えます。

tagged `v0.4.2` releaseにはまだ`reason auth`がないため、再現可能なv0.4.2手順は上のenvironment variable形式です。

同じ0.5.0開発ラインでは、provider固有のmodel IDを暗記せず実測済み候補を確認してuser defaultへ保存できます。

```bash
reason models mistral
reason model set mistral ministral-8b-latest
```

このcatalogは実測ベースのcompatibility metadataであり、provider側の現在availability保証やcorrectness scoreではありません。tagged `v0.4.2` releaseにはmodel管理commandもまだ含まれません。

AI providerを呼びたくない場合は[Offline candidate verification](#offline-candidate-verification)へ進んでください。

## 3. 根拠ありの例を試す

```bash
reason "確認できるdeployment regionを答えて" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact service.region=us-east-1 \
  --hypothesis service.region=us-east-1
```

文章はAIが生成しても、`service.region=us-east-1`というpropositionはHarness-owned structured factと一致するためgroundingできます。

## 4. わざと根拠不足のケースを試す

```bash
reason "DBがHTTP 503のroot causeだと断定できる？" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact http.status_code=503 \
  --fact db.connection_errors=7 \
  --hypothesis incident.root_cause=database
```

この観測だけでは因果関係を証明できません。条件付き回答または`unknown`が期待される安全動作です。

中心となる考え方はこれです。

```text
model confidence != verification authority
```

## 5. 普通のcontextを読ませる

```bash
cat incident.log | reason "この障害を分析して" \
  --provider mistral \
  --model ministral-8b-latest
```

stdinや`--file`の内容はmodel-readable contextです。自動的にtrusted structured factへ昇格しません。

hard authorityが必要なpropositionには、`--fact`、configured evidence-admission path、trusted verifierなどを使います。

## オフラインでcandidateを検証する

別アプリや外部AIがすでに`ReasoningCandidate`を作っている場合、AI endpointを呼ばずにHarnessで検証できます。

```bash
reason run \
  --input examples/input.json \
  --candidate examples/candidate.json \
  --no-config \
  --format json
```

RAG、Agent、recorded output、CI、provider-independent testに向いた使い方です。

## 次にどのコマンドを使う？

| 目的 | Command |
| --- | --- |
| 自然文で質問する | `reason "TASK"` |
| 初回セットアップを完了する（0.5.0開発ライン） | `reason setup` |
| provider credentialを管理する（0.5.0開発ライン） | `reason auth ...` |
| reasoning stateを継続・訂正する | `reason session ...` |
| 既存structured candidateを統合する | `reason run` |
| 既存artifactをvalidateする | `reason verify` |
| soft semantic diagnosticを実行する | `reason semantic-check` |
| machine schemaを確認する | `reason schema` |

次は[CLIガイド](cli.ja.md) → [Reasoning Harnessの仕組み](how-it-works.ja.md)がおすすめです。

全体像へ戻る場合は[Documentation index](README.ja.md)を参照してください。
