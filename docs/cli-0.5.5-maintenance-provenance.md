# Reason CLI 0.5.5 maintenance release provenance

**Scope:** #506 fixes WinGet `doctor` alias detection. The CLI 0.5.5 candidate was built from the immutable `reason-v0.5.4` tag and cherry-picks the existing #504 repair. It intentionally pins Harness Engine 0.6.1, not the source Engine 0.7.0 currently on `main`.

The merge commit introducing this document has two parents. The first parent is the unchanged current `main` code tree, including Engine 0.7.0. The second parent is the explicitly reviewed `release/cli-0.5.5-engine-0.6.1` maintenance branch. The merge uses the `ours` *tree strategy* only to record patch-release ancestry for `.github/workflows/release-cli.yml` (`git merge-base --is-ancestor TAG main`); **the maintenance branch's historical Engine is not overlaid on `main`**. The tagged CLI release is built from the maintenance branch commit, never from the merge's `main` tree.

Before publishing `reason-v0.5.5`, require:

- 4-OS `cli-maintenance-acceptance` success on the **exact** candidate SHA, with Engine 0.6.1 and `reasoning-harness-core`/providers diff-free against `reason-v0.5.4`.
- `reason-v0.5.5` tag creation only after this second-parent merge is reviewed and merged, so the existing release workflow's main-ancestry condition passes.
- Signed archive/checksum/manifest provenance, installer verification and source-tag/CLI/Engine exact version identity. Existing `reason-v0.5.4` and `engine-v*` tags are immutable.
- Physical Windows WinGet `Links/reason.exe` diagnosis and published Microsoft community WinGet manifest validation before closing #506. Windows UAC approval for upgrading global `gh` must not be bypassed; direct CLI patch publication can be separated from third-party manifest approval.

No CLI adoption of Engine 0.7.0 is authorized by this ancestry-only merge.
