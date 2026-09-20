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

The current published split CLI remains **Reason CLI 0.5.2 on Harness Engine 0.4.2**. The independently versioned Harness Engine line has now released **Engine 0.5.0** as `engine-v0.5.0`; a future Reason CLI release may adopt it separately. For macOS/Linux, the published native installer is the normal end-user path (GitHub CLI 2.93+ is required for provenance verification):

```bash
curl -fsSL https://github.com/git-ksk/reasoning-harness/releases/download/reason-v0.5.2/install.sh | sh
reason setup
```

With Rust 1.88+ you can also install the CLI directly:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness \
  --tag reason-v0.5.2 --locked reasoning-harness-cli --bin reason
```

Standalone native archives, installers, `SHA256SUMS`, and release provenance metadata are available from the [Reason CLI v0.5.2 release](https://github.com/git-ksk/reasoning-harness/releases/tag/reason-v0.5.2). `v0.4.2` remains the immutable final unified CLI/Engine historical release.

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
| `reason` | Reason CLI 0.5.x: start a managed TTY-only interactive session; JSON/non-TTY invocations remain non-interactive. |
| `reason -c` / `reason -r [SESSION]` | Continue the latest project session or resume/pick a managed session without learning backing file paths. |
| `reason "TASK"` | Primary human-facing natural-language path; interactive TTYs show coarse Harness progress and Ctrl+C cancels safely. |
| `reason setup` | Reason CLI 0.5.x: first-run provider, credential, model-default, and readiness setup. |
| `reason doctor` / `reason doctor --live-check` | Diagnose CLI/Engine versions, config, credential-store state, provider/model readiness, sessions, trust, MCP, and optional live/update readiness without exposing secrets. |
| `reason update` / `reason update --rollback VERSION` | Reason CLI 0.5.x: provenance-verified update and explicit rollback. |
| `reason uninstall` | Reason CLI 0.5.x: explicit uninstall with data/credential retention by default. |
| `reason auth ...` | Reason CLI 0.5.x: securely add, inspect, rotate, or remove provider credentials. |
| `reason models [provider]` | Reason CLI 0.5.x: inspect the curated model catalog, compatibility metadata, credential readiness, and current default. |
| `reason model set <provider> <model>` | Reason CLI 0.5.x: persist a curated general-use provider/model default without silent fallback. |
| `reason config list/get/set/unset/path/sources` | Reason CLI 0.5.x: inspect safe effective config, explain precedence/provenance, and edit user-level non-secret run defaults. |
| `reason session list` | List managed interactive sessions in human or JSON form. |
| `reason examples [topic]` | Show copy-paste examples for common workflows. |
| `reason completions <shell>` | Generate bash/zsh/fish/PowerShell completion code to stdout without changing shell configuration. |
| `reason --plain "TASK"` | Static accessible terminal presentation; also selected by `NO_COLOR`, `TERM=dumb`, or redirected human output. |
| `reason session ... --store PATH` | Low-level typed session compatibility surface: persist, inspect, add, correct, resume, fork, or close an explicit session file. |
| `reason run` | Structured application/CI integration and live candidate generation. |
| `reason verify` | Deterministically validate a materialized `ReasoningArtifact`. |
| `reason semantic-check` | Run the soft semantic diagnostic runtime without granting it final authority. |
| `reason schema` | Inspect supported versioned machine contracts. |
Interactive REPL details, including `/add`, multiline input, TTY/JSON dispatch, and privacy boundaries, are documented in [Interactive terminal UX](docs/reason-interactive.md).
Human output sections for verified facts, uncertainty, and evidence provenance are documented in [Human answer presentation](docs/reason-human-output.md).
Progress, retry visibility, and Ctrl+C semantics are documented in [Progress, retries, and cancellation](docs/reason-progress.md).
Provider usage, `/usage`, cumulative session accounting, and hard budget guards are documented in [Provider usage and budget guards](docs/reason-usage-budget.md).
Managed session locking, optimistic concurrency, crash recovery, and rollback compatibility are documented in [Managed session durability](docs/reason-managed-sessions.md).
Help, examples, shell completions, and plain/accessibility terminal behavior are documented in [Reason CLI help, examples, and completions](docs/reason-help.md).
Local storage, ephemeral mode, retention/purge, uninstall, and outbound-data boundaries are documented in [Local privacy and retention](docs/reason-local-privacy.md).


Mistral, Google Gemini/AI Studio, NVIDIA Hosted NIM, and Groq provider adapters are implemented outside the correctness authority boundary. Read-only MCP acquisition, external resolvers, trusted deterministic verifiers, bounded investigation, and resumable sessions are also implemented in the current runtime.

## Measured impact: what changes with the Harness?

Reasoning Harness is evaluated on whether it can **preserve useful grounded answers while preventing unsupported certainty at runtime**, not on whether answers merely sound safer.

### Matched-context raw model vs Harness

`product-external-info-v4` is a frozen matched-context evaluation built for this comparison. Of 21 cases, 18 are semantically scored and 3 exercise typed operational failures. In the primary comparison, the raw model and Harness arms receive the **same task, exact target hypothesis, evidence requirement, authority policy, and acquired external snapshot**.

| Model | Grounded target coverage | Expected-unknown preservation | False abstention | Unsupported grounded claims | Missed insufficiency |
| --- | ---: | ---: | ---: | ---: | ---: |
| **Ministral 8B** | **80% → 100%** | **53.8% → 100%** | **1 → 0** | **6 → 0** | **6 → 0** |
| **Gemma 4 31B** | 100% → 100% | **84.6% → 100%** | 0 → 0 | **2 → 0** | **2 → 0** |
| **Gemini 3.5 Flash-Lite** | 100% → 100% | 100% → 100% | 0 → 0 | 0 → 0 | 0 → 0 |
| **GPT-OSS 120B** | **80% → 100%** | **76.9% → 100%** | **1 → 0** | **3 → 0** | **3 → 0** |

Each arrow is **raw model → Harness**. Grounded target coverage measures useful success on the 5 cases that should be answerable. Expected-unknown preservation measures safe abstention on the 13 cases where the evidence should remain insufficient. Unsupported grounded claims count claims exposed as grounded without adequate support; missed insufficiency counts cases that should have remained unknown but were answered definitively.

The Harness is therefore not simply “more conservative.” On Ministral 8B and GPT-OSS 120B it increased grounded target coverage from 80% to 100% while also eliminating the observed unsafe claims. Gemini 3.5 Flash-Lite was already semantically perfect on this frozen corpus; the Harness preserved that boundary without reducing coverage.

### Token and latency observations

| Model | Harness / raw model tokens | Harness / raw accounted latency |
| --- | ---: | ---: |
| **Ministral 8B** | 1.234x | 0.642x |
| **Gemma 4 31B** | 0.673x | 1.159x |
| **Gemini 3.5 Flash-Lite** | 0.641x | 1.034x |
| **GPT-OSS 120B** | 0.938x | 0.860x |

Harnessing is not uniformly a token or latency tax. These are single-run operational observations, not stable performance rankings, and should be interpreted separately from the correctness results.

See the [external-information v4 cross-model comparison](docs/product-external-info-v4-cross-model.md) for the frozen contract, full provenance, and detailed observations.

### Safety replication on the v36 release surface

As a separate post-release supplement, five v36 safety-boundary cases that can be meaningfully compared with a raw model were frozen and rerun. This supplement does **not** rescore v36 utility or planner performance; it asks whether a model given the same policy and raw observation can preserve `unknown` without deterministic Harness enforcement.

| Model | Raw model | Harness | Raw boundary failure |
| --- | ---: | ---: | --- |
| **Ministral 8B** | 4/5 = 80% | **5/5 = 100%** | MCP generic-content non-promotion |
| **GPT-OSS 120B** | **5/5 = 100%** | **5/5 = 100%** | none |
| **Gemini 3.5 Flash-Lite** | 4/5 = 80% | **5/5 = 100%** | authority mismatch |
| **Gemma 4 31B** | 4/5 = 80% | **5/5 = 100%** | MCP generic-content non-promotion |

All four rows completed with zero operational failures and zero output-contract violations. The raw model crossed one safety boundary in three of four models; the Harness reference preserved all five cases for all four models. See the [v36 raw safety supplement](docs/v36-raw-baseline-supplement.md).

### Final v0.4.2 release gate

The final `v0.4.2` release gate serves a different purpose: it checks the product runtime on a fresh frozen 13-case natural-language E2E surface, with every required provider row passing independently and no cross-model averaging.

| Model / provider | Final v0.4.2 release evidence |
| --- | --- |
| **Mistral / Ministral 8B** | PASS — candidate 13/13, operational failures 0, correctness-boundary violations 0 |
| **Groq / GPT-OSS 120B** | PASS — candidate 13/13, operational failures 0, correctness-boundary violations 0 |
| **Gemini 3.5 Flash-Lite** | PASS — 13/13; tool selection `0.6 → 1.0`, trigger exposure `0/3 → 3/3`, avoidable stalls `3 → 0` |
| **Gemma 4 31B** | PASS — candidate 13/13, operational failures 0, correctness-boundary violations 0 |

The exact frozen coordinates, metrics, run IDs, pacing policy, and provenance are in [v0.4.2 v36 release acceptance](docs/natural-language-e2e-v36-result.md). Failed or inconclusive observations remain part of the research record rather than being overwritten by nicer reruns.

### Harness Engine 0.5.0 release acceptance

Harness Engine 0.5.0 completed final hardening on the fresh `engine-0.5-final-v3-freeze` surface and is released independently under `engine-v0.5.0`. The release candidate combines deterministic session explicit-fact continuity (#446), deterministic admitted exact-fact investigation materialization (#450), and evaluator separation of planner target-recall utility from finalization correctness (#445).

All six required provider/model rows passed independently on canonical Actions run `35457038163`; no cross-model averaging was used.

| Model / provider | Role | Fresh cases | Correctness violations | Session replay | Result |
| --- | --- | ---: | ---: | ---: | --- |
| **Mistral / Ministral 14B** | affected required | 3/3 | 0 | 0 | **PASS** |
| **Groq / Qwen 3.8 27B** | affected required | 3/3 | 0 | 0 | **PASS** |
| **Mistral / Ministral 8B** | validated reference | 3/3 | 0 | 0 | **PASS** |
| **Google / Gemini 3.5 Flash-Lite** | validated reference | 3/3 | 0 | 0 | **PASS** |
| **Google / Gemma 4 31B** | validated reference | 3/3 | 0 | 0 | **PASS** |
| **Groq / GPT-OSS 120B** | validated reference | 3/3 | 0 | 0 | **PASS** |

The grounded Averiq case required a supported exact `harness_investigation_admitted_fact_*` claim, and the Orivane correction case required a supported exact `harness_session_correction_target_*` claim, so the deterministic hardening paths had to be exercised rather than merely coinciding with a model-generated answer. The Vardelis no-result case remained fail-closed on every row.

See the [Engine 0.5.0 final-v3 result](docs/engine-0.5-final-v3-result.md) and [Engine 0.5.0 release notes](docs/engine-0.5.0-release.md) for exact coordinates and preserved raw evidence. The already-published Reason CLI 0.5.2 binaries remain on Engine 0.4.2; Engine and CLI releases advance independently.

## Product, engine, and research are separate

`v0.4.2` is the final release where the product CLI and reasoning engine share one version coordinate.

From the next product line onward:

- **Reason CLI** versions the user-facing terminal product and distribution UX.
- **Harness Engine** versions reasoning/correctness behavior.
- **Machine contract IDs** version wire/schema compatibility independently.

The published split CLI line is still **Reason CLI 0.5.2 on Harness Engine 0.4.2**. Separately, Harness Engine 0.5.0 is released as `engine-v0.5.0` from the accepted semantic line. Existing CLI artifacts are immutable; adopting Engine 0.5.0 into a distributed CLI requires a separate CLI release.

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
