# Release channels, readiness and version promotion

[日本語](release-policy.ja.md) | English

## Separate decisions

| Decision | Meaning |
| --- | --- |
| Version / compatibility | `0.x` means the public API may evolve. Existing machine contract IDs still retain their documented guarantees. It does not automatically require a GitHub prerelease label. |
| Release channel | A normal release has completed the required gates for its declared scope. A prerelease is a candidate with explicitly unfinished acceptance and a measurable promotion condition. |
| Distribution scope | Engine source, packaged Reason CLI and downstream adoption are separate deliverables. Engine-only source is a valid normal release scope. |
| Evidence / limitations | State exactly what deterministic, synthetic, live-provider, physical-device and user studies establish. A normal release does not turn unperformed studies into completed evidence. |

## Normal release requirements

Before publishing or promoting a normal release, its reviewed release PR must record:

1. Exact version, tag, source commit, CLI/Engine pair and intended supported scope.
2. Required checks for that scope and their exact-candidate results. Engine runtime changes require their scoped correctness/regression evidence; CLI distribution additionally requires its packaging, provenance, install/update/rollback and applicable physical acceptance. Ordinary CI alone does not replace a required live or device gate.
3. No unfinished required acceptance or known release-blocking defect within the declared scope. Optional research and explicitly deferred features are listed separately and cannot be described as shipped.
4. Compatibility changes, measured evidence limits and accepted residual limitations.
5. Green exact-head CI, verified merged source and unchanged historical tags, frozen observations and published assets.

Use a bare version such as `0.7.1` and set GitHub `prerelease=false`. Before v1.0, the documented v0.x compatibility policy still applies. A future v1.0 requires an explicit API/compatibility decision; changing a GitHub label does not establish v1.0 stability.

## Prerelease requirements and exit conditions

Use a candidate version such as `0.8.0-alpha.1`, `0.8.0-beta.1` or `0.8.0-rc.1`, with the same suffix in its package and tag. Set GitHub `prerelease=true`. Before publishing a candidate, its release notes must identify:

- the intended normal version and proposed scope;
- the specific unfinished required checks or known blockers, with tracking links;
- what result closes each item and qualifies the candidate for normal release;
- the tested commit and intended evaluation use / restrictions.

Do not use prerelease as an indefinite label for all 0.x versions, Engine source releases, CLI non-adoption or unmeasured optional hypotheses. A planned required gate that is still open remains a blocker until its result is accepted or an explicit reviewed scope decision explains its removal.

The normal sequence is `0.8.0-rc.1 → 0.8.0`, after the target scope's gates pass. The final package/tag gets its own immutable identity; candidate tags and observations stay intact. New implementation after an already published bare version requires a new version rather than replacing the old tag.

## GitHub metadata, assets and Latest

GitHub permits changing the prerelease label and notes while preserving a published immutable tag and assets. A classification correction for identical already accepted contents may therefore edit those fields without rebuilding, retagging or uploading replacement assets. Record the reason and evidence link in the notes. Never use a label change to bypass an unfinished required acceptance gate.

`Latest` is repository-wide and shared by Engine and CLI releases. Normal supported **CLI distributions** may explicitly set it; prereleases and **Engine-only source** publications request `latest=false`. That request does not guarantee the generic `/releases/latest` endpoint will exclude Engine releases: on 2026-10-13, after 0.7.1 became a normal release, this endpoint resolved to `engine-v0.7.1` despite that request (previously it returned 404).

Consumers must not use this mixed repository's generic Latest endpoint to choose a CLI binary. Existing CLI lifecycle discovery selects only published `reason-v*` versions without SemVer prerelease suffixes; native installers use an explicit/pinned CLI tag. Install/update also validates the attested CLI/Engine pair. Engine visibility in GitHub Latest does not adopt that Engine into a CLI distribution.

The CLI publication workflow derives the channel from the SemVer prerelease suffix, not the major number. Build metadata such as `+build-42` does not make a version a prerelease. The suffix selects the publication flag only; the release PR still owns scope and readiness approval.

## Engine 0.7 history and current decision

- `engine-v0.7.0` was published at `5c7682d988c18a5a7e7a1954bd2a369cad2e9070` with a legacy GitHub prerelease label and a bare package version. It is preserved as historical publication; it was not named `0.7.0-rc.1`.
- `engine-v0.7.1` at `344e708150eb158b55144a0228f154ef28f63d6d` contains real #512/#514/#515 corrections after 0.7.0. Its correct identity is a **0.7.1 patch**, not rewritten 0.7.0 contents. It is a **normal Engine-only source release** under this policy. Only its GitHub classification and explanatory notes are corrected; its tag, source and assets remain unchanged.
- 0.7.1 accepted scope: bounded successor v17/v30 corrections with deterministic/synthetic regression and preserved historical behavior. Release PR #517 passed 23/23 checks, its exact merged commit passed 22/22, and local locked workspace passed 1,468 tests / 0 failures / 6 existing ignored. Fixtures passed 20/20 with zero unsupported accepted claims; the original archive retained 13 exact hashes. No new live-provider, real-user or independently authenticated human acceptance is claimed.
- #488/#489/#491 remain explicitly deferred hypotheses, outside 0.7.1 scope. Published CLI remains 0.5.4 / Engine 0.6.1. #506's separate CLI 0.5.5 / Engine 0.6.1 candidate still requires physical Windows acceptance before publication; Engine source promotion does not complete it.
- Earlier Engine/CLI GitHub labels remain historical metadata unless separately reviewed. This policy does not retrospectively certify every older prerelease or rewrite immutable tagged documentation. The current GitHub notes and this policy explain the corrected classification.

References: [SemVer](https://semver.org/), [GitHub release management](https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository), [versioning](versioning.md), [0.7.1 scope and evidence](engine-0.7.1-release.md).
