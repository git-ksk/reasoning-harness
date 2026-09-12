# Reason CLI 0.5.0 general-use productization roadmap

Reason CLI 0.5.0 is the first product line that versions independently from Harness Engine. It productizes the accepted **Harness Engine 0.4.2** for ordinary terminal users without changing the v0.4.2 reasoning, authority, admission, verification, finalization, answer-safety, MCP non-promotion, or session replay boundaries.

`v0.4.2` remains the final unified historical release. The first planned split release is:

```text
Reason CLI 0.5.0
Harness Engine 0.4.2
```

The target is not to copy a coding agent. The target is to match the low-friction terminal ergonomics users now expect from mature AI CLIs while preserving Reason's stricter product boundary: install without a toolchain, explicit project trust, guided secure authentication, a useful no-argument interactive mode, understandable verified evidence, easy continuation/resume, visible provider usage, discoverable provider/model/config/MCP controls, actionable diagnostics, private local state, and a verifiable reversible update/uninstall lifecycle.

Tracking: milestone **Reason CLI 0.5.0 — General-use Productization** (#6), parent Issue #359.

## Product journeys

### First use

```text
install verified native Reason binary
        |
        v
enter project -> executable/authority config stays inactive until trusted
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

A first-time user must not need Rust, Cargo, shell-profile edits, plaintext credential files, knowledge of internal Harness JSON contracts, or a security bypass for an unsigned/tampered binary.

### Daily use

```text
reason
  -> interactive session
  -> follow-up / add untrusted file context
  -> inspect verified facts / unresolved items / sources
  -> Ctrl+C safely cancels current work
  -> exit

reason -c
  -> continue latest compatible session

reason -r <session>
  -> resume a selected session
```

The terminal may show Harness-owned lifecycle states such as `Planning`, `Acquiring`, `Verifying`, and `Finalizing`. These are progress states, not hidden chain-of-thought. Normal human output should explain verified facts, uncertainty, and admitted source provenance without turning model prose into authority.

### Recovery

```text
reason doctor
reason auth status
reason config sources
reason mcp test <name>
reason update --check
```

A user-facing operational error should explain what failed, whether the task executed, whether any result is trustworthy, and the next safe command to run.

## Phase 0 — Product/version boundary

- **#355 — complete:** split Reason CLI and Harness Engine SemVer coordinates.
- `v0.4.2` remains the immutable final unified tag.
- future CLI releases use `reason-vX.Y.Z`.
- machine contract identities remain independent compatibility coordinates.

## Phase 1 — Install, trust, authenticate, and reach the first answer

### Distribution umbrella — #358

- **#371 P0 — completed:** one-command native installers for macOS, Linux, and Windows.
- **#382 P0 — completed:** release provenance, signing/notarization where appropriate, and trusted installer/updater verification. SHA-256 remains useful but is not the sole trust root.
- **#372 P0 — completed:** provenance-verified update, explicit rollback, and retention-by-default uninstall lifecycle.
- **#375 P1:** Homebrew and winget channels after the canonical installer/update contract is stable.

### Project trust — #377 (completed)

Project `.reason/config.json` may contain executable or authority-bearing acquisition configuration. Such settings must not activate merely because the current directory came from an untrusted clone. Trust must be explicit, inspectable, revocable, path/canonical-identity aware, and non-interactive-safe. High-risk executable/authority configuration changes after trust must invalidate or re-confirm the relevant trust fingerprint rather than inheriting a permanent blanket approval. `reason trust status/add/list/revoke`, canonical project identity, direct-executable SHA-256 binding, and a prohibition on project-owned `trusted_command` authority are implemented.

### Setup/auth umbrella — #356

- **#361 P0 — completed:** OS-native secure credential backends: macOS Keychain, Windows Credential Manager, Linux Secret Service/keyring where available. No silent plaintext fallback. Secret entry must avoid argv/shell-history exposure.
- **#362 P0 — completed:** `reason auth login/list/status/logout`, secure replacement/rotation, and a storage identity that does not preclude future named accounts.
- **#367 P0 — completed:** provider/model discovery and default switching; no silent model fallback.
- **#363 P0 — completed:** `reason setup` wizard combining provider choice, secure auth, model selection, non-secret defaults, bounded readiness check, billable-check disclosure where applicable, and a first command.

Environment variables remain supported for CI, containers, remote shells, and servers. Their precedence relative to OS-stored credentials must be deterministic and documented.

## Phase 2 — Everyday interactive terminal UX

- **#364 P0 — implemented:** bare `reason` on a human TTY starts an in-memory interactive REPL; JSON/non-TTY/piped automation remains non-interactive. `/add <path>` reuses the existing untrusted context path, multiline prompts use a trailing `\`, previous exposed exchanges are retained only as untrusted in-memory conversation context, and no shell-style history file is written. Managed persistence remains owned by #365/#381.
- **#365 P0:** `-c/--continue`, `-r/--resume`, session listing/picking, and safe checkpoint persistence over the existing typed session runtime.
- **#381 P0:** managed-session store locking/optimistic concurrency, crash recovery, and compatibility/migration across supported CLI updates/rollback.
- **#379 P0:** private local permissions, retention/purge semantics, explicit ephemeral/no-persist use, and disclosure of what data leaves the machine for providers/MCP/resolvers.
- **#378 P0:** human-readable answer presentation that exposes verified facts, unresolved/qualified items, and admitted evidence/source provenance without hidden reasoning or unsupported model prose.
- **#380 P0:** provider attempts/token usage where reported plus enforceable provider-neutral per-run/session budget guards; currency cost is estimated only when pricing provenance is known.
- **#366 P1:** `reason config list/get/set/unset/path/sources`; secrets remain rejected from `reason-config-v1`.
- **#369 P1:** high-level progress/retry status and deterministic safe cancellation. JSON/piped mode remains quiet unless explicitly requested.
- **#373 P1:** self-teaching help, examples, and shell completions for zsh/bash/fish/PowerShell.
- **#383 P1:** accessible/plain terminal rendering, `NO_COLOR`, screen-reader/minimal-terminal behavior, and width/unicode-safe presentation.

The existing one-shot `reason "TASK"`, repeatable `--file`, piped stdin context, and JSON automation surfaces remain supported.

## Phase 3 — External acquisition UX and process isolation

- **#387 P0:** local external-command, MCP, and trusted-verifier subprocesses must start from a minimal scoped environment rather than inheriting unrelated ambient provider/developer secrets.
- **#368 P1:** guided read-only MCP management: `reason mcp add/list/inspect/test/remove`.
- **#386 P1:** secure remote MCP/Streamable HTTP OAuth lifecycle after the local read-only management path, including headless/no-browser auth where practical.

This is a configuration, transport, isolation, and visibility improvement only. MCP output remains acquisition data rather than authority, write-capable or ambiguous capabilities remain fail-closed, and the supported Engine 0.4.2 MCP correctness boundary is unchanged.

## Phase 4 — Diagnostics and operational recovery

- **#357 P0:** `reason doctor` reports Reason CLI and Harness Engine versions separately; checks installation/config sources, credential presence without secret values, provider/model readiness, OS credential-store availability, managed-session paths, project trust, and configured MCP readiness; supports human and machine-readable diagnostics.
- **#370 P0:** recovery-oriented operational errors for credential, model/protocol, quota/rate-limit/outage, structured-output, config/trust, MCP, session, update/version, and distribution-integrity failures.
- **#385 P1:** explicit model retirement/fallback policy. Default behavior never silently changes provider/model execution identity; any future fallback chain records the actual provider/model used.
- **#384 P1:** proxy/custom-CA/headless network diagnostics without recommending insecure TLS bypass.

Typed machine failures remain separate from epistemic `unknown`. Human-friendly remediation must not collapse operational failure into semantic uncertainty.

## Phase 5 — Fresh-install release gate

**#374 P0** is the CLI 0.5.0 acceptance gate. On supported platforms, it must cover:

1. install from a published native artifact without Rust and verify its trusted release identity;
2. an untrusted project cannot activate executable/MCP/trusted-verifier behavior before explicit trust;
3. setup from an empty user configuration/home;
4. secure credential storage or an explicit typed unsupported-headless path, with no secret-valued argv requirement;
5. one-shot execution using configured defaults;
6. interactive execution with a follow-up and ordinary file/context addition;
7. human output shows understandable verified facts, unresolved/qualified state, and admitted source provenance;
8. continue/resume of a persisted session;
9. concurrent/crash-interrupted session cases cannot silently corrupt or overwrite managed state;
10. provider/model/config inspection and switching;
11. visible provider usage plus safe budget exhaustion behavior;
12. `reason doctor` and separate CLI/Engine version reporting;
13. at least one expected operational failure with actionable recovery;
14. private local data permissions, ephemeral/no-persist behavior, and scoped purge/retain-data semantics;
15. local MCP/resolver/verifier subprocesses cannot observe unrelated ambient secret sentinels;
16. update/check, explicit rollback, and uninstall/retain-data behavior with artifact verification;
17. existing JSON/non-interactive contract smoke;
18. zero credential/secret leakage through stdout, stderr, diagnostics, config, session/history, or subprocess environment.

`reason-v0.5.0` is not tagged until this gate is green and no unresolved P0 product blocker remains.

## Priority model

### P0 — required for CLI 0.5.0

#357, #361, #362, #363, #364, #365, #367, #370, #371, #372, #374, #377, #378, #379, #380, #381, #382, and #387.

A P0 item blocks the general-use release unless it is explicitly re-scoped with a documented replacement acceptance path.

### P1 — product parity and polish

#366, #368, #369, #373, #375, #383, #384, #385, and #386.

P1 work is expected for a polished product line but does not automatically block the first safe 0.5.0 release unless implementation exposes a P0 usability, safety, or supportability gap.

## Explicit non-goals for this milestone

- changing Harness Engine 0.4.2 reasoning/authority semantics;
- converting Reason into a write-capable coding agent or background-agent platform;
- adding an approval system for destructive tools that do not exist in the supported product boundary;
- silently switching models/providers to recover from availability or retirement;
- storing provider/integration secrets in `reason-config-v1`, project configuration, session/history artifacts, evidence, or normal CLI argv;
- inheriting the entire parent environment into local MCP/resolver/verifier subprocesses;
- weakening fail-closed MCP/read-only checks for convenience;
- adding rich PDF/image/browser ingestion as a release requirement when text files/stdin plus explicit acquisition/MCP cover the 0.5.0 product path;
- cloud account/session synchronization or built-in hosted telemetry as a prerequisite for local use;
- retroactively changing v0.4.2 or frozen evaluation evidence.

Semantic/utility changes remain in the separate **Harness Engine 0.5.0 — Verified Investigation Utility** milestone (#4) and require fresh evidence before adoption.
