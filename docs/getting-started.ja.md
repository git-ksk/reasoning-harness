# `reason` はじめかた

日本語 | [English](getting-started.md)

このガイドは、fresh machineからReasoning Harnessの考え方を最短で体験するための入口です。対象は現在のsplit release、**Reason CLI 0.5.3 / Harness Engine 0.5.0** です。

## 1. インストール

通常userにはpublished native installerを推奨します。split CLI installerは、install前にrelease provenanceを検証するためGitHub CLI 2.93+を必要とします。

macOS / Linux:

```bash
curl -fsSL https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.3/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.3/install.ps1 | iex
```

Rust 1.88+がある場合は、同じtagged CLIを直接installすることもできます。

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag reason-v0.5.3 --locked reasoning-harness-cli --bin reason
```

その後`reason --version`を確認します。`main`には未release変更が入る場合があるため、再現可能なproduct snapshotにはtagged releaseを使います。provenanceやplatform条件は[Native installer contract](native-installers.ja.md)を参照してください。

## 2. Live providerを使うか、オフラインで試すか決める

自然文pathの初回設定は次を推奨します。

```bash
reason setup
```

provider選択、native OS credential storeへの保存、curated model default、non-billable local readinessを案内します。live provider readiness checkはquota/costを消費し得るため、interactive promptで明示的に同意した場合、または`--live-check`指定時だけ実行します。

CI、container、remote shellではprovider environment variableも引き続き利用できます。Mistralの例:

```bash
export MISTRAL_API_KEY='...'
```

provider credentialはoperational secretであり、trusted evidenceではありません。credential管理には`reason auth ...`、curated model defaultの確認・変更には`reason models` / `reason model set`を使います。このcatalogは実測ベースのcompatibility metadataであり、provider availabilityやcorrectness scoreの保証ではありません。

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
| 初回セットアップを完了する（0.5.x product line） | `reason setup` |
| CLI updateを非破壊で確認する（0.5.x product line） | `reason update --check` |
| provider credentialを管理する（0.5.x product line） | `reason auth ...` |
| reasoning stateを継続・訂正する | `reason session ...` |
| 既存structured candidateを統合する | `reason run` |
| 既存artifactをvalidateする | `reason verify` |
| soft semantic diagnosticを実行する | `reason semantic-check` |
| machine schemaを確認する | `reason schema` |

次は[CLIガイド](cli.ja.md) → [Reasoning Harnessの仕組み](how-it-works.ja.md)がおすすめです。

全体像へ戻る場合は[Documentation index](README.ja.md)を参照してください。
