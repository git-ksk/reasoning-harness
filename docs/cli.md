# Native `reason` CLI

The native Rust `reason` executable is the first supported product surface for Reasoning Harness.
It invokes the same harness-owned validation, evidence, verification, decision, and finalization
boundaries used by the core runtime. Research binaries and `eval*` commands remain useful, but they
do not define the stable product contract.

For the execution/trust model behind `--candidate` versus `--provider`, including how an AI-free run can still produce `accept | reject | unknown`, see [How Reasoning Harness works](how-it-works.md).

## Installation

### Current external preview (`v0.2.0`)

The natural-language-first path is included in the current tagged preview. With Rust 1.88+:

```bash
cargo install --git https://github.com/git-ksk/reasoning-harness --tag v0.2.0 --locked reasoning-harness-cli --bin reason
reason --version
```

This installs only the supported `reason` product binary, not the research binaries. Standalone `v0.2.0` archives are also published for Linux x86_64, macOS arm64, macOS x86_64, and Windows x86_64, with `SHA256SUMS`. Use `main` only for intentionally unreleased development snapshots.

`v0.2.0` remains an external preview under the v0.x support policy even though the documented v1.0 readiness gate has been satisfied. The version number is a product/distribution coordinate; it does not create a new frozen research generation.

## Natural-language AI path

The primary human-facing path accepts a task directly:

```bash
reason "Analyze this incident" --fact http.status_code=503
```

Provider/model can come from `reason-config-v1` or explicit flags. Human output is the default; use `--format json` for automation. Useful inputs are:

| input | trust semantics |
| --- | --- |
| positional `TASK` | user request, not evidence |
| `--file PATH` | untrusted model-readable context; no hard fact is inferred automatically |
| piped stdin | same untrusted context semantics as `--file` |
| `--fact KEY=VALUE` | explicit harness-owned structured fact eligible for deterministic verification |
| `--hypothesis KEY=VALUE` | harness-owned proposition to evaluate/resolve |
| `--resolver-fact KEY=VALUE` | explicit local fact available only through bounded resolution/admission/re-verification |
| `--resolver-command PROGRAM` + `--resolver-arg ARG` | external stdio JSON acquisition adapter; output is untrusted acquired data by default |
| `--resolver-timeout-ms N` / `--resolver-max-response-bytes N` | per-process operational bounds for the external resolver; both must be > 0 |

Example with context plus a typed target:

```bash
cat error.log | reason "Determine whether the database is the root cause" \
  --hypothesis incident.root_cause=database
```

If the context does not contain trusted structured support, the safe result may remain qualified/unknown. This is intentional. `--file` is not a shortcut that promotes document prose into verification authority.

`main` carries the v0.3.0 external resolver lane from #174/#175. `--resolver-command PROGRAM` launches one explicitly configured executable without shell interpolation and exchanges `reason-external-resolver-request-v1` / `reason-external-resolver-response-v1` JSON over stdio. The response can contain acquired evidence or a candidate revision, but cannot contain trusted metadata, verification receipts, verdicts, or final prose. External evidence remains fail-closed unless `resolution.external_command.admission` explicitly allowlists its source and supplies Harness-owned freshness/scope/authority policy; resolver-reported acquisition metadata cannot self-elevate. Admission rejection reasons are typed on resolution-attempt telemetry, and admitted evidence still re-enters ordinary qualification and verification. #178 also adds process timeout/response-size bounds, typed operational terminal states, actual call/latency/optional cost telemetry, and hashed adapter/admission config identities. The command adapter does not generically retry authorization/policy/protocol failures. See [External resolver adapters](external-resolvers.md). A separately configured `resolution.mcp_readonly` lane uses an allowlisted MCP 2026-07-28 stdio `tools/call` as the same acquisition-only boundary; see [Read-only MCP resolver](mcp-resolver.md). An optional `resolution.trusted_command` lane provides explicitly trusted deterministic verification; see [Trusted verifier](trusted-verifier.md).

The final model-rendered answer is also untrusted until `finalize_answer` checks factual-claim coverage. Any newly introduced factual proposition is blocked; when an explicitly configured resolver can verify it, the proposition may re-enter bounded resolution and then be rendered again. If the artifact already authorizes an original requested hypothesis as exact `Known`/`Supported`, deterministic recovery may correct renderer-only omission, exact-key drift, or an exact-target downgrade from `grounded` to `uncertain`. The downgrade path is entered only when the renderer emitted that same exact requested proposition as `uncertain`; authority still comes exclusively from artifact state. Under artifact-global `Unknown`, recovery remains target-only `QualifiedPartialAnswer`. Under artifact-global `Reject`, the global verdict is never overridden, but the successor may expose a target-only `QualifiedPartialAnswer` if the target has direct evidence-bound trusted `Supported` verification and the typed artifact proves structural isolation from every problematic non-target claim (different key, typed blocker, no shared evidence, no inference/dependency path, and no target-local contradiction/qualification/hard adversarial signal). Any ambiguous dependency fails closed. No recovery parses prose, fuzzy-matches proposition keys, creates new authority, or skips the normal answer-safety gate.

Provider transport reliability is kept outside Harness authority. Google/Gemini keeps bounded temporary-429 retry with `Retry-After`, but quota-classified 429 responses fail fast. HTTP 500/502/503/504 may retry at most twice, an otherwise valid success response with no model text may retry once, and the combined Google request is capped at four provider HTTP attempts. Credentials, quota, ordinary 4xx/provider errors, malformed successful responses, unsupported capability, transport interruption, and timeout remain fail-fast in this policy. `provider_attempts` reports the actual adapter HTTP-attempt count; if the Harness performs a separate structured-output fallback call, those adapter attempt counts are summed. Retry exhaustion remains a typed operational failure and never becomes semantic `unknown`, evidence, or abstention.

The natural-language path also runs the current semantic + evidence-sufficiency safety checks before exposing grounded factual claims. These checks are **restrictive only**: they may require more verification, bounded resolution, or abstention, but they cannot turn model confidence into trusted evidence or an `accept` verdict. Supported partial facts can still be shown without requiring them to answer the whole task.

The default is `--safety-profile current` (`verified-target-answer-gate-v1`). Use `rollback` to reproduce the previous claim-local gate (`d3-sufficiency-answer-gate-v2`); legacy `d3-sufficiency` / `d3-sufficiency-v2` selectors are aliases for that rollback. `legacy-v1` / `d3-sufficiency-v1` and `baseline` remain older testing/rollback surfaces; see [Semantic runtime product surface](#semantic-runtime-product-surface) and the [terminology guide](terminology.md) for exact machine identities.

Natural JSON output declares `output_contract: reason-natural-output-v2` inside the normal `reason-cli-output-v1` envelope.

See [How Reasoning Harness works](how-it-works.md) and [product dogfood](product-dogfood.md).

## Which command should I use?

| Goal | Command |
| --- | --- |
| Ask a person-facing natural-language question through the verified runtime | `reason "TASK"` |
| Integrate an existing LLM/agent candidate with structured evidence | `reason run` |
| Validate an already-materialized artifact | `reason verify` |
| Run contradiction/counterexample/unsupported-premise/causal-gap semantic diagnostics | `reason semantic-check` |
| Inspect the exact machine-readable JSON contracts | `reason schema` |

For a human using the CLI directly, start with **`reason "TASK"`**. For application/CI integration and externally generated candidates, start with **`reason run`**. Neither path treats arbitrary prose as trusted evidence.

## Product commands

- `reason run` — execute the harness-owned correctness process from a recorded candidate or live
  provider candidate generation.
- `reason verify` — deterministically validate a `ReasoningArtifact`.
- `reason semantic-check` — execute the current semantic runtime as a soft diagnostic coordinate, with an explicit rollback profile.
- `reason schema` — print the versioned JSON Schema for supported product wire contracts.

`reason eval`, `reason eval-resolution`, and `reason eval-judges` are research/evaluation surfaces.
Their JSON is intentionally not covered by the CLI product-envelope compatibility promise yet.

## Non-interactive input

Following established CLI automation practice, `-` means standard input for supported JSON inputs.
Only one source per command may consume stdin.

```bash
# Harness input from stdin, candidate from a file.
cat examples/input.json | reason run \
  --input - \
  --candidate examples/candidate.json \
  --format json

# Validate an artifact from stdin.
cat examples/artifact.json | reason verify - --format json
```

For `reason run`, `--input`, `--candidate`, and `--receipts` may each use `-`, but no more than one of
them may do so in the same invocation. Live `--provider` generation can be combined with
`--input -` because the provider does not consume stdin.

## stdout and stderr

For supported commands:

- successful `--format json` output writes one JSON document to stdout;
- human-readable progress/warnings and failure diagnostics go to stderr;
- stdout is therefore safe to redirect or pipe when JSON mode succeeds;
- human output is presentation only and is not a machine contract.

Provider text, model confidence, and human rendering never acquire correctness authority merely by
appearing in CLI output.

## JSON product envelope

`reason run --format json`, `reason verify --format json`, and `reason schema` emit a versioned
envelope:

```json
{
  "schema_version": "reason-cli-output-v1",
  "command": "run",
  "cli_version": "0.2.0",
  "contracts": {
    "artifact": "reasoning-artifact-v1",
    "candidate": "reasoning-candidate-v1",
    "config": "reason-config-v1"
  },
  "result": {}
}
```

The envelope version is independent from the executable semver. A future incompatible machine
output change requires a new envelope version rather than silently changing the meaning of
`reason-cli-output-v1`.

Inspect the current wire schemas directly:

```bash
reason schema artifact
reason schema candidate
reason schema config
```

## Exit semantics

Exit status is process state, not epistemic state:

| code | meaning |
| ---: | --- |
| `0` | command completed successfully; for `run`, this includes `accept`, `reject`, and `unknown` |
| `1` | runtime, provider, I/O, JSON, harness-state, or artifact-validation failure |
| `2` | CLI syntax/argument parsing failure emitted by `clap` |

In particular, `unknown` or semantic abstention is not automatically a process failure. Scripts
must inspect the JSON result when they care about epistemic outcome.

When a supported product command is explicitly in JSON mode, process failures remain machine-readable. `run` and `verify` emit the same product envelope with a failed result such as:

```json
{
  "schema_version": "reason-cli-output-v1",
  "command": "run",
  "result": {
    "status": "failed",
    "failure": {
      "failure_class": "input",
      "message": "<stdin>: missing field `task` at line 1 column 2"
    }
  }
}
```

Provider failures use normalized classes such as `credentials`, `rate_limit`, `quota`, `timeout`, `provider_unavailable`, and `protocol`. Configuration/input/harness failures use `configuration`, `input`, and `harness_state`. The process still exits 1. For automation, pass `--format json` explicitly (or use a valid config that resolves to JSON) so failures before normal command output can also be serialized rather than rendered as human diagnostics.

## Provider credentials and configuration

Non-secret run configuration is schema-backed and layered. The supported precedence is:

1. explicit CLI flags;
2. an explicit `--config PATH` file;
3. project `.reason/config.json` in the current working directory;
4. user config;
5. compiled defaults.

User config discovery uses `$REASON_HOME/config.json` when `REASON_HOME` is set, then
`$XDG_CONFIG_HOME/reason/config.json`, `%APPDATA%/reason/config.json` on Windows, or
`~/.config/reason/config.json`. Each config file must declare `"schema_version":
"reason-config-v1"`; unknown fields fail closed instead of being silently ignored.

Example:

```json
{
  "schema_version": "reason-config-v1",
  "run": {
    "provider": "google",
    "model": "gemini-3.5-flash-lite",
    "max_tokens": 1024,
    "format": "json"
  }
}
```

Use `reason schema config` for the current schema. `--no-config` ignores user/project config for a
hermetic invocation, which is recommended in reproducible CI unless an explicit config is part of
the job input. `--config` and `--no-config` are mutually exclusive.

A configured live provider can supply the default provider/model pair. If a CLI `--provider` changes
the configured provider, `--model` must also be supplied explicitly rather than accidentally reusing
a model configured for another provider. A live provider with no explicit or configured model fails
closed.

Provider secrets are deliberately **not fields in `reason-config-v1`**:

- Mistral: `MISTRAL_API_KEY`
- Google: `GEMINI_API_KEY`
- NVIDIA Hosted NIM: `NVIDIA_API_KEY`

The config parser rejects unknown secret-like fields such as `api_key`. Credentials remain
environment/provider-adapter inputs and are never serialized into the effective run configuration.

## Semantic runtime product surface

`reason semantic-check` is the supported product surface for the current semantic runtime. It is intentionally separate from `reason run`: a semantic finding is diagnostic evidence and never gains verification, trusted-evidence, or final-verdict authority merely because it was produced by the model-backed semantic runtime.

The input contract is `semantic-check-input-v1` and contains exactly a `request` plus a harness-owned `artifact`. Inspect it with:

```bash
reason schema semantic-check
```

Run the current profile:

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --format json
```

The current profile executes the stabilized materialization and deterministic typed-precondition path. The JSON result still exposes the exact machine runtime identity (`semantic-decidability-d3-v1`) for reproducibility. It includes the base decision, final semantic decision, decidability disposition, usage, model, and provider-attempt count. `force_abstain` can only make the semantic result more conservative.

The characterized rollback remains explicit. Legacy `--profile v3` is still accepted as an alias:

```bash
reason semantic-check \
  --input examples/semantic-check.json \
  --provider mistral \
  --model ministral-8b-latest \
  --profile rollback \
  --format json
```

Operational failure is separate from semantic outcome. In JSON mode a provider/runtime failure emits a `semantic-check` product envelope containing `operational_failure.failure_class` and returns exit 1; it is never converted into `finding`, `no_finding`, or `abstain`.

See [product roadmap](product-roadmap.md), [ADR-0001](adr/0001-interface-and-packaging-boundaries.md),
and [semantic runtime stabilization](semantic-runtime-stabilization.md).

## Design references

CLI ergonomics intentionally learn from mature terminal-first AI tools without copying their agent
semantics:

- OpenAI Codex treats non-interactive execution, machine-readable output, schema-constrained output,
  and layered/profile configuration as first-class automation concerns. Reasoning Harness adopts the
  separation of automation output from diagnostics, but not Codex's agent/sandbox authority model.
  See <https://github.com/openai/codex>.
- OpenCode exposes a dedicated non-interactive `run` path, stdin-friendly automation, JSON output,
  JSON-Schema-backed configuration, and explicit config discovery/precedence. Reasoning Harness uses
  these as UX references while keeping its own harness-owned evidence and verdict boundaries. See
  <https://opencode.ai/v2/docs/cli> and <https://opencode.ai/v2/docs/config>.

These projects are design references, not wire-compatibility targets. `reason` should stay narrower:
its product value is a predictable evidence-grounded reasoning harness, not another general-purpose
coding agent.
