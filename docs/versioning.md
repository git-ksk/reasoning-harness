# Product, engine, and contract versioning

Reasoning Harness uses separate version coordinates so product UX can evolve without implying that the reasoning/correctness engine changed.

## Coordinates

### 1. Reason CLI SemVer

`reasoning-harness-cli` owns the user-facing **Reason CLI** version.

This version changes for product-facing work such as commands, setup/configuration UX, credential handling, diagnostics, installers, packaging, and supported output behavior. Future CLI release tags use:

```text
reason-vX.Y.Z
```

The release workflow validates that `reason-vX.Y.Z` matches the `reasoning-harness-cli` package version.

### 2. Harness Engine SemVer

`reasoning-harness-core` owns the **Harness Engine** version.

This version is the coordinate for changes to the reasoning/correctness implementation: authority, admission, verification, finalization, answer-safety semantics, deterministic investigation control, and other engine behavior. Engine changes require their own evidence and promotion path; a CLI UX release does not advance the Engine version by itself.

`reasoning-harness-providers` has its own internal crate version. It is an implementation coordinate, not a third user-facing product version.

### 3. Machine contract identities

Wire/schema identifiers such as `reasoning-artifact-v1`, `reason-cli-output-v1`, `reason-config-v1`, and `reason-session-v1` remain independent compatibility coordinates. A package SemVer bump does not silently redefine an existing contract identity.

## Migration boundary

`v0.4.2` is the **final unified historical release**: Reason CLI 0.4.2 and Harness Engine 0.4.2 were released under the same `v0.4.2` tag.

After that boundary, the two lines move independently. The first planned general-use product line is:

```text
Reason CLI 0.5.0
Harness Engine 0.4.2
```

The CLI 0.5.0 productization milestone focuses on setup, secure credentials, diagnostics, and distribution while keeping Engine 0.4.2 semantics fixed. The separate **Harness Engine 0.5.0 — Verified Investigation Utility** milestone owns semantic/utility work that may change the engine and therefore requires fresh evaluation.

Historical unified tags `v0.1.0` through `v0.4.2` remain immutable. The release workflow can manually repackage those exact historical tags, but new automatic CLI releases use only the `reason-v*` namespace.
