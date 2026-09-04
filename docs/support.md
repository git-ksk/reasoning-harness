# Product support and compatibility

`reason` is an external-preview CLI during v0.x. The native runtime remains the correctness owner; product support is defined around explicit CLI/data contracts rather than human-readable output or provider marketing guarantees.

## Supported product surfaces

The compatibility-tracked product commands are:

- `reason run`
- `reason verify`
- `reason semantic-check`
- `reason schema`

`reason eval`, `reason eval-resolution`, `reason eval-judges`, and the dedicated study binaries are research/evaluation surfaces. They may change more rapidly and are not part of the v0.1 product compatibility promise.

## Supported release platforms

Every product pull request runs the credential-free `reason` smoke suite on:

- Linux x86_64 (`ubuntu-24.04` runner class);
- macOS arm64 / Apple Silicon (`macos-15` runner class);
- macOS x86_64 / Intel (`macos-15-intel` runner class);
- Windows x86_64 (`windows-2025` runner class).

Tagged releases package one native `reason` executable for each of those platform classes. Other targets may compile, but are not release-supported until added to the matrix.

## Machine contract policy

The executable semver and machine contract identities are separate coordinates.

Current product identities include:

- `reason-cli-output-v1`
- `reasoning-artifact-v1`
- `reasoning-candidate-v1`
- `reason-config-v1`
- `semantic-check-input-v1`
- semantic runtime identity `semantic-runtime-identity-v1`

Within an existing output-contract identity, consumers should tolerate additive fields. Removing fields, changing field meaning, or changing authority/exit semantics requires a new relevant contract identity rather than a silent change. Config schemas fail closed on unknown fields by design; a config using a newly added field may therefore require the corresponding newer CLI.

Human-readable text is presentation, not a compatibility or correctness contract.

## v0.x breaking changes

Before v1.0, command flags or product schemas may still evolve. Intentional incompatible changes must:

1. be called out in `CHANGELOG.md`;
2. update the relevant machine contract identity when the wire meaning changes;
3. include migration guidance when an existing external workflow would otherwise break;
4. pass cross-platform product smoke before merge.

## Provider support posture

The provider-neutral runtime is the product boundary. Provider adapters normalize transport/API behavior but never become verification authority.

- Mistral, Google Gemini/AI Studio, and NVIDIA Hosted NIM adapters are implemented for live candidate generation.
- Mistral and Google-hosted Gemma are live-smoked for the supported current/rollback `semantic-check` product path. Product dogfood has completed on Ministral 3B/8B/14B, Mistral Small, Gemma 4 31B, and Gemini 3.1/3.5 Flash-Lite on the recorded workload. Completion does not imply equal utility; the recorded target-coverage matrix ranges from 0.00 to 1.00.
- A model/provider can still be incompatible with a specific structured-output protocol. Gemma 4 26B A4B and Nemotron 3.5 Lightning are recorded examples: each product dogfood run failed on invalid structured output after fallback and is treated as operational/protocol evidence, not a semantic score or fabricated abstention.
- Provider quotas, service availability, rate limits, model retirement, and model-specific output quality are external operational dependencies and are reported separately from harness correctness.

Provider credentials remain environment variables and are not accepted in `reason-config-v1`.

## Stability status

v0.1.0 remains the latest published preview and is not retroactively a v1.0 stability claim. Readiness evidence baseline commit `5c5701f77df9dd507c3949294708f8c07a054064` satisfies the documented v1.0 readiness gate: process-level compatibility is pinned across all four supported platforms, current CI/live runtime smoke is green, and the incident-analysis + architecture-review product workload has raw-vs-Harness acceptance evidence including the Ministral 8B #139 closeout. No v1.0 tag or stable-release promise exists until an explicit version/tag/release step is performed through the normal provenance workflow.


For the distinction between product terms, machine/runtime identifiers, and historical research labels, see [Terminology and naming](terminology.md).
