# Release provenance and installer trust

[日本語](release-provenance.ja.md) | English

**Status:** required for split Reason CLI releases (`reason-v0.5.0` and later). Historical unified releases through `v0.4.2` predate this contract and remain immutable.

Reason does not treat a checksum downloaded from the same release page as the only trust root. Split CLI releases use **keyless GitHub artifact attestations backed by GitHub OIDC/Sigstore**, plus SHA-256 as defense in depth.

## What is verified

For every native archive, the release workflow publishes a GitHub attestation. The supported installer accepts a `reason-v*` archive only when `gh attestation verify` proves all of the following:

- repository: `git-ksk/reasoning-harness`;
- signer workflow: `git-ksk/reasoning-harness/.github/workflows/release-cli.yml`;
- source ref: the exact requested `refs/tags/reason-vX.Y.Z`;
- provenance was not produced by a self-hosted runner;
- the attested subject digest matches the downloaded archive.

After provenance verification, the installer also verifies the archive against `SHA256SUMS` and checks the embedded `reason --version` before replacing an existing binary.

`reason-v*` installation therefore requires GitHub CLI **2.93.0 or newer**. Older verifier versions fail closed. The installer pins `GH_HOST=github.com` for attestation verification so an ambient enterprise-host setting cannot redirect the provenance lookup.

## Release manifest

Each split release includes an attested `release-manifest.json` (`reason-release-manifest-v1`) binding:

- repository and signer workflow;
- exact release tag;
- Reason CLI version;
- Harness Engine version;
- Git commit;
- every native archive plus installer/checksum metadata and its SHA-256 digest.

The manifest exists so update/rollback tooling can compare CLI and Engine identities before mutation and reject confused-channel or mismatched metadata. It is itself attested by the same release workflow.

## Key and trust-root rotation

Reason does not keep a long-lived project signing private key for this provenance layer. GitHub Actions obtains a short-lived OIDC identity and the attestation is recorded through GitHub/Sigstore infrastructure. This removes a static release-signing secret from the repository and CI configuration.

Verifier/trust-root rotation is handled by supported GitHub CLI/Sigstore trust metadata. Reason pins the repository, workflow, source ref, and runner policy rather than pinning one permanent leaf certificate or developer key. If that provenance service is unavailable or verification fails, installation/update fails closed; SHA-256 alone does not authorize a new split release.

## Immutable releases

Repository-level immutable releases are enabled. This applies to future releases and prevents post-publication release/tag mutation at the GitHub layer. Attestation verification remains mandatory; immutability is defense in depth, not a replacement for provenance.

## Historical releases

`v0.1.0` through `v0.4.2` were published before this provenance contract. They remain immutable and use their existing SHA-256 path. Reason never retrofits new attestations onto those historical tags and never presents them as equivalent to the `reason-v*` trust contract.

## macOS notarization and Windows code signing

Platform-native signing was evaluated separately from source provenance. The repository currently has no provisioned Apple Developer ID/notarization credential or Windows code-signing identity. Reason therefore does **not** embed a private platform-signing key in the repository and does not instruct users to disable Gatekeeper, SmartScreen, or other OS security controls.

Native signing/notarization can be layered onto the same artifacts later when externally managed signing identities are provisioned. It must not replace or weaken the repository/workflow/tag provenance check above.
