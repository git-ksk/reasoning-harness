# Reason local privacy and retention

Reason CLI 0.5.0 separates credentials, config/trust state, managed conversation sessions, explicit low-level session files, and diagnostic outputs. Harness Engine 0.4.2 authority semantics are unchanged.

## Local storage

Reason-managed config, project trust, and interactive session directories are private user state. On Unix, Reason verifies ownership, sets managed directories to `0700`, and managed state/session files to `0600`. A state path owned by another uid is refused rather than silently adopted. On Windows, Reason uses the current user's application/config location and inherited account ACLs; a custom `REASON_HOME` should itself be private to the user.

Managed interactive sessions contain prompts, snapshots of `/add` file content, Harness-owned typed turn/checkpoint state, and exposed final answer text. They do **not** contain provider credentials or hidden chain-of-thought. Reason does not create a shell-style prompt-history file.

`reason --ephemeral` starts interactive use without writing managed session/history state. It cannot be combined with `-c/--continue`, `-r/--resume`, or `--diagnostic-trace`. One-shot `reason "TASK"` is already non-persistent unless the user explicitly requests a persistent output such as `--diagnostic-trace`.

## Retention and deletion

There is no surprise automatic retention deletion in 0.5.0. The user controls managed session lifecycle explicitly:

- `reason session list` lists compatible sessions and separately identifies corrupt/incompatible entries.
- `reason session export ID --out PATH` creates a new private JSON backup and never removes the source.
- `reason session delete ID --dry-run` previews deletion; mutation requires interactive confirmation or `--yes`.
- `reason session purge --older-than-days N --dry-run` scopes retention cleanup by age.
- `reason session purge --all --dry-run` previews complete managed-session cleanup, including corrupt/incompatible managed JSON entries.
- `reason uninstall` retains config/trust/session state and native credentials by default. `reason uninstall --purge-data` removes only Reason-managed config/trust/interactive-session paths. Explicit-path `reason session ... --store PATH` files remain untouched. Credentials require the separate `--purge-credentials` opt-in.

## What leaves the machine

For model-backed execution, the selected provider receives the model request needed for the task, including the task and untrusted context made available to candidate/final rendering. Harness-owned trusted verification receipts are not promoted into model authority merely by being sent; the Harness remains the authority boundary.

Configured MCP or external resolvers receive the bounded request/arguments required for their configured acquisition lane. A trusted local verifier/subprocess receives only the data defined by its explicit command contract. Project trust still gates executable/network acquisition settings.

Provider credentials are resolved from the native OS credential store or explicit environment source and are never written to Reason config, managed sessions/history, diagnostic traces, stdout, or stderr. Interactive setup uses a no-echo password prompt; `--credential-stdin` is a bounded one-line non-interactive path. Reason itself does not persist terminal input history for setup/auth.

Reason has no first-party telemetry or crash-report upload enabled by default. If such a facility is added later it requires a separately documented policy and explicit opt behavior.
