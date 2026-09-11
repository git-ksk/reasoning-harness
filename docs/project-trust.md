# Project trust

[日本語](project-trust.ja.md) | English

**Status:** implemented on the Reason CLI 0.5.0 development line. The tagged `v0.4.2` release predates these `reason trust` commands.

Reason can read `.reason/config.json` from the current project. Because that file may configure subprocesses, read-only MCP acquisition, or investigation capabilities, **entering an untrusted repository must not activate those capabilities automatically**.

## What is gated

Project configuration is split by effect:

- a project config that contains only ordinary non-secret defaults such as provider, model, token limit, and output format can be read without project trust;
- `resolution.external_command`, `resolution.mcp_readonly`, and `resolution.investigation` require explicit project trust before they can be merged into the effective config;
- `resolution.trusted_command` is **never** authorized from project config, even after trusting the folder. Hard-verifier authority must come from user config or an explicit `--config PATH` chosen by the caller.

An untrusted or stale high-risk project config fails closed. Reason does not open an interactive approval prompt during normal execution, so non-interactive jobs do not deadlock.

## Commands

```bash
reason trust status
reason trust add
reason trust list
reason trust revoke
reason trust revoke --all
```

`status`, `add`, and `revoke` use the current working directory by default. Pass `--project DIR` to inspect or change another folder explicitly. Add `--format json` for automation.

`reason trust add` is the explicit approval step. It reports the canonical project path, current fingerprint, and resolved executable/acquisition programs before persisting trust.

## What the trust record is bound to

The v1 trust record is bound to:

1. the canonical project directory;
2. the normalized high-risk `resolution` configuration;
3. canonical paths for directly configured executable programs;
4. SHA-256 identities of those direct executable files.

Changing high-risk configuration or replacing a directly configured executable at the same path makes the record `stale`; Reason refuses to activate the project acquisition configuration until it is reviewed and trusted again. Moving the repository creates a different project identity. Symlink aliases to the same canonical project resolve to the same identity, while a `.reason/config.json` symlink that escapes the project root is rejected.

Project trust is an **activation boundary**, not a sandbox or a recursive signature over every file a trusted executable might later read. The Harness evidence/authority boundary still applies after acquisition is activated.

## Storage

Trust records use `reason-project-trust-v1` and are stored beside the user config as `project-trust.json` (for example `$REASON_HOME/project-trust.json` when `REASON_HOME` is set). Writes are staged and committed atomically; on Unix the trust file is created with user-only permissions.

`--no-config` remains the hermetic escape hatch: it ignores both user and project config and therefore does not consult project trust.
