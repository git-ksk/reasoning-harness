# Reason CLI 0.5.5 maintenance release provenance

**Scope:** #506 fixes WinGet `doctor` alias detection. The CLI 0.5.5 candidate was built from the immutable `reason-v0.5.4` tag and cherry-picks the existing #504 repair. It intentionally pins Harness Engine 0.6.1, not the source Engine 0.7.0 currently on `main`.

The merge commit introducing this document has two parents. The first parent is the unchanged current `main` code tree, including Engine 0.7.0. The second parent is the explicitly reviewed `release/cli-0.5.5-engine-0.6.1` maintenance branch. The merge uses the `ours` *tree strategy* only to record patch-release ancestry for `.github/workflows/release-cli.yml` (`git merge-base --is-ancestor TAG main`); **the maintenance branch's historical Engine is not overlaid on `main`**. The tagged CLI release is built from the maintenance branch commit, never from the merge's `main` tree.

Before publishing `reason-v0.5.5`, require:

- 4-OS `cli-maintenance-acceptance` success on the **exact** candidate SHA, with Engine 0.6.1 and `reasoning-harness-core`/providers diff-free against `reason-v0.5.4`.
- `reason-v0.5.5` tag creation only after this second-parent merge is reviewed and merged, so the existing release workflow's main-ancestry condition passes.
- Signed archive/checksum/manifest provenance, installer verification and source-tag/CLI/Engine exact version identity. Existing `reason-v0.5.4` and `engine-v*` tags are immutable.
- Physical Windows WinGet `Links/reason.exe` diagnosis and published Microsoft community WinGet manifest validation before closing #506. Windows UAC approval for upgrading global `gh` must not be bypassed; direct CLI patch publication can be separated from third-party manifest approval.

No CLI adoption of Engine 0.7.0 is authorized by this ancestry-only merge.

## Release-channel gate follow-up (2026-10-14)

After the initial ancestry marker (#511), the pinned maintenance line gained a
reviewed release-channel policy/test backport via #527, integrated as
`5a8fe68639cc854a77120419823231dcd4e36c34`. This new, second-parent-only
link records that exact release candidate in `main` ancestry **without importing
any CLI 0.5.5 or Engine 0.6.1 files into the newer `main` source tree**.

Candidate metadata remains Reason CLI 0.5.5 / Harness Engine 0.6.1 /
providers 0.4.2. The GitHub SemVer publication flags now treat `0.5.5` as a
normal release while preserving signed manifests and attestation checks.
This marker is **not publication approval**: required physical Windows WinGet
alias/path and update acceptance remains blocked by device connectivity. Do not
mint `reason-v0.5.5` or close #506 until acceptance and exact-candidate release
gates are complete.

## Final WinGet Clippy patch ancestry (2026-10-14)

The final CLI 0.5.5 / Engine 0.6.1 candidate `ec6e55ff95b8364f9a5d09b8bbfe0ecc0c9d19d7` includes PR #529, a Windows-only `clippy::needless_return` cleanup in the OAuth browser opener. Its PR gate passed 22/22 checks, and local Rust 1.88.0 strict Windows Clippy, fmt, CLI tests (203 passed, three ignored) and macOS Clippy, fmt, CLI tests (208 passed, three ignored) passed. The prior candidate `5a8fe68639cc854a77120419823231dcd4e36c34` remains preserved in history.

The final candidate is recorded only as an additional merge parent using `git merge -s ours`: the source files on `main` retain Engine 0.7.2 and are **not** replaced with the maintenance branch's Engine 0.6.1. Publication still requires the exact-head four-OS maintenance workflow and main-ancestry gate, followed by release artifact/attestation verification. Do not treat this ancestry marker as WinGet official index acceptance or as permission to close #506 before physical 0.5.4→0.5.5 upgrade E2E.
