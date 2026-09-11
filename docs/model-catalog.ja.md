# Provider / model カタログ

日本語 | [English](model-catalog.md)

**状態:** Reason CLI 0.5.0開発ラインで実装。tagged `v0.4.2` releaseにはまだ`reason models` / `reason model set`は含まれません。

Reasonが表示するのは、providerが公開している全model名のlive copyではなく、**Reason側で実測根拠を持つcurated compatibility catalog**です。providerにmodelが存在することと、Reasonの現在のstructured generation / runtime protocolへ適合することは別です。

## コマンド

```bash
reason models
reason models mistral
reason models --configured
reason model set mistral ministral-8b-latest
```

`reason models`はread-onlyです。`reason model set`が変更するのはuser configの`run.provider` / `run.model`だけで、既存のnon-secret run設定やresolution設定は保持します。書き込みはCLIから見てstaged + atomic commitし、Unixのuser configはmode `0600`で保存します。

catalogにないmodelへ**黙ってfallbackすることはありません**。研究・advanced用途では従来どおり`--model`で任意IDを明示できますが、product-facing setterでdefault保存できるのはgeneral-use対象のcatalog entryだけです。

## Compatibility label

- `validated` — そのprovider identityがcanonical v0.4.2 release acceptanceで使われている。
- `observed` — 現行adapterで関連するfrozen product workloadを完走したが、canonical release-acceptance identityではない。
- `limited` — 現行Reason roleに対するprotocol / capability limitationが実測されている。provenance / research用に一覧へ残すが、general-use defaultには保存できない。

これは**operational compatibility metadataであってcorrectness scoreではありません**。最終的なcorrectness authorityはmodel選択ではなく、Harnessのevidence admission / verification / finalizationが持ちます。

## 初期curated catalog

| Provider | Model | Compatibility | General-use default? | Evidence coordinate |
| --- | --- | --- | --- | --- |
| Mistral | `ministral-8b-latest` | `validated` | 可・recommended | v0.4.2 release acceptance |
| Mistral | `ministral-14b-latest` | `observed` | 可 | frozen `product-external-info-v4` |
| Google | `gemini-3.5-flash-lite` | `validated` | 可・recommended | v0.4.2 release acceptance |
| Google | `gemma-4-31b-it` | `validated` | 可 | v0.4.2 release acceptance |
| Groq | `openai/gpt-oss-120b` | `validated` | 可・recommended | v0.4.2 release acceptance |
| Groq | `qwen/qwen3.8-27b` | `observed` | 可 | frozen `product-external-info-v4` |
| Groq | `openai/gpt-oss-20b` | `observed` | 可 | frozen `product-external-info-v4` |
| NVIDIA | `nvidia/nemotron-3.5-lightning-30b-a3b` | `limited` | 不可 | semantic D3 negative-control evidence |

catalogは意図的に保守的です。repository内に対応するruntime evidenceがある場合だけentry追加・label変更します。model retirement / provider lifecycleは別trackで扱い、設定済みmodelをReasonが勝手に別identityへ置換することはしません。

## Credential状態

catalog outputは`available`、`missing`、`invalid_environment`、`credential_store_unavailable`などのcredential readinessだけを表示します。credential bytesやmasked fragmentは返しません。environment variableがOS credential storeより優先される既存precedenceも維持します。

## Config境界

`reason model set`が編集するのは**user configだけ**です。project `.reason/config.json`、project trust、secret storage、Harness Engine 0.4.2のreasoning / authority semanticsには触れません。
