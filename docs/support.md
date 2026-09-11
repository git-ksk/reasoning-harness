# Product support and compatibility

`reason` is an external-preview CLI during v0.x. The native runtime remains the correctness owner; product support is defined around explicit CLI/data contracts rather than human-readable output or provider marketing guarantees.

## Supported product surfaces

The compatibility-tracked product surfaces are:

- direct natural-language `reason "TASK"` execution;
- `reason session start|inspect|resume|add|correct|fork|close`;
- `reason run`;
- `reason verify`;
- `reason semantic-check`;
- `reason schema`.

`reason eval`, `reason eval-resolution`, `reason eval-judges`, and the dedicated study binaries are research/evaluation surfaces. They may change more rapidly and are not part of the v0.1 product compatibility promise.

## Supported release platforms

Every product pull request runs the credential-free `reason` smoke suite on:

- Linux x86_64 (`ubuntu-24.04` runner class);
- macOS arm64 / Apple Silicon (`macos-15` runner class);
- macOS x86_64 / Intel (`macos-15-intel` runner class);
- Windows x86_64 (`windows-2025` runner class).

Tagged releases package one native `reason` executable for each of those platform classes. Other targets may compile, but are not release-supported until added to the matrix.

## Machine contract policy

Product compatibility uses three separate coordinates:

- **Reason CLI SemVer** — the user-facing product/distribution version owned by `reasoning-harness-cli`;
- **Harness Engine SemVer** — the reasoning/correctness implementation version owned by `reasoning-harness-core`;
- **machine contract identities** — wire/schema compatibility IDs such as `reason-cli-output-v1` and `reasoning-artifact-v1`.

`reasoning-harness-providers` has an internal crate version but is not exposed as a third user-facing product version. `v0.4.2` is the final unified CLI/Engine release; future CLI releases use `reason-vX.Y.Z` and can advance without changing the Engine. See [Product, engine, and contract versioning](versioning.md).

Current product identities include:

- `reason-cli-output-v1`
- `reasoning-artifact-v1`
- `reasoning-candidate-v1`
- `reason-config-v1`
- `semantic-check-input-v1`
- `reason-natural-output-v4`
- `reason-session-v1`
- exposed-text policy `harness-canonical-exposed-text-v1`
- bounded-investigation runtime `bounded-investigation-v1`
- semantic runtime identity `semantic-runtime-identity-v1`

Within an existing output-contract identity, consumers should tolerate additive fields. Removing fields, changing field meaning, or changing authority/exit semantics requires a new relevant contract identity rather than a silent change. Config schemas fail closed on unknown fields by design; a config using a newly added field may therefore require the corresponding newer CLI.

Human-readable wording remains presentation rather than a byte-stable compatibility contract. However, the natural-language product path now treats the authority of exposed factual text as a correctness boundary: under `harness-canonical-exposed-text-v1`, model renderer text is advisory and the exposed grounded/qualified text is Harness-constructed from accepted claims. See [Exposed-text safety](exposed-text-safety.md).

## v0.x breaking changes

Before v1.0, command flags or product schemas may still evolve. Intentional incompatible changes must:

1. be called out in `CHANGELOG.md`;
2. update the relevant machine contract identity when the wire meaning changes;
3. include migration guidance when an existing external workflow would otherwise break;
4. pass cross-platform product smoke before merge.

## Provider support posture

The provider-neutral runtime is the product boundary. Provider adapters normalize transport/API behavior but never become verification authority.

- Mistral, Google Gemini/AI Studio, NVIDIA Hosted NIM, and GroqCloud adapters are implemented for live candidate generation.
- Mistral and Google-hosted Gemma are live-smoked for the supported current/rollback `semantic-check` product path. Product dogfood has completed on Ministral 3B/8B/14B, Mistral Small, Gemma 4 31B, and Gemini 3.1/3.5 Flash-Lite on the recorded workload. Completion does not imply equal utility; the recorded target-coverage matrix ranges from 0.00 to 1.00.
- A model/provider can still be incompatible with a specific structured-output protocol. Gemma 4 26B A4B and Nemotron 3.5 Lightning are recorded examples: each product dogfood run failed on invalid structured output after fallback and is treated as operational/protocol evidence, not a semantic score or fabricated abstention.
- Provider quotas, service availability, rate limits, model retirement, and model-specific output quality are external operational dependencies and are reported separately from harness correctness.
- v0.4.2 release acceptance (`natural-language-e2e-v36-freeze`) passed independently on Mistral, Groq, Gemini 3.5 Flash-Lite, and Gemma 4 31B. Every required candidate row completed 13/13 with operational, generation, and correctness-boundary failures all `0`; Gemini's paired follow-up row improved tool selection `0.6 -> 1.0`, trigger exposure `0 -> 3`, and avoidable stalls `3 -> 0` at unchanged target recall `1.0`.

Provider credentials remain environment variables and are not accepted in `reason-config-v1`.

## Stability status

v0.4.2 is the current external-preview release. It preserves the v0.4.x product/authority foundation while adding deterministic safe acquisition precedence, generic Groq natural-language provider parity, structured planner/action hardening, and provider/evaluation resilience validated by immutable v36 release acceptance. Admission, authority, verification, finalization, answer safety, MCP non-promotion, and session replay remain Harness-owned. The documented v1.0 readiness gate is satisfied, but v0.4.2 intentionally remains a prerelease/v0.x compatibility promise rather than a stable v1.0 claim. A future v1.0 still requires an explicit version/tag/release decision through the normal provenance workflow.


For the distinction between product terms, machine/runtime identifiers, and historical research labels, see [Terminology and naming](terminology.md).
