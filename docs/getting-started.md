# Getting Started with `reason`

[日本語](getting-started.ja.md) | English

This guide is the shortest path from a fresh machine to understanding what Reasoning Harness does. It targets the current `v0.4.2` external preview.

## 1. Install

With Rust 1.88+:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.4.2 --locked reasoning-harness-cli --bin reason

reason --version
```

You can also use a standalone release archive for Linux x86_64, macOS arm64, macOS x86_64, or Windows x86_64.

`main` may contain unreleased work. Use the tagged release when you want a reproducible product snapshot.

## 2. Choose a live provider or stay offline

For the natural-language path, configure a provider credential. Example with Mistral:

```bash
export MISTRAL_API_KEY='...'
```

Provider credentials are operational secrets, not trusted evidence.

On the Reason CLI 0.5.0 development line, you can instead store the credential in the native OS credential store without editing shell startup files:

```bash
reason auth login mistral
```

The tagged `v0.4.2` release predates `reason auth`, so its reproducible setup remains the environment-variable form above.

On the same 0.5.0 development line, you can discover tested model choices and persist a user default without remembering provider model IDs:

```bash
reason models mistral
reason model set mistral ministral-8b-latest
```

The catalog is evidence-based compatibility metadata, not a provider availability promise or correctness score. The tagged `v0.4.2` release predates these model-management commands too.

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
| Manage provider credentials (0.5.0 development) | `reason auth ...` |
| Continue or correct a persisted reasoning state | `reason session ...` |
| Integrate existing structured candidate output | `reason run` |
| Validate an existing artifact | `reason verify` |
| Run soft semantic diagnostics | `reason semantic-check` |
| Inspect machine schemas | `reason schema` |

Next: read the [CLI guide](cli.md), then [How Reasoning Harness works](how-it-works.md).

For the broader map, return to the [documentation index](README.md).
