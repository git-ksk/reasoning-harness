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

## Phase 2 — Everyday interactive terminal UX ✅ COMPLETE

**Completed 2026-09-12.** All Phase 2 P0/P1 items are implemented and merged through `main` commit `c0233728cedea4f0d5cc558e6bff220df8d937d0`; the merge-post `ci`, `cli-platform-smoke`, `credential-store-smoke`, and `lifecycle-smoke` workflows all passed.

- **#364 P0 — implemented:** bare `reason` on a human TTY starts an in-memory interactive REPL; JSON/non-TTY/piped automation remains non-interactive. `/add <path>` reuses the existing untrusted context path, multiline prompts use a trailing `\`, previous exposed exchanges are retained only as untrusted in-memory conversation context, and no shell-style history file is written. Managed persistence remains owned by #365/#381.
- **#365 P0 — implemented:** `-c/--continue`, `-r/--resume [id]`, TTY picker, managed `reason session list`, stable ids/titles, and successful-turn checkpoint persistence over the existing typed `SessionFile`/`ReasoningThread` runtime. A product-layer conversation wrapper preserves Core immutable task identity; prior exposed turns remain `untrusted_context`. Store concurrency/recovery/migration remains #381.
- **#381 P0 — implemented:** managed-session store locking, in-memory digest optimistic concurrency, crash-safe atomic replacement/temp recovery, corrupt/incompatible listing, and a no-destructive-migration 0.5.x rollback policy.
- **#379 P0 — implemented:** private managed-state permissions/ownership checks, explicit `--ephemeral`, managed session export/delete/scoped purge, deterministic uninstall retention/purge boundaries, and provider/MCP/resolver outbound-data disclosure with no first-party telemetry by default.
- **#378 P0 — implemented:** human output now separates finalized answer, canonical verified facts, unresolved/qualified propositions, supporting evidence/source provenance, untrusted context, and typed acquisition/rejection notes. Interactive `/status` and `/evidence` reconstruct the same view from typed checkpoints; JSON is unchanged and no hidden reasoning or unsupported model prose gains authority.
- **#380 P0 — implemented:** additive human/JSON provider/resolver usage, interactive `/usage`, rollback-safe cumulative managed-session accounting, hard `max_model_calls` / reported output-token / measurable total-token guards with typed operational exhaustion, and currency estimates only from explicit operator pricing provenance.
- **#366 P1 — implemented:** `reason config list/get/set/unset/path/sources` exposes effective safe run defaults with per-key provenance and precedence, while editing only the user layer. Secrets and authority-bearing resolver/MCP/trusted-verifier settings remain outside this surface and rejected by `reason-config-v1`.
- **#369 P1 — implemented:** human full-TTY execution shows only Harness-owned `Planning` / `Acquiring` / `Verifying` / `Finalizing` phases, bounded provider retry/elapsed status, and optional secret/CoT-free `--verbose` operational details. JSON/piped/non-TTY mode stays quiet; Ctrl+C returns typed `cancelled` without committing an incomplete ordinary turn as a successful checkpoint.
- **#373 P1 — implemented:** self-teaching help, copy-paste examples, and stdout-only shell completions for zsh/bash/fish/PowerShell; common natural-language workflows are prioritized while advanced/research surfaces remain explicitly discoverable.
- **#383 P1 — implemented:** centralized terminal presentation policy with explicit `--plain`, `NO_COLOR`, `TERM=dumb`, and non-TTY detection; plain mode suppresses decorative progress while preserving prompts/Ctrl+C, JSON/piped output stays decoration-free, and human rendering avoids width-based byte truncation so Unicode remains intact.

The existing one-shot `reason "TASK"`, repeatable `--file`, piped stdin context, and JSON automation surfaces remain supported.

## Phase 3 — External acquisition UX and process isolation ⚠️ CORE IMPLEMENTED / 0.5.0 HARDENING OPEN

- **#387 P0 — implemented:** current external-command, MCP v3 (plus v2 compatibility), and trusted-command subprocesses start from a documented minimal cross-platform environment instead of inheriting ambient provider/developer secrets. Explicit future integration credentials have a scoped injection boundary; secret-valued project config and arbitrary ambient-variable inheritance remain unavailable. Historical frozen `mcp_readonly_v1` remains untouched. See [Local subprocess environment isolation](reason-subprocess-isolation.md).
- **#368 P1 — implemented:** guided management for the single active local read-only MCP acquisition source via `reason mcp add/list/inspect/test/remove`; add writes only non-secret user configuration, inspect/list hide argument values, and `test` performs negotiation plus `tools/list` read-only verification without invoking the selected tool.
- **#386 P1 — implemented:** secure remote MCP `2026-07-28` Streamable HTTP support with read-only stateless discovery/acquisition, OAuth authorization-code + PKCE login/status/logout, browser and `--no-browser` flows, issuer/state/resource binding, native OS credential-store persistence, HTTPS enforcement, and project-trust gating. Tokens remain outside `reason-config-v1` and resolver authority.

### 0.5.0 hardening required before Phase 3 is considered fully complete

- **#414 P1:** discover MCP Protected Resource Metadata and authorization-server/OIDC metadata instead of requiring hand-authored OAuth endpoints; validate issuer/resource relationships fail-closed.
- **#415 P1:** distinguish OAuth `insufficient_scope` from generic permission denial and provide explicit, non-silent scope step-up recovery.
- **#416 P1 — implemented:** remote HTTP readiness/acquisition now shares Reason's safe Ctrl+C cancellation token; pending `tools/list` / `tools/call` work is dropped promptly, the joined worker exits before the command returns, and CLI cancellation remains the existing typed `cancelled` operational outcome rather than semantic `unknown`.
- **#419 P1 — implemented:** recursively rejects secret-bearing nested MCP `fixed_arguments` across local, remote, and investigation MCP configuration without echoing secret values, preserving the documented non-secret configuration boundary for hand-authored low-level config.

### Tracked compatibility / lifecycle follow-ups

- **#417:** recognize MCP 2026 `input_required` mid-tool transitions and return a typed fail-closed compatibility outcome until interactive elicitation is supported.
- **#418:** make remote MCP config removal and native OAuth credential cleanup behavior explicit and machine-readable.

The already-merged Phase 3 core remains safe-by-default: MCP output is acquisition data rather than authority, write-capable or ambiguous capabilities remain fail-closed, and the supported Engine 0.4.2 MCP correctness boundary is unchanged. The open hardening items above are interoperability, cancellation, and secret-boundary work; they do not authorize MCP output or weaken verification.

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
