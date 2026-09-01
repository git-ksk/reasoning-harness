# Reasoning Harness

[日本語](README.ja.md) | English

**Build AI answers through an evidence-grounded reasoning runtime, not a single model pass.**

Reasoning Harness is a native AI CLI/runtime for systems that should say **`unknown` instead of guessing** when the available evidence does not justify an answer. The model proposes; the harness owns the path from evidence to a grounded result.

```text
LLM / Agent / RAG
       |
       v
 structured candidate
       |
       v
 Reasoning Harness
       |
       +--> accept
       +--> reject
       +--> unknown
```

The model is a **candidate generator, not an authority**. The harness owns evidence binding, deterministic validation, verification, uncertainty, and final decision boundaries.

> Stochastic intelligence, deterministic process.

## When would I use this?

Use Reasoning Harness when your application already uses an LLM or agent, but you do not want its output to become a trusted conclusion just because the model produced it.

Common examples:

- **RAG / research assistants** — detect when retrieved evidence does not support the proposed answer.
- **AI research pipelines** — prevent a model from turning missing or conflicting evidence into a confident assertion.
- **Agents and CI workflows** — validate a structured reasoning artifact before another automated step consumes it.
- **Lower-cost models** — use inexpensive candidate generation while keeping trust decisions in a provider-neutral harness.

A useful mental model is:

```text
Without the harness:
  evidence -> LLM -> answer

With the harness:
  evidence -> LLM/agent -> candidate -> verify/diagnose -> accept | reject | unknown
```

## Product direction: natural-language-first AI CLI

The v0.1.0 preview proved the structured runtime and automation contracts. The next primary product
direction ([Issue #107](https://github.com/git-ksk/reasoning-harness/issues/107)) is to make the
AI-backed path feel like a normal terminal AI tool while keeping the verification loop underneath:

```bash
reason "Analyze this incident and explain the most supported cause"
reason "Review this architecture" --file architecture.md --file template.yaml
cat error.log | reason "Find the most supported root cause"
```

Conceptually:

```text
natural-language task
        ↓
model candidate
        ↓
Reasoning Harness
  verify / diagnose / check sufficiency
        ↓
missing support? -> bounded resolution / regenerate -> re-verify
        ↓
grounded answer | qualified answer | unknown
```

The user-facing goal is **natural language in, grounded natural language out**. Structured JSON remains
important, but primarily as an advanced integration/debug surface and as the internal representation
that keeps the runtime inspectable.

This direction deliberately uses the research program rather than bypassing it: D3, evidence binding,
unsupported-premise/causal diagnostics, evidence sufficiency and abstention, bounded resolution,
verification receipts, and final-claim coverage become mechanisms behind the simple CLI.

## What do I give it?

`reason` is currently **non-interactive and structured-data-first**. It is not a chat client where arbitrary prose is treated as trusted evidence.

For the main `reason run` path, your application provides:

1. a `HarnessInput` JSON document containing the task and harness-owned evidence; and
2. a `ReasoningCandidate` JSON document proposed by your model/agent, or a configured live provider that generates the candidate.

The CLI then materializes and checks a `ReasoningArtifact` without allowing the model to create trusted evidence or verification receipts.

Inspect the exact contracts at any time:

```bash
reason schema artifact
reason schema candidate
reason schema config
reason schema semantic-check
```

## Current v0.1.0 execution modes

The current v0.1.0 structured foundation exposes two `reason run` modes. They use the same verification pipeline; the only difference is **who creates the untrusted candidate**. Going forward, live provider generation is the basis of the primary natural-language UX; bring-your-own-candidate remains an advanced integration path.

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
| `reason semantic-check ...` | **Yes** | D3/v3 is a model-backed soft semantic diagnostic surface. |

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

For a deeper walkthrough, including state transitions, receipts, qualification, and where model-backed D3 fits, see [How Reasoning Harness works](docs/how-it-works.md).

## 30-second quickstart

Examples below use a POSIX shell. On Windows, save the same JSON documents to files and invoke `reason.exe` with equivalent paths.

### 1. Install

v0.1.0 is an external preview, not a v1.0 stability claim.

With Rust 1.88+:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag v0.1.0 --locked reasoning-harness-cli --bin reason

reason --version
```

Or download the standalone `reason` binary for Linux x64, macOS Apple Silicon/Intel, or Windows x64 from the [v0.1.0 release](https://github.com/git-ksk/reasoning-harness/releases/tag/v0.1.0). The release includes `SHA256SUMS`.

### 2. Run a self-contained offline example

No API key or repository checkout is required. Create a tiny evidence document and an untrusted candidate:

```bash
cat > /tmp/reason-input.json <<'JSON'
{
  "task": "Determine what can be concluded from the supplied evidence.",
  "evidence": [{
    "id": "e1",
    "source": "demo",
    "observation": "The source states that service.region is us-east-1.",
    "facts": {"service.region": "us-east-1"}
  }]
}
JSON

cat > /tmp/reason-candidate.json <<'JSON'
{
  "claims": [
    {
      "id": "c1",
      "statement": "The service is in us-east-1.",
      "proposed_state": "known",
      "proposition": {"key": "service.region", "value": "us-east-1"},
      "evidence_ids": ["e1"]
    },
    {
      "id": "c2",
      "statement": "The service is highly available.",
      "proposed_state": "unknown",
      "evidence_ids": []
    }
  ],
  "inferences": []
}
JSON

reason run \
  --input /tmp/reason-input.json \
  --candidate /tmp/reason-candidate.json \
  --no-config \
  --format json
```

The example contains one evidence-backed claim and one claim with no evidence for the requested conclusion. The harness therefore returns:

```json
{
  "result": {
    "outcome": {
      "verdict": "unknown"
    }
  }
}
```

That `unknown` is a successful harness outcome, not a process failure. The command exits `0`; automation should inspect the JSON verdict.

### 3. Look at what the harness changed

In the same output, the evidence-backed candidate claim is promoted through a harness-owned verification receipt to `supported`, while the unsupported claim remains `unknown`.

Useful fields include:

```text
result.outcome.verdict
result.outcome.artifact.claims
result.outcome.artifact.verification_receipts
result.outcome.artifact.*_findings
```

## Three practical usage patterns

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

This is the core product pattern.

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

## Semantic safety check

The adopted semantic runtime is available separately so a soft diagnostic can never silently become final-verdict authority:

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

The default profile is `semantic-decidability-d3-v1`. The characterized `soft-semantic-v3` profile remains available as an explicit rollback with `--profile v3`.

Use this surface when you specifically need a semantic contradiction/counterexample/unsupported-premise/causal-gap diagnostic. For normal application integration, start with `reason run`.

## Supported product commands

| Command | Use it for |
| --- | --- |
| `reason run` | Run candidate output through the harness-owned correctness process. |
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

The v0.1.0 preview includes:

- typed `HarnessInput`, `ReasoningCandidate`, and `ReasoningArtifact` contracts;
- evidence binding and deterministic provenance/reference validation;
- structured-fact verification and trusted verification receipts;
- contradiction, counterexample, assumption, causal, temporal/scope, and evidence-qualification diagnostics;
- `accept | reject | unknown` outcomes with fail-closed runtime behavior;
- bounded resolution/finalization primitives and `ReasoningPolicy` constraints;
- durable `ReasoningThread` event/checkpoint replay primitives;
- adopted `semantic-decidability-d3-v1` semantic runtime with explicit v3 rollback;
- Mistral, Google, and NVIDIA provider adapters outside the correctness authority boundary;
- versioned JSON product envelopes, schema-backed layered config, stdin support, and typed failure classes;
- credential-free product smoke on Linux x64, macOS Apple Silicon/Intel, and Windows x64.

See the [CLI guide](docs/cli.md) for the full invocation contract, the [Japanese CLI guide](docs/cli.ja.md), and [support policy](docs/support.md) for v0.x compatibility boundaries.

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

The longer-term goal is not only to diagnose bad reasoning. It is to identify what support is missing, acquire or verify additional evidence through external adapters, re-run the same authority boundaries, and refuse to fabricate completion when sufficient support cannot be established.

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
