# Reasoning Harness

[日本語](README.ja.md) | English

**Stop AI from turning missing evidence into confident answers.**

Reasoning Harness is an evidence-grounded AI runtime with a native CLI called **`reason`**. The model proposes an answer; the Harness decides which factual claims are actually supported, which need qualification, and which must remain unknown.

> **The model is a candidate generator, not an authority.**

```text
 task + evidence
       |
       v
      model  -> untrusted candidate
       |
       v
 Reasoning Harness
       |
       +--> grounded answer
       +--> qualified answer
       +--> unknown / abstain
```

## Why use it?

Use Reasoning Harness when an LLM or agent is useful, but **"the model said so" is not enough to trust the result**.

Typical uses:

- **RAG / research assistants** — keep answers inside the evidence that was actually verified.
- **Incident / architecture analysis** — expose observations without silently upgrading correlation into root cause.
- **Agents and CI** — check model-produced results before another automated step consumes them.
- **Lower-cost models** — use a cheaper model for candidate generation while keeping trust decisions in a provider-neutral runtime.

Without the Harness:

```text
evidence -> LLM -> answer
```

With the Harness:

```text
evidence -> LLM -> candidate -> verify / resolve -> grounded | qualified | unknown
```

## See the difference

Suppose an incident contains two verified observations:

```text
HTTP status = 503
DB connection errors = 7
```

A fluent model can easily jump to:

```text
"The database caused the incident."
```

Reasoning Harness keeps the stronger causal claim separate from the observations. If no trusted causal evidence establishes the root cause, the safe result remains qualified or unknown:

```text
The database is not confirmed as the root cause.
HTTP 503 and seven connection errors were observed, but that does not establish causation.
```

That distinction is the product: **useful AI output without letting model confidence create authority.**

## Quickstart

`v0.4.2` is the current external preview. With Rust 1.88+:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.4.2 --locked reasoning-harness-cli --bin reason

reason --version
```

Standalone archives and `SHA256SUMS` are also available from the [v0.4.2 release](https://github.com/git-ksk/reasoning-harness/releases/tag/v0.4.2).

Give `reason` a natural-language task plus evidence you actually want the Harness to treat as a structured fact:

```bash
export MISTRAL_API_KEY='...'

reason "Report the verified deployment region" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact service.region=us-east-1 \
  --hypothesis service.region=us-east-1
```

Now try a deliberately insufficient case:

```bash
reason "Is the database definitely the root cause?" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact http.status_code=503 \
  --fact db.connection_errors=7 \
  --hypothesis incident.root_cause=database
```

A qualified answer or `unknown` is a successful safety outcome when the evidence does not justify a stronger conclusion.

**No provider key?** You can also verify an externally generated structured candidate completely offline. See the [Getting Started guide](docs/getting-started.md).

## What can the result mean?

| Result | Meaning |
| --- | --- |
| **Grounded answer** | The exposed factual claim is covered by Harness-owned verified state. |
| **Qualified answer** | Supported observations can be shown, but an unsupported stronger conclusion stays explicitly uncertain. |
| **Unknown / abstain** | The Harness cannot safely expose the requested conclusion from the available trusted evidence. |

`unknown` is an epistemic result, not automatically an operational failure.

## What counts as evidence?

The important distinction is between **context** and **authority**:

| Input | Harness meaning |
| --- | --- |
| positional `TASK` | What you want answered. Not evidence. |
| `--file PATH` / piped stdin | Model-readable context. Untrusted until separately verified. |
| `--fact KEY=VALUE` | Explicit structured evidence eligible for deterministic verification. |
| `--hypothesis KEY=VALUE` | The proposition you want evaluated or resolved. |
| external resolver / read-only MCP output | Acquired data. It still has to pass Harness-owned admission and verification. |

A sentence appearing in a document does not become true merely because a retriever or model returned it.

For the full input/configuration contract, see the [CLI guide](docs/cli.md).

## Application and automation patterns

### Check an existing LLM or RAG answer

If your application already owns retrieval and candidate generation:

```bash
reason run \
  --input retrieved-evidence.json \
  --candidate model-candidate.json \
  --format json > checked-result.json
```

Gate the next step on the structured Harness result rather than on the model prose itself.

### Let `reason` generate the candidate

```bash
reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

The provider still produces only an **untrusted candidate**. The same Harness-owned verification path runs afterward.

### Use it in CI or an agent pipeline

```bash
reason verify artifact.json --format json
```

or:

```bash
cat artifact.json | reason verify - --format json
```

Exit status represents process state, not epistemic state. Scripts that care about `accept | reject | unknown` should inspect the JSON result.

## How does it work without trusting another LLM judge?

Reasoning Harness does not ask another model, "Does this answer sound correct?" and then trust the answer. It separates proposal from authority:

```text
 External AI / Agent / RAG        Harness-owned input
           |                         |
           v                         v
    untrusted candidate       evidence / policy
           |                         |
           +-----------+-------------+
                       v
                materialize safely
                       |
                validate structure
                       |
                verify evidence
                       |
                run diagnostics
                       |
                acceptance policy
                       |
             accept | reject | unknown
```

A model cannot self-certify a claim by labeling it `known`, `supported`, or `contradicted`. Strong state must be re-established inside the Harness boundary through deterministic checks or explicitly trusted verifiers.

Read [How Reasoning Harness works](docs/how-it-works.md) for the detailed execution model.

## Current product surface

| Command | Use it for |
| --- | --- |
| `reason "TASK"` | Primary human-facing natural-language path. |
| `reason session ...` | Persist, inspect, add to, correct, resume, fork, or close reasoning sessions. |
| `reason run` | Structured application/CI integration and live candidate generation. |
| `reason verify` | Deterministically validate a materialized `ReasoningArtifact`. |
| `reason semantic-check` | Run the soft semantic diagnostic runtime without granting it final authority. |
| `reason schema` | Inspect supported versioned machine contracts. |

Mistral, Google Gemini/AI Studio, NVIDIA Hosted NIM, and Groq provider adapters are implemented outside the correctness authority boundary. Read-only MCP acquisition, external resolvers, trusted deterministic verifiers, bounded investigation, and resumable sessions are also implemented in the current runtime.

## Why trust the project claims?

The project keeps product claims tied to frozen, reproducible evaluation evidence rather than replacing failed observations with nicer reruns.

The final `v0.4.2` release gate used a fresh frozen 13-case natural-language E2E evaluation. Every required provider row had to pass independently; no cross-model averaging was used.

| Model / provider | Final v0.4.2 release evidence |
| --- | --- |
| **Mistral / Ministral 8B** | PASS — candidate 13/13, operational failures 0, correctness-boundary violations 0 |
| **Groq / GPT-OSS 120B** | PASS — candidate 13/13, operational failures 0, correctness-boundary violations 0 |
| **Gemini 3.5 Flash-Lite** | PASS — 13/13; tool selection `0.6 -> 1.0`, trigger exposure `0/3 -> 3/3`, avoidable stalls `3 -> 0` |
| **Gemma 4 31B** | PASS — candidate 13/13, operational failures 0, correctness-boundary violations 0 |

The exact frozen coordinates, metrics, run IDs, pacing policy, and provenance are in [v0.4.2 v36 release acceptance](docs/natural-language-e2e-v36-result.md). Historical studies remain available as research evidence but are intentionally kept out of the main product path.

## Product, engine, and research are separate

`v0.4.2` is the final release where the product CLI and reasoning engine share one version coordinate.

From the next product line onward:

- **Reason CLI** versions the user-facing terminal product and distribution UX.
- **Harness Engine** versions reasoning/correctness behavior.
- **Machine contract IDs** version wire/schema compatibility independently.

The planned general-use line is **Reason CLI 0.5.0 on Harness Engine 0.4.2**. This lets setup, secure credential storage, interactive UX, installers, diagnostics, and lifecycle management improve without implying that correctness semantics changed.

See [versioning](docs/versioning.md) and the [Reason CLI 0.5.0 roadmap](docs/reason-cli-0.5-roadmap.md).

## Documentation

Do not read the `docs/` directory chronologically. It contains both product documentation and the preserved research record.

Start with the **[Documentation index](docs/README.md)**, which separates:

- getting started and daily CLI use;
- application / CI / MCP integration;
- architecture and trust boundaries;
- current roadmap and project status;
- historical research and evaluation evidence.

Japanese documentation is linked from the same index.

## What this is not

- A general-purpose chat client or coding agent.
- A prompt collection.
- A model-specific agent framework.
- A post-hoc LLM judge that can self-certify another model's output.
- A claim that open-world LLM reasoning is mathematically solved.
- A replacement for deterministic oracles such as compilers, tests, schemas, policy engines, or proof checkers.
- A web crawler or RAG framework embedded in the correctness core.

## Development

Rust 1.88+ is the supported toolchain. First-party runtime components are Rust-only.

```bash
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

See [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), the [architecture guide](docs/architecture.md), and the [documentation index](docs/README.md).
