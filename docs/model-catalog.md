# Provider/model catalog

[日本語](model-catalog.ja.md) | English

**Status:** implemented on the Reason CLI 0.5.0 development line. The tagged `v0.4.2` release predates `reason models` / `reason model set`.

Reason exposes a **curated compatibility catalog**, not a live copy of every model name a provider happens to advertise. This distinction matters: provider existence does not prove that a model satisfies Reason's current structured-generation/runtime protocol.

## Commands

```bash
reason models
reason models mistral
reason models --configured
reason model set mistral ministral-8b-latest
```

`reason models` is read-only. `reason model set` writes only the user config's `run.provider` and `run.model`; existing non-secret run and resolution settings are preserved. The update is staged and committed atomically from the CLI perspective, and Unix user config files are written with mode `0600`.

An unlisted model is **never silently substituted**. You can still use an arbitrary provider model explicitly with `--model` for research/advanced use, but the product-facing setter accepts only catalog entries marked for general use.

## Compatibility labels

- `validated` — used by the canonical v0.4.2 release-acceptance evidence for that provider identity.
- `observed` — completed a relevant frozen product workload with the current adapter, but is not the canonical release-acceptance identity.
- `limited` — known observations show a protocol/capability limitation for current Reason roles. The model remains visible for provenance/research but cannot be saved as a general-use default.

These labels are **operational compatibility metadata, not correctness scores**. Harness correctness authority still comes from evidence admission, verification, and finalization—not from the chosen model.

## Initial curated catalog

| Provider | Model | Compatibility | General-use default? | Evidence coordinate |
| --- | --- | --- | --- | --- |
| Mistral | `ministral-8b-latest` | `validated` | yes, recommended | v0.4.2 release acceptance |
| Mistral | `ministral-14b-latest` | `observed` | yes | frozen `product-external-info-v4` |
| Google | `gemini-3.5-flash-lite` | `validated` | yes, recommended | v0.4.2 release acceptance |
| Google | `gemma-4-31b-it` | `validated` | yes | v0.4.2 release acceptance |
| Groq | `openai/gpt-oss-120b` | `validated` | yes, recommended | v0.4.2 release acceptance |
| Groq | `qwen/qwen3.8-27b` | `observed` | yes | frozen `product-external-info-v4` |
| Groq | `openai/gpt-oss-20b` | `observed` | yes | frozen `product-external-info-v4` |
| NVIDIA | `nvidia/nemotron-3.5-lightning-30b-a3b` | `limited` | no | semantic D3 negative-control evidence |

The catalog is intentionally conservative. Models can be added or moved between labels only when the repository has matching runtime evidence. Model retirement/provider lifecycle handling is tracked separately; Reason must not silently replace a configured model with a different identity.

## Credential state

Catalog output reports credential readiness as a status such as `available`, `missing`, `invalid_environment`, or `credential_store_unavailable`. It never returns credential bytes or masked credential fragments. Environment variables retain their documented precedence over the OS credential store.

## Config boundary

`reason model set` edits the **user** config only. It does not write project `.reason/config.json`, approve project trust, store secrets, or alter Harness Engine 0.4.2 reasoning/authority semantics.
