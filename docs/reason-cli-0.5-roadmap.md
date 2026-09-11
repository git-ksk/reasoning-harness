# Reason CLI 0.5.0 general-use productization roadmap

Reason CLI 0.5.0 is the first product line that versions independently from Harness Engine. It productizes the accepted **Harness Engine 0.4.2** for ordinary terminal users without changing the v0.4.2 reasoning, authority, admission, verification, finalization, answer-safety, MCP non-promotion, or session replay boundaries.

`v0.4.2` remains the final unified historical release. The first planned split release is:

```text
Reason CLI 0.5.0
Harness Engine 0.4.2
```

The target is not to copy a coding agent. The target is to match the low-friction terminal ergonomics users now expect from mature AI CLIs: install without a toolchain, guided authentication, a useful no-argument interactive mode, easy continuation/resume, discoverable provider/model/config controls, actionable diagnostics, and a reversible update/uninstall lifecycle.

Tracking: milestone **Reason CLI 0.5.0 — General-use Productization** (#6), parent Issue #359.

## Product journeys

### First use

```text
install native Reason binary
        |
        v
reason setup
        |
        +--> choose provider/model
        +--> save credential in OS-native secret storage
        +--> bounded connectivity/readiness check
        |
        v
reason "TASK"
```

A first-time user must not need Rust, Cargo, shell-profile edits, plaintext credential files, or knowledge of internal Harness JSON contracts.

### Daily use

```text
reason
  -> interactive session
  -> follow-up
  -> Ctrl+C safely cancels current work
  -> exit

reason -c
  -> continue latest compatible session

reason -r <session>
  -> resume a selected session
```

The terminal may show Harness-owned lifecycle states such as `Planning`, `Acquiring`, `Verifying`, and `Finalizing`. These are progress states, not hidden chain-of-thought.

### Recovery

```text
reason doctor
reason auth status
reason config sources
reason mcp test <name>
reason update --check
```

A user-facing operational error should explain what failed, whether any result is trustworthy, and the next safe command to run.

## Phase 0 — Product/version boundary

- **#355 — complete:** split Reason CLI and Harness Engine SemVer coordinates.
- `v0.4.2` remains the immutable final unified tag.
- future CLI releases use `reason-vX.Y.Z`.
- machine contract identities remain independent compatibility coordinates.

## Phase 1 — Install, authenticate, and reach the first answer

### Distribution umbrella — #358

- **#371 P0:** one-command native installers for macOS, Linux, and Windows with checksum verification.
- **#372 P0:** update, explicit rollback, and uninstall lifecycle.
- **#375 P1:** Homebrew and winget channels after the canonical installer/update contract is stable.

### Setup/auth umbrella — #356

- **#361 P0:** OS-native secure credential backends: macOS Keychain, Windows Credential Manager, Linux Secret Service/keyring where available. No silent plaintext fallback.
- **#362 P0:** `reason auth login/list/status/logout`.
- **#367 P0:** provider/model discovery and default switching.
- **#363 P0:** `reason setup` wizard combining provider choice, secure auth, model selection, non-secret defaults, readiness check, and a first command.

Environment variables remain supported for CI, containers, remote shells, and servers. Their precedence relative to OS-stored credentials must be deterministic and documented.

## Phase 2 — Everyday interactive terminal UX

- **#364 P0:** `reason` with no arguments starts an interactive REPL instead of returning a usage error.
- **#365 P0:** `-c/--continue`, `-r/--resume`, session listing/picking, and safe checkpoint persistence over the existing typed session runtime.
- **#366 P1:** `reason config list/get/set/unset/path/sources`; secrets remain rejected from `reason-config-v1`.
- **#369 P1:** high-level progress/retry status and deterministic safe cancellation. JSON/piped mode remains quiet unless explicitly requested.
- **#373 P1:** self-teaching help, examples, and shell completions for zsh/bash/fish/PowerShell.

The existing one-shot `reason "TASK"` and JSON automation surfaces remain supported.

## Phase 3 — External acquisition UX

- **#368 P1:** guided read-only MCP management: `reason mcp add/list/inspect/test/remove`.

This is a configuration and visibility improvement only. MCP output remains acquisition data rather than authority, write-capable or ambiguous capabilities remain fail-closed, and the supported Engine 0.4.2 MCP correctness boundary is unchanged.

## Phase 4 — Diagnostics and recovery

- **#357 P0:** `reason doctor` reports Reason CLI and Harness Engine versions separately; checks installation/config sources, credential presence without secret values, provider/model readiness, OS credential-store availability, session paths, and configured MCP readiness; supports human and machine-readable diagnostics.
- **#370 P0:** recovery-oriented operational errors for credential, model/protocol, quota/rate-limit/outage, structured-output, config, MCP, session, and update/version failures.

Typed machine failures remain separate from epistemic `unknown`. Human-friendly remediation must not collapse operational failure into semantic uncertainty.

## Phase 5 — Fresh-install release gate

**#374 P0** is the CLI 0.5.0 acceptance gate. On supported platforms, it must cover:

1. install from a published native artifact without Rust;
2. setup from an empty user configuration/home;
3. secure credential storage or an explicit typed unsupported-headless path;
4. one-shot execution using configured defaults;
5. interactive execution with a follow-up;
6. continue/resume of a persisted session;
7. provider/model/config inspection and switching;
8. `reason doctor` and separate CLI/Engine version reporting;
9. at least one expected operational failure with actionable recovery;
10. update/check and uninstall/retain-data behavior;
11. existing JSON/non-interactive contract smoke;
12. zero credential leakage through stdout, stderr, diagnostics, config, or session artifacts.

`reason-v0.5.0` is not tagged until this gate is green and no unresolved P0 product blocker remains.

## Priority model

### P0 — required for CLI 0.5.0

#361, #362, #363, #364, #365, #367, #370, #371, #372, #357, and #374.

A P0 item blocks the general-use release unless it is explicitly re-scoped with a documented replacement acceptance path.

### P1 — product parity and polish

#366, #368, #369, #373, and #375.

P1 work is expected for a polished product line but does not automatically block the first safe 0.5.0 release unless implementation exposes a P0 usability, safety, or supportability gap.

## Explicit non-goals for this milestone

- changing Harness Engine 0.4.2 reasoning/authority semantics;
- converting Reason into a write-capable coding agent;
- adding an approval system for destructive tools that do not exist in the supported product boundary;
- storing provider secrets in `reason-config-v1`, project configuration, session artifacts, or evidence;
- weakening fail-closed MCP/read-only checks for convenience;
- retroactively changing v0.4.2 or frozen evaluation evidence.

Semantic/utility changes remain in the separate **Harness Engine 0.5.0 — Verified Investigation Utility** milestone (#4) and require fresh evidence before adoption.
