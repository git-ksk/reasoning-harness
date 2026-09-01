# `reason` CLI 日本語ガイド

[English CLI guide](cli.md) | 日本語

`reason`はReasoning Harnessの最初のproduct surfaceです。LLM/Agentの出力をそのまま信用するのではなく、Harness管理のevidence・verification・decision boundaryへ通すための非対話型CLIです。

`--candidate`と`--provider`の2つの使い方、AIなしでも`accept | reject | unknown`を判定できる原理は[Reasoning Harnessの仕組み](how-it-works.ja.md)で詳しく説明しています。

## 自然文AI path

人が使うprimary pathは、taskをそのまま渡せます。

```bash
reason "この障害を分析して" --fact http.status_code=503
```

provider/modelは`reason-config-v1`または明示flagから取ります。defaultはhuman-readable出力で、自動化では`--format json`を使えます。

| input | trust上の意味 |
| --- | --- |
| positional `TASK` | ユーザー依頼。evidenceではない |
| `--file PATH` | modelが読めるuntrusted context。自動でhard factにはしない |
| piped stdin | `--file`と同じuntrusted context |
| `--fact KEY=VALUE` | deterministic verification対象にできるHarness-owned structured fact |
| `--hypothesis KEY=VALUE` | 評価/解決するHarness-owned proposition |
| `--resolver-fact KEY=VALUE` | bounded resolution → admission → 再verification経由でだけ使う明示local fact |

context + target例:

```bash
cat error.log | reason "DBがroot causeか確認して" \
  --hypothesis incident.root_cause=database
```

trustedなstructured supportがなければ、結果が条件付き/`unknown`のままになることがあります。これは仕様です。`--file`に文章があるだけでverification authorityへ昇格はしません。

最終自然文もmodelがrenderするだけでは信用しません。`finalize_answer`がfactual-claim coverageを確認し、新しい事実を勝手に混ぜた場合はblockします。明示resolverで確認できる場合のみbounded resolutionへ戻し、再verification後に再renderできます。

自然文JSON出力は通常の`reason-cli-output-v1` envelope内で`output_contract: reason-natural-output-v1`を明示します。

詳しくは[仕組みの日本語解説](how-it-works.ja.md)と[product dogfood](product-dogfood.ja.md)を参照してください。

## まずどのコマンドを使う？

| やりたいこと | コマンド |
| --- | --- |
| LLM/Agentの候補回答を根拠付きでチェックしたい | `reason run` |
| 完成済みartifactが構造・根拠ルールを満たすか確認したい | `reason verify` |
| contradiction/counterexampleなどsemantic診断をしたい | `reason semantic-check` |
| 入出力JSON Schemaを確認したい | `reason schema` |

通常のアプリ統合は**`reason run`から始める**のが基本です。

## インストール

Rust 1.88+がある場合:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.1.0 --locked reasoning-harness-cli --bin reason
reason --version
```

Rustなしの場合は[v0.1.0 Release](https://github.com/git-ksk/reasoning-harness/releases/tag/v0.1.0)のstandalone binaryを利用できます。

## 最小サンプル

```bash
reason run \
  --input examples/input.json \
  --candidate examples/candidate.json \
  --no-config \
  --format json
```

`examples/input.json`がHarness管理のtask/evidence、`examples/candidate.json`がLLM/Agent相当のuntrusted candidateです。

結果ではまず次を確認します。

```text
result.outcome.verdict
result.outcome.artifact.claims
result.outcome.artifact.verification_receipts
```

`accept | reject | unknown`は推論結果です。`unknown`でもコマンド自体が正常に完了すればexit codeは`0`です。

## stdin / CI

`-`でstdinを使えます。

```bash
cat examples/input.json | reason run \
  --input - \
  --candidate examples/candidate.json \
  --no-config \
  --format json
```

artifact検証:

```bash
cat artifact.json | reason verify - --format json
```

exit code:

- `0`: 処理成功。`run`のverdictは`accept/reject/unknown`を別途JSONで確認する。
- `1`: input/provider/runtime/validationなどの処理失敗。
- `2`: CLIの引数/usageエラー。

JSON modeでは失敗時もmachine-readableなfailure envelopeを返します。

## live provider

Mistral例:

```bash
export MISTRAL_API_KEY='...'
reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

live providerは候補を生成するだけです。その出力はtrusted evidenceではなく、その後に通常のHarness検証が走ります。

対応adapter:

- Mistral: `MISTRAL_API_KEY`
- Google Gemini/AI Studio: `GEMINI_API_KEY`
- NVIDIA Hosted NIM: `NVIDIA_API_KEY`

secretは`reason-config-v1`へ保存する設計ではありません。

## config

優先順位:

1. CLI flag
2. `--config PATH`
3. project `.reason/config.json`
4. user config
5. built-in default

再現可能なCIでは`--no-config`を使うとambient configを無視できます。

Schema:

```bash
reason schema config
```

## semantic-check

D3 semantic runtimeを明示的に使うproduct commandです。

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

defaultは`semantic-decidability-d3-v1`。v3 rollback:

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --profile v3 \
  --format json
```

semantic findingはsoft diagnosticであり、trusted evidenceやfinal verdict authorityにはなりません。

## Schema discovery

```bash
reason schema artifact
reason schema candidate
reason schema config
reason schema semantic-check
```

現在のmachine contract identity:

- `reason-cli-output-v1`
- `reasoning-artifact-v1`
- `reasoning-candidate-v1`
- `reason-config-v1`
- `semantic-check-input-v1`

## もう少し詳しく

- [日本語README](../README.ja.md)
- [English full CLI guide](cli.md)
- [Product support / compatibility](support.md)
- [Product roadmap](product-roadmap.md)
