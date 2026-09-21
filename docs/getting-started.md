# Getting Started with `reason`

[日本語](getting-started.ja.md) | English

This guide is the shortest path from a fresh machine to understanding what Reasoning Harness does. It targets the current split release: **Reason CLI 0.5.3 on Harness Engine 0.5.0**.

## 1. Install

The published native installer is the normal end-user path. Split CLI installers require GitHub CLI 2.93+ so release provenance can be verified before installation.

macOS / Linux:

```bash
curl -fsSL https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.3/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.3/install.ps1 | iex
```

With Rust 1.88+, you can alternatively install the same tagged CLI directly:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag reason-v0.5.3 --locked reasoning-harness-cli --bin reason
```

Then confirm `reason --version`. `main` may contain unreleased work; use the tagged release for a reproducible product snapshot. See [Native installer contract](native-installers.md) for provenance and platform details.

## 2. Choose a live provider or stay offline

For the natural-language path, the recommended first-run flow is:

```bash
reason setup
```

It guides provider selection, native OS credential storage, a curated model default, and a non-billable local readiness check. A live provider readiness check can consume quota or incur cost, so it runs only after explicit interactive consent or with `--live-check`.

For CI, containers, or remote shells, provider environment variables remain supported. Example with Mistral:

```bash
export MISTRAL_API_KEY='...'
```

Provider credentials are operational secrets, not trusted evidence. Use `reason auth ...` for credential management and `reason models` / `reason model set` to inspect or change the curated model default. The catalog is evidence-based compatibility metadata, not a provider availability promise or correctness score.

If you do not want to call an AI provider, skip to [Offline candidate verification](#offline-candidate-verification).

## 3. Run a grounded example

```bash
reason "Report the verified deployment region" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact service.region=us-east-1 \
  --hypothesis service.region=us-east-1
```

The model may phrase the answer, but the proposition can be grounded because the Harness owns a matching structured fact.

## 4. Run an intentionally insufficient example

```bash
reason "Is the database definitely the root cause?" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact http.status_code=503 \
  --fact db.connection_errors=7 \
  --hypothesis incident.root_cause=database
```

The observations do not prove causation. A qualified answer or `unknown` is therefore the expected safe behavior.

This is the core mental model:

```text
model confidence != verification authority
```

## 5. Add ordinary context without making it trusted

```bash
cat incident.log | reason "Analyze this incident" \
  --provider mistral \
  --model ministral-8b-latest
```

Piped text and `--file` content are model-readable context. They do not automatically become trusted structured facts.

Use `--fact`, a configured evidence-admission path, or a trusted verifier when a proposition needs hard authority.

## Offline candidate verification

If another application or AI already produced a `ReasoningCandidate`, Reasoning Harness can check it without calling an AI endpoint:

```bash
reason run \
  --input examples/input.json \
  --candidate examples/candidate.json \
  --no-config \
  --format json
```

This is useful for RAG systems, agents, recorded outputs, CI, and provider-independent testing.

## Which command should I use next?

| Goal | Command |
| --- | --- |
| Ask a natural-language question | `reason "TASK"` |
| Complete first-run setup (0.5.x product line) | `reason setup` |
| Check for a CLI update without mutation (0.5.x product line) | `reason update --check` |
| Manage provider credentials (0.5.x product line) | `reason auth ...` |
| Continue or correct a persisted reasoning state | `reason session ...` |
| Integrate existing structured candidate output | `reason run` |
| Validate an existing artifact | `reason verify` |
| Run soft semantic diagnostics | `reason semantic-check` |
| Inspect machine schemas | `reason schema` |

Next: read the [CLI guide](cli.md), then [How Reasoning Harness works](how-it-works.md).

For the broader map, return to the [documentation index](README.md).
