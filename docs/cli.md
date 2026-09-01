# Native `reason` CLI

The native Rust `reason` executable is the first supported product surface for Reasoning Harness.
It invokes the same harness-owned validation, evidence, verification, decision, and finalization
boundaries used by the core runtime. Research binaries and `eval*` commands remain useful, but they
do not define the stable product contract.

## Product commands

- `reason run` — execute the harness-owned correctness process from a recorded candidate or live
  provider candidate generation.
- `reason verify` — deterministically validate a `ReasoningArtifact`.
- `reason schema` — print the versioned JSON Schema for a supported artifact or candidate wire
  contract.

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
  "cli_version": "0.0.1",
  "contracts": {
    "artifact": "reasoning-artifact-v1",
    "candidate": "reasoning-candidate-v1"
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

## Provider credentials and configuration

The current product surface keeps provider secrets out of repository/project configuration:

- Mistral: `MISTRAL_API_KEY`
- Google: `GEMINI_API_KEY`
- NVIDIA Hosted NIM: `NVIDIA_API_KEY`

Explicit CLI options currently control non-secret run configuration, with compiled defaults used when
an option is omitted. A layered user/project configuration file is a later CLI-1 hardening step; its
precedence will be compatibility-tested before it is called supported. Secrets will remain outside
ordinary checked-in config by default.

## D3 runtime note

`semantic-decidability-d3-v1` is the adopted default **semantic runtime profile**, but the ordinary
`reason run` command does not yet silently claim that it executes the D3 semantic runtime. Product
wiring is tracked separately in Issue #93 so the soft semantic runtime cannot accidentally gain
verification or final-verdict authority during CLI productization.

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
