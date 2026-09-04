# Reasoning Harness

[日本語](README.ja.md) | English

**Stop AI from turning missing evidence into confident answers.**

Reasoning Harness is a native AI CLI/runtime that puts an evidence-and-verification layer around model output. You give it a task and the evidence you actually trust; the model proposes an answer, while the harness decides what can be exposed as grounded, qualified, or still unknown.

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

The model is a **candidate generator, not an authority**. Evidence admission, verification, uncertainty, and final factual-claim coverage remain harness-owned.

## When would I use this?

Use Reasoning Harness when an LLM or agent is useful, but **"the model said so" is not enough to trust the result**.

Typical uses:

- **RAG / research assistants** — avoid answering beyond what retrieved evidence actually supports.
- **Incident / architecture analysis** — return the observations that are supported while keeping an unproven root cause or overall conclusion uncertain.
- **Agents and CI** — validate a model-produced result before another automated step consumes it.
- **Lower-cost models** — let a cheaper model generate candidates while keeping trust decisions in a provider-neutral runtime.

A useful mental model is:

```text
Without the harness:
  evidence -> LLM -> answer

With the harness:
  evidence -> LLM -> candidate -> verify / resolve -> grounded | qualified | unknown
```

## 30-second quickstart

### 1. Install the current v0.2.0 preview

`v0.2.0` is the current natural-language-first external preview. With Rust 1.88+:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.2.0 --locked reasoning-harness-cli --bin reason

reason --version
```

Standalone archives and `SHA256SUMS` are available from the [v0.2.0 release](https://github.com/git-ksk/reasoning-harness/releases/tag/v0.2.0). Install from `main` only when you intentionally want unreleased development changes.

### 2. Give it a task and an explicit fact

```bash
export MISTRAL_API_KEY='...'

reason "Report the verified deployment region" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact service.region=us-east-1 \
  --hypothesis service.region=us-east-1
```

The model generates and renders an answer, but the structured fact is what allows the harness to verify the proposition. Provider/model can normally come from config, so the explicit flags are optional once configured.

### 3. Try an intentionally insufficient case

```bash
reason "Is the database definitely the root cause?" \
  --provider mistral \
  --model ministral-8b-latest \
  --fact http.status_code=503 \
  --fact db.connection_errors=7 \
  --hypothesis incident.root_cause=database
```

The safe result should not promote those observations into a proven causal conclusion. Depending on the candidate and verified state, `reason` can expose a qualified answer or remain `unknown`. That is a successful safety outcome, not automatically a process error.

> **No provider key?** The advanced structured path can verify an externally generated candidate completely offline. See [Advanced structured execution modes](#advanced-structured-execution-modes).

## What will the answer look like?

The human-facing path is designed around three useful outcomes:

| Situation | User-facing behavior | Meaning |
| --- | --- | --- |
| Evidence supports the requested target | **Grounded answer** | The factual claim is covered by harness-owned verified state. |
| Some observations are supported but the requested conclusion is not | **Qualified answer** | Useful facts can be shown while the unsupported conclusion stays explicitly uncertain. |
| The harness cannot safely expose an answer | **Unknown / abstain** | More evidence or a configured resolver is required. |

For example, if HTTP 503 and seven database connection errors are verified but no causal evidence establishes the root cause, a useful qualified answer is conceptually:

> The database is not confirmed as the root cause. HTTP 503 and seven connection errors were observed in the same window, but that does not establish causation.

The important part is not the exact wording; it is that supported observations can remain useful without being upgraded into a stronger unsupported conclusion.

## What do I give it?

For normal use, start with a natural-language task and add only the context or authority you actually have:

| Input | What it means to the harness |
| --- | --- |
| positional `TASK` | What you want answered. It is **not evidence**. |
| `--file PATH` / piped stdin | Context the model may read. It stays **untrusted** until separately verified. |
| `--fact KEY=VALUE` | Explicit structured evidence owned by the harness and eligible for deterministic verification. |
| `--hypothesis KEY=VALUE` | The proposition you want evaluated or resolved. |
| `--resolver-fact KEY=VALUE` | A local fact available only through bounded resolution, admission, and re-verification. |
| `--resolver-command PROGRAM` | External stdio JSON resolver acquisition on `main`; acquired evidence remains untrusted until Harness-owned admission. |
| `resolution.mcp_readonly` config | Allowlisted read-only MCP acquisition through `mcp_readonly_v1`; MCP output is never authority by itself. |

If trusted support is missing, a qualified answer or `unknown` is expected behavior. A document merely containing a sentence does not make that sentence verified evidence.

Structured `HarnessInput` / `ReasoningCandidate` JSON remains available for applications, CI, reproducibility, and offline candidate checking.

For v0.3.0 development on `main`, an external process can also be wired through the existing bounded-resolution boundary with `--resolver-command`. The process cannot mint authority: its wire schema exposes acquisition/revision contributions only. External evidence remains fail-closed unless an explicit source allowlist and Harness-owned freshness/scope/authority policy admits it; admitted evidence still re-enters ordinary qualification and verification. See [External resolver adapters](docs/external-resolvers.md), [Read-only MCP resolver](docs/mcp-resolver.md), and [Trusted verifier](docs/trusted-verifier.md).

## Application and automation patterns

### A. Check an LLM/RAG answer before publishing it

Your application retrieves evidence and asks a model to produce a structured candidate. Feed both into `reason`:

```bash
reason run \
  --input retrieved-evidence.json \
  --candidate model-candidate.json \
  --format json > checked-result.json
```

Then gate the next step on `result.outcome.verdict` instead of trusting the model response directly.

Today, retrieval documents do **not** automatically become trusted evidence just because they came from a RAG system. Your integration must represent the evidence/provenance in `HarnessInput` (and trusted receipts/oracles where appropriate).

This is the core **integration** pattern when another application already owns retrieval or candidate generation.

### B. Let `reason` generate the candidate with a live provider

For example, with Mistral:

```bash
export MISTRAL_API_KEY='...'

reason run \
  --input evidence.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

The provider generates an **untrusted candidate**. The same harness-owned correctness process still runs afterward.

Google Gemini/AI Studio and NVIDIA Hosted NIM adapters are also implemented. Provider credentials remain environment variables and are never accepted as trusted evidence.

### C. Use it as a CI / agent safety gate

Validate a previously materialized artifact:

```bash
reason verify artifact.json --format json
```

Or use stdin in a pipeline:

```bash
cat artifact.json | reason verify - --format json
```

Process-state semantics are stable for automation:

- `0` — command completed; a successful `run` may still be `accept`, `reject`, or `unknown`.
- `1` — input, provider, runtime, validation, or other operational failure.
- `2` — CLI argument/usage error.

In JSON mode, product failures are also emitted as machine-readable failure envelopes.

## Advanced structured execution modes

The structured foundation still exposes two `reason run` modes for advanced integrations. They use the same verification pipeline; the only difference is **who creates the untrusted candidate**.

| Mode | Command shape | Does Reasoning Harness call an AI model? | Typical use |
| --- | --- | --- | --- |
| **Bring your own candidate** | `reason run --input ... --candidate ...` | **No** | Your app, RAG system, Claude/ChatGPT/Codex-like agent, or another model already produced structured output. |
| **Live provider candidate generation** | `reason run --input ... --provider ... --model ...` | **Yes** | You want `reason` itself to ask Mistral, Google, or NVIDIA for the candidate before checking it. |

Other product commands have their own AI requirements:

| Command | AI required inside `reason`? | Why |
| --- | --- | --- |
| `reason run --candidate ...` | **No** | Deterministic materialization, evidence verification, diagnostics, and acceptance policy can operate on an existing candidate. |
| `reason verify artifact.json` | **No** | Validates an already materialized artifact and its invariants. |
| `reason run --provider ...` | **Yes** | The provider is used to generate the untrusted candidate. |
| `reason semantic-check ...` | **Yes** | The semantic runtime is a model-backed soft diagnostic surface. |

So Reasoning Harness is **not inherently an AI endpoint client**. AI is optional for the core candidate-checking path.

## How can it judge a candidate without calling AI?

Because the harness does not ask, "Does this answer sound correct?" It asks narrower questions that can be checked against typed state and harness-owned evidence.

The important boundary is:

```text
External AI / Agent / RAG
        |
        | proposes claims and inference edges
        v
 ReasoningCandidate          HarnessInput
   (untrusted)          (task + owned evidence)
        |                        |
        +-----------+------------+
                    v
          1. Materialize safely
                    |
                    v
          2. Validate structure
                    |
                    v
          3. Verify against evidence
                    |
                    v
          4. Run diagnostics
                    |
                    v
          5. Apply acceptance policy
                    |
        +-----------+-----------+
        |           |           |
      accept      reject      unknown
```

A model cannot certify itself. If a candidate says a claim is `known`, `supported`, `inferred`, or even `contradicted`, the default materialization boundary does **not** trust that label. Those strong model-proposed states enter the artifact as `assumed`; only `unknown` and explicit `assumed` remain conservative as proposed.

For structured propositions, a deterministic verifier can then compare the candidate's typed `key=value` proposition with structured facts in harness-owned evidence. A matching fact can create a harness-owned `VerificationReceipt` with `supported`; a conflicting fact can create `contradicted`; missing or disqualified evidence creates no hard receipt and preserves uncertainty.

The current strict product policy is intentionally conservative:

- any `contradicted` claim -> `reject`;
- any remaining `assumed` or `unknown` claim -> `unknown`;
- otherwise, with non-empty adequately established claims -> `accept`;
- no claims -> `unknown`.

Diagnostics such as contradiction/counterexample discovery, assumption inspection, evidence qualification, and Five Whys checks are inspectable signals. They do not get to invent trusted evidence or silently override the verifier/acceptance boundary.

This is why `reason run --candidate ...` can be useful with **zero API keys**: the model work happened elsewhere, while the harness performs the trust decision with deterministic rules and explicitly trusted verifier inputs.

For a deeper walkthrough, including state transitions, receipts, qualification, and where the semantic safety runtime fits, see [How Reasoning Harness works](docs/how-it-works.md). For raw-model-vs-harness evaluation, see [product dogfood](docs/product-dogfood.md). The [terminology guide](docs/terminology.md) separates product concepts from compatibility IDs and historical research phase names.

## Semantic safety check

The adopted semantic runtime is available separately so a soft diagnostic can never silently become final-verdict authority:

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

Use the descriptive CLI selectors `--profile current` (default) and `--profile rollback`. The exact machine configuration IDs remain `semantic-decidability-d3-v1` and `soft-semantic-v3` for reproducibility; legacy `d3` / `v3` selectors remain accepted aliases.

Use this advanced surface when you specifically need a semantic contradiction/counterexample/unsupported-premise/causal-gap diagnostic. For a normal human task, start with `reason "TASK"`; for structured application/CI integration, start with `reason run`.

## Supported product commands

| Command | Use it for |
| --- | --- |
| `reason "TASK"` | Primary human-facing natural-language path through the verified runtime. |
| `reason run` | Structured application/CI path for candidate output and harness-owned evidence. |
| `reason verify` | Deterministically validate a finalized `ReasoningArtifact`. |
| `reason semantic-check` | Run the adopted soft semantic runtime without granting it final authority. |
| `reason schema` | Inspect versioned machine-readable product contracts. |

`reason eval`, `reason eval-resolution`, `reason eval-judges`, and dedicated study binaries are research/evaluation surfaces. They are not part of the v0.1 product compatibility promise.

## Why not just ask another LLM to judge the answer?

Because another model is still stochastic output. Reasoning Harness deliberately keeps authority outside model prose:

- models cannot create harness-owned evidence;
- models cannot create trusted verification receipts;
- soft semantic findings cannot directly force a trusted final answer;
- operational provider failure is not converted into semantic evidence;
- `unknown` is preserved when support is insufficient.

Deterministic oracles such as tests, schemas, compilers, databases, policy engines, or trusted human review can be integrated as evidence/verifier sources without becoming model-owned authority.

## Current capabilities

The current `v0.2.0` external preview includes the capabilities below. `main` may move ahead of the tagged release; use the tag when you need a reproducible product snapshot.

- typed `HarnessInput`, `ReasoningCandidate`, and `ReasoningArtifact` contracts;
- evidence binding and deterministic provenance/reference validation;
- structured-fact verification and trusted verification receipts;
- contradiction, counterexample, assumption, causal, temporal/scope, and evidence-qualification diagnostics;
- `accept | reject | unknown` outcomes with fail-closed runtime behavior;
- bounded resolution/finalization primitives and `ReasoningPolicy` constraints;
- durable `ReasoningThread` event/checkpoint replay primitives;
- current semantic runtime with an explicit rollback profile; exact compatibility IDs remain documented for reproducibility;
- Mistral, Google, and NVIDIA provider adapters outside the correctness authority boundary;
- versioned JSON product envelopes, schema-backed layered config, stdin support, and typed failure classes;
- credential-free product smoke on Linux x64, macOS Apple Silicon/Intel, and Windows x64.
- recorded product dogfood across Ministral 3B/8B/14B, Mistral Small, Gemma 4 31B, and Gemini 3.1/3.5 Flash-Lite; Gemma 4 26B A4B and Nemotron 3.5 Lightning remain protocol-incomplete on this product workload.

See the [CLI guide](docs/cli.md) for the full invocation contract, the [Japanese CLI guide](docs/cli.ja.md), the [terminology guide](docs/terminology.md), and [support policy](docs/support.md) for v0.x compatibility boundaries.

## What this is not

- A chat client or general-purpose coding agent.
- A prompt collection.
- A model-specific agent framework.
- A post-hoc LLM judge that can self-certify another model's output.
- A claim that open-world LLM reasoning can be made mathematically correct.
- A replacement for deterministic oracles such as compilers, tests, schemas, policy engines, or proof checkers.
- A general-purpose web crawler or RAG framework embedded in the correctness core.

## Research direction

The research question behind the project is:

> Can a small or inexpensive model become materially more reliable when its reasoning is forced through typed intermediate state, evidence binding, explicit uncertainty, adversarial passes, deterministic acceptance gates, and bounded resolution/re-verification before finalization?

The next planned capability milestone is **v0.3.0 — External Evidence & Resolution** (#173): identify missing support, acquire or verify additional evidence through real external adapters, re-run the same authority boundaries, and refuse to fabricate completion when sufficient support cannot be established. Read-only MCP acquisition is one adapter path (#176), not a new correctness boundary.

Research and product development proceed on separate tracks. New reasoning mechanisms enter the supported CLI only after calibration, independent frozen evaluation, operational stabilization, explicit runtime identity/rollback, and compatibility coverage.

See the [research plan](docs/research-plan.md), [product roadmap](docs/product-roadmap.md), and [project status](docs/project-status.md).

## Development

Rust 1.88+ is the supported toolchain. The repository intentionally has no Node.js/TypeScript runtime dependency.

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

Additional design documentation: [architecture](docs/architecture.md), [reasoning policy](docs/reasoning-policy.md), [evidence qualification](docs/evidence-qualification.md), [grounded resolution](docs/grounded-resolution.md), [ADR-0001](docs/adr/0001-interface-and-packaging-boundaries.md), and [ADR-0002](docs/adr/0002-grounded-resolution-and-finalization.md).
